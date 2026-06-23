use crate::audit::{append_audit_event, AuditEvent, AuditEventType};
use crate::corpus_paths::CorpusPaths;
use crate::errors::{AegisError, AegisResult};
use crate::grounded_answer::{GroundedAnswer, GroundedStatement, GroundedStatementStatus, GroundedSupportLevel};
use crate::locators::CitationLocator;
use crate::source_metadata::IngestionStatus;
use crate::source_registry::SourceRegistry;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalAnswer {
    pub final_answer_id: String,
    pub grounded_answer_id: String,
    pub source_id: String,
    pub version_id: String,
    pub query: String,
    pub created_at: DateTime<Utc>,
    pub answer_mode: FinalAnswerMode,
    pub statement_count: usize,
    pub unsupported_count: usize,
    pub statements: Vec<FinalAnswerStatement>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FinalAnswerMode {
    ContractOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FinalAnswerStatement {
    pub statement_id: String,
    pub grounded_statement_id: String,
    pub status: FinalAnswerStatementStatus,
    pub text: String,
    pub claim_ids: Vec<String>,
    pub evidence_ids: Vec<String>,
    pub chunk_ids: Vec<String>,
    pub locators: Vec<CitationLocator>,
    pub support_level: FinalAnswerSupportLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalAnswerMetadata {
    pub final_answer_id: String,
    pub grounded_answer_id: String,
    pub statement_count: usize,
    pub unsupported_count: usize,
    pub needs_evidence_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FinalAnswerStatementStatus {
    Supported,
    NeedsEvidence,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FinalAnswerSupportLevel {
    DirectGroundedStatement,
    MissingEvidence,
}

pub struct FinalAnswerService {
    paths: CorpusPaths,
}

pub fn build_final_answer(root: impl Into<PathBuf>, source_id: &str, grounded_answer_id: &str) -> AegisResult<FinalAnswer> {
    FinalAnswerService::new(root).build_final_answer(source_id, grounded_answer_id)
}

pub fn read_final_answer(root: impl Into<PathBuf>, source_id: &str, final_answer_id: &str) -> AegisResult<FinalAnswer> {
    FinalAnswerService::new(root).read_final_answer(source_id, final_answer_id)
}

pub fn list_final_answers(root: impl Into<PathBuf>, source_id: &str) -> AegisResult<Vec<FinalAnswerMetadata>> {
    FinalAnswerService::new(root).list_final_answers(source_id)
}

pub fn build_final_answer_from_grounded_answer(grounded_answer: &GroundedAnswer) -> AegisResult<FinalAnswer> {
    if grounded_answer.source_id.trim().is_empty() {
        return Err(AegisError::FinalAnswerInputMissing);
    }
    if grounded_answer.grounded_answer_id.trim().is_empty() {
        return Err(AegisError::FinalAnswerInvalidId);
    }
    if grounded_answer.statements.is_empty() {
        return Err(AegisError::FinalAnswerEmptyGroundedAnswer);
    }
    let statements = grounded_answer
        .statements
        .iter()
        .map(final_statement_from_grounded_statement)
        .collect::<Vec<_>>();
    let final_answer_id = deterministic_final_answer_id(grounded_answer, &statements);
    Ok(FinalAnswer {
        final_answer_id,
        grounded_answer_id: grounded_answer.grounded_answer_id.clone(),
        source_id: grounded_answer.source_id.clone(),
        version_id: grounded_answer.version_id.clone(),
        query: grounded_answer.query.clone(),
        created_at: Utc::now(),
        answer_mode: FinalAnswerMode::ContractOnly,
        statement_count: statements.len(),
        unsupported_count: statements.iter().filter(|statement| statement.status != FinalAnswerStatementStatus::Supported).count(),
        statements,
        warnings: Vec::new(),
    })
}

impl FinalAnswerService {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { paths: CorpusPaths::new(root) }
    }

    pub fn build_final_answer(&self, source_id: &str, grounded_answer_id: &str) -> AegisResult<FinalAnswer> {
        self.paths.ensure_layout()?;
        if source_id.trim().is_empty() {
            return Err(AegisError::FinalAnswerInputMissing);
        }
        validate_final_answer_id(grounded_answer_id)?;
        let grounded_answer = crate::grounded_answer::read_grounded_answer(self.paths.root.clone(), source_id, grounded_answer_id)?;
        let final_answer = build_final_answer_from_grounded_answer(&grounded_answer)?;
        self.write_final_answer(&final_answer)?;
        self.mark_ready(&final_answer.source_id)?;
        self.append_audit(AuditEventType::GroundedAnswerBuilt, &final_answer.source_id, &final_answer.version_id, "final answer built")?;
        Ok(final_answer)
    }

    pub fn read_final_answer(&self, source_id: &str, final_answer_id: &str) -> AegisResult<FinalAnswer> {
        if source_id.trim().is_empty() {
            return Err(AegisError::FinalAnswerInputMissing);
        }
        validate_final_answer_id(final_answer_id)?;
        self.paths.ensure_layout()?;
        let registry = SourceRegistry::load(&self.paths.registry_path())?;
        let record = registry.get_source(source_id)?;
        let path = self.answer_path(&record.source_id, &record.version_id, final_answer_id);
        if !path.exists() {
            return Err(AegisError::FinalAnswerMissing);
        }
        let content = fs::read_to_string(&path).map_err(|_| AegisError::FinalAnswerReadFailed)?;
        serde_json::from_str(&content).map_err(|_| AegisError::FinalAnswerReadFailed)
    }

    pub fn list_final_answers(&self, source_id: &str) -> AegisResult<Vec<FinalAnswerMetadata>> {
        if source_id.trim().is_empty() {
            return Err(AegisError::FinalAnswerInputMissing);
        }
        let registry = SourceRegistry::load(&self.paths.registry_path())?;
        let record = registry.get_source(source_id)?;
        let final_answer_dir = self.paths.source_version_dir(&record.source_id, &record.version_id).join("final_answers");
        if !final_answer_dir.exists() {
            return Ok(Vec::new());
        }

        let mut items = Vec::new();
        for entry in fs::read_dir(&final_answer_dir).map_err(|_| AegisError::FinalAnswerReadFailed)? {
            let entry = entry.map_err(|_| AegisError::FinalAnswerReadFailed)?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            let content = fs::read_to_string(&path).map_err(|_| AegisError::FinalAnswerReadFailed)?;
            let answer: FinalAnswer = serde_json::from_str(&content).map_err(|_| AegisError::FinalAnswerReadFailed)?;
            items.push(FinalAnswerMetadata {
                final_answer_id: answer.final_answer_id,
                grounded_answer_id: answer.grounded_answer_id,
                statement_count: answer.statement_count,
                unsupported_count: answer.unsupported_count,
                needs_evidence_count: answer
                    .statements
                    .iter()
                    .filter(|statement| statement.status == FinalAnswerStatementStatus::NeedsEvidence)
                    .count(),
            });
        }
        items.sort_by(|left, right| left.final_answer_id.cmp(&right.final_answer_id));
        Ok(items)
    }

    fn write_final_answer(&self, final_answer: &FinalAnswer) -> AegisResult<()> {
        let path = self.answer_path(&final_answer.source_id, &final_answer.version_id, &final_answer.final_answer_id);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|_| AegisError::FinalAnswerWriteFailed)?;
        }
        let body = serde_json::to_string_pretty(final_answer)?;
        fs::write(&path, body).map_err(|_| AegisError::FinalAnswerWriteFailed)?;
        Ok(())
    }

    fn answer_path(&self, source_id: &str, version_id: &str, final_answer_id: &str) -> PathBuf {
        self.paths
            .source_version_dir(source_id, version_id)
            .join("final_answers")
            .join(format!("{final_answer_id}.json"))
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

fn final_statement_from_grounded_statement(statement: &GroundedStatement) -> FinalAnswerStatement {
    FinalAnswerStatement {
        statement_id: deterministic_final_statement_id(statement),
        grounded_statement_id: statement.statement_id.clone(),
        status: match statement.status {
            GroundedStatementStatus::Supported => FinalAnswerStatementStatus::Supported,
            GroundedStatementStatus::NeedsEvidence => FinalAnswerStatementStatus::NeedsEvidence,
            GroundedStatementStatus::Unsupported => FinalAnswerStatementStatus::Unsupported,
        },
        text: statement.text.clone(),
        claim_ids: statement.claim_ids.clone(),
        evidence_ids: statement.evidence_ids.clone(),
        chunk_ids: statement.chunk_ids.clone(),
        locators: statement.locators.clone(),
        support_level: match statement.support_level {
            GroundedSupportLevel::DirectClaim => FinalAnswerSupportLevel::DirectGroundedStatement,
            GroundedSupportLevel::MissingEvidence => FinalAnswerSupportLevel::MissingEvidence,
        },
    }
}

fn deterministic_final_answer_id(answer: &GroundedAnswer, statements: &[FinalAnswerStatement]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(answer.grounded_answer_id.as_bytes());
    hasher.update(answer.source_id.as_bytes());
    hasher.update(answer.version_id.as_bytes());
    hasher.update(answer.query.as_bytes());
    hasher.update(serde_json::to_string(&FinalAnswerMode::ContractOnly).unwrap().as_bytes());
    for statement in statements {
        hasher.update(statement.statement_id.as_bytes());
    }
    format!("fan_{:x}", hasher.finalize())
}

fn deterministic_final_statement_id(statement: &GroundedStatement) -> String {
    let mut hasher = Sha256::new();
    hasher.update(statement.statement_id.as_bytes());
    hasher.update(serde_json::to_string(&statement.status).unwrap().as_bytes());
    hasher.update(statement.text.as_bytes());
    hasher.update(serde_json::to_string(&statement.claim_ids).unwrap().as_bytes());
    hasher.update(serde_json::to_string(&statement.evidence_ids).unwrap().as_bytes());
    hasher.update(serde_json::to_string(&statement.chunk_ids).unwrap().as_bytes());
    hasher.update(serde_json::to_string(&statement.locators).unwrap().as_bytes());
    hasher.update(serde_json::to_string(&statement.support_level).unwrap().as_bytes());
    format!("fst_{:x}", hasher.finalize())
}

fn validate_final_answer_id(final_answer_id: &str) -> AegisResult<()> {
    if final_answer_id.trim().is_empty() {
        return Err(AegisError::FinalAnswerInvalidId);
    }
    if final_answer_id.contains('/') || final_answer_id.contains('\\') || final_answer_id.contains("..") {
        return Err(AegisError::FinalAnswerInvalidId);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grounded_answer::{build_grounded_answer, read_grounded_answer, GroundedAnswerMode, GroundedStatementStatus, GroundedSupportLevel, GroundedAnswerService};
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

    fn prepare_grounded(root: &PathBuf) -> (String, String, String, String) {
        let source_path = root.join("notes.md");
        fs::write(&source_path, "alpha beta gamma").unwrap();
        let authority = CorpusAuthority::new(root.clone());
        authority.register_source(source_path.to_string_lossy().to_string(), valid_metadata()).unwrap();
        let corpus = CorpusPaths::new(root.clone());
        let registry = SourceRegistry::load(&corpus.registry_path()).unwrap();
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
        let evidence = EvidenceService::new(root.clone()).build_evidence_pack(&record.source_id, "alpha", 5).unwrap();
        let draft = crate::answer_draft::AnswerDraftService::new(root.clone()).build_answer_draft(&record.source_id, &evidence.evidence_pack_id).unwrap();
        let grounded = GroundedAnswerService::new(root.clone()).build_grounded_answer(&record.source_id, &draft.answer_draft_id).unwrap();
        (record.source_id, record.version_id, draft.answer_draft_id, grounded.grounded_answer_id)
    }

    #[test]
    fn final_answer_serializes_with_required_fields_and_enum_values() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, version_id, draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let grounded = read_grounded_answer(temp.path().to_path_buf(), &source_id, &grounded_id).unwrap();
        let final_answer = build_final_answer_from_grounded_answer(&grounded).unwrap();
        let json = serde_json::to_value(&final_answer).unwrap();
        assert_eq!(json["final_answer_id"].as_str().unwrap().starts_with("fan_"), true);
        assert_eq!(json["answer_mode"], "contract_only");
        assert_eq!(json["statements"].as_array().unwrap().len(), 1);
        assert_eq!(json["statements"][0]["status"], "supported");
        assert_eq!(json["statements"][0]["support_level"], "direct_grounded_statement");
        assert_eq!(final_answer.source_id, source_id);
        assert_eq!(final_answer.version_id, version_id);
        assert_eq!(final_answer.grounded_answer_id, grounded_id);
        assert_eq!(final_answer.statement_count, 1);
        assert_eq!(final_answer.unsupported_count, 0);
        assert!(!draft_id.is_empty());
    }

    #[test]
    fn final_answer_builder_preserves_order_and_statuses() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, version_id, draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let grounded = read_grounded_answer(temp.path().to_path_buf(), &source_id, &grounded_id).unwrap();
        let mut modified = grounded.clone();
        modified.statements[0].status = GroundedStatementStatus::NeedsEvidence;
        modified.statements[0].support_level = GroundedSupportLevel::MissingEvidence;
        let final_answer = build_final_answer_from_grounded_answer(&modified).unwrap();
        assert_eq!(final_answer.statements.len(), 1);
        assert_eq!(final_answer.statements[0].text, modified.statements[0].text);
        assert_eq!(final_answer.statements[0].status, FinalAnswerStatementStatus::NeedsEvidence);
        assert_eq!(final_answer.statements[0].support_level, FinalAnswerSupportLevel::MissingEvidence);
        assert_eq!(final_answer.statement_count, 1);
        assert_eq!(final_answer.unsupported_count, 1);
        assert_eq!(final_answer.source_id, source_id);
        assert_eq!(final_answer.version_id, version_id);
        assert_eq!(final_answer.grounded_answer_id, grounded_id);
        assert!(!draft_id.is_empty());
    }

    #[test]
    fn final_answer_listing_returns_metadata_without_paths() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, _version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let answer = service.build_final_answer(&source_id, &grounded_id).unwrap();
        let listed = service.list_final_answers(&source_id).unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].final_answer_id, answer.final_answer_id);
        assert_eq!(listed[0].grounded_answer_id, answer.grounded_answer_id);
        assert_eq!(listed[0].statement_count, 1);
        assert_eq!(listed[0].unsupported_count, 0);
        assert_eq!(listed[0].needs_evidence_count, 0);
        let json = serde_json::to_string(&listed[0]).unwrap();
        assert!(!json.contains(temp.path().to_string_lossy().as_ref()));
    }

    #[test]
    fn final_answer_listing_orders_results_deterministically() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let first = service.build_final_answer(&source_id, &grounded_id).unwrap();
        let mut second = service.build_final_answer(&source_id, &grounded_id).unwrap();
        second.final_answer_id = "fan_0000000000000000000000000000000000000000000000000000000000000001".to_string();
        second.grounded_answer_id = first.grounded_answer_id.clone();
        second.statement_count = 2;
        second.unsupported_count = 1;
        second.statements.push(FinalAnswerStatement {
            statement_id: "fst_demo".to_string(),
            grounded_statement_id: "gst_demo".to_string(),
            status: FinalAnswerStatementStatus::NeedsEvidence,
            text: "Needs evidence".to_string(),
            claim_ids: vec!["dcl_demo".to_string()],
            evidence_ids: vec!["evi_demo".to_string()],
            chunk_ids: vec!["chk_demo".to_string()],
            locators: vec![locator()],
            support_level: FinalAnswerSupportLevel::MissingEvidence,
        });
        let path = CorpusPaths::new(temp.path().to_path_buf())
            .source_version_dir(&source_id, &version_id)
            .join("final_answers")
            .join("fan_0000000000000000000000000000000000000000000000000000000000000001.json");
        fs::write(&path, serde_json::to_string_pretty(&second).unwrap()).unwrap();

        let listed = service.list_final_answers(&source_id).unwrap();
        assert_eq!(listed.len(), 2);
        assert!(listed[0].final_answer_id <= listed[1].final_answer_id);
        assert!(listed.iter().any(|item| item.final_answer_id == first.final_answer_id));
        assert!(listed.iter().any(|item| item.final_answer_id == "fan_0000000000000000000000000000000000000000000000000000000000000001"));
    }

    #[test]
    fn final_answer_listing_counts_mixed_statuses_correctly() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let mut answer = service.build_final_answer(&source_id, &grounded_id).unwrap();
        answer.final_answer_id = "fan_mixed".to_string();
        answer.statement_count = 3;
        answer.unsupported_count = 2;
        answer.statements.push(FinalAnswerStatement {
            statement_id: "fst_needs".to_string(),
            grounded_statement_id: "gst_needs".to_string(),
            status: FinalAnswerStatementStatus::NeedsEvidence,
            text: "Needs evidence".to_string(),
            claim_ids: vec!["dcl_needs".to_string()],
            evidence_ids: vec!["evi_needs".to_string()],
            chunk_ids: vec!["chk_needs".to_string()],
            locators: vec![locator()],
            support_level: FinalAnswerSupportLevel::MissingEvidence,
        });
        answer.statements.push(FinalAnswerStatement {
            statement_id: "fst_unsupported".to_string(),
            grounded_statement_id: "gst_unsupported".to_string(),
            status: FinalAnswerStatementStatus::Unsupported,
            text: "Unsupported".to_string(),
            claim_ids: vec!["dcl_unsupported".to_string()],
            evidence_ids: vec!["evi_unsupported".to_string()],
            chunk_ids: vec!["chk_unsupported".to_string()],
            locators: vec![locator()],
            support_level: FinalAnswerSupportLevel::MissingEvidence,
        });
        let path = CorpusPaths::new(temp.path().to_path_buf())
            .source_version_dir(&source_id, &version_id)
            .join("final_answers")
            .join("fan_mixed.json");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, serde_json::to_string_pretty(&answer).unwrap()).unwrap();

        let listed = service.list_final_answers(&source_id).unwrap();
        let metadata = listed.iter().find(|item| item.final_answer_id == "fan_mixed").unwrap();
        assert_eq!(metadata.statement_count, 3);
        assert_eq!(metadata.unsupported_count, 2);
        assert_eq!(metadata.needs_evidence_count, 1);
    }

    #[test]
    fn final_answer_listing_is_empty_when_directory_missing() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, _version_id, _draft_id, _grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let listed = service.list_final_answers(&source_id).unwrap();
        assert!(listed.is_empty());
        let final_answer_dir = CorpusPaths::new(temp.path().to_path_buf())
            .source_version_dir(&source_id, "srcv_demo")
            .join("final_answers");
        assert!(!final_answer_dir.exists());
    }

    #[test]
    fn final_answer_listing_rejects_empty_source_id() {
        let temp = tempfile::tempdir().unwrap();
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        assert!(matches!(service.list_final_answers(""), Err(AegisError::FinalAnswerInputMissing)));
    }

    #[test]
    fn final_answer_listing_reports_missing_source() {
        let temp = tempfile::tempdir().unwrap();
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        assert!(matches!(service.list_final_answers("src_missing"), Err(AegisError::SourceNotFound(_))));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn final_answer_listing_rejects_traversal_like_source_id() {
        let temp = tempfile::tempdir().unwrap();
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        assert!(matches!(service.list_final_answers("../src"), Err(AegisError::SourceNotFound(_))));
        assert!(matches!(service.list_final_answers("src\\.."), Err(AegisError::SourceNotFound(_))));
        assert!(matches!(service.list_final_answers("src/.."), Err(AegisError::SourceNotFound(_))));
    }

    #[test]
    fn final_answer_listing_returns_typed_error_for_malformed_file() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let answer = service.build_final_answer(&source_id, &grounded_id).unwrap();
        let bad_path = CorpusPaths::new(temp.path().to_path_buf())
            .source_version_dir(&source_id, &version_id)
            .join("final_answers")
            .join("fan_bad.json");
        fs::write(&bad_path, "{not-json").unwrap();
        assert!(matches!(service.list_final_answers(&source_id), Err(AegisError::FinalAnswerReadFailed)));
        assert!(bad_path.exists());
        assert!(answer.final_answer_id.starts_with("fan_"));
    }

    #[test]
    fn final_answer_builder_rejects_empty_grounded_answer() {
        let grounded = GroundedAnswer {
            grounded_answer_id: "gan_empty".to_string(),
            answer_draft_id: "adr_x".to_string(),
            evidence_pack_id: "evp_x".to_string(),
            source_id: "src_x".to_string(),
            version_id: "srcv_x".to_string(),
            query: "alpha".to_string(),
            created_at: Utc::now(),
            answer_mode: GroundedAnswerMode::ContractOnly,
            statement_count: 0,
            unsupported_count: 0,
            statements: Vec::new(),
            warnings: Vec::new(),
        };
        assert!(matches!(build_final_answer_from_grounded_answer(&grounded), Err(AegisError::FinalAnswerEmptyGroundedAnswer)));
    }

    #[test]
    fn final_answer_adapter_builds_and_reads_round_trip() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let final_answer = build_final_answer(temp.path().to_path_buf(), &source_id, &grounded_id).unwrap();
        assert!(final_answer.final_answer_id.starts_with("fan_"));
        assert_eq!(final_answer.source_id, source_id);
        assert_eq!(final_answer.version_id, version_id);
        assert_eq!(final_answer.statements.len(), 1);
        let read_back = read_final_answer(temp.path().to_path_buf(), &source_id, &final_answer.final_answer_id).unwrap();
        assert_eq!(read_back.final_answer_id, final_answer.final_answer_id);
        assert_eq!(read_back.statements[0].status, FinalAnswerStatementStatus::Supported);
        let registry = SourceRegistry::load(&CorpusPaths::new(temp.path().to_path_buf()).registry_path()).unwrap();
        assert_eq!(registry.get_source(&source_id).unwrap().ingestion_status, IngestionStatus::GroundedAnswerReady);
    }

    #[test]
    fn final_answer_adapter_rejects_invalid_ids_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        assert!(matches!(read_final_answer(temp.path().to_path_buf(), "src_demo", ""), Err(AegisError::FinalAnswerInvalidId)));
        assert!(matches!(read_final_answer(temp.path().to_path_buf(), "src_demo", "../x"), Err(AegisError::FinalAnswerInvalidId)));
        assert!(matches!(read_final_answer(temp.path().to_path_buf(), "src_demo", "..\\x"), Err(AegisError::FinalAnswerInvalidId)));
        assert!(matches!(read_final_answer(temp.path().to_path_buf(), "src_demo", "x/y"), Err(AegisError::FinalAnswerInvalidId)));
        assert!(matches!(read_final_answer(temp.path().to_path_buf(), "src_demo", "x\\y"), Err(AegisError::FinalAnswerInvalidId)));
    }

    #[test]
    fn final_answer_adapter_maps_missing_and_malformed_read_back() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let final_answer = build_final_answer(temp.path().to_path_buf(), &source_id, &grounded_id).unwrap();
        assert!(matches!(read_final_answer(temp.path().to_path_buf(), &source_id, "fan_missing"), Err(AegisError::FinalAnswerMissing)));
        let corpus = CorpusPaths::new(temp.path().to_path_buf());
        let bad_path = corpus.source_version_dir(&source_id, &version_id).join("final_answers").join("fan_bad.json");
        fs::create_dir_all(bad_path.parent().unwrap()).unwrap();
        fs::write(&bad_path, "{not-json").unwrap();
        assert!(matches!(read_final_answer(temp.path().to_path_buf(), &source_id, "fan_bad"), Err(AegisError::FinalAnswerReadFailed)));
        assert!(final_answer.final_answer_id.starts_with("fan_"));
    }

    #[test]
    fn final_answer_adapter_failure_does_not_write_answer() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let corpus = CorpusPaths::new(temp.path().to_path_buf());
        let result = build_final_answer(temp.path().to_path_buf(), &source_id, "fan_missing");
        assert!(matches!(result, Err(AegisError::GroundedAnswerMissing)));
        let final_answer_dir = corpus.source_version_dir(&source_id, &version_id).join("final_answers");
        assert!(!final_answer_dir.exists() || fs::read_dir(final_answer_dir).unwrap().next().is_none());
        let registry = SourceRegistry::load(&corpus.registry_path()).unwrap();
        assert_eq!(registry.get_source(&source_id).unwrap().ingestion_status, IngestionStatus::GroundedAnswerReady);
        assert!(matches!(read_grounded_answer(temp.path().to_path_buf(), &source_id, &grounded_id), Ok(_)));
    }

    #[test]
    fn final_answer_adapter_rejects_empty_source_id() {
        let temp = tempfile::tempdir().unwrap();
        assert!(matches!(build_final_answer(temp.path().to_path_buf(), "", "gan_x"), Err(AegisError::FinalAnswerInputMissing)));
        assert!(matches!(read_final_answer(temp.path().to_path_buf(), "", "fan_x"), Err(AegisError::FinalAnswerInputMissing)));
    }

    #[test]
    fn final_answer_deterministic_id_is_stable_for_identical_grounded_answer() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, _version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let grounded = read_grounded_answer(temp.path().to_path_buf(), &source_id, &grounded_id).unwrap();
        let first = build_final_answer_from_grounded_answer(&grounded).unwrap();
        let second = build_final_answer_from_grounded_answer(&grounded).unwrap();
        assert_eq!(first.final_answer_id, second.final_answer_id);
        assert_eq!(first.statements[0].statement_id, second.statements[0].statement_id);
    }

    #[test]
    fn final_answer_deterministic_id_changes_when_statement_changes() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, _version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let grounded = read_grounded_answer(temp.path().to_path_buf(), &source_id, &grounded_id).unwrap();
        let baseline = build_final_answer_from_grounded_answer(&grounded).unwrap();
        let mut changed = grounded.clone();
        changed.statements[0].text = "Evidence states: changed".to_string();
        let revised = build_final_answer_from_grounded_answer(&changed).unwrap();
        assert_ne!(baseline.final_answer_id, revised.final_answer_id);
        assert_ne!(baseline.statements[0].statement_id, revised.statements[0].statement_id);
    }

    #[test]
    fn final_answer_read_failure_does_not_create_side_effects() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let corpus = CorpusPaths::new(temp.path().to_path_buf());
        let read_result = read_final_answer(temp.path().to_path_buf(), &source_id, "fan_missing");
        assert!(matches!(read_result, Err(AegisError::FinalAnswerMissing)));
        let final_answer_dir = corpus.source_version_dir(&source_id, &version_id).join("final_answers");
        assert!(!final_answer_dir.exists());
        assert!(matches!(read_grounded_answer(temp.path().to_path_buf(), &source_id, &grounded_id), Ok(_)));
    }

    #[test]
    fn pipeline_smoke_persists_grounded_to_final_contract_chain() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, version_id, _draft_id, _grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let grounded_service = GroundedAnswerService::new(temp.path().to_path_buf());
        let final_service = FinalAnswerService::new(temp.path().to_path_buf());

        let draft_service = crate::answer_draft::AnswerDraftService::new(temp.path().to_path_buf());
        let evidence_service = EvidenceService::new(temp.path().to_path_buf());
        let evidence = evidence_service.build_evidence_pack(&source_id, "alpha", 5).unwrap();
        let draft = draft_service.build_answer_draft(&source_id, &evidence.evidence_pack_id).unwrap();

        let grounded_a = grounded_service.build_grounded_answer(&source_id, &draft.answer_draft_id).unwrap();
        let grounded_b = read_grounded_answer(temp.path().to_path_buf(), &source_id, &grounded_a.grounded_answer_id).unwrap();
        let grounded_c = grounded_service.build_grounded_answer(&source_id, &draft.answer_draft_id).unwrap();
        assert_eq!(grounded_a.grounded_answer_id, grounded_c.grounded_answer_id);
        assert_eq!(grounded_a.statements.len(), grounded_b.statements.len());
        assert_eq!(grounded_b.statements.len(), 1);
        assert_eq!(grounded_b.statements[0].status, GroundedStatementStatus::Supported);
        assert_eq!(grounded_b.statements[0].claim_ids.len(), 1);
        assert_eq!(grounded_b.statements[0].evidence_ids.len(), 1);
        assert_eq!(grounded_b.statements[0].chunk_ids.len(), 1);
        assert_eq!(grounded_b.statements[0].locators.len(), 1);

        let final_a = final_service.build_final_answer(&source_id, &grounded_b.grounded_answer_id).unwrap();
        let final_b = read_final_answer(temp.path().to_path_buf(), &source_id, &final_a.final_answer_id).unwrap();
        let final_c = final_service.build_final_answer(&source_id, &grounded_b.grounded_answer_id).unwrap();
        assert_eq!(final_a.final_answer_id, final_c.final_answer_id);
        assert_eq!(final_a.final_answer_id.starts_with("fan_"), true);
        assert_eq!(final_b.final_answer_id, final_a.final_answer_id);
        assert_eq!(final_a.source_id, source_id);
        assert_eq!(final_a.version_id, version_id);
        assert_eq!(final_a.statement_count, grounded_b.statement_count);
        assert_eq!(final_a.statement_count, final_b.statement_count);
        assert_eq!(final_a.statements.len(), grounded_b.statements.len());
        assert_eq!(final_a.statements.len(), 1);
        assert_eq!(final_a.statements[0].status, FinalAnswerStatementStatus::Supported);
        assert_eq!(final_a.statements[0].claim_ids, grounded_b.statements[0].claim_ids);
        assert_eq!(final_a.statements[0].evidence_ids, grounded_b.statements[0].evidence_ids);
        assert_eq!(final_a.statements[0].chunk_ids, grounded_b.statements[0].chunk_ids);
        assert_eq!(final_a.statements[0].locators, grounded_b.statements[0].locators);
        assert_eq!(final_a.statements[0].text, grounded_b.statements[0].text);
        assert_eq!(final_a.unsupported_count, 0);
        assert_eq!(final_b.statements[0].status, FinalAnswerStatementStatus::Supported);
        assert!(!final_a.final_answer_id.contains('/'));
        assert!(!final_a.final_answer_id.contains('\\'));
        assert!(!grounded_b.grounded_answer_id.contains('/'));
        assert!(!grounded_b.grounded_answer_id.contains('\\'));
    }

    #[test]
    fn pipeline_smoke_preserves_needs_evidence_and_unsupported_statements() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, version_id, draft_id, _grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let mut draft = crate::answer_draft::AnswerDraftService::new(temp.path().to_path_buf())
            .read_answer_draft(&source_id, &draft_id)
            .unwrap();
        draft.claims[0].status = crate::answer_draft::DraftClaimStatus::NeedsEvidence;
        draft.claims[0].confidence = crate::answer_draft::DraftClaimConfidence::MissingEvidence;
        let corpus = CorpusPaths::new(temp.path().to_path_buf());
        let draft_path = corpus.source_version_dir(&source_id, &version_id).join("answer_drafts").join(format!("{}.json", draft.answer_draft_id));
        fs::write(draft_path, serde_json::to_string_pretty(&draft).unwrap()).unwrap();

        let grounded = GroundedAnswerService::new(temp.path().to_path_buf())
            .build_grounded_answer(&source_id, &draft_id)
            .unwrap();
        assert_eq!(grounded.statements[0].status, GroundedStatementStatus::NeedsEvidence);
        assert_eq!(grounded.statements[0].support_level, GroundedSupportLevel::MissingEvidence);

        let final_answer = FinalAnswerService::new(temp.path().to_path_buf())
            .build_final_answer(&source_id, &grounded.grounded_answer_id)
            .unwrap();
        assert_eq!(final_answer.statements[0].status, FinalAnswerStatementStatus::NeedsEvidence);
        assert_eq!(final_answer.statements[0].support_level, FinalAnswerSupportLevel::MissingEvidence);
        assert_eq!(final_answer.statements[0].claim_ids, grounded.statements[0].claim_ids);
        assert_eq!(final_answer.statements[0].evidence_ids, grounded.statements[0].evidence_ids);
        assert_eq!(final_answer.statements[0].locators, grounded.statements[0].locators);
        assert_eq!(final_answer.unsupported_count, 1);
    }

    #[test]
    fn final_answer_adapter_preserves_supported_needs_evidence_and_unsupported_statuses() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, version_id, draft_id, _grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let mut draft = crate::answer_draft::AnswerDraftService::new(temp.path().to_path_buf())
            .read_answer_draft(&source_id, &draft_id)
            .unwrap();
        draft.claims[0].status = crate::answer_draft::DraftClaimStatus::NeedsEvidence;
        draft.claims[0].confidence = crate::answer_draft::DraftClaimConfidence::MissingEvidence;
        let corpus = CorpusPaths::new(temp.path().to_path_buf());
        let draft_path = corpus.source_version_dir(&source_id, &version_id).join("answer_drafts").join(format!("{}.json", draft.answer_draft_id));
        fs::write(draft_path, serde_json::to_string_pretty(&draft).unwrap()).unwrap();
        let needs_evidence = build_grounded_answer(temp.path().to_path_buf(), &source_id, &draft_id).unwrap();
        let final_answer = build_final_answer_from_grounded_answer(&needs_evidence).unwrap();
        assert_eq!(final_answer.statements[0].status, FinalAnswerStatementStatus::NeedsEvidence);

        let mut unsupported = needs_evidence.clone();
        unsupported.statements[0].status = GroundedStatementStatus::Unsupported;
        unsupported.statements[0].support_level = GroundedSupportLevel::MissingEvidence;
        let final_answer = build_final_answer_from_grounded_answer(&unsupported).unwrap();
        assert_eq!(final_answer.statements[0].status, FinalAnswerStatementStatus::Unsupported);
        assert_eq!(final_answer.statements[0].claim_ids.len(), 1);
        assert_eq!(final_answer.statements[0].evidence_ids, unsupported.statements[0].evidence_ids);
        assert_eq!(final_answer.statements[0].locators, unsupported.statements[0].locators);
    }
}
