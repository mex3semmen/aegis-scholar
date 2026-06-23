use crate::answer_draft::{AnswerDraft, AnswerDraftService, DraftClaim, DraftClaimStatus};
use crate::audit::{append_audit_event, AuditEvent, AuditEventType};
use crate::corpus_paths::CorpusPaths;
use crate::errors::{AegisError, AegisResult};
use crate::locators::CitationLocator;
use crate::source_metadata::IngestionStatus;
use crate::source_registry::SourceRegistry;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroundedAnswer {
    pub grounded_answer_id: String,
    pub answer_draft_id: String,
    pub evidence_pack_id: String,
    pub source_id: String,
    pub version_id: String,
    pub query: String,
    pub created_at: DateTime<Utc>,
    pub answer_mode: GroundedAnswerMode,
    pub statement_count: usize,
    pub unsupported_count: usize,
    pub statements: Vec<GroundedStatement>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GroundedAnswerMode {
    ContractOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GroundedStatement {
    pub statement_id: String,
    pub status: GroundedStatementStatus,
    pub text: String,
    pub claim_ids: Vec<String>,
    pub evidence_ids: Vec<String>,
    pub chunk_ids: Vec<String>,
    pub locators: Vec<CitationLocator>,
    pub support_level: GroundedSupportLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GroundedStatementStatus {
    Supported,
    NeedsEvidence,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GroundedSupportLevel {
    DirectClaim,
    MissingEvidence,
}

pub struct GroundedAnswerService {
    paths: CorpusPaths,
}

pub fn build_grounded_answer(root: impl Into<PathBuf>, source_id: &str, answer_draft_id: &str) -> AegisResult<GroundedAnswer> {
    GroundedAnswerService::new(root).build_grounded_answer(source_id, answer_draft_id)
}

pub fn read_grounded_answer(root: impl Into<PathBuf>, source_id: &str, grounded_answer_id: &str) -> AegisResult<GroundedAnswer> {
    GroundedAnswerService::new(root).read_grounded_answer(source_id, grounded_answer_id)
}

impl GroundedAnswerService {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { paths: CorpusPaths::new(root) }
    }

    pub fn build_grounded_answer(&self, source_id: &str, answer_draft_id: &str) -> AegisResult<GroundedAnswer> {
        let result = (|| -> AegisResult<GroundedAnswer> {
            if source_id.trim().is_empty() {
                return Err(AegisError::GroundedAnswerInputMissing);
            }
            validate_grounded_answer_id(answer_draft_id)?;
            self.paths.ensure_layout()?;
            let draft = AnswerDraftService::new(self.paths.root.clone()).read_answer_draft(source_id, answer_draft_id)?;
            if draft.claims.is_empty() {
                return Err(AegisError::GroundedAnswerEmptyDraft);
            }
            let statements = draft.claims.iter().map(statement_from_claim).collect::<Vec<_>>();
            let grounded_answer_id = deterministic_grounded_answer_id(&draft, &statements);
            let answer = GroundedAnswer {
                grounded_answer_id: grounded_answer_id.clone(),
                answer_draft_id: draft.answer_draft_id.clone(),
                evidence_pack_id: draft.evidence_pack_id.clone(),
                source_id: draft.source_id.clone(),
                version_id: draft.version_id.clone(),
                query: draft.query.clone(),
                created_at: Utc::now(),
                answer_mode: GroundedAnswerMode::ContractOnly,
                statement_count: statements.len(),
                unsupported_count: statements.iter().filter(|s| s.status != GroundedStatementStatus::Supported).count(),
                statements,
                warnings: Vec::new(),
            };
            self.write_answer(&answer)?;
            self.mark_ready(&answer.source_id)?;
            self.append_audit(AuditEventType::GroundedAnswerBuilt, &answer.source_id, &answer.version_id, "grounded answer built")?;
            Ok(answer)
        })();
        if result.is_err() {
            let _ = self.append_audit(AuditEventType::GroundedAnswerFailed, source_id, "", "grounded answer build failed");
        }
        result
    }

    pub fn read_grounded_answer(&self, source_id: &str, grounded_answer_id: &str) -> AegisResult<GroundedAnswer> {
        if source_id.trim().is_empty() {
            return Err(AegisError::GroundedAnswerInputMissing);
        }
        validate_grounded_answer_id(grounded_answer_id)?;
        self.paths.ensure_layout()?;
        let registry = SourceRegistry::load(&self.paths.registry_path())?;
        let record = registry.get_source(source_id)?;
        let path = self.answer_path(&record.source_id, &record.version_id, grounded_answer_id);
        if !path.exists() {
            return Err(AegisError::GroundedAnswerMissing);
        }
        let content = fs::read_to_string(&path).map_err(|_| AegisError::GroundedAnswerReadFailed)?;
        serde_json::from_str(&content).map_err(|_| AegisError::GroundedAnswerReadFailed)
    }

    fn write_answer(&self, answer: &GroundedAnswer) -> AegisResult<()> {
        let path = self.answer_path(&answer.source_id, &answer.version_id, &answer.grounded_answer_id);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|_| AegisError::GroundedAnswerWriteFailed)?;
        }
        let body = serde_json::to_string_pretty(answer)?;
        fs::write(&path, body).map_err(|_| AegisError::GroundedAnswerWriteFailed)?;
        Ok(())
    }

    fn answer_path(&self, source_id: &str, version_id: &str, grounded_answer_id: &str) -> PathBuf {
        self.paths
            .source_version_dir(source_id, version_id)
            .join("grounded_answers")
            .join(format!("{grounded_answer_id}.json"))
    }

    fn mark_ready(&self, source_id: &str) -> AegisResult<()> {
        let registry_path = self.paths.registry_path();
        let mut registry = SourceRegistry::load(&registry_path)?;
        let mut record = registry.get_source(source_id)?;
        record.ingestion_status = IngestionStatus::GroundedAnswerReady;
        registry.replace(record)?;
        registry.save(&registry_path)?;
        Ok(())
    }

    fn append_audit(&self, event_type: AuditEventType, source_id: &str, version_id: &str, summary: &str) -> AegisResult<()> {
        let event = AuditEvent::new(event_type, Some(source_id.to_string()), Some(version_id.to_string()), summary);
        append_audit_event(&self.paths.audit_events_path(), &event)
    }
}

fn statement_from_claim(claim: &DraftClaim) -> GroundedStatement {
    let support_level = match claim.status {
        DraftClaimStatus::Supported => GroundedSupportLevel::DirectClaim,
        DraftClaimStatus::NeedsEvidence | DraftClaimStatus::Unsupported => GroundedSupportLevel::MissingEvidence,
    };
    GroundedStatement {
        statement_id: deterministic_statement_id(claim),
        status: match claim.status {
            DraftClaimStatus::Supported => GroundedStatementStatus::Supported,
            DraftClaimStatus::NeedsEvidence => GroundedStatementStatus::NeedsEvidence,
            DraftClaimStatus::Unsupported => GroundedStatementStatus::Unsupported,
        },
        text: claim.text.clone(),
        claim_ids: vec![claim.claim_id.clone()],
        evidence_ids: claim.evidence_ids.clone(),
        chunk_ids: claim.chunk_ids.clone(),
        locators: claim.locators.clone(),
        support_level,
    }
}

fn deterministic_grounded_answer_id(draft: &AnswerDraft, statements: &[GroundedStatement]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(draft.answer_draft_id.as_bytes());
    hasher.update(draft.evidence_pack_id.as_bytes());
    hasher.update(draft.source_id.as_bytes());
    hasher.update(draft.version_id.as_bytes());
    hasher.update(draft.query.as_bytes());
    hasher.update(serde_json::to_string(&GroundedAnswerMode::ContractOnly).unwrap().as_bytes());
    for statement in statements {
        hasher.update(statement.statement_id.as_bytes());
    }
    format!("gan_{:x}", hasher.finalize())
}

fn deterministic_statement_id(claim: &DraftClaim) -> String {
    let mut hasher = Sha256::new();
    hasher.update(claim.claim_id.as_bytes());
    hasher.update(serde_json::to_string(&claim.status).unwrap().as_bytes());
    hasher.update(claim.text.as_bytes());
    hasher.update(serde_json::to_string(&claim.evidence_ids).unwrap().as_bytes());
    hasher.update(serde_json::to_string(&claim.chunk_ids).unwrap().as_bytes());
    hasher.update(serde_json::to_string(&claim.locators).unwrap().as_bytes());
    let support_level = match claim.status {
        DraftClaimStatus::Supported => GroundedSupportLevel::DirectClaim,
        DraftClaimStatus::NeedsEvidence | DraftClaimStatus::Unsupported => GroundedSupportLevel::MissingEvidence,
    };
    hasher.update(serde_json::to_string(&support_level).unwrap().as_bytes());
    format!("gst_{:x}", hasher.finalize())
}

fn validate_grounded_answer_id(grounded_answer_id: &str) -> AegisResult<()> {
    if grounded_answer_id.trim().is_empty() {
        return Err(AegisError::GroundedAnswerInvalidId);
    }
    if grounded_answer_id.contains('/') || grounded_answer_id.contains('\\') || grounded_answer_id.contains("..") {
        return Err(AegisError::GroundedAnswerInvalidId);
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

    fn prepare_draft(root: &PathBuf) -> (String, String, String) {
        let source_path = root.join("notes.md");
        fs::write(&source_path, "alpha beta gamma").unwrap();
        let authority = CorpusAuthority::new(root.clone());
        authority.register_source(source_path.to_string_lossy().to_string(), valid_metadata()).unwrap();
        let corpus = CorpusPaths::new(root.clone());
        let registry = SourceRegistry::load(&corpus.registry_path()).unwrap();
        let record = registry.sources.first().unwrap().clone();
        let chunk = ChunkRecord { chunk_id: "chk_demo".to_string(), source_id: record.source_id.clone(), version_id: record.version_id.clone(), locator: locator(), text: "alpha beta gamma".to_string(), content_hash: "sha256:chunk".to_string(), extraction_unit_hash: "sha256:unit".to_string(), chunk_index: 0, discipline: None, subdiscipline: None, method_tags: Vec::new(), topic_tags: Vec::new(), extraction_confidence: None };
        let report = ChunkingReport { source_id: record.source_id.clone(), version_id: record.version_id.clone(), chunked_at: Utc::now(), chunk_count: 1, extraction_report_hash: "sha256:extraction".to_string(), warnings: Vec::new(), chunks: vec![chunk] };
        let chunk_path = corpus.source_version_dir(&record.source_id, &record.version_id).join("chunks").join("chunks.json");
        fs::create_dir_all(chunk_path.parent().unwrap()).unwrap();
        fs::write(chunk_path, serde_json::to_string_pretty(&report).unwrap()).unwrap();
        let index = RetrievalIndex { source_id: record.source_id.clone(), version_id: record.version_id.clone(), indexed_at: Utc::now(), chunk_count: 1, index_version: "retrieval-index-v1".to_string(), chunk_report_hash: "sha256:extraction".to_string(), entries: vec![RetrievalIndexEntry { chunk_id: "chk_demo".to_string(), source_id: record.source_id.clone(), version_id: record.version_id.clone(), locator: locator(), text_hash: "sha256:chunk".to_string(), normalized_terms: vec!["alpha".to_string()] }], warnings: Vec::new() };
        let index_path = corpus.source_version_dir(&record.source_id, &record.version_id).join("retrieval").join("index.json");
        fs::create_dir_all(index_path.parent().unwrap()).unwrap();
        fs::write(index_path, serde_json::to_string_pretty(&index).unwrap()).unwrap();
        let evidence = EvidenceService::new(root.clone()).build_evidence_pack(&record.source_id, "alpha", 5).unwrap();
        let draft = crate::answer_draft::AnswerDraftService::new(root.clone()).build_answer_draft(&record.source_id, &evidence.evidence_pack_id).unwrap();
        (record.source_id, record.version_id, draft.answer_draft_id)
    }

    fn write_manual_draft(root: &PathBuf, source_id: &str, version_id: &str, draft: crate::answer_draft::AnswerDraft) {
        let corpus = CorpusPaths::new(root.clone());
        let path = corpus
            .source_version_dir(source_id, version_id)
            .join("answer_drafts")
            .join(format!("{}.json", draft.answer_draft_id));
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, serde_json::to_string_pretty(&draft).unwrap()).unwrap();
    }

    #[test]
    fn build_and_read_grounded_answer_round_trips() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, version_id, draft_id) = prepare_draft(&temp.path().to_path_buf());
        let answer = build_grounded_answer(temp.path().to_path_buf(), &source_id, &draft_id).unwrap();
        assert!(answer.grounded_answer_id.starts_with("gan_"));
        assert_eq!(answer.source_id, source_id);
        assert_eq!(answer.version_id, version_id);
        assert_eq!(answer.statement_count, 1);
        assert_eq!(answer.statements[0].text, "Evidence states: alpha beta gamma");
        assert_eq!(answer.statements[0].status, GroundedStatementStatus::Supported);
        assert_eq!(answer.statements[0].support_level, GroundedSupportLevel::DirectClaim);
        let read_back = read_grounded_answer(temp.path().to_path_buf(), &source_id, &answer.grounded_answer_id).unwrap();
        assert_eq!(read_back.grounded_answer_id, answer.grounded_answer_id);
        let registry = SourceRegistry::load(&CorpusPaths::new(temp.path().to_path_buf()).registry_path()).unwrap();
        assert_eq!(registry.get_source(&source_id).unwrap().ingestion_status, IngestionStatus::GroundedAnswerReady);
    }

    #[test]
    fn grounded_answer_rejects_invalid_ids() {
        let temp = tempfile::tempdir().unwrap();
        assert!(matches!(read_grounded_answer(temp.path().to_path_buf(), "", "gan_x"), Err(AegisError::GroundedAnswerInputMissing)));
        assert!(matches!(read_grounded_answer(temp.path().to_path_buf(), "src_demo", ""), Err(AegisError::GroundedAnswerInvalidId)));
        assert!(matches!(read_grounded_answer(temp.path().to_path_buf(), "src_demo", "../x"), Err(AegisError::GroundedAnswerInvalidId)));
        assert!(matches!(read_grounded_answer(temp.path().to_path_buf(), "src_demo", "..\\x"), Err(AegisError::GroundedAnswerInvalidId)));
        assert!(matches!(read_grounded_answer(temp.path().to_path_buf(), "src_demo", "/x"), Err(AegisError::GroundedAnswerInvalidId)));
        assert!(matches!(read_grounded_answer(temp.path().to_path_buf(), "src_demo", "\\x"), Err(AegisError::GroundedAnswerInvalidId)));
        assert!(matches!(read_grounded_answer(temp.path().to_path_buf(), "src_demo", "x/y"), Err(AegisError::GroundedAnswerInvalidId)));
        assert!(matches!(read_grounded_answer(temp.path().to_path_buf(), "src_demo", "x\\y"), Err(AegisError::GroundedAnswerInvalidId)));
    }

    #[test]
    fn grounded_answer_enums_serialize_as_snake_case() {
        assert_eq!(serde_json::to_string(&GroundedAnswerMode::ContractOnly).unwrap(), "\"contract_only\"");
        assert_eq!(serde_json::to_string(&GroundedStatementStatus::NeedsEvidence).unwrap(), "\"needs_evidence\"");
        assert_eq!(serde_json::to_string(&GroundedSupportLevel::MissingEvidence).unwrap(), "\"missing_evidence\"");
    }

    #[test]
    fn grounded_answer_missing_and_malformed_are_typed_errors() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, version_id, draft_id) = prepare_draft(&temp.path().to_path_buf());
        assert!(matches!(read_grounded_answer(temp.path().to_path_buf(), &source_id, "gan_missing"), Err(AegisError::GroundedAnswerMissing)));

        let corpus = CorpusPaths::new(temp.path().to_path_buf());
        let bad_path = corpus.source_version_dir(&source_id, &version_id).join("grounded_answers").join("gan_bad.json");
        fs::create_dir_all(bad_path.parent().unwrap()).unwrap();
        fs::write(&bad_path, "{not-json").unwrap();
        assert!(matches!(read_grounded_answer(temp.path().to_path_buf(), &source_id, "gan_bad"), Err(AegisError::GroundedAnswerReadFailed)));

        let draft_path = corpus.source_version_dir(&source_id, &version_id).join("answer_drafts").join(format!("{}.json", draft_id));
        assert!(draft_path.exists());
    }

    #[test]
    fn grounded_answer_failure_and_status_hardening() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, version_id, draft_id) = prepare_draft(&temp.path().to_path_buf());
        let result = build_grounded_answer(temp.path().to_path_buf(), &source_id, "gan_missing");
        assert!(matches!(result, Err(AegisError::AnswerDraftMissing)));
        let corpus = CorpusPaths::new(temp.path().to_path_buf());
        let grounded_dir = corpus.source_version_dir(&source_id, &version_id).join("grounded_answers");
        assert!(!grounded_dir.exists() || fs::read_dir(grounded_dir).unwrap().next().is_none());
        let registry = SourceRegistry::load(&corpus.registry_path()).unwrap();
        assert_ne!(registry.get_source(&source_id).unwrap().ingestion_status, IngestionStatus::GroundedAnswerReady);
        let audit = fs::read_to_string(corpus.audit_events_path()).unwrap();
        assert!(audit.contains("grounded_answer_failed"));
        assert!(audit.lines().all(|line| serde_json::from_str::<serde_json::Value>(line).is_ok()));

        let mut draft = crate::answer_draft::AnswerDraftService::new(temp.path().to_path_buf())
            .read_answer_draft(&source_id, &draft_id)
            .unwrap();
        draft.claims[0].status = crate::answer_draft::DraftClaimStatus::NeedsEvidence;
        draft.claims[0].confidence = crate::answer_draft::DraftClaimConfidence::MissingEvidence;
        write_manual_draft(&temp.path().to_path_buf(), &source_id, &version_id, draft);
        let answer = build_grounded_answer(temp.path().to_path_buf(), &source_id, &draft_id).unwrap();
        assert_eq!(answer.statements[0].status, GroundedStatementStatus::NeedsEvidence);
        assert_eq!(answer.statements[0].support_level, GroundedSupportLevel::MissingEvidence);
        assert_eq!(answer.unsupported_count, 1);
        let registry = SourceRegistry::load(&corpus.registry_path()).unwrap();
        assert_eq!(registry.get_source(&source_id).unwrap().ingestion_status, IngestionStatus::GroundedAnswerReady);
    }

    #[test]
    fn grounded_answer_mechanical_projection_preserves_claim_scope() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, _version_id, draft_id) = prepare_draft(&temp.path().to_path_buf());
        let answer = build_grounded_answer(temp.path().to_path_buf(), &source_id, &draft_id).unwrap();
        assert_eq!(answer.statements.len(), 1);
        assert_eq!(answer.statements[0].claim_ids.len(), 1);
        assert_eq!(answer.statements[0].text, "Evidence states: alpha beta gamma");
        assert!(answer.statements[0].text.starts_with("Evidence states: "));
        assert_eq!(answer.statements[0].evidence_ids.len(), 1);
        assert_eq!(answer.statements[0].chunk_ids, vec!["chk_demo".to_string()]);
        assert_eq!(answer.statements[0].locators, vec![locator()]);
        assert_eq!(answer.statements[0].status, GroundedStatementStatus::Supported);
        assert_eq!(answer.statements[0].support_level, GroundedSupportLevel::DirectClaim);
        assert!(answer.grounded_answer_id.starts_with("gan_"));
        assert_eq!(answer.answer_draft_id, draft_id);
        assert!(answer.evidence_pack_id.starts_with("evp_"));
        assert_eq!(answer.statement_count, answer.statements.len());
        assert_eq!(answer.unsupported_count, 0);
    }

    #[test]
    fn grounded_answer_adapter_boundary_rejects_invalid_ids() {
        let temp = tempfile::tempdir().unwrap();
        assert!(matches!(read_grounded_answer(temp.path().to_path_buf(), "", "gan_x"), Err(AegisError::GroundedAnswerInputMissing)));
        assert!(matches!(read_grounded_answer(temp.path().to_path_buf(), "src_demo", ""), Err(AegisError::GroundedAnswerInvalidId)));
        assert!(matches!(read_grounded_answer(temp.path().to_path_buf(), "src_demo", "../x"), Err(AegisError::GroundedAnswerInvalidId)));
        assert!(matches!(read_grounded_answer(temp.path().to_path_buf(), "src_demo", "..\\x"), Err(AegisError::GroundedAnswerInvalidId)));
        assert!(matches!(read_grounded_answer(temp.path().to_path_buf(), "src_demo", "/x"), Err(AegisError::GroundedAnswerInvalidId)));
        assert!(matches!(read_grounded_answer(temp.path().to_path_buf(), "src_demo", "\\x"), Err(AegisError::GroundedAnswerInvalidId)));
        assert!(matches!(read_grounded_answer(temp.path().to_path_buf(), "src_demo", "x/y"), Err(AegisError::GroundedAnswerInvalidId)));
        assert!(matches!(read_grounded_answer(temp.path().to_path_buf(), "src_demo", "x\\y"), Err(AegisError::GroundedAnswerInvalidId)));
    }

    #[test]
    fn grounded_answer_adapter_boundary_keeps_status_and_projection() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, version_id, draft_id) = prepare_draft(&temp.path().to_path_buf());
        let mut draft = AnswerDraftService::new(temp.path().to_path_buf())
            .read_answer_draft(&source_id, &draft_id)
            .unwrap();
        draft.claims[0].status = crate::answer_draft::DraftClaimStatus::Unsupported;
        draft.claims[0].confidence = crate::answer_draft::DraftClaimConfidence::MissingEvidence;
        write_manual_draft(&temp.path().to_path_buf(), &source_id, &version_id, draft);
        let answer = build_grounded_answer(temp.path().to_path_buf(), &source_id, &draft_id).unwrap();
        assert_eq!(answer.statements[0].status, GroundedStatementStatus::Unsupported);
        assert_eq!(answer.statements[0].support_level, GroundedSupportLevel::MissingEvidence);
        assert_eq!(answer.statement_count, 1);
        assert_eq!(answer.unsupported_count, 1);
    }
}
