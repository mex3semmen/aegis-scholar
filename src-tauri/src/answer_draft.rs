use crate::audit::{append_audit_event, AuditEvent, AuditEventType};
use crate::corpus_paths::CorpusPaths;
use crate::errors::{AegisError, AegisResult};
use crate::evidence::{EvidenceItem, EvidencePack, EvidenceService};
use crate::locators::CitationLocator;
use crate::source_metadata::IngestionStatus;
use crate::source_registry::SourceRegistry;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerDraft {
    pub answer_draft_id: String,
    pub evidence_pack_id: String,
    pub source_id: String,
    pub version_id: String,
    pub query: String,
    pub created_at: DateTime<Utc>,
    pub draft_mode: AnswerDraftMode,
    pub claim_count: usize,
    pub unsupported_count: usize,
    pub claims: Vec<DraftClaim>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnswerDraftMode {
    EvidenceOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftClaim {
    pub claim_id: String,
    pub status: DraftClaimStatus,
    pub text: String,
    pub evidence_ids: Vec<String>,
    pub chunk_ids: Vec<String>,
    pub locators: Vec<CitationLocator>,
    pub confidence: DraftClaimConfidence,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DraftClaimStatus {
    Supported,
    NeedsEvidence,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DraftClaimConfidence {
    Mechanical,
    MissingEvidence,
}

pub struct AnswerDraftService {
    paths: CorpusPaths,
}

impl AnswerDraftService {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { paths: CorpusPaths::new(root) }
    }

    pub fn build_answer_draft(&self, source_id: &str, evidence_pack_id: &str) -> AegisResult<AnswerDraft> {
        self.paths.ensure_layout()?;
        let result = (|| -> AegisResult<AnswerDraft> {
            if source_id.trim().is_empty() {
                return Err(AegisError::AnswerDraftInvalidId);
            }
            validate_answer_draft_id(evidence_pack_id)?;
            let evidence = EvidenceService::new(self.paths.root.clone()).read_evidence_pack(source_id, evidence_pack_id)?;
            if evidence.items.is_empty() {
                return Err(AegisError::AnswerDraftEmpty);
            }
            let claims = evidence
                .items
                .iter()
                .map(mechanical_claim_from_item)
                .collect::<Vec<_>>();
            let answer_draft_id = deterministic_answer_draft_id(&evidence, &claims);
            let draft = AnswerDraft {
                answer_draft_id: answer_draft_id.clone(),
                evidence_pack_id: evidence.evidence_pack_id.clone(),
                source_id: evidence.source_id.clone(),
                version_id: evidence.version_id.clone(),
                query: evidence.query.clone(),
                created_at: Utc::now(),
                draft_mode: AnswerDraftMode::EvidenceOnly,
                claim_count: claims.len(),
                unsupported_count: claims
                    .iter()
                    .filter(|claim| claim.status != DraftClaimStatus::Supported)
                    .count(),
                claims,
                warnings: Vec::new(),
            };
            self.write_draft(&draft)?;
            self.mark_ready(&draft.source_id)?;
            self.append_audit(AuditEventType::AnswerDraftBuilt, &draft.source_id, &draft.version_id, "answer draft built")?;
            Ok(draft)
        })();
        if result.is_err() {
            let _ = self.append_audit(AuditEventType::AnswerDraftFailed, source_id, "", "answer draft build failed");
        }
        result
    }

    pub fn read_answer_draft(&self, source_id: &str, answer_draft_id: &str) -> AegisResult<AnswerDraft> {
        self.paths.ensure_layout()?;
        validate_answer_draft_id(answer_draft_id)?;
        let registry = SourceRegistry::load(&self.paths.registry_path())?;
        let record = registry.get_source(source_id)?;
        let path = self.draft_path(&record.source_id, &record.version_id, answer_draft_id);
        if !path.exists() {
            return Err(AegisError::AnswerDraftMissing);
        }
        let content = fs::read_to_string(&path).map_err(|_| AegisError::AnswerDraftReadFailed)?;
        serde_json::from_str(&content).map_err(|_| AegisError::AnswerDraftReadFailed)
    }

    fn write_draft(&self, draft: &AnswerDraft) -> AegisResult<()> {
        let path = self.draft_path(&draft.source_id, &draft.version_id, &draft.answer_draft_id);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|_| AegisError::AnswerDraftWriteFailed)?;
        }
        let body = serde_json::to_string_pretty(draft)?;
        fs::write(&path, body).map_err(|_| AegisError::AnswerDraftWriteFailed)?;
        Ok(())
    }

    fn draft_path(&self, source_id: &str, version_id: &str, answer_draft_id: &str) -> PathBuf {
        self.paths
            .source_version_dir(source_id, version_id)
            .join("answer_drafts")
            .join(format!("{answer_draft_id}.json"))
    }

    fn mark_ready(&self, source_id: &str) -> AegisResult<()> {
        let registry_path = self.paths.registry_path();
        let mut registry = SourceRegistry::load(&registry_path)?;
        let mut record = registry.get_source(source_id)?;
        record.ingestion_status = IngestionStatus::EvidenceReady;
        registry.replace(record)?;
        registry.save(&registry_path)?;
        Ok(())
    }

    fn append_audit(&self, event_type: AuditEventType, source_id: &str, version_id: &str, summary: &str) -> AegisResult<()> {
        let event = AuditEvent::new(event_type, Some(source_id.to_string()), Some(version_id.to_string()), summary);
        append_audit_event(&self.paths.audit_events_path(), &event)
    }
}

fn mechanical_claim_from_item(item: &EvidenceItem) -> DraftClaim {
    let text = format!("Evidence states: {}", item.preview);
    DraftClaim {
        claim_id: deterministic_claim_id(item, &text, &DraftClaimStatus::Supported),
        status: DraftClaimStatus::Supported,
        text,
        evidence_ids: vec![item.evidence_id.clone()],
        chunk_ids: vec![item.chunk_id.clone()],
        locators: vec![item.locator.clone()],
        confidence: DraftClaimConfidence::Mechanical,
    }
}

fn deterministic_answer_draft_id(evidence: &EvidencePack, claims: &[DraftClaim]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(evidence.evidence_pack_id.as_bytes());
    hasher.update(evidence.source_id.as_bytes());
    hasher.update(evidence.version_id.as_bytes());
    hasher.update(evidence.query.as_bytes());
    hasher.update(format!("{:?}", AnswerDraftMode::EvidenceOnly).as_bytes());
    for claim in claims {
        hasher.update(claim.claim_id.as_bytes());
    }
    format!("adr_{:x}", hasher.finalize())
}

fn deterministic_claim_id(item: &EvidenceItem, claim_text: &str, status: &DraftClaimStatus) -> String {
    let mut hasher = Sha256::new();
    hasher.update(item.evidence_id.as_bytes());
    hasher.update(item.chunk_id.as_bytes());
    hasher.update(item.text_hash.as_bytes());
    hasher.update(serde_json::to_string(&item.locator).unwrap().as_bytes());
    hasher.update(claim_text.as_bytes());
    hasher.update(serde_json::to_string(status).unwrap().as_bytes());
    format!("dcl_{:x}", hasher.finalize())
}

fn validate_answer_draft_id(answer_draft_id: &str) -> AegisResult<()> {
    if answer_draft_id.trim().is_empty() {
        return Err(AegisError::AnswerDraftInvalidId);
    }
    if answer_draft_id.contains('/') || answer_draft_id.contains('\\') || answer_draft_id.contains("..") {
        return Err(AegisError::AnswerDraftInvalidId);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunking::{ChunkRecord, ChunkingReport};
    use crate::corpus_authority::CorpusAuthority;
    use crate::evidence::EvidenceService;
    use crate::locators::CitationLocator;
    use crate::retrieval::{RetrievalIndex, RetrievalIndexEntry};
    use crate::source_metadata::{IngestionStatus, SourceMetadataInput, SourceType};
    use std::fs;

    fn valid_metadata() -> SourceMetadataInput {
        SourceMetadataInput {
            title: "Notes".to_string(),
            source_type: SourceType::MarkdownNote,
            discipline: "psychology".to_string(),
            subdiscipline: Some("statistics".to_string()),
            language: "en".to_string(),
            tags: vec!["study".to_string()],
            reliability_notes: None,
        }
    }

    fn locator() -> CitationLocator {
        CitationLocator::paragraph("paragraph:1", Some(vec!["Notes".to_string()]), 0, 16)
    }

    fn prepare_evidence(root: &PathBuf) -> (String, String) {
        let source_path = root.join("notes.md");
        fs::write(&source_path, "alpha beta gamma").unwrap();
        let authority = CorpusAuthority::new(root.clone());
        authority.register_source(source_path.to_string_lossy().to_string(), valid_metadata()).unwrap();
        let registry = SourceRegistry::load(&crate::corpus_paths::CorpusPaths::new(root.clone()).registry_path()).unwrap();
        let record = registry.sources.first().unwrap().clone();
        let chunk = ChunkRecord {
            chunk_id: "chk_demo".to_string(),
            source_id: record.source_id.clone(),
            version_id: record.version_id.clone(),
            locator: locator(),
            text: "alpha beta gamma".to_string(),
            content_hash: "sha256:chunk".to_string(),
            extraction_unit_hash: "sha256:unit".to_string(),
            chunk_index: 0,
            discipline: None,
            subdiscipline: None,
            method_tags: Vec::new(),
            topic_tags: Vec::new(),
            extraction_confidence: None,
        };
        let report = ChunkingReport {
            source_id: record.source_id.clone(),
            version_id: record.version_id.clone(),
            chunked_at: Utc::now(),
            chunk_count: 1,
            extraction_report_hash: "sha256:extraction".to_string(),
            warnings: Vec::new(),
            chunks: vec![chunk],
        };
        let corpus = crate::corpus_paths::CorpusPaths::new(root.clone());
        let chunk_path = corpus.source_version_dir(&record.source_id, &record.version_id).join("chunks").join("chunks.json");
        fs::create_dir_all(chunk_path.parent().unwrap()).unwrap();
        fs::write(chunk_path, serde_json::to_string_pretty(&report).unwrap()).unwrap();
        let index = RetrievalIndex {
            source_id: record.source_id.clone(),
            version_id: record.version_id.clone(),
            indexed_at: Utc::now(),
            chunk_count: 1,
            index_version: "retrieval-index-v1".to_string(),
            chunk_report_hash: "sha256:extraction".to_string(),
            entries: vec![RetrievalIndexEntry {
                chunk_id: "chk_demo".to_string(),
                source_id: record.source_id.clone(),
                version_id: record.version_id.clone(),
                locator: locator(),
                text_hash: "sha256:chunk".to_string(),
                normalized_terms: vec!["alpha".to_string()],
            }],
            warnings: Vec::new(),
        };
        let index_path = corpus.source_version_dir(&record.source_id, &record.version_id).join("retrieval").join("index.json");
        fs::create_dir_all(index_path.parent().unwrap()).unwrap();
        fs::write(index_path, serde_json::to_string_pretty(&index).unwrap()).unwrap();
        (record.source_id, record.version_id)
    }

    #[test]
    fn build_answer_draft_creates_mechanical_claims_and_marks_ready() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, version_id) = prepare_evidence(&temp.path().to_path_buf());
        let evidence = EvidenceService::new(temp.path().to_path_buf()).build_evidence_pack(&source_id, "alpha", 5).unwrap();
        let service = AnswerDraftService::new(temp.path().to_path_buf());
        let draft = service.build_answer_draft(&source_id, &evidence.evidence_pack_id).unwrap();
        assert!(draft.answer_draft_id.starts_with("adr_"));
        assert_eq!(draft.evidence_pack_id, evidence.evidence_pack_id);
        assert_eq!(draft.source_id, source_id);
        assert_eq!(draft.version_id, version_id);
        assert_eq!(draft.claim_count, 1);
        assert_eq!(draft.claims[0].text, "Evidence states: alpha beta gamma");
        assert!(draft.claims[0].claim_id.starts_with("dcl_"));
        assert_eq!(draft.claims[0].evidence_ids, vec![evidence.items[0].evidence_id.clone()]);
        assert_eq!(draft.claims[0].chunk_ids, vec!["chk_demo".to_string()]);
        let registry = SourceRegistry::load(&crate::corpus_paths::CorpusPaths::new(temp.path().to_path_buf()).registry_path()).unwrap();
        assert_eq!(registry.get_source(&source_id).unwrap().ingestion_status, IngestionStatus::EvidenceReady);
    }

    #[test]
    fn answer_draft_rejects_missing_or_traversal_ids() {
        let temp = tempfile::tempdir().unwrap();
        let service = AnswerDraftService::new(temp.path().to_path_buf());
        assert!(matches!(service.read_answer_draft("src_demo", ""), Err(AegisError::AnswerDraftInvalidId)));
        assert!(matches!(service.read_answer_draft("src_demo", "../x"), Err(AegisError::AnswerDraftInvalidId)));
        assert!(matches!(service.read_answer_draft("src_demo", "x/y"), Err(AegisError::AnswerDraftInvalidId)));
    }

    #[test]
    fn answer_draft_missing_and_malformed_are_typed_errors() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, version_id) = prepare_evidence(&temp.path().to_path_buf());
        let service = AnswerDraftService::new(temp.path().to_path_buf());
        assert!(matches!(service.read_answer_draft(&source_id, "adr_missing"), Err(AegisError::AnswerDraftMissing)));

        let corpus = crate::corpus_paths::CorpusPaths::new(temp.path().to_path_buf());
        let bad_path = corpus.source_version_dir(&source_id, &version_id).join("answer_drafts").join("adr_bad.json");
        fs::create_dir_all(bad_path.parent().unwrap()).unwrap();
        fs::write(&bad_path, "{not-json").unwrap();
        assert!(matches!(service.read_answer_draft(&source_id, "adr_bad"), Err(AegisError::AnswerDraftReadFailed)));
    }
}
