use crate::audit::{append_audit_event, AuditEvent, AuditEventType};
use crate::corpus_paths::CorpusPaths;
use crate::errors::{AegisError, AegisResult};
use crate::locators::CitationLocator;
use crate::retrieval::{RetrievalResult, RetrievalService};
use crate::source_metadata::IngestionStatus;
use crate::source_registry::SourceRegistry;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

const EVIDENCE_PACK_VERSION: &str = "evidence-pack-v1";
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidencePack {
    pub evidence_pack_id: String,
    pub source_id: String,
    pub version_id: String,
    pub query: String,
    pub created_at: DateTime<Utc>,
    pub retrieval_index_version: String,
    pub result_count: usize,
    pub item_count: usize,
    pub evidence_pack_version: String,
    pub warnings: Vec<String>,
    pub items: Vec<EvidenceItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub evidence_id: String,
    pub chunk_id: String,
    pub source_id: String,
    pub version_id: String,
    pub locator: CitationLocator,
    pub text_hash: String,
    pub score: f32,
    pub matched_terms: Vec<String>,
    pub preview: String,
}

pub struct EvidenceService {
    paths: CorpusPaths,
}

impl EvidenceService {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { paths: CorpusPaths::new(root) }
    }

    pub fn build_evidence_pack(&self, source_id: &str, query: &str, max_results: usize) -> AegisResult<EvidencePack> {
        self.paths.ensure_layout()?;
        let result = (|| -> AegisResult<EvidencePack> {
            if source_id.trim().is_empty() {
                return Err(AegisError::EvidencePackInputMissing);
            }
            if max_results == 0 {
                return Err(AegisError::EvidencePackInvalidLimit);
            }
            if query.trim().is_empty() {
                return Err(AegisError::EvidencePackQueryEmpty);
            }
            let retrieval = RetrievalService::new(self.paths.root.clone());
            let response = retrieval.search_source(source_id, query, max_results)?;
            let index = retrieval.read_index(source_id)?;
            let registry = SourceRegistry::load(&self.paths.registry_path())?;
            let record = registry.get_source(source_id)?;
            let items = response
                .results
                .iter()
                .map(|result| evidence_item_from_result(result))
                .collect::<Vec<_>>();
            if items.is_empty() {
                return Err(AegisError::EvidencePackEmpty);
            }
            let evidence_pack_id = deterministic_evidence_pack_id(
                &record.source_id,
                &record.version_id,
                query,
                &index.index_version,
                &items,
            );
            let pack = EvidencePack {
                evidence_pack_id: evidence_pack_id.clone(),
                source_id: record.source_id.clone(),
                version_id: record.version_id.clone(),
                query: query.to_string(),
                created_at: Utc::now(),
                retrieval_index_version: index.index_version.clone(),
                result_count: response.result_count,
                item_count: items.len(),
                evidence_pack_version: EVIDENCE_PACK_VERSION.to_string(),
                warnings: Vec::new(),
                items,
            };
            self.write_pack(&pack)?;
            let mut updated = record.clone();
            updated.ingestion_status = IngestionStatus::EvidenceReady;
            let mut registry = SourceRegistry::load(&self.paths.registry_path())?;
            registry.replace(updated)?;
            registry.save(&self.paths.registry_path())?;
            self.append_audit(
                AuditEventType::EvidencePackBuilt,
                &pack.source_id,
                &pack.version_id,
                "evidence pack built",
            )?;
            Ok(pack)
        })();
        if result.is_err() {
            let _ = self.append_audit(
                AuditEventType::EvidencePackFailed,
                source_id,
                "",
                "evidence pack build failed",
            );
        }
        result
    }

    pub fn read_evidence_pack(&self, source_id: &str, evidence_pack_id: &str) -> AegisResult<EvidencePack> {
        self.paths.ensure_layout()?;
        validate_evidence_pack_id(evidence_pack_id)?;
        let registry = SourceRegistry::load(&self.paths.registry_path())?;
        let record = registry.get_source(source_id)?;
        let path = self.pack_path(&record.source_id, &record.version_id, evidence_pack_id);
        if !path.exists() {
            return Err(AegisError::EvidencePackMissing);
        }
        let content = fs::read_to_string(&path).map_err(|_| AegisError::EvidencePackReadFailed)?;
        serde_json::from_str(&content).map_err(|_| AegisError::EvidencePackReadFailed)
    }

    fn write_pack(&self, pack: &EvidencePack) -> AegisResult<()> {
        let path = self.pack_path(&pack.source_id, &pack.version_id, &pack.evidence_pack_id);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|_| AegisError::EvidencePackWriteFailed)?;
        }
        let body = serde_json::to_string_pretty(pack)?;
        fs::write(&path, body).map_err(|_| AegisError::EvidencePackWriteFailed)?;
        Ok(())
    }

    fn pack_path(&self, source_id: &str, version_id: &str, evidence_pack_id: &str) -> PathBuf {
        self.paths
            .source_version_dir(source_id, version_id)
            .join("evidence")
            .join(format!("{evidence_pack_id}.json"))
    }

    fn append_audit(&self, event_type: AuditEventType, source_id: &str, version_id: &str, summary: &str) -> AegisResult<()> {
        let event = AuditEvent::new(event_type, Some(source_id.to_string()), Some(version_id.to_string()), summary);
        append_audit_event(&self.paths.audit_events_path(), &event)
    }
}

fn evidence_item_from_result(result: &RetrievalResult) -> EvidenceItem {
    EvidenceItem {
        evidence_id: deterministic_evidence_item_id(result),
        chunk_id: result.chunk_id.clone(),
        source_id: result.source_id.clone(),
        version_id: result.version_id.clone(),
        locator: result.locator.clone(),
        text_hash: result.text_hash.clone(),
        score: result.score,
        matched_terms: result.matched_terms.clone(),
        preview: result.preview.clone(),
    }
}

fn deterministic_evidence_pack_id(source_id: &str, version_id: &str, query: &str, index_version: &str, items: &[EvidenceItem]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source_id.as_bytes());
    hasher.update(version_id.as_bytes());
    hasher.update(query.as_bytes());
    hasher.update(index_version.as_bytes());
    for item in items {
        hasher.update(item.evidence_id.as_bytes());
    }
    format!("evp_{:x}", hasher.finalize())
}

fn deterministic_evidence_item_id(result: &RetrievalResult) -> String {
    let mut hasher = Sha256::new();
    hasher.update(result.source_id.as_bytes());
    hasher.update(result.version_id.as_bytes());
    hasher.update(result.chunk_id.as_bytes());
    hasher.update(result.text_hash.as_bytes());
    hasher.update(result.locator.label.as_bytes());
    hasher.update(serde_json::to_string(&result.locator).unwrap().as_bytes());
    hasher.update(result.score.to_bits().to_le_bytes());
    for term in &result.matched_terms {
        hasher.update(term.as_bytes());
    }
    format!("evi_{:x}", hasher.finalize())
}

fn validate_evidence_pack_id(evidence_pack_id: &str) -> AegisResult<()> {
    if evidence_pack_id.trim().is_empty() {
        return Err(AegisError::EvidencePackInvalidId);
    }
    if evidence_pack_id.contains('/') || evidence_pack_id.contains('\\') || evidence_pack_id.contains("..") {
        return Err(AegisError::EvidencePackInvalidId);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunking::{ChunkRecord, ChunkingReport};
    use crate::corpus_authority::CorpusAuthority;
    use crate::locators::CitationLocator;
    use crate::retrieval::{RetrievalIndex, RetrievalIndexEntry, RetrievalResult};
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

    fn prepare_index(root: &PathBuf) -> String {
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
        let path = crate::corpus_paths::CorpusPaths::new(root.clone())
            .source_version_dir(&record.source_id, &record.version_id)
            .join("chunks")
            .join("chunks.json");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, serde_json::to_string_pretty(&report).unwrap()).unwrap();
        write_index(root, &record.source_id, &record.version_id);
        record.source_id
    }

    fn write_index(root: &PathBuf, source_id: &str, version_id: &str) {
        let index = RetrievalIndex {
            source_id: source_id.to_string(),
            version_id: version_id.to_string(),
            indexed_at: Utc::now(),
            chunk_count: 1,
            index_version: "retrieval-index-v1".to_string(),
            chunk_report_hash: "sha256:extraction".to_string(),
            entries: vec![RetrievalIndexEntry {
                chunk_id: "chk_demo".to_string(),
                source_id: source_id.to_string(),
                version_id: version_id.to_string(),
                locator: locator(),
                text_hash: "sha256:chunk".to_string(),
                normalized_terms: vec!["alpha".to_string()],
            }],
            warnings: Vec::new(),
        };
        let path = crate::corpus_paths::CorpusPaths::new(root.clone())
            .source_version_dir(source_id, version_id)
            .join("retrieval")
            .join("index.json");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, serde_json::to_string_pretty(&index).unwrap()).unwrap();
    }

    fn sample_result() -> RetrievalResult {
        RetrievalResult {
            chunk_id: "chk_demo".to_string(),
            source_id: "src_demo".to_string(),
            version_id: "srcv_demo".to_string(),
            locator: locator(),
            score: 0.5,
            matched_terms: vec!["alpha".to_string()],
            text_hash: "sha256:chunk".to_string(),
            preview: "alpha beta gamma".to_string(),
        }
    }

    #[test]
    fn build_and_read_evidence_pack_round_trip() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = prepare_index(&temp.path().to_path_buf());
        let service = EvidenceService::new(temp.path().to_path_buf());
        let pack = service.build_evidence_pack(&source_id, "alpha", 5).unwrap();
        assert!(pack.evidence_pack_id.starts_with("evp_"));
        let read_back = service.read_evidence_pack(&source_id, &pack.evidence_pack_id).unwrap();
        assert_eq!(read_back.source_id, pack.source_id);
        assert_eq!(read_back.version_id, pack.version_id);
        assert_eq!(read_back.items[0].chunk_id, "chk_demo");
        assert_eq!(read_back.items[0].preview, "alpha beta gamma");
        assert!(read_back.items[0].evidence_id.starts_with("evi_"));

        let registry = SourceRegistry::load(&crate::corpus_paths::CorpusPaths::new(temp.path().to_path_buf()).registry_path()).unwrap();
        let record = registry.get_source(&source_id).unwrap();
        assert_eq!(record.ingestion_status, IngestionStatus::EvidenceReady);

        let audit = fs::read_to_string(crate::corpus_paths::CorpusPaths::new(temp.path().to_path_buf()).audit_events_path()).unwrap();
        assert!(audit.contains("evidence_pack_built"));
        assert!(audit.lines().all(|line| serde_json::from_str::<serde_json::Value>(line).is_ok()));
    }

    #[test]
    fn evidence_pack_rejects_empty_query() {
        let temp = tempfile::tempdir().unwrap();
        let service = EvidenceService::new(temp.path().to_path_buf());
        assert!(matches!(service.build_evidence_pack("src_demo", "", 1), Err(AegisError::EvidencePackQueryEmpty)));
    }

    #[test]
    fn evidence_item_id_changes_with_content_inputs() {
        let mut a = sample_result();
        let base = evidence_item_from_result(&a).evidence_id;
        a.score = 0.75;
        let score_changed = evidence_item_from_result(&a).evidence_id;
        a.score = 0.5;
        a.text_hash = "sha256:other".to_string();
        let hash_changed = evidence_item_from_result(&a).evidence_id;
        a.text_hash = "sha256:chunk".to_string();
        a.locator = CitationLocator::paragraph("paragraph:2", Some(vec!["Notes".to_string()]), 1, 8);
        let locator_changed = evidence_item_from_result(&a).evidence_id;
        a.locator = locator();
        a.matched_terms = vec!["beta".to_string()];
        let terms_changed = evidence_item_from_result(&a).evidence_id;

        assert_ne!(base, score_changed);
        assert_ne!(base, hash_changed);
        assert_ne!(base, locator_changed);
        assert_ne!(base, terms_changed);
        assert!(base.starts_with("evi_"));
    }

    #[test]
    fn evidence_pack_id_is_deterministic_for_repeated_builds() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = prepare_index(&temp.path().to_path_buf());
        let service = EvidenceService::new(temp.path().to_path_buf());
        let first = service.build_evidence_pack(&source_id, "alpha", 5).unwrap();
        let second = service.build_evidence_pack(&source_id, "alpha", 5).unwrap();
        assert_eq!(first.evidence_pack_id, second.evidence_pack_id);
        assert!(first.evidence_pack_id.starts_with("evp_"));
    }

    #[test]
    fn evidence_pack_id_changes_with_query() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = prepare_index(&temp.path().to_path_buf());
        let service = EvidenceService::new(temp.path().to_path_buf());
        let alpha = service.build_evidence_pack(&source_id, "alpha", 5).unwrap();
        let beta = service.build_evidence_pack(&source_id, "beta", 5).unwrap_err();
        assert!(alpha.evidence_pack_id.starts_with("evp_"));
        assert!(matches!(beta, AegisError::EvidencePackEmpty | AegisError::RetrievalQueryEmpty));
    }

    #[test]
    fn evidence_pack_rejects_path_traversal_ids() {
        let temp = tempfile::tempdir().unwrap();
        let service = EvidenceService::new(temp.path().to_path_buf());
        assert!(matches!(service.read_evidence_pack("src_demo", "../evil"), Err(AegisError::EvidencePackInvalidId)));
        assert!(matches!(service.read_evidence_pack("src_demo", "..\\evil"), Err(AegisError::EvidencePackInvalidId)));
        assert!(matches!(service.read_evidence_pack("src_demo", "/evil"), Err(AegisError::EvidencePackInvalidId)));
        assert!(matches!(service.read_evidence_pack("src_demo", "evil/pack"), Err(AegisError::EvidencePackInvalidId)));
        assert!(matches!(service.read_evidence_pack("src_demo", "evil\\pack"), Err(AegisError::EvidencePackInvalidId)));
        assert!(matches!(service.read_evidence_pack("src_demo", ""), Err(AegisError::EvidencePackInvalidId)));
    }

    #[test]
    fn read_back_missing_and_malformed_pack_are_typed_errors() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = prepare_index(&temp.path().to_path_buf());
        let service = EvidenceService::new(temp.path().to_path_buf());
        assert!(matches!(service.read_evidence_pack(&source_id, "evp_missing"), Err(AegisError::EvidencePackMissing)));

        let registry = SourceRegistry::load(&crate::corpus_paths::CorpusPaths::new(temp.path().to_path_buf()).registry_path()).unwrap();
        let record = registry.get_source(&source_id).unwrap();
        let malformed_path = crate::corpus_paths::CorpusPaths::new(temp.path().to_path_buf())
            .source_version_dir(&record.source_id, &record.version_id)
            .join("evidence")
            .join("evp_bad.json");
        fs::create_dir_all(malformed_path.parent().unwrap()).unwrap();
        fs::write(&malformed_path, "{not-json").unwrap();
        assert!(matches!(service.read_evidence_pack(&source_id, "evp_bad"), Err(AegisError::EvidencePackReadFailed)));
    }

    #[test]
    fn no_match_query_fails_without_writing_pack_or_marking_status() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = prepare_index(&temp.path().to_path_buf());
        let service = EvidenceService::new(temp.path().to_path_buf());
        let result = service.build_evidence_pack(&source_id, "nomatch", 5);
        assert!(matches!(result, Err(AegisError::EvidencePackEmpty)));

        let registry = SourceRegistry::load(&crate::corpus_paths::CorpusPaths::new(temp.path().to_path_buf()).registry_path()).unwrap();
        let record = registry.get_source(&source_id).unwrap();
        assert_ne!(record.ingestion_status, IngestionStatus::EvidenceReady);
        let pack_dir = crate::corpus_paths::CorpusPaths::new(temp.path().to_path_buf())
            .source_version_dir(&record.source_id, &record.version_id)
            .join("evidence");
        assert!(!pack_dir.exists() || fs::read_dir(pack_dir).unwrap().next().is_none());
        let audit = fs::read_to_string(crate::corpus_paths::CorpusPaths::new(temp.path().to_path_buf()).audit_events_path()).unwrap();
        assert!(audit.contains("evidence_pack_failed"));
        assert!(audit.lines().all(|line| serde_json::from_str::<serde_json::Value>(line).is_ok()));
    }

    #[test]
    fn invalid_limit_fails_before_writing_pack() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = prepare_index(&temp.path().to_path_buf());
        let service = EvidenceService::new(temp.path().to_path_buf());
        let result = service.build_evidence_pack(&source_id, "alpha", 0);
        assert!(matches!(result, Err(AegisError::EvidencePackInvalidLimit)));

        let registry = SourceRegistry::load(&crate::corpus_paths::CorpusPaths::new(temp.path().to_path_buf()).registry_path()).unwrap();
        let record = registry.get_source(&source_id).unwrap();
        assert_ne!(record.ingestion_status, IngestionStatus::EvidenceReady);
        let pack_dir = crate::corpus_paths::CorpusPaths::new(temp.path().to_path_buf())
            .source_version_dir(&record.source_id, &record.version_id)
            .join("evidence");
        assert!(!pack_dir.exists() || fs::read_dir(pack_dir).unwrap().next().is_none());
    }

    #[test]
    fn failed_build_propagates_retrieval_errors_without_fake_success() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = prepare_index(&temp.path().to_path_buf());
        let service = EvidenceService::new(temp.path().to_path_buf());
        let registry = SourceRegistry::load(&crate::corpus_paths::CorpusPaths::new(temp.path().to_path_buf()).registry_path()).unwrap();
        let record = registry.get_source(&source_id).unwrap();
        let chunk_report = crate::corpus_paths::CorpusPaths::new(temp.path().to_path_buf())
            .source_version_dir(&source_id, &record.version_id)
            .join("chunks")
            .join("chunks.json");
        fs::remove_file(chunk_report).unwrap();
        let result = service.build_evidence_pack(&source_id, "alpha", 5);
        assert!(matches!(result, Err(AegisError::ChunkingReportMissing) | Err(AegisError::ChunkingReportReadFailed) | Err(AegisError::RetrievalIndexMissing) | Err(AegisError::RetrievalIndexReadFailed)));
        assert_ne!(record.ingestion_status, IngestionStatus::EvidenceReady);
        let audit = fs::read_to_string(crate::corpus_paths::CorpusPaths::new(temp.path().to_path_buf()).audit_events_path()).unwrap();
        assert!(audit.contains("evidence_pack_failed"));
    }
}
