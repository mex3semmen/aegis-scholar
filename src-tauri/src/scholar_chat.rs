use crate::errors::{AegisError, AegisResult};
use crate::locators::CitationLocator;
use crate::retrieval::{RetrievalResponse, RetrievalService};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const SCHOLAR_CHAT_RETRIEVAL_PREVIEW_LIMIT: usize = 5;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatMode {
    LectureLearning,
    ThesisWriting,
    LiteratureReview,
    Flashcards,
    StatisticsMethods,
    GeneralScholar,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GroundingPolicy {
    LocalOnly,
    LocalFirst,
    AllowMarkedModelKnowledge,
    ExternalAdaptersLater,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatStatus {
    PreviewOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatRequest {
    pub prompt: String,
    pub mode: ScholarChatMode,
    pub grounding_policy: GroundingPolicy,
    pub selected_source_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatGroundingPlan {
    pub selected_source_count: usize,
    pub local_corpus_required: bool,
    pub retrieval_would_run: bool,
    pub evidence_pack_would_be_required: bool,
    pub model_knowledge_allowed: bool,
    pub external_adapters_available: bool,
    pub summary: String,
    pub steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatResponse {
    pub status: ScholarChatStatus,
    pub normalized_prompt: String,
    pub mode: ScholarChatMode,
    pub grounding_policy: GroundingPolicy,
    pub selected_source_ids: Vec<String>,
    pub selected_source_count: usize,
    pub grounding_plan: ScholarChatGroundingPlan,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScholarChatRetrievalCandidate {
    pub source_id: String,
    pub version_id: String,
    pub chunk_id: String,
    pub score: f32,
    pub matched_terms: Vec<String>,
    pub preview: String,
    pub locator: CitationLocator,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScholarChatRetrievalPreviewResponse {
    pub status: ScholarChatStatus,
    pub normalized_prompt: String,
    pub mode: ScholarChatMode,
    pub grounding_policy: GroundingPolicy,
    pub selected_source_ids: Vec<String>,
    pub selected_source_count: usize,
    pub candidate_count: usize,
    pub candidates: Vec<ScholarChatRetrievalCandidate>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatEvidencePlanStatus {
    EvidencePlanPreview,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScholarChatEvidenceCandidate {
    pub source_id: String,
    pub version_id: String,
    pub chunk_id: String,
    pub score: f32,
    pub matched_terms: Vec<String>,
    pub preview: String,
    pub locator: CitationLocator,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatEvidencePlan {
    pub retrieval_candidate_count: usize,
    pub evidence_candidate_count: usize,
    pub evidence_required: bool,
    pub evidence_pack_would_be_built_later: bool,
    pub summary: String,
    pub steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScholarChatEvidencePlanResponse {
    pub status: ScholarChatEvidencePlanStatus,
    pub normalized_prompt: String,
    pub mode: ScholarChatMode,
    pub grounding_policy: GroundingPolicy,
    pub selected_source_ids: Vec<String>,
    pub selected_source_count: usize,
    pub retrieval_candidate_count: usize,
    pub evidence_candidate_count: usize,
    pub evidence_plan: ScholarChatEvidencePlan,
    pub candidates: Vec<ScholarChatEvidenceCandidate>,
    pub warnings: Vec<String>,
}

enum ScholarChatPreviewKind {
    Request,
    Retrieval,
}

pub fn preview_scholar_chat_request(
    _root: impl Into<PathBuf>,
    request: ScholarChatRequest,
) -> AegisResult<ScholarChatResponse> {
    let normalized_prompt = normalized_prompt_or_err(request.prompt)?;
    let (selected_source_ids, selected_source_count) = normalize_selected_source_ids(request.selected_source_ids)?;
    let warnings = preview_warnings(&request.grounding_policy, selected_source_count, ScholarChatPreviewKind::Request);
    let grounding_plan = grounding_plan(&request.mode, &request.grounding_policy, selected_source_count);

    Ok(ScholarChatResponse {
        status: ScholarChatStatus::PreviewOnly,
        normalized_prompt,
        mode: request.mode,
        grounding_policy: request.grounding_policy,
        selected_source_ids,
        selected_source_count,
        grounding_plan,
        warnings,
    })
}

pub fn preview_scholar_chat_retrieval(
    root: impl Into<PathBuf>,
    request: ScholarChatRequest,
) -> AegisResult<ScholarChatRetrievalPreviewResponse> {
    let normalized_prompt = normalized_prompt_or_err(request.prompt)?;
    let (selected_source_ids, selected_source_count) = normalize_selected_source_ids(request.selected_source_ids)?;
    let mut warnings = preview_warnings(&request.grounding_policy, selected_source_count, ScholarChatPreviewKind::Retrieval);

    let retrieval_service = RetrievalService::new(root);
    let mut candidates = Vec::new();

    if selected_source_count > 0 {
        for source_id in &selected_source_ids {
            match retrieval_service.preview_search_source(source_id, &normalized_prompt, SCHOLAR_CHAT_RETRIEVAL_PREVIEW_LIMIT) {
                Ok(response) => {
                    if response.results.is_empty() {
                        warnings.push(format!("No retrieval candidates matched selected source {source_id}."));
                    }
                    candidates.extend(convert_retrieval_response(response));
                }
                Err(AegisError::RetrievalIndexMissing)
                | Err(AegisError::RetrievalIndexReadFailed)
                | Err(AegisError::ChunkingReportMissing)
                | Err(AegisError::ChunkingReportReadFailed) => {
                    warnings.push(format!("Retrieval data is not ready for selected source {source_id}; skipping this source."));
                }
                Err(error) => return Err(error),
            }
        }
    }

    candidates.sort_by(|left, right| {
        right
            .score
            .partial_cmp(&left.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.source_id.cmp(&right.source_id))
            .then_with(|| left.chunk_id.cmp(&right.chunk_id))
    });

    if selected_source_count > 0 && candidates.is_empty() && !warnings.iter().any(|warning| warning.contains("Retrieval data is not ready")) {
        warnings.push("No retrieval candidates matched the selected sources.".to_string());
    }

    Ok(ScholarChatRetrievalPreviewResponse {
        status: ScholarChatStatus::PreviewOnly,
        normalized_prompt,
        mode: request.mode,
        grounding_policy: request.grounding_policy,
        selected_source_ids,
        selected_source_count,
        candidate_count: candidates.len(),
        candidates,
        warnings,
    })
}

pub fn preview_scholar_chat_evidence_plan(
    root: impl Into<PathBuf>,
    request: ScholarChatRequest,
) -> AegisResult<ScholarChatEvidencePlanResponse> {
    let retrieval_preview = preview_scholar_chat_retrieval(root, request)?;
    let mut warnings = retrieval_preview.warnings.clone();
    if retrieval_preview.selected_source_count == 0 && !warnings.iter().any(|warning| warning.contains("No selected sources")) {
        warnings.push("No selected sources; evidence plan preview is unscoped.".to_string());
    }
    if retrieval_preview.candidate_count == 0 {
        warnings.push("No retrieval candidates were eligible for Evidence Pack assembly yet.".to_string());
    }
    warnings.push("This is a preview only; no Evidence Pack was built.".to_string());

    let evidence_candidates = convert_retrieval_candidates_from_preview(&retrieval_preview.candidates);
    let evidence_candidate_count = evidence_candidates.len();
    let evidence_plan = evidence_plan(
        &retrieval_preview.mode,
        &retrieval_preview.grounding_policy,
        retrieval_preview.selected_source_count,
        retrieval_preview.candidate_count,
        evidence_candidate_count,
    );

    Ok(ScholarChatEvidencePlanResponse {
        status: ScholarChatEvidencePlanStatus::EvidencePlanPreview,
        normalized_prompt: retrieval_preview.normalized_prompt,
        mode: retrieval_preview.mode,
        grounding_policy: retrieval_preview.grounding_policy,
        selected_source_ids: retrieval_preview.selected_source_ids,
        selected_source_count: retrieval_preview.selected_source_count,
        retrieval_candidate_count: retrieval_preview.candidate_count,
        evidence_candidate_count,
        evidence_plan,
        candidates: evidence_candidates,
        warnings,
    })
}

fn normalized_prompt_or_err(prompt: String) -> AegisResult<String> {
    let normalized_prompt = prompt.trim().to_string();
    if normalized_prompt.is_empty() {
        return Err(AegisError::ScholarChatPromptEmpty);
    }
    Ok(normalized_prompt)
}

fn normalize_selected_source_ids(source_ids: Vec<String>) -> AegisResult<(Vec<String>, usize)> {
    let mut selected_source_ids = Vec::new();
    for source_id in source_ids {
        let normalized_source_id = source_id.trim().to_string();
        validate_source_id(&normalized_source_id)?;
        selected_source_ids.push(normalized_source_id);
    }
    let selected_source_count = selected_source_ids.len();
    Ok((selected_source_ids, selected_source_count))
}

fn preview_warnings(policy: &GroundingPolicy, selected_source_count: usize, kind: ScholarChatPreviewKind) -> Vec<String> {
    let mut warnings = Vec::new();
    if selected_source_count == 0 {
        match kind {
            ScholarChatPreviewKind::Request => {
                warnings.push("No selected sources; preview cannot plan source-scoped grounding yet.".to_string());
            }
            ScholarChatPreviewKind::Retrieval => {
                warnings.push("No selected sources; retrieval preview is unscoped.".to_string());
            }
        }
    }
    match policy {
        GroundingPolicy::LocalOnly => warnings.push("local_only requires local evidence before an answer can be presented as grounded.".to_string()),
        GroundingPolicy::ExternalAdaptersLater => warnings.push("External scholarly adapters are not implemented in this preview.".to_string()),
        GroundingPolicy::AllowMarkedModelKnowledge => warnings.push("Model knowledge is not used in this preview and would need to be clearly marked later.".to_string()),
        GroundingPolicy::LocalFirst => {}
    }
    match kind {
        ScholarChatPreviewKind::Request => warnings.push("This is a contract preview only; no retrieval, evidence-pack build, or model inference was run.".to_string()),
        ScholarChatPreviewKind::Retrieval => warnings.push("This is a retrieval preview only; no answer was generated.".to_string()),
    }
    warnings
}

fn grounding_plan(mode: &ScholarChatMode, policy: &GroundingPolicy, selected_source_count: usize) -> ScholarChatGroundingPlan {
    ScholarChatGroundingPlan {
        selected_source_count,
        local_corpus_required: matches!(policy, GroundingPolicy::LocalOnly | GroundingPolicy::LocalFirst),
        retrieval_would_run: selected_source_count > 0,
        evidence_pack_would_be_required: true,
        model_knowledge_allowed: matches!(policy, GroundingPolicy::AllowMarkedModelKnowledge),
        external_adapters_available: false,
        summary: grounding_summary(mode, policy, selected_source_count),
        steps: vec![
            "Normalize prompt and validate selected source IDs.".to_string(),
            "Resolve selected course or project context before retrieval.".to_string(),
            "Search registered local sources before any answer synthesis.".to_string(),
            "Assemble an Evidence Pack before grounded answer generation.".to_string(),
            "Return source/evidence status with any future answer.".to_string(),
        ],
    }
}

fn convert_retrieval_response(response: RetrievalResponse) -> Vec<ScholarChatRetrievalCandidate> {
    response
        .results
        .into_iter()
        .map(|result| ScholarChatRetrievalCandidate {
            source_id: result.source_id,
            version_id: result.version_id,
            chunk_id: result.chunk_id,
            score: result.score,
            matched_terms: result.matched_terms,
            preview: result.preview,
            locator: result.locator,
        })
        .collect()
}

fn convert_retrieval_candidates_from_preview(
    candidates: &[ScholarChatRetrievalCandidate],
) -> Vec<ScholarChatEvidenceCandidate> {
    candidates
        .iter()
        .map(|result| ScholarChatEvidenceCandidate {
            source_id: result.source_id.clone(),
            version_id: result.version_id.clone(),
            chunk_id: result.chunk_id.clone(),
            score: result.score,
            matched_terms: result.matched_terms.clone(),
            preview: result.preview.clone(),
            locator: result.locator.clone(),
        })
        .collect()
}

fn evidence_plan(
    mode: &ScholarChatMode,
    policy: &GroundingPolicy,
    selected_source_count: usize,
    retrieval_candidate_count: usize,
    evidence_candidate_count: usize,
) -> ScholarChatEvidencePlan {
    ScholarChatEvidencePlan {
        retrieval_candidate_count,
        evidence_candidate_count,
        evidence_required: true,
        evidence_pack_would_be_built_later: true,
        summary: evidence_plan_summary(mode, policy, selected_source_count, retrieval_candidate_count, evidence_candidate_count),
        steps: vec![
            "Normalize prompt and validate selected source IDs.".to_string(),
            "Reuse retrieval-preview candidates over the selected local sources.".to_string(),
            "Mark retrieval candidates that would be eligible for Evidence Pack assembly later.".to_string(),
            "Return preview-only evidence readiness warnings without building an Evidence Pack.".to_string(),
        ],
    }
}

fn evidence_plan_summary(
    mode: &ScholarChatMode,
    policy: &GroundingPolicy,
    selected_source_count: usize,
    retrieval_candidate_count: usize,
    evidence_candidate_count: usize,
) -> String {
    format!(
        "Preview plans a {:?} request with {:?} grounding over {} selected source(s), yielding {} retrieval candidate(s) and {} evidence candidate(s); no Evidence Pack is built yet.",
        mode,
        policy,
        selected_source_count,
        retrieval_candidate_count,
        evidence_candidate_count,
    )
}

fn grounding_summary(mode: &ScholarChatMode, policy: &GroundingPolicy, selected_source_count: usize) -> String {
    format!(
        "Preview plans a {:?} request with {:?} grounding over {} selected source(s); no answer is generated.",
        mode, policy, selected_source_count
    )
}

fn validate_source_id(source_id: &str) -> AegisResult<()> {
    if source_id.trim().is_empty() {
        return Err(AegisError::ScholarChatInvalidSourceId);
    }
    if source_id.contains('/') || source_id.contains('\\') || source_id.contains("..") {
        return Err(AegisError::ScholarChatInvalidSourceId);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn request(prompt: &str) -> ScholarChatRequest {
        ScholarChatRequest {
            prompt: prompt.to_string(),
            mode: ScholarChatMode::LectureLearning,
            grounding_policy: GroundingPolicy::LocalFirst,
            selected_source_ids: vec![" src_demo ".to_string()],
        }
    }

    #[test]
    fn scholar_chat_preview_rejects_empty_prompt() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_request(temp.path(), request("   "));
        assert!(matches!(result, Err(AegisError::ScholarChatPromptEmpty)));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn scholar_chat_preview_trims_prompt_and_sources_deterministically() {
        let temp = tempfile::tempdir().unwrap();
        let first = preview_scholar_chat_request(temp.path(), request("  Explain retrieval  ")).unwrap();
        let second = preview_scholar_chat_request(temp.path(), request("  Explain retrieval  ")).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.normalized_prompt, "Explain retrieval");
        assert_eq!(first.selected_source_ids, vec!["src_demo"]);
        assert_eq!(first.status, ScholarChatStatus::PreviewOnly);
    }

    #[test]
    fn scholar_chat_preview_rejects_invalid_source_ids_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["", " ", "..", "../evil", "evil/source", "evil\\source"] {
            let mut request = request("Explain alpha");
            request.selected_source_ids = vec![invalid.to_string()];
            let result = preview_scholar_chat_request(temp.path(), request);
            assert!(matches!(result, Err(AegisError::ScholarChatInvalidSourceId)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn scholar_chat_preview_allows_no_selected_sources_with_warning() {
        let temp = tempfile::tempdir().unwrap();
        let mut request = request("Explain alpha");
        request.selected_source_ids = Vec::new();
        let response = preview_scholar_chat_request(temp.path(), request).unwrap();
        assert_eq!(response.selected_source_count, 0);
        assert!(response.warnings.iter().any(|warning| warning.contains("No selected sources")));
        assert!(!response.grounding_plan.retrieval_would_run);
    }

    #[test]
    fn scholar_chat_preview_local_only_policy_is_visible_in_warnings_and_plan() {
        let temp = tempfile::tempdir().unwrap();
        let mut request = request("Explain alpha");
        request.grounding_policy = GroundingPolicy::LocalOnly;
        let response = preview_scholar_chat_request(temp.path(), request).unwrap();
        assert!(response.grounding_plan.local_corpus_required);
        assert!(response.warnings.iter().any(|warning| warning.contains("local_only")));
    }

    #[test]
    fn scholar_chat_preview_output_is_path_free_and_non_mutating() {
        let temp = tempfile::tempdir().unwrap();
        let response = preview_scholar_chat_request(temp.path(), request("Explain alpha")).unwrap();
        let debug = format!("{response:?}");
        let json = serde_json::to_string(&response).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert!(!temp.path().join(".aegis").exists());
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 0);
    }

    fn retrieval_request(prompt: &str, selected_source_ids: Vec<String>) -> ScholarChatRequest {
        ScholarChatRequest {
            prompt: prompt.to_string(),
            mode: ScholarChatMode::LectureLearning,
            grounding_policy: GroundingPolicy::LocalFirst,
            selected_source_ids,
        }
    }

    fn evidence_plan_request(prompt: &str, selected_source_ids: Vec<String>) -> ScholarChatRequest {
        retrieval_request(prompt, selected_source_ids)
    }

    fn build_source_with_index(temp: &tempfile::TempDir, text: &str) -> String {
        let source_path = temp.path().join("note.md");
        fs::write(&source_path, text).unwrap();
        let authority = crate::corpus_authority::CorpusAuthority::new(temp.path());
        let source = authority
            .register_source(
                &source_path,
                crate::source_metadata::SourceMetadataInput {
                    title: "Notes".to_string(),
                    source_type: crate::source_metadata::SourceType::MarkdownNote,
                    discipline: "psychology".to_string(),
                    subdiscipline: Some("statistics".to_string()),
                    language: "en".to_string(),
                    tags: vec!["study".to_string()],
                    reliability_notes: None,
                },
            )
            .unwrap();
        crate::extraction::ExtractionService::new(temp.path())
            .extract_source(&source.source_id)
            .unwrap();
        crate::chunking::ChunkingService::new(temp.path())
            .chunk_source(&source.source_id)
            .unwrap();
        RetrievalService::new(temp.path())
            .build_index(&source.source_id)
            .unwrap();
        source.source_id
    }

    #[test]
    fn scholar_chat_retrieval_preview_rejects_empty_prompt() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_retrieval(temp.path(), retrieval_request("   ", vec![]));
        assert!(matches!(result, Err(AegisError::ScholarChatPromptEmpty)));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn scholar_chat_retrieval_preview_rejects_invalid_source_ids_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["", " ", "..", "../evil", "evil/source", "evil\\source"] {
            let result = preview_scholar_chat_retrieval(temp.path(), retrieval_request("Explain alpha", vec![invalid.to_string()]));
            assert!(matches!(result, Err(AegisError::ScholarChatInvalidSourceId)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn scholar_chat_retrieval_preview_allows_empty_sources_with_warning() {
        let temp = tempfile::tempdir().unwrap();
        let response = preview_scholar_chat_retrieval(temp.path(), retrieval_request("Explain alpha", vec![])).unwrap();
        assert_eq!(response.selected_source_count, 0);
        assert_eq!(response.candidate_count, 0);
        assert!(response.warnings.iter().any(|warning| warning.contains("unscoped")));
        assert_eq!(response.status, ScholarChatStatus::PreviewOnly);
    }

    #[test]
    fn scholar_chat_retrieval_preview_does_not_build_missing_indexes() {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("note.md");
        fs::write(&source_path, "alpha beta\n").unwrap();
        let authority = crate::corpus_authority::CorpusAuthority::new(temp.path());
        let source = authority.register_source(&source_path, crate::source_metadata::SourceMetadataInput {
            title: "Notes".to_string(),
            source_type: crate::source_metadata::SourceType::MarkdownNote,
            discipline: "psychology".to_string(),
            subdiscipline: Some("statistics".to_string()),
            language: "en".to_string(),
            tags: vec!["study".to_string()],
            reliability_notes: None,
        }).unwrap();
        crate::extraction::ExtractionService::new(temp.path()).extract_source(&source.source_id).unwrap();
        crate::chunking::ChunkingService::new(temp.path()).chunk_source(&source.source_id).unwrap();
        let source_id = source.source_id.clone();
        let version_id = source.version_id.clone();

        let response = preview_scholar_chat_retrieval(
            temp.path(),
            retrieval_request("alpha", vec![source_id.clone()]),
        )
        .unwrap();

        assert_eq!(response.selected_source_ids, vec![source_id.clone()]);
        assert_eq!(response.candidate_count, 0);
        assert!(response.warnings.iter().any(|warning| warning.contains("not ready")));
        let index_path = temp
            .path()
            .join(".aegis")
            .join("corpus")
            .join("sources")
            .join(&source_id)
            .join("versions")
            .join(&version_id)
            .join("retrieval")
            .join("index.json");
        assert!(!index_path.exists());
    }

    #[test]
    fn scholar_chat_retrieval_preview_is_deterministic_and_path_free() {
        let temp = tempfile::tempdir().unwrap();
        let source_a = build_source_with_index(&temp, "alpha beta\n\nalpha gamma\n");
        let source_b = build_source_with_index(&temp, "alpha delta\n\nalpha epsilon\n");
        let request = retrieval_request("  alpha  ", vec![source_b.clone(), source_a.clone()]);
        let first = preview_scholar_chat_retrieval(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_retrieval(temp.path(), request).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.normalized_prompt, "alpha");
        assert_eq!(first.selected_source_ids, vec![source_b, source_a]);
        assert!(first.candidates.windows(2).all(|pair| pair[0].score >= pair[1].score));
        let debug = format!("{first:?}");
        let json = serde_json::to_string(&first).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert!(!temp.path().join(".aegis").join("corpus").join("sources").join("missing").exists());
    }

    #[test]
    fn scholar_chat_evidence_plan_rejects_empty_prompt() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_evidence_plan(temp.path(), evidence_plan_request("   ", vec![]));
        assert!(matches!(result, Err(AegisError::ScholarChatPromptEmpty)));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn scholar_chat_evidence_plan_rejects_invalid_source_ids_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["", " ", "..", "../evil", "evil/source", "evil\\source"] {
            let result = preview_scholar_chat_evidence_plan(temp.path(), evidence_plan_request("Explain alpha", vec![invalid.to_string()]));
            assert!(matches!(result, Err(AegisError::ScholarChatInvalidSourceId)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn scholar_chat_evidence_plan_allows_no_selected_sources_with_warning() {
        let temp = tempfile::tempdir().unwrap();
        let response = preview_scholar_chat_evidence_plan(temp.path(), evidence_plan_request("Explain alpha", vec![])).unwrap();
        assert_eq!(response.selected_source_count, 0);
        assert_eq!(response.retrieval_candidate_count, 0);
        assert_eq!(response.evidence_candidate_count, 0);
        assert!(response.warnings.iter().any(|warning| warning.contains("No selected sources")));
        assert!(response.warnings.iter().any(|warning| warning.contains("Evidence Pack was built")));
        assert_eq!(response.status, ScholarChatEvidencePlanStatus::EvidencePlanPreview);
        assert!(response.evidence_plan.evidence_required);
        assert!(response.evidence_plan.evidence_pack_would_be_built_later);
    }

    #[test]
    fn scholar_chat_evidence_plan_does_not_build_missing_indexes() {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("note.md");
        fs::write(&source_path, "alpha beta\n").unwrap();
        let authority = crate::corpus_authority::CorpusAuthority::new(temp.path());
        let source = authority.register_source(&source_path, crate::source_metadata::SourceMetadataInput {
            title: "Notes".to_string(),
            source_type: crate::source_metadata::SourceType::MarkdownNote,
            discipline: "psychology".to_string(),
            subdiscipline: Some("statistics".to_string()),
            language: "en".to_string(),
            tags: vec!["study".to_string()],
            reliability_notes: None,
        }).unwrap();
        crate::extraction::ExtractionService::new(temp.path()).extract_source(&source.source_id).unwrap();
        crate::chunking::ChunkingService::new(temp.path()).chunk_source(&source.source_id).unwrap();
        let source_id = source.source_id.clone();
        let version_id = source.version_id.clone();

        let response = preview_scholar_chat_evidence_plan(
            temp.path(),
            evidence_plan_request("alpha", vec![source_id.clone()]),
        )
        .unwrap();

        assert_eq!(response.selected_source_ids, vec![source_id.clone()]);
        assert_eq!(response.retrieval_candidate_count, 0);
        assert_eq!(response.evidence_candidate_count, 0);
        assert!(response.warnings.iter().any(|warning| warning.contains("not ready")));
        let index_path = temp
            .path()
            .join(".aegis")
            .join("corpus")
            .join("sources")
            .join(&source_id)
            .join("versions")
            .join(&version_id)
            .join("retrieval")
            .join("index.json");
        assert!(!index_path.exists());
    }

    #[test]
    fn scholar_chat_evidence_plan_is_deterministic_and_path_free() {
        let temp = tempfile::tempdir().unwrap();
        let source_a = build_source_with_index(&temp, "alpha beta\n\nalpha gamma\n");
        let source_b = build_source_with_index(&temp, "alpha delta\n\nalpha epsilon\n");
        let request = evidence_plan_request("  alpha  ", vec![source_b.clone(), source_a.clone()]);
        let first = preview_scholar_chat_evidence_plan(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_evidence_plan(temp.path(), request).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.normalized_prompt, "alpha");
        assert_eq!(first.selected_source_ids, vec![source_b, source_a]);
        assert_eq!(first.retrieval_candidate_count, first.candidates.len());
        assert_eq!(first.evidence_candidate_count, first.candidates.len());
        assert!(first.candidates.windows(2).all(|pair| pair[0].score >= pair[1].score));
        let debug = format!("{first:?}");
        let json = serde_json::to_string(&first).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert!(!temp.path().join(".aegis").join("corpus").join("sources").join("missing").exists());
    }
}
