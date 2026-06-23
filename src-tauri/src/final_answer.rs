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
pub struct AnswerArtifactExportResult {
    pub manifest: AnswerArtifactExportManifest,
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
#[serde(rename_all = "snake_case")]
pub enum ExportedArtifactKind {
    Manifest,
    Issues,
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
        fs::write(&issues_path, serde_json::to_string_pretty(&issues)?).map_err(|_| AegisError::FinalAnswerWriteFailed)?;
        written_files.push(ExportedArtifactFile {
            relative_path: "export_issues.json".to_string(),
            artifact_kind: ExportedArtifactKind::Issues,
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
            manifest,
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
                    ExportedArtifactKind::Manifest | ExportedArtifactKind::Issues => unreachable!(),
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
        assert!(export_root.join(&source_id).join("answer_drafts").join(format!("{draft_id}.json")).exists());
        assert!(export_root.join(&source_id).join("grounded_answers").join(format!("{grounded_id}.json")).exists());
        assert!(export_root.join(&source_id).join("final_answers").join(format!("{}.json", final_answer.final_answer_id)).exists());
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
