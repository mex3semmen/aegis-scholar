use crate::audit::{append_audit_event, AuditEvent, AuditEventType};
use crate::corpus_paths::CorpusPaths;
use crate::errors::{AegisError, AegisResult};
use crate::grounded_answer::{GroundedAnswer, GroundedStatement, GroundedStatementStatus, GroundedSupportLevel};
use crate::locators::CitationLocator;
use crate::source_metadata::IngestionStatus;
use crate::source_registry::SourceRegistry;
use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

pub const ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION: &str = "answer_artifact_export.v1";
pub const ANSWER_ARTIFACT_EXPORT_INTEGRITY_ALGORITHM: &str = "sha256";

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerArtifactOverview {
    pub source_id: String,
    pub draft_count: usize,
    pub grounded_answer_count: usize,
    pub final_answer_count: usize,
    pub final_answers: Vec<FinalAnswerMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerArtifactSourceMetadata {
    pub source_id: String,
    pub draft_count: usize,
    pub grounded_answer_count: usize,
    pub final_answer_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerArtifactHealth {
    pub source_count: usize,
    pub draft_count: usize,
    pub grounded_answer_count: usize,
    pub final_answer_count: usize,
    pub malformed_final_answer_count: usize,
    pub unsupported_statement_count: usize,
    pub needs_evidence_statement_count: usize,
    pub sources: Vec<AnswerArtifactSourceHealth>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerArtifactIssue {
    pub source_id: String,
    pub issue_kind: AnswerArtifactIssueKind,
    pub final_answer_id: Option<String>,
    pub grounded_answer_id: Option<String>,
    pub statement_index: Option<usize>,
    pub statement_status: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerArtifactExportManifest {
    #[serde(default)]
    pub schema_version: String,
    pub source_count: usize,
    pub draft_count: usize,
    pub grounded_answer_count: usize,
    pub final_answer_count: usize,
    pub issue_count: usize,
    pub sources: Vec<AnswerArtifactExportSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerArtifactExportSource {
    pub source_id: String,
    pub draft_count: usize,
    pub grounded_answer_count: usize,
    pub final_answer_count: usize,
    pub final_answers: Vec<FinalAnswerMetadata>,
    pub issue_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerArtifactExportIssues {
    #[serde(default)]
    pub schema_version: String,
    pub issues: Vec<AnswerArtifactIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerArtifactExportResult {
    pub schema_version: String,
    pub manifest: AnswerArtifactExportManifest,
    pub integrity: AnswerArtifactExportIntegrity,
    pub exported_source_count: usize,
    pub exported_draft_count: usize,
    pub exported_grounded_answer_count: usize,
    pub exported_final_answer_count: usize,
    pub exported_issue_count: usize,
    pub export_id: String,
    pub written_files: Vec<ExportedArtifactFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedArtifactFile {
    pub relative_path: String,
    pub artifact_kind: ExportedArtifactKind,
    pub source_id: Option<String>,
    pub artifact_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnswerArtifactExportIntegrity {
    #[serde(default)]
    pub schema_version: String,
    pub algorithm: String,
    pub files: Vec<AnswerArtifactExportIntegrityFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnswerArtifactExportIntegrityFile {
    pub relative_path: String,
    pub byte_count: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnswerArtifactExportSummary {
    #[serde(default)]
    pub schema_version: String,
    pub export_id: String,
    pub generated_from: String,
    pub export_scope: String,
    pub non_goals: Vec<String>,
    pub source_count: usize,
    pub draft_count: usize,
    pub grounded_answer_count: usize,
    pub final_answer_count: usize,
    pub issue_count: usize,
    pub issue_kinds: Vec<AnswerArtifactExportIssueKindCount>,
    pub sources: Vec<AnswerArtifactExportSummarySource>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnswerArtifactExportSummarySource {
    pub source_id: String,
    pub draft_count: usize,
    pub grounded_answer_count: usize,
    pub final_answer_count: usize,
    pub issue_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnswerArtifactExportIssueKindCount {
    pub issue_kind: AnswerArtifactIssueKind,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerArtifactExportBundleInspection {
    pub schema_version: Option<String>,
    pub manifest_schema_version: Option<String>,
    pub issues_schema_version: Option<String>,
    pub summary_schema_version: Option<String>,
    pub integrity_schema_version: Option<String>,
    pub integrity_algorithm: Option<String>,
    pub inspection_summary: AnswerArtifactExportBundleInspectionSummary,
    pub report_preview: AnswerArtifactExportBundleInspectionReportPreview,
    pub has_manifest: bool,
    pub has_issues: bool,
    pub has_summary: bool,
    pub has_integrity: bool,
    pub is_consistent: bool,
    pub issue_count: usize,
    pub warning_count: usize,
    pub errors: Vec<AnswerArtifactExportBundleInspectionIssue>,
    pub warnings: Vec<AnswerArtifactExportBundleInspectionIssue>,
    pub manifest_counts: Option<AnswerArtifactExportManifest>,
    pub summary_counts: Option<AnswerArtifactExportSummary>,
    pub issue_kind_counts: Option<Vec<AnswerArtifactExportIssueKindCount>>,
    pub integrity_counts: Option<AnswerArtifactExportIntegrity>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnswerArtifactExportBundleInspectionSummary {
    pub is_consistent: bool,
    pub schema_supported: bool,
    pub integrity_verified: bool,
    pub issue_count: usize,
    pub warning_count: usize,
    pub issue_counts_by_kind: Vec<AnswerArtifactExportBundleInspectionIssueKindCount>,
    pub checked_file_count: usize,
    pub integrity_file_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnswerArtifactExportBundleInspectionReportPreview {
    pub title: String,
    pub schema_version: String,
    pub is_consistent: bool,
    pub integrity_verified: bool,
    pub issue_count: usize,
    pub warning_count: usize,
    pub issue_counts_by_kind: Vec<AnswerArtifactExportBundleInspectionIssueKindCount>,
    pub sections: Vec<AnswerArtifactExportBundleInspectionReportSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnswerArtifactExportBundleInspectionReportSection {
    pub heading: String,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnswerArtifactExportBundleInspectionIssueKindCount {
    pub kind: AnswerArtifactExportBundleInspectionIssueKind,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnswerArtifactExportBundleInspectionIssue {
    pub kind: AnswerArtifactExportBundleInspectionIssueKind,
    pub message: String,
    pub relative_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnswerArtifactExportBundleInspectionIssueKind {
    MissingManifest,
    ManifestReadFailed,
    MissingIssues,
    IssuesReadFailed,
    MissingSummary,
    SummaryReadFailed,
    MissingIntegrity,
    IntegrityReadFailed,
    IntegritySchemaVersionMissing,
    IntegritySchemaVersionUnsupported,
    IntegrityAlgorithmMissing,
    IntegrityAlgorithmUnsupported,
    IntegrityDuplicatePath,
    IntegrityPathInvalid,
    IntegrityMissingFile,
    IntegrityByteCountMismatch,
    IntegrityDigestMismatch,
    SchemaVersionMissing,
    SchemaVersionUnsupported,
    SchemaVersionMismatch,
    SummaryCountsMismatch,
    SummaryIssueCountMismatch,
    SummaryIssueKindCountsMismatch,
    SummaryExportIdMismatch,
    SummaryMetadataMismatch,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExportedArtifactKind {
    Manifest,
    Issues,
    Summary,
    Integrity,
    AnswerDraft,
    GroundedAnswer,
    FinalAnswer,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AnswerArtifactIssueKind {
    MalformedFinalAnswer,
    UnsupportedStatement,
    NeedsEvidenceStatement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnswerArtifactSourceHealth {
    pub source_id: String,
    pub draft_count: usize,
    pub grounded_answer_count: usize,
    pub final_answer_count: usize,
    pub malformed_final_answer_count: usize,
    pub unsupported_statement_count: usize,
    pub needs_evidence_statement_count: usize,
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

pub fn get_answer_artifact_overview(root: impl Into<PathBuf>, source_id: &str) -> AegisResult<AnswerArtifactOverview> {
    FinalAnswerService::new(root).get_answer_artifact_overview(source_id)
}

pub fn list_answer_artifact_sources(root: impl Into<PathBuf>) -> AegisResult<Vec<AnswerArtifactSourceMetadata>> {
    FinalAnswerService::new(root).list_answer_artifact_sources()
}

pub fn get_answer_artifact_health(root: impl Into<PathBuf>) -> AegisResult<AnswerArtifactHealth> {
    FinalAnswerService::new(root).get_answer_artifact_health()
}

pub fn list_answer_artifact_issues(root: impl Into<PathBuf>) -> AegisResult<Vec<AnswerArtifactIssue>> {
    FinalAnswerService::new(root).list_answer_artifact_issues()
}

pub fn get_answer_artifact_export_manifest(root: impl Into<PathBuf>) -> AegisResult<AnswerArtifactExportManifest> {
    FinalAnswerService::new(root).get_answer_artifact_export_manifest()
}

pub fn inspect_answer_artifact_export_bundle(export_root: impl Into<PathBuf>) -> AegisResult<AnswerArtifactExportBundleInspection> {
    inspect_answer_artifact_export_bundle_impl(export_root.into())
}

pub fn export_answer_artifacts(root: impl Into<PathBuf>, export_root: impl Into<PathBuf>) -> AegisResult<AnswerArtifactExportResult> {
    FinalAnswerService::new(root).export_answer_artifacts(export_root)
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

    pub fn get_answer_artifact_overview(&self, source_id: &str) -> AegisResult<AnswerArtifactOverview> {
        if source_id.trim().is_empty() {
            return Err(AegisError::FinalAnswerInputMissing);
        }
        let registry = SourceRegistry::load(&self.paths.registry_path())?;
        let record = registry.get_source(source_id)?;
        let version_dir = self.paths.source_version_dir(&record.source_id, &record.version_id);
        let draft_count = count_json_files(version_dir.join("answer_drafts"))?;
        let grounded_answer_count = count_json_files(version_dir.join("grounded_answers"))?;
        let final_answers = self.list_final_answers(source_id)?;
        Ok(AnswerArtifactOverview {
            source_id: record.source_id,
            draft_count,
            grounded_answer_count,
            final_answer_count: final_answers.len(),
            final_answers,
        })
    }

    pub fn list_answer_artifact_sources(&self) -> AegisResult<Vec<AnswerArtifactSourceMetadata>> {
        let registry = SourceRegistry::load(&self.paths.registry_path())?;
        let mut sources = Vec::new();
        for record in registry.list_sources() {
            let version_dir = self.paths.source_version_dir(&record.source_id, &record.version_id);
            let draft_count = count_json_files(version_dir.join("answer_drafts"))?;
            let grounded_answer_count = count_json_files(version_dir.join("grounded_answers"))?;
            let final_answers = self.list_final_answers(&record.source_id)?;
            if draft_count == 0 && grounded_answer_count == 0 && final_answers.is_empty() {
                continue;
            }
            sources.push(AnswerArtifactSourceMetadata {
                source_id: record.source_id,
                draft_count,
                grounded_answer_count,
                final_answer_count: final_answers.len(),
            });
        }
        sources.sort_by(|left, right| left.source_id.cmp(&right.source_id));
        Ok(sources)
    }

    pub fn get_answer_artifact_health(&self) -> AegisResult<AnswerArtifactHealth> {
        let registry = SourceRegistry::load(&self.paths.registry_path())?;
        let mut sources = Vec::new();
        let mut source_count = 0;
        let mut draft_count = 0;
        let mut grounded_answer_count = 0;
        let mut final_answer_count = 0;
        let mut malformed_final_answer_count = 0;
        let mut unsupported_statement_count = 0;
        let mut needs_evidence_statement_count = 0;

        for record in registry.list_sources() {
            let counts = answer_artifact_counts_for_source(self, &record.source_id, &record.version_id)?;
            if counts.is_empty() {
                continue;
            }
            source_count += 1;
            draft_count += counts.draft_count;
            grounded_answer_count += counts.grounded_answer_count;
            final_answer_count += counts.final_answer_count;
            malformed_final_answer_count += counts.malformed_final_answer_count;
            unsupported_statement_count += counts.unsupported_statement_count;
            needs_evidence_statement_count += counts.needs_evidence_statement_count;
            sources.push(AnswerArtifactSourceHealth {
                source_id: record.source_id,
                draft_count: counts.draft_count,
                grounded_answer_count: counts.grounded_answer_count,
                final_answer_count: counts.final_answer_count,
                malformed_final_answer_count: counts.malformed_final_answer_count,
                unsupported_statement_count: counts.unsupported_statement_count,
                needs_evidence_statement_count: counts.needs_evidence_statement_count,
            });
        }

        sources.sort_by(|left, right| left.source_id.cmp(&right.source_id));
        Ok(AnswerArtifactHealth {
            source_count,
            draft_count,
            grounded_answer_count,
            final_answer_count,
            malformed_final_answer_count,
            unsupported_statement_count,
            needs_evidence_statement_count,
            sources,
        })
    }

    pub fn list_answer_artifact_issues(&self) -> AegisResult<Vec<AnswerArtifactIssue>> {
        let registry = SourceRegistry::load(&self.paths.registry_path())?;
        let mut issues = Vec::new();

        for record in registry.list_sources() {
            let version_dir = self.paths.source_version_dir(&record.source_id, &record.version_id);
            let final_answer_dir = version_dir.join("final_answers");
            if !final_answer_dir.exists() {
                continue;
            }

            for entry in fs::read_dir(&final_answer_dir).map_err(|_| AegisError::FinalAnswerReadFailed)? {
                let entry = entry.map_err(|_| AegisError::FinalAnswerReadFailed)?;
                let path = entry.path();
                if path.extension().and_then(|value| value.to_str()) != Some("json") {
                    continue;
                }
                let content = fs::read_to_string(&path).map_err(|_| AegisError::FinalAnswerReadFailed)?;
                match serde_json::from_str::<FinalAnswer>(&content) {
                    Ok(answer) => {
                        issues.extend(answer.statements.iter().enumerate().filter_map(|(index, statement)| {
                            let issue_kind = match statement.status {
                                FinalAnswerStatementStatus::Supported => return None,
                                FinalAnswerStatementStatus::NeedsEvidence => AnswerArtifactIssueKind::NeedsEvidenceStatement,
                                FinalAnswerStatementStatus::Unsupported => AnswerArtifactIssueKind::UnsupportedStatement,
                            };
                            Some(AnswerArtifactIssue {
                                source_id: record.source_id.clone(),
                                issue_kind,
                                final_answer_id: Some(answer.final_answer_id.clone()),
                                grounded_answer_id: Some(answer.grounded_answer_id.clone()),
                                statement_index: Some(index),
                                statement_status: Some(serde_json::to_string(&statement.status).unwrap().trim_matches('"').to_string()),
                                message: match statement.status {
                                    FinalAnswerStatementStatus::NeedsEvidence => "statement needs evidence".to_string(),
                                    FinalAnswerStatementStatus::Unsupported => "statement is unsupported".to_string(),
                                    FinalAnswerStatementStatus::Supported => unreachable!(),
                                },
                            })
                        }));
                    }
                    Err(_) => {
                        issues.push(AnswerArtifactIssue {
                            source_id: record.source_id.clone(),
                            issue_kind: AnswerArtifactIssueKind::MalformedFinalAnswer,
                            final_answer_id: None,
                            grounded_answer_id: None,
                            statement_index: None,
                            statement_status: None,
                            message: "final answer could not be read".to_string(),
                        });
                    }
                }
            }
        }

        issues.sort_by(|left, right| {
            (
                &left.source_id,
                issue_kind_rank(&left.issue_kind),
                left.final_answer_id.as_deref().unwrap_or(""),
                left.statement_index.unwrap_or(usize::MAX),
            )
                .cmp(&(
                    &right.source_id,
                    issue_kind_rank(&right.issue_kind),
                    right.final_answer_id.as_deref().unwrap_or(""),
                    right.statement_index.unwrap_or(usize::MAX),
                ))
        });

        Ok(issues)
    }

    pub fn get_answer_artifact_export_manifest(&self) -> AegisResult<AnswerArtifactExportManifest> {
        let registry = SourceRegistry::load(&self.paths.registry_path())?;
        let issues = self.list_answer_artifact_issues()?;
        let mut sources = Vec::new();
        let mut source_count = 0;
        let mut draft_count = 0;
        let mut grounded_answer_count = 0;
        let mut final_answer_count = 0;
        let mut issue_count = 0;

        for record in registry.list_sources() {
            let version_dir = self.paths.source_version_dir(&record.source_id, &record.version_id);
            let per_source_draft_count = count_json_files(version_dir.join("answer_drafts"))?;
            let per_source_grounded_answer_count = count_json_files(version_dir.join("grounded_answers"))?;
            let final_answers = collect_manifest_final_answers(version_dir.join("final_answers"))?;
            let per_source_issue_count = issues.iter().filter(|issue| issue.source_id == record.source_id).count();
            if per_source_draft_count == 0
                && per_source_grounded_answer_count == 0
                && final_answers.is_empty()
                && per_source_issue_count == 0
            {
                continue;
            }

            source_count += 1;
            draft_count += per_source_draft_count;
            grounded_answer_count += per_source_grounded_answer_count;
            final_answer_count += final_answers.len();
            issue_count += per_source_issue_count;

            sources.push(AnswerArtifactExportSource {
                source_id: record.source_id,
                draft_count: per_source_draft_count,
                grounded_answer_count: per_source_grounded_answer_count,
                final_answer_count: final_answers.len(),
                final_answers,
                issue_count: per_source_issue_count,
            });
        }

        sources.sort_by(|left, right| left.source_id.cmp(&right.source_id));
        Ok(AnswerArtifactExportManifest {
            schema_version: ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION.to_string(),
            source_count,
            draft_count,
            grounded_answer_count,
            final_answer_count,
            issue_count,
            sources,
        })
    }

    pub fn export_answer_artifacts(&self, export_root: impl Into<PathBuf>) -> AegisResult<AnswerArtifactExportResult> {
        let export_root = export_root.into();
        self.validate_export_root(&export_root)?;
        if export_root.exists() {
            let mut entries = fs::read_dir(&export_root).map_err(|_| AegisError::ExportDestinationExists)?;
            if entries.next().is_some() {
                return Err(AegisError::ExportDestinationExists);
            }
        } else {
            fs::create_dir_all(&export_root).map_err(|_| AegisError::FinalAnswerWriteFailed)?;
        }

        let manifest = self.get_answer_artifact_export_manifest()?;
        let registry = SourceRegistry::load(&self.paths.registry_path())?;
        let issues = self.list_answer_artifact_issues()?;
        let export_issues = AnswerArtifactExportIssues {
            schema_version: ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION.to_string(),
            issues: issues.clone(),
        };
        let mut written_files = Vec::new();
        let mut exported_source_count = 0;
        let mut exported_draft_count = 0;
        let mut exported_grounded_answer_count = 0;
        let mut exported_final_answer_count = 0;
        let mut exported_issue_count = 0;

        for record in registry.list_sources() {
            let source_manifest = match manifest.sources.iter().find(|item| item.source_id == record.source_id) {
                Some(item) => item,
                None => continue,
            };
            let source_export_dir = export_root.join(&record.source_id);
            fs::create_dir_all(&source_export_dir).map_err(|_| AegisError::FinalAnswerWriteFailed)?;

            let source_root = self.paths.source_version_dir(&record.source_id, &record.version_id);
            let copied_drafts = copy_artifact_files(&source_root.join("answer_drafts"), &source_export_dir.join("answer_drafts"), &record.source_id, ExportedArtifactKind::AnswerDraft, None)?;
            let copied_grounded = copy_artifact_files(&source_root.join("grounded_answers"), &source_export_dir.join("grounded_answers"), &record.source_id, ExportedArtifactKind::GroundedAnswer, None)?;
            let copied_finals = copy_artifact_files(
                &source_root.join("final_answers"),
                &source_export_dir.join("final_answers"),
                &record.source_id,
                ExportedArtifactKind::FinalAnswer,
                Some(&source_manifest.final_answers),
            )?;

            if copied_drafts.is_empty()
                && copied_grounded.is_empty()
                && copied_finals.is_empty()
                && source_manifest.issue_count == 0
            {
                continue;
            }

            exported_source_count += 1;
            exported_draft_count += copied_drafts.len();
            exported_grounded_answer_count += copied_grounded.len();
            exported_final_answer_count += copied_finals.len();
            exported_issue_count += issues.iter().filter(|issue| issue.source_id == record.source_id).count();
            written_files.extend(copied_drafts);
            written_files.extend(copied_grounded);
            written_files.extend(copied_finals);
        }

        let manifest_path = export_root.join("export_manifest.json");
        fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?).map_err(|_| AegisError::FinalAnswerWriteFailed)?;
        written_files.push(ExportedArtifactFile {
            relative_path: "export_manifest.json".to_string(),
            artifact_kind: ExportedArtifactKind::Manifest,
            source_id: None,
            artifact_id: None,
        });

        let issues_path = export_root.join("export_issues.json");
        fs::write(&issues_path, serde_json::to_string_pretty(&export_issues)?).map_err(|_| AegisError::FinalAnswerWriteFailed)?;
        written_files.push(ExportedArtifactFile {
            relative_path: "export_issues.json".to_string(),
            artifact_kind: ExportedArtifactKind::Issues,
            source_id: None,
            artifact_id: None,
        });

        let summary = build_export_summary(&manifest, &export_issues)?;
        let summary_path = export_root.join("summary.json");
        fs::write(&summary_path, serde_json::to_string_pretty(&summary)?).map_err(|_| AegisError::FinalAnswerWriteFailed)?;
        written_files.push(ExportedArtifactFile {
            relative_path: "summary.json".to_string(),
            artifact_kind: ExportedArtifactKind::Summary,
            source_id: None,
            artifact_id: None,
        });

        let integrity = build_export_integrity(&export_root, &written_files)?;
        let integrity_path = export_root.join("export_integrity.json");
        fs::write(&integrity_path, serde_json::to_string_pretty(&integrity)?).map_err(|_| AegisError::FinalAnswerWriteFailed)?;
        written_files.push(ExportedArtifactFile {
            relative_path: "export_integrity.json".to_string(),
            artifact_kind: ExportedArtifactKind::Integrity,
            source_id: None,
            artifact_id: None,
        });

        written_files.sort_by(|left, right| {
            (
                &left.relative_path,
                left.source_id.as_deref().unwrap_or(""),
                left.artifact_id.as_deref().unwrap_or(""),
            )
                .cmp(&(
                    &right.relative_path,
                    right.source_id.as_deref().unwrap_or(""),
                    right.artifact_id.as_deref().unwrap_or(""),
                ))
        });

        Ok(AnswerArtifactExportResult {
            schema_version: ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION.to_string(),
            manifest,
            integrity,
            exported_source_count,
            exported_draft_count,
            exported_grounded_answer_count,
            exported_final_answer_count,
            exported_issue_count,
            export_id: export_root.file_name().and_then(|value| value.to_str()).unwrap_or("export").to_string(),
            written_files,
        })
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

fn count_json_files(dir: PathBuf) -> AegisResult<usize> {
    if !dir.exists() {
        return Ok(0);
    }
    let mut count = 0;
    for entry in fs::read_dir(&dir).map_err(|_| AegisError::FinalAnswerReadFailed)? {
        let entry = entry.map_err(|_| AegisError::FinalAnswerReadFailed)?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) == Some("json") {
            count += 1;
        }
    }
    Ok(count)
}

#[derive(Debug, Default, Clone)]
struct AnswerArtifactCounts {
    draft_count: usize,
    grounded_answer_count: usize,
    final_answer_count: usize,
    malformed_final_answer_count: usize,
    unsupported_statement_count: usize,
    needs_evidence_statement_count: usize,
}

impl AnswerArtifactCounts {
    fn is_empty(&self) -> bool {
        self.draft_count == 0
            && self.grounded_answer_count == 0
            && self.final_answer_count == 0
            && self.malformed_final_answer_count == 0
            && self.unsupported_statement_count == 0
            && self.needs_evidence_statement_count == 0
    }
}

fn answer_artifact_counts_for_source(service: &FinalAnswerService, source_id: &str, version_id: &str) -> AegisResult<AnswerArtifactCounts> {
    let version_dir = service.paths.source_version_dir(source_id, version_id);
    let draft_count = count_json_files(version_dir.join("answer_drafts"))?;
    let grounded_answer_count = count_json_files(version_dir.join("grounded_answers"))?;

    let final_answer_dir = version_dir.join("final_answers");
    if !final_answer_dir.exists() {
        return Ok(AnswerArtifactCounts {
            draft_count,
            grounded_answer_count,
            ..Default::default()
        });
    }

    let mut final_answer_count = 0;
    let mut malformed_final_answer_count = 0;
    let mut unsupported_statement_count = 0;
    let mut needs_evidence_statement_count = 0;
    for entry in fs::read_dir(&final_answer_dir).map_err(|_| AegisError::FinalAnswerReadFailed)? {
        let entry = entry.map_err(|_| AegisError::FinalAnswerReadFailed)?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        match fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str::<FinalAnswer>(&content) {
                Ok(answer) => {
                    final_answer_count += 1;
                    unsupported_statement_count += answer
                        .statements
                        .iter()
                        .filter(|statement| statement.status == FinalAnswerStatementStatus::Unsupported)
                        .count();
                    needs_evidence_statement_count += answer
                        .statements
                        .iter()
                        .filter(|statement| statement.status == FinalAnswerStatementStatus::NeedsEvidence)
                        .count();
                }
                Err(_) => malformed_final_answer_count += 1,
            },
            Err(_) => malformed_final_answer_count += 1,
        }
    }

    Ok(AnswerArtifactCounts {
        draft_count,
        grounded_answer_count,
        final_answer_count,
        malformed_final_answer_count,
        unsupported_statement_count,
        needs_evidence_statement_count,
    })
}

fn issue_kind_rank(issue_kind: &AnswerArtifactIssueKind) -> usize {
    match issue_kind {
        AnswerArtifactIssueKind::MalformedFinalAnswer => 0,
        AnswerArtifactIssueKind::NeedsEvidenceStatement => 1,
        AnswerArtifactIssueKind::UnsupportedStatement => 2,
    }
}

fn build_export_summary(
    manifest: &AnswerArtifactExportManifest,
    export_issues: &AnswerArtifactExportIssues,
)
-> AegisResult<AnswerArtifactExportSummary> {
    let issues = &export_issues.issues;
    let issue_kinds = [
        AnswerArtifactIssueKind::MalformedFinalAnswer,
        AnswerArtifactIssueKind::NeedsEvidenceStatement,
        AnswerArtifactIssueKind::UnsupportedStatement,
    ]
    .iter()
    .map(|issue_kind| AnswerArtifactExportIssueKindCount {
        issue_kind: issue_kind.clone(),
        count: issues.iter().filter(|issue| issue.issue_kind == *issue_kind).count(),
    })
    .filter(|item| item.count > 0)
    .collect::<Vec<_>>();

    let sources = manifest
        .sources
        .iter()
        .map(|source| AnswerArtifactExportSummarySource {
            source_id: source.source_id.clone(),
            draft_count: source.draft_count,
            grounded_answer_count: source.grounded_answer_count,
            final_answer_count: source.final_answer_count,
            issue_count: source.issue_count,
        })
        .collect::<Vec<_>>();

    let mut hasher = Sha256::new();
    hasher.update(serde_json::to_string(manifest)?);
    hasher.update(serde_json::to_string(export_issues)?);
    let export_id = format!("sha256:{:x}", hasher.finalize());

    Ok(AnswerArtifactExportSummary {
        schema_version: ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION.to_string(),
        export_id,
        generated_from: "persisted_artifacts".to_string(),
        export_scope: "manual_artifact_export".to_string(),
        non_goals: vec![
            "no_generation".to_string(),
            "no_repair".to_string(),
            "no_editing".to_string(),
            "no_import".to_string(),
            "no_share".to_string(),
        ],
        source_count: manifest.source_count,
        draft_count: manifest.draft_count,
        grounded_answer_count: manifest.grounded_answer_count,
        final_answer_count: manifest.final_answer_count,
        issue_count: issues.len(),
        issue_kinds,
        sources,
    })
}

fn build_export_integrity(
    export_root: &Path,
    written_files: &[ExportedArtifactFile],
) -> AegisResult<AnswerArtifactExportIntegrity> {
    let mut files = written_files
        .iter()
        .filter(|item| item.relative_path != "export_integrity.json")
        .map(|item| {
            let path = export_root.join(&item.relative_path);
            let bytes = fs::read(&path).map_err(|_| AegisError::FinalAnswerReadFailed)?;
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            Ok(AnswerArtifactExportIntegrityFile {
                relative_path: item.relative_path.clone(),
                byte_count: bytes.len() as u64,
                sha256: format!("sha256:{:x}", hasher.finalize()),
            })
        })
        .collect::<AegisResult<Vec<_>>>()?;

    files.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));

    Ok(AnswerArtifactExportIntegrity {
        schema_version: ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION.to_string(),
        algorithm: ANSWER_ARTIFACT_EXPORT_INTEGRITY_ALGORITHM.to_string(),
        files,
    })
}

fn inspect_answer_artifact_export_bundle_impl(export_root: PathBuf) -> AegisResult<AnswerArtifactExportBundleInspection> {
    if export_root.as_os_str().is_empty() || export_root.to_string_lossy().trim().is_empty() {
        return Err(AegisError::ExportBundleInputMissing);
    }

    let mut errors: Vec<AnswerArtifactExportBundleInspectionIssue> = Vec::new();
    let mut warnings: Vec<AnswerArtifactExportBundleInspectionIssue> = Vec::new();

    let manifest: Option<AnswerArtifactExportManifest> = read_bundle_json_file(
        &export_root,
        "export_manifest.json",
        AnswerArtifactExportBundleInspectionIssueKind::MissingManifest,
        AnswerArtifactExportBundleInspectionIssueKind::ManifestReadFailed,
        &mut errors,
    );
    let issues: Option<AnswerArtifactExportIssues> = read_bundle_issues_file(
        &export_root,
        &mut errors,
    );
    let summary: Option<AnswerArtifactExportSummary> = read_bundle_json_file(
        &export_root,
        "summary.json",
        AnswerArtifactExportBundleInspectionIssueKind::MissingSummary,
        AnswerArtifactExportBundleInspectionIssueKind::SummaryReadFailed,
        &mut errors,
    );
    let integrity: Option<AnswerArtifactExportIntegrity> = read_bundle_integrity_file(&export_root, &mut errors);

    let issue_kind_counts = issues
        .as_ref()
        .map(|items| issue_kind_counts_from_export_issues(&items.issues));
    let schema_version = common_export_bundle_schema_version(
        manifest.as_ref(),
        issues.as_ref(),
        summary.as_ref(),
        integrity.as_ref(),
    );

    validate_export_bundle_schema_versions(
        manifest.as_ref(),
        issues.as_ref(),
        summary.as_ref(),
        integrity.as_ref(),
        &mut errors,
    );
    validate_export_bundle_integrity(
        &export_root,
        manifest.as_ref(),
        issues.as_ref(),
        summary.as_ref(),
        integrity.as_ref(),
        &mut errors,
    );

    if let (Some(manifest), Some(issues), Some(summary)) = (manifest.as_ref(), issues.as_ref(), summary.as_ref()) {
        let expected_summary = build_export_summary(manifest, issues)?;
        if summary.source_count != expected_summary.source_count
            || summary.draft_count != expected_summary.draft_count
            || summary.grounded_answer_count != expected_summary.grounded_answer_count
            || summary.final_answer_count != expected_summary.final_answer_count
            || summary.sources != expected_summary.sources
        {
            errors.push(inspection_issue(
                AnswerArtifactExportBundleInspectionIssueKind::SummaryCountsMismatch,
                "summary.json source and artifact counts do not match the manifest".to_string(),
                Some("summary.json".to_string()),
            ));
        }
        if summary.issue_count != expected_summary.issue_count {
            errors.push(inspection_issue(
                AnswerArtifactExportBundleInspectionIssueKind::SummaryIssueCountMismatch,
                "summary.json issue_count does not match export_issues.json".to_string(),
                Some("summary.json".to_string()),
            ));
        }
        if summary.issue_kinds != expected_summary.issue_kinds {
            errors.push(inspection_issue(
                AnswerArtifactExportBundleInspectionIssueKind::SummaryIssueKindCountsMismatch,
                "summary.json issue-kind counts do not match export_issues.json".to_string(),
                Some("summary.json".to_string()),
            ));
        }
        if summary.export_id != expected_summary.export_id {
            errors.push(inspection_issue(
                AnswerArtifactExportBundleInspectionIssueKind::SummaryExportIdMismatch,
                "summary.json export_id does not match the derived hash".to_string(),
                Some("summary.json".to_string()),
            ));
        }
        if summary.generated_from != expected_summary.generated_from
            || summary.export_scope != expected_summary.export_scope
            || summary.non_goals != expected_summary.non_goals
        {
            errors.push(inspection_issue(
                AnswerArtifactExportBundleInspectionIssueKind::SummaryMetadataMismatch,
                "summary.json metadata does not match the expected export summary contract".to_string(),
                Some("summary.json".to_string()),
            ));
        }
    }

    errors.sort_by(|left, right| {
        (
            export_bundle_issue_kind_rank(&left.kind),
            left.relative_path.as_deref().unwrap_or(""),
            left.message.as_str(),
        )
            .cmp(&(
                export_bundle_issue_kind_rank(&right.kind),
                right.relative_path.as_deref().unwrap_or(""),
                right.message.as_str(),
            ))
    });
    warnings.sort_by(|left, right| {
        (
            export_bundle_issue_kind_rank(&left.kind),
            left.relative_path.as_deref().unwrap_or(""),
            left.message.as_str(),
        )
            .cmp(&(
                export_bundle_issue_kind_rank(&right.kind),
                right.relative_path.as_deref().unwrap_or(""),
                right.message.as_str(),
            ))
    });

    let inspection_summary = build_export_bundle_inspection_summary(
        schema_version.as_deref().is_some(),
        manifest.as_ref(),
        issues.as_ref(),
        summary.as_ref(),
        integrity.as_ref(),
        &errors,
        &warnings,
    );
    let report_preview = build_export_bundle_inspection_report_preview(&inspection_summary, &errors, &warnings);

    Ok(AnswerArtifactExportBundleInspection {
        schema_version,
        manifest_schema_version: non_empty_schema_version(manifest.as_ref().map(|value| value.schema_version.as_str())),
        issues_schema_version: non_empty_schema_version(issues.as_ref().map(|value| value.schema_version.as_str())),
        summary_schema_version: non_empty_schema_version(summary.as_ref().map(|value| value.schema_version.as_str())),
        integrity_schema_version: non_empty_schema_version(integrity.as_ref().map(|value| value.schema_version.as_str())),
        integrity_algorithm: integrity.as_ref().map(|value| value.algorithm.clone()).filter(|value| !value.trim().is_empty()),
        inspection_summary,
        report_preview,
        has_manifest: manifest.is_some(),
        has_issues: issues.is_some(),
        has_summary: summary.is_some(),
        has_integrity: integrity.is_some(),
        is_consistent: errors.is_empty(),
        issue_count: errors.len(),
        warning_count: warnings.len(),
        errors,
        warnings,
        manifest_counts: manifest,
        summary_counts: summary,
        issue_kind_counts,
        integrity_counts: integrity,
    })
}

fn read_bundle_json_file<T: DeserializeOwned>(
    export_root: &Path,
    relative_path: &str,
    missing_kind: AnswerArtifactExportBundleInspectionIssueKind,
    read_failed_kind: AnswerArtifactExportBundleInspectionIssueKind,
    issues: &mut Vec<AnswerArtifactExportBundleInspectionIssue>,
) -> Option<T> {
    let path = export_root.join(relative_path);
    if !path.exists() {
        issues.push(inspection_issue(
            missing_kind,
            format!("{relative_path} is missing"),
            Some(relative_path.to_string()),
        ));
        return None;
    }

    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(_) => {
            issues.push(inspection_issue(
                read_failed_kind,
                format!("{relative_path} could not be read"),
                Some(relative_path.to_string()),
            ));
            return None;
        }
    };

    match serde_json::from_str::<T>(&content) {
        Ok(value) => Some(value),
        Err(_) => {
            issues.push(inspection_issue(
                read_failed_kind,
                format!("{relative_path} is malformed"),
                Some(relative_path.to_string()),
            ));
            None
        }
    }
}

fn read_bundle_issues_file(
    export_root: &Path,
    issues: &mut Vec<AnswerArtifactExportBundleInspectionIssue>,
) -> Option<AnswerArtifactExportIssues> {
    let relative_path = "export_issues.json";
    let path = export_root.join(relative_path);
    if !path.exists() {
        issues.push(inspection_issue(
            AnswerArtifactExportBundleInspectionIssueKind::MissingIssues,
            format!("{relative_path} is missing"),
            Some(relative_path.to_string()),
        ));
        return None;
    }

    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(_) => {
            issues.push(inspection_issue(
                AnswerArtifactExportBundleInspectionIssueKind::IssuesReadFailed,
                format!("{relative_path} could not be read"),
                Some(relative_path.to_string()),
            ));
            return None;
        }
    };

    let value = match serde_json::from_str::<serde_json::Value>(&content) {
        Ok(value) => value,
        Err(_) => {
            issues.push(inspection_issue(
                AnswerArtifactExportBundleInspectionIssueKind::IssuesReadFailed,
                format!("{relative_path} is malformed"),
                Some(relative_path.to_string()),
            ));
            return None;
        }
    };

    if value.is_array() {
        return match serde_json::from_value::<Vec<AnswerArtifactIssue>>(value) {
            Ok(items) => Some(AnswerArtifactExportIssues {
                schema_version: String::new(),
                issues: items,
            }),
            Err(_) => {
                issues.push(inspection_issue(
                    AnswerArtifactExportBundleInspectionIssueKind::IssuesReadFailed,
                    format!("{relative_path} is malformed"),
                    Some(relative_path.to_string()),
                ));
                None
            }
        };
    }

    match serde_json::from_value::<AnswerArtifactExportIssues>(value) {
        Ok(value) => Some(value),
        Err(_) => {
            issues.push(inspection_issue(
                AnswerArtifactExportBundleInspectionIssueKind::IssuesReadFailed,
                format!("{relative_path} is malformed"),
                Some(relative_path.to_string()),
            ));
            None
        }
    }
}

fn read_bundle_integrity_file(
    export_root: &Path,
    issues: &mut Vec<AnswerArtifactExportBundleInspectionIssue>,
) -> Option<AnswerArtifactExportIntegrity> {
    let relative_path = "export_integrity.json";
    let path = export_root.join(relative_path);
    if !path.exists() {
        issues.push(inspection_issue(
            AnswerArtifactExportBundleInspectionIssueKind::MissingIntegrity,
            format!("{relative_path} is missing"),
            Some(relative_path.to_string()),
        ));
        return None;
    }

    let content = match fs::read_to_string(&path) {
        Ok(content) => content,
        Err(_) => {
            issues.push(inspection_issue(
                AnswerArtifactExportBundleInspectionIssueKind::IntegrityReadFailed,
                format!("{relative_path} could not be read"),
                Some(relative_path.to_string()),
            ));
            return None;
        }
    };

    match serde_json::from_str::<AnswerArtifactExportIntegrity>(&content) {
        Ok(value) => Some(value),
        Err(_) => {
            issues.push(inspection_issue(
                AnswerArtifactExportBundleInspectionIssueKind::IntegrityReadFailed,
                format!("{relative_path} is malformed"),
                Some(relative_path.to_string()),
            ));
            None
        }
    }
}

fn validate_export_bundle_schema_versions(
    manifest: Option<&AnswerArtifactExportManifest>,
    issues: Option<&AnswerArtifactExportIssues>,
    summary: Option<&AnswerArtifactExportSummary>,
    integrity: Option<&AnswerArtifactExportIntegrity>,
    errors: &mut Vec<AnswerArtifactExportBundleInspectionIssue>,
) {
    let versions = [
        ("export_manifest.json", manifest.map(|value| value.schema_version.as_str())),
        ("export_issues.json", issues.map(|value| value.schema_version.as_str())),
        ("summary.json", summary.map(|value| value.schema_version.as_str())),
        ("export_integrity.json", integrity.map(|value| value.schema_version.as_str())),
    ];

    for (relative_path, version) in versions {
        let Some(version) = version else {
            continue;
        };
        let trimmed = version.trim();
        if trimmed.is_empty() {
            errors.push(inspection_issue(
                AnswerArtifactExportBundleInspectionIssueKind::SchemaVersionMissing,
                format!("{relative_path} is missing schema_version"),
                Some(relative_path.to_string()),
            ));
        } else if trimmed != ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION {
            errors.push(inspection_issue(
                AnswerArtifactExportBundleInspectionIssueKind::SchemaVersionUnsupported,
                format!("{relative_path} has unsupported schema_version"),
                Some(relative_path.to_string()),
            ));
        }
    }

    let mut present_versions = [
        manifest.map(|value| value.schema_version.trim()),
        issues.map(|value| value.schema_version.trim()),
        summary.map(|value| value.schema_version.trim()),
        integrity.map(|value| value.schema_version.trim()),
    ]
    .into_iter()
    .flatten()
    .filter(|version| !version.is_empty())
    .collect::<Vec<_>>();
    present_versions.sort_unstable();
    present_versions.dedup();

    if present_versions.len() > 1 {
        errors.push(inspection_issue(
            AnswerArtifactExportBundleInspectionIssueKind::SchemaVersionMismatch,
            "bundle files do not use the same schema_version".to_string(),
            None,
        ));
    }
}

fn validate_export_bundle_integrity(
    export_root: &Path,
    _manifest: Option<&AnswerArtifactExportManifest>,
    _issues: Option<&AnswerArtifactExportIssues>,
    _summary: Option<&AnswerArtifactExportSummary>,
    integrity: Option<&AnswerArtifactExportIntegrity>,
    errors: &mut Vec<AnswerArtifactExportBundleInspectionIssue>,
) {
    let Some(integrity) = integrity else {
        return;
    };

    let relative_path = "export_integrity.json".to_string();
    let integrity_schema_version = integrity.schema_version.trim();
    if integrity_schema_version.is_empty() {
        errors.push(inspection_issue(
            AnswerArtifactExportBundleInspectionIssueKind::IntegritySchemaVersionMissing,
            "export_integrity.json is missing schema_version".to_string(),
            Some(relative_path.clone()),
        ));
    } else if integrity_schema_version != ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION {
        errors.push(inspection_issue(
            AnswerArtifactExportBundleInspectionIssueKind::IntegritySchemaVersionUnsupported,
            "export_integrity.json has unsupported schema_version".to_string(),
            Some(relative_path.clone()),
        ));
    }

    let algorithm = integrity.algorithm.trim();
    if algorithm.is_empty() {
        errors.push(inspection_issue(
            AnswerArtifactExportBundleInspectionIssueKind::IntegrityAlgorithmMissing,
            "export_integrity.json is missing algorithm".to_string(),
            Some(relative_path.clone()),
        ));
    } else if algorithm != ANSWER_ARTIFACT_EXPORT_INTEGRITY_ALGORITHM {
        errors.push(inspection_issue(
            AnswerArtifactExportBundleInspectionIssueKind::IntegrityAlgorithmUnsupported,
            "export_integrity.json has unsupported algorithm".to_string(),
            Some(relative_path.clone()),
        ));
    }

    let mut seen_paths = std::collections::BTreeSet::new();
    for file in &integrity.files {
        let normalized = file.relative_path.trim();
        let path = Path::new(normalized);
        if normalized.is_empty()
            || path.is_absolute()
            || path.components().any(|component| !matches!(component, std::path::Component::Normal(_)))
        {
            errors.push(inspection_issue(
                AnswerArtifactExportBundleInspectionIssueKind::IntegrityPathInvalid,
                "export_integrity.json lists an invalid relative_path".to_string(),
                Some("export_integrity.json".to_string()),
            ));
            continue;
        }
        if !seen_paths.insert(normalized.to_string()) {
            errors.push(inspection_issue(
                AnswerArtifactExportBundleInspectionIssueKind::IntegrityDuplicatePath,
                "export_integrity.json lists a duplicate relative_path".to_string(),
                Some("export_integrity.json".to_string()),
            ));
            continue;
        }
        let file_path = export_root.join(normalized);
        if !file_path.exists() {
            errors.push(inspection_issue(
                AnswerArtifactExportBundleInspectionIssueKind::IntegrityMissingFile,
                format!("{normalized} is missing from the export bundle"),
                Some("export_integrity.json".to_string()),
            ));
            continue;
        }
        let bytes = match fs::read(&file_path) {
            Ok(bytes) => bytes,
            Err(_) => {
                errors.push(inspection_issue(
                    AnswerArtifactExportBundleInspectionIssueKind::IntegrityReadFailed,
                    format!("{normalized} could not be read"),
                    Some("export_integrity.json".to_string()),
                ));
                continue;
            }
        };
        if bytes.len() as u64 != file.byte_count {
            errors.push(inspection_issue(
                AnswerArtifactExportBundleInspectionIssueKind::IntegrityByteCountMismatch,
                format!("{normalized} byte_count does not match the bundle file"),
                Some("export_integrity.json".to_string()),
            ));
        }
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let digest = format!("sha256:{:x}", hasher.finalize());
        if digest != file.sha256 {
            errors.push(inspection_issue(
                AnswerArtifactExportBundleInspectionIssueKind::IntegrityDigestMismatch,
                format!("{normalized} digest does not match the bundle file"),
                Some("export_integrity.json".to_string()),
            ));
        }
    }
}

fn build_export_bundle_inspection_summary(
    schema_supported: bool,
    manifest: Option<&AnswerArtifactExportManifest>,
    issues: Option<&AnswerArtifactExportIssues>,
    summary: Option<&AnswerArtifactExportSummary>,
    integrity: Option<&AnswerArtifactExportIntegrity>,
    errors: &[AnswerArtifactExportBundleInspectionIssue],
    warnings: &[AnswerArtifactExportBundleInspectionIssue],
) -> AnswerArtifactExportBundleInspectionSummary {
    let combined_issues = errors
        .iter()
        .chain(warnings.iter())
        .cloned()
        .collect::<Vec<_>>();
    let issue_counts = inspection_issue_kind_counts_from_bundle_issues(&combined_issues);

    let integrity_issue = |kind: &AnswerArtifactExportBundleInspectionIssueKind| {
        matches!(
            kind,
            AnswerArtifactExportBundleInspectionIssueKind::MissingIntegrity
                | AnswerArtifactExportBundleInspectionIssueKind::IntegrityReadFailed
                | AnswerArtifactExportBundleInspectionIssueKind::IntegritySchemaVersionMissing
                | AnswerArtifactExportBundleInspectionIssueKind::IntegritySchemaVersionUnsupported
                | AnswerArtifactExportBundleInspectionIssueKind::IntegrityAlgorithmMissing
                | AnswerArtifactExportBundleInspectionIssueKind::IntegrityAlgorithmUnsupported
                | AnswerArtifactExportBundleInspectionIssueKind::IntegrityDuplicatePath
                | AnswerArtifactExportBundleInspectionIssueKind::IntegrityPathInvalid
                | AnswerArtifactExportBundleInspectionIssueKind::IntegrityMissingFile
                | AnswerArtifactExportBundleInspectionIssueKind::IntegrityByteCountMismatch
                | AnswerArtifactExportBundleInspectionIssueKind::IntegrityDigestMismatch
        )
    };

    let checked_file_count = usize::from(manifest.is_some())
        + usize::from(issues.is_some())
        + usize::from(summary.is_some())
        + usize::from(integrity.is_some());

    AnswerArtifactExportBundleInspectionSummary {
        is_consistent: errors.is_empty(),
        schema_supported,
        integrity_verified: errors.is_empty()
            && integrity.is_some()
            && !errors.iter().any(|issue| integrity_issue(&issue.kind))
            && !warnings.iter().any(|issue| integrity_issue(&issue.kind)),
        issue_count: errors.len() + warnings.len(),
        warning_count: warnings.len(),
        issue_counts_by_kind: issue_counts,
        checked_file_count,
        integrity_file_count: integrity.map(|value| value.files.len()).unwrap_or(0),
    }
}

fn build_export_bundle_inspection_report_preview(
    inspection_summary: &AnswerArtifactExportBundleInspectionSummary,
    errors: &[AnswerArtifactExportBundleInspectionIssue],
    warnings: &[AnswerArtifactExportBundleInspectionIssue],
) -> AnswerArtifactExportBundleInspectionReportPreview {
    let mut sections = vec![
        AnswerArtifactExportBundleInspectionReportSection {
            heading: "Status".to_string(),
            lines: vec![
                format!("Consistent: {}", yes_no(inspection_summary.is_consistent)),
                format!("Schema supported: {}", yes_no(inspection_summary.schema_supported)),
                format!("Integrity verified: {}", yes_no(inspection_summary.integrity_verified)),
                format!("Issue count: {}", inspection_summary.issue_count),
                format!("Warning count: {}", inspection_summary.warning_count),
                format!("Checked files: {}", inspection_summary.checked_file_count),
                format!("Integrity files: {}", inspection_summary.integrity_file_count),
            ],
        },
        AnswerArtifactExportBundleInspectionReportSection {
            heading: "Issue counts by kind".to_string(),
            lines: if inspection_summary.issue_counts_by_kind.is_empty() {
                vec!["No issue kinds reported.".to_string()]
            } else {
                inspection_summary
                    .issue_counts_by_kind
                    .iter()
                    .map(|item| format!("{} = {}", inspection_issue_kind_name(&item.kind), item.count))
                    .collect::<Vec<_>>()
            },
        },
    ];

    let diagnostic_lines = errors
        .iter()
        .map(|issue| format!("error | {} | {}", inspection_issue_kind_name(&issue.kind), issue.message))
        .chain(warnings.iter().map(|issue| format!("warning | {} | {}", inspection_issue_kind_name(&issue.kind), issue.message)))
        .collect::<Vec<_>>();
    if !diagnostic_lines.is_empty() {
        sections.push(AnswerArtifactExportBundleInspectionReportSection {
            heading: "Issues".to_string(),
            lines: diagnostic_lines,
        });
    }

    AnswerArtifactExportBundleInspectionReportPreview {
        title: "Export bundle inspection report preview".to_string(),
        schema_version: ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION.to_string(),
        is_consistent: inspection_summary.is_consistent,
        integrity_verified: inspection_summary.integrity_verified,
        issue_count: inspection_summary.issue_count,
        warning_count: inspection_summary.warning_count,
        issue_counts_by_kind: inspection_summary.issue_counts_by_kind.clone(),
        sections,
    }
}

fn inspection_issue_kind_counts_from_bundle_issues(
    issues: &[AnswerArtifactExportBundleInspectionIssue],
) -> Vec<AnswerArtifactExportBundleInspectionIssueKindCount> {
    let mut counts: Vec<AnswerArtifactExportBundleInspectionIssueKindCount> = Vec::new();
    for issue in issues {
        if let Some(existing) = counts.iter_mut().find(|item| item.kind == issue.kind) {
            existing.count += 1;
        } else {
            counts.push(AnswerArtifactExportBundleInspectionIssueKindCount { kind: issue.kind.clone(), count: 1 });
        }
    }

    counts.sort_by(|left, right| {
        (
            export_bundle_issue_kind_rank(&left.kind),
            left.count,
        )
            .cmp(&(
                export_bundle_issue_kind_rank(&right.kind),
                right.count,
            ))
    });

    counts
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn inspection_issue_kind_name(kind: &AnswerArtifactExportBundleInspectionIssueKind) -> String {
    serde_json::to_string(kind)
        .map(|value| value.trim_matches('"').to_string())
        .unwrap_or_else(|_| format!("{kind:?}"))
}

fn non_empty_schema_version(version: Option<&str>) -> Option<String> {
    version
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn common_export_bundle_schema_version(
    manifest: Option<&AnswerArtifactExportManifest>,
    issues: Option<&AnswerArtifactExportIssues>,
    summary: Option<&AnswerArtifactExportSummary>,
    integrity: Option<&AnswerArtifactExportIntegrity>,
) -> Option<String> {
    let manifest_version = manifest.map(|value| value.schema_version.trim());
    let issues_version = issues.map(|value| value.schema_version.trim());
    let summary_version = summary.map(|value| value.schema_version.trim());
    let integrity_version = integrity.map(|value| value.schema_version.trim());
    let versions = [manifest_version, issues_version, summary_version, integrity_version]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    if versions.len() == 4
        && versions.iter().all(|version| !version.is_empty())
        && versions.iter().all(|version| *version == ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION)
    {
        Some(ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION.to_string())
    } else {
        None
    }
}

fn issue_kind_counts_from_export_issues(issues: &[AnswerArtifactIssue]) -> Vec<AnswerArtifactExportIssueKindCount> {
    let mut counts = [
        AnswerArtifactIssueKind::MalformedFinalAnswer,
        AnswerArtifactIssueKind::NeedsEvidenceStatement,
        AnswerArtifactIssueKind::UnsupportedStatement,
    ]
    .iter()
    .map(|issue_kind| AnswerArtifactExportIssueKindCount {
        issue_kind: issue_kind.clone(),
        count: issues.iter().filter(|issue| issue.issue_kind == *issue_kind).count(),
    })
    .filter(|item| item.count > 0)
    .collect::<Vec<_>>();

    counts.sort_by(|left, right| issue_kind_rank(&left.issue_kind).cmp(&issue_kind_rank(&right.issue_kind)));
    counts
}

fn inspection_issue(
    kind: AnswerArtifactExportBundleInspectionIssueKind,
    message: String,
    relative_path: Option<String>,
) -> AnswerArtifactExportBundleInspectionIssue {
    AnswerArtifactExportBundleInspectionIssue { kind, message, relative_path }
}

fn export_bundle_issue_kind_rank(kind: &AnswerArtifactExportBundleInspectionIssueKind) -> usize {
    match kind {
        AnswerArtifactExportBundleInspectionIssueKind::MissingManifest => 0,
        AnswerArtifactExportBundleInspectionIssueKind::ManifestReadFailed => 1,
        AnswerArtifactExportBundleInspectionIssueKind::MissingIssues => 2,
        AnswerArtifactExportBundleInspectionIssueKind::IssuesReadFailed => 3,
        AnswerArtifactExportBundleInspectionIssueKind::MissingSummary => 4,
        AnswerArtifactExportBundleInspectionIssueKind::SummaryReadFailed => 5,
        AnswerArtifactExportBundleInspectionIssueKind::MissingIntegrity => 6,
        AnswerArtifactExportBundleInspectionIssueKind::IntegrityReadFailed => 7,
        AnswerArtifactExportBundleInspectionIssueKind::IntegritySchemaVersionMissing => 8,
        AnswerArtifactExportBundleInspectionIssueKind::IntegritySchemaVersionUnsupported => 9,
        AnswerArtifactExportBundleInspectionIssueKind::IntegrityAlgorithmMissing => 10,
        AnswerArtifactExportBundleInspectionIssueKind::IntegrityAlgorithmUnsupported => 11,
        AnswerArtifactExportBundleInspectionIssueKind::IntegrityDuplicatePath => 12,
        AnswerArtifactExportBundleInspectionIssueKind::IntegrityPathInvalid => 13,
        AnswerArtifactExportBundleInspectionIssueKind::IntegrityMissingFile => 14,
        AnswerArtifactExportBundleInspectionIssueKind::IntegrityByteCountMismatch => 15,
        AnswerArtifactExportBundleInspectionIssueKind::IntegrityDigestMismatch => 16,
        AnswerArtifactExportBundleInspectionIssueKind::SchemaVersionMissing => 17,
        AnswerArtifactExportBundleInspectionIssueKind::SchemaVersionUnsupported => 18,
        AnswerArtifactExportBundleInspectionIssueKind::SchemaVersionMismatch => 19,
        AnswerArtifactExportBundleInspectionIssueKind::SummaryCountsMismatch => 20,
        AnswerArtifactExportBundleInspectionIssueKind::SummaryIssueCountMismatch => 21,
        AnswerArtifactExportBundleInspectionIssueKind::SummaryIssueKindCountsMismatch => 22,
        AnswerArtifactExportBundleInspectionIssueKind::SummaryExportIdMismatch => 23,
        AnswerArtifactExportBundleInspectionIssueKind::SummaryMetadataMismatch => 24,
    }
}

fn copy_artifact_files(
    source_dir: &PathBuf,
    export_dir: &PathBuf,
    source_id: &str,
    artifact_kind: ExportedArtifactKind,
    allowed_final_answers: Option<&[FinalAnswerMetadata]>,
) -> AegisResult<Vec<ExportedArtifactFile>> {
    if !source_dir.exists() {
        return Ok(Vec::new());
    }
    fs::create_dir_all(export_dir).map_err(|_| AegisError::FinalAnswerWriteFailed)?;
    let mut written = Vec::new();
    for entry in fs::read_dir(source_dir).map_err(|_| AegisError::FinalAnswerReadFailed)? {
        let entry = entry.map_err(|_| AegisError::FinalAnswerReadFailed)?;
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let artifact_id = match path.file_stem().and_then(|value| value.to_str()) {
            Some(value) => value.to_string(),
            None => continue,
        };

        if matches!(artifact_kind, ExportedArtifactKind::FinalAnswer) {
            let Some(allowed) = allowed_final_answers else {
                continue;
            };
            if !allowed.iter().any(|item| item.final_answer_id == artifact_id) {
                continue;
            }
        }

        let content = fs::read_to_string(&path).map_err(|_| AegisError::FinalAnswerReadFailed)?;
        let export_path = export_dir.join(format!("{artifact_id}.json"));
        fs::write(&export_path, content).map_err(|_| AegisError::FinalAnswerWriteFailed)?;
        written.push(ExportedArtifactFile {
            relative_path: PathBuf::from(source_id)
                .join(match artifact_kind {
                    ExportedArtifactKind::AnswerDraft => "answer_drafts",
                    ExportedArtifactKind::GroundedAnswer => "grounded_answers",
                    ExportedArtifactKind::FinalAnswer => "final_answers",
                    ExportedArtifactKind::Manifest | ExportedArtifactKind::Issues | ExportedArtifactKind::Summary | ExportedArtifactKind::Integrity => unreachable!(),
                })
                .join(format!("{artifact_id}.json"))
                .to_string_lossy()
                .to_string(),
            artifact_kind: artifact_kind.clone(),
            source_id: Some(source_id.to_string()),
            artifact_id: Some(artifact_id),
        });
    }
    written.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    Ok(written)
}

impl FinalAnswerService {
    fn validate_export_root(&self, export_root: &PathBuf) -> AegisResult<()> {
        if export_root.as_os_str().is_empty() {
            return Err(AegisError::ExportDestinationMissing);
        }
        if export_root
            .components()
            .any(|component| component.as_os_str().to_string_lossy() == ".aegis")
        {
            return Err(AegisError::ExportDestinationInsideCorpus);
        }
        Ok(())
    }
}

fn collect_manifest_final_answers(final_answer_dir: PathBuf) -> AegisResult<Vec<FinalAnswerMetadata>> {
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
        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(_) => continue,
        };
        match serde_json::from_str::<FinalAnswer>(&content) {
            Ok(answer) => items.push(FinalAnswerMetadata {
                final_answer_id: answer.final_answer_id,
                grounded_answer_id: answer.grounded_answer_id,
                statement_count: answer.statement_count,
                unsupported_count: answer.unsupported_count,
                needs_evidence_count: answer
                    .statements
                    .iter()
                    .filter(|statement| statement.status == FinalAnswerStatementStatus::NeedsEvidence)
                    .count(),
            }),
            Err(_) => continue,
        }
    }
    items.sort_by(|left, right| left.final_answer_id.cmp(&right.final_answer_id));
    Ok(items)
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

    fn snapshot_export_source_state(source_root: &PathBuf) -> (usize, usize, usize) {
        (
            fs::read_dir(source_root.join("answer_drafts")).map(|entries| entries.count()).unwrap_or(0),
            fs::read_dir(source_root.join("grounded_answers")).map(|entries| entries.count()).unwrap_or(0),
            fs::read_dir(source_root.join("final_answers")).map(|entries| entries.count()).unwrap_or(0),
        )
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

    fn read_export_summary(export_root: &PathBuf) -> AnswerArtifactExportSummary {
        serde_json::from_str(&fs::read_to_string(export_root.join("summary.json")).unwrap()).unwrap()
    }

    fn read_export_manifest(export_root: &PathBuf) -> AnswerArtifactExportManifest {
        serde_json::from_str(&fs::read_to_string(export_root.join("export_manifest.json")).unwrap()).unwrap()
    }

    fn read_export_issues(export_root: &PathBuf) -> AnswerArtifactExportIssues {
        serde_json::from_str(&fs::read_to_string(export_root.join("export_issues.json")).unwrap()).unwrap()
    }

    fn read_export_integrity(export_root: &PathBuf) -> AnswerArtifactExportIntegrity {
        serde_json::from_str(&fs::read_to_string(export_root.join("export_integrity.json")).unwrap()).unwrap()
    }

    fn remove_bundle_schema_version(export_root: &PathBuf, relative_path: &str) {
        let path = export_root.join(relative_path);
        let mut value: serde_json::Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        value.as_object_mut().unwrap().remove("schema_version");
        fs::write(path, serde_json::to_string_pretty(&value).unwrap()).unwrap();
    }

    fn set_bundle_schema_version(export_root: &PathBuf, relative_path: &str, schema_version: &str) {
        let path = export_root.join(relative_path);
        let mut value: serde_json::Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        value.as_object_mut().unwrap().insert(
            "schema_version".to_string(),
            serde_json::Value::String(schema_version.to_string()),
        );
        fs::write(path, serde_json::to_string_pretty(&value).unwrap()).unwrap();
    }

    fn mutate_export_integrity(export_root: &PathBuf, mutate: impl FnOnce(&mut serde_json::Value)) {
        let path = export_root.join("export_integrity.json");
        let mut value: serde_json::Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        mutate(&mut value);
        fs::write(path, serde_json::to_string_pretty(&value).unwrap()).unwrap();
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
    fn answer_artifact_overview_counts_existing_artifacts_without_paths() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, _version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let grounded = service.build_final_answer(&source_id, &grounded_id).unwrap();
        let overview = service.get_answer_artifact_overview(&source_id).unwrap();
        assert_eq!(overview.source_id, source_id);
        assert_eq!(overview.draft_count, 1);
        assert_eq!(overview.grounded_answer_count, 1);
        assert_eq!(overview.final_answer_count, 1);
        assert_eq!(overview.final_answers.len(), 1);
        assert_eq!(overview.final_answers[0].final_answer_id, grounded.final_answer_id);
        assert_eq!(overview.final_answers[0].grounded_answer_id, grounded.grounded_answer_id);
        assert_eq!(overview.final_answers[0].statement_count, grounded.statement_count);
        assert!(overview.final_answers[0].unsupported_count <= overview.final_answers[0].statement_count);
        assert!(!format!("{overview:?}").contains(".aegis"));
    }

    #[test]
    fn answer_artifact_overview_matches_list_final_answers_and_is_deterministic() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let mut first = service.build_final_answer(&source_id, &grounded_id).unwrap();
        first.final_answer_id = "fan_b".to_string();
        let mut second = service.build_final_answer(&source_id, &grounded_id).unwrap();
        second.final_answer_id = "fan_a".to_string();
        let final_dir = CorpusPaths::new(temp.path().to_path_buf())
            .source_version_dir(&source_id, &version_id)
            .join("final_answers");
        fs::create_dir_all(&final_dir).unwrap();
        fs::write(final_dir.join("fan_b.json"), serde_json::to_string_pretty(&first).unwrap()).unwrap();
        fs::write(final_dir.join("fan_a.json"), serde_json::to_string_pretty(&second).unwrap()).unwrap();
        let listed = service.list_final_answers(&source_id).unwrap();
        let overview = service.get_answer_artifact_overview(&source_id).unwrap();
        let listed_ids: Vec<_> = listed.iter().map(|item| item.final_answer_id.clone()).collect();
        let overview_ids: Vec<_> = overview.final_answers.iter().map(|item| item.final_answer_id.clone()).collect();
        assert!(listed_ids.windows(2).all(|pair| pair[0] <= pair[1]));
        assert!(listed_ids.contains(&"fan_a".to_string()));
        assert!(listed_ids.contains(&"fan_b".to_string()));
        assert_eq!(overview_ids, listed_ids);
        assert_eq!(overview.final_answer_count, listed_ids.len());
    }

    #[test]
    fn answer_artifact_overview_counts_multiple_artifacts_correctly() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, version_id, draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let draft = crate::answer_draft::AnswerDraftService::new(temp.path().to_path_buf())
            .read_answer_draft(&source_id, &draft_id)
            .unwrap();
        let draft_dir = CorpusPaths::new(temp.path().to_path_buf())
            .source_version_dir(&source_id, &version_id)
            .join("answer_drafts");
        fs::create_dir_all(&draft_dir).unwrap();
        fs::write(draft_dir.join("adr_extra.json"), serde_json::to_string_pretty(&draft).unwrap()).unwrap();

        let grounded = service.build_final_answer(&source_id, &grounded_id).unwrap();
        let grounded_dir = CorpusPaths::new(temp.path().to_path_buf())
            .source_version_dir(&source_id, &version_id)
            .join("grounded_answers");
        fs::create_dir_all(&grounded_dir).unwrap();
        let mut grounded_extra = crate::grounded_answer::read_grounded_answer(temp.path().to_path_buf(), &source_id, &grounded.grounded_answer_id).unwrap();
        grounded_extra.grounded_answer_id = "gan_extra".to_string();
        fs::write(grounded_dir.join("gan_extra.json"), serde_json::to_string_pretty(&grounded_extra).unwrap()).unwrap();

        let overview = service.get_answer_artifact_overview(&source_id).unwrap();
        assert_eq!(overview.draft_count, 2);
        assert_eq!(overview.grounded_answer_count, 2);
        assert_eq!(overview.final_answer_count, overview.final_answers.len());
        assert_eq!(overview.final_answers.len(), 1);
        assert_eq!(overview.final_answers[0].final_answer_id, grounded.final_answer_id);
    }

    #[test]
    fn answer_artifact_overview_reports_typed_error_for_malformed_final_answer() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let _ = service.build_final_answer(&source_id, &grounded_id).unwrap();
        let bad_path = CorpusPaths::new(temp.path().to_path_buf())
            .source_version_dir(&source_id, &version_id)
            .join("final_answers")
            .join("fan_bad.json");
        fs::write(&bad_path, "{not-json").unwrap();
        assert!(matches!(service.get_answer_artifact_overview(&source_id), Err(AegisError::FinalAnswerReadFailed)));
    }

    #[test]
    fn answer_artifact_overview_rejects_empty_and_missing_sources() {
        let temp = tempfile::tempdir().unwrap();
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        assert!(matches!(service.get_answer_artifact_overview(""), Err(AegisError::FinalAnswerInputMissing)));
        assert!(matches!(service.get_answer_artifact_overview("src_missing"), Err(AegisError::SourceNotFound(_))));
    }

    #[test]
    fn answer_artifact_overview_does_not_create_directories_or_build_artifacts() {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("notes.md");
        fs::write(&source_path, "alpha beta gamma").unwrap();
        let authority = CorpusAuthority::new(temp.path().to_path_buf());
        authority.register_source(source_path.to_string_lossy().to_string(), valid_metadata()).unwrap();
        let registry = SourceRegistry::load(&CorpusPaths::new(temp.path().to_path_buf()).registry_path()).unwrap();
        let source_id = registry.sources.first().unwrap().source_id.clone();
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let overview = service.get_answer_artifact_overview(&source_id).unwrap();
        assert_eq!(overview.draft_count, 0);
        assert_eq!(overview.grounded_answer_count, 0);
        assert_eq!(overview.final_answer_count, 0);
        let corpus = CorpusPaths::new(temp.path().to_path_buf());
        let version_dir = corpus.source_version_dir(&source_id, &registry.sources.first().unwrap().version_id);
        assert!(!version_dir.join("answer_drafts").exists());
        assert!(!version_dir.join("grounded_answers").exists());
        assert!(!version_dir.join("final_answers").exists());
    }

    #[test]
    fn answer_artifact_source_index_returns_deterministic_counts_without_paths() {
        let temp = tempfile::tempdir().unwrap();
        let (source_a, _version_a, draft_a, grounded_a) = prepare_grounded(&temp.path().to_path_buf());
        let source_path_b = temp.path().join("notes_b.md");
        fs::write(&source_path_b, "delta epsilon zeta").unwrap();
        let authority = CorpusAuthority::new(temp.path().to_path_buf());
        authority.register_source(source_path_b.to_string_lossy().to_string(), valid_metadata()).unwrap();
        let registry = SourceRegistry::load(&CorpusPaths::new(temp.path().to_path_buf()).registry_path()).unwrap();
        let source_b = registry.sources.iter().find(|record| record.source_id != source_a).cloned().unwrap();

        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let draft = crate::answer_draft::AnswerDraftService::new(temp.path().to_path_buf())
            .read_answer_draft(&source_a, &draft_a)
            .unwrap();
        let draft_path_b = CorpusPaths::new(temp.path().to_path_buf())
            .source_version_dir(&source_b.source_id, &source_b.version_id)
            .join("answer_drafts")
            .join(format!("{}.json", draft.answer_draft_id));
        fs::create_dir_all(draft_path_b.parent().unwrap()).unwrap();
        fs::write(&draft_path_b, serde_json::to_string_pretty(&draft).unwrap()).unwrap();

        let grounded = service.build_final_answer(&source_a, &grounded_a).unwrap();
        let grounded_path_b = CorpusPaths::new(temp.path().to_path_buf())
            .source_version_dir(&source_b.source_id, &source_b.version_id)
            .join("grounded_answers")
            .join("gan_copy.json");
        fs::create_dir_all(grounded_path_b.parent().unwrap()).unwrap();
        let mut grounded_copy = crate::grounded_answer::read_grounded_answer(temp.path().to_path_buf(), &source_a, &grounded.grounded_answer_id).unwrap();
        grounded_copy.grounded_answer_id = "gan_copy".to_string();
        fs::write(&grounded_path_b, serde_json::to_string_pretty(&grounded_copy).unwrap()).unwrap();

        let final_path_b = CorpusPaths::new(temp.path().to_path_buf())
            .source_version_dir(&source_b.source_id, &source_b.version_id)
            .join("final_answers")
            .join("fan_copy.json");
        fs::create_dir_all(final_path_b.parent().unwrap()).unwrap();
        let mut final_copy = service.build_final_answer(&source_a, &grounded.grounded_answer_id).unwrap();
        final_copy.final_answer_id = "fan_copy".to_string();
        fs::write(&final_path_b, serde_json::to_string_pretty(&final_copy).unwrap()).unwrap();

        let index = service.list_answer_artifact_sources().unwrap();
        assert_eq!(index.len(), 2);
        assert!(index.windows(2).all(|pair| pair[0].source_id <= pair[1].source_id));
        assert!(index.iter().all(|item| !format!("{item:?}").contains(".aegis")));
        let by_id = |source_id: &str| index.iter().find(|item| item.source_id == source_id).unwrap();
        assert_eq!(by_id(&source_a).draft_count, 1);
        assert_eq!(by_id(&source_a).grounded_answer_count, 1);
        assert_eq!(by_id(&source_a).final_answer_count, 1);
        assert_eq!(by_id(&source_b.source_id).draft_count, 1);
        assert_eq!(by_id(&source_b.source_id).grounded_answer_count, 1);
        assert_eq!(by_id(&source_b.source_id).final_answer_count, 1);
    }

    #[test]
    fn answer_artifact_source_index_is_empty_when_storage_has_no_relevant_artifacts() {
        let temp = tempfile::tempdir().unwrap();
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let index = service.list_answer_artifact_sources().unwrap();
        assert!(index.is_empty());
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn answer_artifact_health_is_zero_for_empty_storage() {
        let temp = tempfile::tempdir().unwrap();
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let health = service.get_answer_artifact_health().unwrap();
        assert_eq!(health.source_count, 0);
        assert_eq!(health.draft_count, 0);
        assert_eq!(health.grounded_answer_count, 0);
        assert_eq!(health.final_answer_count, 0);
        assert_eq!(health.malformed_final_answer_count, 0);
        assert_eq!(health.unsupported_statement_count, 0);
        assert_eq!(health.needs_evidence_statement_count, 0);
        assert!(health.sources.is_empty());
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn answer_artifact_health_counts_sources_deterministically_without_paths() {
        let temp = tempfile::tempdir().unwrap();
        let (source_a, _version_a, draft_a, grounded_a) = prepare_grounded(&temp.path().to_path_buf());
        let source_path_b = temp.path().join("notes_b.md");
        fs::write(&source_path_b, "delta epsilon zeta").unwrap();
        let authority = CorpusAuthority::new(temp.path().to_path_buf());
        authority.register_source(source_path_b.to_string_lossy().to_string(), valid_metadata()).unwrap();
        let registry = SourceRegistry::load(&CorpusPaths::new(temp.path().to_path_buf()).registry_path()).unwrap();
        let source_b = registry.sources.iter().find(|record| record.source_id != source_a).cloned().unwrap();

        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let draft = crate::answer_draft::AnswerDraftService::new(temp.path().to_path_buf())
            .read_answer_draft(&source_a, &draft_a)
            .unwrap();
        let grounded = crate::grounded_answer::read_grounded_answer(temp.path().to_path_buf(), &source_a, &grounded_a).unwrap();
        let final_answer = service.build_final_answer(&source_a, &grounded.grounded_answer_id).unwrap();

        let source_b_dir = CorpusPaths::new(temp.path().to_path_buf()).source_version_dir(&source_b.source_id, &source_b.version_id);
        fs::create_dir_all(source_b_dir.join("answer_drafts")).unwrap();
        fs::create_dir_all(source_b_dir.join("grounded_answers")).unwrap();
        fs::create_dir_all(source_b_dir.join("final_answers")).unwrap();
        let mut draft_b = draft.clone();
        draft_b.answer_draft_id = "adr_copy".to_string();
        draft_b.source_id = source_b.source_id.clone();
        draft_b.version_id = source_b.version_id.clone();
        fs::write(
            source_b_dir.join("answer_drafts").join("adr_copy.json"),
            serde_json::to_string_pretty(&draft_b).unwrap(),
        )
        .unwrap();

        let mut grounded_b = grounded.clone();
        grounded_b.grounded_answer_id = "gan_copy".to_string();
        grounded_b.source_id = source_b.source_id.clone();
        grounded_b.version_id = source_b.version_id.clone();
        fs::write(
            source_b_dir.join("grounded_answers").join("gan_copy.json"),
            serde_json::to_string_pretty(&grounded_b).unwrap(),
        )
        .unwrap();

        let mut final_b = final_answer.clone();
        final_b.final_answer_id = "fan_copy".to_string();
        final_b.source_id = source_b.source_id.clone();
        final_b.version_id = source_b.version_id.clone();
        fs::write(
            source_b_dir.join("final_answers").join("fan_copy.json"),
            serde_json::to_string_pretty(&final_b).unwrap(),
        )
        .unwrap();

        let health = service.get_answer_artifact_health().unwrap();
        assert_eq!(health.source_count, 2);
        assert_eq!(health.draft_count, 2);
        assert_eq!(health.grounded_answer_count, 2);
        assert_eq!(health.final_answer_count, 2);
        assert_eq!(health.malformed_final_answer_count, 0);
        assert_eq!(health.unsupported_statement_count, 0);
        assert_eq!(health.needs_evidence_statement_count, 0);
        assert!(health.sources.windows(2).all(|pair| pair[0].source_id <= pair[1].source_id));
        assert!(health.sources.iter().all(|item| !format!("{item:?}").contains(".aegis")));
        let per_source_drafts: usize = health.sources.iter().map(|item| item.draft_count).sum();
        let per_source_grounded: usize = health.sources.iter().map(|item| item.grounded_answer_count).sum();
        let per_source_final: usize = health.sources.iter().map(|item| item.final_answer_count).sum();
        let per_source_malformed: usize = health.sources.iter().map(|item| item.malformed_final_answer_count).sum();
        let per_source_unsupported: usize = health.sources.iter().map(|item| item.unsupported_statement_count).sum();
        let per_source_needs_evidence: usize = health.sources.iter().map(|item| item.needs_evidence_statement_count).sum();
        assert_eq!(health.draft_count, per_source_drafts);
        assert_eq!(health.grounded_answer_count, per_source_grounded);
        assert_eq!(health.final_answer_count, per_source_final);
        assert_eq!(health.malformed_final_answer_count, per_source_malformed);
        assert_eq!(health.unsupported_statement_count, per_source_unsupported);
        assert_eq!(health.needs_evidence_statement_count, per_source_needs_evidence);
        assert!(!format!("{health:?}").contains(".aegis"));
    }

    #[test]
    fn answer_artifact_health_counts_malformed_finals_conservatively() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let _ = service.build_final_answer(&source_id, &grounded_id).unwrap();
        let final_dir = CorpusPaths::new(temp.path().to_path_buf())
            .source_version_dir(&source_id, &version_id)
            .join("final_answers");
        fs::write(final_dir.join("fan_bad.json"), "{not-json").unwrap();
        fs::write(final_dir.join("notes.txt"), "ignore me").unwrap();

        let health = service.get_answer_artifact_health().unwrap();
        assert_eq!(health.source_count, 1);
        assert_eq!(health.final_answer_count, 1);
        assert_eq!(health.malformed_final_answer_count, 1);
        assert_eq!(health.unsupported_statement_count, 0);
        assert_eq!(health.needs_evidence_statement_count, 0);
    }

    #[test]
    fn answer_artifact_health_counts_unsupported_and_needs_evidence_statements() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, version_id, draft_id, _grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let mut draft = crate::answer_draft::AnswerDraftService::new(temp.path().to_path_buf())
            .read_answer_draft(&source_id, &draft_id)
            .unwrap();
        draft.claims[0].status = crate::answer_draft::DraftClaimStatus::NeedsEvidence;
        draft.claims[0].confidence = crate::answer_draft::DraftClaimConfidence::MissingEvidence;
        let draft_path = CorpusPaths::new(temp.path().to_path_buf())
            .source_version_dir(&source_id, &version_id)
            .join("answer_drafts")
            .join(format!("{}.json", draft.answer_draft_id));
        fs::write(&draft_path, serde_json::to_string_pretty(&draft).unwrap()).unwrap();
        let grounded_needs = build_grounded_answer(temp.path().to_path_buf(), &source_id, &draft_id).unwrap();
        let final_needs = service.build_final_answer(&source_id, &grounded_needs.grounded_answer_id).unwrap();

        let mut unsupported_grounded = grounded_needs.clone();
        unsupported_grounded.grounded_answer_id = "gan_unsupported".to_string();
        unsupported_grounded.statements[0].status = GroundedStatementStatus::Unsupported;
        unsupported_grounded.statements[0].support_level = GroundedSupportLevel::MissingEvidence;
        let unsupported_final = build_final_answer_from_grounded_answer(&unsupported_grounded).unwrap();
        let final_dir = CorpusPaths::new(temp.path().to_path_buf())
            .source_version_dir(&source_id, &version_id)
            .join("final_answers");
        fs::write(
            final_dir.join("fan_unsupported.json"),
            serde_json::to_string_pretty(&unsupported_final).unwrap(),
        )
        .unwrap();

        let health = service.get_answer_artifact_health().unwrap();
        assert_eq!(health.final_answer_count, 2);
        assert_eq!(health.malformed_final_answer_count, 0);
        assert_eq!(health.unsupported_statement_count, 1);
        assert_eq!(health.needs_evidence_statement_count, 1);
        assert_eq!(final_needs.statements[0].status, FinalAnswerStatementStatus::NeedsEvidence);
    }

    #[test]
    fn answer_artifact_health_does_not_create_directories_or_build_artifacts() {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("notes.md");
        fs::write(&source_path, "alpha beta gamma").unwrap();
        let authority = CorpusAuthority::new(temp.path().to_path_buf());
        authority.register_source(source_path.to_string_lossy().to_string(), valid_metadata()).unwrap();
        let registry = SourceRegistry::load(&CorpusPaths::new(temp.path().to_path_buf()).registry_path()).unwrap();
        let source_id = registry.sources.first().unwrap().source_id.clone();
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let health = service.get_answer_artifact_health().unwrap();
        assert_eq!(health.source_count, 0);
        assert_eq!(health.draft_count, 0);
        assert_eq!(health.grounded_answer_count, 0);
        assert_eq!(health.final_answer_count, 0);
        let corpus = CorpusPaths::new(temp.path().to_path_buf());
        let version_dir = corpus.source_version_dir(&source_id, &registry.sources.first().unwrap().version_id);
        assert!(!version_dir.join("answer_drafts").exists());
        assert!(!version_dir.join("grounded_answers").exists());
        assert!(!version_dir.join("final_answers").exists());
    }

    #[test]
    fn answer_artifact_issues_is_empty_for_empty_storage() {
        let temp = tempfile::tempdir().unwrap();
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let issues = service.list_answer_artifact_issues().unwrap();
        assert!(issues.is_empty());
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn answer_artifact_issues_counts_malformed_supported_and_unsupported_statements() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, version_id, draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let valid_final = service.build_final_answer(&source_id, &grounded_id).unwrap();
        let final_dir = CorpusPaths::new(temp.path().to_path_buf())
            .source_version_dir(&source_id, &version_id)
            .join("final_answers");
        let mut unsupported = valid_final.clone();
        unsupported.final_answer_id = "fan_unsupported".to_string();
        unsupported.statements[0].status = FinalAnswerStatementStatus::Unsupported;
        unsupported.statement_count = 1;
        unsupported.unsupported_count = 1;
        fs::write(
            final_dir.join("fan_unsupported.json"),
            serde_json::to_string_pretty(&unsupported).unwrap(),
        )
        .unwrap();
        let mut needs = valid_final.clone();
        needs.final_answer_id = "fan_needs".to_string();
        needs.statements[0].status = FinalAnswerStatementStatus::NeedsEvidence;
        needs.statement_count = 1;
        needs.unsupported_count = 1;
        fs::write(final_dir.join("fan_needs.json"), serde_json::to_string_pretty(&needs).unwrap()).unwrap();
        fs::write(final_dir.join("fan_bad.json"), "{not-json").unwrap();
        fs::write(final_dir.join("notes.txt"), "ignore me").unwrap();

        let issues = service.list_answer_artifact_issues().unwrap();
        assert_eq!(issues.len(), 3);
        assert_eq!(issues[0].issue_kind, AnswerArtifactIssueKind::MalformedFinalAnswer);
        assert_eq!(issues[1].issue_kind, AnswerArtifactIssueKind::NeedsEvidenceStatement);
        assert_eq!(issues[2].issue_kind, AnswerArtifactIssueKind::UnsupportedStatement);
        assert_eq!(issues[0].source_id, source_id);
        assert_eq!(issues[1].final_answer_id.as_deref(), Some("fan_needs"));
        assert_eq!(issues[2].final_answer_id.as_deref(), Some("fan_unsupported"));
        assert_eq!(issues[1].statement_status.as_deref(), Some("needs_evidence"));
        assert_eq!(issues[2].statement_status.as_deref(), Some("unsupported"));
        assert!(!format!("{issues:?}").contains(".aegis"));
        assert!(!draft_id.is_empty());
    }

    #[test]
    fn answer_artifact_issues_do_not_include_supported_statements() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, _version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let _ = service.build_final_answer(&source_id, &grounded_id).unwrap();

        let issues = service.list_answer_artifact_issues().unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn answer_artifact_export_manifest_is_empty_for_empty_storage() {
        let temp = tempfile::tempdir().unwrap();
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let manifest = service.get_answer_artifact_export_manifest().unwrap();
        assert_eq!(manifest.schema_version, ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION);
        assert_eq!(manifest.source_count, 0);
        assert_eq!(manifest.draft_count, 0);
        assert_eq!(manifest.grounded_answer_count, 0);
        assert_eq!(manifest.final_answer_count, 0);
        assert_eq!(manifest.issue_count, 0);
        assert!(manifest.sources.is_empty());
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn answer_artifact_export_manifest_counts_artifacts_and_issues() {
        let temp = tempfile::tempdir().unwrap();
        let (source_a, version_a, draft_a, grounded_a) = prepare_grounded(&temp.path().to_path_buf());
        let source_path_b = temp.path().join("notes_b.md");
        fs::write(&source_path_b, "delta epsilon zeta").unwrap();
        let authority = CorpusAuthority::new(temp.path().to_path_buf());
        authority.register_source(source_path_b.to_string_lossy().to_string(), valid_metadata()).unwrap();
        let registry = SourceRegistry::load(&CorpusPaths::new(temp.path().to_path_buf()).registry_path()).unwrap();
        let source_b = registry.sources.iter().find(|record| record.source_id != source_a).cloned().unwrap();

        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let draft = crate::answer_draft::AnswerDraftService::new(temp.path().to_path_buf())
            .read_answer_draft(&source_a, &draft_a)
            .unwrap();
        let grounded = crate::grounded_answer::read_grounded_answer(temp.path().to_path_buf(), &source_a, &grounded_a).unwrap();
        let final_answer = service.build_final_answer(&source_a, &grounded.grounded_answer_id).unwrap();

        let source_a_dir = CorpusPaths::new(temp.path().to_path_buf()).source_version_dir(&source_a, &version_a);
        let draft_dir_a = source_a_dir.join("answer_drafts");
        fs::write(draft_dir_a.join("adr_extra.json"), serde_json::to_string_pretty(&draft).unwrap()).unwrap();
        let grounded_dir_a = source_a_dir.join("grounded_answers");
        fs::write(grounded_dir_a.join("gan_extra.json"), serde_json::to_string_pretty(&grounded).unwrap()).unwrap();
        let final_dir_a = source_a_dir.join("final_answers");
        let mut unsupported = final_answer.clone();
        unsupported.final_answer_id = "fan_unsupported".to_string();
        unsupported.statements[0].status = FinalAnswerStatementStatus::Unsupported;
        unsupported.statement_count = 1;
        unsupported.unsupported_count = 1;
        fs::write(final_dir_a.join("fan_unsupported.json"), serde_json::to_string_pretty(&unsupported).unwrap()).unwrap();
        fs::write(final_dir_a.join("fan_bad.json"), "{not-json").unwrap();

        let source_b_dir = CorpusPaths::new(temp.path().to_path_buf()).source_version_dir(&source_b.source_id, &source_b.version_id);
        fs::create_dir_all(source_b_dir.join("answer_drafts")).unwrap();
        fs::create_dir_all(source_b_dir.join("grounded_answers")).unwrap();
        fs::create_dir_all(source_b_dir.join("final_answers")).unwrap();
        fs::write(source_b_dir.join("answer_drafts").join("adr_b.json"), serde_json::to_string_pretty(&draft).unwrap()).unwrap();
        fs::write(source_b_dir.join("grounded_answers").join("gan_b.json"), serde_json::to_string_pretty(&grounded).unwrap()).unwrap();
        let mut needs = final_answer.clone();
        needs.final_answer_id = "fan_b_needs".to_string();
        needs.statements[0].status = FinalAnswerStatementStatus::NeedsEvidence;
        needs.statement_count = 1;
        needs.unsupported_count = 1;
        fs::write(source_b_dir.join("final_answers").join("fan_b_needs.json"), serde_json::to_string_pretty(&needs).unwrap()).unwrap();

        let manifest = service.get_answer_artifact_export_manifest().unwrap();
        let issues = service.list_answer_artifact_issues().unwrap();

        assert_eq!(manifest.schema_version, ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION);
        assert_eq!(manifest.source_count, 2);
        assert_eq!(manifest.draft_count, 3);
        assert_eq!(manifest.grounded_answer_count, 3);
        assert_eq!(manifest.final_answer_count, 3);
        assert_eq!(manifest.issue_count, issues.len());
        assert!(manifest.sources.windows(2).all(|pair| pair[0].source_id <= pair[1].source_id));
        assert!(manifest.sources.iter().all(|item| !format!("{item:?}").contains(".aegis")));
        let source_a_manifest = manifest.sources.iter().find(|item| item.source_id == source_a).unwrap();
        let source_b_manifest = manifest.sources.iter().find(|item| item.source_id == source_b.source_id).unwrap();
        assert_eq!(source_a_manifest.final_answers.len(), 2);
        assert_eq!(source_b_manifest.final_answers.len(), 1);
        assert_eq!(source_a_manifest.issue_count + source_b_manifest.issue_count, manifest.issue_count);
        assert_eq!(source_a_manifest.issue_count, issues.iter().filter(|issue| issue.source_id == source_a).count());
        assert_eq!(source_b_manifest.issue_count, issues.iter().filter(|issue| issue.source_id == source_b.source_id).count());
        assert!(source_a_manifest.final_answers.iter().all(|item| item.final_answer_id != "fan_bad"));
        assert!(source_a_manifest.final_answers.iter().any(|item| item.final_answer_id == final_answer.final_answer_id));
        assert!(source_a_manifest.final_answers.iter().any(|item| item.final_answer_id == "fan_unsupported"));
        assert_eq!(source_b_manifest.final_answers[0].final_answer_id, "fan_b_needs");
        assert!(!format!("{manifest:?}").contains(".aegis"));
    }

    #[test]
    fn answer_artifact_export_manifest_does_not_create_directories_or_artifacts() {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("notes.md");
        fs::write(&source_path, "alpha beta gamma").unwrap();
        let authority = CorpusAuthority::new(temp.path().to_path_buf());
        authority.register_source(source_path.to_string_lossy().to_string(), valid_metadata()).unwrap();
        let registry = SourceRegistry::load(&CorpusPaths::new(temp.path().to_path_buf()).registry_path()).unwrap();
        let source_id = registry.sources.first().unwrap().source_id.clone();
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let manifest = service.get_answer_artifact_export_manifest().unwrap();
        assert_eq!(manifest.source_count, 0);
        let version_dir = CorpusPaths::new(temp.path().to_path_buf()).source_version_dir(&source_id, &registry.sources.first().unwrap().version_id);
        assert!(!version_dir.join("answer_drafts").exists());
        assert!(!version_dir.join("grounded_answers").exists());
        assert!(!version_dir.join("final_answers").exists());
        assert!(!version_dir.join("export").exists());
    }

    #[test]
    fn answer_artifact_export_requires_explicit_destination_and_is_deterministic() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, version_id, draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let _draft = crate::answer_draft::AnswerDraftService::new(temp.path().to_path_buf())
            .read_answer_draft(&source_id, &draft_id)
            .unwrap();
        let grounded = crate::grounded_answer::read_grounded_answer(temp.path().to_path_buf(), &source_id, &grounded_id).unwrap();
        let final_answer = service.build_final_answer(&source_id, &grounded.grounded_answer_id).unwrap();
        let manifest = service.get_answer_artifact_export_manifest().unwrap();
        let export_root = temp.path().join("answer-export");
        fs::create_dir(&export_root).unwrap();

        let source_root = CorpusPaths::new(temp.path().to_path_buf()).source_version_dir(&source_id, &version_id);
        let before_snapshot = snapshot_export_source_state(&source_root);
        let result = service.export_answer_artifacts(&export_root).unwrap();
        let after_snapshot = snapshot_export_source_state(&source_root);

        assert_eq!(result.schema_version, ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION);
        assert_eq!(result.manifest.schema_version, ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION);
        assert_eq!(result.manifest.source_count, manifest.source_count);
        assert_eq!(result.manifest.issue_count, manifest.issue_count);
        assert_eq!(result.exported_source_count, manifest.source_count);
        assert_eq!(result.exported_draft_count, manifest.draft_count);
        assert_eq!(result.exported_grounded_answer_count, manifest.grounded_answer_count);
        assert_eq!(result.exported_final_answer_count, manifest.final_answer_count);
        assert_eq!(result.exported_issue_count, manifest.issue_count);
        assert!(!result.export_id.is_empty());
        assert!(result.written_files.iter().all(|item| !item.relative_path.starts_with('C') && !item.relative_path.contains(':') && !item.relative_path.starts_with('\\')));
        assert!(result.written_files.iter().all(|item| !PathBuf::from(&item.relative_path).is_absolute()));
        assert!(!format!("{result:?}").contains(temp.path().to_string_lossy().as_ref()));
        assert!(!format!("{result:?}").contains(".aegis"));
        assert!(export_root.join("export_manifest.json").exists());
        assert!(export_root.join("export_issues.json").exists());
        assert!(export_root.join("summary.json").exists());
        assert_eq!(read_export_manifest(&export_root).schema_version, ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION);
        assert_eq!(read_export_issues(&export_root).schema_version, ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION);
        assert_eq!(read_export_summary(&export_root).schema_version, ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION);
        assert_eq!(read_export_integrity(&export_root).schema_version, ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION);
        assert!(export_root.join(&source_id).join("answer_drafts").join(format!("{draft_id}.json")).exists());
        assert!(export_root.join(&source_id).join("grounded_answers").join(format!("{grounded_id}.json")).exists());
        assert!(export_root.join(&source_id).join("final_answers").join(format!("{}.json", final_answer.final_answer_id)).exists());
        assert!(result.written_files.iter().any(|item| item.artifact_kind == ExportedArtifactKind::Summary));
        assert!(result.written_files.iter().any(|item| item.artifact_kind == ExportedArtifactKind::Integrity));
        assert_eq!(before_snapshot, after_snapshot);
        assert!(matches!(service.export_answer_artifacts(&export_root), Err(AegisError::ExportDestinationExists)));
    }

    #[test]
    fn answer_artifact_export_rejects_empty_destination_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        assert!(matches!(service.export_answer_artifacts(""), Err(AegisError::ExportDestinationMissing)));
        assert!(!temp.path().join("export_manifest.json").exists());
        assert!(!temp.path().join("export_issues.json").exists());
        assert!(!temp.path().join("summary.json").exists());
    }

    #[test]
    fn answer_artifact_export_summary_is_empty_for_empty_storage() {
        let temp = tempfile::tempdir().unwrap();
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let export_root = temp.path().join("empty-export");

        let result = service.export_answer_artifacts(&export_root).unwrap();
        let summary = read_export_summary(&export_root);
        let summary_json = fs::read_to_string(export_root.join("summary.json")).unwrap();

        assert!(!summary.export_id.is_empty());
        assert_eq!(summary.schema_version, ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION);
        assert_eq!(result.schema_version, ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION);
        assert_eq!(summary.generated_from, "persisted_artifacts");
        assert_eq!(summary.export_scope, "manual_artifact_export");
        assert_eq!(summary.source_count, 0);
        assert_eq!(summary.draft_count, 0);
        assert_eq!(summary.grounded_answer_count, 0);
        assert_eq!(summary.final_answer_count, 0);
        assert_eq!(summary.issue_count, 0);
        assert!(summary.issue_kinds.is_empty());
        assert!(summary.sources.is_empty());
        assert!(summary.non_goals.iter().any(|item| item == "no_generation"));
        assert!(summary.non_goals.iter().any(|item| item == "no_repair"));
        assert!(summary.non_goals.iter().any(|item| item == "no_editing"));
        assert!(summary.non_goals.iter().any(|item| item == "no_import"));
        assert!(summary.non_goals.iter().any(|item| item == "no_share"));
        assert!(result.written_files.iter().any(|item| item.relative_path == "summary.json"));
        assert!(result.written_files.iter().any(|item| item.artifact_kind == ExportedArtifactKind::Summary));
        assert!(!format!("{summary:?}").contains(temp.path().to_string_lossy().as_ref()));
        assert!(!format!("{summary:?}").contains(".aegis"));
        assert!(!summary_json.contains(temp.path().to_string_lossy().as_ref()));
        assert!(!summary_json.contains(".aegis"));
    }

    #[test]
    fn answer_artifact_export_summary_matches_manifest_issues_and_is_deterministic() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let valid_final = service.build_final_answer(&source_id, &grounded_id).unwrap();
        let final_dir = CorpusPaths::new(temp.path().to_path_buf())
            .source_version_dir(&source_id, &version_id)
            .join("final_answers");
        fs::write(final_dir.join("fan_bad.json"), "{not-json").unwrap();
        let mut unsupported = valid_final.clone();
        unsupported.final_answer_id = "fan_unsupported".to_string();
        unsupported.statements[0].status = FinalAnswerStatementStatus::Unsupported;
        unsupported.statement_count = 1;
        unsupported.unsupported_count = 1;
        fs::write(final_dir.join("fan_unsupported.json"), serde_json::to_string_pretty(&unsupported).unwrap()).unwrap();
        let mut needs = valid_final.clone();
        needs.final_answer_id = "fan_needs".to_string();
        needs.statements[0].status = FinalAnswerStatementStatus::NeedsEvidence;
        needs.statement_count = 1;
        needs.unsupported_count = 1;
        fs::write(final_dir.join("fan_needs.json"), serde_json::to_string_pretty(&needs).unwrap()).unwrap();

        let export_root_a = temp.path().join("export-a");
        let export_root_b = temp.path().join("export-b");
        let first = service.export_answer_artifacts(&export_root_a).unwrap();
        let _second = service.export_answer_artifacts(&export_root_b).unwrap();
        let summary_a = read_export_summary(&export_root_a);
        let summary_b = read_export_summary(&export_root_b);
        let issues = service.list_answer_artifact_issues().unwrap();

        assert!(!summary_a.export_id.is_empty());
        assert!(!summary_b.export_id.is_empty());
        assert_eq!(summary_a.export_id, summary_b.export_id);
        assert_eq!(summary_a.schema_version, ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION);
        assert_eq!(summary_b.schema_version, ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION);
        assert_eq!(summary_a.source_count, first.manifest.source_count);
        assert_eq!(summary_a.draft_count, first.manifest.draft_count);
        assert_eq!(summary_a.grounded_answer_count, first.manifest.grounded_answer_count);
        assert_eq!(summary_a.final_answer_count, first.manifest.final_answer_count);
        assert_eq!(summary_a.issue_count, first.manifest.issue_count);
        assert_eq!(summary_a.issue_count, issues.len());
        assert_eq!(summary_a.sources.len(), first.manifest.sources.len());
        assert_eq!(summary_a.sources[0].source_id, first.manifest.sources[0].source_id);
        assert_eq!(summary_a.sources[0].draft_count, first.manifest.sources[0].draft_count);
        assert_eq!(summary_a.sources[0].grounded_answer_count, first.manifest.sources[0].grounded_answer_count);
        assert_eq!(summary_a.sources[0].final_answer_count, first.manifest.sources[0].final_answer_count);
        assert_eq!(summary_a.sources[0].issue_count, first.manifest.sources[0].issue_count);
        assert_eq!(summary_a.issue_kinds.iter().map(|item| item.count).sum::<usize>(), summary_a.issue_count);
        assert_eq!(summary_a, summary_b);
        assert!(!format!("{summary_a:?}").contains(temp.path().to_string_lossy().as_ref()));
        assert!(!format!("{summary_a:?}").contains(".aegis"));
        assert!(summary_a.issue_kinds.iter().all(|item| item.count > 0));
        assert!(summary_a.issue_kinds.iter().any(|item| item.issue_kind == AnswerArtifactIssueKind::MalformedFinalAnswer));
        assert!(summary_a.issue_kinds.iter().any(|item| item.issue_kind == AnswerArtifactIssueKind::NeedsEvidenceStatement));
        assert!(summary_a.issue_kinds.iter().any(|item| item.issue_kind == AnswerArtifactIssueKind::UnsupportedStatement));
        assert!(summary_a.sources.iter().all(|item| item.issue_count == issues.iter().filter(|issue| issue.source_id == item.source_id).count()));
        assert!(first.written_files.iter().any(|item| item.relative_path == "summary.json" && item.artifact_kind == ExportedArtifactKind::Summary));
        assert_eq!(first.manifest.sources[0].final_answers.len(), 3);
        assert!(first.manifest.sources[0].final_answers.iter().all(|item| item.final_answer_id != "fan_bad"));
        assert!(first.manifest.issue_count > 0);
        assert!(first.manifest.sources[0].issue_count > 0);
        assert!(first.written_files.iter().any(|item| item.artifact_kind == ExportedArtifactKind::Integrity));
    }

    #[test]
    fn answer_artifact_export_summary_export_id_changes_when_exported_content_changes() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let valid_final = service.build_final_answer(&source_id, &grounded_id).unwrap();
        let final_dir = CorpusPaths::new(temp.path().to_path_buf())
            .source_version_dir(&source_id, &version_id)
            .join("final_answers");
        fs::write(final_dir.join("fan_bad.json"), "{not-json").unwrap();

        let export_root_a = temp.path().join("export-a");
        let baseline = service.export_answer_artifacts(&export_root_a).unwrap();
        let baseline_summary = read_export_summary(&export_root_a);

        let mut changed = valid_final.clone();
        changed.final_answer_id = "fan_changed".to_string();
        changed.statements[0].text = "changed summary content".to_string();
        changed.statement_count = 1;
        fs::write(final_dir.join("fan_changed.json"), serde_json::to_string_pretty(&changed).unwrap()).unwrap();

        let export_root_b = temp.path().join("export-b");
        let updated = service.export_answer_artifacts(&export_root_b).unwrap();
        let updated_summary = read_export_summary(&export_root_b);

        assert_eq!(baseline.exported_issue_count, baseline_summary.issue_count);
        assert_eq!(updated.exported_issue_count, updated_summary.issue_count);
        assert_ne!(baseline_summary.export_id, updated_summary.export_id);
        assert_ne!(baseline_summary, updated_summary);
        assert!(!format!("{updated_summary:?}").contains(temp.path().to_string_lossy().as_ref()));
        assert!(!format!("{updated_summary:?}").contains(".aegis"));
    }

    #[test]
    fn answer_artifact_export_bundle_inspection_is_consistent_for_valid_export() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, _version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let final_answer = service.build_final_answer(&source_id, &grounded_id).unwrap();
        let export_root = temp.path().join("export-bundle");

        let result = service.export_answer_artifacts(&export_root).unwrap();
        let inspection = inspect_answer_artifact_export_bundle(&export_root).unwrap();
        let summary = read_export_summary(&export_root);

        assert!(inspection.has_manifest);
        assert!(inspection.has_issues);
        assert!(inspection.has_summary);
        assert!(inspection.has_integrity);
        assert_eq!(inspection.schema_version.as_deref(), Some(ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION));
        assert_eq!(inspection.manifest_schema_version.as_deref(), Some(ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION));
        assert_eq!(inspection.issues_schema_version.as_deref(), Some(ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION));
        assert_eq!(inspection.summary_schema_version.as_deref(), Some(ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION));
        assert_eq!(inspection.integrity_schema_version.as_deref(), Some(ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION));
        assert_eq!(inspection.integrity_algorithm.as_deref(), Some(ANSWER_ARTIFACT_EXPORT_INTEGRITY_ALGORITHM));
        assert!(inspection.is_consistent);
        assert_eq!(inspection.issue_count, 0);
        assert_eq!(inspection.warning_count, 0);
        assert!(inspection.errors.is_empty());
        assert!(inspection.warnings.is_empty());
        assert!(inspection.manifest_counts.is_some());
        assert!(inspection.summary_counts.is_some());
        assert!(inspection.integrity_counts.is_some());
        assert!(inspection.issue_kind_counts.is_some());
        assert_eq!(inspection.manifest_counts.as_ref().unwrap().source_count, result.manifest.source_count);
        assert_eq!(inspection.manifest_counts.as_ref().unwrap().draft_count, result.manifest.draft_count);
        assert_eq!(inspection.manifest_counts.as_ref().unwrap().grounded_answer_count, result.manifest.grounded_answer_count);
        assert_eq!(inspection.manifest_counts.as_ref().unwrap().final_answer_count, result.manifest.final_answer_count);
        assert_eq!(inspection.manifest_counts.as_ref().unwrap().issue_count, result.manifest.issue_count);
        assert_eq!(inspection.manifest_counts.as_ref().unwrap().sources.len(), result.manifest.sources.len());
        assert_eq!(inspection.manifest_counts.as_ref().unwrap().sources[0].source_id, result.manifest.sources[0].source_id);
        assert_eq!(inspection.manifest_counts.as_ref().unwrap().sources[0].final_answers.len(), result.manifest.sources[0].final_answers.len());
        assert_eq!(inspection.summary_counts.as_ref().unwrap(), &summary);
        assert_eq!(inspection.manifest_counts.as_ref().unwrap().schema_version, ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION);
        assert_eq!(inspection.summary_counts.as_ref().unwrap().schema_version, ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION);
        assert_eq!(inspection.integrity_counts.as_ref().unwrap().schema_version, ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION);
        assert_eq!(inspection.integrity_counts.as_ref().unwrap().algorithm, ANSWER_ARTIFACT_EXPORT_INTEGRITY_ALGORITHM);
        assert_eq!(inspection.integrity_counts.as_ref().unwrap().files.len(), result.written_files.len() - 1);
        assert!(inspection.inspection_summary.is_consistent);
        assert!(inspection.inspection_summary.schema_supported);
        assert!(inspection.inspection_summary.integrity_verified);
        assert_eq!(inspection.inspection_summary.issue_count, 0);
        assert_eq!(inspection.inspection_summary.warning_count, 0);
        assert_eq!(inspection.inspection_summary.checked_file_count, 4);
        assert_eq!(inspection.inspection_summary.integrity_file_count, result.written_files.len() - 1);
        assert!(inspection.inspection_summary.issue_counts_by_kind.is_empty());
        assert!(inspection.summary_counts.as_ref().unwrap().issue_kinds.is_empty());
        assert!(inspection.issue_kind_counts.as_ref().unwrap().is_empty());
        assert!(inspection.summary_counts.as_ref().unwrap().sources.windows(2).all(|pair| pair[0].source_id <= pair[1].source_id));
        assert!(format!("{inspection:?}").find(temp.path().to_string_lossy().as_ref()).is_none());
        assert!(!final_answer.final_answer_id.is_empty());
    }

    #[test]
    fn answer_artifact_export_bundle_schema_version_is_written_to_bundle_metadata() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, _version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let _ = service.build_final_answer(&source_id, &grounded_id).unwrap();
        let export_root = temp.path().join("schema-version-bundle");

        let result = service.export_answer_artifacts(&export_root).unwrap();
        let manifest = read_export_manifest(&export_root);
        let issues = read_export_issues(&export_root);
        let summary = read_export_summary(&export_root);

        assert_eq!(result.schema_version, ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION);
        assert_eq!(result.manifest.schema_version, ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION);
        assert_eq!(manifest.schema_version, ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION);
        assert_eq!(issues.schema_version, ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION);
        assert_eq!(summary.schema_version, ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION);
        assert_eq!(issues.issues.len(), result.exported_issue_count);
        assert!(!format!("{result:?}{manifest:?}{issues:?}{summary:?}").contains(temp.path().to_string_lossy().as_ref()));
    }

    #[test]
    fn answer_artifact_export_integrity_is_written_and_deterministic() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, _version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let _ = service.build_final_answer(&source_id, &grounded_id).unwrap();
        let export_root_a = temp.path().join("integrity-a");
        let export_root_b = temp.path().join("integrity-b");

        let result_a = service.export_answer_artifacts(&export_root_a).unwrap();
        let result_b = service.export_answer_artifacts(&export_root_b).unwrap();
        let integrity_a = read_export_integrity(&export_root_a);
        let integrity_b = read_export_integrity(&export_root_b);

        assert_eq!(result_a.integrity, integrity_a);
        assert_eq!(result_b.integrity, integrity_b);
        assert_eq!(integrity_a.schema_version, ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION);
        assert_eq!(integrity_a.algorithm, ANSWER_ARTIFACT_EXPORT_INTEGRITY_ALGORITHM);
        assert_eq!(integrity_a, integrity_b);
        assert!(integrity_a.files.windows(2).all(|pair| pair[0].relative_path <= pair[1].relative_path));
        assert!(integrity_a.files.iter().all(|file| !file.relative_path.is_empty() && !file.relative_path.contains("..") && !Path::new(&file.relative_path).is_absolute()));
        assert!(integrity_a.files.iter().all(|file| file.relative_path != "export_integrity.json"));
        assert_eq!(integrity_a.files.len(), result_a.written_files.len() - 1);
    }

    #[test]
    fn answer_artifact_export_bundle_inspection_detects_missing_integrity_file() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, _version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let _ = service.build_final_answer(&source_id, &grounded_id).unwrap();
        let export_root = temp.path().join("missing-integrity-bundle");
        service.export_answer_artifacts(&export_root).unwrap();
        fs::remove_file(export_root.join("export_integrity.json")).unwrap();

        let inspection = inspect_answer_artifact_export_bundle(&export_root).unwrap();

        assert!(!inspection.has_integrity);
        assert!(inspection.errors.iter().any(|issue| issue.kind == AnswerArtifactExportBundleInspectionIssueKind::MissingIntegrity));
        assert!(!inspection.is_consistent);
        assert!(!inspection.inspection_summary.is_consistent);
        assert!(!inspection.inspection_summary.schema_supported);
        assert!(!inspection.inspection_summary.integrity_verified);
        assert_eq!(inspection.inspection_summary.issue_count, 1);
        assert_eq!(inspection.inspection_summary.checked_file_count, 3);
        assert_eq!(inspection.inspection_summary.integrity_file_count, 0);
        assert_eq!(
            inspection.inspection_summary.issue_counts_by_kind.iter().map(|item| (&item.kind, item.count)).collect::<Vec<_>>(),
            vec![(&AnswerArtifactExportBundleInspectionIssueKind::MissingIntegrity, 1)]
        );
        assert!(!format!("{inspection:?}").contains(temp.path().to_string_lossy().as_ref()));
    }

    #[test]
    fn answer_artifact_export_bundle_inspection_detects_malformed_integrity_file() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, _version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let _ = service.build_final_answer(&source_id, &grounded_id).unwrap();
        let export_root = temp.path().join("malformed-integrity-bundle");
        service.export_answer_artifacts(&export_root).unwrap();
        fs::write(export_root.join("export_integrity.json"), "{not-json").unwrap();

        let inspection = inspect_answer_artifact_export_bundle(&export_root).unwrap();

        assert!(inspection.errors.iter().any(|issue| issue.kind == AnswerArtifactExportBundleInspectionIssueKind::IntegrityReadFailed));
        assert!(!inspection.is_consistent);
        assert!(!inspection.inspection_summary.is_consistent);
        assert!(!inspection.inspection_summary.schema_supported);
        assert!(!inspection.inspection_summary.integrity_verified);
        assert_eq!(inspection.inspection_summary.issue_count, 1);
        assert_eq!(inspection.inspection_summary.checked_file_count, 3);
        assert_eq!(inspection.inspection_summary.integrity_file_count, 0);
        assert_eq!(
            inspection.inspection_summary.issue_counts_by_kind.iter().map(|item| (&item.kind, item.count)).collect::<Vec<_>>(),
            vec![(&AnswerArtifactExportBundleInspectionIssueKind::IntegrityReadFailed, 1)]
        );
        assert!(!format!("{inspection:?}").contains(temp.path().to_string_lossy().as_ref()));
    }

    #[test]
    fn answer_artifact_export_bundle_inspection_detects_unsupported_integrity_algorithm() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, _version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let _ = service.build_final_answer(&source_id, &grounded_id).unwrap();
        let export_root = temp.path().join("unsupported-algorithm-bundle");
        service.export_answer_artifacts(&export_root).unwrap();
        mutate_export_integrity(&export_root, |value| {
            value["algorithm"] = serde_json::Value::String("md5".to_string());
        });

        let inspection = inspect_answer_artifact_export_bundle(&export_root).unwrap();

        assert!(inspection.errors.iter().any(|issue| issue.kind == AnswerArtifactExportBundleInspectionIssueKind::IntegrityAlgorithmUnsupported));
        assert!(!inspection.is_consistent);
    }

    #[test]
    fn answer_artifact_export_bundle_inspection_detects_missing_listed_integrity_file() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, _version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let _ = service.build_final_answer(&source_id, &grounded_id).unwrap();
        let export_root = temp.path().join("missing-listed-integrity-file-bundle");
        service.export_answer_artifacts(&export_root).unwrap();
        mutate_export_integrity(&export_root, |value| {
            let files = value["files"].as_array_mut().unwrap();
            files[0]["relative_path"] = serde_json::Value::String("missing-file.json".to_string());
        });

        let inspection = inspect_answer_artifact_export_bundle(&export_root).unwrap();

        assert!(inspection.errors.iter().any(|issue| issue.kind == AnswerArtifactExportBundleInspectionIssueKind::IntegrityMissingFile));
        assert!(!inspection.is_consistent);
    }

    #[test]
    fn answer_artifact_export_bundle_inspection_detects_missing_and_mismatched_integrity_entries() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, _version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let _ = service.build_final_answer(&source_id, &grounded_id).unwrap();
        let export_root = temp.path().join("integrity-entry-bundle");
        service.export_answer_artifacts(&export_root).unwrap();

        mutate_export_integrity(&export_root, |value| {
            let files = value["files"].as_array_mut().unwrap();
            if let Some(first) = files.first_mut() {
                first["relative_path"] = serde_json::Value::String("./bad.json".to_string());
            }
        });

        let inspection = inspect_answer_artifact_export_bundle(&export_root).unwrap();
        assert!(inspection.errors.iter().any(|issue| issue.kind == AnswerArtifactExportBundleInspectionIssueKind::IntegrityPathInvalid));

        mutate_export_integrity(&export_root, |value| {
            let files = value["files"].as_array_mut().unwrap();
            if let Some(first) = files.first_mut() {
                first["relative_path"] = serde_json::Value::String("missing/../bad.json".to_string());
            }
        });

        let inspection = inspect_answer_artifact_export_bundle(&export_root).unwrap();
        assert!(inspection.errors.iter().any(|issue| issue.kind == AnswerArtifactExportBundleInspectionIssueKind::IntegrityPathInvalid));

        mutate_export_integrity(&export_root, |value| {
            let files = value["files"].as_array_mut().unwrap();
            let duplicate = files.first().cloned().unwrap();
            if let Some(first) = files.first_mut() {
                first["relative_path"] = serde_json::Value::String("summary.json".to_string());
                first["byte_count"] = serde_json::Value::Number(serde_json::Number::from(0));
                first["sha256"] = serde_json::Value::String("sha256:deadbeef".to_string());
            }
            files.push(duplicate);
        });

        let inspection = inspect_answer_artifact_export_bundle(&export_root).unwrap();
        assert!(inspection.errors.iter().any(|issue| issue.kind == AnswerArtifactExportBundleInspectionIssueKind::IntegrityDuplicatePath));
        assert!(inspection.errors.iter().any(|issue| issue.kind == AnswerArtifactExportBundleInspectionIssueKind::IntegrityByteCountMismatch));
        assert!(inspection.errors.iter().any(|issue| issue.kind == AnswerArtifactExportBundleInspectionIssueKind::IntegrityDigestMismatch));
        assert!(!inspection.is_consistent);
        assert!(!inspection.inspection_summary.is_consistent);
        assert!(inspection.inspection_summary.schema_supported);
        assert!(!inspection.inspection_summary.integrity_verified);
        assert_eq!(inspection.inspection_summary.issue_count, inspection.issue_count);
        assert_eq!(inspection.inspection_summary.warning_count, inspection.warning_count);
        assert_eq!(inspection.inspection_summary.checked_file_count, 4);
        assert_eq!(inspection.inspection_summary.integrity_file_count, inspection.integrity_counts.as_ref().unwrap().files.len());
        assert_eq!(
            inspection.inspection_summary.issue_counts_by_kind.iter().map(|item| (&item.kind, item.count)).collect::<Vec<_>>(),
            vec![
                (&AnswerArtifactExportBundleInspectionIssueKind::IntegrityDuplicatePath, 1),
                (&AnswerArtifactExportBundleInspectionIssueKind::IntegrityPathInvalid, 1),
                (&AnswerArtifactExportBundleInspectionIssueKind::IntegrityByteCountMismatch, 1),
                (&AnswerArtifactExportBundleInspectionIssueKind::IntegrityDigestMismatch, 1),
            ]
        );
    }

    #[test]
    fn answer_artifact_export_bundle_inspection_detects_missing_schema_version_deterministically() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, _version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let _ = service.build_final_answer(&source_id, &grounded_id).unwrap();
        let export_root = temp.path().join("missing-schema-bundle");
        service.export_answer_artifacts(&export_root).unwrap();
        remove_bundle_schema_version(&export_root, "export_manifest.json");
        remove_bundle_schema_version(&export_root, "export_issues.json");
        remove_bundle_schema_version(&export_root, "summary.json");
        let before = [
            fs::read_to_string(export_root.join("export_manifest.json")).unwrap(),
            fs::read_to_string(export_root.join("export_issues.json")).unwrap(),
            fs::read_to_string(export_root.join("summary.json")).unwrap(),
        ];

        let first = inspect_answer_artifact_export_bundle(&export_root).unwrap();
        let second = inspect_answer_artifact_export_bundle(&export_root).unwrap();
        let schema_errors = first
            .errors
            .iter()
            .filter(|issue| issue.kind == AnswerArtifactExportBundleInspectionIssueKind::SchemaVersionMissing)
            .map(|issue| (issue.kind.clone(), issue.relative_path.clone()))
            .collect::<Vec<_>>();
        let after = [
            fs::read_to_string(export_root.join("export_manifest.json")).unwrap(),
            fs::read_to_string(export_root.join("export_issues.json")).unwrap(),
            fs::read_to_string(export_root.join("summary.json")).unwrap(),
        ];

        assert!(!first.is_consistent);
        assert_eq!(first.schema_version, None);
        assert_eq!(first.manifest_schema_version, None);
        assert_eq!(first.issues_schema_version, None);
        assert_eq!(first.summary_schema_version, None);
        assert_eq!(first.inspection_summary.issue_count, first.errors.len());
        assert_eq!(first.inspection_summary.warning_count, 0);
        assert!(!first.inspection_summary.schema_supported);
        assert!(!first.inspection_summary.integrity_verified);
        assert_eq!(first.inspection_summary.checked_file_count, 4);
        assert!(first.inspection_summary.integrity_file_count > 0);
        assert_eq!(first.inspection_summary.issue_counts_by_kind, second.inspection_summary.issue_counts_by_kind);
        assert!(first.inspection_summary.issue_counts_by_kind.iter().any(|item| item.kind == AnswerArtifactExportBundleInspectionIssueKind::SchemaVersionMissing && item.count == 3));
        assert_eq!(first.inspection_summary.issue_counts_by_kind.iter().map(|item| item.count).sum::<usize>(), first.inspection_summary.issue_count);
        assert_eq!(schema_errors.len(), 3);
        assert_eq!(
            schema_errors,
            vec![
                (
                    AnswerArtifactExportBundleInspectionIssueKind::SchemaVersionMissing,
                    Some("export_issues.json".to_string())
                ),
                (
                    AnswerArtifactExportBundleInspectionIssueKind::SchemaVersionMissing,
                    Some("export_manifest.json".to_string())
                ),
                (
                    AnswerArtifactExportBundleInspectionIssueKind::SchemaVersionMissing,
                    Some("summary.json".to_string())
                ),
            ]
        );
        assert_eq!(first.errors, second.errors);
        assert_eq!(before, after);
        assert!(!format!("{first:?}").contains(temp.path().to_string_lossy().as_ref()));
    }

    #[test]
    fn answer_artifact_export_bundle_inspection_detects_unsupported_schema_version() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, _version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let _ = service.build_final_answer(&source_id, &grounded_id).unwrap();
        let export_root = temp.path().join("unsupported-schema-bundle");
        service.export_answer_artifacts(&export_root).unwrap();
        set_bundle_schema_version(&export_root, "export_manifest.json", "answer_artifact_export.v999");
        set_bundle_schema_version(&export_root, "export_issues.json", "answer_artifact_export.v999");
        set_bundle_schema_version(&export_root, "summary.json", "answer_artifact_export.v999");

        let inspection = inspect_answer_artifact_export_bundle(&export_root).unwrap();
        let inspection_again = inspect_answer_artifact_export_bundle(&export_root).unwrap();
        let unsupported = inspection
            .errors
            .iter()
            .filter(|issue| issue.kind == AnswerArtifactExportBundleInspectionIssueKind::SchemaVersionUnsupported)
            .collect::<Vec<_>>();

        assert!(!inspection.is_consistent);
        assert_eq!(inspection.schema_version, None);
        assert_eq!(unsupported.len(), 3);
        assert!(unsupported.iter().all(|issue| matches!(
            issue.relative_path.as_deref(),
            Some("export_manifest.json" | "export_issues.json" | "summary.json")
        )));
        assert!(inspection.errors.iter().any(|issue| issue.kind == AnswerArtifactExportBundleInspectionIssueKind::SchemaVersionMismatch));
        assert_eq!(inspection.inspection_summary.issue_count, inspection.issue_count);
        assert_eq!(inspection.inspection_summary.warning_count, inspection.warning_count);
        assert!(!inspection.inspection_summary.schema_supported);
        assert!(!inspection.inspection_summary.integrity_verified);
        assert_eq!(inspection.inspection_summary.checked_file_count, 4);
        assert!(inspection.inspection_summary.integrity_file_count > 0);
        assert_eq!(inspection.inspection_summary, inspection_again.inspection_summary);
        assert_eq!(inspection.inspection_summary.issue_counts_by_kind, inspection_again.inspection_summary.issue_counts_by_kind);
        assert!(inspection.inspection_summary.issue_counts_by_kind.iter().any(|item| item.kind == AnswerArtifactExportBundleInspectionIssueKind::SchemaVersionUnsupported && item.count == 3));
        assert!(inspection.inspection_summary.issue_counts_by_kind.iter().any(|item| item.kind == AnswerArtifactExportBundleInspectionIssueKind::SchemaVersionMismatch && item.count == 1));
        assert_eq!(inspection.inspection_summary.issue_counts_by_kind.iter().map(|item| item.count).sum::<usize>(), inspection.inspection_summary.issue_count);
        assert!(!format!("{inspection:?}").contains(temp.path().to_string_lossy().as_ref()));
    }

    #[test]
    fn answer_artifact_export_bundle_inspection_reports_malformed_versioned_issues_object() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, _version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let _ = service.build_final_answer(&source_id, &grounded_id).unwrap();
        let export_root = temp.path().join("malformed-issues-bundle");
        service.export_answer_artifacts(&export_root).unwrap();
        fs::write(
            export_root.join("export_issues.json"),
            r#"{"schema_version":"answer_artifact_export.v1","issues":"not-an-array"}"#,
        )
        .unwrap();

        let inspection = inspect_answer_artifact_export_bundle(&export_root).unwrap();

        assert!(!inspection.is_consistent);
        assert!(inspection.errors.iter().any(|issue| issue.kind == AnswerArtifactExportBundleInspectionIssueKind::IssuesReadFailed));
        assert!(!inspection.errors.iter().any(|issue| issue.kind == AnswerArtifactExportBundleInspectionIssueKind::SchemaVersionMissing));
        assert!(!inspection.errors.iter().any(|issue| issue.kind == AnswerArtifactExportBundleInspectionIssueKind::SchemaVersionUnsupported));
        assert!(!format!("{inspection:?}").contains(temp.path().to_string_lossy().as_ref()));
    }

    #[test]
    fn answer_artifact_export_bundle_inspection_detects_mismatched_schema_versions() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, _version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let _ = service.build_final_answer(&source_id, &grounded_id).unwrap();
        let export_root = temp.path().join("mismatched-schema-bundle");
        service.export_answer_artifacts(&export_root).unwrap();
        set_bundle_schema_version(&export_root, "summary.json", "answer_artifact_export.v2");

        let inspection = inspect_answer_artifact_export_bundle(&export_root).unwrap();

        assert!(!inspection.is_consistent);
        assert_eq!(inspection.schema_version, None);
        assert_eq!(inspection.manifest_schema_version.as_deref(), Some(ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION));
        assert_eq!(inspection.issues_schema_version.as_deref(), Some(ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION));
        assert_eq!(inspection.summary_schema_version.as_deref(), Some("answer_artifact_export.v2"));
        assert!(inspection.errors.iter().any(|issue| issue.kind == AnswerArtifactExportBundleInspectionIssueKind::SchemaVersionUnsupported));
        assert!(inspection.errors.iter().any(|issue| issue.kind == AnswerArtifactExportBundleInspectionIssueKind::SchemaVersionMismatch));
        assert!(!format!("{inspection:?}").contains(temp.path().to_string_lossy().as_ref()));
    }

    #[test]
    fn answer_artifact_export_bundle_inspection_reports_legacy_issue_array_as_missing_schema_version() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, _version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let _ = service.build_final_answer(&source_id, &grounded_id).unwrap();
        let export_root = temp.path().join("legacy-issues-bundle");
        service.export_answer_artifacts(&export_root).unwrap();
        let legacy_issues = read_export_issues(&export_root).issues;
        fs::write(
            export_root.join("export_issues.json"),
            serde_json::to_string_pretty(&legacy_issues).unwrap(),
        )
        .unwrap();

        let inspection = inspect_answer_artifact_export_bundle(&export_root).unwrap();

        assert!(inspection.has_issues);
        assert_eq!(inspection.issues_schema_version, None);
        assert!(inspection.errors.iter().any(|issue| {
            issue.kind == AnswerArtifactExportBundleInspectionIssueKind::SchemaVersionMissing
                && issue.relative_path.as_deref() == Some("export_issues.json")
        }));
        assert!(!inspection.errors.iter().any(|issue| issue.kind == AnswerArtifactExportBundleInspectionIssueKind::IssuesReadFailed));
        assert!(!format!("{inspection:?}").contains(temp.path().to_string_lossy().as_ref()));
    }

    #[test]
    fn answer_artifact_export_bundle_inspection_report_preview_is_deterministic_and_path_free() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, _version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let _ = service.build_final_answer(&source_id, &grounded_id).unwrap();
        let export_root = temp.path().join("preview-bundle");
        service.export_answer_artifacts(&export_root).unwrap();

        let before = fs::read_dir(&export_root).unwrap().count();
        let first = inspect_answer_artifact_export_bundle(&export_root).unwrap();
        let second = inspect_answer_artifact_export_bundle(&export_root).unwrap();
        let after = fs::read_dir(&export_root).unwrap().count();

        assert_eq!(before, after);
        assert_eq!(first.report_preview, second.report_preview);
        assert_eq!(first.report_preview.title, "Export bundle inspection report preview");
        assert_eq!(first.report_preview.schema_version, ANSWER_ARTIFACT_EXPORT_SCHEMA_VERSION);
        assert!(first.report_preview.is_consistent);
        assert!(first.report_preview.integrity_verified);
        assert_eq!(first.report_preview.issue_count, 0);
        assert_eq!(first.report_preview.warning_count, 0);
        assert!(first.report_preview.issue_counts_by_kind.is_empty());
        assert_eq!(
            first.report_preview.sections.iter().map(|section| section.heading.as_str()).collect::<Vec<_>>(),
            vec!["Status", "Issue counts by kind"]
        );
        assert!(first.report_preview.sections[1].lines.iter().any(|line| line == "No issue kinds reported."));
        assert!(!format!("{first:?}").contains(temp.path().to_string_lossy().as_ref()));
        assert!(!format!("{:?}", &first.report_preview).contains(temp.path().to_string_lossy().as_ref()));
        assert!(!format!("{:?}", &second.report_preview).contains(temp.path().to_string_lossy().as_ref()));
    }

    #[test]
    fn answer_artifact_export_bundle_inspection_report_preview_summarizes_invalid_bundle_deterministically() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, _version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let _ = service.build_final_answer(&source_id, &grounded_id).unwrap();
        let export_root = temp.path().join("invalid-preview-bundle");
        service.export_answer_artifacts(&export_root).unwrap();
        set_bundle_schema_version(&export_root, "export_manifest.json", "answer_artifact_export.v999");
        set_bundle_schema_version(&export_root, "export_issues.json", "answer_artifact_export.v999");
        set_bundle_schema_version(&export_root, "summary.json", "answer_artifact_export.v999");

        let first = inspect_answer_artifact_export_bundle(&export_root).unwrap();
        let second = inspect_answer_artifact_export_bundle(&export_root).unwrap();

        assert_eq!(first.report_preview, second.report_preview);
        assert!(!first.report_preview.is_consistent);
        assert!(!first.report_preview.integrity_verified);
        assert!(first.report_preview.issue_count > 0);
        assert_eq!(first.report_preview.warning_count, second.report_preview.warning_count);
        assert!(first.report_preview.issue_counts_by_kind.iter().any(|item| item.kind == AnswerArtifactExportBundleInspectionIssueKind::SchemaVersionUnsupported && item.count == 3));
        assert!(first.report_preview.issue_counts_by_kind.iter().any(|item| item.kind == AnswerArtifactExportBundleInspectionIssueKind::SchemaVersionMismatch && item.count == 1));
        assert_eq!(
            first.report_preview.sections.iter().map(|section| section.heading.as_str()).collect::<Vec<_>>(),
            vec!["Status", "Issue counts by kind", "Issues"]
        );
        assert!(first.report_preview.sections[2].lines.iter().any(|line| line.contains("schema_version_unsupported")));
        assert!(first.report_preview.sections[2].lines.iter().any(|line| line.contains("schema_version_mismatch")));
        assert!(!format!("{first:?}").contains(temp.path().to_string_lossy().as_ref()));
        assert!(!format!("{:?}", &first.report_preview).contains(temp.path().to_string_lossy().as_ref()));
    }

    #[test]
    fn answer_artifact_export_bundle_inspection_reports_missing_files_for_empty_directory() {
        let temp = tempfile::tempdir().unwrap();
        let export_root = temp.path().join("empty-export-bundle");
        fs::create_dir(&export_root).unwrap();

        let inspection = inspect_answer_artifact_export_bundle(&export_root).unwrap();

        assert!(!inspection.has_manifest);
        assert!(!inspection.has_issues);
        assert!(!inspection.has_summary);
        assert!(!inspection.has_integrity);
        assert!(!inspection.is_consistent);
        assert_eq!(inspection.issue_count, 4);
        assert_eq!(inspection.warning_count, 0);
        assert!(!inspection.inspection_summary.is_consistent);
        assert!(!inspection.inspection_summary.schema_supported);
        assert!(!inspection.inspection_summary.integrity_verified);
        assert_eq!(inspection.inspection_summary.issue_count, 4);
        assert_eq!(inspection.inspection_summary.warning_count, 0);
        assert_eq!(inspection.inspection_summary.checked_file_count, 0);
        assert_eq!(inspection.inspection_summary.integrity_file_count, 0);
        assert_eq!(
            inspection.inspection_summary.issue_counts_by_kind.iter().map(|item| (&item.kind, item.count)).collect::<Vec<_>>(),
            vec![
                (&AnswerArtifactExportBundleInspectionIssueKind::MissingManifest, 1),
                (&AnswerArtifactExportBundleInspectionIssueKind::MissingIssues, 1),
                (&AnswerArtifactExportBundleInspectionIssueKind::MissingSummary, 1),
                (&AnswerArtifactExportBundleInspectionIssueKind::MissingIntegrity, 1),
            ]
        );
        assert!(inspection.manifest_counts.is_none());
        assert!(inspection.summary_counts.is_none());
        assert!(inspection.integrity_counts.is_none());
        assert!(inspection.issue_kind_counts.is_none());
        assert!(inspection.errors.iter().any(|issue| issue.kind == AnswerArtifactExportBundleInspectionIssueKind::MissingManifest));
        assert!(inspection.errors.iter().any(|issue| issue.kind == AnswerArtifactExportBundleInspectionIssueKind::MissingIssues));
        assert!(inspection.errors.iter().any(|issue| issue.kind == AnswerArtifactExportBundleInspectionIssueKind::MissingSummary));
        assert!(inspection.errors.iter().any(|issue| issue.kind == AnswerArtifactExportBundleInspectionIssueKind::MissingIntegrity));
        assert_eq!(
            inspection.errors.iter().map(|issue| &issue.kind).collect::<Vec<_>>(),
            vec![
                &AnswerArtifactExportBundleInspectionIssueKind::MissingManifest,
                &AnswerArtifactExportBundleInspectionIssueKind::MissingIssues,
                &AnswerArtifactExportBundleInspectionIssueKind::MissingSummary,
                &AnswerArtifactExportBundleInspectionIssueKind::MissingIntegrity,
            ]
        );
        assert!(inspection.errors.iter().all(|issue| matches!(issue.relative_path.as_deref(), Some("export_manifest.json" | "export_issues.json" | "summary.json" | "export_integrity.json"))));
        assert!(fs::read_dir(&export_root).unwrap().next().is_none());
        assert!(!format!("{inspection:?}").contains(temp.path().to_string_lossy().as_ref()));
    }

    #[test]
    fn answer_artifact_export_bundle_inspection_ignores_unrelated_files_safely() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, _version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let _ = service.build_final_answer(&source_id, &grounded_id).unwrap();
        let export_root = temp.path().join("bundle-with-noise");
        service.export_answer_artifacts(&export_root).unwrap();

        fs::write(export_root.join("notes.txt"), "ignore me").unwrap();
        fs::create_dir_all(export_root.join("nested").join("subdir")).unwrap();
        fs::write(export_root.join("nested").join("subdir").join("ignore.json"), "{still-ignore}").unwrap();

        let inspection = inspect_answer_artifact_export_bundle(&export_root).unwrap();

        assert!(inspection.is_consistent);
        assert!(inspection.errors.is_empty());
        assert!(inspection.warnings.is_empty());
        assert!(inspection.has_manifest);
        assert!(inspection.has_issues);
        assert!(inspection.has_summary);
        assert!(!format!("{inspection:?}").contains(temp.path().to_string_lossy().as_ref()));
    }

    #[test]
    fn answer_artifact_export_bundle_inspection_rejects_empty_input_before_filesystem_access() {
        assert!(matches!(inspect_answer_artifact_export_bundle(""), Err(AegisError::ExportBundleInputMissing)));
    }

    #[test]
    fn answer_artifact_export_bundle_inspection_detects_malformed_files() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, _version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let _ = service.build_final_answer(&source_id, &grounded_id).unwrap();

        let manifest_root = temp.path().join("manifest-bundle");
        service.export_answer_artifacts(&manifest_root).unwrap();
        fs::write(manifest_root.join("export_manifest.json"), "{not-json").unwrap();
        let manifest_inspection = inspect_answer_artifact_export_bundle(&manifest_root).unwrap();
        assert!(!manifest_inspection.has_manifest);
        assert!(manifest_inspection.errors.iter().any(|issue| issue.kind == AnswerArtifactExportBundleInspectionIssueKind::ManifestReadFailed));
        assert!(!manifest_inspection.is_consistent);

        let issues_root = temp.path().join("issues-bundle");
        service.export_answer_artifacts(&issues_root).unwrap();
        fs::write(issues_root.join("export_issues.json"), "{not-json").unwrap();
        let issues_inspection = inspect_answer_artifact_export_bundle(&issues_root).unwrap();
        assert!(!issues_inspection.has_issues);
        assert!(issues_inspection.errors.iter().any(|issue| issue.kind == AnswerArtifactExportBundleInspectionIssueKind::IssuesReadFailed));
        assert!(!issues_inspection.is_consistent);

        let summary_root = temp.path().join("summary-bundle");
        service.export_answer_artifacts(&summary_root).unwrap();
        fs::write(summary_root.join("summary.json"), "{not-json").unwrap();
        let summary_inspection = inspect_answer_artifact_export_bundle(&summary_root).unwrap();
        assert!(!summary_inspection.has_summary);
        assert!(summary_inspection.errors.iter().any(|issue| issue.kind == AnswerArtifactExportBundleInspectionIssueKind::SummaryReadFailed));
        assert!(!summary_inspection.is_consistent);
    }

    #[test]
    fn answer_artifact_export_bundle_inspection_detects_summary_mismatches() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, _version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let _ = service.build_final_answer(&source_id, &grounded_id).unwrap();
        let export_root = temp.path().join("mismatch-bundle");
        service.export_answer_artifacts(&export_root).unwrap();

        let mut summary = read_export_summary(&export_root);
        summary.source_count += 1;
        summary.draft_count += 1;
        summary.grounded_answer_count += 1;
        summary.final_answer_count += 1;
        summary.issue_count += 1;
        summary.issue_kinds = vec![AnswerArtifactExportIssueKindCount { issue_kind: AnswerArtifactIssueKind::UnsupportedStatement, count: 99 }];
        summary.export_id = "sha256:deadbeef".to_string();
        summary.generated_from = "unexpected".to_string();
        summary.export_scope = "unexpected".to_string();
        summary.non_goals.push("unexpected_goal".to_string());
        summary.sources[0].draft_count += 1;
        fs::write(export_root.join("summary.json"), serde_json::to_string_pretty(&summary).unwrap()).unwrap();

        let inspection = inspect_answer_artifact_export_bundle(&export_root).unwrap();
        assert!(!inspection.is_consistent);
        assert!(inspection.errors.iter().any(|issue| issue.kind == AnswerArtifactExportBundleInspectionIssueKind::SummaryCountsMismatch));
        assert!(inspection.errors.iter().any(|issue| issue.kind == AnswerArtifactExportBundleInspectionIssueKind::SummaryIssueCountMismatch));
        assert!(inspection.errors.iter().any(|issue| issue.kind == AnswerArtifactExportBundleInspectionIssueKind::SummaryIssueKindCountsMismatch));
        assert!(inspection.errors.iter().any(|issue| issue.kind == AnswerArtifactExportBundleInspectionIssueKind::SummaryExportIdMismatch));
        assert!(inspection.errors.iter().any(|issue| issue.kind == AnswerArtifactExportBundleInspectionIssueKind::SummaryMetadataMismatch));
        assert!(inspection.manifest_counts.is_some());
        assert!(inspection.summary_counts.is_some());
        assert!(inspection.issue_kind_counts.as_ref().unwrap().is_empty());
        assert!(!format!("{inspection:?}").contains(temp.path().to_string_lossy().as_ref()));
    }

    #[test]
    fn answer_artifact_export_excludes_malformed_final_answers_from_valid_metadata() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let valid_final = service.build_final_answer(&source_id, &grounded_id).unwrap();
        let final_dir = CorpusPaths::new(temp.path().to_path_buf())
            .source_version_dir(&source_id, &version_id)
            .join("final_answers");
        fs::write(final_dir.join("fan_bad.json"), "{not-json").unwrap();
        let mut unsupported = valid_final.clone();
        unsupported.final_answer_id = "fan_unsupported".to_string();
        unsupported.statements[0].status = FinalAnswerStatementStatus::Unsupported;
        unsupported.statement_count = 1;
        unsupported.unsupported_count = 1;
        fs::write(final_dir.join("fan_unsupported.json"), serde_json::to_string_pretty(&unsupported).unwrap()).unwrap();
        let mut needs = valid_final.clone();
        needs.final_answer_id = "fan_needs".to_string();
        needs.statements[0].status = FinalAnswerStatementStatus::NeedsEvidence;
        needs.statement_count = 1;
        needs.unsupported_count = 1;
        fs::write(final_dir.join("fan_needs.json"), serde_json::to_string_pretty(&needs).unwrap()).unwrap();
        let export_root = temp.path().join("export-destination");

        let result = service.export_answer_artifacts(&export_root).unwrap();
        let final_exports = result
            .written_files
            .iter()
            .filter(|item| item.artifact_kind == ExportedArtifactKind::FinalAnswer)
            .collect::<Vec<_>>();
        assert_eq!(result.manifest.issue_count, service.list_answer_artifact_issues().unwrap().len());
        assert_eq!(result.manifest.sources.len(), 1);
        assert_eq!(result.manifest.sources[0].issue_count, service.list_answer_artifact_issues().unwrap().len());
        assert_eq!(result.manifest.sources[0].final_answers.len(), 3);
        assert!(result.manifest.sources[0].final_answers.iter().all(|item| item.final_answer_id != "fan_bad"));
        assert_eq!(final_exports.len(), 3);
        assert!(final_exports.iter().all(|item| item.relative_path.contains("final_answers")));
    }

    #[test]
    fn answer_artifact_issues_are_deterministic_across_multiple_sources() {
        let temp = tempfile::tempdir().unwrap();
        let (source_a, version_a, _draft_a, grounded_a) = prepare_grounded(&temp.path().to_path_buf());
        let source_path_b = temp.path().join("notes_b.md");
        fs::write(&source_path_b, "delta epsilon zeta").unwrap();
        let authority = CorpusAuthority::new(temp.path().to_path_buf());
        authority.register_source(source_path_b.to_string_lossy().to_string(), valid_metadata()).unwrap();
        let registry = SourceRegistry::load(&CorpusPaths::new(temp.path().to_path_buf()).registry_path()).unwrap();
        let source_b = registry.sources.iter().find(|record| record.source_id != source_a).cloned().unwrap();

        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let valid_final_a = service.build_final_answer(&source_a, &grounded_a).unwrap();
        let dir_a = CorpusPaths::new(temp.path().to_path_buf())
            .source_version_dir(&source_a, &version_a)
            .join("final_answers");
        let mut unsupported_a = valid_final_a.clone();
        unsupported_a.final_answer_id = "fan_a".to_string();
        unsupported_a.statements[0].status = FinalAnswerStatementStatus::Unsupported;
        unsupported_a.statement_count = 1;
        unsupported_a.unsupported_count = 1;
        fs::write(dir_a.join("fan_a.json"), serde_json::to_string_pretty(&unsupported_a).unwrap()).unwrap();

        let source_b_dir = CorpusPaths::new(temp.path().to_path_buf()).source_version_dir(&source_b.source_id, &source_b.version_id);
        fs::create_dir_all(source_b_dir.join("final_answers")).unwrap();
        fs::write(source_b_dir.join("final_answers").join("fan_b_bad.json"), "{not-json").unwrap();

        let issues = service.list_answer_artifact_issues().unwrap();
        assert_eq!(issues.len(), 2);
        assert!(issues.windows(2).all(|pair| {
            (
                &pair[0].source_id,
                issue_kind_rank(&pair[0].issue_kind),
                pair[0].final_answer_id.as_deref().unwrap_or(""),
                pair[0].statement_index.unwrap_or(usize::MAX),
            ) <= (
                &pair[1].source_id,
                issue_kind_rank(&pair[1].issue_kind),
                pair[1].final_answer_id.as_deref().unwrap_or(""),
                pair[1].statement_index.unwrap_or(usize::MAX),
            )
        }));
        assert!(issues.iter().all(|item| !item.message.contains(":\\") && !item.message.contains("/")));
    }

    #[test]
    fn answer_artifact_source_index_ignores_unrelated_files_and_reports_typed_malformed_final_answer() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, version_id, _draft_id, grounded_id) = prepare_grounded(&temp.path().to_path_buf());
        let service = FinalAnswerService::new(temp.path().to_path_buf());
        let _ = service.build_final_answer(&source_id, &grounded_id).unwrap();
        let final_dir = CorpusPaths::new(temp.path().to_path_buf())
            .source_version_dir(&source_id, &version_id)
            .join("final_answers");
        fs::write(final_dir.join("notes.txt"), "ignore me").unwrap();
        let bad_path = final_dir.join("fan_bad.json");
        fs::write(&bad_path, "{not-json").unwrap();
        assert!(matches!(service.list_answer_artifact_sources(), Err(AegisError::FinalAnswerReadFailed)));
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
