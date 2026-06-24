use crate::errors::{AegisError, AegisResult};
use crate::locators::CitationLocator;
use crate::retrieval::{RetrievalResponse, RetrievalService};
use crate::local_runtime::{
    preview_local_model_runtime_health,
    preview_local_runtime_invocation_plan,
    smoke_test_local_runtime_inference,
    LocalModelRuntimeConfig,
    LocalModelRuntimeHealthStatus,
    LocalRuntimeInvocationPlanRequest,
    LocalRuntimeInvocationPlanStatus,
    LocalRuntimeSmokeInferenceRequest,
    LocalRuntimeSmokeInferenceStatus,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatPromptPackStatus {
    PromptPackPreview,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatPromptPackSectionKind {
    SystemOrPolicyInstructions,
    ModeInstructions,
    GroundingInstructions,
    SourceContext,
    UserPrompt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatPromptPackSection {
    pub kind: ScholarChatPromptPackSectionKind,
    pub title: String,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScholarChatPromptContextItem {
    pub source_id: String,
    pub version_id: String,
    pub chunk_id: String,
    pub score: f32,
    pub matched_terms: Vec<String>,
    pub preview: String,
    pub locator: CitationLocator,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatPromptPack {
    pub section_count: usize,
    pub context_item_count: usize,
    pub estimated_input_char_count: usize,
    pub sections: Vec<ScholarChatPromptPackSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScholarChatPromptPackPreviewResponse {
    pub status: ScholarChatPromptPackStatus,
    pub normalized_prompt: String,
    pub mode: ScholarChatMode,
    pub grounding_policy: GroundingPolicy,
    pub selected_source_ids: Vec<String>,
    pub selected_source_count: usize,
    pub evidence_candidate_count: usize,
    pub prompt_pack: ScholarChatPromptPack,
    pub context_items: Vec<ScholarChatPromptContextItem>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatAnswerReadinessStatus {
    Blocked,
    NeedsSources,
    NeedsRetrievalIndex,
    NeedsEvidenceCandidates,
    NeedsRuntimeConfig,
    NeedsExecutionConsent,
    ReadyForDraftInferenceLater,
    ReadyForGroundedDraftLater,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatAnswerReadinessOutputClassification {
    Blocked,
    UngroundedDraft,
    SourceContextDraft,
    GroundedDraftCandidate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatAnswerReadinessBlocker {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatAnswerReadinessWarning {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScholarChatAnswerReadinessRequest {
    pub scholar_chat_request: ScholarChatRequest,
    pub runtime_config: LocalModelRuntimeConfig,
    pub allow_model_execution: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScholarChatAnswerReadinessPreview {
    pub status: ScholarChatAnswerReadinessStatus,
    pub normalized_prompt: String,
    pub mode: ScholarChatMode,
    pub grounding_policy: GroundingPolicy,
    pub selected_source_count: usize,
    pub retrieval_candidate_count: usize,
    pub evidence_candidate_count: usize,
    pub prompt_pack_ready: bool,
    pub runtime_health_status: LocalModelRuntimeHealthStatus,
    pub invocation_plan_status: LocalRuntimeInvocationPlanStatus,
    pub allow_model_execution: bool,
    pub would_generate_answer_now: bool,
    pub would_build_evidence_pack_now: bool,
    pub would_create_final_answer_now: bool,
    pub future_output_classification: ScholarChatAnswerReadinessOutputClassification,
    pub blockers: Vec<ScholarChatAnswerReadinessBlocker>,
    pub warnings: Vec<ScholarChatAnswerReadinessWarning>,
    pub next_required_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatDraftInferenceStatus {
    Blocked,
    NeedsSources,
    NeedsEvidence,
    NeedsRuntimeConfig,
    NeedsExecutionConsent,
    InferenceSucceeded,
    InferenceFailed,
    TimedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatDraftOutputClassification {
    Blocked,
    UngroundedModelDraft,
    SourceContextDraft,
    GroundedDraftCandidate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatDraftInferenceBlocker {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatDraftInferenceWarning {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScholarChatDraftInferenceRequest {
    pub scholar_chat_request: ScholarChatRequest,
    pub runtime_config: LocalModelRuntimeConfig,
    pub allow_model_execution: bool,
    pub timeout_ms: Option<u64>,
    pub max_output_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScholarChatDraftInferencePreview {
    pub status: ScholarChatDraftInferenceStatus,
    pub output_classification: ScholarChatDraftOutputClassification,
    pub normalized_prompt: String,
    pub mode: ScholarChatMode,
    pub grounding_policy: GroundingPolicy,
    pub selected_source_count: usize,
    pub retrieval_candidate_count: usize,
    pub evidence_candidate_count: usize,
    pub prompt_pack_section_count: usize,
    pub prompt_char_count: usize,
    pub runtime_health_status: LocalModelRuntimeHealthStatus,
    pub invocation_plan_status: LocalRuntimeInvocationPlanStatus,
    pub allow_model_execution: bool,
    pub execution_attempted: bool,
    pub safe_model_file_name: Option<String>,
    pub safe_executable_file_name: Option<String>,
    pub stdout_preview: String,
    pub stderr_preview: String,
    pub duration_ms: u64,
    pub exit_code: Option<i32>,
    pub draft_only: bool,
    pub preview_only: bool,
    pub not_final_answer: bool,
    pub not_grounded_answer: bool,
    pub no_answer_artifact_created: bool,
    pub no_evidence_pack_built: bool,
    pub no_persistence: bool,
    pub blockers: Vec<ScholarChatDraftInferenceBlocker>,
    pub warnings: Vec<ScholarChatDraftInferenceWarning>,
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

pub fn preview_scholar_chat_prompt_pack(
    root: impl Into<PathBuf>,
    request: ScholarChatRequest,
) -> AegisResult<ScholarChatPromptPackPreviewResponse> {
    let evidence_plan = preview_scholar_chat_evidence_plan(root, request)?;
    let context_items = convert_evidence_candidates_to_prompt_context_items(&evidence_plan.candidates);
    let prompt_pack = build_prompt_pack(
        &evidence_plan.mode,
        &evidence_plan.grounding_policy,
        &evidence_plan.normalized_prompt,
        &evidence_plan.selected_source_ids,
        &context_items,
        evidence_plan.evidence_candidate_count,
    );
    let mut warnings = evidence_plan.warnings.clone();
    if evidence_plan.selected_source_count == 0 {
        push_warning(&mut warnings, "No selected sources; prompt pack preview is unscoped.");
    }
    if evidence_plan.evidence_candidate_count == 0 {
        push_warning(&mut warnings, "No evidence candidates were eligible for prompt-pack assembly yet.");
    }
    push_warning(&mut warnings, "This is a prompt pack preview only; no model inference was run.");
    if matches!(evidence_plan.grounding_policy, GroundingPolicy::LocalOnly) {
        push_warning(&mut warnings, "local_only requires local evidence before a prompt pack can be turned into an answer.");
    }
    if matches!(evidence_plan.grounding_policy, GroundingPolicy::AllowMarkedModelKnowledge) {
        push_warning(&mut warnings, "Model knowledge would need to be clearly marked later.");
    }

    Ok(ScholarChatPromptPackPreviewResponse {
        status: ScholarChatPromptPackStatus::PromptPackPreview,
        normalized_prompt: evidence_plan.normalized_prompt,
        mode: evidence_plan.mode,
        grounding_policy: evidence_plan.grounding_policy,
        selected_source_ids: evidence_plan.selected_source_ids,
        selected_source_count: evidence_plan.selected_source_count,
        evidence_candidate_count: evidence_plan.evidence_candidate_count,
        prompt_pack,
        context_items,
        warnings,
    })
}

pub fn preview_scholar_chat_answer_readiness(
    root: impl Into<PathBuf>,
    request: ScholarChatAnswerReadinessRequest,
) -> AegisResult<ScholarChatAnswerReadinessPreview> {
    let root = root.into();
    let scholar_chat_request = request.scholar_chat_request;
    let request_preview = preview_scholar_chat_request(&root, scholar_chat_request.clone())?;
    let retrieval_preview = preview_scholar_chat_retrieval(&root, scholar_chat_request.clone())?;
    let evidence_plan_preview = preview_scholar_chat_evidence_plan(&root, scholar_chat_request.clone())?;
    let prompt_pack_preview = preview_scholar_chat_prompt_pack(&root, scholar_chat_request.clone())?;
    let runtime_health_preview = preview_local_model_runtime_health(&root, request.runtime_config.clone())?;
    let invocation_plan_preview = preview_local_runtime_invocation_plan(
        &root,
        LocalRuntimeInvocationPlanRequest {
            runtime_config: request.runtime_config,
            prompt_text: Some(request_preview.normalized_prompt.clone()),
            estimated_input_char_count: Some(request_preview.normalized_prompt.chars().count() as u32),
            max_output_tokens: Some(1),
            stop_sequences: None,
        },
    )?;

    let selected_source_count = request_preview.selected_source_count;
    let retrieval_candidate_count = retrieval_preview.candidate_count;
    let evidence_candidate_count = evidence_plan_preview.evidence_candidate_count;
    let prompt_pack_ready = prompt_pack_preview.prompt_pack.section_count > 0;
    let runtime_ready = matches!(runtime_health_preview.status, LocalModelRuntimeHealthStatus::ReadyToTestLater);
    let invocation_ready = matches!(invocation_plan_preview.status, LocalRuntimeInvocationPlanStatus::ReadyToInvokeLater);

    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    let mut next_required_actions = Vec::new();

    for warning in request_preview.warnings {
        push_readiness_warning(&mut warnings, "request_preview", &warning);
    }
    for warning in retrieval_preview.warnings {
        push_readiness_warning(&mut warnings, "retrieval_preview", &warning);
    }
    for warning in evidence_plan_preview.warnings {
        push_readiness_warning(&mut warnings, "evidence_plan_preview", &warning);
    }
    for warning in prompt_pack_preview.warnings {
        push_readiness_warning(&mut warnings, "prompt_pack_preview", &warning);
    }
    for warning in runtime_health_preview.warnings {
        push_readiness_warning(&mut warnings, &warning.kind, &warning.message);
    }
    for warning in invocation_plan_preview.plan.warnings {
        push_readiness_warning(&mut warnings, &warning.kind, &warning.message);
    }
    for blocker in invocation_plan_preview.plan.blockers {
        push_readiness_blocker(&mut blockers, &blocker.kind, &blocker.message);
    }

    if selected_source_count == 0 {
        push_readiness_action(
            &mut next_required_actions,
            "Select one or more Scholar Chat sources.",
        );
        if matches!(request_preview.grounding_policy, GroundingPolicy::LocalOnly) {
            push_readiness_blocker(
                &mut blockers,
                "needs_sources",
                "local_only requires selected sources before a grounded draft can be prepared.",
            );
        }
    }

    if selected_source_count > 0 && retrieval_candidate_count == 0 {
        push_readiness_action(
            &mut next_required_actions,
            "Build or refresh the retrieval index for the selected sources.",
        );
        if matches!(request_preview.grounding_policy, GroundingPolicy::LocalOnly) {
            push_readiness_blocker(
                &mut blockers,
                "blocked",
                "local_only requires local evidence before a grounded draft can be prepared.",
            );
        } else {
            push_readiness_blocker(
                &mut blockers,
                "needs_retrieval_index",
                "Retrieval data is not ready for the selected sources yet.",
            );
        }
    }

    if selected_source_count > 0 && evidence_candidate_count == 0 {
        push_readiness_action(
            &mut next_required_actions,
            "Assemble local evidence candidates for the selected sources.",
        );
        if matches!(request_preview.grounding_policy, GroundingPolicy::LocalOnly) {
            push_readiness_blocker(
                &mut blockers,
                "blocked",
                "local_only requires local evidence before a grounded draft can be prepared.",
            );
        } else {
            push_readiness_blocker(
                &mut blockers,
                "needs_evidence_candidates",
                "No evidence candidates are available yet for the selected sources.",
            );
        }
    }

    if !runtime_ready {
        push_readiness_action(
            &mut next_required_actions,
            "Configure a local runtime model and executable.",
        );
        push_readiness_blocker(
            &mut blockers,
            "needs_runtime_config",
            "The local runtime configuration is not ready yet.",
        );
    }

    if !request.allow_model_execution {
        push_readiness_action(
            &mut next_required_actions,
            "Allow future model execution when you are ready to proceed.",
        );
        push_readiness_blocker(
            &mut blockers,
            "needs_execution_consent",
            "Future model execution is not allowed yet.",
        );
    }

    if prompt_pack_ready {
        push_readiness_action(
            &mut next_required_actions,
            "The prompt pack can be assembled later from the current request preview.",
        );
    }

    let status = readiness_status(
        request_preview.grounding_policy.clone(),
        selected_source_count,
        retrieval_candidate_count,
        evidence_candidate_count,
        runtime_ready,
        invocation_ready,
        request.allow_model_execution,
    );

    if matches!(request_preview.grounding_policy, GroundingPolicy::AllowMarkedModelKnowledge)
        && runtime_ready
        && invocation_ready
        && request.allow_model_execution
    {
        push_readiness_warning(
            &mut warnings,
            "future_draft_marking_required",
            "A future ungrounded draft would need explicit model-knowledge marking later.",
        );
    }

    Ok(ScholarChatAnswerReadinessPreview {
        status: status.clone(),
        normalized_prompt: request_preview.normalized_prompt,
        mode: request_preview.mode,
        grounding_policy: request_preview.grounding_policy,
        selected_source_count,
        retrieval_candidate_count,
        evidence_candidate_count,
        prompt_pack_ready,
        runtime_health_status: runtime_health_preview.status,
        invocation_plan_status: invocation_plan_preview.status,
        allow_model_execution: request.allow_model_execution,
        would_generate_answer_now: false,
        would_build_evidence_pack_now: false,
        would_create_final_answer_now: false,
        future_output_classification: readiness_output_classification(status),
        blockers,
        warnings,
        next_required_actions,
    })
}

pub fn preview_scholar_chat_draft_inference(
    root: impl Into<PathBuf>,
    request: ScholarChatDraftInferenceRequest,
) -> AegisResult<ScholarChatDraftInferencePreview> {
    let root = root.into();
    let scholar_chat_request = request.scholar_chat_request;
    let request_preview = preview_scholar_chat_request(&root, scholar_chat_request.clone())?;
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();

    for warning in &request_preview.warnings {
        push_draft_warning(&mut warnings, "request_preview", warning);
    }

    if matches!(request_preview.grounding_policy, GroundingPolicy::ExternalAdaptersLater) {
        push_draft_warning(
            &mut warnings,
            "external_adapters_unavailable",
            "External adapters are not implemented yet and are unused in this preview.",
        );
    }

    if !request.allow_model_execution {
        push_draft_blocker(
            &mut blockers,
            "needs_execution_consent",
            "Future model execution is not allowed yet.",
        );
        push_draft_warning(
            &mut warnings,
            "execution_consent_required",
            "Draft inference preview will not consult the local runtime until execution consent is granted.",
        );
        return Ok(build_draft_inference_preview(
            request_preview.normalized_prompt,
            request_preview.mode,
            request_preview.grounding_policy,
            request_preview.selected_source_count,
            0,
            0,
            0,
            String::new(),
            LocalModelRuntimeHealthStatus::NotConfigured,
            LocalRuntimeInvocationPlanStatus::PreviewOnly,
            ScholarChatDraftInferenceStatus::NeedsExecutionConsent,
            ScholarChatDraftOutputClassification::Blocked,
            request.allow_model_execution,
            false,
            None,
            None,
            String::new(),
            String::new(),
            0,
            None,
            blockers,
            warnings,
        ));
    }

    let prompt_ready = request_preview.selected_source_count > 0
        || matches!(request_preview.grounding_policy, GroundingPolicy::AllowMarkedModelKnowledge);
    if !prompt_ready {
        push_draft_blocker(
            &mut blockers,
            "needs_sources",
            "Selected sources are required before a local draft can be prepared.",
        );
        return Ok(build_draft_inference_preview(
            request_preview.normalized_prompt,
            request_preview.mode,
            request_preview.grounding_policy,
            request_preview.selected_source_count,
            0,
            0,
            0,
            String::new(),
            LocalModelRuntimeHealthStatus::NotConfigured,
            LocalRuntimeInvocationPlanStatus::PreviewOnly,
            ScholarChatDraftInferenceStatus::NeedsSources,
            ScholarChatDraftOutputClassification::Blocked,
            request.allow_model_execution,
            false,
            None,
            None,
            String::new(),
            String::new(),
            0,
            None,
            blockers,
            warnings,
        ));
    }

    let runtime_health_preview = preview_local_model_runtime_health(&root, request.runtime_config.clone())?;
    let runtime_health_status = runtime_health_preview.status.clone();
    for warning in &runtime_health_preview.warnings {
        push_draft_warning(&mut warnings, &warning.kind, &warning.message);
    }
    if !matches!(runtime_health_status, LocalModelRuntimeHealthStatus::ReadyToTestLater) {
        push_draft_blocker(
            &mut blockers,
            "needs_runtime_config",
            "The local runtime configuration is not ready yet.",
        );
        return Ok(build_draft_inference_preview(
            request_preview.normalized_prompt,
            request_preview.mode,
            request_preview.grounding_policy,
            request_preview.selected_source_count,
            0,
            0,
            0,
            String::new(),
            runtime_health_status,
            LocalRuntimeInvocationPlanStatus::NotConfigured,
            ScholarChatDraftInferenceStatus::NeedsRuntimeConfig,
            ScholarChatDraftOutputClassification::Blocked,
            request.allow_model_execution,
            false,
            runtime_health_preview.model_file_name,
            None,
            String::new(),
            String::new(),
            0,
            None,
            blockers,
            warnings,
        ));
    }

    let evidence_plan_preview = if matches!(request_preview.grounding_policy, GroundingPolicy::LocalOnly) {
        Some(preview_scholar_chat_evidence_plan(&root, scholar_chat_request.clone())?)
    } else {
        None
    };
    let retrieval_candidate_count = evidence_plan_preview
        .as_ref()
        .map_or(0, |preview| preview.retrieval_candidate_count);
    let evidence_candidate_count = evidence_plan_preview
        .as_ref()
        .map_or(0, |preview| preview.evidence_candidate_count);

    if let Some(evidence_plan_preview) = &evidence_plan_preview {
        for warning in &evidence_plan_preview.warnings {
            push_draft_warning(&mut warnings, "evidence_plan_preview", warning);
        }
    }

    if matches!(request_preview.grounding_policy, GroundingPolicy::LocalOnly) && evidence_candidate_count == 0 {
        push_draft_blocker(
            &mut blockers,
            "needs_evidence",
            "local_only requires local evidence candidates before draft inference can proceed.",
        );
        push_draft_warning(
            &mut warnings,
            "evidence_required",
            "No local evidence candidates are available for a local_only draft preview.",
        );
        return Ok(build_draft_inference_preview(
            request_preview.normalized_prompt,
            request_preview.mode,
            request_preview.grounding_policy,
            request_preview.selected_source_count,
            retrieval_candidate_count,
            evidence_candidate_count,
            0,
            String::new(),
            runtime_health_status,
            LocalRuntimeInvocationPlanStatus::PreviewOnly,
            ScholarChatDraftInferenceStatus::NeedsEvidence,
            ScholarChatDraftOutputClassification::Blocked,
            request.allow_model_execution,
            false,
            runtime_health_preview.model_file_name,
            None,
            String::new(),
            String::new(),
            0,
            None,
            blockers,
            warnings,
        ));
    }

    let prompt_pack_preview = preview_scholar_chat_prompt_pack(&root, scholar_chat_request.clone())?;
    let prompt_pack_text = render_prompt_pack_for_runtime(&prompt_pack_preview.prompt_pack);
    let invocation_plan_preview = preview_local_runtime_invocation_plan(
        &root,
        LocalRuntimeInvocationPlanRequest {
            runtime_config: request.runtime_config.clone(),
            prompt_text: Some(prompt_pack_text.clone()),
            estimated_input_char_count: Some(prompt_pack_text.chars().count() as u32),
            max_output_tokens: request.max_output_tokens,
            stop_sequences: None,
        },
    )?;
    let invocation_plan_status = invocation_plan_preview.status.clone();
    for warning in &prompt_pack_preview.warnings {
        push_draft_warning(&mut warnings, "prompt_pack_preview", warning);
    }
    for warning in &invocation_plan_preview.plan.warnings {
        push_draft_warning(&mut warnings, &warning.kind, &warning.message);
    }
    for blocker in &invocation_plan_preview.plan.blockers {
        push_draft_blocker(&mut blockers, &blocker.kind, &blocker.message);
    }

    let output_classification = draft_output_classification(&request_preview.grounding_policy, false);

    let smoke_result = smoke_test_local_runtime_inference(
        &root,
        LocalRuntimeSmokeInferenceRequest {
            runtime_config: request.runtime_config,
            allow_execution: true,
            prompt: Some(prompt_pack_text.clone()),
            timeout_ms: request.timeout_ms,
            max_output_tokens: request.max_output_tokens,
        },
    )?;
    for warning in smoke_result.warnings.iter() {
        push_draft_warning(&mut warnings, &warning.kind, &warning.message);
    }
    for blocker in smoke_result.blockers.iter() {
        push_draft_blocker(&mut blockers, &blocker.kind, &blocker.message);
    }

    let status = match smoke_result.status {
        LocalRuntimeSmokeInferenceStatus::InferenceSucceeded => ScholarChatDraftInferenceStatus::InferenceSucceeded,
        LocalRuntimeSmokeInferenceStatus::InferenceFailed => ScholarChatDraftInferenceStatus::InferenceFailed,
        LocalRuntimeSmokeInferenceStatus::TimedOut => ScholarChatDraftInferenceStatus::TimedOut,
        LocalRuntimeSmokeInferenceStatus::Blocked => ScholarChatDraftInferenceStatus::Blocked,
        LocalRuntimeSmokeInferenceStatus::NotConfigured
        | LocalRuntimeSmokeInferenceStatus::ModelMissing
        | LocalRuntimeSmokeInferenceStatus::ExecutableMissing => ScholarChatDraftInferenceStatus::NeedsRuntimeConfig,
    };

    Ok(build_draft_inference_preview(
        request_preview.normalized_prompt,
        request_preview.mode,
        request_preview.grounding_policy,
        request_preview.selected_source_count,
        retrieval_candidate_count,
        evidence_candidate_count,
        prompt_pack_preview.prompt_pack.section_count,
        prompt_pack_text,
        runtime_health_status,
        invocation_plan_status,
        status,
        output_classification,
        request.allow_model_execution,
        smoke_result.execution_attempted,
        smoke_result.safe_model_file_name,
        smoke_result.safe_executable_file_name,
        smoke_result.stdout_preview,
        smoke_result.stderr_preview,
        smoke_result.duration_ms,
        smoke_result.exit_code,
        blockers,
        warnings,
    ))
}

fn render_prompt_pack_for_runtime(prompt_pack: &ScholarChatPromptPack) -> String {
    let mut lines = Vec::new();
    for section in &prompt_pack.sections {
        lines.push(format!("## {}", section.title));
        lines.extend(section.lines.iter().cloned());
        lines.push(String::new());
    }
    lines.join("\n").trim().to_string()
}

fn draft_output_classification(
    grounding_policy: &GroundingPolicy,
    blocked: bool,
) -> ScholarChatDraftOutputClassification {
    if blocked {
        return ScholarChatDraftOutputClassification::Blocked;
    }
    match grounding_policy {
        GroundingPolicy::AllowMarkedModelKnowledge => ScholarChatDraftOutputClassification::UngroundedModelDraft,
        GroundingPolicy::LocalOnly => ScholarChatDraftOutputClassification::GroundedDraftCandidate,
        GroundingPolicy::LocalFirst | GroundingPolicy::ExternalAdaptersLater => {
            ScholarChatDraftOutputClassification::SourceContextDraft
        }
    }
}

fn push_draft_warning(
    warnings: &mut Vec<ScholarChatDraftInferenceWarning>,
    kind: &str,
    message: &str,
) {
    if !warnings.iter().any(|warning| warning.kind == kind && warning.message == message) {
        warnings.push(ScholarChatDraftInferenceWarning {
            kind: kind.to_string(),
            message: message.to_string(),
        });
    }
}

fn push_draft_blocker(
    blockers: &mut Vec<ScholarChatDraftInferenceBlocker>,
    kind: &str,
    message: &str,
) {
    if !blockers.iter().any(|blocker| blocker.kind == kind && blocker.message == message) {
        blockers.push(ScholarChatDraftInferenceBlocker {
            kind: kind.to_string(),
            message: message.to_string(),
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn build_draft_inference_preview(
    normalized_prompt: String,
    mode: ScholarChatMode,
    grounding_policy: GroundingPolicy,
    selected_source_count: usize,
    retrieval_candidate_count: usize,
    evidence_candidate_count: usize,
    prompt_pack_section_count: usize,
    prompt_pack_text: String,
    runtime_health_status: LocalModelRuntimeHealthStatus,
    invocation_plan_status: LocalRuntimeInvocationPlanStatus,
    status: ScholarChatDraftInferenceStatus,
    output_classification: ScholarChatDraftOutputClassification,
    allow_model_execution: bool,
    execution_attempted: bool,
    safe_model_file_name: Option<String>,
    safe_executable_file_name: Option<String>,
    stdout_preview: String,
    stderr_preview: String,
    duration_ms: u64,
    exit_code: Option<i32>,
    blockers: Vec<ScholarChatDraftInferenceBlocker>,
    warnings: Vec<ScholarChatDraftInferenceWarning>,
) -> ScholarChatDraftInferencePreview {
    ScholarChatDraftInferencePreview {
        status,
        output_classification,
        normalized_prompt,
        mode,
        grounding_policy,
        selected_source_count,
        retrieval_candidate_count,
        evidence_candidate_count,
        prompt_pack_section_count,
        prompt_char_count: prompt_pack_text.chars().count(),
        runtime_health_status,
        invocation_plan_status,
        allow_model_execution,
        execution_attempted,
        safe_model_file_name,
        safe_executable_file_name,
        stdout_preview,
        stderr_preview,
        duration_ms,
        exit_code,
        draft_only: true,
        preview_only: true,
        not_final_answer: true,
        not_grounded_answer: true,
        no_answer_artifact_created: true,
        no_evidence_pack_built: true,
        no_persistence: true,
        blockers,
        warnings,
    }
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

fn convert_evidence_candidates_to_prompt_context_items(
    candidates: &[ScholarChatEvidenceCandidate],
) -> Vec<ScholarChatPromptContextItem> {
    candidates
        .iter()
        .map(|result| ScholarChatPromptContextItem {
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

fn build_prompt_pack(
    mode: &ScholarChatMode,
    policy: &GroundingPolicy,
    normalized_prompt: &str,
    selected_source_ids: &[String],
    context_items: &[ScholarChatPromptContextItem],
    evidence_candidate_count: usize,
) -> ScholarChatPromptPack {
    let sections = vec![
        ScholarChatPromptPackSection {
            kind: ScholarChatPromptPackSectionKind::SystemOrPolicyInstructions,
            title: "System or policy instructions".to_string(),
            lines: system_or_policy_instructions(policy),
        },
        ScholarChatPromptPackSection {
            kind: ScholarChatPromptPackSectionKind::ModeInstructions,
            title: "Mode instructions".to_string(),
            lines: mode_instructions(mode),
        },
        ScholarChatPromptPackSection {
            kind: ScholarChatPromptPackSectionKind::GroundingInstructions,
            title: "Grounding instructions".to_string(),
            lines: grounding_pack_instructions(policy, selected_source_ids.len(), evidence_candidate_count),
        },
        ScholarChatPromptPackSection {
            kind: ScholarChatPromptPackSectionKind::SourceContext,
            title: "Source context".to_string(),
            lines: source_context_lines(selected_source_ids, context_items),
        },
        ScholarChatPromptPackSection {
            kind: ScholarChatPromptPackSectionKind::UserPrompt,
            title: "User prompt".to_string(),
            lines: vec![normalized_prompt.to_string()],
        },
    ];

    ScholarChatPromptPack {
        section_count: sections.len(),
        context_item_count: context_items.len(),
        estimated_input_char_count: estimate_prompt_pack_char_count(&sections, context_items),
        sections,
    }
}

fn system_or_policy_instructions(policy: &GroundingPolicy) -> Vec<String> {
    let mut lines = vec![
        "AEGIS Scholar local-first academic Scholar Chat workspace.".to_string(),
        "Preview only; no model inference or answer generation is run here.".to_string(),
    ];
    match policy {
        GroundingPolicy::LocalOnly => lines.push("Use only selected local evidence.".to_string()),
        GroundingPolicy::LocalFirst => lines.push("Prefer selected local evidence before any later fallback.".to_string()),
        GroundingPolicy::AllowMarkedModelKnowledge => lines.push("Model knowledge may be used later only if clearly marked.".to_string()),
        GroundingPolicy::ExternalAdaptersLater => lines.push("External scholarly adapters are not implemented yet.".to_string()),
    }
    lines
}

fn mode_instructions(mode: &ScholarChatMode) -> Vec<String> {
    match mode {
        ScholarChatMode::LectureLearning => vec![
            "Answer from course or lecture material first.".to_string(),
            "Prioritize what was taught and keep explanations grounded.".to_string(),
        ],
        ScholarChatMode::ThesisWriting => vec![
            "Support scientific writing, outlining, and literature synthesis.".to_string(),
            "Keep claims grounded and ready for citation.".to_string(),
        ],
        ScholarChatMode::LiteratureReview => vec![
            "Compare and synthesize papers with provenance.".to_string(),
            "Prefer source-linked evidence over general summaries.".to_string(),
        ],
        ScholarChatMode::Flashcards => vec![
            "Generate source-linked study-card candidates later.".to_string(),
            "Keep prompts compact and recall-oriented.".to_string(),
        ],
        ScholarChatMode::StatisticsMethods => vec![
            "Explain methods and support reproducible academic work.".to_string(),
            "Keep terminology precise and source-linked.".to_string(),
        ],
        ScholarChatMode::GeneralScholar => vec![
            "General academic assistant, local-first.".to_string(),
            "Use selected sources before any later fallback.".to_string(),
        ],
    }
}

fn grounding_pack_instructions(
    policy: &GroundingPolicy,
    selected_source_count: usize,
    evidence_candidate_count: usize,
) -> Vec<String> {
    let mut lines = vec![
        format!("Selected source count: {selected_source_count}."),
        format!("Evidence candidate count: {evidence_candidate_count}."),
    ];
    match policy {
        GroundingPolicy::LocalOnly => lines.push("local_only cannot answer without local evidence.".to_string()),
        GroundingPolicy::LocalFirst => lines.push("Prefer local evidence before any later fallback.".to_string()),
        GroundingPolicy::AllowMarkedModelKnowledge => lines.push("Model knowledge is only allowed when clearly marked later.".to_string()),
        GroundingPolicy::ExternalAdaptersLater => lines.push("External adapters are not implemented in this preview.".to_string()),
    }
    lines
}

fn source_context_lines(selected_source_ids: &[String], context_items: &[ScholarChatPromptContextItem]) -> Vec<String> {
    if selected_source_ids.is_empty() {
        return vec!["No selected sources; prompt pack preview is unscoped.".to_string()];
    }

    let mut counts_by_source = BTreeMap::new();
    for item in context_items {
        *counts_by_source.entry(item.source_id.clone()).or_insert(0usize) += 1;
    }

    let mut lines = vec![format!("Selected source IDs: {}.", selected_source_ids.join(", "))];
    for source_id in selected_source_ids {
        let count = counts_by_source.get(source_id).copied().unwrap_or(0);
        lines.push(format!("{source_id}: {count} evidence candidate(s)."));
    }
    lines
}

fn estimate_prompt_pack_char_count(
    sections: &[ScholarChatPromptPackSection],
    context_items: &[ScholarChatPromptContextItem],
) -> usize {
    let section_chars = sections.iter().fold(0usize, |acc, section| {
        let title_chars = section.title.chars().count();
        let line_chars = section.lines.iter().map(|line| line.chars().count()).sum::<usize>();
        let separator_chars = section.lines.len().saturating_sub(1);
        acc + title_chars + line_chars + separator_chars
    });
    let context_chars = context_items.iter().fold(0usize, |acc, item| {
        acc + item.source_id.chars().count()
            + item.version_id.chars().count()
            + item.chunk_id.chars().count()
            + item.preview.chars().count()
            + item.matched_terms.iter().map(|term| term.chars().count()).sum::<usize>()
            + locator_summary_chars(&item.locator)
    });
    section_chars + context_chars
}

fn locator_summary_chars(locator: &CitationLocator) -> usize {
    let section = locator
        .section_path
        .as_ref()
        .map(|value| value.iter().map(|part| part.chars().count()).sum::<usize>())
        .unwrap_or(0);
    let start = locator.character_start.map(|value| value.to_string().chars().count()).unwrap_or(0);
    let end = locator.character_end.map(|value| value.to_string().chars().count()).unwrap_or(0);
    locator.label.chars().count() + section + start + end
}

fn push_warning(warnings: &mut Vec<String>, message: &str) {
    if !warnings.iter().any(|warning| warning == message) {
        warnings.push(message.to_string());
    }
}

fn push_readiness_warning(
    warnings: &mut Vec<ScholarChatAnswerReadinessWarning>,
    kind: &str,
    message: &str,
) {
    if !warnings.iter().any(|warning| warning.kind == kind && warning.message == message) {
        warnings.push(ScholarChatAnswerReadinessWarning {
            kind: kind.to_string(),
            message: message.to_string(),
        });
    }
}

fn push_readiness_blocker(
    blockers: &mut Vec<ScholarChatAnswerReadinessBlocker>,
    kind: &str,
    message: &str,
) {
    if !blockers.iter().any(|blocker| blocker.kind == kind && blocker.message == message) {
        blockers.push(ScholarChatAnswerReadinessBlocker {
            kind: kind.to_string(),
            message: message.to_string(),
        });
    }
}

fn push_readiness_action(actions: &mut Vec<String>, action: &str) {
    if !actions.iter().any(|existing| existing == action) {
        actions.push(action.to_string());
    }
}

fn readiness_status(
    policy: GroundingPolicy,
    selected_source_count: usize,
    retrieval_candidate_count: usize,
    evidence_candidate_count: usize,
    runtime_ready: bool,
    invocation_ready: bool,
    allow_model_execution: bool,
) -> ScholarChatAnswerReadinessStatus {
    if selected_source_count == 0 {
        if matches!(policy, GroundingPolicy::AllowMarkedModelKnowledge) && runtime_ready && invocation_ready && allow_model_execution {
            return ScholarChatAnswerReadinessStatus::ReadyForDraftInferenceLater;
        }
        return ScholarChatAnswerReadinessStatus::NeedsSources;
    }
    if retrieval_candidate_count == 0 {
        return if matches!(policy, GroundingPolicy::LocalOnly) {
            ScholarChatAnswerReadinessStatus::Blocked
        } else {
            ScholarChatAnswerReadinessStatus::NeedsRetrievalIndex
        };
    }
    if evidence_candidate_count == 0 {
        return if matches!(policy, GroundingPolicy::LocalOnly) {
            ScholarChatAnswerReadinessStatus::Blocked
        } else {
            ScholarChatAnswerReadinessStatus::NeedsEvidenceCandidates
        };
    }
    if !runtime_ready {
        return ScholarChatAnswerReadinessStatus::NeedsRuntimeConfig;
    }
    if !allow_model_execution {
        return ScholarChatAnswerReadinessStatus::NeedsExecutionConsent;
    }
    if matches!(policy, GroundingPolicy::AllowMarkedModelKnowledge) {
        ScholarChatAnswerReadinessStatus::ReadyForDraftInferenceLater
    } else {
        ScholarChatAnswerReadinessStatus::ReadyForGroundedDraftLater
    }
}

fn readiness_output_classification(
    status: ScholarChatAnswerReadinessStatus,
) -> ScholarChatAnswerReadinessOutputClassification {
    match status {
        ScholarChatAnswerReadinessStatus::Blocked
        | ScholarChatAnswerReadinessStatus::NeedsSources
        | ScholarChatAnswerReadinessStatus::NeedsRuntimeConfig
        | ScholarChatAnswerReadinessStatus::NeedsExecutionConsent => {
            ScholarChatAnswerReadinessOutputClassification::Blocked
        }
        ScholarChatAnswerReadinessStatus::NeedsRetrievalIndex
        | ScholarChatAnswerReadinessStatus::NeedsEvidenceCandidates => {
            ScholarChatAnswerReadinessOutputClassification::SourceContextDraft
        }
        ScholarChatAnswerReadinessStatus::ReadyForDraftInferenceLater => {
            ScholarChatAnswerReadinessOutputClassification::UngroundedDraft
        }
        ScholarChatAnswerReadinessStatus::ReadyForGroundedDraftLater => {
            ScholarChatAnswerReadinessOutputClassification::GroundedDraftCandidate
        }
    }
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
    use crate::local_runtime::LocalModelRuntimeKind;
    use std::{env, fs, path::PathBuf, process::Command};

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

    fn prompt_pack_request(prompt: &str, selected_source_ids: Vec<String>) -> ScholarChatRequest {
        ScholarChatRequest {
            prompt: prompt.to_string(),
            mode: ScholarChatMode::ThesisWriting,
            grounding_policy: GroundingPolicy::LocalOnly,
            selected_source_ids,
        }
    }

    fn runtime_config(model_path: Option<&str>, executable_path: Option<&str>) -> LocalModelRuntimeConfig {
        LocalModelRuntimeConfig {
            runtime_kind: LocalModelRuntimeKind::LlamaCpp,
            model_path: model_path.map(|value| value.to_string()),
            executable_path: executable_path.map(|value| value.to_string()),
            context_window: Some(512),
            gpu_layers: Some(0),
            temperature: Some(0.0),
        }
    }

    fn answer_readiness_request(
        prompt: &str,
        grounding_policy: GroundingPolicy,
        selected_source_ids: Vec<String>,
        runtime_config: LocalModelRuntimeConfig,
        allow_model_execution: bool,
    ) -> ScholarChatAnswerReadinessRequest {
        ScholarChatAnswerReadinessRequest {
            scholar_chat_request: ScholarChatRequest {
                prompt: prompt.to_string(),
                mode: ScholarChatMode::ThesisWriting,
                grounding_policy,
                selected_source_ids,
            },
            runtime_config,
            allow_model_execution,
        }
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

    fn build_runtime_fixture(temp: &tempfile::TempDir) -> LocalModelRuntimeConfig {
        let model_path = temp.path().join("ready-model.gguf");
        let executable_path = temp.path().join("ready-smoke-helper.exe");
        fs::write(&model_path, "gguf placeholder").unwrap();
        fs::write(&executable_path, "runtime placeholder").unwrap();
        runtime_config(
            Some(model_path.to_string_lossy().as_ref()),
            Some(executable_path.to_string_lossy().as_ref()),
        )
    }

    fn smoke_helper_executable(temp: &tempfile::TempDir) -> PathBuf {
        let source_path = temp.path().join("smoke_helper.rs");
        let executable_path = temp.path().join(if cfg!(windows) { "smoke_helper.exe" } else { "smoke_helper" });
        let source = r#"
use std::{env, thread, time::Duration};

fn prompt_argument(args: &[String]) -> String {
    args.windows(2)
        .find(|pair| pair[0] == "-p" || pair[0] == "--prompt")
        .map(|pair| pair[1].clone())
        .unwrap_or_default()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let prompt = prompt_argument(&args);
    println!("stdout marker");
    println!("args={}", args.join(" | "));
    println!("{}", "S".repeat(5000));
    eprintln!("stderr marker");
    eprintln!("args={}", args.join(" | "));
    eprintln!("{}", "E".repeat(5000));
    if prompt.contains("SLEEP") {
        thread::sleep(Duration::from_millis(700));
    }
    if prompt.contains("FAIL") {
        std::process::exit(7);
    }
}
"#;
        fs::write(&source_path, source).unwrap();
        let rustc = env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
        let status = Command::new(rustc)
            .arg("--crate-type")
            .arg("bin")
            .arg("--edition")
            .arg("2021")
            .arg(&source_path)
            .arg("-o")
            .arg(&executable_path)
            .status()
            .unwrap();
        assert!(status.success());
        executable_path
    }

    fn build_draft_runtime_fixture(temp: &tempfile::TempDir) -> LocalModelRuntimeConfig {
        let model_path = temp.path().join("draft-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        let executable_path = smoke_helper_executable(temp);
        runtime_config(
            Some(model_path.to_string_lossy().as_ref()),
            Some(executable_path.to_string_lossy().as_ref()),
        )
    }

    fn draft_inference_request(
        prompt: &str,
        grounding_policy: GroundingPolicy,
        selected_source_ids: Vec<String>,
        runtime_config: LocalModelRuntimeConfig,
        allow_model_execution: bool,
        timeout_ms: Option<u64>,
        max_output_tokens: Option<u32>,
    ) -> ScholarChatDraftInferenceRequest {
        ScholarChatDraftInferenceRequest {
            scholar_chat_request: ScholarChatRequest {
                prompt: prompt.to_string(),
                mode: ScholarChatMode::ThesisWriting,
                grounding_policy,
                selected_source_ids,
            },
            runtime_config,
            allow_model_execution,
            timeout_ms,
            max_output_tokens,
        }
    }

    fn assert_readiness_boundary_fields(preview: &ScholarChatAnswerReadinessPreview) {
        assert!(!preview.would_generate_answer_now);
        assert!(!preview.would_build_evidence_pack_now);
        assert!(!preview.would_create_final_answer_now);
        assert!(preview.prompt_pack_ready);
    }

    fn assert_draft_boundary_fields(preview: &ScholarChatDraftInferencePreview) {
        assert!(preview.draft_only);
        assert!(preview.preview_only);
        assert!(preview.not_final_answer);
        assert!(preview.not_grounded_answer);
        assert!(preview.no_answer_artifact_created);
        assert!(preview.no_evidence_pack_built);
        assert!(preview.no_persistence);
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

    #[test]
    fn scholar_chat_prompt_pack_rejects_empty_prompt() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_prompt_pack(temp.path(), prompt_pack_request("   ", vec![]));
        assert!(matches!(result, Err(AegisError::ScholarChatPromptEmpty)));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn scholar_chat_prompt_pack_rejects_invalid_source_ids_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["", " ", "..", "../evil", "evil/source", "evil\\source"] {
            let result = preview_scholar_chat_prompt_pack(temp.path(), prompt_pack_request("Explain alpha", vec![invalid.to_string()]));
            assert!(matches!(result, Err(AegisError::ScholarChatInvalidSourceId)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn scholar_chat_prompt_pack_allows_no_selected_sources_with_warning() {
        let temp = tempfile::tempdir().unwrap();
        let response = preview_scholar_chat_prompt_pack(temp.path(), prompt_pack_request("Explain alpha", vec![])).unwrap();
        assert_eq!(response.selected_source_count, 0);
        assert_eq!(response.evidence_candidate_count, 0);
        assert_eq!(response.context_items.len(), 0);
        assert_eq!(response.status, ScholarChatPromptPackStatus::PromptPackPreview);
        assert!(response.warnings.iter().any(|warning| warning.contains("unscoped")));
        assert!(response.warnings.iter().any(|warning| warning.contains("no model inference")));
    }

    #[test]
    fn scholar_chat_prompt_pack_includes_mode_and_grounding_sections() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta\n\nalpha gamma\n");
        let response = preview_scholar_chat_prompt_pack(
            temp.path(),
            prompt_pack_request("  alpha  ", vec![source_id.clone()]),
        )
        .unwrap();
        assert_eq!(response.normalized_prompt, "alpha");
        assert_eq!(response.selected_source_ids, vec![source_id.clone()]);
        assert_eq!(response.prompt_pack.section_count, 5);
        assert_eq!(response.prompt_pack.context_item_count, response.context_items.len());
        assert!(response.prompt_pack.sections.iter().any(|section| section.kind == ScholarChatPromptPackSectionKind::SystemOrPolicyInstructions));
        assert!(response.prompt_pack.sections.iter().any(|section| section.kind == ScholarChatPromptPackSectionKind::ModeInstructions));
        assert!(response.prompt_pack.sections.iter().any(|section| section.kind == ScholarChatPromptPackSectionKind::GroundingInstructions));
        assert!(response.prompt_pack.sections.iter().any(|section| section.kind == ScholarChatPromptPackSectionKind::SourceContext));
        assert!(response.prompt_pack.sections.iter().any(|section| section.kind == ScholarChatPromptPackSectionKind::UserPrompt));
        assert!(response.prompt_pack.sections.iter().any(|section| section.lines.iter().any(|line| line.contains("local evidence"))));
        assert!(response.warnings.iter().any(|warning| warning.contains("local evidence")));
    }

    #[test]
    fn scholar_chat_prompt_pack_is_deterministic_and_path_free() {
        let temp = tempfile::tempdir().unwrap();
        let source_a = build_source_with_index(&temp, "alpha beta\n\nalpha gamma\n");
        let source_b = build_source_with_index(&temp, "alpha delta\n\nalpha epsilon\n");
        let request = prompt_pack_request("  alpha  ", vec![source_b.clone(), source_a.clone()]);
        let first = preview_scholar_chat_prompt_pack(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_prompt_pack(temp.path(), request).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.selected_source_ids, vec![source_b, source_a]);
        assert_eq!(first.prompt_pack.section_count, first.prompt_pack.sections.len());
        assert_eq!(first.prompt_pack.context_item_count, first.context_items.len());
        let debug = format!("{first:?}");
        let json = serde_json::to_string(&first).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert!(!temp.path().join(".aegis").join("corpus").join("sources").join("missing").exists());
    }

    #[test]
    fn scholar_chat_answer_readiness_rejects_empty_prompt() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_answer_readiness(
            temp.path(),
            answer_readiness_request(
                "   ",
                GroundingPolicy::LocalOnly,
                vec![],
                runtime_config(None, None),
                false,
            ),
        );
        assert!(matches!(result, Err(AegisError::ScholarChatPromptEmpty)));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn scholar_chat_answer_readiness_rejects_invalid_source_ids_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["", " ", "..", "../evil", "evil/source", "evil\\source"] {
            let result = preview_scholar_chat_answer_readiness(
                temp.path(),
                answer_readiness_request(
                    "Explain alpha",
                    GroundingPolicy::LocalOnly,
                    vec![invalid.to_string()],
                    runtime_config(None, None),
                    false,
                ),
            );
            assert!(matches!(result, Err(AegisError::ScholarChatInvalidSourceId)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn scholar_chat_answer_readiness_requires_sources_for_local_only() {
        let temp = tempfile::tempdir().unwrap();
        let runtime_config = build_runtime_fixture(&temp);
        let before_entries = fs::read_dir(temp.path()).unwrap().count();
        let response = preview_scholar_chat_answer_readiness(
            temp.path(),
            answer_readiness_request(
                "Explain alpha",
                GroundingPolicy::LocalOnly,
                vec![],
                runtime_config,
                true,
            ),
        )
        .unwrap();
        let after_entries = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(response.status, ScholarChatAnswerReadinessStatus::NeedsSources);
        assert_eq!(response.future_output_classification, ScholarChatAnswerReadinessOutputClassification::Blocked);
        assert_eq!(response.selected_source_count, 0);
        assert!(response.blockers.iter().any(|blocker| blocker.kind == "needs_sources"));
        assert!(response.next_required_actions.iter().any(|action| action.contains("Select one or more Scholar Chat sources")));
        assert_eq!(before_entries, after_entries);
        assert_readiness_boundary_fields(&response);
        let debug = format!("{response:?}");
        let json = serde_json::to_string(&response).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
    }

    #[test]
    fn scholar_chat_answer_readiness_blocks_local_only_without_retrieval_candidates() {
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
        let runtime_config = build_runtime_fixture(&temp);
        let before_entries = fs::read_dir(temp.path()).unwrap().count();
        let response = preview_scholar_chat_answer_readiness(
            temp.path(),
            answer_readiness_request(
                "Explain alpha",
                GroundingPolicy::LocalOnly,
                vec![source.source_id.clone()],
                runtime_config,
                true,
            ),
        )
        .unwrap();
        let after_entries = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(response.status, ScholarChatAnswerReadinessStatus::Blocked);
        assert_eq!(response.future_output_classification, ScholarChatAnswerReadinessOutputClassification::Blocked);
        assert_eq!(response.selected_source_count, 1);
        assert_eq!(response.retrieval_candidate_count, 0);
        assert_eq!(response.evidence_candidate_count, 0);
        assert!(response.blockers.iter().any(|blocker| blocker.kind == "blocked"));
        assert!(response.next_required_actions.iter().any(|action| action.contains("retrieval index")));
        assert_eq!(before_entries, after_entries);
        assert_readiness_boundary_fields(&response);
        let debug = format!("{response:?}");
        let json = serde_json::to_string(&response).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
    }

    #[test]
    fn scholar_chat_answer_readiness_needs_runtime_config_when_local_runtime_is_missing() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta\n\nalpha gamma\n");
        let before_entries = fs::read_dir(temp.path()).unwrap().count();
        let response = preview_scholar_chat_answer_readiness(
            temp.path(),
            answer_readiness_request(
                "Explain alpha",
                GroundingPolicy::LocalFirst,
                vec![source_id.clone()],
                runtime_config(None, None),
                true,
            ),
        )
        .unwrap();
        let after_entries = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(response.status, ScholarChatAnswerReadinessStatus::NeedsRuntimeConfig);
        assert_eq!(response.runtime_health_status, LocalModelRuntimeHealthStatus::ConfigPresent);
        assert!(response.blockers.iter().any(|blocker| blocker.kind == "needs_runtime_config"));
        assert_eq!(before_entries, after_entries);
        assert_readiness_boundary_fields(&response);
        let debug = format!("{response:?}");
        let json = serde_json::to_string(&response).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
    }

    #[test]
    fn scholar_chat_answer_readiness_requires_execution_consent_when_runtime_is_ready() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta\n\nalpha gamma\n");
        let runtime_config = build_runtime_fixture(&temp);
        let before_entries = fs::read_dir(temp.path()).unwrap().count();
        let response = preview_scholar_chat_answer_readiness(
            temp.path(),
            answer_readiness_request(
                "Explain alpha",
                GroundingPolicy::LocalFirst,
                vec![source_id.clone()],
                runtime_config,
                false,
            ),
        )
        .unwrap();
        let after_entries = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(response.status, ScholarChatAnswerReadinessStatus::NeedsExecutionConsent);
        assert_eq!(response.invocation_plan_status, LocalRuntimeInvocationPlanStatus::ReadyToInvokeLater);
        assert!(response.blockers.iter().any(|blocker| blocker.kind == "needs_execution_consent"));
        assert_eq!(before_entries, after_entries);
        assert_readiness_boundary_fields(&response);
        let debug = format!("{response:?}");
        let json = serde_json::to_string(&response).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
    }

    #[test]
    fn scholar_chat_answer_readiness_can_be_ready_for_draft_inference_later() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta\n\nalpha gamma\n");
        let runtime_config = build_runtime_fixture(&temp);
        let before_entries = fs::read_dir(temp.path()).unwrap().count();
        let request = answer_readiness_request(
            "  Explain alpha  ",
            GroundingPolicy::AllowMarkedModelKnowledge,
            vec![source_id.clone()],
            runtime_config,
            true,
        );
        let first = preview_scholar_chat_answer_readiness(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_answer_readiness(temp.path(), request).unwrap();
        let after_entries = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(first, second);
        assert_eq!(first.status, ScholarChatAnswerReadinessStatus::ReadyForDraftInferenceLater);
        assert_eq!(first.future_output_classification, ScholarChatAnswerReadinessOutputClassification::UngroundedDraft);
        assert_eq!(first.normalized_prompt, "Explain alpha");
        assert_eq!(first.mode, ScholarChatMode::ThesisWriting);
        assert_eq!(first.grounding_policy, GroundingPolicy::AllowMarkedModelKnowledge);
        assert_eq!(first.selected_source_count, 1);
        assert!(first.prompt_pack_ready);
        assert!(first.would_generate_answer_now == false);
        assert!(first.would_build_evidence_pack_now == false);
        assert!(first.would_create_final_answer_now == false);
        assert!(first.warnings.iter().any(|warning| warning.kind == "future_draft_marking_required"));
        assert!(first.next_required_actions.iter().any(|action| action.contains("prompt pack")));
        assert_eq!(before_entries, after_entries);
        assert_readiness_boundary_fields(&first);
        let debug = format!("{first:?}");
        let json = serde_json::to_string(&first).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
    }

    #[test]
    fn scholar_chat_draft_inference_rejects_empty_prompt() {
        let temp = tempfile::tempdir().unwrap();
        let runtime_config = build_draft_runtime_fixture(&temp);
        let before_entries = fs::read_dir(temp.path()).unwrap().count();
        let result = preview_scholar_chat_draft_inference(
            temp.path(),
            draft_inference_request(
                "   ",
                GroundingPolicy::AllowMarkedModelKnowledge,
                vec![],
                runtime_config,
                true,
                None,
                None,
            ),
        );
        assert!(matches!(result, Err(AegisError::ScholarChatPromptEmpty)));
        let after_entries = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(before_entries, after_entries);
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn scholar_chat_draft_inference_rejects_invalid_source_ids_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        let runtime_config = build_draft_runtime_fixture(&temp);
        let before_entries = fs::read_dir(temp.path()).unwrap().count();
        for invalid in ["", " ", "..", "../evil", "evil/source", "evil\\source"] {
            let result = preview_scholar_chat_draft_inference(
                temp.path(),
                draft_inference_request(
                    "Explain alpha",
                    GroundingPolicy::AllowMarkedModelKnowledge,
                    vec![invalid.to_string()],
                    runtime_config.clone(),
                    true,
                    None,
                    None,
                ),
            );
            assert!(matches!(result, Err(AegisError::ScholarChatInvalidSourceId)));
            assert!(!temp.path().join(".aegis").exists());
        }
        let after_entries = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(before_entries, after_entries);
    }

    #[test]
    fn scholar_chat_draft_inference_blocks_when_execution_is_disabled() {
        let temp = tempfile::tempdir().unwrap();
        let runtime_config = build_draft_runtime_fixture(&temp);
        let before_entries = fs::read_dir(temp.path()).unwrap().count();
        let request = draft_inference_request(
            "Explain alpha",
            GroundingPolicy::AllowMarkedModelKnowledge,
            vec![],
            runtime_config,
            false,
            None,
            None,
        );
        let first = preview_scholar_chat_draft_inference(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_draft_inference(temp.path(), request).unwrap();
        let response = first.clone();
        let after_entries = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(first, second);
        assert_eq!(response.status, ScholarChatDraftInferenceStatus::NeedsExecutionConsent);
        assert_eq!(response.output_classification, ScholarChatDraftOutputClassification::Blocked);
        assert!(!response.execution_attempted);
        assert_eq!(response.prompt_pack_section_count, 0);
        assert_eq!(response.prompt_char_count, 0);
        assert_eq!(response.runtime_health_status, LocalModelRuntimeHealthStatus::NotConfigured);
        assert_eq!(response.invocation_plan_status, LocalRuntimeInvocationPlanStatus::PreviewOnly);
        assert!(response.blockers.iter().any(|blocker| blocker.kind == "needs_execution_consent"));
        assert_eq!(before_entries, after_entries);
        assert_draft_boundary_fields(&response);
        let debug = format!("{response:?}");
        let json = serde_json::to_string(&response).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
    }

    #[test]
    fn scholar_chat_draft_inference_blocks_local_only_without_evidence_candidates() {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("note-no-index.md");
        fs::write(&source_path, "alpha beta\n").unwrap();
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
        let runtime_config = build_draft_runtime_fixture(&temp);
        let before_entries = fs::read_dir(temp.path()).unwrap().count();
        let response = preview_scholar_chat_draft_inference(
            temp.path(),
            draft_inference_request(
                "Explain alpha",
                GroundingPolicy::LocalOnly,
                vec![source.source_id.clone()],
                runtime_config,
                true,
                None,
                None,
            ),
        )
        .unwrap();
        let after_entries = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(response.status, ScholarChatDraftInferenceStatus::NeedsEvidence);
        assert_eq!(response.output_classification, ScholarChatDraftOutputClassification::Blocked);
        assert!(!response.execution_attempted);
        assert_eq!(response.prompt_pack_section_count, 0);
        assert_eq!(response.prompt_char_count, 0);
        assert!(response.blockers.iter().any(|blocker| blocker.kind == "needs_evidence"));
        assert!(response.warnings.iter().any(|warning| warning.kind == "evidence_required"));
        assert_eq!(before_entries, after_entries);
        assert_draft_boundary_fields(&response);
        let debug = format!("{response:?}");
        let json = serde_json::to_string(&response).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
    }

    #[test]
    fn scholar_chat_draft_inference_needs_runtime_config_when_runtime_is_missing() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta\n\nalpha gamma\n");
        let before_entries = fs::read_dir(temp.path()).unwrap().count();
        let response = preview_scholar_chat_draft_inference(
            temp.path(),
            draft_inference_request(
                "Explain alpha",
                GroundingPolicy::LocalFirst,
                vec![source_id.clone()],
                LocalModelRuntimeConfig {
                    runtime_kind: LocalModelRuntimeKind::None,
                    model_path: None,
                    executable_path: None,
                    context_window: Some(512),
                    gpu_layers: Some(0),
                    temperature: Some(0.0),
                },
                true,
                None,
                None,
            ),
        )
        .unwrap();
        assert_eq!(response.status, ScholarChatDraftInferenceStatus::NeedsRuntimeConfig);
        assert_eq!(response.output_classification, ScholarChatDraftOutputClassification::Blocked);
        assert!(!response.execution_attempted);
        assert_eq!(response.prompt_pack_section_count, 0);
        assert_eq!(response.prompt_char_count, 0);
        assert!(response.blockers.iter().any(|blocker| blocker.kind == "needs_runtime_config"));
        let after_entries = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(before_entries, after_entries);
        assert_draft_boundary_fields(&response);
        let debug = format!("{response:?}");
        let json = serde_json::to_string(&response).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
    }

    #[test]
    fn scholar_chat_draft_inference_reports_missing_model_and_executable_without_paths() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta\n\nalpha gamma\n");
        let before_entries = fs::read_dir(temp.path()).unwrap().count();
        let response = preview_scholar_chat_draft_inference(
            temp.path(),
            draft_inference_request(
                "Explain alpha",
                GroundingPolicy::LocalFirst,
                vec![source_id.clone()],
                runtime_config(
                    Some(temp.path().join("missing-model.gguf").to_string_lossy().as_ref()),
                    Some(temp.path().join("missing-draft-helper.exe").to_string_lossy().as_ref()),
                ),
                true,
                None,
                None,
            ),
        )
        .unwrap();
        assert_eq!(response.status, ScholarChatDraftInferenceStatus::NeedsRuntimeConfig);
        assert_eq!(response.output_classification, ScholarChatDraftOutputClassification::Blocked);
        assert!(!response.execution_attempted);
        assert_eq!(response.prompt_pack_section_count, 0);
        assert_eq!(response.prompt_char_count, 0);
        assert_eq!(response.safe_model_file_name.as_deref(), Some("missing-model.gguf"));
        assert!(response.safe_executable_file_name.is_none());
        let after_entries = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(before_entries, after_entries);
        assert_draft_boundary_fields(&response);
        let debug = format!("{response:?}");
        let json = serde_json::to_string(&response).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
    }

    #[test]
    fn scholar_chat_draft_inference_is_deterministic_and_path_free() {
        let temp = tempfile::tempdir().unwrap();
        let runtime_config = build_draft_runtime_fixture(&temp);
        let before_entries = fs::read_dir(temp.path()).unwrap().count();
        let request = draft_inference_request(
            "  Explain alpha  ",
            GroundingPolicy::AllowMarkedModelKnowledge,
            vec![],
            runtime_config,
            true,
            None,
            None,
        );
        let first = preview_scholar_chat_draft_inference(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_draft_inference(temp.path(), request).unwrap();
        let after_entries = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(first, second);
        assert_eq!(first.status, ScholarChatDraftInferenceStatus::InferenceSucceeded);
        assert_eq!(first.output_classification, ScholarChatDraftOutputClassification::UngroundedModelDraft);
        assert_eq!(first.normalized_prompt, "Explain alpha");
        assert_eq!(first.mode, ScholarChatMode::ThesisWriting);
        assert_eq!(first.grounding_policy, GroundingPolicy::AllowMarkedModelKnowledge);
        assert_eq!(first.selected_source_count, 0);
        assert_eq!(first.retrieval_candidate_count, 0);
        assert_eq!(first.evidence_candidate_count, 0);
        assert!(first.prompt_pack_section_count > 0);
        assert!(first.execution_attempted);
        assert_eq!(first.allow_model_execution, true);
        assert!(first.stdout_preview.contains("stdout marker"));
        assert!(first.stderr_preview.contains("stderr marker"));
        assert_draft_boundary_fields(&first);
        assert_eq!(before_entries, after_entries);
        let debug = format!("{first:?}");
        let json = serde_json::to_string(&first).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
    }
}
