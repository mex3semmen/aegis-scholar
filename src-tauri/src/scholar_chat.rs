use crate::corpus_paths::CorpusPaths;
use crate::errors::{AegisError, AegisResult};
use crate::locators::CitationLocator;
use crate::source_registry::SourceRegistry;
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
use std::fs;
use std::collections::{BTreeMap, BTreeSet};
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatDraftGroundingInspectionStatus {
    Blocked,
    NoDraftText,
    NoEvidenceCandidates,
    Inspected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatDraftGroundingSupportStatus {
    NotInspected,
    Unsupported,
    WeaklySupported,
    SupportedByLocalEvidence,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatDraftGroundingInspectionBlocker {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatDraftGroundingInspectionWarning {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatDraftGroundingInspectionRequest {
    pub scholar_chat_request: ScholarChatRequest,
    pub draft_text: Option<String>,
    pub max_items: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatGroundedAnswerBuildIntentRequest {
    pub grounding_request: ScholarChatDraftGroundingInspectionRequest,
    pub answer_draft_id: Option<String>,
    pub explicit_user_intent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatDraftGroundingInspectionItem {
    pub item_index: usize,
    pub text_preview: String,
    pub support_status: ScholarChatDraftGroundingSupportStatus,
    pub matched_evidence_count: usize,
    pub source_ids: Vec<String>,
    pub locator_previews: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatDraftGroundingInspectionPreview {
    pub status: ScholarChatDraftGroundingInspectionStatus,
    pub normalized_prompt: String,
    pub draft_char_count: usize,
    pub selected_source_count: usize,
    pub evidence_candidate_count: usize,
    pub inspected_item_count: usize,
    pub unsupported_item_count: usize,
    pub weakly_supported_item_count: usize,
    pub supported_item_count: usize,
    pub items: Vec<ScholarChatDraftGroundingInspectionItem>,
    pub inspection_only: bool,
    pub not_grounded_answer: bool,
    pub not_final_answer: bool,
    pub no_evidence_pack_built: bool,
    pub no_answer_artifact_created: bool,
    pub no_persistence: bool,
    pub no_llm_call: bool,
    pub no_runtime_execution: bool,
    pub blockers: Vec<ScholarChatDraftGroundingInspectionBlocker>,
    pub warnings: Vec<ScholarChatDraftGroundingInspectionWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatGroundedDraftReadinessStatus {
    Blocked,
    NeedsReview,
    ReadyForGroundedDraftLater,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatGroundedDraftReadinessPreview {
    pub status: ScholarChatGroundedDraftReadinessStatus,
    pub inspection_status: ScholarChatDraftGroundingInspectionStatus,
    pub normalized_prompt: String,
    pub selected_source_count: usize,
    pub evidence_candidate_count: usize,
    pub inspected_item_count: usize,
    pub supported_item_count: usize,
    pub weakly_supported_item_count: usize,
    pub unsupported_item_count: usize,
    pub summary: String,
    pub preview_only: bool,
    pub not_grounded_answer: bool,
    pub not_final_answer: bool,
    pub no_answer_artifact_created: bool,
    pub no_evidence_pack_built: bool,
    pub no_persistence: bool,
    pub no_llm_call: bool,
    pub no_runtime_execution: bool,
    pub blockers: Vec<ScholarChatDraftGroundingInspectionBlocker>,
    pub warnings: Vec<ScholarChatDraftGroundingInspectionWarning>,
    pub next_required_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatGroundedAnswerBuildPlanStatus {
    Blocked,
    NeedsReview,
    PlanReadyLater,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatGroundedAnswerBuildPlanPreview {
    pub status: ScholarChatGroundedAnswerBuildPlanStatus,
    pub readiness_status: ScholarChatGroundedDraftReadinessStatus,
    pub normalized_prompt: String,
    pub selected_source_count: usize,
    pub evidence_candidate_count: usize,
    pub inspected_item_count: usize,
    pub supported_item_count: usize,
    pub weakly_supported_item_count: usize,
    pub unsupported_item_count: usize,
    pub summary: String,
    pub planned_steps: Vec<String>,
    pub preview_only: bool,
    pub not_answer_draft: bool,
    pub not_grounded_answer: bool,
    pub not_final_answer: bool,
    pub no_answer_artifact_created: bool,
    pub no_evidence_pack_built: bool,
    pub no_persistence: bool,
    pub no_llm_call: bool,
    pub no_runtime_execution: bool,
    pub blockers: Vec<ScholarChatDraftGroundingInspectionBlocker>,
    pub warnings: Vec<ScholarChatDraftGroundingInspectionWarning>,
    pub next_required_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatGroundedAnswerCandidateStatus {
    Blocked,
    NeedsReview,
    CandidateReadyLater,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatGroundedAnswerCandidateItem {
    pub item_index: usize,
    pub statement_preview: String,
    pub support_status: ScholarChatDraftGroundingSupportStatus,
    pub source_ids: Vec<String>,
    pub locator_previews: Vec<String>,
    pub matched_evidence_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatGroundedAnswerCandidatePreview {
    pub status: ScholarChatGroundedAnswerCandidateStatus,
    pub normalized_prompt: String,
    pub selected_source_count: usize,
    pub evidence_candidate_count: usize,
    pub inspected_item_count: usize,
    pub supported_item_count: usize,
    pub weakly_supported_item_count: usize,
    pub unsupported_item_count: usize,
    pub candidate_statement_count: usize,
    pub summary: String,
    pub candidate_items: Vec<ScholarChatGroundedAnswerCandidateItem>,
    pub preview_only: bool,
    pub not_answer_draft: bool,
    pub not_grounded_answer: bool,
    pub not_final_answer: bool,
    pub no_answer_artifact_created: bool,
    pub no_evidence_pack_built: bool,
    pub no_persistence: bool,
    pub no_llm_call: bool,
    pub no_runtime_execution: bool,
    pub blockers: Vec<ScholarChatDraftGroundingInspectionBlocker>,
    pub warnings: Vec<ScholarChatDraftGroundingInspectionWarning>,
    pub next_required_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatGroundedAnswerWriteEligibilityStatus {
    Blocked,
    NeedsReview,
    WriteEligibleLater,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatGroundedAnswerWriteEligibilityPreview {
    pub status: ScholarChatGroundedAnswerWriteEligibilityStatus,
    pub candidate_status: ScholarChatGroundedAnswerCandidateStatus,
    pub normalized_prompt: String,
    pub selected_source_count: usize,
    pub evidence_candidate_count: usize,
    pub inspected_item_count: usize,
    pub supported_item_count: usize,
    pub weakly_supported_item_count: usize,
    pub unsupported_item_count: usize,
    pub candidate_statement_count: usize,
    pub eligibility_reasons: Vec<String>,
    pub blockers: Vec<ScholarChatDraftGroundingInspectionBlocker>,
    pub warnings: Vec<ScholarChatDraftGroundingInspectionWarning>,
    pub next_required_actions: Vec<String>,
    pub summary: String,
    pub preview_only: bool,
    pub not_answer_draft: bool,
    pub not_grounded_answer: bool,
    pub not_final_answer: bool,
    pub no_answer_artifact_created: bool,
    pub no_evidence_pack_built: bool,
    pub no_persistence: bool,
    pub no_llm_call: bool,
    pub no_runtime_execution: bool,
    pub no_registry_status_change: bool,
    pub no_audit_write: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatGroundedAnswerBuildIntentStatus {
    Blocked,
    NeedsReview,
    IntentReadyLater,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatGroundedAnswerBuildIntentPreview {
    pub status: ScholarChatGroundedAnswerBuildIntentStatus,
    pub write_eligibility_status: ScholarChatGroundedAnswerWriteEligibilityStatus,
    pub candidate_status: ScholarChatGroundedAnswerCandidateStatus,
    pub normalized_prompt: String,
    pub selected_source_count: usize,
    pub evidence_candidate_count: usize,
    pub inspected_item_count: usize,
    pub supported_item_count: usize,
    pub weakly_supported_item_count: usize,
    pub unsupported_item_count: usize,
    pub candidate_statement_count: usize,
    pub required_inputs: Vec<String>,
    pub missing_inputs: Vec<String>,
    pub intent_reasons: Vec<String>,
    pub blockers: Vec<ScholarChatDraftGroundingInspectionBlocker>,
    pub warnings: Vec<ScholarChatDraftGroundingInspectionWarning>,
    pub next_required_actions: Vec<String>,
    pub summary: String,
    pub preview_only: bool,
    pub not_answer_draft: bool,
    pub not_grounded_answer: bool,
    pub not_final_answer: bool,
    pub no_answer_artifact_created: bool,
    pub no_evidence_pack_built: bool,
    pub no_persistence: bool,
    pub no_llm_call: bool,
    pub no_runtime_execution: bool,
    pub no_registry_status_change: bool,
    pub no_audit_write: bool,
    pub no_grounded_answer_service_call: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatGroundedAnswerBuildRequestStatus {
    Blocked,
    NeedsReview,
    RequestReadyLater,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatGroundedAnswerBuildRequestPreviewRequest {
    pub build_intent_request: ScholarChatGroundedAnswerBuildIntentRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatGroundedAnswerBuildRequestPreview {
    pub status: ScholarChatGroundedAnswerBuildRequestStatus,
    pub build_intent_status: ScholarChatGroundedAnswerBuildIntentStatus,
    pub write_eligibility_status: ScholarChatGroundedAnswerWriteEligibilityStatus,
    pub candidate_status: ScholarChatGroundedAnswerCandidateStatus,
    pub normalized_prompt: String,
    pub selected_source_count: usize,
    pub evidence_candidate_count: usize,
    pub inspected_item_count: usize,
    pub supported_item_count: usize,
    pub weakly_supported_item_count: usize,
    pub unsupported_item_count: usize,
    pub candidate_statement_count: usize,
    pub answer_draft_id: Option<String>,
    pub selected_source_ids: Vec<String>,
    pub required_inputs: Vec<String>,
    pub missing_inputs: Vec<String>,
    pub request_reasons: Vec<String>,
    pub blockers: Vec<ScholarChatDraftGroundingInspectionBlocker>,
    pub warnings: Vec<ScholarChatDraftGroundingInspectionWarning>,
    pub next_required_actions: Vec<String>,
    pub summary: String,
    pub preview_only: bool,
    pub not_answer_draft: bool,
    pub not_grounded_answer: bool,
    pub not_final_answer: bool,
    pub no_answer_artifact_created: bool,
    pub no_evidence_pack_built: bool,
    pub no_persistence: bool,
    pub no_llm_call: bool,
    pub no_runtime_execution: bool,
    pub no_registry_status_change: bool,
    pub no_audit_write: bool,
    pub no_grounded_answer_service_call: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatGroundedAnswerBuildPreflightStatus {
    Blocked,
    NeedsReview,
    PreflightReadyLater,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatGroundedAnswerBuildPreflightPreviewRequest {
    pub build_request_preview_request: ScholarChatGroundedAnswerBuildRequestPreviewRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatGroundedAnswerBuildPreflightPreview {
    pub status: ScholarChatGroundedAnswerBuildPreflightStatus,
    pub build_request_status: ScholarChatGroundedAnswerBuildRequestStatus,
    pub build_intent_status: ScholarChatGroundedAnswerBuildIntentStatus,
    pub write_eligibility_status: ScholarChatGroundedAnswerWriteEligibilityStatus,
    pub candidate_status: ScholarChatGroundedAnswerCandidateStatus,
    pub normalized_prompt: String,
    pub selected_source_count: usize,
    pub evidence_candidate_count: usize,
    pub inspected_item_count: usize,
    pub supported_item_count: usize,
    pub weakly_supported_item_count: usize,
    pub unsupported_item_count: usize,
    pub candidate_statement_count: usize,
    pub answer_draft_id: Option<String>,
    pub selected_source_ids: Vec<String>,
    pub answer_draft_present: bool,
    pub answer_draft_readable: bool,
    pub answer_draft_claim_count: usize,
    pub required_inputs: Vec<String>,
    pub missing_inputs: Vec<String>,
    pub preflight_reasons: Vec<String>,
    pub blockers: Vec<ScholarChatDraftGroundingInspectionBlocker>,
    pub warnings: Vec<ScholarChatDraftGroundingInspectionWarning>,
    pub next_required_actions: Vec<String>,
    pub summary: String,
    pub preview_only: bool,
    pub not_answer_draft: bool,
    pub not_grounded_answer: bool,
    pub not_final_answer: bool,
    pub no_answer_artifact_created: bool,
    pub no_evidence_pack_built: bool,
    pub no_persistence: bool,
    pub no_llm_call: bool,
    pub no_runtime_execution: bool,
    pub no_registry_status_change: bool,
    pub no_audit_write: bool,
    pub no_grounded_answer_service_call: bool,
    pub no_grounded_answer_write: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatGroundedAnswerExecutionReadinessStatus {
    Blocked,
    NeedsReview,
    ExecutionReadyLater,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatGroundedAnswerExecutionReadinessPreviewRequest {
    pub build_preflight_preview_request: ScholarChatGroundedAnswerBuildPreflightPreviewRequest,
    pub execution_consent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScholarChatGroundedAnswerExecutionReadinessPreview {
    pub status: ScholarChatGroundedAnswerExecutionReadinessStatus,
    pub build_preflight_status: ScholarChatGroundedAnswerBuildPreflightStatus,
    pub build_request_status: ScholarChatGroundedAnswerBuildRequestStatus,
    pub build_intent_status: ScholarChatGroundedAnswerBuildIntentStatus,
    pub write_eligibility_status: ScholarChatGroundedAnswerWriteEligibilityStatus,
    pub candidate_status: ScholarChatGroundedAnswerCandidateStatus,
    pub normalized_prompt: String,
    pub answer_draft_id: Option<String>,
    pub selected_source_ids: Vec<String>,
    pub selected_source_count: usize,
    pub evidence_candidate_count: usize,
    pub inspected_item_count: usize,
    pub supported_item_count: usize,
    pub weakly_supported_item_count: usize,
    pub unsupported_item_count: usize,
    pub candidate_statement_count: usize,
    pub answer_draft_present: bool,
    pub answer_draft_readable: bool,
    pub answer_draft_claim_count: usize,
    pub execution_consent: bool,
    pub required_inputs: Vec<String>,
    pub missing_inputs: Vec<String>,
    pub readiness_reasons: Vec<String>,
    pub blockers: Vec<ScholarChatDraftGroundingInspectionBlocker>,
    pub warnings: Vec<ScholarChatDraftGroundingInspectionWarning>,
    pub next_required_actions: Vec<String>,
    pub summary: String,
    pub preview_only: bool,
    pub not_answer_draft: bool,
    pub not_grounded_answer: bool,
    pub not_final_answer: bool,
    pub no_answer_artifact_created: bool,
    pub no_evidence_pack_built: bool,
    pub no_persistence: bool,
    pub no_llm_call: bool,
    pub no_runtime_execution: bool,
    pub no_registry_status_change: bool,
    pub no_audit_write: bool,
    pub no_grounded_answer_service_call: bool,
    pub no_grounded_answer_write: bool,
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

pub fn preview_scholar_chat_draft_grounding_inspection(
    root: impl Into<PathBuf>,
    request: ScholarChatDraftGroundingInspectionRequest,
) -> AegisResult<ScholarChatDraftGroundingInspectionPreview> {
    let root = root.into();
    let scholar_chat_request = request.scholar_chat_request;
    let request_preview = preview_scholar_chat_request(&root, scholar_chat_request.clone())?;
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    let normalized_draft_text = normalize_optional_draft_text(request.draft_text);

    for warning in &request_preview.warnings {
        push_grounding_inspection_warning(&mut warnings, "request_preview", warning);
    }
    push_grounding_inspection_warning(
        &mut warnings,
        "boundary",
        "This is a draft grounding inspection preview only; no grounded answer, final answer, Evidence Pack, or persistence was created.",
    );

    let Some(normalized_draft_text) = normalized_draft_text else {
        push_grounding_inspection_blocker(
            &mut blockers,
            "draft_text_missing",
            "No draft text was provided to inspect.",
        );
        return Ok(build_draft_grounding_inspection_preview(
            request_preview.normalized_prompt,
            0,
            request_preview.selected_source_count,
            0,
            Vec::new(),
            0,
            0,
            0,
            blockers,
            warnings,
        ));
    };

    if request_preview.selected_source_count == 0 {
        push_grounding_inspection_blocker(
            &mut blockers,
            "needs_sources",
            "No Scholar Chat source context was selected for this inspection.",
        );
        push_grounding_inspection_warning(
            &mut warnings,
            "unscoped_inspection",
            "No Scholar Chat source context selected; inspection will be unscoped.",
        );
        return Ok(build_draft_grounding_inspection_preview(
            request_preview.normalized_prompt,
            normalized_draft_text.chars().count(),
            request_preview.selected_source_count,
            0,
            Vec::new(),
            0,
            0,
            0,
            blockers,
            warnings,
        ));
    }

    let evidence_plan_preview = preview_scholar_chat_evidence_plan(&root, scholar_chat_request.clone())?;
    for warning in &evidence_plan_preview.warnings {
        push_grounding_inspection_warning(&mut warnings, "evidence_plan_preview", warning);
    }

    let evidence_candidate_count = evidence_plan_preview.evidence_candidate_count;
    if evidence_candidate_count == 0 {
        push_grounding_inspection_blocker(
            &mut blockers,
            "needs_evidence_candidates",
            "No local evidence candidates were available for the selected sources.",
        );
        push_grounding_inspection_warning(
            &mut warnings,
            "evidence_required",
            "No local evidence candidates are available yet for draft grounding inspection.",
        );
        return Ok(build_draft_grounding_inspection_preview(
            request_preview.normalized_prompt,
            normalized_draft_text.chars().count(),
            request_preview.selected_source_count,
            evidence_candidate_count,
            Vec::new(),
            0,
            0,
            0,
            blockers,
            warnings,
        ));
    }

    let max_items = request
        .max_items
        .unwrap_or(SCHOLAR_CHAT_DRAFT_GROUNDING_INSPECTION_LIMIT)
        .clamp(1, SCHOLAR_CHAT_DRAFT_GROUNDING_INSPECTION_LIMIT);
    let inspected_items = inspect_draft_grounding_items(&normalized_draft_text, &evidence_plan_preview.candidates, max_items);
    if inspected_items.items.is_empty() {
        push_grounding_inspection_blocker(
            &mut blockers,
            "draft_text_missing",
            "Draft text did not contain inspectable content.",
        );
        push_grounding_inspection_warning(
            &mut warnings,
            "draft_text_empty",
            "No inspectable draft sentences were found in the provided draft text.",
        );
        return Ok(build_draft_grounding_inspection_preview(
            request_preview.normalized_prompt,
            normalized_draft_text.chars().count(),
            request_preview.selected_source_count,
            evidence_candidate_count,
            Vec::new(),
            0,
            0,
            0,
            blockers,
            warnings,
        ));
    }
    if inspected_items.was_clamped {
        push_grounding_inspection_warning(
            &mut warnings,
            "inspection_clamped",
            &format!("Only the first {max_items} draft items were inspected."),
        );
    }
    warnings.extend(inspected_items.warnings);

    Ok(build_draft_grounding_inspection_preview(
        request_preview.normalized_prompt,
        normalized_draft_text.chars().count(),
        request_preview.selected_source_count,
        evidence_candidate_count,
        inspected_items.items,
        inspected_items.supported_item_count,
        inspected_items.weakly_supported_item_count,
        inspected_items.unsupported_item_count,
        blockers,
        warnings,
    ))
}

pub fn preview_scholar_chat_grounded_draft_readiness(
    root: impl Into<PathBuf>,
    request: ScholarChatDraftGroundingInspectionRequest,
) -> AegisResult<ScholarChatGroundedDraftReadinessPreview> {
    let root = root.into();
    let inspection_preview = preview_scholar_chat_draft_grounding_inspection(&root, request)?;
    let status = grounded_draft_readiness_status(&inspection_preview);
    let mut blockers = inspection_preview.blockers.clone();
    let mut warnings = inspection_preview.warnings.clone();

    push_grounding_inspection_warning(
        &mut warnings,
        "boundary",
        "This is a grounded-draft readiness preview only; it is not a grounded answer, final answer, Evidence Pack, or persisted artifact.",
    );

    match status {
        ScholarChatGroundedDraftReadinessStatus::Blocked => {
            if blockers.is_empty() {
                push_grounding_inspection_blocker(
                    &mut blockers,
                    "readiness_blocked",
                    "Grounded-draft readiness is blocked until draft text, source context, and local evidence are available.",
                );
            }
        }
        ScholarChatGroundedDraftReadinessStatus::NeedsReview => {
            push_grounding_inspection_warning(
                &mut warnings,
                "needs_review",
                "Weakly supported or unsupported draft items remain and need review before a grounded-answer path is added.",
            );
        }
        ScholarChatGroundedDraftReadinessStatus::ReadyForGroundedDraftLater => {
            push_grounding_inspection_warning(
                &mut warnings,
                "ready_for_grounded_draft_later",
                "All inspected items were supported by local evidence. This is still only a readiness preview.",
            );
        }
    }

    let summary = grounded_draft_readiness_summary(&status, &inspection_preview);
    let next_required_actions = grounded_draft_readiness_next_required_actions(&status, &inspection_preview);

    Ok(ScholarChatGroundedDraftReadinessPreview {
        status,
        inspection_status: inspection_preview.status,
        normalized_prompt: inspection_preview.normalized_prompt,
        selected_source_count: inspection_preview.selected_source_count,
        evidence_candidate_count: inspection_preview.evidence_candidate_count,
        inspected_item_count: inspection_preview.inspected_item_count,
        supported_item_count: inspection_preview.supported_item_count,
        weakly_supported_item_count: inspection_preview.weakly_supported_item_count,
        unsupported_item_count: inspection_preview.unsupported_item_count,
        summary,
        preview_only: true,
        not_grounded_answer: true,
        not_final_answer: true,
        no_answer_artifact_created: true,
        no_evidence_pack_built: true,
        no_persistence: true,
        no_llm_call: true,
        no_runtime_execution: true,
        blockers,
        warnings,
        next_required_actions,
    })
}

pub fn preview_scholar_chat_grounded_answer_build_plan(
    root: impl Into<PathBuf>,
    request: ScholarChatDraftGroundingInspectionRequest,
) -> AegisResult<ScholarChatGroundedAnswerBuildPlanPreview> {
    let root = root.into();
    let readiness_preview = preview_scholar_chat_grounded_draft_readiness(&root, request)?;
    let status = grounded_answer_build_plan_status(&readiness_preview);
    let mut blockers = readiness_preview.blockers.clone();
    let mut warnings = readiness_preview.warnings.clone();

    push_grounding_inspection_warning(
        &mut warnings,
        "boundary",
        "This is a grounded-answer build plan preview only; it is not an AnswerDraft, GroundedAnswer, FinalAnswer, Evidence Pack, or persisted artifact.",
    );

    match status {
        ScholarChatGroundedAnswerBuildPlanStatus::Blocked => {
            if blockers.is_empty() {
                push_grounding_inspection_blocker(
                    &mut blockers,
                    "build_plan_blocked",
                    "Grounded-answer build planning is blocked until draft grounding readiness is available.",
                );
            }
        }
        ScholarChatGroundedAnswerBuildPlanStatus::NeedsReview => {
            push_grounding_inspection_warning(
                &mut warnings,
                "needs_review",
                "Weakly supported or unsupported draft items remain and should be reviewed before planning a GroundedAnswer.",
            );
        }
        ScholarChatGroundedAnswerBuildPlanStatus::PlanReadyLater => {
            push_grounding_inspection_warning(
                &mut warnings,
                "plan_ready_later",
                "All inspected items were supported by local evidence. This is still only a plan preview.",
            );
        }
    }

    let summary = grounded_answer_build_plan_summary(&status, &readiness_preview);
    let planned_steps = grounded_answer_build_plan_planned_steps(&status);
    let next_required_actions =
        grounded_answer_build_plan_next_required_actions(&status, &readiness_preview);

    Ok(ScholarChatGroundedAnswerBuildPlanPreview {
        status,
        readiness_status: readiness_preview.status,
        normalized_prompt: readiness_preview.normalized_prompt,
        selected_source_count: readiness_preview.selected_source_count,
        evidence_candidate_count: readiness_preview.evidence_candidate_count,
        inspected_item_count: readiness_preview.inspected_item_count,
        supported_item_count: readiness_preview.supported_item_count,
        weakly_supported_item_count: readiness_preview.weakly_supported_item_count,
        unsupported_item_count: readiness_preview.unsupported_item_count,
        summary,
        planned_steps,
        preview_only: true,
        not_answer_draft: true,
        not_grounded_answer: true,
        not_final_answer: true,
        no_answer_artifact_created: true,
        no_evidence_pack_built: true,
        no_persistence: true,
        no_llm_call: true,
        no_runtime_execution: true,
        blockers,
        warnings,
        next_required_actions,
    })
}

pub fn preview_scholar_chat_grounded_answer_candidate(
    root: impl Into<PathBuf>,
    request: ScholarChatDraftGroundingInspectionRequest,
) -> AegisResult<ScholarChatGroundedAnswerCandidatePreview> {
    let root = root.into();
    let build_plan_preview = preview_scholar_chat_grounded_answer_build_plan(&root, request.clone())?;
    let status = grounded_answer_candidate_status(&build_plan_preview);
    let mut blockers = build_plan_preview.blockers.clone();
    let mut warnings = build_plan_preview.warnings.clone();

    push_grounding_inspection_warning(
        &mut warnings,
        "boundary",
        "This is a grounded-answer candidate preview only; it is not an AnswerDraft, GroundedAnswer, FinalAnswer, Evidence Pack, or persisted artifact.",
    );

    let candidate_items = if matches!(status, ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater) {
        let inspection_preview = preview_scholar_chat_draft_grounding_inspection(&root, request)?;
        grounded_answer_candidate_items_from_inspection(&inspection_preview)
    } else {
        Vec::new()
    };

    match status {
        ScholarChatGroundedAnswerCandidateStatus::Blocked => {
            if blockers.is_empty() {
                push_grounding_inspection_blocker(
                    &mut blockers,
                    "candidate_blocked",
                    "Grounded-answer candidate preview is blocked until grounded-draft readiness is available.",
                );
            }
        }
        ScholarChatGroundedAnswerCandidateStatus::NeedsReview => {
            push_grounding_inspection_warning(
                &mut warnings,
                "needs_review",
                "Weakly supported or unsupported draft items remain and should be reviewed before a grounded-answer candidate is considered.",
            );
        }
        ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater => {
            push_grounding_inspection_warning(
                &mut warnings,
                "candidate_ready_later",
                "All inspected items were supported by local evidence. This is still only a grounded-answer candidate preview.",
            );
        }
    }

    let summary = grounded_answer_candidate_summary(&status, &build_plan_preview);
    let next_required_actions = grounded_answer_candidate_next_required_actions(&status, &build_plan_preview);

    Ok(ScholarChatGroundedAnswerCandidatePreview {
        status,
        normalized_prompt: build_plan_preview.normalized_prompt,
        selected_source_count: build_plan_preview.selected_source_count,
        evidence_candidate_count: build_plan_preview.evidence_candidate_count,
        inspected_item_count: build_plan_preview.inspected_item_count,
        supported_item_count: build_plan_preview.supported_item_count,
        weakly_supported_item_count: build_plan_preview.weakly_supported_item_count,
        unsupported_item_count: build_plan_preview.unsupported_item_count,
        candidate_statement_count: candidate_items.len(),
        summary,
        candidate_items,
        preview_only: true,
        not_answer_draft: true,
        not_grounded_answer: true,
        not_final_answer: true,
        no_answer_artifact_created: true,
        no_evidence_pack_built: true,
        no_persistence: true,
        no_llm_call: true,
        no_runtime_execution: true,
        blockers,
        warnings,
        next_required_actions,
    })
}

pub fn preview_scholar_chat_grounded_answer_write_eligibility(
    root: impl Into<PathBuf>,
    request: ScholarChatDraftGroundingInspectionRequest,
) -> AegisResult<ScholarChatGroundedAnswerWriteEligibilityPreview> {
    let root = root.into();
    let candidate_preview = preview_scholar_chat_grounded_answer_candidate(&root, request)?;
    Ok(grounded_answer_write_eligibility_preview_from_candidate_preview(candidate_preview))
}

const SCHOLAR_CHAT_DRAFT_GROUNDING_INSPECTION_LIMIT: usize = 8;

struct DraftGroundingInspectionItems {
    items: Vec<ScholarChatDraftGroundingInspectionItem>,
    supported_item_count: usize,
    weakly_supported_item_count: usize,
    unsupported_item_count: usize,
    warnings: Vec<ScholarChatDraftGroundingInspectionWarning>,
    was_clamped: bool,
}

fn build_draft_grounding_inspection_preview(
    normalized_prompt: String,
    draft_char_count: usize,
    selected_source_count: usize,
    evidence_candidate_count: usize,
    items: Vec<ScholarChatDraftGroundingInspectionItem>,
    supported_item_count: usize,
    weakly_supported_item_count: usize,
    unsupported_item_count: usize,
    blockers: Vec<ScholarChatDraftGroundingInspectionBlocker>,
    warnings: Vec<ScholarChatDraftGroundingInspectionWarning>,
) -> ScholarChatDraftGroundingInspectionPreview {
    let inspected_item_count = items.len();
    ScholarChatDraftGroundingInspectionPreview {
        status: if draft_char_count == 0 {
            ScholarChatDraftGroundingInspectionStatus::NoDraftText
        } else if selected_source_count == 0 {
            ScholarChatDraftGroundingInspectionStatus::Blocked
        } else if evidence_candidate_count == 0 {
            ScholarChatDraftGroundingInspectionStatus::NoEvidenceCandidates
        } else {
            ScholarChatDraftGroundingInspectionStatus::Inspected
        },
        normalized_prompt,
        draft_char_count,
        selected_source_count,
        evidence_candidate_count,
        inspected_item_count,
        unsupported_item_count,
        weakly_supported_item_count,
        supported_item_count,
        items,
        inspection_only: true,
        not_grounded_answer: true,
        not_final_answer: true,
        no_evidence_pack_built: true,
        no_answer_artifact_created: true,
        no_persistence: true,
        no_llm_call: true,
        no_runtime_execution: true,
        blockers,
        warnings,
    }
}

fn grounded_draft_readiness_status(
    inspection_preview: &ScholarChatDraftGroundingInspectionPreview,
) -> ScholarChatGroundedDraftReadinessStatus {
    match inspection_preview.status {
        ScholarChatDraftGroundingInspectionStatus::Blocked
        | ScholarChatDraftGroundingInspectionStatus::NoDraftText
        | ScholarChatDraftGroundingInspectionStatus::NoEvidenceCandidates => {
            ScholarChatGroundedDraftReadinessStatus::Blocked
        }
        ScholarChatDraftGroundingInspectionStatus::Inspected => {
            if inspection_preview.inspected_item_count == 0 {
                ScholarChatGroundedDraftReadinessStatus::Blocked
            } else if inspection_preview.unsupported_item_count > 0
                || inspection_preview.weakly_supported_item_count > 0
            {
                ScholarChatGroundedDraftReadinessStatus::NeedsReview
            } else {
                ScholarChatGroundedDraftReadinessStatus::ReadyForGroundedDraftLater
            }
        }
    }
}

fn grounded_draft_readiness_summary(
    status: &ScholarChatGroundedDraftReadinessStatus,
    inspection_preview: &ScholarChatDraftGroundingInspectionPreview,
) -> String {
    match status {
        ScholarChatGroundedDraftReadinessStatus::Blocked => {
            match inspection_preview.status {
                ScholarChatDraftGroundingInspectionStatus::NoDraftText => {
                    "Grounded-draft readiness is blocked because no draft text was provided.".to_string()
                }
                ScholarChatDraftGroundingInspectionStatus::NoEvidenceCandidates => {
                    "Grounded-draft readiness is blocked because no local evidence candidates were available.".to_string()
                }
                ScholarChatDraftGroundingInspectionStatus::Blocked => {
                    "Grounded-draft readiness is blocked because no Scholar Chat source context was selected.".to_string()
                }
                ScholarChatDraftGroundingInspectionStatus::Inspected => {
                    "Grounded-draft readiness is blocked because no inspectable draft items were found.".to_string()
                }
            }
        }
        ScholarChatGroundedDraftReadinessStatus::NeedsReview => {
            "The draft is not ready yet: weakly supported or unsupported items remain.".to_string()
        }
        ScholarChatGroundedDraftReadinessStatus::ReadyForGroundedDraftLater => {
            "All inspected items were supported by local evidence. This is still only a readiness preview.".to_string()
        }
    }
}

fn grounded_draft_readiness_next_required_actions(
    status: &ScholarChatGroundedDraftReadinessStatus,
    inspection_preview: &ScholarChatDraftGroundingInspectionPreview,
) -> Vec<String> {
    match status {
        ScholarChatGroundedDraftReadinessStatus::Blocked => {
            match inspection_preview.status {
                ScholarChatDraftGroundingInspectionStatus::NoDraftText => {
                    vec!["Provide draft text before previewing grounded-draft readiness.".to_string()]
                }
                ScholarChatDraftGroundingInspectionStatus::NoEvidenceCandidates => {
                    vec![
                        "Add local evidence candidates for the selected sources before retrying grounded-draft readiness."
                            .to_string(),
                    ]
                }
                ScholarChatDraftGroundingInspectionStatus::Blocked => {
                    vec!["Select Scholar Chat source context before previewing grounded-draft readiness.".to_string()]
                }
                ScholarChatDraftGroundingInspectionStatus::Inspected => {
                    vec!["Review the draft grounding inspection and try again.".to_string()]
                }
            }
        }
        ScholarChatGroundedDraftReadinessStatus::NeedsReview => vec![
            "Review weakly supported and unsupported draft items before treating this draft as ready for a future grounded-answer path."
                .to_string(),
        ],
        ScholarChatGroundedDraftReadinessStatus::ReadyForGroundedDraftLater => vec![
            "A grounded-answer implementation can be added later without changing this readiness preview.".to_string(),
        ],
    }
}

fn grounded_answer_build_plan_status(
    readiness_preview: &ScholarChatGroundedDraftReadinessPreview,
) -> ScholarChatGroundedAnswerBuildPlanStatus {
    match readiness_preview.status {
        ScholarChatGroundedDraftReadinessStatus::Blocked => ScholarChatGroundedAnswerBuildPlanStatus::Blocked,
        ScholarChatGroundedDraftReadinessStatus::NeedsReview => ScholarChatGroundedAnswerBuildPlanStatus::NeedsReview,
        ScholarChatGroundedDraftReadinessStatus::ReadyForGroundedDraftLater => {
            ScholarChatGroundedAnswerBuildPlanStatus::PlanReadyLater
        }
    }
}

fn grounded_answer_build_plan_summary(
    status: &ScholarChatGroundedAnswerBuildPlanStatus,
    readiness_preview: &ScholarChatGroundedDraftReadinessPreview,
) -> String {
    match status {
        ScholarChatGroundedAnswerBuildPlanStatus::Blocked => match readiness_preview.status {
            ScholarChatGroundedDraftReadinessStatus::Blocked => {
                "Grounded-answer build planning is blocked because grounded-draft readiness is blocked.".to_string()
            }
            ScholarChatGroundedDraftReadinessStatus::NeedsReview => {
                "Grounded-answer build planning is blocked because grounded-draft readiness still needs review.".to_string()
            }
            ScholarChatGroundedDraftReadinessStatus::ReadyForGroundedDraftLater => {
                "Grounded-answer build planning is blocked until the readiness preview is available.".to_string()
            }
        },
        ScholarChatGroundedAnswerBuildPlanStatus::NeedsReview => {
            "The draft is not yet ready for a grounded-answer build plan because weakly supported or unsupported items remain.".to_string()
        }
        ScholarChatGroundedAnswerBuildPlanStatus::PlanReadyLater => {
            "All inspected items were supported by local evidence. This is still only a grounded-answer build plan preview.".to_string()
        }
    }
}

fn grounded_answer_build_plan_planned_steps(
    status: &ScholarChatGroundedAnswerBuildPlanStatus,
) -> Vec<String> {
    match status {
        ScholarChatGroundedAnswerBuildPlanStatus::Blocked => vec![
            "Resolve grounded-draft readiness blockers.".to_string(),
            "Re-run draft grounding inspection and readiness preview.".to_string(),
            "Only then add a future GroundedAnswer implementation.".to_string(),
        ],
        ScholarChatGroundedAnswerBuildPlanStatus::NeedsReview => vec![
            "Review supported draft items.".to_string(),
            "Resolve weakly supported and unsupported items.".to_string(),
            "Require an explicit implementation phase before writing GroundedAnswer.".to_string(),
        ],
        ScholarChatGroundedAnswerBuildPlanStatus::PlanReadyLater => vec![
            "Review supported draft items.".to_string(),
            "Map supported draft items to future grounded claims.".to_string(),
            "Require an explicit implementation phase before writing GroundedAnswer.".to_string(),
        ],
    }
}

fn grounded_answer_build_plan_next_required_actions(
    status: &ScholarChatGroundedAnswerBuildPlanStatus,
    readiness_preview: &ScholarChatGroundedDraftReadinessPreview,
) -> Vec<String> {
    let mut next_required_actions = readiness_preview.next_required_actions.clone();
    match status {
        ScholarChatGroundedAnswerBuildPlanStatus::Blocked => {
            push_unique_text(
                &mut next_required_actions,
                "Resolve grounded-draft readiness blockers before any GroundedAnswer implementation.",
            );
        }
        ScholarChatGroundedAnswerBuildPlanStatus::NeedsReview => {
            push_unique_text(
                &mut next_required_actions,
                "Review weakly supported and unsupported draft items before any GroundedAnswer implementation.",
            );
        }
        ScholarChatGroundedAnswerBuildPlanStatus::PlanReadyLater => {
            push_unique_text(
                &mut next_required_actions,
                "A GroundedAnswer implementation can be added later without changing this plan preview.",
            );
        }
    }
    next_required_actions
}

fn grounded_answer_candidate_status(
    build_plan_preview: &ScholarChatGroundedAnswerBuildPlanPreview,
) -> ScholarChatGroundedAnswerCandidateStatus {
    match build_plan_preview.status {
        ScholarChatGroundedAnswerBuildPlanStatus::Blocked => ScholarChatGroundedAnswerCandidateStatus::Blocked,
        ScholarChatGroundedAnswerBuildPlanStatus::NeedsReview => ScholarChatGroundedAnswerCandidateStatus::NeedsReview,
        ScholarChatGroundedAnswerBuildPlanStatus::PlanReadyLater => {
            ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater
        }
    }
}

fn grounded_answer_candidate_summary(
    status: &ScholarChatGroundedAnswerCandidateStatus,
    build_plan_preview: &ScholarChatGroundedAnswerBuildPlanPreview,
) -> String {
    match status {
        ScholarChatGroundedAnswerCandidateStatus::Blocked => match build_plan_preview.status {
            ScholarChatGroundedAnswerBuildPlanStatus::Blocked => {
                "Grounded-answer candidate preview is blocked because grounded-answer build planning is blocked."
                    .to_string()
            }
            ScholarChatGroundedAnswerBuildPlanStatus::NeedsReview => {
                "Grounded-answer candidate preview is blocked because grounded-answer build planning still needs review."
                    .to_string()
            }
            ScholarChatGroundedAnswerBuildPlanStatus::PlanReadyLater => {
                "Grounded-answer candidate preview is blocked until the build-plan preview is available.".to_string()
            }
        },
        ScholarChatGroundedAnswerCandidateStatus::NeedsReview => {
            "The draft is not yet ready for a grounded-answer candidate because weakly supported or unsupported items remain."
                .to_string()
        }
        ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater => {
            "All inspected items were supported by local evidence. This is still only a grounded-answer candidate preview."
                .to_string()
        }
    }
}

fn grounded_answer_candidate_next_required_actions(
    status: &ScholarChatGroundedAnswerCandidateStatus,
    build_plan_preview: &ScholarChatGroundedAnswerBuildPlanPreview,
) -> Vec<String> {
    let mut next_required_actions = build_plan_preview.next_required_actions.clone();
    match status {
        ScholarChatGroundedAnswerCandidateStatus::Blocked => {
            push_unique_text(
                &mut next_required_actions,
                "Resolve grounded-answer build-plan blockers before considering a grounded-answer candidate.",
            );
        }
        ScholarChatGroundedAnswerCandidateStatus::NeedsReview => {
            push_unique_text(
                &mut next_required_actions,
                "Review weakly supported and unsupported draft items before considering a grounded-answer candidate.",
            );
        }
        ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater => {
            push_unique_text(
                &mut next_required_actions,
                "A GroundedAnswer implementation can be added later without changing this candidate preview.",
            );
        }
    }
    next_required_actions
}

fn grounded_answer_candidate_items_from_inspection(
    inspection_preview: &ScholarChatDraftGroundingInspectionPreview,
) -> Vec<ScholarChatGroundedAnswerCandidateItem> {
    inspection_preview
        .items
        .iter()
        .map(|item| ScholarChatGroundedAnswerCandidateItem {
            item_index: item.item_index,
            statement_preview: item.text_preview.clone(),
            support_status: item.support_status.clone(),
            source_ids: item.source_ids.clone(),
            locator_previews: item.locator_previews.clone(),
            matched_evidence_count: item.matched_evidence_count,
        })
        .collect()
}

fn grounded_answer_write_eligibility_status(
    candidate_preview: &ScholarChatGroundedAnswerCandidatePreview,
) -> ScholarChatGroundedAnswerWriteEligibilityStatus {
    match candidate_preview.status {
        ScholarChatGroundedAnswerCandidateStatus::Blocked => {
            ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked
        }
        ScholarChatGroundedAnswerCandidateStatus::NeedsReview => {
            ScholarChatGroundedAnswerWriteEligibilityStatus::NeedsReview
        }
        ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater => {
            if candidate_preview.candidate_statement_count > 0 {
                ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater
            } else {
                ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked
            }
        }
    }
}

fn grounded_answer_write_eligibility_summary(
    status: &ScholarChatGroundedAnswerWriteEligibilityStatus,
    candidate_preview: &ScholarChatGroundedAnswerCandidatePreview,
) -> String {
    match status {
        ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked => {
            match candidate_preview.status {
                ScholarChatGroundedAnswerCandidateStatus::Blocked => {
                    "Grounded-answer write eligibility is blocked because grounded-answer candidate preview is blocked.".to_string()
                }
                ScholarChatGroundedAnswerCandidateStatus::NeedsReview => {
                    "Grounded-answer write eligibility is blocked because grounded-answer candidate preview still needs review.".to_string()
                }
                ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater => {
                    if candidate_preview.candidate_statement_count == 0 {
                        "Grounded-answer write eligibility is blocked because no candidate statements were available.".to_string()
                    } else {
                        "Grounded-answer write eligibility is blocked until the candidate preview is available.".to_string()
                    }
                }
            }
        }
        ScholarChatGroundedAnswerWriteEligibilityStatus::NeedsReview => {
            "The draft is not yet ready for a grounded-answer write because weakly supported or unsupported items remain.".to_string()
        }
        ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater => {
            "All inspected items were supported by local evidence. A future GroundedAnswer write can be added later.".to_string()
        }
    }
}

fn grounded_answer_write_eligibility_reasons(
    status: &ScholarChatGroundedAnswerWriteEligibilityStatus,
    candidate_preview: &ScholarChatGroundedAnswerCandidatePreview,
) -> Vec<String> {
    let mut reasons = vec![
        format!("Candidate status: {:?}", candidate_preview.status),
        format!("Candidate statements: {}", candidate_preview.candidate_statement_count),
    ];
    match status {
        ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked => match candidate_preview.status {
            ScholarChatGroundedAnswerCandidateStatus::Blocked => {
                reasons.push("Grounded-answer candidate preview is blocked.".to_string());
            }
            ScholarChatGroundedAnswerCandidateStatus::NeedsReview => {
                reasons.push("Grounded-answer candidate preview still needs review.".to_string());
            }
            ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater => {
                if candidate_preview.candidate_statement_count == 0 {
                    reasons.push("No candidate statements were available.".to_string());
                } else {
                    reasons.push("Grounded-answer write eligibility is still blocked.".to_string());
                }
            }
        },
        ScholarChatGroundedAnswerWriteEligibilityStatus::NeedsReview => {
            reasons.push("Weakly supported or unsupported draft items remain.".to_string());
        }
        ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater => {
            reasons.push("All inspected items were supported by local evidence.".to_string());
            reasons.push(
                "A future GroundedAnswer write can be added later after an explicit implementation phase."
                    .to_string(),
            );
        }
    }
    reasons
}

fn grounded_answer_write_eligibility_next_required_actions(
    status: &ScholarChatGroundedAnswerWriteEligibilityStatus,
    candidate_preview: &ScholarChatGroundedAnswerCandidatePreview,
) -> Vec<String> {
    let mut next_required_actions = candidate_preview.next_required_actions.clone();
    match status {
        ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked => {
            push_unique_text(
                &mut next_required_actions,
                "Resolve grounded-answer candidate blockers before any GroundedAnswer write implementation.",
            );
        }
        ScholarChatGroundedAnswerWriteEligibilityStatus::NeedsReview => {
            push_unique_text(
                &mut next_required_actions,
                "Review weakly supported and unsupported draft items before any GroundedAnswer write implementation.",
            );
        }
        ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater => {
            push_unique_text(
                &mut next_required_actions,
                "A GroundedAnswer write implementation can be added later without changing this preview.",
            );
        }
    }
    next_required_actions
}

fn grounded_answer_write_eligibility_preview_from_candidate_preview(
    candidate_preview: ScholarChatGroundedAnswerCandidatePreview,
) -> ScholarChatGroundedAnswerWriteEligibilityPreview {
    let status = grounded_answer_write_eligibility_status(&candidate_preview);
    let mut blockers = candidate_preview.blockers.clone();
    let mut warnings = candidate_preview.warnings.clone();

    push_grounding_inspection_warning(
        &mut warnings,
        "boundary",
        "This is a grounded-answer write-eligibility preview only; it is not an AnswerDraft, GroundedAnswer, FinalAnswer, Evidence Pack, or persisted artifact.",
    );

    match status {
        ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked => {
            if blockers.is_empty() {
                push_grounding_inspection_blocker(
                    &mut blockers,
                    "write_eligibility_blocked",
                    "Grounded-answer write eligibility is blocked until grounded-answer candidate preview is available.",
                );
            }
            if candidate_preview.candidate_statement_count == 0 {
                push_grounding_inspection_blocker(
                    &mut blockers,
                    "candidate_statements_missing",
                    "No candidate statements were available for write eligibility.",
                );
            }
        }
        ScholarChatGroundedAnswerWriteEligibilityStatus::NeedsReview => {
            push_grounding_inspection_warning(
                &mut warnings,
                "needs_review",
                "Weakly supported or unsupported draft items remain and need review before a future GroundedAnswer write is added.",
            );
        }
        ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater => {
            push_grounding_inspection_warning(
                &mut warnings,
                "write_eligible_later",
                "All inspected items were supported by local evidence. This is still only a write-eligibility preview.",
            );
        }
    }

    let summary = grounded_answer_write_eligibility_summary(&status, &candidate_preview);
    let eligibility_reasons = grounded_answer_write_eligibility_reasons(&status, &candidate_preview);
    let next_required_actions =
        grounded_answer_write_eligibility_next_required_actions(&status, &candidate_preview);

    ScholarChatGroundedAnswerWriteEligibilityPreview {
        status,
        candidate_status: candidate_preview.status,
        normalized_prompt: candidate_preview.normalized_prompt,
        selected_source_count: candidate_preview.selected_source_count,
        evidence_candidate_count: candidate_preview.evidence_candidate_count,
        inspected_item_count: candidate_preview.inspected_item_count,
        supported_item_count: candidate_preview.supported_item_count,
        weakly_supported_item_count: candidate_preview.weakly_supported_item_count,
        unsupported_item_count: candidate_preview.unsupported_item_count,
        candidate_statement_count: candidate_preview.candidate_statement_count,
        eligibility_reasons,
        blockers,
        warnings,
        next_required_actions,
        summary,
        preview_only: true,
        not_answer_draft: true,
        not_grounded_answer: true,
        not_final_answer: true,
        no_answer_artifact_created: true,
        no_evidence_pack_built: true,
        no_persistence: true,
        no_llm_call: true,
        no_runtime_execution: true,
        no_registry_status_change: true,
        no_audit_write: true,
    }
}

pub fn preview_scholar_chat_grounded_answer_build_intent(
    root: impl Into<PathBuf>,
    request: ScholarChatGroundedAnswerBuildIntentRequest,
) -> AegisResult<ScholarChatGroundedAnswerBuildIntentPreview> {
    let root = root.into();
    let normalized_prompt = normalized_prompt_or_err(request.grounding_request.scholar_chat_request.prompt.clone())?;
    let normalized_answer_draft_id = normalize_optional_answer_draft_id(request.answer_draft_id)?;
    let write_eligibility_preview =
        preview_scholar_chat_grounded_answer_write_eligibility(&root, request.grounding_request.clone())?;
    Ok(grounded_answer_build_intent_preview_from_write_eligibility_preview(
        write_eligibility_preview,
        normalized_prompt,
        normalized_answer_draft_id,
        request.explicit_user_intent,
    ))
}

pub fn preview_scholar_chat_grounded_answer_build_request(
    root: impl Into<PathBuf>,
    request: ScholarChatGroundedAnswerBuildRequestPreviewRequest,
) -> AegisResult<ScholarChatGroundedAnswerBuildRequestPreview> {
    let root = root.into();
    let normalized_prompt = normalized_prompt_or_err(
        request
            .build_intent_request
            .grounding_request
            .scholar_chat_request
            .prompt
            .clone(),
    )?;
    let normalized_answer_draft_id =
        normalize_optional_answer_draft_id(request.build_intent_request.answer_draft_id.clone())?;
    let (normalized_selected_source_ids, _selected_source_count) = normalize_selected_source_ids(
        request
            .build_intent_request
            .grounding_request
            .scholar_chat_request
            .selected_source_ids
            .clone(),
    )?;
    let normalized_build_intent_request = ScholarChatGroundedAnswerBuildIntentRequest {
        grounding_request: ScholarChatDraftGroundingInspectionRequest {
            scholar_chat_request: ScholarChatRequest {
                prompt: normalized_prompt.clone(),
                mode: request
                    .build_intent_request
                    .grounding_request
                    .scholar_chat_request
                    .mode,
                grounding_policy: request
                    .build_intent_request
                    .grounding_request
                    .scholar_chat_request
                    .grounding_policy,
                selected_source_ids: normalized_selected_source_ids.clone(),
            },
            draft_text: request.build_intent_request.grounding_request.draft_text.clone(),
            max_items: request.build_intent_request.grounding_request.max_items,
        },
        answer_draft_id: normalized_answer_draft_id.clone(),
        explicit_user_intent: request.build_intent_request.explicit_user_intent,
    };
    let build_intent_preview =
        preview_scholar_chat_grounded_answer_build_intent(&root, normalized_build_intent_request)?;
    Ok(grounded_answer_build_request_preview_from_build_intent_preview(
        build_intent_preview,
        normalized_prompt,
        normalized_answer_draft_id,
        normalized_selected_source_ids,
    ))
}

pub fn preview_scholar_chat_grounded_answer_build_preflight(
    root: impl Into<PathBuf>,
    request: ScholarChatGroundedAnswerBuildPreflightPreviewRequest,
) -> AegisResult<ScholarChatGroundedAnswerBuildPreflightPreview> {
    let root = root.into();
    let build_request_preview =
        preview_scholar_chat_grounded_answer_build_request(&root, request.build_request_preview_request)?;
    let normalized_prompt = build_request_preview.normalized_prompt.clone();
    let selected_source_ids = build_request_preview.selected_source_ids.clone();
    let answer_draft_id = build_request_preview.answer_draft_id.clone();
    let answer_draft_id_present = answer_draft_id.is_some();
    let mut answer_draft_present = false;
    let mut answer_draft_readable = false;
    let mut answer_draft_claim_count = 0usize;
    let mut blockers = build_request_preview.blockers.clone();
    let mut warnings = build_request_preview.warnings.clone();

    push_grounding_inspection_warning(
        &mut warnings,
        "boundary",
        "This is a grounded-answer build preflight preview only; it is not an AnswerDraft, GroundedAnswer, FinalAnswer, Evidence Pack, or persisted artifact.",
    );

    let status = match build_request_preview.status {
        ScholarChatGroundedAnswerBuildRequestStatus::Blocked => ScholarChatGroundedAnswerBuildPreflightStatus::Blocked,
        ScholarChatGroundedAnswerBuildRequestStatus::NeedsReview => ScholarChatGroundedAnswerBuildPreflightStatus::NeedsReview,
        ScholarChatGroundedAnswerBuildRequestStatus::RequestReadyLater => {
            if let Some(answer_draft_id) = answer_draft_id.as_ref() {
                let corpus_paths = CorpusPaths::new(root.clone());
                let registry = SourceRegistry::load(&corpus_paths.registry_path());
                if let Ok(registry) = registry {
                    let mut answer_draft_missing = true;
                    for lookup_source_id in &selected_source_ids {
                        let record = match registry.get_source(lookup_source_id) {
                            Ok(record) => record,
                            Err(_) => continue,
                        };
                        let draft_path = corpus_paths
                            .source_version_dir(&record.source_id, &record.version_id)
                            .join("answer_drafts")
                            .join(format!("{answer_draft_id}.json"));
                        if !draft_path.exists() {
                            continue;
                        }
                        answer_draft_missing = false;
                        answer_draft_present = true;
                        match fs::read_to_string(&draft_path)
                            .ok()
                            .and_then(|content| serde_json::from_str::<crate::answer_draft::AnswerDraft>(&content).ok())
                        {
                            Some(answer_draft) => {
                                answer_draft_readable = true;
                                answer_draft_claim_count = answer_draft.claim_count;
                                break;
                            }
                            None => {
                                push_grounding_inspection_blocker(
                                    &mut blockers,
                                    "answer_draft_unreadable",
                                    "The referenced AnswerDraft is unreadable.",
                                );
                                break;
                            }
                        }
                    }
                    if answer_draft_missing {
                        push_grounding_inspection_blocker(
                            &mut blockers,
                            "answer_draft_missing",
                            "The referenced AnswerDraft is missing.",
                        );
                    }
                } else {
                    push_grounding_inspection_blocker(
                        &mut blockers,
                        "answer_draft_missing",
                        "The referenced AnswerDraft is missing.",
                    );
                }
                if answer_draft_readable {
                    ScholarChatGroundedAnswerBuildPreflightStatus::PreflightReadyLater
                } else {
                    if answer_draft_present && !blockers.iter().any(|blocker| blocker.kind == "answer_draft_unreadable") {
                        push_grounding_inspection_blocker(
                            &mut blockers,
                            "answer_draft_unreadable",
                            "The referenced AnswerDraft is unreadable.",
                        );
                    }
                    ScholarChatGroundedAnswerBuildPreflightStatus::Blocked
                }
            } else {
                push_grounding_inspection_blocker(
                    &mut blockers,
                    "answer_draft_id_missing",
                    "No answer draft ID was provided.",
                );
                ScholarChatGroundedAnswerBuildPreflightStatus::Blocked
            }
        }
    };

    if matches!(status, ScholarChatGroundedAnswerBuildPreflightStatus::NeedsReview) {
        push_grounding_inspection_warning(
            &mut warnings,
            "needs_review",
            "The build request still needs review before AnswerDraft preflight can be accepted.",
        );
    }
    if matches!(status, ScholarChatGroundedAnswerBuildPreflightStatus::PreflightReadyLater) {
        push_grounding_inspection_warning(
            &mut warnings,
            "preflight_ready_later",
            "The referenced AnswerDraft is readable. This is still only a preflight preview.",
        );
    }

    let required_inputs = grounded_answer_build_preflight_required_inputs();
    let missing_inputs = grounded_answer_build_preflight_missing_inputs(
        &build_request_preview,
        answer_draft_id_present,
        answer_draft_id.as_ref(),
        answer_draft_present,
        answer_draft_readable,
        &status,
    );
    let preflight_reasons = grounded_answer_build_preflight_reasons(
        &build_request_preview,
        answer_draft_id_present,
        answer_draft_present,
        answer_draft_readable,
        answer_draft_claim_count,
        &status,
    );
    let next_required_actions = grounded_answer_build_preflight_next_required_actions(
        &status,
        &build_request_preview,
        answer_draft_id_present,
        answer_draft_present,
        answer_draft_readable,
    );
    let summary = grounded_answer_build_preflight_summary(
        &status,
        &build_request_preview,
        answer_draft_id_present,
        answer_draft_present,
        answer_draft_readable,
        answer_draft_claim_count,
    );

    Ok(ScholarChatGroundedAnswerBuildPreflightPreview {
        status,
        build_request_status: build_request_preview.status,
        build_intent_status: build_request_preview.build_intent_status,
        write_eligibility_status: build_request_preview.write_eligibility_status,
        candidate_status: build_request_preview.candidate_status,
        normalized_prompt,
        selected_source_count: build_request_preview.selected_source_count,
        evidence_candidate_count: build_request_preview.evidence_candidate_count,
        inspected_item_count: build_request_preview.inspected_item_count,
        supported_item_count: build_request_preview.supported_item_count,
        weakly_supported_item_count: build_request_preview.weakly_supported_item_count,
        unsupported_item_count: build_request_preview.unsupported_item_count,
        candidate_statement_count: build_request_preview.candidate_statement_count,
        answer_draft_id,
        selected_source_ids,
        answer_draft_present,
        answer_draft_readable,
        answer_draft_claim_count,
        required_inputs,
        missing_inputs,
        preflight_reasons,
        blockers,
        warnings,
        next_required_actions,
        summary,
        preview_only: true,
        not_answer_draft: true,
        not_grounded_answer: true,
        not_final_answer: true,
        no_answer_artifact_created: true,
        no_evidence_pack_built: true,
        no_persistence: true,
        no_llm_call: true,
        no_runtime_execution: true,
        no_registry_status_change: true,
        no_audit_write: true,
        no_grounded_answer_service_call: true,
        no_grounded_answer_write: true,
    })
}

pub fn preview_scholar_chat_grounded_answer_execution_readiness(
    root: impl Into<PathBuf>,
    request: ScholarChatGroundedAnswerExecutionReadinessPreviewRequest,
) -> AegisResult<ScholarChatGroundedAnswerExecutionReadinessPreview> {
    let root = root.into();
    let build_preflight_preview =
        preview_scholar_chat_grounded_answer_build_preflight(&root, request.build_preflight_preview_request)?;
    let normalized_prompt = build_preflight_preview.normalized_prompt.clone();
    let answer_draft_id = build_preflight_preview.answer_draft_id.clone();
    let selected_source_ids = build_preflight_preview.selected_source_ids.clone();
    let answer_draft_present = build_preflight_preview.answer_draft_present;
    let answer_draft_readable = build_preflight_preview.answer_draft_readable;
    let answer_draft_claim_count = build_preflight_preview.answer_draft_claim_count;
    let build_preflight_status = build_preflight_preview.status.clone();
    let mut blockers = build_preflight_preview.blockers.clone();
    let mut warnings = build_preflight_preview.warnings.clone();

    push_grounding_inspection_warning(
        &mut warnings,
        "boundary",
        "This is an execution-readiness preview only; it is not an AnswerDraft, GroundedAnswer, FinalAnswer, Evidence Pack, or persisted artifact.",
    );
    if !request.execution_consent {
        push_grounding_inspection_blocker(
            &mut blockers,
            "execution_consent_missing",
            "Execution consent was not given.",
        );
    }

    let status = match build_preflight_status {
        ScholarChatGroundedAnswerBuildPreflightStatus::Blocked => ScholarChatGroundedAnswerExecutionReadinessStatus::Blocked,
        ScholarChatGroundedAnswerBuildPreflightStatus::NeedsReview => ScholarChatGroundedAnswerExecutionReadinessStatus::NeedsReview,
        ScholarChatGroundedAnswerBuildPreflightStatus::PreflightReadyLater => {
            if request.execution_consent {
                ScholarChatGroundedAnswerExecutionReadinessStatus::ExecutionReadyLater
            } else {
                ScholarChatGroundedAnswerExecutionReadinessStatus::Blocked
            }
        }
    };

    if matches!(status, ScholarChatGroundedAnswerExecutionReadinessStatus::NeedsReview) {
        push_grounding_inspection_warning(
            &mut warnings,
            "needs_review",
            "The grounded-answer build preflight still needs review before execution readiness can be accepted.",
        );
    }
    if matches!(status, ScholarChatGroundedAnswerExecutionReadinessStatus::ExecutionReadyLater) {
        push_grounding_inspection_warning(
            &mut warnings,
            "execution_ready_later",
            "Execution consent is given and the build preflight is ready later. This is still only a preview.",
        );
    }

    let required_inputs = grounded_answer_execution_readiness_required_inputs();
    let missing_inputs = grounded_answer_execution_readiness_missing_inputs(
        &build_preflight_status,
        answer_draft_readable,
        request.execution_consent,
    );
    let readiness_reasons = grounded_answer_execution_readiness_reasons(
        &build_preflight_preview,
        request.execution_consent,
        &status,
    );
    let next_required_actions = grounded_answer_execution_readiness_next_required_actions(
        &status,
        &build_preflight_preview,
        request.execution_consent,
    );
    let summary = grounded_answer_execution_readiness_summary(
        &status,
        &build_preflight_preview,
        request.execution_consent,
    );

    Ok(ScholarChatGroundedAnswerExecutionReadinessPreview {
        status,
        build_preflight_status,
        build_request_status: build_preflight_preview.build_request_status,
        build_intent_status: build_preflight_preview.build_intent_status,
        write_eligibility_status: build_preflight_preview.write_eligibility_status,
        candidate_status: build_preflight_preview.candidate_status,
        normalized_prompt,
        answer_draft_id,
        selected_source_ids,
        selected_source_count: build_preflight_preview.selected_source_count,
        evidence_candidate_count: build_preflight_preview.evidence_candidate_count,
        inspected_item_count: build_preflight_preview.inspected_item_count,
        supported_item_count: build_preflight_preview.supported_item_count,
        weakly_supported_item_count: build_preflight_preview.weakly_supported_item_count,
        unsupported_item_count: build_preflight_preview.unsupported_item_count,
        candidate_statement_count: build_preflight_preview.candidate_statement_count,
        answer_draft_present,
        answer_draft_readable,
        answer_draft_claim_count,
        execution_consent: request.execution_consent,
        required_inputs,
        missing_inputs,
        readiness_reasons,
        blockers,
        warnings,
        next_required_actions,
        summary,
        preview_only: true,
        not_answer_draft: true,
        not_grounded_answer: true,
        not_final_answer: true,
        no_answer_artifact_created: true,
        no_evidence_pack_built: true,
        no_persistence: true,
        no_llm_call: true,
        no_runtime_execution: true,
        no_registry_status_change: true,
        no_audit_write: true,
        no_grounded_answer_service_call: true,
        no_grounded_answer_write: true,
    })
}

fn grounded_answer_build_preflight_required_inputs() -> Vec<String> {
    vec![
        "build_request_ready_later".to_string(),
        "answer_draft_id".to_string(),
        "answer_draft_present".to_string(),
        "answer_draft_readable".to_string(),
    ]
}

fn grounded_answer_build_preflight_missing_inputs(
    build_request_preview: &ScholarChatGroundedAnswerBuildRequestPreview,
    answer_draft_id_present: bool,
    answer_draft_id: Option<&String>,
    answer_draft_present: bool,
    answer_draft_readable: bool,
    status: &ScholarChatGroundedAnswerBuildPreflightStatus,
) -> Vec<String> {
    let mut missing_inputs = Vec::new();
    if !matches!(
        build_request_preview.status,
        ScholarChatGroundedAnswerBuildRequestStatus::RequestReadyLater
    ) {
        missing_inputs.push("build_request_ready_later".to_string());
    } else if !matches!(status, ScholarChatGroundedAnswerBuildPreflightStatus::PreflightReadyLater) {
        if !answer_draft_id_present || answer_draft_id.is_none() {
            missing_inputs.push("answer_draft_id".to_string());
        } else if !answer_draft_present {
            missing_inputs.push("answer_draft_present".to_string());
            missing_inputs.push("answer_draft_readable".to_string());
        } else if !answer_draft_readable {
            missing_inputs.push("answer_draft_readable".to_string());
        }
    }
    missing_inputs
}

fn grounded_answer_build_preflight_summary(
    status: &ScholarChatGroundedAnswerBuildPreflightStatus,
    build_request_preview: &ScholarChatGroundedAnswerBuildRequestPreview,
    answer_draft_id_present: bool,
    answer_draft_present: bool,
    answer_draft_readable: bool,
    answer_draft_claim_count: usize,
) -> String {
    match status {
        ScholarChatGroundedAnswerBuildPreflightStatus::Blocked => {
            if !matches!(
                build_request_preview.status,
                ScholarChatGroundedAnswerBuildRequestStatus::RequestReadyLater
            ) {
                format!(
                    "Grounded-answer build preflight is blocked because the build request is {:?}.",
                    build_request_preview.status
                )
            } else if !answer_draft_id_present {
                "Grounded-answer build preflight is blocked until an answer draft ID is provided.".to_string()
            } else if !answer_draft_present {
                "Grounded-answer build preflight is blocked until a matching AnswerDraft artifact is found."
                    .to_string()
            } else if !answer_draft_readable {
                "Grounded-answer build preflight is blocked because the referenced AnswerDraft artifact is unreadable."
                    .to_string()
            } else {
                "Grounded-answer build preflight is blocked until the request is ready later.".to_string()
            }
        }
        ScholarChatGroundedAnswerBuildPreflightStatus::NeedsReview => {
            "Grounded-answer build preflight needs review because the build request still needs review."
                .to_string()
        }
        ScholarChatGroundedAnswerBuildPreflightStatus::PreflightReadyLater => format!(
            "The grounded-answer build preflight is ready later: the build request is ready later, the referenced AnswerDraft is readable, and it contains {} claim(s).",
            answer_draft_claim_count
        ),
    }
}

fn grounded_answer_build_preflight_reasons(
    build_request_preview: &ScholarChatGroundedAnswerBuildRequestPreview,
    answer_draft_id_present: bool,
    answer_draft_present: bool,
    answer_draft_readable: bool,
    answer_draft_claim_count: usize,
    status: &ScholarChatGroundedAnswerBuildPreflightStatus,
) -> Vec<String> {
    let mut reasons = vec![
        format!("Build request status: {:?}", build_request_preview.status),
        format!("Build intent status: {:?}", build_request_preview.build_intent_status),
        format!("Write eligibility status: {:?}", build_request_preview.write_eligibility_status),
        format!("Candidate status: {:?}", build_request_preview.candidate_status),
        format!("Answer draft ID present: {}", answer_draft_id_present),
        format!("Answer draft present: {}", answer_draft_present),
        format!("Answer draft readable: {}", answer_draft_readable),
        format!("Answer draft claim count: {}", answer_draft_claim_count),
    ];
    match status {
        ScholarChatGroundedAnswerBuildPreflightStatus::Blocked => {
            push_unique_text(
                &mut reasons,
                "The preflight is blocked until the build request is ready later and the referenced AnswerDraft artifact is readable.",
            );
        }
        ScholarChatGroundedAnswerBuildPreflightStatus::NeedsReview => {
            push_unique_text(
                &mut reasons,
                "The build request still needs review before the AnswerDraft preflight can be accepted.",
            );
        }
        ScholarChatGroundedAnswerBuildPreflightStatus::PreflightReadyLater => {
            push_unique_text(
                &mut reasons,
                "The referenced AnswerDraft artifact is readable and the preflight is ready later.",
            );
        }
    }
    reasons
}

fn grounded_answer_build_preflight_next_required_actions(
    status: &ScholarChatGroundedAnswerBuildPreflightStatus,
    build_request_preview: &ScholarChatGroundedAnswerBuildRequestPreview,
    answer_draft_id_present: bool,
    answer_draft_present: bool,
    answer_draft_readable: bool,
) -> Vec<String> {
    let mut next_required_actions = Vec::new();
    match status {
        ScholarChatGroundedAnswerBuildPreflightStatus::Blocked => {
            if !matches!(
                build_request_preview.status,
                ScholarChatGroundedAnswerBuildRequestStatus::RequestReadyLater
            ) {
                push_unique_text(
                    &mut next_required_actions,
                    "Bring the grounded-answer build request to request_ready_later first.",
                );
            } else if !answer_draft_id_present {
                push_unique_text(
                    &mut next_required_actions,
                    "Provide an answer draft ID before a future grounded-answer build request can proceed.",
                );
            } else if !answer_draft_present {
                push_unique_text(
                    &mut next_required_actions,
                    "Locate or create the matching AnswerDraft artifact before a future grounded-answer build request can proceed.",
                );
            } else if !answer_draft_readable {
                push_unique_text(
                    &mut next_required_actions,
                    "Create or repair the referenced AnswerDraft artifact before a future grounded-answer build request can proceed.",
                );
            }
        }
        ScholarChatGroundedAnswerBuildPreflightStatus::NeedsReview => {
            push_unique_text(
                &mut next_required_actions,
                "Review weakly supported or unsupported items before checking preflight again.",
            );
        }
        ScholarChatGroundedAnswerBuildPreflightStatus::PreflightReadyLater => {
            push_unique_text(
                &mut next_required_actions,
                "A future GroundedAnswer service call can be added later without changing this preview.",
            );
        }
    }
    next_required_actions
}

fn grounded_answer_execution_readiness_required_inputs() -> Vec<String> {
    vec![
        "build_preflight_ready_later".to_string(),
        "answer_draft_readable".to_string(),
        "execution_consent".to_string(),
    ]
}

fn grounded_answer_execution_readiness_missing_inputs(
    build_preflight_status: &ScholarChatGroundedAnswerBuildPreflightStatus,
    answer_draft_readable: bool,
    execution_consent: bool,
) -> Vec<String> {
    let mut missing_inputs = Vec::new();
    if !matches!(
        build_preflight_status,
        ScholarChatGroundedAnswerBuildPreflightStatus::PreflightReadyLater
    ) {
        missing_inputs.push("build_preflight_ready_later".to_string());
    }
    if !answer_draft_readable {
        missing_inputs.push("answer_draft_readable".to_string());
    }
    if !execution_consent {
        missing_inputs.push("execution_consent".to_string());
    }
    missing_inputs
}

fn grounded_answer_execution_readiness_summary(
    status: &ScholarChatGroundedAnswerExecutionReadinessStatus,
    build_preflight_preview: &ScholarChatGroundedAnswerBuildPreflightPreview,
    execution_consent: bool,
) -> String {
    match status {
        ScholarChatGroundedAnswerExecutionReadinessStatus::Blocked => {
            if !matches!(
                build_preflight_preview.status,
                ScholarChatGroundedAnswerBuildPreflightStatus::PreflightReadyLater
            ) {
                format!(
                    "Execution-readiness preview is blocked because the grounded-answer build preflight is {:?}.",
                    build_preflight_preview.status
                )
            } else if !build_preflight_preview.answer_draft_readable {
                "Execution-readiness preview is blocked because the referenced AnswerDraft artifact is unreadable."
                    .to_string()
            } else if !execution_consent {
                "Execution-readiness preview is blocked until execution consent is given.".to_string()
            } else {
                "Execution-readiness preview is blocked until the request is ready later.".to_string()
            }
        }
        ScholarChatGroundedAnswerExecutionReadinessStatus::NeedsReview => {
            "Execution-readiness preview needs review because the grounded-answer build preflight still needs review."
                .to_string()
        }
        ScholarChatGroundedAnswerExecutionReadinessStatus::ExecutionReadyLater => {
            "The execution-readiness preview is ready later: the build preflight is ready later, the referenced AnswerDraft is readable, and execution consent is true."
                .to_string()
        }
    }
}

fn grounded_answer_execution_readiness_reasons(
    build_preflight_preview: &ScholarChatGroundedAnswerBuildPreflightPreview,
    execution_consent: bool,
    status: &ScholarChatGroundedAnswerExecutionReadinessStatus,
) -> Vec<String> {
    let mut reasons = vec![
        format!("Build preflight status: {:?}", build_preflight_preview.status),
        format!("Build request status: {:?}", build_preflight_preview.build_request_status),
        format!("Build intent status: {:?}", build_preflight_preview.build_intent_status),
        format!("Write eligibility status: {:?}", build_preflight_preview.write_eligibility_status),
        format!("Candidate status: {:?}", build_preflight_preview.candidate_status),
        format!("Answer draft present: {}", build_preflight_preview.answer_draft_present),
        format!("Answer draft readable: {}", build_preflight_preview.answer_draft_readable),
        format!("Answer draft claim count: {}", build_preflight_preview.answer_draft_claim_count),
        format!("Execution consent: {}", execution_consent),
    ];
    match status {
        ScholarChatGroundedAnswerExecutionReadinessStatus::Blocked => {
            push_unique_text(
                &mut reasons,
                "The execution-readiness preview is blocked until the build preflight is ready later, the referenced AnswerDraft is readable, and execution consent is given.",
            );
        }
        ScholarChatGroundedAnswerExecutionReadinessStatus::NeedsReview => {
            push_unique_text(
                &mut reasons,
                "The build preflight still needs review before execution readiness can be accepted.",
            );
        }
        ScholarChatGroundedAnswerExecutionReadinessStatus::ExecutionReadyLater => {
            push_unique_text(
                &mut reasons,
                "The referenced AnswerDraft is readable, execution consent is true, and the execution-readiness preview is ready later.",
            );
        }
    }
    reasons
}

fn grounded_answer_execution_readiness_next_required_actions(
    status: &ScholarChatGroundedAnswerExecutionReadinessStatus,
    build_preflight_preview: &ScholarChatGroundedAnswerBuildPreflightPreview,
    execution_consent: bool,
) -> Vec<String> {
    let mut next_required_actions = Vec::new();
    match status {
        ScholarChatGroundedAnswerExecutionReadinessStatus::Blocked => {
            if !matches!(
                build_preflight_preview.status,
                ScholarChatGroundedAnswerBuildPreflightStatus::PreflightReadyLater
            ) {
                push_unique_text(
                    &mut next_required_actions,
                    "Bring the grounded-answer build preflight to preflight_ready_later first.",
                );
            }
            if !build_preflight_preview.answer_draft_readable {
                push_unique_text(
                    &mut next_required_actions,
                    "Create or repair the referenced AnswerDraft artifact before execution readiness can proceed.",
                );
            }
            if !execution_consent {
                push_unique_text(
                    &mut next_required_actions,
                    "Confirm execution consent before a future grounded-answer build can proceed.",
                );
            }
        }
        ScholarChatGroundedAnswerExecutionReadinessStatus::NeedsReview => {
            push_unique_text(
                &mut next_required_actions,
                "Review the grounded-answer build preflight before checking execution readiness again.",
            );
        }
        ScholarChatGroundedAnswerExecutionReadinessStatus::ExecutionReadyLater => {
            push_unique_text(
                &mut next_required_actions,
                "A future GroundedAnswer service call can be added later when execution is enabled.",
            );
        }
    }
    next_required_actions
}

fn normalize_optional_draft_text(draft_text: Option<String>) -> Option<String> {
    draft_text
        .map(|text| text.trim().to_string())
        .and_then(|text| if text.is_empty() { None } else { Some(text) })
}

fn normalize_optional_answer_draft_id(answer_draft_id: Option<String>) -> AegisResult<Option<String>> {
    match answer_draft_id {
        None => Ok(None),
        Some(answer_draft_id) => {
            let normalized_answer_draft_id = answer_draft_id.trim().to_string();
            if normalized_answer_draft_id.is_empty() {
                Ok(None)
            } else {
                validate_answer_draft_id(&normalized_answer_draft_id)?;
                Ok(Some(normalized_answer_draft_id))
            }
        }
    }
}

fn grounded_answer_build_intent_status(
    write_eligibility_preview: &ScholarChatGroundedAnswerWriteEligibilityPreview,
    explicit_user_intent: bool,
    answer_draft_id_present: bool,
) -> ScholarChatGroundedAnswerBuildIntentStatus {
    match write_eligibility_preview.status {
        ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked => ScholarChatGroundedAnswerBuildIntentStatus::Blocked,
        ScholarChatGroundedAnswerWriteEligibilityStatus::NeedsReview => {
            ScholarChatGroundedAnswerBuildIntentStatus::NeedsReview
        }
        ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater => {
            if explicit_user_intent && answer_draft_id_present {
                ScholarChatGroundedAnswerBuildIntentStatus::IntentReadyLater
            } else {
                ScholarChatGroundedAnswerBuildIntentStatus::Blocked
            }
        }
    }
}

fn grounded_answer_build_intent_required_inputs() -> Vec<String> {
    vec![
        "write_eligible_later".to_string(),
        "explicit_user_intent".to_string(),
        "answer_draft_id".to_string(),
    ]
}

fn grounded_answer_build_intent_missing_inputs(
    write_eligibility_preview: &ScholarChatGroundedAnswerWriteEligibilityPreview,
    explicit_user_intent: bool,
    answer_draft_id_present: bool,
) -> Vec<String> {
    let mut missing_inputs = Vec::new();
    if !matches!(
        write_eligibility_preview.status,
        ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater
    ) {
        missing_inputs.push("write_eligible_later".to_string());
    }
    if !explicit_user_intent {
        missing_inputs.push("explicit_user_intent".to_string());
    }
    if !answer_draft_id_present {
        missing_inputs.push("answer_draft_id".to_string());
    }
    missing_inputs
}

fn grounded_answer_build_intent_summary(
    status: &ScholarChatGroundedAnswerBuildIntentStatus,
    write_eligibility_preview: &ScholarChatGroundedAnswerWriteEligibilityPreview,
    explicit_user_intent: bool,
    answer_draft_id_present: bool,
) -> String {
    match status {
        ScholarChatGroundedAnswerBuildIntentStatus::Blocked => match write_eligibility_preview.status {
            ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked => {
                "Grounded-answer build intent is blocked because grounded-answer write eligibility is blocked."
                    .to_string()
            }
            ScholarChatGroundedAnswerWriteEligibilityStatus::NeedsReview => {
                "Grounded-answer build intent is blocked because grounded-answer write eligibility still needs review."
                    .to_string()
            }
            ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater => {
                if !explicit_user_intent && !answer_draft_id_present {
                    "Grounded-answer build intent is blocked until explicit user intent and an answer draft ID are provided."
                        .to_string()
                } else if !explicit_user_intent {
                    "Grounded-answer build intent is blocked until explicit user intent is provided."
                        .to_string()
                } else {
                    "Grounded-answer build intent is blocked until an answer draft ID is provided."
                        .to_string()
                }
            }
        },
        ScholarChatGroundedAnswerBuildIntentStatus::NeedsReview => {
            "The draft is not yet ready for a grounded-answer build intent because weakly supported or unsupported items remain."
                .to_string()
        }
        ScholarChatGroundedAnswerBuildIntentStatus::IntentReadyLater => {
            "All inputs are present. A future user-confirmed GroundedAnswer build intent can be accepted later."
                .to_string()
        }
    }
}

fn grounded_answer_build_intent_reasons(
    status: &ScholarChatGroundedAnswerBuildIntentStatus,
    write_eligibility_preview: &ScholarChatGroundedAnswerWriteEligibilityPreview,
    explicit_user_intent: bool,
    answer_draft_id_present: bool,
) -> Vec<String> {
    let mut reasons = vec![
        format!("Write eligibility status: {:?}", write_eligibility_preview.status),
        format!("Candidate status: {:?}", write_eligibility_preview.candidate_status),
        format!("Explicit user intent: {}", explicit_user_intent),
        format!("Answer draft ID provided: {}", answer_draft_id_present),
    ];
    match status {
        ScholarChatGroundedAnswerBuildIntentStatus::Blocked => match write_eligibility_preview.status {
            ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked => {
                reasons.push("Grounded-answer write eligibility is blocked.".to_string());
            }
            ScholarChatGroundedAnswerWriteEligibilityStatus::NeedsReview => {
                reasons.push("Grounded-answer write eligibility still needs review.".to_string());
            }
            ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater => {
                if !explicit_user_intent && !answer_draft_id_present {
                    reasons.push("Explicit user intent and answer draft ID are both missing.".to_string());
                } else if !explicit_user_intent {
                    reasons.push("Explicit user intent is missing.".to_string());
                } else {
                    reasons.push("Answer draft ID is missing.".to_string());
                }
            }
        },
        ScholarChatGroundedAnswerBuildIntentStatus::NeedsReview => {
            reasons.push("Weakly supported or unsupported draft items remain.".to_string());
        }
        ScholarChatGroundedAnswerBuildIntentStatus::IntentReadyLater => {
            reasons.push("All inspected items were supported by local evidence.".to_string());
            reasons.push(
                "A future user-confirmed GroundedAnswer build intent can be accepted later after an explicit implementation phase."
                    .to_string(),
            );
        }
    }
    reasons
}

fn grounded_answer_build_intent_next_required_actions(
    status: &ScholarChatGroundedAnswerBuildIntentStatus,
    write_eligibility_preview: &ScholarChatGroundedAnswerWriteEligibilityPreview,
    explicit_user_intent: bool,
    answer_draft_id_present: bool,
) -> Vec<String> {
    let mut next_required_actions = write_eligibility_preview.next_required_actions.clone();
    match status {
        ScholarChatGroundedAnswerBuildIntentStatus::Blocked => {
            if !matches!(
                write_eligibility_preview.status,
                ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater
            ) {
                push_unique_text(
                    &mut next_required_actions,
                    "Resolve grounded-answer write-eligibility blockers before any GroundedAnswer service call.",
                );
            }
            if !explicit_user_intent {
                push_unique_text(
                    &mut next_required_actions,
                    "Provide explicit user intent before any GroundedAnswer service call.",
                );
            }
            if !answer_draft_id_present {
                push_unique_text(
                    &mut next_required_actions,
                    "Provide an answer draft ID before any GroundedAnswer service call.",
                );
            }
        }
        ScholarChatGroundedAnswerBuildIntentStatus::NeedsReview => {
            push_unique_text(
                &mut next_required_actions,
                "Review weakly supported and unsupported draft items before any GroundedAnswer service call.",
            );
        }
        ScholarChatGroundedAnswerBuildIntentStatus::IntentReadyLater => {
            push_unique_text(
                &mut next_required_actions,
                "A future user-confirmed GroundedAnswer service call can be added later without changing this preview.",
            );
        }
    }
    next_required_actions
}

fn grounded_answer_build_intent_preview_from_write_eligibility_preview(
    write_eligibility_preview: ScholarChatGroundedAnswerWriteEligibilityPreview,
    normalized_prompt: String,
    normalized_answer_draft_id: Option<String>,
    explicit_user_intent: bool,
) -> ScholarChatGroundedAnswerBuildIntentPreview {
    let answer_draft_id_present = normalized_answer_draft_id.is_some();
    let status = grounded_answer_build_intent_status(
        &write_eligibility_preview,
        explicit_user_intent,
        answer_draft_id_present,
    );
    let mut blockers = write_eligibility_preview.blockers.clone();
    let mut warnings = write_eligibility_preview.warnings.clone();

    push_grounding_inspection_warning(
        &mut warnings,
        "boundary",
        "This is a grounded-answer build-intent preview only; it is not an AnswerDraft, GroundedAnswer, FinalAnswer, Evidence Pack, or persisted artifact.",
    );

    match status {
        ScholarChatGroundedAnswerBuildIntentStatus::Blocked => {
            if !matches!(
                write_eligibility_preview.status,
                ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater
            ) {
                push_grounding_inspection_blocker(
                    &mut blockers,
                    "write_eligibility_blocked",
                    "Grounded-answer build intent is blocked until grounded-answer write eligibility is available.",
                );
            }
            if !explicit_user_intent {
                push_grounding_inspection_blocker(
                    &mut blockers,
                    "explicit_user_intent_missing",
                    "Explicit user intent was not provided.",
                );
            }
            if !answer_draft_id_present {
                push_grounding_inspection_blocker(
                    &mut blockers,
                    "answer_draft_id_missing",
                    "No answer draft ID was provided.",
                );
            }
        }
        ScholarChatGroundedAnswerBuildIntentStatus::NeedsReview => {
            push_grounding_inspection_warning(
                &mut warnings,
                "needs_review",
                "Weakly supported or unsupported draft items remain and need review before a future GroundedAnswer build intent is added.",
            );
        }
        ScholarChatGroundedAnswerBuildIntentStatus::IntentReadyLater => {
            push_grounding_inspection_warning(
                &mut warnings,
                "intent_ready_later",
                "All inputs are present. This is still only a grounded-answer build-intent preview.",
            );
        }
    }

    let summary = grounded_answer_build_intent_summary(
        &status,
        &write_eligibility_preview,
        explicit_user_intent,
        answer_draft_id_present,
    );
    let required_inputs = grounded_answer_build_intent_required_inputs();
    let missing_inputs = grounded_answer_build_intent_missing_inputs(
        &write_eligibility_preview,
        explicit_user_intent,
        answer_draft_id_present,
    );
    let intent_reasons = grounded_answer_build_intent_reasons(
        &status,
        &write_eligibility_preview,
        explicit_user_intent,
        answer_draft_id_present,
    );
    let next_required_actions = grounded_answer_build_intent_next_required_actions(
        &status,
        &write_eligibility_preview,
        explicit_user_intent,
        answer_draft_id_present,
    );

    ScholarChatGroundedAnswerBuildIntentPreview {
        status,
        write_eligibility_status: write_eligibility_preview.status,
        candidate_status: write_eligibility_preview.candidate_status,
        normalized_prompt,
        selected_source_count: write_eligibility_preview.selected_source_count,
        evidence_candidate_count: write_eligibility_preview.evidence_candidate_count,
        inspected_item_count: write_eligibility_preview.inspected_item_count,
        supported_item_count: write_eligibility_preview.supported_item_count,
        weakly_supported_item_count: write_eligibility_preview.weakly_supported_item_count,
        unsupported_item_count: write_eligibility_preview.unsupported_item_count,
        candidate_statement_count: write_eligibility_preview.candidate_statement_count,
        required_inputs,
        missing_inputs,
        intent_reasons,
        blockers,
        warnings,
        next_required_actions,
        summary,
        preview_only: true,
        not_answer_draft: true,
        not_grounded_answer: true,
        not_final_answer: true,
        no_answer_artifact_created: true,
        no_evidence_pack_built: true,
        no_persistence: true,
        no_llm_call: true,
        no_runtime_execution: true,
        no_registry_status_change: true,
        no_audit_write: true,
        no_grounded_answer_service_call: true,
    }
}

fn grounded_answer_build_request_status(
    build_intent_preview: &ScholarChatGroundedAnswerBuildIntentPreview,
) -> ScholarChatGroundedAnswerBuildRequestStatus {
    match build_intent_preview.status {
        ScholarChatGroundedAnswerBuildIntentStatus::Blocked => ScholarChatGroundedAnswerBuildRequestStatus::Blocked,
        ScholarChatGroundedAnswerBuildIntentStatus::NeedsReview => {
            ScholarChatGroundedAnswerBuildRequestStatus::NeedsReview
        }
        ScholarChatGroundedAnswerBuildIntentStatus::IntentReadyLater => {
            ScholarChatGroundedAnswerBuildRequestStatus::RequestReadyLater
        }
    }
}

fn grounded_answer_build_request_required_inputs() -> Vec<String> {
    vec![
        "build_intent_ready_later".to_string(),
        "answer_draft_id".to_string(),
        "selected_source_ids".to_string(),
    ]
}

fn grounded_answer_build_request_missing_inputs(
    build_intent_preview: &ScholarChatGroundedAnswerBuildIntentPreview,
    answer_draft_id: &Option<String>,
    selected_source_ids: &[String],
) -> Vec<String> {
    let mut missing_inputs = Vec::new();
    if !matches!(
        build_intent_preview.status,
        ScholarChatGroundedAnswerBuildIntentStatus::IntentReadyLater
    ) {
        missing_inputs.push("build_intent_ready_later".to_string());
    }
    if answer_draft_id.is_none() {
        missing_inputs.push("answer_draft_id".to_string());
    }
    if selected_source_ids.is_empty() {
        missing_inputs.push("selected_source_ids".to_string());
    }
    missing_inputs
}

fn grounded_answer_build_request_summary(
    status: &ScholarChatGroundedAnswerBuildRequestStatus,
    build_intent_preview: &ScholarChatGroundedAnswerBuildIntentPreview,
    answer_draft_id: &Option<String>,
    selected_source_ids: &[String],
) -> String {
    let answer_draft_id_summary = if answer_draft_id.is_some() { "present" } else { "missing" };
    let selected_source_summary = if selected_source_ids.is_empty() {
        "no selected source IDs"
    } else {
        "selected source IDs are normalized and ready"
    };
    match status {
        ScholarChatGroundedAnswerBuildRequestStatus::Blocked => format!(
            "Grounded-answer build request is blocked because the build intent is {:?}; answer draft ID is {}; {}.",
            build_intent_preview.status,
            answer_draft_id_summary,
            selected_source_summary,
        ),
        ScholarChatGroundedAnswerBuildRequestStatus::NeedsReview => format!(
            "Grounded-answer build request still needs review because the build intent is {:?}; answer draft ID is {}; {}.",
            build_intent_preview.status,
            answer_draft_id_summary,
            selected_source_summary,
        ),
        ScholarChatGroundedAnswerBuildRequestStatus::RequestReadyLater => {
            "The grounded-answer build request is normalized and ready later for a future GroundedAnswer service call."
                .to_string()
        }
    }
}

fn grounded_answer_build_request_reasons(
    build_intent_preview: &ScholarChatGroundedAnswerBuildIntentPreview,
    answer_draft_id: &Option<String>,
    selected_source_ids: &[String],
) -> Vec<String> {
    let mut request_reasons = vec![
        format!("Build intent status: {:?}", build_intent_preview.status),
        format!("Write eligibility status: {:?}", build_intent_preview.write_eligibility_status),
        format!("Candidate status: {:?}", build_intent_preview.candidate_status),
        format!("Answer draft ID provided: {}", answer_draft_id.is_some()),
        format!("Selected source count: {}", selected_source_ids.len()),
    ];
    match build_intent_preview.status {
        ScholarChatGroundedAnswerBuildIntentStatus::Blocked => {
            request_reasons.push("Grounded-answer build intent must be ready later before a request can be accepted.".to_string());
        }
        ScholarChatGroundedAnswerBuildIntentStatus::NeedsReview => {
            request_reasons.push(
                "Weakly supported or unsupported draft items remain and need review before a future GroundedAnswer service call."
                    .to_string(),
            );
        }
        ScholarChatGroundedAnswerBuildIntentStatus::IntentReadyLater => {
            request_reasons.push(
                "The request fields are normalized and ready later for a future GroundedAnswer service call.".to_string(),
            );
        }
    }
    request_reasons
}

fn grounded_answer_build_request_next_required_actions(
    status: &ScholarChatGroundedAnswerBuildRequestStatus,
    build_intent_preview: &ScholarChatGroundedAnswerBuildIntentPreview,
    answer_draft_id: &Option<String>,
    selected_source_ids: &[String],
) -> Vec<String> {
    let mut next_required_actions = build_intent_preview.next_required_actions.clone();
    match status {
        ScholarChatGroundedAnswerBuildRequestStatus::Blocked => {
            if !matches!(
                build_intent_preview.status,
                ScholarChatGroundedAnswerBuildIntentStatus::IntentReadyLater
            ) {
                push_unique_text(
                    &mut next_required_actions,
                    "Resolve grounded-answer build-intent blockers before any GroundedAnswer service call.",
                );
            }
            if answer_draft_id.is_none() {
                push_unique_text(
                    &mut next_required_actions,
                    "Provide an answer draft ID before any GroundedAnswer service call.",
                );
            }
            if selected_source_ids.is_empty() {
                push_unique_text(
                    &mut next_required_actions,
                    "Select at least one source ID before any GroundedAnswer service call.",
                );
            }
        }
        ScholarChatGroundedAnswerBuildRequestStatus::NeedsReview => {
            push_unique_text(
                &mut next_required_actions,
                "Review the normalized request fields before any GroundedAnswer service call.",
            );
        }
        ScholarChatGroundedAnswerBuildRequestStatus::RequestReadyLater => {
            push_unique_text(
                &mut next_required_actions,
                "A future user-confirmed GroundedAnswer service call can be added later using the normalized request fields.",
            );
        }
    }
    next_required_actions
}

fn grounded_answer_build_request_preview_from_build_intent_preview(
    build_intent_preview: ScholarChatGroundedAnswerBuildIntentPreview,
    normalized_prompt: String,
    normalized_answer_draft_id: Option<String>,
    normalized_selected_source_ids: Vec<String>,
) -> ScholarChatGroundedAnswerBuildRequestPreview {
    let status = grounded_answer_build_request_status(&build_intent_preview);
    let mut blockers = build_intent_preview.blockers.clone();
    let mut warnings = build_intent_preview.warnings.clone();

    push_grounding_inspection_warning(
        &mut warnings,
        "boundary",
        "This is a grounded-answer build-request preview only; it is not an AnswerDraft, GroundedAnswer, FinalAnswer, Evidence Pack, or persisted artifact.",
    );

    match status {
        ScholarChatGroundedAnswerBuildRequestStatus::Blocked => {
            push_grounding_inspection_blocker(
                &mut blockers,
                "build_request_blocked",
                "Grounded-answer build request is blocked until the build intent is ready later.",
            );
        }
        ScholarChatGroundedAnswerBuildRequestStatus::NeedsReview => {
            push_grounding_inspection_warning(
                &mut warnings,
                "request_needs_review",
                "The normalized grounded-answer build request still needs review before any GroundedAnswer service call.",
            );
        }
        ScholarChatGroundedAnswerBuildRequestStatus::RequestReadyLater => {
            push_grounding_inspection_warning(
                &mut warnings,
                "request_ready_later",
                "The normalized grounded-answer build request is ready later for a future GroundedAnswer service call.",
            );
        }
    }

    let answer_draft_id = normalized_answer_draft_id;
    let selected_source_count = normalized_selected_source_ids.len();
    let required_inputs = grounded_answer_build_request_required_inputs();
    let missing_inputs = grounded_answer_build_request_missing_inputs(
        &build_intent_preview,
        &answer_draft_id,
        &normalized_selected_source_ids,
    );
    let request_reasons = grounded_answer_build_request_reasons(
        &build_intent_preview,
        &answer_draft_id,
        &normalized_selected_source_ids,
    );
    let next_required_actions = grounded_answer_build_request_next_required_actions(
        &status,
        &build_intent_preview,
        &answer_draft_id,
        &normalized_selected_source_ids,
    );
    let summary = grounded_answer_build_request_summary(
        &status,
        &build_intent_preview,
        &answer_draft_id,
        &normalized_selected_source_ids,
    );

    ScholarChatGroundedAnswerBuildRequestPreview {
        status,
        build_intent_status: build_intent_preview.status,
        write_eligibility_status: build_intent_preview.write_eligibility_status,
        candidate_status: build_intent_preview.candidate_status,
        normalized_prompt,
        selected_source_count,
        evidence_candidate_count: build_intent_preview.evidence_candidate_count,
        inspected_item_count: build_intent_preview.inspected_item_count,
        supported_item_count: build_intent_preview.supported_item_count,
        weakly_supported_item_count: build_intent_preview.weakly_supported_item_count,
        unsupported_item_count: build_intent_preview.unsupported_item_count,
        candidate_statement_count: build_intent_preview.candidate_statement_count,
        answer_draft_id,
        selected_source_ids: normalized_selected_source_ids,
        required_inputs,
        missing_inputs,
        request_reasons,
        blockers,
        warnings,
        next_required_actions,
        summary,
        preview_only: true,
        not_answer_draft: true,
        not_grounded_answer: true,
        not_final_answer: true,
        no_answer_artifact_created: true,
        no_evidence_pack_built: true,
        no_persistence: true,
        no_llm_call: true,
        no_runtime_execution: true,
        no_registry_status_change: true,
        no_audit_write: true,
        no_grounded_answer_service_call: true,
    }
}

fn push_grounding_inspection_warning(
    warnings: &mut Vec<ScholarChatDraftGroundingInspectionWarning>,
    kind: &str,
    message: &str,
) {
    if !warnings.iter().any(|warning| warning.kind == kind && warning.message == message) {
        warnings.push(ScholarChatDraftGroundingInspectionWarning {
            kind: kind.to_string(),
            message: message.to_string(),
        });
    }
}

fn push_unique_text(items: &mut Vec<String>, value: &str) {
    if !items.iter().any(|item| item == value) {
        items.push(value.to_string());
    }
}

fn push_grounding_inspection_blocker(
    blockers: &mut Vec<ScholarChatDraftGroundingInspectionBlocker>,
    kind: &str,
    message: &str,
) {
    if !blockers.iter().any(|blocker| blocker.kind == kind && blocker.message == message) {
        blockers.push(ScholarChatDraftGroundingInspectionBlocker {
            kind: kind.to_string(),
            message: message.to_string(),
        });
    }
}

fn inspect_draft_grounding_items(
    draft_text: &str,
    evidence_candidates: &[ScholarChatEvidenceCandidate],
    max_items: usize,
) -> DraftGroundingInspectionItems {
    let mut items = Vec::new();
    let mut supported_item_count = 0;
    let mut weakly_supported_item_count = 0;
    let mut unsupported_item_count = 0;
    let warnings = Vec::new();

    let segments = split_draft_text_into_segments(draft_text);
    let was_clamped = segments.len() > max_items;
    for (item_index, segment) in segments.into_iter().take(max_items).enumerate() {
        let normalized_segment = compact_text_preview(&segment, 180);
        let item_terms = inspection_terms(&normalized_segment);
        if item_terms.is_empty() {
            continue;
        }

        let mut matched_evidence = Vec::new();
        for candidate in evidence_candidates {
            let candidate_terms = inspection_terms(&candidate.preview)
                .into_iter()
                .chain(candidate.matched_terms.iter().flat_map(|term| inspection_terms(term).into_iter()))
                .collect::<BTreeSet<_>>();
            let overlap = item_terms.intersection(&candidate_terms).count();
            if overlap > 0 {
                matched_evidence.push((overlap, candidate));
            }
        }

        let support_status = if matched_evidence.is_empty() {
            unsupported_item_count += 1;
            ScholarChatDraftGroundingSupportStatus::Unsupported
        } else {
            let best_overlap = matched_evidence.iter().map(|(overlap, _)| *overlap).max().unwrap_or(0);
            if best_overlap >= 2 {
                supported_item_count += 1;
                ScholarChatDraftGroundingSupportStatus::SupportedByLocalEvidence
            } else {
                weakly_supported_item_count += 1;
                ScholarChatDraftGroundingSupportStatus::WeaklySupported
            }
        };

        let mut source_ids = BTreeSet::new();
        let mut locator_previews = BTreeSet::new();
        let matched_evidence_count = matched_evidence.len();
        for (_, candidate) in matched_evidence {
            source_ids.insert(candidate.source_id.clone());
            locator_previews.insert(locator_preview(&candidate.locator));
        }

        items.push(ScholarChatDraftGroundingInspectionItem {
            item_index,
            text_preview: normalized_segment,
            support_status,
            matched_evidence_count,
            source_ids: source_ids.into_iter().collect(),
            locator_previews: locator_previews.into_iter().collect(),
        });
    }

    DraftGroundingInspectionItems {
        items,
        supported_item_count,
        weakly_supported_item_count,
        unsupported_item_count,
        warnings,
        was_clamped,
    }
}

fn split_draft_text_into_segments(draft_text: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();

    for ch in draft_text.chars() {
        if matches!(ch, '.' | '!' | '?' | '\n' | '\r') {
            let trimmed = current.trim();
            if !trimmed.is_empty() {
                segments.push(trimmed.to_string());
            }
            current.clear();
        } else {
            current.push(ch);
        }
    }

    let trimmed = current.trim();
    if !trimmed.is_empty() {
        segments.push(trimmed.to_string());
    }

    if segments.is_empty() && !draft_text.trim().is_empty() {
        segments.push(draft_text.trim().to_string());
    }

    segments
}

fn compact_text_preview(text: &str, max_chars: usize) -> String {
    let compacted = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut preview = compacted.trim().to_string();
    if preview.chars().count() > max_chars {
        let mut truncated = String::new();
        for ch in preview.chars().take(max_chars.saturating_sub(1)) {
            truncated.push(ch);
        }
        truncated.push('…');
        preview = truncated;
    }
    preview
}

fn inspection_terms(text: &str) -> BTreeSet<String> {
    const MIN_MEANINGFUL_TERM_LEN: usize = 3;

    text.split(|ch: char| !ch.is_alphanumeric())
        .filter_map(|term| {
            let normalized = term.trim().to_lowercase();
            if normalized.is_empty() {
                None
            } else if is_stopword_for_draft_grounding_inspection(&normalized) {
                None
            } else if normalized.chars().all(|ch| ch.is_numeric()) {
                Some(normalized)
            } else if normalized.chars().count() < MIN_MEANINGFUL_TERM_LEN {
                None
            } else {
                Some(normalized)
            }
        })
        .collect()
}

fn is_stopword_for_draft_grounding_inspection(term: &str) -> bool {
    matches!(
        term,
        "the" | "and" | "or" | "of" | "to" | "in" | "a" | "an" | "is" | "are" | "was" | "were" | "with" | "for" | "on" | "by" | "as" | "at" | "from" | "this" | "that"
    )
}

fn locator_preview(locator: &CitationLocator) -> String {
    let mut parts = vec![format!("{:?}", locator.locator_type).to_lowercase()];
    if !locator.label.trim().is_empty() {
        parts.push(format!("label={}", locator.label.trim()));
    }
    if let Some(page) = locator.page {
        parts.push(format!("page={page}"));
    }
    if let Some(slide) = locator.slide {
        parts.push(format!("slide={slide}"));
    }
    if let Some(start) = locator.character_start {
        parts.push(format!("chars={start}-{}", locator.character_end.unwrap_or(start)));
    }
    parts.join(" | ")
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

    fn grounding_request(
        prompt: &str,
        draft_text: Option<&str>,
        selected_source_ids: Vec<String>,
    ) -> ScholarChatDraftGroundingInspectionRequest {
        ScholarChatDraftGroundingInspectionRequest {
            scholar_chat_request: ScholarChatRequest {
                prompt: prompt.to_string(),
                mode: ScholarChatMode::LectureLearning,
                grounding_policy: GroundingPolicy::LocalFirst,
                selected_source_ids,
            },
            draft_text: draft_text.map(|value| value.to_string()),
            max_items: Some(4),
        }
    }

    fn build_intent_request(
        prompt: &str,
        draft_text: Option<&str>,
        selected_source_ids: Vec<String>,
        answer_draft_id: Option<&str>,
        explicit_user_intent: bool,
    ) -> ScholarChatGroundedAnswerBuildIntentRequest {
        ScholarChatGroundedAnswerBuildIntentRequest {
            grounding_request: grounding_request(prompt, draft_text, selected_source_ids),
            answer_draft_id: answer_draft_id.map(|value| value.to_string()),
            explicit_user_intent,
        }
    }

    fn build_request_request(
        prompt: &str,
        draft_text: Option<&str>,
        selected_source_ids: Vec<String>,
        answer_draft_id: Option<&str>,
        explicit_user_intent: bool,
    ) -> ScholarChatGroundedAnswerBuildRequestPreviewRequest {
        ScholarChatGroundedAnswerBuildRequestPreviewRequest {
            build_intent_request: build_intent_request(
                prompt,
                draft_text,
                selected_source_ids,
                answer_draft_id,
                explicit_user_intent,
            ),
        }
    }

    fn build_preflight_request(
        prompt: &str,
        draft_text: Option<&str>,
        selected_source_ids: Vec<String>,
        answer_draft_id: Option<&str>,
        explicit_user_intent: bool,
    ) -> ScholarChatGroundedAnswerBuildPreflightPreviewRequest {
        ScholarChatGroundedAnswerBuildPreflightPreviewRequest {
            build_request_preview_request: build_request_request(
                prompt,
                draft_text,
                selected_source_ids,
                answer_draft_id,
                explicit_user_intent,
            ),
        }
    }

    fn execution_readiness_request(
        prompt: &str,
        draft_text: Option<&str>,
        selected_source_ids: Vec<String>,
        answer_draft_id: Option<&str>,
        explicit_user_intent: bool,
        execution_consent: bool,
    ) -> ScholarChatGroundedAnswerExecutionReadinessPreviewRequest {
        ScholarChatGroundedAnswerExecutionReadinessPreviewRequest {
            build_preflight_preview_request: build_preflight_request(
                prompt,
                draft_text,
                selected_source_ids,
                answer_draft_id,
                explicit_user_intent,
            ),
            execution_consent,
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

    fn build_readable_answer_draft_fixture(temp: &tempfile::TempDir) -> (String, String, String, usize) {
        let source_id = build_source_with_index(temp, "alpha beta gamma\nalpha beta delta\n");
        let evidence = crate::evidence::EvidenceService::new(temp.path())
            .build_evidence_pack(&source_id, "alpha grounded evidence", 4)
            .unwrap();
        let draft = crate::answer_draft::AnswerDraftService::new(temp.path())
            .build_answer_draft(&source_id, &evidence.evidence_pack_id)
            .unwrap();
        (source_id, draft.answer_draft_id, draft.version_id, draft.claim_count)
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

    fn count_entries_recursively(path: &std::path::Path) -> usize {
        fn inner(path: &std::path::Path, count: &mut usize) {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    *count += 1;
                    let entry_path = entry.path();
                    if entry_path.is_dir() {
                        inner(&entry_path, count);
                    }
                }
            }
        }

        let mut count = 0;
        inner(path, &mut count);
        count
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

    fn assert_draft_grounding_inspection_boundary_fields(
        preview: &ScholarChatDraftGroundingInspectionPreview,
    ) {
        assert!(preview.inspection_only);
        assert!(preview.not_grounded_answer);
        assert!(preview.not_final_answer);
        assert!(preview.no_evidence_pack_built);
        assert!(preview.no_answer_artifact_created);
        assert!(preview.no_persistence);
        assert!(preview.no_llm_call);
        assert!(preview.no_runtime_execution);
    }

    fn assert_grounded_draft_readiness_boundary_fields(
        preview: &ScholarChatGroundedDraftReadinessPreview,
    ) {
        assert!(preview.preview_only);
        assert!(preview.not_grounded_answer);
        assert!(preview.not_final_answer);
        assert!(preview.no_answer_artifact_created);
        assert!(preview.no_evidence_pack_built);
        assert!(preview.no_persistence);
        assert!(preview.no_llm_call);
        assert!(preview.no_runtime_execution);
    }

    fn assert_grounded_draft_readiness_deterministic_and_path_free(
        temp: &tempfile::TempDir,
        request: ScholarChatDraftGroundingInspectionRequest,
    ) -> ScholarChatGroundedDraftReadinessPreview {
        let before_entries = count_entries_recursively(temp.path());
        let first = preview_scholar_chat_grounded_draft_readiness(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_grounded_draft_readiness(temp.path(), request).unwrap();
        let after_entries = count_entries_recursively(temp.path());
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_grounded_draft_readiness_boundary_fields(preview);
        }
        first
    }

    fn assert_grounded_answer_build_plan_boundary_fields(
        preview: &ScholarChatGroundedAnswerBuildPlanPreview,
    ) {
        assert!(preview.preview_only);
        assert!(preview.not_answer_draft);
        assert!(preview.not_grounded_answer);
        assert!(preview.not_final_answer);
        assert!(preview.no_answer_artifact_created);
        assert!(preview.no_evidence_pack_built);
        assert!(preview.no_persistence);
        assert!(preview.no_llm_call);
        assert!(preview.no_runtime_execution);
    }

    fn assert_grounded_answer_candidate_boundary_fields(
        preview: &ScholarChatGroundedAnswerCandidatePreview,
    ) {
        assert!(preview.preview_only);
        assert!(preview.not_answer_draft);
        assert!(preview.not_grounded_answer);
        assert!(preview.not_final_answer);
        assert!(preview.no_answer_artifact_created);
        assert!(preview.no_evidence_pack_built);
        assert!(preview.no_persistence);
        assert!(preview.no_llm_call);
        assert!(preview.no_runtime_execution);
    }

    fn assert_grounded_answer_write_eligibility_boundary_fields(
        preview: &ScholarChatGroundedAnswerWriteEligibilityPreview,
    ) {
        assert!(preview.preview_only);
        assert!(preview.not_answer_draft);
        assert!(preview.not_grounded_answer);
        assert!(preview.not_final_answer);
        assert!(preview.no_answer_artifact_created);
        assert!(preview.no_evidence_pack_built);
        assert!(preview.no_persistence);
        assert!(preview.no_llm_call);
        assert!(preview.no_runtime_execution);
        assert!(preview.no_registry_status_change);
        assert!(preview.no_audit_write);
    }

    fn assert_grounded_answer_build_intent_boundary_fields(
        preview: &ScholarChatGroundedAnswerBuildIntentPreview,
    ) {
        assert!(preview.preview_only);
        assert!(preview.not_answer_draft);
        assert!(preview.not_grounded_answer);
        assert!(preview.not_final_answer);
        assert!(preview.no_answer_artifact_created);
        assert!(preview.no_evidence_pack_built);
        assert!(preview.no_persistence);
        assert!(preview.no_llm_call);
        assert!(preview.no_runtime_execution);
        assert!(preview.no_registry_status_change);
        assert!(preview.no_audit_write);
        assert!(preview.no_grounded_answer_service_call);
    }

    fn assert_grounded_answer_build_request_boundary_fields(
        preview: &ScholarChatGroundedAnswerBuildRequestPreview,
    ) {
        assert!(preview.preview_only);
        assert!(preview.not_answer_draft);
        assert!(preview.not_grounded_answer);
        assert!(preview.not_final_answer);
        assert!(preview.no_answer_artifact_created);
        assert!(preview.no_evidence_pack_built);
        assert!(preview.no_persistence);
        assert!(preview.no_llm_call);
        assert!(preview.no_runtime_execution);
        assert!(preview.no_registry_status_change);
        assert!(preview.no_audit_write);
        assert!(preview.no_grounded_answer_service_call);
    }

    fn assert_grounded_answer_build_preflight_boundary_fields(
        preview: &ScholarChatGroundedAnswerBuildPreflightPreview,
    ) {
        assert!(preview.preview_only);
        assert!(preview.not_answer_draft);
        assert!(preview.not_grounded_answer);
        assert!(preview.not_final_answer);
        assert!(preview.no_answer_artifact_created);
        assert!(preview.no_evidence_pack_built);
        assert!(preview.no_persistence);
        assert!(preview.no_llm_call);
        assert!(preview.no_runtime_execution);
        assert!(preview.no_registry_status_change);
        assert!(preview.no_audit_write);
        assert!(preview.no_grounded_answer_service_call);
        assert!(preview.no_grounded_answer_write);
    }

    fn assert_grounded_answer_execution_readiness_boundary_fields(
        preview: &ScholarChatGroundedAnswerExecutionReadinessPreview,
    ) {
        assert!(preview.preview_only);
        assert!(preview.not_answer_draft);
        assert!(preview.not_grounded_answer);
        assert!(preview.not_final_answer);
        assert!(preview.no_answer_artifact_created);
        assert!(preview.no_evidence_pack_built);
        assert!(preview.no_persistence);
        assert!(preview.no_llm_call);
        assert!(preview.no_runtime_execution);
        assert!(preview.no_registry_status_change);
        assert!(preview.no_audit_write);
        assert!(preview.no_grounded_answer_service_call);
        assert!(preview.no_grounded_answer_write);
    }

    fn assert_grounded_answer_build_plan_deterministic_and_path_free(
        temp: &tempfile::TempDir,
        request: ScholarChatDraftGroundingInspectionRequest,
    ) -> ScholarChatGroundedAnswerBuildPlanPreview {
        let before_entries = count_entries_recursively(temp.path());
        let first = preview_scholar_chat_grounded_answer_build_plan(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_grounded_answer_build_plan(temp.path(), request).unwrap();
        let after_entries = count_entries_recursively(temp.path());
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_grounded_answer_build_plan_boundary_fields(preview);
        }
        first
    }

    fn assert_grounded_answer_candidate_deterministic_and_path_free(
        temp: &tempfile::TempDir,
        request: ScholarChatDraftGroundingInspectionRequest,
    ) -> ScholarChatGroundedAnswerCandidatePreview {
        let before_entries = count_entries_recursively(temp.path());
        let first = preview_scholar_chat_grounded_answer_candidate(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_grounded_answer_candidate(temp.path(), request).unwrap();
        let after_entries = count_entries_recursively(temp.path());
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_grounded_answer_candidate_boundary_fields(preview);
        }
        first
    }

    fn assert_grounded_answer_write_eligibility_deterministic_and_path_free(
        temp: &tempfile::TempDir,
        request: ScholarChatDraftGroundingInspectionRequest,
    ) -> ScholarChatGroundedAnswerWriteEligibilityPreview {
        let before_entries = count_entries_recursively(temp.path());
        let first = preview_scholar_chat_grounded_answer_write_eligibility(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_grounded_answer_write_eligibility(temp.path(), request).unwrap();
        let after_entries = count_entries_recursively(temp.path());
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_grounded_answer_write_eligibility_boundary_fields(preview);
        }
        first
    }

    fn assert_grounded_answer_build_intent_deterministic_and_path_free(
        temp: &tempfile::TempDir,
        request: ScholarChatGroundedAnswerBuildIntentRequest,
    ) -> ScholarChatGroundedAnswerBuildIntentPreview {
        let before_entries = count_entries_recursively(temp.path());
        let first = preview_scholar_chat_grounded_answer_build_intent(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_grounded_answer_build_intent(temp.path(), request).unwrap();
        let after_entries = count_entries_recursively(temp.path());
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_grounded_answer_build_intent_boundary_fields(preview);
        }
        first
    }

    fn assert_grounded_answer_execution_readiness_deterministic_and_path_free(
        temp: &tempfile::TempDir,
        request: ScholarChatGroundedAnswerExecutionReadinessPreviewRequest,
    ) -> ScholarChatGroundedAnswerExecutionReadinessPreview {
        let before_entries = count_entries_recursively(temp.path());
        let first = preview_scholar_chat_grounded_answer_execution_readiness(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_grounded_answer_execution_readiness(temp.path(), request).unwrap();
        let after_entries = count_entries_recursively(temp.path());
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_grounded_answer_execution_readiness_boundary_fields(preview);
        }
        first
    }

    fn assert_grounded_answer_build_request_deterministic_and_path_free(
        temp: &tempfile::TempDir,
        request: ScholarChatGroundedAnswerBuildRequestPreviewRequest,
    ) -> ScholarChatGroundedAnswerBuildRequestPreview {
        let before_entries = count_entries_recursively(temp.path());
        let first = preview_scholar_chat_grounded_answer_build_request(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_grounded_answer_build_request(temp.path(), request).unwrap();
        let after_entries = count_entries_recursively(temp.path());
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_grounded_answer_build_request_boundary_fields(preview);
        }
        first
    }

    fn assert_grounded_answer_build_preflight_deterministic_and_path_free(
        temp: &tempfile::TempDir,
        request: ScholarChatGroundedAnswerBuildPreflightPreviewRequest,
    ) -> ScholarChatGroundedAnswerBuildPreflightPreview {
        let before_entries = count_entries_recursively(temp.path());
        let first = preview_scholar_chat_grounded_answer_build_preflight(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_grounded_answer_build_preflight(temp.path(), request).unwrap();
        let after_entries = count_entries_recursively(temp.path());
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_grounded_answer_build_preflight_boundary_fields(preview);
        }
        first
    }

    #[test]
    fn scholar_chat_draft_grounding_inspection_drops_stopwords_and_short_noise_terms() {
        let terms = inspection_terms("the and of to in a an is are was were with for on by as at from this that 12 alpha 2024 x y");
        assert!(!terms.contains("the"));
        assert!(!terms.contains("and"));
        assert!(!terms.contains("x"));
        assert!(terms.contains("12"));
        assert!(terms.contains("2024"));
        assert!(terms.contains("alpha"));
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
        let mut first_sanitized = first.clone();
        let mut second_sanitized = second.clone();
        first_sanitized.duration_ms = 0;
        second_sanitized.duration_ms = 0;
        assert_eq!(first_sanitized, second_sanitized);
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

    #[test]
    fn scholar_chat_draft_grounding_inspection_rejects_empty_prompt() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_draft_grounding_inspection(
            temp.path(),
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: request("   "),
                draft_text: Some("Draft text".to_string()),
                max_items: Some(4),
            },
        );
        assert!(matches!(result, Err(AegisError::ScholarChatPromptEmpty)));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn scholar_chat_draft_grounding_inspection_rejects_invalid_source_ids_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["", " ", "..", "../evil", "evil/source", "evil\\source"] {
            let result = preview_scholar_chat_draft_grounding_inspection(
                temp.path(),
                ScholarChatDraftGroundingInspectionRequest {
                    scholar_chat_request: ScholarChatRequest {
                        prompt: "Explain grounded text".to_string(),
                        mode: ScholarChatMode::LectureLearning,
                        grounding_policy: GroundingPolicy::LocalFirst,
                        selected_source_ids: vec![invalid.to_string()],
                    },
                    draft_text: Some("alpha beta.".to_string()),
                    max_items: Some(4),
                },
            );
            assert!(matches!(result, Err(AegisError::ScholarChatInvalidSourceId)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn scholar_chat_draft_grounding_inspection_reports_no_draft_text() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_draft_grounding_inspection(
            temp.path(),
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: request("Explain grounded text"),
                draft_text: Some("   ".to_string()),
                max_items: Some(4),
            },
        )
        .unwrap();
        assert_eq!(result.status, ScholarChatDraftGroundingInspectionStatus::NoDraftText);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "draft_text_missing"));
        assert_draft_grounding_inspection_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_draft_grounding_inspection_blocks_without_selected_sources() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_draft_grounding_inspection(
            temp.path(),
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "Explain grounded text".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![],
                },
                draft_text: Some("alpha beta.".to_string()),
                max_items: Some(4),
            },
        )
        .unwrap();
        assert_eq!(result.status, ScholarChatDraftGroundingInspectionStatus::Blocked);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "needs_sources"));
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.message.contains("No Scholar Chat source context selected")));
        assert_draft_grounding_inspection_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_draft_grounding_inspection_reports_no_evidence_candidates() {
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

        let result = preview_scholar_chat_draft_grounding_inspection(
            temp.path(),
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "Explain grounded text".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalOnly,
                    selected_source_ids: vec![source.source_id.clone()],
                },
                draft_text: Some("alpha beta.".to_string()),
                max_items: Some(4),
            },
        )
        .unwrap();
        assert_eq!(result.status, ScholarChatDraftGroundingInspectionStatus::NoEvidenceCandidates);
        assert!(result
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "needs_evidence_candidates"));
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.message.contains("No local evidence candidates")));
        assert_draft_grounding_inspection_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_draft_grounding_inspection_is_deterministic_and_path_free() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let before_entries = count_entries_recursively(temp.path());
        let request = ScholarChatDraftGroundingInspectionRequest {
            scholar_chat_request: ScholarChatRequest {
                prompt: "  alpha beta grounded evidence  ".to_string(),
                mode: ScholarChatMode::ThesisWriting,
                grounding_policy: GroundingPolicy::LocalFirst,
                selected_source_ids: vec![source_id.clone()],
            },
            draft_text: Some("Alpha beta. Gamma delta? Alpha beta gamma.".to_string()),
            max_items: Some(8),
        };
        let first = preview_scholar_chat_draft_grounding_inspection(temp.path(), request.clone()).unwrap();
        let second = preview_scholar_chat_draft_grounding_inspection(temp.path(), request).unwrap();
        let after_entries = count_entries_recursively(temp.path());
        assert_eq!(first, second);
        assert_eq!(first.normalized_prompt, "alpha beta grounded evidence");
        assert_eq!(first.status, ScholarChatDraftGroundingInspectionStatus::Inspected);
        assert_eq!(first.selected_source_count, 1);
        assert!(!first.items.is_empty());
        assert!(first.items.iter().any(|item| item.support_status == ScholarChatDraftGroundingSupportStatus::SupportedByLocalEvidence));
        let debug = format!("{first:?}");
        let json = serde_json::to_string(&first).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert_eq!(before_entries, after_entries);
        assert_draft_grounding_inspection_boundary_fields(&first);
    }

    #[test]
    fn scholar_chat_draft_grounding_inspection_supports_local_evidence_and_clamps_items() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = preview_scholar_chat_draft_grounding_inspection(
            temp.path(),
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![source_id],
                },
                draft_text: Some(
                    "Alpha beta support. Gamma. Theta. Alpha beta gamma evidence. Delta alpha beta. More alpha beta. Another alpha beta.".to_string(),
                ),
                max_items: Some(4),
            },
        )
        .unwrap();
        assert_eq!(result.status, ScholarChatDraftGroundingInspectionStatus::Inspected);
        assert_eq!(result.items.len(), 4);
        assert_eq!(
            result.supported_item_count + result.weakly_supported_item_count + result.unsupported_item_count,
            result.items.len()
        );
        assert!(result.items.iter().any(|item| item.support_status == ScholarChatDraftGroundingSupportStatus::SupportedByLocalEvidence));
        assert!(result.items.iter().any(|item| item.support_status == ScholarChatDraftGroundingSupportStatus::WeaklySupported));
        assert!(result.items.iter().any(|item| item.support_status == ScholarChatDraftGroundingSupportStatus::Unsupported));
        assert!(result.items.iter().all(|item| !item.text_preview.contains("  ")));
        assert!(result.items.iter().all(|item| !item.locator_previews.iter().any(|preview| preview.contains("section_path"))));
        assert!(result.warnings.iter().any(|warning| warning.kind == "inspection_clamped"));
        assert_draft_grounding_inspection_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_draft_grounding_inspection_local_only_support_needs_clear_overlap() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = preview_scholar_chat_draft_grounding_inspection(
            temp.path(),
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::AllowMarkedModelKnowledge,
                    selected_source_ids: vec![source_id],
                },
                draft_text: Some("Gamma.".to_string()),
                max_items: Some(4),
            },
        )
        .unwrap();
        assert_eq!(result.status, ScholarChatDraftGroundingInspectionStatus::Inspected);
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.supported_item_count, 0);
        assert_eq!(result.weakly_supported_item_count, 1);
        assert_eq!(result.unsupported_item_count, 0);
        assert_eq!(result.items[0].support_status, ScholarChatDraftGroundingSupportStatus::WeaklySupported);
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.message.contains("Model knowledge is not used in this preview")));
        assert_draft_grounding_inspection_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_draft_grounding_inspection_marks_single_meaningful_overlap_as_weakly_supported() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = preview_scholar_chat_draft_grounding_inspection(
            temp.path(),
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha beta grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![source_id],
                },
                draft_text: Some("The alpha.".to_string()),
                max_items: Some(4),
            },
        )
        .unwrap();
        assert_eq!(result.status, ScholarChatDraftGroundingInspectionStatus::Inspected);
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.supported_item_count, 0);
        assert_eq!(result.weakly_supported_item_count, 1);
        assert_eq!(result.unsupported_item_count, 0);
        assert_eq!(result.items[0].support_status, ScholarChatDraftGroundingSupportStatus::WeaklySupported);
        assert_eq!(result.items[0].matched_evidence_count, 1);
        assert_draft_grounding_inspection_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_draft_grounding_inspection_requires_two_meaningful_terms_for_supported_overlap() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = preview_scholar_chat_draft_grounding_inspection(
            temp.path(),
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha beta grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![source_id],
                },
                draft_text: Some("The alpha beta.".to_string()),
                max_items: Some(4),
            },
        )
        .unwrap();
        assert_eq!(result.status, ScholarChatDraftGroundingInspectionStatus::Inspected);
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.supported_item_count, 1);
        assert_eq!(result.weakly_supported_item_count, 0);
        assert_eq!(result.unsupported_item_count, 0);
        assert_eq!(result.items[0].support_status, ScholarChatDraftGroundingSupportStatus::SupportedByLocalEvidence);
        assert!(result.items[0].matched_evidence_count >= 1);
        assert_draft_grounding_inspection_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_draft_grounding_inspection_leaves_unrelated_items_unsupported() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = preview_scholar_chat_draft_grounding_inspection(
            temp.path(),
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha beta grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![source_id],
                },
                draft_text: Some("Zeta kappa.".to_string()),
                max_items: Some(4),
            },
        )
        .unwrap();
        assert_eq!(result.status, ScholarChatDraftGroundingInspectionStatus::Inspected);
        assert_eq!(result.items.len(), 1);
        assert_eq!(result.supported_item_count, 0);
        assert_eq!(result.weakly_supported_item_count, 0);
        assert_eq!(result.unsupported_item_count, 1);
        assert_eq!(result.items[0].support_status, ScholarChatDraftGroundingSupportStatus::Unsupported);
        assert_eq!(result.items[0].matched_evidence_count, 0);
        assert_draft_grounding_inspection_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_grounded_draft_readiness_rejects_empty_prompt() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_grounded_draft_readiness(
            temp.path(),
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "   ".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![],
                },
                draft_text: Some("Alpha beta.".to_string()),
                max_items: Some(4),
            },
        );
        assert!(matches!(result, Err(AegisError::ScholarChatPromptEmpty)));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn scholar_chat_grounded_draft_readiness_blocks_without_draft_text() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_draft_readiness_deterministic_and_path_free(
            &temp,
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![source_id],
                },
                draft_text: Some("   ".to_string()),
                max_items: Some(4),
            },
        );
        assert_eq!(result.status, ScholarChatGroundedDraftReadinessStatus::Blocked);
        assert_eq!(result.inspection_status, ScholarChatDraftGroundingInspectionStatus::NoDraftText);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "draft_text_missing"));
        assert!(result.summary.contains("blocked"));
    }

    #[test]
    fn scholar_chat_grounded_draft_readiness_blocks_without_selected_sources() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_grounded_draft_readiness_deterministic_and_path_free(
            &temp,
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![],
                },
                draft_text: Some("Alpha beta.".to_string()),
                max_items: Some(4),
            },
        );
        assert_eq!(result.status, ScholarChatGroundedDraftReadinessStatus::Blocked);
        assert_eq!(result.inspection_status, ScholarChatDraftGroundingInspectionStatus::Blocked);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "needs_sources"));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("Select Scholar Chat source context")));
    }

    #[test]
    fn scholar_chat_grounded_draft_readiness_blocks_without_evidence_candidates() {
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

        let result = assert_grounded_draft_readiness_deterministic_and_path_free(
            &temp,
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalOnly,
                    selected_source_ids: vec![source.source_id.clone()],
                },
                draft_text: Some("Alpha beta.".to_string()),
                max_items: Some(4),
            },
        );
        assert_eq!(result.status, ScholarChatGroundedDraftReadinessStatus::Blocked);
        assert_eq!(result.inspection_status, ScholarChatDraftGroundingInspectionStatus::NoEvidenceCandidates);
        assert!(result
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "needs_evidence_candidates"));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("Add local evidence candidates")));
    }

    #[test]
    fn scholar_chat_grounded_draft_readiness_marks_weak_or_unsupported_items_for_review() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let weak_result = assert_grounded_draft_readiness_deterministic_and_path_free(
            &temp,
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![source_id.clone()],
                },
                draft_text: Some("The alpha.".to_string()),
                max_items: Some(4),
            },
        );
        assert_eq!(weak_result.status, ScholarChatGroundedDraftReadinessStatus::NeedsReview);
        assert_eq!(weak_result.inspection_status, ScholarChatDraftGroundingInspectionStatus::Inspected);
        assert_eq!(weak_result.weakly_supported_item_count, 1);
        assert_eq!(weak_result.unsupported_item_count, 0);
        assert!(weak_result
            .warnings
            .iter()
            .any(|warning| warning.kind == "needs_review"));

        let unsupported_result = assert_grounded_draft_readiness_deterministic_and_path_free(
            &temp,
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![source_id],
                },
                draft_text: Some("Zeta kappa.".to_string()),
                max_items: Some(4),
            },
        );
        assert_eq!(unsupported_result.status, ScholarChatGroundedDraftReadinessStatus::NeedsReview);
        assert_eq!(unsupported_result.inspection_status, ScholarChatDraftGroundingInspectionStatus::Inspected);
        assert_eq!(unsupported_result.supported_item_count, 0);
        assert_eq!(unsupported_result.weakly_supported_item_count, 0);
        assert_eq!(unsupported_result.unsupported_item_count, 1);
    }

    #[test]
    fn scholar_chat_grounded_draft_readiness_is_ready_only_when_every_item_has_local_support() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let request = ScholarChatDraftGroundingInspectionRequest {
            scholar_chat_request: ScholarChatRequest {
                prompt: "alpha grounded evidence".to_string(),
                mode: ScholarChatMode::LectureLearning,
                grounding_policy: GroundingPolicy::LocalFirst,
                selected_source_ids: vec![source_id],
            },
            draft_text: Some("Alpha beta.".to_string()),
            max_items: Some(4),
        };
        let first = assert_grounded_draft_readiness_deterministic_and_path_free(&temp, request);
        assert_eq!(first.status, ScholarChatGroundedDraftReadinessStatus::ReadyForGroundedDraftLater);
        assert_eq!(first.inspection_status, ScholarChatDraftGroundingInspectionStatus::Inspected);
        assert_eq!(first.inspected_item_count, 1);
        assert_eq!(first.supported_item_count, 1);
        assert_eq!(first.weakly_supported_item_count, 0);
        assert_eq!(first.unsupported_item_count, 0);
        assert!(first.summary.contains("All inspected items were supported by local evidence"));
    }

    #[test]
    fn scholar_chat_grounded_answer_build_plan_rejects_empty_prompt() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_grounded_answer_build_plan(
            temp.path(),
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "   ".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![],
                },
                draft_text: Some("Alpha beta.".to_string()),
                max_items: Some(4),
            },
        );
        assert!(matches!(result, Err(AegisError::ScholarChatPromptEmpty)));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn scholar_chat_grounded_answer_build_plan_blocks_without_draft_text() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_build_plan_deterministic_and_path_free(
            &temp,
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![source_id],
                },
                draft_text: Some("   ".to_string()),
                max_items: Some(4),
            },
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildPlanStatus::Blocked);
        assert_eq!(result.readiness_status, ScholarChatGroundedDraftReadinessStatus::Blocked);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "draft_text_missing"));
        assert!(result.summary.contains("blocked"));
    }

    #[test]
    fn scholar_chat_grounded_answer_build_plan_blocks_without_selected_sources() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_grounded_answer_build_plan_deterministic_and_path_free(
            &temp,
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![],
                },
                draft_text: Some("Alpha beta.".to_string()),
                max_items: Some(4),
            },
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildPlanStatus::Blocked);
        assert_eq!(result.readiness_status, ScholarChatGroundedDraftReadinessStatus::Blocked);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "needs_sources"));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("Select Scholar Chat source context")));
    }

    #[test]
    fn scholar_chat_grounded_answer_build_plan_blocks_without_evidence_candidates() {
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

        let result = assert_grounded_answer_build_plan_deterministic_and_path_free(
            &temp,
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalOnly,
                    selected_source_ids: vec![source.source_id.clone()],
                },
                draft_text: Some("Alpha beta.".to_string()),
                max_items: Some(4),
            },
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildPlanStatus::Blocked);
        assert_eq!(result.readiness_status, ScholarChatGroundedDraftReadinessStatus::Blocked);
        assert!(result
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "needs_evidence_candidates"));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("Add local evidence candidates")));
    }

    #[test]
    fn scholar_chat_grounded_answer_build_plan_marks_weak_or_unsupported_items_for_review() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let weak_result = assert_grounded_answer_build_plan_deterministic_and_path_free(
            &temp,
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![source_id.clone()],
                },
                draft_text: Some("The alpha.".to_string()),
                max_items: Some(4),
            },
        );
        assert_eq!(weak_result.status, ScholarChatGroundedAnswerBuildPlanStatus::NeedsReview);
        assert_eq!(weak_result.readiness_status, ScholarChatGroundedDraftReadinessStatus::NeedsReview);
        assert_eq!(weak_result.weakly_supported_item_count, 1);
        assert_eq!(weak_result.unsupported_item_count, 0);
        assert!(weak_result
            .warnings
            .iter()
            .any(|warning| warning.kind == "needs_review"));

        let unsupported_result = assert_grounded_answer_build_plan_deterministic_and_path_free(
            &temp,
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![source_id],
                },
                draft_text: Some("Zeta kappa.".to_string()),
                max_items: Some(4),
            },
        );
        assert_eq!(unsupported_result.status, ScholarChatGroundedAnswerBuildPlanStatus::NeedsReview);
        assert_eq!(unsupported_result.readiness_status, ScholarChatGroundedDraftReadinessStatus::NeedsReview);
        assert_eq!(unsupported_result.supported_item_count, 0);
        assert_eq!(unsupported_result.weakly_supported_item_count, 0);
        assert_eq!(unsupported_result.unsupported_item_count, 1);
    }

    #[test]
    fn scholar_chat_grounded_answer_build_plan_is_ready_only_when_every_item_has_local_support() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let request = ScholarChatDraftGroundingInspectionRequest {
            scholar_chat_request: ScholarChatRequest {
                prompt: "alpha grounded evidence".to_string(),
                mode: ScholarChatMode::LectureLearning,
                grounding_policy: GroundingPolicy::LocalFirst,
                selected_source_ids: vec![source_id],
            },
            draft_text: Some("Alpha beta.".to_string()),
            max_items: Some(4),
        };
        let first = assert_grounded_answer_build_plan_deterministic_and_path_free(&temp, request);
        assert_eq!(first.status, ScholarChatGroundedAnswerBuildPlanStatus::PlanReadyLater);
        assert_eq!(
            first.readiness_status,
            ScholarChatGroundedDraftReadinessStatus::ReadyForGroundedDraftLater
        );
        assert_eq!(first.inspected_item_count, 1);
        assert_eq!(first.supported_item_count, 1);
        assert_eq!(first.weakly_supported_item_count, 0);
        assert_eq!(first.unsupported_item_count, 0);
        assert!(first.summary.contains("All inspected items were supported by local evidence"));
    }

    #[test]
    fn scholar_chat_grounded_answer_candidate_rejects_empty_prompt() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_grounded_answer_candidate(
            temp.path(),
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "   ".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![],
                },
                draft_text: Some("Alpha beta.".to_string()),
                max_items: Some(4),
            },
        );
        assert!(matches!(result, Err(AegisError::ScholarChatPromptEmpty)));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn scholar_chat_grounded_answer_candidate_blocks_without_selected_sources() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_grounded_answer_candidate_deterministic_and_path_free(
            &temp,
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![],
                },
                draft_text: Some("Alpha beta.".to_string()),
                max_items: Some(4),
            },
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerCandidateStatus::Blocked);
        assert_eq!(result.candidate_statement_count, 0);
        assert!(result.candidate_items.is_empty());
        assert_eq!(result.inspected_item_count, 0);
        assert_eq!(result.supported_item_count, 0);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "needs_sources"));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("Select Scholar Chat source context")));
    }

    #[test]
    fn scholar_chat_grounded_answer_candidate_blocks_without_draft_text() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_candidate_deterministic_and_path_free(
            &temp,
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![source_id],
                },
                draft_text: Some("   ".to_string()),
                max_items: Some(4),
            },
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerCandidateStatus::Blocked);
        assert_eq!(result.candidate_statement_count, 0);
        assert!(result.candidate_items.is_empty());
        assert_eq!(result.inspected_item_count, 0);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "draft_text_missing"));
        assert!(result.summary.contains("blocked"));
    }

    #[test]
    fn scholar_chat_grounded_answer_candidate_blocks_without_evidence_candidates() {
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

        let result = assert_grounded_answer_candidate_deterministic_and_path_free(
            &temp,
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalOnly,
                    selected_source_ids: vec![source.source_id.clone()],
                },
                draft_text: Some("Alpha beta.".to_string()),
                max_items: Some(4),
            },
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerCandidateStatus::Blocked);
        assert_eq!(result.candidate_statement_count, 0);
        assert!(result.candidate_items.is_empty());
        assert_eq!(result.evidence_candidate_count, 0);
        assert!(result
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "needs_evidence_candidates"));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("Add local evidence candidates")));
    }

    #[test]
    fn scholar_chat_grounded_answer_candidate_marks_weak_or_unsupported_items_for_review() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let weak_result = assert_grounded_answer_candidate_deterministic_and_path_free(
            &temp,
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![source_id.clone()],
                },
                draft_text: Some("The alpha.".to_string()),
                max_items: Some(4),
            },
        );
        assert_eq!(weak_result.status, ScholarChatGroundedAnswerCandidateStatus::NeedsReview);
        assert_eq!(weak_result.candidate_statement_count, 0);
        assert!(weak_result.candidate_items.is_empty());
        assert_eq!(weak_result.weakly_supported_item_count, 1);
        assert_eq!(weak_result.unsupported_item_count, 0);
        assert!(weak_result
            .warnings
            .iter()
            .any(|warning| warning.kind == "needs_review"));

        let unsupported_result = assert_grounded_answer_candidate_deterministic_and_path_free(
            &temp,
            ScholarChatDraftGroundingInspectionRequest {
                scholar_chat_request: ScholarChatRequest {
                    prompt: "alpha grounded evidence".to_string(),
                    mode: ScholarChatMode::LectureLearning,
                    grounding_policy: GroundingPolicy::LocalFirst,
                    selected_source_ids: vec![source_id],
                },
                draft_text: Some("Zeta kappa.".to_string()),
                max_items: Some(4),
            },
        );
        assert_eq!(unsupported_result.status, ScholarChatGroundedAnswerCandidateStatus::NeedsReview);
        assert_eq!(unsupported_result.candidate_statement_count, 0);
        assert!(unsupported_result.candidate_items.is_empty());
        assert_eq!(unsupported_result.supported_item_count, 0);
        assert_eq!(unsupported_result.weakly_supported_item_count, 0);
        assert_eq!(unsupported_result.unsupported_item_count, 1);
    }

    #[test]
    fn scholar_chat_grounded_answer_candidate_is_ready_only_when_every_item_has_local_support() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let request = ScholarChatDraftGroundingInspectionRequest {
            scholar_chat_request: ScholarChatRequest {
                prompt: "alpha grounded evidence".to_string(),
                mode: ScholarChatMode::LectureLearning,
                grounding_policy: GroundingPolicy::LocalFirst,
                selected_source_ids: vec![source_id.clone()],
            },
            draft_text: Some("Alpha beta. Alpha beta gamma.".to_string()),
            max_items: Some(4),
        };
        let candidate_preview = assert_grounded_answer_candidate_deterministic_and_path_free(&temp, request.clone());
        let build_plan_preview = preview_scholar_chat_grounded_answer_build_plan(temp.path(), request.clone()).unwrap();
        assert_eq!(build_plan_preview.status, ScholarChatGroundedAnswerBuildPlanStatus::PlanReadyLater);
        assert_eq!(candidate_preview.selected_source_count, build_plan_preview.selected_source_count);
        assert_eq!(candidate_preview.evidence_candidate_count, build_plan_preview.evidence_candidate_count);
        assert_eq!(candidate_preview.inspected_item_count, build_plan_preview.inspected_item_count);
        assert_eq!(candidate_preview.supported_item_count, build_plan_preview.supported_item_count);
        assert_eq!(candidate_preview.weakly_supported_item_count, build_plan_preview.weakly_supported_item_count);
        assert_eq!(candidate_preview.unsupported_item_count, build_plan_preview.unsupported_item_count);
        assert_eq!(candidate_preview.status, ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater);
        assert_eq!(candidate_preview.candidate_statement_count, 2);
        assert_eq!(candidate_preview.candidate_statement_count, candidate_preview.candidate_items.len());
        assert_eq!(candidate_preview.inspected_item_count, 2);
        assert_eq!(candidate_preview.supported_item_count, 2);
        assert_eq!(candidate_preview.weakly_supported_item_count, 0);
        assert_eq!(candidate_preview.unsupported_item_count, 0);
        assert!(candidate_preview.summary.contains("All inspected items were supported by local evidence"));
        let inspection_preview = preview_scholar_chat_draft_grounding_inspection(temp.path(), request).unwrap();
        assert_eq!(inspection_preview.status, ScholarChatDraftGroundingInspectionStatus::Inspected);
        assert_eq!(candidate_preview.candidate_items.len(), inspection_preview.items.len());
        for (candidate_item, inspection_item) in candidate_preview.candidate_items.iter().zip(inspection_preview.items.iter()) {
            assert_eq!(candidate_item.item_index, inspection_item.item_index);
            assert_eq!(candidate_item.statement_preview, inspection_item.text_preview);
            assert_eq!(candidate_item.support_status, inspection_item.support_status);
            assert_eq!(candidate_item.matched_evidence_count, inspection_item.matched_evidence_count);
            assert_eq!(candidate_item.source_ids, inspection_item.source_ids);
            assert_eq!(candidate_item.locator_previews, inspection_item.locator_previews);
        }
    }

    #[test]
    fn scholar_chat_grounded_answer_build_intent_rejects_empty_prompt() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_grounded_answer_build_intent(
            temp.path(),
            build_intent_request(
                "   ",
                Some("Alpha beta."),
                vec!["src_demo".to_string()],
                Some("draft-1"),
                true,
            ),
        );
        assert!(matches!(result, Err(AegisError::ScholarChatPromptEmpty)));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn scholar_chat_grounded_answer_build_intent_rejects_invalid_answer_draft_ids_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["..", "../evil", "evil/draft", "evil\\draft"] {
            let result = preview_scholar_chat_grounded_answer_build_intent(
                temp.path(),
                build_intent_request(
                    "alpha grounded evidence",
                    Some("Alpha beta."),
                    vec!["src_demo".to_string()],
                    Some(invalid),
                    true,
                ),
            );
            assert!(matches!(result, Err(AegisError::AnswerDraftInvalidId)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn scholar_chat_grounded_answer_build_intent_treats_whitespace_answer_draft_id_as_missing_input() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_build_intent_deterministic_and_path_free(
            &temp,
            build_intent_request(
                "alpha grounded evidence",
                Some("Alpha beta. Alpha beta gamma."),
                vec![source_id],
                Some("   "),
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildIntentStatus::Blocked);
        assert_eq!(result.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater);
        assert!(result.required_inputs.contains(&"answer_draft_id".to_string()));
        assert!(result.missing_inputs.contains(&"answer_draft_id".to_string()));
        assert!(!result
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "answer_draft_id_invalid"));
        assert_grounded_answer_build_intent_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_grounded_answer_build_intent_blocks_without_draft_text() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_build_intent_deterministic_and_path_free(
            &temp,
            build_intent_request(
                "alpha grounded evidence",
                Some("   "),
                vec![source_id],
                Some("draft-1"),
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildIntentStatus::Blocked);
        assert_eq!(result.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::Blocked);
        assert!(result.missing_inputs.contains(&"write_eligible_later".to_string()));
        assert!(result
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "write_eligibility_blocked"));
        assert_grounded_answer_build_intent_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_grounded_answer_build_intent_blocks_without_selected_sources() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_grounded_answer_build_intent_deterministic_and_path_free(
            &temp,
            build_intent_request(
                "alpha grounded evidence",
                Some("Alpha beta."),
                vec![],
                Some("draft-1"),
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildIntentStatus::Blocked);
        assert_eq!(result.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::Blocked);
        assert!(result.missing_inputs.contains(&"write_eligible_later".to_string()));
        assert!(result
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "write_eligibility_blocked"));
        assert_grounded_answer_build_intent_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_grounded_answer_build_intent_needs_review_when_candidate_needs_review() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_build_intent_deterministic_and_path_free(
            &temp,
            build_intent_request(
                "alpha grounded evidence",
                Some("The alpha."),
                vec![source_id],
                Some("draft-1"),
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildIntentStatus::NeedsReview);
        assert_eq!(result.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::NeedsReview);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::NeedsReview);
        assert_eq!(result.selected_source_count, 1);
        assert_eq!(result.evidence_candidate_count, 1);
        assert_eq!(result.inspected_item_count, 1);
        assert_eq!(result.supported_item_count, 0);
        assert_eq!(result.weakly_supported_item_count, 1);
        assert_eq!(result.unsupported_item_count, 0);
        assert_eq!(result.candidate_statement_count, 0);
        assert!(result
            .intent_reasons
            .iter()
            .any(|reason| reason.contains("Weakly supported or unsupported draft items remain.")));
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.kind == "needs_review"));
        assert_grounded_answer_build_intent_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_grounded_answer_build_intent_blocks_without_explicit_user_intent() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_build_intent_deterministic_and_path_free(
            &temp,
            build_intent_request(
                "alpha grounded evidence",
                Some("Alpha beta. Alpha beta gamma."),
                vec![source_id],
                Some("draft-1"),
                false,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildIntentStatus::Blocked);
        assert_eq!(result.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater);
        assert!(result.required_inputs.contains(&"explicit_user_intent".to_string()));
        assert!(result.missing_inputs.contains(&"explicit_user_intent".to_string()));
        assert!(result
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "explicit_user_intent_missing"));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("Provide explicit user intent before any GroundedAnswer service call.")));
        assert_grounded_answer_build_intent_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_grounded_answer_build_intent_blocks_without_answer_draft_id() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_build_intent_deterministic_and_path_free(
            &temp,
            build_intent_request(
                "alpha grounded evidence",
                Some("Alpha beta. Alpha beta gamma."),
                vec![source_id],
                None,
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildIntentStatus::Blocked);
        assert_eq!(result.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater);
        assert!(result.required_inputs.contains(&"answer_draft_id".to_string()));
        assert!(result.missing_inputs.contains(&"answer_draft_id".to_string()));
        assert!(result
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "answer_draft_id_missing"));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("Provide an answer draft ID before any GroundedAnswer service call.")));
        assert_grounded_answer_build_intent_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_grounded_answer_build_intent_is_ready_only_when_all_inputs_present() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let request = build_intent_request(
            "alpha grounded evidence",
            Some("Alpha beta. Alpha beta gamma."),
            vec![source_id.clone()],
            Some("draft-1"),
            true,
        );
        let build_intent_preview = assert_grounded_answer_build_intent_deterministic_and_path_free(&temp, request.clone());
        let write_eligibility_preview = assert_grounded_answer_write_eligibility_deterministic_and_path_free(&temp, request.grounding_request);
        assert_eq!(build_intent_preview.status, ScholarChatGroundedAnswerBuildIntentStatus::IntentReadyLater);
        assert_eq!(build_intent_preview.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater);
        assert_eq!(build_intent_preview.candidate_status, ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater);
        assert_eq!(build_intent_preview.selected_source_count, write_eligibility_preview.selected_source_count);
        assert_eq!(build_intent_preview.evidence_candidate_count, write_eligibility_preview.evidence_candidate_count);
        assert_eq!(build_intent_preview.inspected_item_count, write_eligibility_preview.inspected_item_count);
        assert_eq!(build_intent_preview.supported_item_count, write_eligibility_preview.supported_item_count);
        assert_eq!(build_intent_preview.weakly_supported_item_count, write_eligibility_preview.weakly_supported_item_count);
        assert_eq!(build_intent_preview.unsupported_item_count, write_eligibility_preview.unsupported_item_count);
        assert_eq!(build_intent_preview.candidate_statement_count, write_eligibility_preview.candidate_statement_count);
        assert_eq!(
            build_intent_preview.required_inputs,
            vec![
                "write_eligible_later".to_string(),
                "explicit_user_intent".to_string(),
                "answer_draft_id".to_string(),
            ]
        );
        assert!(build_intent_preview.missing_inputs.is_empty());
        assert!(build_intent_preview
            .intent_reasons
            .iter()
            .any(|reason| reason.contains("All inspected items were supported by local evidence")));
        assert!(build_intent_preview
            .warnings
            .iter()
            .any(|warning| warning.kind == "intent_ready_later"));
        assert!(build_intent_preview
            .next_required_actions
            .iter()
            .any(|action| action.contains("A future user-confirmed GroundedAnswer service call can be added later without changing this preview.")));
        assert_grounded_answer_build_intent_boundary_fields(&build_intent_preview);
    }

    #[test]
    fn scholar_chat_grounded_answer_build_request_rejects_empty_prompt() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_grounded_answer_build_request(
            temp.path(),
            build_request_request(
                "   ",
                Some("Alpha beta."),
                vec!["src_demo".to_string()],
                Some("draft-1"),
                true,
            ),
        );
        assert!(matches!(result, Err(AegisError::ScholarChatPromptEmpty)));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn scholar_chat_grounded_answer_build_request_rejects_invalid_answer_draft_ids_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["..", "../evil", "evil/draft", "evil\\draft"] {
            let result = preview_scholar_chat_grounded_answer_build_request(
                temp.path(),
                build_request_request(
                    "alpha grounded evidence",
                    Some("Alpha beta."),
                    vec!["src_demo".to_string()],
                    Some(invalid),
                    true,
                ),
            );
            assert!(matches!(result, Err(AegisError::AnswerDraftInvalidId)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn scholar_chat_grounded_answer_build_request_rejects_invalid_source_ids_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["..", "../evil", "evil/source", "evil\\source"] {
            let result = preview_scholar_chat_grounded_answer_build_request(
                temp.path(),
                build_request_request(
                    "alpha grounded evidence",
                    Some("Alpha beta."),
                    vec![invalid.to_string()],
                    Some("draft-1"),
                    true,
                ),
            );
            assert!(matches!(result, Err(AegisError::ScholarChatInvalidSourceId)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn scholar_chat_grounded_answer_build_request_blocks_without_selected_sources() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_grounded_answer_build_request_deterministic_and_path_free(
            &temp,
            build_request_request(
                "alpha grounded evidence",
                Some("Alpha beta. Alpha beta gamma."),
                vec![],
                Some("draft-1"),
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildRequestStatus::Blocked);
        assert_eq!(result.build_intent_status, ScholarChatGroundedAnswerBuildIntentStatus::Blocked);
        assert_eq!(result.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::Blocked);
        assert!(result.required_inputs.contains(&"selected_source_ids".to_string()));
        assert!(result.missing_inputs.contains(&"selected_source_ids".to_string()));
        assert_grounded_answer_build_request_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_grounded_answer_build_request_treats_whitespace_answer_draft_id_as_missing_input() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_build_request_deterministic_and_path_free(
            &temp,
            build_request_request(
                "alpha grounded evidence",
                Some("Alpha beta. Alpha beta gamma."),
                vec![source_id],
                Some("   "),
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildRequestStatus::Blocked);
        assert_eq!(result.build_intent_status, ScholarChatGroundedAnswerBuildIntentStatus::Blocked);
        assert_eq!(result.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater);
        assert!(result.required_inputs.contains(&"answer_draft_id".to_string()));
        assert!(result.missing_inputs.contains(&"answer_draft_id".to_string()));
        assert!(result.answer_draft_id.is_none());
        assert_grounded_answer_build_request_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_grounded_answer_build_request_needs_review_when_build_intent_needs_review() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_build_request_deterministic_and_path_free(
            &temp,
            build_request_request(
                "alpha grounded evidence",
                Some("The alpha."),
                vec![source_id],
                Some("draft-1"),
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildRequestStatus::NeedsReview);
        assert_eq!(result.build_intent_status, ScholarChatGroundedAnswerBuildIntentStatus::NeedsReview);
        assert_eq!(result.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::NeedsReview);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::NeedsReview);
        assert_eq!(result.candidate_statement_count, 0);
        assert!(result
            .request_reasons
            .iter()
            .any(|reason| reason.contains("Weakly supported or unsupported draft items remain")));
        assert_grounded_answer_build_request_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_grounded_answer_build_request_is_ready_only_when_build_intent_is_ready_later() {
        let temp = tempfile::tempdir().unwrap();
        let source_a = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let source_b = build_source_with_index(&temp, "alpha beta gamma\nalpha beta epsilon\n");
        let request = build_request_request(
            "alpha grounded evidence",
            Some("Alpha beta. Alpha beta gamma."),
            vec![format!("  {source_b}  "), format!("  {source_a}  ")],
            Some("  draft-1  "),
            true,
        );
        let first = assert_grounded_answer_build_request_deterministic_and_path_free(&temp, request.clone());
        let build_intent_preview = preview_scholar_chat_grounded_answer_build_intent(
            temp.path(),
            request.build_intent_request.clone(),
        )
        .unwrap();
        assert_eq!(first.status, ScholarChatGroundedAnswerBuildRequestStatus::RequestReadyLater);
        assert_eq!(first.build_intent_status, ScholarChatGroundedAnswerBuildIntentStatus::IntentReadyLater);
        assert_eq!(first.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater);
        assert_eq!(first.candidate_status, ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater);
        assert_eq!(first.selected_source_count, 2);
        assert_eq!(first.selected_source_ids, vec![source_b.clone(), source_a.clone()]);
        assert_eq!(first.answer_draft_id.as_deref(), Some("draft-1"));
        assert_eq!(first.selected_source_count, build_intent_preview.selected_source_count);
        assert_eq!(first.evidence_candidate_count, build_intent_preview.evidence_candidate_count);
        assert_eq!(first.inspected_item_count, build_intent_preview.inspected_item_count);
        assert_eq!(first.supported_item_count, build_intent_preview.supported_item_count);
        assert_eq!(first.weakly_supported_item_count, build_intent_preview.weakly_supported_item_count);
        assert_eq!(first.unsupported_item_count, build_intent_preview.unsupported_item_count);
        assert_eq!(first.candidate_statement_count, build_intent_preview.candidate_statement_count);
        assert!(first
            .request_reasons
            .iter()
            .any(|reason| reason.contains("ready later for a future GroundedAnswer service call")));
        assert_grounded_answer_build_request_boundary_fields(&first);
    }

    #[test]
    fn scholar_chat_grounded_answer_build_request_allows_missing_answer_draft_file_when_ready_later() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_build_request_deterministic_and_path_free(
            &temp,
            build_request_request(
                "alpha grounded evidence",
                Some("Alpha beta. Alpha beta gamma."),
                vec![source_id],
                Some("missing-draft-1"),
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildRequestStatus::RequestReadyLater);
        assert_eq!(result.build_intent_status, ScholarChatGroundedAnswerBuildIntentStatus::IntentReadyLater);
        assert_eq!(result.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater);
        assert_eq!(result.answer_draft_id.as_deref(), Some("missing-draft-1"));
        assert!(result.missing_inputs.contains(&"answer_draft_id".to_string()) == false);
        assert_grounded_answer_build_request_boundary_fields(&result);
    }

    #[test]
    fn scholar_chat_grounded_answer_build_preflight_rejects_empty_prompt() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_grounded_answer_build_preflight(
            temp.path(),
            build_preflight_request(
                "   ",
                Some("Alpha beta."),
                vec!["src_demo".to_string()],
                Some("draft-1"),
                true,
            ),
        );
        assert!(matches!(result, Err(AegisError::ScholarChatPromptEmpty)));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn scholar_chat_grounded_answer_build_preflight_rejects_invalid_answer_draft_ids_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["..", "../evil", "evil/draft", "evil\\draft"] {
            let result = preview_scholar_chat_grounded_answer_build_preflight(
                temp.path(),
                build_preflight_request(
                    "alpha grounded evidence",
                    Some("Alpha beta."),
                    vec!["src_demo".to_string()],
                    Some(invalid),
                    true,
                ),
            );
            assert!(matches!(result, Err(AegisError::AnswerDraftInvalidId)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn scholar_chat_grounded_answer_build_preflight_rejects_invalid_source_ids_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["..", "../evil", "evil/source", "evil\\source"] {
            let result = preview_scholar_chat_grounded_answer_build_preflight(
                temp.path(),
                build_preflight_request(
                    "alpha grounded evidence",
                    Some("Alpha beta."),
                    vec![invalid.to_string()],
                    Some("draft-1"),
                    true,
                ),
            );
            assert!(matches!(result, Err(AegisError::ScholarChatInvalidSourceId)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn scholar_chat_grounded_answer_build_preflight_rejects_invalid_selected_source_ids_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["", " ", "..", "../evil", "evil/source", "evil\\source"] {
            let result = preview_scholar_chat_grounded_answer_build_preflight(
                temp.path(),
                build_preflight_request(
                    "alpha grounded evidence",
                    Some("Alpha beta."),
                    vec![invalid.to_string()],
                    Some("draft-1"),
                    true,
                ),
            );
            assert!(matches!(result, Err(AegisError::ScholarChatInvalidSourceId)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn scholar_chat_grounded_answer_build_preflight_blocks_when_build_request_is_blocked() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_grounded_answer_build_preflight_deterministic_and_path_free(
            &temp,
            build_preflight_request(
                "alpha grounded evidence",
                Some("Alpha beta. Alpha beta gamma."),
                vec![],
                Some("draft-1"),
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildPreflightStatus::Blocked);
        assert_eq!(result.build_request_status, ScholarChatGroundedAnswerBuildRequestStatus::Blocked);
        assert_eq!(result.build_intent_status, ScholarChatGroundedAnswerBuildIntentStatus::Blocked);
        assert_eq!(result.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::Blocked);
        assert!(result.missing_inputs.contains(&"build_request_ready_later".to_string()));
        assert!(!result.answer_draft_present);
        assert!(!result.answer_draft_readable);
    }

    #[test]
    fn scholar_chat_grounded_answer_build_preflight_needs_review_when_build_request_needs_review() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_build_preflight_deterministic_and_path_free(
            &temp,
            build_preflight_request(
                "alpha grounded evidence",
                Some("The alpha."),
                vec![source_id],
                Some("draft-1"),
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildPreflightStatus::NeedsReview);
        assert_eq!(result.build_request_status, ScholarChatGroundedAnswerBuildRequestStatus::NeedsReview);
        assert_eq!(result.build_intent_status, ScholarChatGroundedAnswerBuildIntentStatus::NeedsReview);
        assert_eq!(result.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::NeedsReview);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::NeedsReview);
        assert!(result
            .preflight_reasons
            .iter()
            .any(|reason| reason.contains("needs review")));
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.kind == "needs_review"));
    }

    #[test]
    fn scholar_chat_grounded_answer_build_preflight_blocks_without_answer_draft_id() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_build_preflight_deterministic_and_path_free(
            &temp,
            build_preflight_request(
                "alpha grounded evidence",
                Some("Alpha beta. Alpha beta gamma."),
                vec![source_id],
                None,
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildPreflightStatus::Blocked);
        assert_eq!(result.build_request_status, ScholarChatGroundedAnswerBuildRequestStatus::Blocked);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "answer_draft_id_missing"));
        assert!(!result.answer_draft_present);
        assert!(!result.answer_draft_readable);
    }

    #[test]
    fn scholar_chat_grounded_answer_build_preflight_treats_whitespace_answer_draft_id_as_missing_input() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_build_preflight_deterministic_and_path_free(
            &temp,
            build_preflight_request(
                "alpha grounded evidence",
                Some("Alpha beta. Alpha beta gamma."),
                vec![source_id],
                Some("   "),
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildPreflightStatus::Blocked);
        assert_eq!(result.build_request_status, ScholarChatGroundedAnswerBuildRequestStatus::Blocked);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "answer_draft_id_missing"));
        assert!(!result.answer_draft_present);
        assert!(!result.answer_draft_readable);
    }

    #[test]
    fn scholar_chat_grounded_answer_build_preflight_blocks_when_answer_draft_missing() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_build_preflight_deterministic_and_path_free(
            &temp,
            build_preflight_request(
                "alpha grounded evidence",
                Some("Alpha beta. Alpha beta gamma."),
                vec![source_id],
                Some("missing-draft-1"),
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildPreflightStatus::Blocked);
        assert_eq!(result.build_request_status, ScholarChatGroundedAnswerBuildRequestStatus::RequestReadyLater);
        assert!(!result.answer_draft_present);
        assert!(!result.answer_draft_readable);
        assert_eq!(result.answer_draft_claim_count, 0);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "answer_draft_missing"));
        assert!(result.missing_inputs.contains(&"answer_draft_present".to_string()));
        assert!(result.missing_inputs.contains(&"answer_draft_readable".to_string()));
    }

    #[test]
    fn scholar_chat_grounded_answer_build_preflight_blocks_when_answer_draft_malformed() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, answer_draft_id, version_id, _) = build_readable_answer_draft_fixture(&temp);
        let draft_path = crate::corpus_paths::CorpusPaths::new(temp.path())
            .source_version_dir(&source_id, &version_id)
            .join("answer_drafts")
            .join(format!("{answer_draft_id}.json"));
        fs::write(&draft_path, "{not valid json").unwrap();
        let result = assert_grounded_answer_build_preflight_deterministic_and_path_free(
            &temp,
            build_preflight_request(
                "alpha grounded evidence",
                Some("Alpha beta. Alpha beta gamma."),
                vec![source_id],
                Some(&answer_draft_id),
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildPreflightStatus::Blocked);
        assert_eq!(result.build_request_status, ScholarChatGroundedAnswerBuildRequestStatus::RequestReadyLater);
        assert!(result.answer_draft_present);
        assert!(!result.answer_draft_readable);
        assert_eq!(result.answer_draft_claim_count, 0);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "answer_draft_unreadable"));
        assert!(result.missing_inputs.contains(&"answer_draft_readable".to_string()));
    }

    #[test]
    fn scholar_chat_grounded_answer_build_preflight_is_ready_only_when_build_request_is_ready_and_answer_draft_readable() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, answer_draft_id, _version_id, claim_count) = build_readable_answer_draft_fixture(&temp);
        let request = build_preflight_request(
            "alpha grounded evidence",
            Some("Alpha beta. Alpha beta gamma."),
            vec![format!("  {source_id}  ")],
            Some(&answer_draft_id),
            true,
        );
        let result = assert_grounded_answer_build_preflight_deterministic_and_path_free(&temp, request);
        assert_eq!(result.status, ScholarChatGroundedAnswerBuildPreflightStatus::PreflightReadyLater);
        assert_eq!(result.build_request_status, ScholarChatGroundedAnswerBuildRequestStatus::RequestReadyLater);
        assert_eq!(result.answer_draft_id.as_deref(), Some(answer_draft_id.as_str()));
        assert!(result.answer_draft_present);
        assert!(result.answer_draft_readable);
        assert_eq!(result.answer_draft_claim_count, claim_count);
        assert_eq!(result.selected_source_ids, vec![source_id]);
        assert!(result
            .preflight_reasons
            .iter()
            .any(|reason| reason.contains("Answer draft readable: true")));
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.kind == "preflight_ready_later"));
    }

    #[test]
    fn scholar_chat_grounded_answer_execution_readiness_rejects_empty_prompt() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_grounded_answer_execution_readiness(
            temp.path(),
            execution_readiness_request(
                "   ",
                Some("Alpha beta."),
                vec!["src_demo".to_string()],
                Some("draft-1"),
                true,
                true,
            ),
        );
        assert!(matches!(result, Err(AegisError::ScholarChatPromptEmpty)));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn scholar_chat_grounded_answer_execution_readiness_rejects_invalid_ids_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["..", "../evil", "evil/draft", "evil\\draft"] {
            let result = preview_scholar_chat_grounded_answer_execution_readiness(
                temp.path(),
                execution_readiness_request(
                    "alpha grounded evidence",
                    Some("Alpha beta."),
                    vec!["src_demo".to_string()],
                    Some(invalid),
                    true,
                    true,
                ),
            );
            assert!(matches!(result, Err(AegisError::AnswerDraftInvalidId)));
            assert!(!temp.path().join(".aegis").exists());
        }
        for invalid in ["", " ", "..", "../evil", "evil/source", "evil\\source"] {
            let result = preview_scholar_chat_grounded_answer_execution_readiness(
                temp.path(),
                execution_readiness_request(
                    "alpha grounded evidence",
                    Some("Alpha beta."),
                    vec![invalid.to_string()],
                    Some("draft-1"),
                    true,
                    true,
                ),
            );
            assert!(matches!(result, Err(AegisError::ScholarChatInvalidSourceId)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn scholar_chat_grounded_answer_execution_readiness_blocks_when_build_preflight_is_blocked() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_grounded_answer_execution_readiness_deterministic_and_path_free(
            &temp,
            execution_readiness_request(
                "alpha grounded evidence",
                Some("Alpha beta. Alpha beta gamma."),
                vec![],
                Some("draft-1"),
                true,
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerExecutionReadinessStatus::Blocked);
        assert_eq!(result.build_preflight_status, ScholarChatGroundedAnswerBuildPreflightStatus::Blocked);
        assert_eq!(result.build_request_status, ScholarChatGroundedAnswerBuildRequestStatus::Blocked);
        assert_eq!(result.build_intent_status, ScholarChatGroundedAnswerBuildIntentStatus::Blocked);
        assert_eq!(result.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::Blocked);
        assert!(result.missing_inputs.contains(&"build_preflight_ready_later".to_string()));
        assert!(!result.answer_draft_present);
        assert!(!result.answer_draft_readable);
    }

    #[test]
    fn scholar_chat_grounded_answer_execution_readiness_needs_review_when_build_preflight_needs_review() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_execution_readiness_deterministic_and_path_free(
            &temp,
            execution_readiness_request(
                "alpha grounded evidence",
                Some("The alpha."),
                vec![source_id],
                Some("draft-1"),
                true,
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerExecutionReadinessStatus::NeedsReview);
        assert_eq!(result.build_preflight_status, ScholarChatGroundedAnswerBuildPreflightStatus::NeedsReview);
        assert_eq!(result.build_request_status, ScholarChatGroundedAnswerBuildRequestStatus::NeedsReview);
        assert_eq!(result.build_intent_status, ScholarChatGroundedAnswerBuildIntentStatus::NeedsReview);
        assert_eq!(result.write_eligibility_status, ScholarChatGroundedAnswerWriteEligibilityStatus::NeedsReview);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::NeedsReview);
        assert!(result
            .readiness_reasons
            .iter()
            .any(|reason| reason.contains("needs review")));
    }

    #[test]
    fn scholar_chat_grounded_answer_execution_readiness_blocks_when_answer_draft_missing() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, answer_draft_id, version_id, _claim_count) = build_readable_answer_draft_fixture(&temp);
        let draft_path = crate::corpus_paths::CorpusPaths::new(temp.path())
            .source_version_dir(&source_id, &version_id)
            .join("answer_drafts")
            .join(format!("{answer_draft_id}.json"));
        fs::remove_file(&draft_path).unwrap();
        let result = assert_grounded_answer_execution_readiness_deterministic_and_path_free(
            &temp,
            execution_readiness_request(
                "alpha grounded evidence",
                Some("Alpha beta. Alpha beta gamma."),
                vec![format!("  {source_id}  ")],
                Some(&format!("  {answer_draft_id}  ")),
                true,
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerExecutionReadinessStatus::Blocked);
        assert_eq!(result.build_preflight_status, ScholarChatGroundedAnswerBuildPreflightStatus::Blocked);
        assert!(!result.answer_draft_present);
        assert!(!result.answer_draft_readable);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "answer_draft_missing"));
    }

    #[test]
    fn scholar_chat_grounded_answer_execution_readiness_blocks_when_answer_draft_malformed() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, answer_draft_id, version_id, _claim_count) = build_readable_answer_draft_fixture(&temp);
        let draft_path = crate::corpus_paths::CorpusPaths::new(temp.path())
            .source_version_dir(&source_id, &version_id)
            .join("answer_drafts")
            .join(format!("{answer_draft_id}.json"));
        fs::write(&draft_path, "{not valid json").unwrap();
        let result = assert_grounded_answer_execution_readiness_deterministic_and_path_free(
            &temp,
            execution_readiness_request(
                "alpha grounded evidence",
                Some("Alpha beta. Alpha beta gamma."),
                vec![format!("  {source_id}  ")],
                Some(&format!("  {answer_draft_id}  ")),
                true,
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerExecutionReadinessStatus::Blocked);
        assert_eq!(result.build_preflight_status, ScholarChatGroundedAnswerBuildPreflightStatus::Blocked);
        assert!(result.answer_draft_present);
        assert!(!result.answer_draft_readable);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "answer_draft_unreadable"));
    }

    #[test]
    fn scholar_chat_grounded_answer_execution_readiness_blocks_without_execution_consent() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, answer_draft_id, _version_id, claim_count) = build_readable_answer_draft_fixture(&temp);
        let result = assert_grounded_answer_execution_readiness_deterministic_and_path_free(
            &temp,
            execution_readiness_request(
                "  alpha grounded evidence  ",
                Some("  Alpha beta. Alpha beta gamma.  "),
                vec![format!("  {source_id}  ")],
                Some(&format!("  {answer_draft_id}  ")),
                true,
                false,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerExecutionReadinessStatus::Blocked);
        assert_eq!(result.build_preflight_status, ScholarChatGroundedAnswerBuildPreflightStatus::PreflightReadyLater);
        assert_eq!(result.answer_draft_id.as_deref(), Some(answer_draft_id.as_str()));
        assert_eq!(result.selected_source_ids, vec![source_id]);
        assert_eq!(result.answer_draft_claim_count, claim_count);
        assert_eq!(result.missing_inputs, vec!["execution_consent".to_string()]);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "execution_consent_missing"));
    }

    #[test]
    fn scholar_chat_grounded_answer_execution_readiness_is_ready_later_only_when_execution_consent_is_given() {
        let temp = tempfile::tempdir().unwrap();
        let (source_id, answer_draft_id, _version_id, claim_count) = build_readable_answer_draft_fixture(&temp);
        let result = assert_grounded_answer_execution_readiness_deterministic_and_path_free(
            &temp,
            execution_readiness_request(
                "  alpha grounded evidence  ",
                Some("  Alpha beta. Alpha beta gamma.  "),
                vec![format!("  {source_id}  ")],
                Some(&format!("  {answer_draft_id}  ")),
                true,
                true,
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerExecutionReadinessStatus::ExecutionReadyLater);
        assert_eq!(result.build_preflight_status, ScholarChatGroundedAnswerBuildPreflightStatus::PreflightReadyLater);
        assert_eq!(result.answer_draft_id.as_deref(), Some(answer_draft_id.as_str()));
        assert_eq!(result.selected_source_ids, vec![source_id]);
        assert_eq!(result.answer_draft_claim_count, claim_count);
        assert!(result.missing_inputs.is_empty());
        assert!(result
            .readiness_reasons
            .iter()
            .any(|reason| reason.contains("Execution consent: true")));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("future GroundedAnswer service call")));
    }

    #[test]
    fn scholar_chat_grounded_answer_write_eligibility_rejects_empty_prompt() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_scholar_chat_grounded_answer_write_eligibility(
            temp.path(),
            grounding_request("   ", Some("Alpha beta."), vec![]),
        );
        assert!(matches!(result, Err(AegisError::ScholarChatPromptEmpty)));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn scholar_chat_grounded_answer_write_eligibility_blocks_without_selected_sources() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_grounded_answer_write_eligibility_deterministic_and_path_free(
            &temp,
            grounding_request("alpha grounded evidence", Some("Alpha beta."), vec![]),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::Blocked);
        assert_eq!(result.candidate_statement_count, 0);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "needs_sources"));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("Select Scholar Chat source context")));
    }

    #[test]
    fn scholar_chat_grounded_answer_write_eligibility_rejects_invalid_source_ids_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["", " ", "..", "../evil", "evil/source", "evil\\source"] {
            let result = preview_scholar_chat_grounded_answer_write_eligibility(
                temp.path(),
                grounding_request("alpha grounded evidence", Some("Alpha beta."), vec![invalid.to_string()]),
            );
            assert!(matches!(result, Err(AegisError::ScholarChatInvalidSourceId)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn scholar_chat_grounded_answer_write_eligibility_blocks_without_draft_text() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_write_eligibility_deterministic_and_path_free(
            &temp,
            grounding_request("alpha grounded evidence", Some("   "), vec![source_id]),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::Blocked);
        assert_eq!(result.candidate_statement_count, 0);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "draft_text_missing"));
        assert!(result.summary.contains("blocked"));
    }

    #[test]
    fn scholar_chat_grounded_answer_write_eligibility_blocks_without_evidence_candidates() {
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

        let result = assert_grounded_answer_write_eligibility_deterministic_and_path_free(
            &temp,
            grounding_request(
                "alpha grounded evidence",
                Some("Alpha beta."),
                vec![source.source_id.clone()],
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::Blocked);
        assert_eq!(result.candidate_statement_count, 0);
        assert_eq!(result.evidence_candidate_count, 0);
        assert!(result
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "needs_evidence_candidates"));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("Add local evidence candidates")));
    }

    #[test]
    fn scholar_chat_grounded_answer_write_eligibility_needs_review_when_candidate_needs_review() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let result = assert_grounded_answer_write_eligibility_deterministic_and_path_free(
            &temp,
            grounding_request(
                "alpha grounded evidence",
                Some("The alpha."),
                vec![source_id],
            ),
        );
        assert_eq!(result.status, ScholarChatGroundedAnswerWriteEligibilityStatus::NeedsReview);
        assert_eq!(result.candidate_status, ScholarChatGroundedAnswerCandidateStatus::NeedsReview);
        assert_eq!(result.candidate_statement_count, 0);
        assert_eq!(result.supported_item_count, 0);
        assert_eq!(result.weakly_supported_item_count, 1);
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.kind == "needs_review"));
        assert!(result
            .eligibility_reasons
            .iter()
            .any(|reason| reason.contains("Weakly supported or unsupported")));
    }

    #[test]
    fn scholar_chat_grounded_answer_write_eligibility_is_ready_only_when_candidate_ready_later_and_statements_exist() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source_with_index(&temp, "alpha beta gamma\nalpha beta delta\n");
        let request = grounding_request(
            "alpha grounded evidence",
            Some("Alpha beta. Alpha beta gamma."),
            vec![source_id.clone()],
        );
        let candidate_preview = assert_grounded_answer_candidate_deterministic_and_path_free(&temp, request.clone());
        let write_eligibility_preview = assert_grounded_answer_write_eligibility_deterministic_and_path_free(&temp, request);
        assert_eq!(candidate_preview.status, ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater);
        assert_eq!(write_eligibility_preview.status, ScholarChatGroundedAnswerWriteEligibilityStatus::WriteEligibleLater);
        assert_eq!(write_eligibility_preview.candidate_status, ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater);
        assert!(write_eligibility_preview.candidate_statement_count > 0);
        assert_eq!(write_eligibility_preview.candidate_statement_count, candidate_preview.candidate_statement_count);
        assert_eq!(write_eligibility_preview.selected_source_count, candidate_preview.selected_source_count);
        assert_eq!(write_eligibility_preview.evidence_candidate_count, candidate_preview.evidence_candidate_count);
        assert_eq!(write_eligibility_preview.inspected_item_count, candidate_preview.inspected_item_count);
        assert_eq!(write_eligibility_preview.supported_item_count, candidate_preview.supported_item_count);
        assert_eq!(write_eligibility_preview.weakly_supported_item_count, candidate_preview.weakly_supported_item_count);
        assert_eq!(write_eligibility_preview.unsupported_item_count, candidate_preview.unsupported_item_count);
        assert!(write_eligibility_preview
            .eligibility_reasons
            .iter()
            .any(|reason| reason.contains("All inspected items were supported by local evidence")));
        assert!(write_eligibility_preview
            .next_required_actions
            .iter()
            .any(|action| action.contains("A GroundedAnswer write implementation can be added later")));
    }

    #[test]
    fn scholar_chat_grounded_answer_write_eligibility_rejects_zero_candidate_statements_even_when_candidate_status_is_ready_later() {
        let candidate_preview = ScholarChatGroundedAnswerCandidatePreview {
            status: ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater,
            normalized_prompt: "alpha grounded evidence".to_string(),
            selected_source_count: 1,
            evidence_candidate_count: 1,
            inspected_item_count: 1,
            supported_item_count: 1,
            weakly_supported_item_count: 0,
            unsupported_item_count: 0,
            candidate_statement_count: 0,
            summary: "candidate preview".to_string(),
            candidate_items: Vec::new(),
            preview_only: true,
            not_answer_draft: true,
            not_grounded_answer: true,
            not_final_answer: true,
            no_answer_artifact_created: true,
            no_evidence_pack_built: true,
            no_persistence: true,
            no_llm_call: true,
            no_runtime_execution: true,
            blockers: Vec::new(),
            warnings: Vec::new(),
            next_required_actions: Vec::new(),
        };
        let preview = grounded_answer_write_eligibility_preview_from_candidate_preview(candidate_preview);
        assert_eq!(preview.status, ScholarChatGroundedAnswerWriteEligibilityStatus::Blocked);
        assert_eq!(preview.candidate_status, ScholarChatGroundedAnswerCandidateStatus::CandidateReadyLater);
        assert_eq!(preview.candidate_statement_count, 0);
        assert!(preview
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "candidate_statements_missing"));
        assert!(preview
            .eligibility_reasons
            .iter()
            .any(|reason| reason.contains("No candidate statements were available")));
        assert_grounded_answer_write_eligibility_boundary_fields(&preview);
    }
}
