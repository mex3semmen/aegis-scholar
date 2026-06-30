import { createSignal, onMount, Setter } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import WorkspaceShell from "./workspaces/WorkspaceShell";
import ScholarChatWorkspace from "./workspaces/ScholarChatWorkspace";
import SourcesWorkspace from "./workspaces/SourcesWorkspace";
import EvidencePacksWorkspace from "./workspaces/EvidencePacksWorkspace";

type CorpusStatus = {
  source_count: number;
  registered_count: number;
  extracted_count: number;
  failed_count: number;
};

type RegisteredSource = {
  source_id: string;
  version_id: string;
  title: string;
  source_type: string;
  ingestion_status: string;
};

const RETRIEVAL_SEARCH_DISPLAY_LIMIT = 10;

type ScholarChatMode =
  | "lecture_learning"
  | "thesis_writing"
  | "literature_review"
  | "flashcards"
  | "statistics_methods"
  | "general_scholar";

type GroundingPolicy =
  | "local_only"
  | "local_first"
  | "allow_marked_model_knowledge"
  | "external_adapters_later";

type ScholarChatRequest = {
  prompt: string;
  mode: ScholarChatMode;
  grounding_policy: GroundingPolicy;
  selected_source_ids: string[];
};

type WorkspaceSection = "scholar_chat" | "sources" | "evidence_packs" | "developer_diagnostics";

const WORKSPACE_SECTIONS: { value: WorkspaceSection; label: string; targetId: string; description: string }[] = [
  {
    value: "scholar_chat",
    label: "Scholar Chat",
    targetId: "scholar-chat",
    description: "Primary chat-first workflow surface.",
  },
  {
    value: "sources",
    label: "Sources",
    targetId: "sources",
    description: "Source readiness and registration context.",
  },
  {
    value: "evidence_packs",
    label: "Evidence Packs",
    targetId: "evidence-packs",
    description: "Guarded local evidence-pack inspection.",
  },
  {
    value: "developer_diagnostics",
    label: "Developer Diagnostics",
    targetId: "developer-diagnostics",
    description: "Runtime setup, preview, and inspection.",
  },
];

const SCHOLAR_CHAT_PROMPT_SUGGESTIONS: { label: string; prompt: string }[] = [
  {
    label: "Summarize a paper",
    prompt: "Summarize the main argument, method, and limitations of this paper.",
  },
  {
    label: "Plan a literature review",
    prompt: "Help me plan a local, source-grounded literature review around this topic.",
  },
  {
    label: "Explain a method",
    prompt: "Explain this method in plain language and note what local sources I should inspect next.",
  },
  {
    label: "Prepare evidence",
    prompt: "Outline the evidence I should gather before building a local evidence pack.",
  },
];

type ScholarChatTranscriptRole = "user" | "assistant" | "system";
type ScholarChatTranscriptKind = "prompt" | "workflow_preview" | "execution_gate" | "system";

type ScholarChatTranscriptMessage = {
  id: number;
  role: ScholarChatTranscriptRole;
  kind: ScholarChatTranscriptKind;
  prompt: string;
  title: string;
  content: string;
  created_at: number;
  workflow_preview?: ScholarChatAgenticWorkflowPlanPreview;
  execution_gate_preview?: ScholarChatAgenticWorkflowExecutionGatePreview;
};

const SCHOLAR_CHAT_MODES: { value: ScholarChatMode; label: string }[] = [
  { value: "lecture_learning", label: "Lecture learning" },
  { value: "thesis_writing", label: "Thesis writing" },
  { value: "literature_review", label: "Literature review" },
  { value: "flashcards", label: "Flashcards" },
  { value: "statistics_methods", label: "Statistics / methods" },
  { value: "general_scholar", label: "General scholar" },
];

const GROUNDING_POLICIES: { value: GroundingPolicy; label: string }[] = [
  { value: "local_only", label: "Local only" },
  { value: "local_first", label: "Local first" },
  { value: "allow_marked_model_knowledge", label: "Allow marked model knowledge" },
  { value: "external_adapters_later", label: "External adapters later" },
];

type ScholarChatGroundingPlan = {
  selected_source_count: number;
  local_corpus_required: boolean;
  retrieval_would_run: boolean;
  evidence_pack_would_be_required: boolean;
  model_knowledge_allowed: boolean;
  external_adapters_available: boolean;
  summary: string;
  steps: string[];
};

type ScholarChatResponse = {
  status: "preview_only";
  normalized_prompt: string;
  mode: ScholarChatMode;
  grounding_policy: GroundingPolicy;
  selected_source_ids: string[];
  selected_source_count: number;
  grounding_plan: ScholarChatGroundingPlan;
  warnings: string[];
};

type ScholarChatAgenticWorkflowPlanStatus = "blocked" | "needs_review" | "plan_ready_later";

type ScholarChatAgenticWorkflowIntent =
  | "source_registration_needed"
  | "extract_text"
  | "chunk_source"
  | "build_or_inspect_retrieval"
  | "build_evidence_pack"
  | "inspect_evidence_pack"
  | "ask_local_sources"
  | "explain_blocker"
  | "unknown_or_unsupported";

type ScholarChatAgenticWorkflowPlanPreview = {
  status: ScholarChatAgenticWorkflowPlanStatus;
  recognized_intent: ScholarChatAgenticWorkflowIntent;
  normalized_prompt: string;
  mode: ScholarChatMode;
  grounding_policy: GroundingPolicy;
  selected_source_ids: string[];
  selected_source_count: number;
  required_local_context: string[];
  planned_steps: string[];
  blockers: string[];
  warnings: string[];
  next_required_actions: string[];
  summary: string;
  execution_allowed: boolean;
  preview_only: boolean;
  no_runtime_execution: boolean;
  no_llm_call: boolean;
  no_answer_generated: boolean;
  no_evidence_pack_built: boolean;
  no_persistence: boolean;
  no_artifact_write: boolean;
  no_registry_status_change: boolean;
  no_audit_write: boolean;
};

type ScholarChatAgenticWorkflowExecutionGateStatus = "blocked" | "needs_review" | "execution_ready_later";

type ScholarChatAgenticWorkflowExecutionGateDecision =
  | "blocked"
  | "needs_context"
  | "needs_consent"
  | "ready_later"
  | "no_action_available";

type ScholarChatAgenticWorkflowFutureAction =
  | "register_source_later"
  | "extract_text_later"
  | "chunk_source_later"
  | "inspect_retrieval_later"
  | "build_evidence_pack_later"
  | "inspect_evidence_pack_later"
  | "ask_local_sources_later"
  | "no_action_available";

type ScholarChatAgenticWorkflowExecutionGatePreview = {
  status: ScholarChatAgenticWorkflowExecutionGateStatus;
  planned_intent: ScholarChatAgenticWorkflowIntent;
  gate_decision: ScholarChatAgenticWorkflowExecutionGateDecision;
  consent_required: boolean;
  user_consent_present: boolean;
  allowed_future_action: ScholarChatAgenticWorkflowFutureAction;
  blocked_reason: string;
  blockers: string[];
  warnings: string[];
  required_local_context: string[];
  planned_steps: string[];
  next_required_actions: string[];
  safety_invariants: string[];
  selected_source_ids: string[];
  selected_source_count: number;
  execution_allowed_now: boolean;
  preview_only: boolean;
  no_filesystem_write: boolean;
  no_backend_mutation: boolean;
  no_runtime_execution: boolean;
  no_llm_call: boolean;
  no_network_call: boolean;
};

type ScholarChatRetrievalCandidate = {
  source_id: string;
  version_id: string;
  chunk_id: string;
  score: number;
  matched_terms: string[];
  preview: string;
  locator: CitationLocator;
};

type ScholarChatRetrievalPreviewResponse = {
  status: "preview_only";
  normalized_prompt: string;
  mode: ScholarChatMode;
  grounding_policy: GroundingPolicy;
  selected_source_ids: string[];
  selected_source_count: number;
  candidate_count: number;
  candidates: ScholarChatRetrievalCandidate[];
  warnings: string[];
};

type ScholarChatEvidenceCandidate = {
  source_id: string;
  version_id: string;
  chunk_id: string;
  score: number;
  matched_terms: string[];
  preview: string;
  locator: CitationLocator;
};

type ScholarChatEvidencePlan = {
  retrieval_candidate_count: number;
  evidence_candidate_count: number;
  evidence_required: boolean;
  evidence_pack_would_be_built_later: boolean;
  summary: string;
  steps: string[];
};

type ScholarChatEvidencePlanResponse = {
  status: "evidence_plan_preview";
  normalized_prompt: string;
  mode: ScholarChatMode;
  grounding_policy: GroundingPolicy;
  selected_source_ids: string[];
  selected_source_count: number;
  retrieval_candidate_count: number;
  evidence_candidate_count: number;
  evidence_plan: ScholarChatEvidencePlan;
  candidates: ScholarChatEvidenceCandidate[];
  warnings: string[];
};

type ScholarChatPromptPackStatus = "prompt_pack_preview";

type ScholarChatPromptPackSectionKind =
  | "system_or_policy_instructions"
  | "mode_instructions"
  | "grounding_instructions"
  | "source_context"
  | "user_prompt";

type ScholarChatPromptPackSection = {
  kind: ScholarChatPromptPackSectionKind;
  title: string;
  lines: string[];
};

type ScholarChatPromptContextItem = {
  source_id: string;
  version_id: string;
  chunk_id: string;
  score: number;
  matched_terms: string[];
  preview: string;
  locator: CitationLocator;
};

type ScholarChatPromptPack = {
  section_count: number;
  context_item_count: number;
  estimated_input_char_count: number;
  sections: ScholarChatPromptPackSection[];
};

type ScholarChatPromptPackPreviewResponse = {
  status: ScholarChatPromptPackStatus;
  normalized_prompt: string;
  mode: ScholarChatMode;
  grounding_policy: GroundingPolicy;
  selected_source_ids: string[];
  selected_source_count: number;
  evidence_candidate_count: number;
  prompt_pack: ScholarChatPromptPack;
  context_items: ScholarChatPromptContextItem[];
  warnings: string[];
};

type ScholarChatScientificMetadataQueryPlanStatus =
  | "blocked"
  | "query_plan_ready"
  | "needs_provider_config"
  | "needs_provider_selection"
  | "needs_query_goal"
  | "needs_network_policy_review"
  | "needs_provider_terms_review"
  | "needs_institutional_access_review"
  | "unsupported_provider";

type ScholarChatScientificMetadataProviderRequestStatus =
  | "blocked"
  | "needs_provider_config"
  | "needs_provider_selection"
  | "needs_query_goal"
  | "needs_network_policy_review"
  | "needs_provider_terms_review"
  | "needs_institutional_access_review"
  | "unsupported_provider"
  | "provider_request_ready_later";

type ScholarChatScientificMetadataProviderRequestStrategy =
  | "blocked"
  | "provider_policy_review_first"
  | "public_metadata_request_preview"
  | "institutional_boundary_request_preview"
  | "provider_request_preview_only";

type ScholarChatScientificMetadataQueryPlanPreviewRequest = {
  query: string;
  mode?: string | null;
  context_tags?: string[] | null;
  preferred_metadata_sources?: string[] | null;
  preferred_psychology_source_families?: string[] | null;
  provider_override?: string[] | null;
  query_goal?: string | null;
  require_open_access?: boolean | null;
  require_doi?: boolean | null;
  year_from?: number | null;
  year_to?: number | null;
  include_disabled_providers?: boolean;
  include_institutional_providers?: boolean;
  include_rate_limit_notes?: boolean;
  include_attribution_requirements?: boolean;
  include_query_templates?: boolean;
  include_filter_plan?: boolean;
  include_result_field_plan?: boolean;
  execution_requested?: boolean;
  allow_network?: boolean;
  allow_provider_terms_unreviewed?: boolean;
  allow_metadata_record_write?: boolean;
};

type ScholarChatScientificMetadataProviderRequestPreviewRequest = {
  query_plan_preview_request: ScholarChatScientificMetadataQueryPlanPreviewRequest;
  include_request_templates?: boolean;
  include_header_plan?: boolean;
  include_param_plan?: boolean;
  include_body_plan?: boolean;
};

type ScholarChatScientificMetadataProviderRequestPreview = {
  status: ScholarChatScientificMetadataProviderRequestStatus;
  normalized_query: string;
  normalized_mode: string | null;
  normalized_context_tags: string[];
  normalized_preferred_metadata_sources: string[];
  normalized_preferred_psychology_source_families: string[];
  normalized_provider_override: string[] | null;
  unknown_provider_ids: string[];
  normalized_query_goal: string | null;
  include_disabled_providers: boolean;
  include_institutional_providers: boolean;
  include_rate_limit_notes: boolean;
  include_attribution_requirements: boolean;
  include_query_templates: boolean;
  include_filter_plan: boolean;
  include_result_field_plan: boolean;
  include_request_templates: boolean;
  include_header_plan: boolean;
  include_param_plan: boolean;
  include_body_plan: boolean;
  execution_requested: boolean;
  allow_network: boolean;
  allow_provider_terms_unreviewed: boolean;
  allow_metadata_record_write: boolean;
  query_plan_status: ScholarChatScientificMetadataQueryPlanStatus;
  query_plan_strategy: string;
  selected_provider_ids: string[];
  selected_provider_count: number;
  public_metadata_provider_ids: string[];
  institutional_boundary_provider_ids: string[];
  provider_request_strategy: ScholarChatScientificMetadataProviderRequestStrategy;
  blockers: string[];
  warnings: string[];
  next_required_actions: string[];
  summary: string;
  preview_only: boolean;
  metadata_provider_request_preview_only: boolean;
  dry_run_only: boolean;
  execution_disabled: boolean;
  no_url_building: boolean;
  no_network_call: boolean;
  no_http_client: boolean;
  no_api_key_read: boolean;
  no_environment_read: boolean;
  no_scraping: boolean;
  no_connector_call: boolean;
  no_source_import: boolean;
  no_metadata_record_write: boolean;
  no_metadata_persistence: boolean;
  no_retrieval_execution: boolean;
  no_model_loading: boolean;
  no_runtime_inference: boolean;
  no_llm_call: boolean;
  no_answer_generated: boolean;
  no_literature_review_created: boolean;
  no_evidence_pack_created: boolean;
  no_artifact_write: boolean;
  no_persistence: boolean;
  no_registry_status_change: boolean;
  no_audit_write: boolean;
};

const OPENALEX_READONLY_PANEL_BOUNDARY_CHECKLIST = [
  { label: "Provider request preview command", value: "wired" },
  { label: "OpenAlex execution command", value: "not wired" },
  { label: "Cache/write gate command", value: "not wired" },
  { label: "Write button", value: "absent" },
  { label: "Automatic downstream actions", value: "absent" },
  { label: "Network allowed", value: "false" },
  { label: "Metadata writes allowed", value: "false" },
] as const;

type ScholarChatAnswerReadinessStatus =
  | "blocked"
  | "needs_sources"
  | "needs_retrieval_index"
  | "needs_evidence_candidates"
  | "needs_runtime_config"
  | "needs_execution_consent"
  | "ready_for_draft_inference_later"
  | "ready_for_grounded_draft_later";

type ScholarChatAnswerReadinessOutputClassification =
  | "blocked"
  | "ungrounded_draft"
  | "source_context_draft"
  | "grounded_draft_candidate";

type ScholarChatAnswerReadinessBlocker = {
  kind: string;
  message: string;
};

type ScholarChatAnswerReadinessWarning = {
  kind: string;
  message: string;
};

type ScholarChatAnswerReadinessRequest = {
  scholar_chat_request: ScholarChatRequest;
  runtime_config: LocalModelRuntimeConfig;
  allow_model_execution: boolean;
};

type ScholarChatAnswerReadinessPreview = {
  status: ScholarChatAnswerReadinessStatus;
  normalized_prompt: string;
  mode: ScholarChatMode;
  grounding_policy: GroundingPolicy;
  selected_source_count: number;
  retrieval_candidate_count: number;
  evidence_candidate_count: number;
  prompt_pack_ready: boolean;
  runtime_health_status: LocalModelRuntimeHealthStatus;
  invocation_plan_status: LocalRuntimeInvocationPlanStatus;
  allow_model_execution: boolean;
  would_generate_answer_now: boolean;
  would_build_evidence_pack_now: boolean;
  would_create_final_answer_now: boolean;
  future_output_classification: ScholarChatAnswerReadinessOutputClassification;
  blockers: ScholarChatAnswerReadinessBlocker[];
  warnings: ScholarChatAnswerReadinessWarning[];
  next_required_actions: string[];
};

type ScholarChatDraftInferenceStatus =
  | "blocked"
  | "needs_sources"
  | "needs_evidence"
  | "needs_runtime_config"
  | "needs_execution_consent"
  | "inference_succeeded"
  | "inference_failed"
  | "timed_out";

type ScholarChatDraftOutputClassification =
  | "blocked"
  | "ungrounded_model_draft"
  | "source_context_draft"
  | "grounded_draft_candidate";

type ScholarChatDraftInferenceBlocker = {
  kind: string;
  message: string;
};

type ScholarChatDraftInferenceWarning = {
  kind: string;
  message: string;
};

type ScholarChatDraftInferenceRequest = {
  scholar_chat_request: ScholarChatRequest;
  runtime_config: LocalModelRuntimeConfig;
  allow_model_execution: boolean;
  timeout_ms: number | null;
  max_output_tokens: number | null;
};

type ScholarChatDraftInferencePreview = {
  status: ScholarChatDraftInferenceStatus;
  output_classification: ScholarChatDraftOutputClassification;
  normalized_prompt: string;
  mode: ScholarChatMode;
  grounding_policy: GroundingPolicy;
  selected_source_count: number;
  retrieval_candidate_count: number;
  evidence_candidate_count: number;
  prompt_pack_section_count: number;
  prompt_char_count: number;
  runtime_health_status: LocalModelRuntimeHealthStatus;
  invocation_plan_status: LocalRuntimeInvocationPlanStatus;
  allow_model_execution: boolean;
  execution_attempted: boolean;
  safe_model_file_name: string | null;
  safe_executable_file_name: string | null;
  stdout_preview: string;
  stderr_preview: string;
  duration_ms: number;
  exit_code: number | null;
  draft_only: boolean;
  preview_only: boolean;
  not_final_answer: boolean;
  not_grounded_answer: boolean;
  no_answer_artifact_created: boolean;
  no_evidence_pack_built: boolean;
  no_persistence: boolean;
  blockers: ScholarChatDraftInferenceBlocker[];
  warnings: ScholarChatDraftInferenceWarning[];
};

type ScholarChatDraftGroundingInspectionStatus =
  | "blocked"
  | "no_draft_text"
  | "no_evidence_candidates"
  | "inspected";

type ScholarChatDraftGroundingSupportStatus =
  | "not_inspected"
  | "unsupported"
  | "weakly_supported"
  | "supported_by_local_evidence";

type ScholarChatDraftGroundingInspectionBlocker = {
  kind: string;
  message: string;
};

type ScholarChatDraftGroundingInspectionWarning = {
  kind: string;
  message: string;
};

type ScholarChatDraftGroundingInspectionRequest = {
  scholar_chat_request: ScholarChatRequest;
  draft_text: string | null;
  max_items: number | null;
};

type ScholarChatDraftGroundingInspectionItem = {
  item_index: number;
  text_preview: string;
  support_status: ScholarChatDraftGroundingSupportStatus;
  matched_evidence_count: number;
  source_ids: string[];
  locator_previews: string[];
};

type ScholarChatDraftGroundingInspectionPreview = {
  status: ScholarChatDraftGroundingInspectionStatus;
  normalized_prompt: string;
  draft_char_count: number;
  selected_source_count: number;
  evidence_candidate_count: number;
  inspected_item_count: number;
  unsupported_item_count: number;
  weakly_supported_item_count: number;
  supported_item_count: number;
  items: ScholarChatDraftGroundingInspectionItem[];
  inspection_only: boolean;
  not_grounded_answer: boolean;
  not_final_answer: boolean;
  no_evidence_pack_built: boolean;
  no_answer_artifact_created: boolean;
  no_persistence: boolean;
  no_llm_call: boolean;
  no_runtime_execution: boolean;
  blockers: ScholarChatDraftGroundingInspectionBlocker[];
  warnings: ScholarChatDraftGroundingInspectionWarning[];
};

type ScholarChatGroundedDraftReadinessStatus =
  | "blocked"
  | "needs_review"
  | "ready_for_grounded_draft_later";

type ScholarChatGroundedDraftReadinessPreview = {
  status: ScholarChatGroundedDraftReadinessStatus;
  inspection_status: ScholarChatDraftGroundingInspectionStatus;
  normalized_prompt: string;
  selected_source_count: number;
  evidence_candidate_count: number;
  inspected_item_count: number;
  supported_item_count: number;
  weakly_supported_item_count: number;
  unsupported_item_count: number;
  summary: string;
  preview_only: boolean;
  not_grounded_answer: boolean;
  not_final_answer: boolean;
  no_answer_artifact_created: boolean;
  no_evidence_pack_built: boolean;
  no_persistence: boolean;
  no_llm_call: boolean;
  no_runtime_execution: boolean;
  blockers: ScholarChatDraftGroundingInspectionBlocker[];
  warnings: ScholarChatDraftGroundingInspectionWarning[];
  next_required_actions: string[];
};

type LocalRuntimeVersionProbeStatus = "blocked" | "probe_succeeded" | "probe_failed" | "timed_out";

type LocalRuntimeVersionProbePreviewRequest = {
  probe_readiness_preview_request: LocalRuntimeProbeReadinessPreviewRequest;
  allow_probe_execution: boolean;
  timeout_ms: number | null;
};

type LocalRuntimeVersionProbePreview = {
  status: LocalRuntimeVersionProbeStatus;
  probe_readiness_status: LocalRuntimeProbeReadinessStatus;
  validation_status: LocalRuntimeValidationStatus;
  adapter_contract_status: LocalRuntimeAdapterContractStatus;
  adapter_kind: LocalRuntimeAdapterKind;
  normalized_model_family: string | null;
  normalized_model_format: string;
  safe_executable_file_name: string | null;
  safe_model_file_name: string | null;
  probe_consent: boolean;
  allow_probe_execution: boolean;
  execution_attempted: boolean;
  probe_argument: string;
  timeout_ms: number;
  duration_ms: number;
  exit_code: number | null;
  stdout_preview: string;
  stderr_preview: string;
  stdout_truncated: boolean;
  stderr_truncated: boolean;
  blockers: LocalRuntimeProbeWarning[];
  warnings: LocalRuntimeProbeWarning[];
  next_required_actions: string[];
  summary: string;
  preview_only: boolean;
  no_model_load: boolean;
  no_model_path_argument: boolean;
  no_llm_call: boolean;
  no_runtime_inference: boolean;
  no_persistence: boolean;
  no_artifact_write: boolean;
  no_registry_status_change: boolean;
  no_audit_write: boolean;
};

type LocalRuntimeCapabilityStatus = "blocked" | "needs_review" | "capability_ready_later";

type LocalRuntimeCapabilityPreviewRequest = {
  version_probe_preview_request: LocalRuntimeVersionProbePreviewRequest;
};

type LocalRuntimeCapabilityPreview = {
  status: LocalRuntimeCapabilityStatus;
  version_probe_status: LocalRuntimeVersionProbeStatus;
  probe_readiness_status: LocalRuntimeProbeReadinessStatus;
  validation_status: LocalRuntimeValidationStatus;
  adapter_contract_status: LocalRuntimeAdapterContractStatus;
  adapter_kind: LocalRuntimeAdapterKind;
  normalized_model_family: string | null;
  normalized_model_format: string;
  safe_executable_file_name: string | null;
  safe_model_file_name: string | null;
  probe_consent: boolean;
  allow_probe_execution: boolean;
  version_probe_execution_attempted: boolean;
  version_probe_exit_code: number | null;
  version_probe_timed_out: boolean;
  version_probe_stdout_preview: string;
  version_probe_stderr_preview: string;
  inferred_runtime_available: boolean;
  inferred_version_text: string | null;
  capability_reasons: string[];
  blockers: LocalRuntimeProbeWarning[];
  warnings: LocalRuntimeProbeWarning[];
  next_required_actions: string[];
  summary: string;
  preview_only: boolean;
  no_new_process_spawn: boolean;
  no_binary_probe_beyond_wrapped_version_probe: boolean;
  no_model_path_argument: boolean;
  no_model_file_read: boolean;
  no_model_load: boolean;
  no_runtime_inference: boolean;
  no_smoke_inference: boolean;
  no_llm_call: boolean;
  no_persistence: boolean;
  no_artifact_write: boolean;
  no_registry_status_change: boolean;
  no_audit_write: boolean;
};

type LocalRuntimeSmokeReadinessStatus = "blocked" | "needs_review" | "smoke_ready_later";

type LocalRuntimeSmokeReadinessPreviewRequest = {
  capability_preview_request: LocalRuntimeCapabilityPreviewRequest;
  smoke_consent: boolean;
  diagnostic_prompt: string | null;
  max_output_tokens: number | null;
  timeout_ms: number | null;
};

type LocalRuntimeSmokeReadinessPreview = {
  status: LocalRuntimeSmokeReadinessStatus;
  capability_status: LocalRuntimeCapabilityStatus;
  version_probe_status: LocalRuntimeVersionProbeStatus;
  probe_readiness_status: LocalRuntimeProbeReadinessStatus;
  validation_status: LocalRuntimeValidationStatus;
  adapter_contract_status: LocalRuntimeAdapterContractStatus;
  adapter_kind: LocalRuntimeAdapterKind;
  normalized_model_family: string | null;
  normalized_model_format: string;
  safe_executable_file_name: string | null;
  safe_model_file_name: string | null;
  probe_consent: boolean;
  allow_probe_execution: boolean;
  smoke_consent: boolean;
  normalized_diagnostic_prompt: string;
  diagnostic_prompt_char_count: number;
  max_output_tokens: number;
  timeout_ms: number;
  required_inputs: string[];
  missing_inputs: string[];
  readiness_reasons: string[];
  blockers: LocalRuntimeProbeWarning[];
  warnings: LocalRuntimeProbeWarning[];
  next_required_actions: string[];
  summary: string;
  preview_only: boolean;
  no_new_process_spawn: boolean;
  no_smoke_inference_execution: boolean;
  no_model_path_argument: boolean;
  no_model_file_read: boolean;
  no_model_load: boolean;
  no_llm_call: boolean;
  no_persistence: boolean;
  no_artifact_write: boolean;
  no_registry_status_change: boolean;
  no_audit_write: boolean;
  diagnostic_only: boolean;
  not_scholar_chat_answer: boolean;
  no_answer_generated: boolean;
  no_grounding_applied: boolean;
  no_evidence_pack_used: boolean;
};

type LocalRuntimeSmokeExecutionPlanStatus = "blocked" | "needs_review" | "plan_ready_later";

type LocalRuntimeSmokeExecutionPlanPreviewRequest = {
  smoke_readiness_preview_request: LocalRuntimeSmokeReadinessPreviewRequest;
};

type LocalRuntimeSmokeExecutionPlanPreview = {
  status: LocalRuntimeSmokeExecutionPlanStatus;
  smoke_readiness_status: LocalRuntimeSmokeReadinessStatus;
  capability_status: LocalRuntimeCapabilityStatus;
  version_probe_status: LocalRuntimeVersionProbeStatus;
  probe_readiness_status: LocalRuntimeProbeReadinessStatus;
  validation_status: LocalRuntimeValidationStatus;
  adapter_contract_status: LocalRuntimeAdapterContractStatus;
  adapter_kind: LocalRuntimeAdapterKind;
  normalized_model_family: string | null;
  normalized_model_format: string;
  safe_executable_file_name: string | null;
  safe_model_file_name: string | null;
  probe_consent: boolean;
  allow_probe_execution: boolean;
  smoke_consent: boolean;
  normalized_diagnostic_prompt: string;
  diagnostic_prompt_char_count: number;
  max_output_tokens: number;
  timeout_ms: number;
  planned_operation: string;
  planned_inputs: string[];
  planned_safe_arguments: string[];
  planned_outputs: string[];
  required_inputs: string[];
  missing_inputs: string[];
  plan_reasons: string[];
  blockers: LocalRuntimeProbeWarning[];
  warnings: LocalRuntimeProbeWarning[];
  next_required_actions: string[];
  summary: string;
  preview_only: boolean;
  no_process_spawn: boolean;
  no_smoke_inference_execution: boolean;
  no_model_file_read: boolean;
  no_model_load: boolean;
  no_llm_call: boolean;
  no_persistence: boolean;
  no_artifact_write: boolean;
  no_registry_status_change: boolean;
  no_audit_write: boolean;
  diagnostic_only: boolean;
  not_scholar_chat_answer: boolean;
  no_answer_generated: boolean;
  no_grounding_applied: boolean;
  no_evidence_pack_used: boolean;
};

type ScholarChatGroundedAnswerBuildPlanStatus =
  | "blocked"
  | "needs_review"
  | "plan_ready_later";

type ScholarChatGroundedAnswerBuildPlanPreview = {
  status: ScholarChatGroundedAnswerBuildPlanStatus;
  readiness_status: ScholarChatGroundedDraftReadinessStatus;
  normalized_prompt: string;
  selected_source_count: number;
  evidence_candidate_count: number;
  inspected_item_count: number;
  supported_item_count: number;
  weakly_supported_item_count: number;
  unsupported_item_count: number;
  summary: string;
  planned_steps: string[];
  preview_only: boolean;
  not_answer_draft: boolean;
  not_grounded_answer: boolean;
  not_final_answer: boolean;
  no_answer_artifact_created: boolean;
  no_evidence_pack_built: boolean;
  no_persistence: boolean;
  no_llm_call: boolean;
  no_runtime_execution: boolean;
  blockers: ScholarChatDraftGroundingInspectionBlocker[];
  warnings: ScholarChatDraftGroundingInspectionWarning[];
  next_required_actions: string[];
};

type ScholarChatGroundedAnswerCandidateStatus =
  | "blocked"
  | "needs_review"
  | "candidate_ready_later";

type ScholarChatGroundedAnswerCandidateItem = {
  item_index: number;
  statement_preview: string;
  support_status: ScholarChatDraftGroundingSupportStatus;
  source_ids: string[];
  locator_previews: string[];
  matched_evidence_count: number;
};

type ScholarChatGroundedAnswerCandidatePreview = {
  status: ScholarChatGroundedAnswerCandidateStatus;
  normalized_prompt: string;
  selected_source_count: number;
  evidence_candidate_count: number;
  inspected_item_count: number;
  supported_item_count: number;
  weakly_supported_item_count: number;
  unsupported_item_count: number;
  candidate_statement_count: number;
  summary: string;
  candidate_items: ScholarChatGroundedAnswerCandidateItem[];
  preview_only: boolean;
  not_answer_draft: boolean;
  not_grounded_answer: boolean;
  not_final_answer: boolean;
  no_answer_artifact_created: boolean;
  no_evidence_pack_built: boolean;
  no_persistence: boolean;
  no_llm_call: boolean;
  no_runtime_execution: boolean;
  blockers: ScholarChatDraftGroundingInspectionBlocker[];
  warnings: ScholarChatDraftGroundingInspectionWarning[];
  next_required_actions: string[];
};

type ScholarChatGroundedAnswerWriteEligibilityStatus =
  | "blocked"
  | "needs_review"
  | "write_eligible_later";

type ScholarChatGroundedAnswerWriteEligibilityPreview = {
  status: ScholarChatGroundedAnswerWriteEligibilityStatus;
  candidate_status: ScholarChatGroundedAnswerCandidateStatus;
  normalized_prompt: string;
  selected_source_count: number;
  evidence_candidate_count: number;
  inspected_item_count: number;
  supported_item_count: number;
  weakly_supported_item_count: number;
  unsupported_item_count: number;
  candidate_statement_count: number;
  eligibility_reasons: string[];
  blockers: ScholarChatDraftGroundingInspectionBlocker[];
  warnings: ScholarChatDraftGroundingInspectionWarning[];
  next_required_actions: string[];
  summary: string;
  preview_only: boolean;
  not_answer_draft: boolean;
  not_grounded_answer: boolean;
  not_final_answer: boolean;
  no_answer_artifact_created: boolean;
  no_evidence_pack_built: boolean;
  no_persistence: boolean;
  no_llm_call: boolean;
  no_runtime_execution: boolean;
  no_registry_status_change: boolean;
  no_audit_write: boolean;
};

type ScholarChatGroundedAnswerBuildIntentStatus =
  | "blocked"
  | "needs_review"
  | "intent_ready_later";

type ScholarChatGroundedAnswerBuildIntentRequest = {
  grounding_request: ScholarChatDraftGroundingInspectionRequest;
  answer_draft_id: string | null;
  explicit_user_intent: boolean;
};

type ScholarChatGroundedAnswerBuildIntentPreview = {
  status: ScholarChatGroundedAnswerBuildIntentStatus;
  write_eligibility_status: ScholarChatGroundedAnswerWriteEligibilityStatus;
  candidate_status: ScholarChatGroundedAnswerCandidateStatus;
  normalized_prompt: string;
  selected_source_count: number;
  evidence_candidate_count: number;
  inspected_item_count: number;
  supported_item_count: number;
  weakly_supported_item_count: number;
  unsupported_item_count: number;
  candidate_statement_count: number;
  required_inputs: string[];
  missing_inputs: string[];
  intent_reasons: string[];
  blockers: ScholarChatDraftGroundingInspectionBlocker[];
  warnings: ScholarChatDraftGroundingInspectionWarning[];
  next_required_actions: string[];
  summary: string;
  preview_only: boolean;
  not_answer_draft: boolean;
  not_grounded_answer: boolean;
  not_final_answer: boolean;
  no_answer_artifact_created: boolean;
  no_evidence_pack_built: boolean;
  no_persistence: boolean;
  no_llm_call: boolean;
  no_runtime_execution: boolean;
  no_registry_status_change: boolean;
  no_audit_write: boolean;
  no_grounded_answer_service_call: boolean;
};

type ScholarChatGroundedAnswerBuildRequestStatus =
  | "blocked"
  | "needs_review"
  | "request_ready_later";

type ScholarChatGroundedAnswerBuildRequestPreviewRequest = {
  build_intent_request: ScholarChatGroundedAnswerBuildIntentRequest;
};

type ScholarChatGroundedAnswerBuildRequestPreview = {
  status: ScholarChatGroundedAnswerBuildRequestStatus;
  build_intent_status: ScholarChatGroundedAnswerBuildIntentStatus;
  write_eligibility_status: ScholarChatGroundedAnswerWriteEligibilityStatus;
  candidate_status: ScholarChatGroundedAnswerCandidateStatus;
  normalized_prompt: string;
  selected_source_count: number;
  evidence_candidate_count: number;
  inspected_item_count: number;
  supported_item_count: number;
  weakly_supported_item_count: number;
  unsupported_item_count: number;
  candidate_statement_count: number;
  answer_draft_id: string | null;
  selected_source_ids: string[];
  required_inputs: string[];
  missing_inputs: string[];
  request_reasons: string[];
  blockers: ScholarChatDraftGroundingInspectionBlocker[];
  warnings: ScholarChatDraftGroundingInspectionWarning[];
  next_required_actions: string[];
  summary: string;
  preview_only: boolean;
  not_answer_draft: boolean;
  not_grounded_answer: boolean;
  not_final_answer: boolean;
  no_answer_artifact_created: boolean;
  no_evidence_pack_built: boolean;
  no_persistence: boolean;
  no_llm_call: boolean;
  no_runtime_execution: boolean;
  no_registry_status_change: boolean;
  no_audit_write: boolean;
  no_grounded_answer_service_call: boolean;
};

type ScholarChatGroundedAnswerBuildPreflightStatus =
  | "blocked"
  | "needs_review"
  | "preflight_ready_later";

type ScholarChatGroundedAnswerBuildPreflightPreviewRequest = {
  build_request_preview_request: ScholarChatGroundedAnswerBuildRequestPreviewRequest;
};

type ScholarChatGroundedAnswerBuildPreflightPreview = {
  status: ScholarChatGroundedAnswerBuildPreflightStatus;
  build_request_status: ScholarChatGroundedAnswerBuildRequestStatus;
  build_intent_status: ScholarChatGroundedAnswerBuildIntentStatus;
  write_eligibility_status: ScholarChatGroundedAnswerWriteEligibilityStatus;
  candidate_status: ScholarChatGroundedAnswerCandidateStatus;
  normalized_prompt: string;
  selected_source_count: number;
  evidence_candidate_count: number;
  inspected_item_count: number;
  supported_item_count: number;
  weakly_supported_item_count: number;
  unsupported_item_count: number;
  candidate_statement_count: number;
  answer_draft_id: string | null;
  selected_source_ids: string[];
  answer_draft_present: boolean;
  answer_draft_readable: boolean;
  answer_draft_claim_count: number;
  required_inputs: string[];
  missing_inputs: string[];
  preflight_reasons: string[];
  blockers: ScholarChatDraftGroundingInspectionBlocker[];
  warnings: ScholarChatDraftGroundingInspectionWarning[];
  next_required_actions: string[];
  summary: string;
  preview_only: boolean;
  not_answer_draft: boolean;
  not_grounded_answer: boolean;
  not_final_answer: boolean;
  no_answer_artifact_created: boolean;
  no_evidence_pack_built: boolean;
  no_persistence: boolean;
  no_llm_call: boolean;
  no_runtime_execution: boolean;
  no_registry_status_change: boolean;
  no_audit_write: boolean;
  no_grounded_answer_service_call: boolean;
  no_grounded_answer_write: boolean;
};

type ScholarChatGroundedAnswerExecutionReadinessStatus =
  | "blocked"
  | "needs_review"
  | "execution_ready_later";

type ScholarChatGroundedAnswerExecutionReadinessPreviewRequest = {
  build_preflight_preview_request: ScholarChatGroundedAnswerBuildPreflightPreviewRequest;
  execution_consent: boolean;
};

type ScholarChatGroundedAnswerExecutionReadinessPreview = {
  status: ScholarChatGroundedAnswerExecutionReadinessStatus;
  build_preflight_status: ScholarChatGroundedAnswerBuildPreflightStatus;
  build_request_status: ScholarChatGroundedAnswerBuildRequestStatus;
  build_intent_status: ScholarChatGroundedAnswerBuildIntentStatus;
  write_eligibility_status: ScholarChatGroundedAnswerWriteEligibilityStatus;
  candidate_status: ScholarChatGroundedAnswerCandidateStatus;
  normalized_prompt: string;
  answer_draft_id: string | null;
  selected_source_ids: string[];
  selected_source_count: number;
  evidence_candidate_count: number;
  inspected_item_count: number;
  supported_item_count: number;
  weakly_supported_item_count: number;
  unsupported_item_count: number;
  candidate_statement_count: number;
  answer_draft_present: boolean;
  answer_draft_readable: boolean;
  answer_draft_claim_count: number;
  execution_consent: boolean;
  required_inputs: string[];
  missing_inputs: string[];
  readiness_reasons: string[];
  blockers: ScholarChatDraftGroundingInspectionBlocker[];
  warnings: ScholarChatDraftGroundingInspectionWarning[];
  next_required_actions: string[];
  summary: string;
  preview_only: boolean;
  not_answer_draft: boolean;
  not_grounded_answer: boolean;
  not_final_answer: boolean;
  no_answer_artifact_created: boolean;
  no_evidence_pack_built: boolean;
  no_persistence: boolean;
  no_llm_call: boolean;
  no_runtime_execution: boolean;
  no_registry_status_change: boolean;
  no_audit_write: boolean;
  no_grounded_answer_service_call: boolean;
  no_grounded_answer_write: boolean;
};

type ScholarChatGroundedAnswerExecutionPlanStatus =
  | "blocked"
  | "needs_review"
  | "plan_ready_later";

type ScholarChatGroundedAnswerExecutionPlanPreviewRequest = {
  execution_readiness_preview_request: ScholarChatGroundedAnswerExecutionReadinessPreviewRequest;
};

type ScholarChatGroundedAnswerExecutionPlanPreview = {
  status: ScholarChatGroundedAnswerExecutionPlanStatus;
  readiness_status: ScholarChatGroundedAnswerExecutionReadinessStatus;
  build_preflight_status: ScholarChatGroundedAnswerBuildPreflightStatus;
  build_request_status: ScholarChatGroundedAnswerBuildRequestStatus;
  build_intent_status: ScholarChatGroundedAnswerBuildIntentStatus;
  write_eligibility_status: ScholarChatGroundedAnswerWriteEligibilityStatus;
  candidate_status: ScholarChatGroundedAnswerCandidateStatus;
  normalized_prompt: string;
  answer_draft_id: string | null;
  selected_source_ids: string[];
  selected_source_count: number;
  evidence_candidate_count: number;
  inspected_item_count: number;
  supported_item_count: number;
  weakly_supported_item_count: number;
  unsupported_item_count: number;
  candidate_statement_count: number;
  answer_draft_present: boolean;
  answer_draft_readable: boolean;
  answer_draft_claim_count: number;
  execution_consent: boolean;
  planned_operation: string;
  planned_inputs: string[];
  planned_outputs: string[];
  planned_write_targets: string[];
  required_inputs: string[];
  missing_inputs: string[];
  plan_reasons: string[];
  blockers: ScholarChatDraftGroundingInspectionBlocker[];
  warnings: ScholarChatDraftGroundingInspectionWarning[];
  next_required_actions: string[];
  summary: string;
  preview_only: boolean;
  not_answer_draft: boolean;
  not_grounded_answer: boolean;
  not_final_answer: boolean;
  no_answer_artifact_created: boolean;
  no_evidence_pack_built: boolean;
  no_persistence: boolean;
  no_llm_call: boolean;
  no_runtime_execution: boolean;
  no_registry_status_change: boolean;
  no_audit_write: boolean;
  no_grounded_answer_service_call: boolean;
  no_grounded_answer_write: boolean;
};

type ScholarChatRuntimeDiagnosticBridgeStatus =
  | "blocked"
  | "needs_review"
  | "runtime_diagnostic_ready_later";

type ScholarChatRuntimeDiagnosticBridgePreviewRequest = {
  scholar_chat_request: ScholarChatRequest;
  smoke_execution_plan_preview_request: LocalRuntimeSmokeExecutionPlanPreviewRequest;
};

type ScholarChatRuntimeDiagnosticBridgePreview = {
  status: ScholarChatRuntimeDiagnosticBridgeStatus;
  normalized_prompt: string;
  selected_source_count: number;
  smoke_execution_plan_status: LocalRuntimeSmokeExecutionPlanStatus;
  smoke_readiness_status: LocalRuntimeSmokeReadinessStatus;
  capability_status: LocalRuntimeCapabilityStatus;
  version_probe_status: LocalRuntimeVersionProbeStatus;
  probe_readiness_status: LocalRuntimeProbeReadinessStatus;
  validation_status: LocalRuntimeValidationStatus;
  adapter_contract_status: LocalRuntimeAdapterContractStatus;
  adapter_kind: LocalRuntimeAdapterKind;
  normalized_model_family: string | null;
  normalized_model_format: string;
  safe_executable_file_name: string | null;
  safe_model_file_name: string | null;
  diagnostic_prompt_char_count: number;
  max_output_tokens: number;
  timeout_ms: number;
  runtime_diagnostic_reasons: string[];
  blockers: LocalRuntimeProbeWarning[];
  warnings: LocalRuntimeProbeWarning[];
  next_required_actions: string[];
  summary: string;
  preview_only: boolean;
  no_smoke_execution: boolean;
  no_runtime_inference: boolean;
  no_llm_call: boolean;
  no_answer_generated: boolean;
  no_answer_draft_created: boolean;
  no_grounded_answer_created: boolean;
  no_final_answer_created: boolean;
  no_grounding_applied: boolean;
  no_evidence_pack_built: boolean;
  no_persistence: boolean;
  no_artifact_write: boolean;
  no_registry_status_change: boolean;
  no_audit_write: boolean;
};

type ScholarChatRuntimeDiagnosticResultStatus =
  | "blocked"
  | "needs_review"
  | "runtime_diagnostic_failed"
  | "runtime_diagnostic_succeeded_later";

type ScholarChatRuntimeDiagnosticResultPreviewRequest = {
  bridge_preview_request: ScholarChatRuntimeDiagnosticBridgePreviewRequest;
  diagnostic_preview: LocalRuntimeSmokeDiagnosticPreview;
};

type ScholarChatRuntimeDiagnosticResultPreview = {
  status: ScholarChatRuntimeDiagnosticResultStatus;
  bridge_status: ScholarChatRuntimeDiagnosticBridgeStatus;
  smoke_diagnostic_status: LocalRuntimeSmokeDiagnosticStatus;
  smoke_execution_plan_status: LocalRuntimeSmokeExecutionPlanStatus;
  smoke_readiness_status: LocalRuntimeSmokeReadinessStatus;
  capability_status: LocalRuntimeCapabilityStatus;
  version_probe_status: LocalRuntimeVersionProbeStatus;
  probe_readiness_status: LocalRuntimeProbeReadinessStatus;
  validation_status: LocalRuntimeValidationStatus;
  adapter_contract_status: LocalRuntimeAdapterContractStatus;
  adapter_kind: LocalRuntimeAdapterKind;
  normalized_model_family: string | null;
  normalized_model_format: string;
  safe_executable_file_name: string | null;
  safe_model_file_name: string | null;
  diagnostic_prompt_char_count: number;
  max_output_tokens: number;
  timeout_ms: number;
  exit_code: number | null;
  stdout_preview: string;
  stderr_preview: string;
  stdout_truncated: boolean;
  stderr_truncated: boolean;
  runtime_result_reasons: string[];
  blockers: LocalRuntimeSmokeInferenceBlocker[];
  warnings: LocalRuntimeSmokeInferenceWarning[];
  next_required_actions: string[];
  summary: string;
  preview_only: boolean;
  diagnostic_result_only: boolean;
  no_smoke_execution: boolean;
  no_runtime_inference: boolean;
  no_new_process_spawn: boolean;
  no_llm_call: boolean;
  no_answer_generated: boolean;
  no_answer_draft_created: boolean;
  no_grounded_answer_created: boolean;
  no_final_answer_created: boolean;
  no_grounding_applied: boolean;
  no_evidence_pack_built: boolean;
  no_persistence: boolean;
  no_artifact_write: boolean;
  no_registry_status_change: boolean;
  no_audit_write: boolean;
};

type LocalModelRuntimeKind = "llama_cpp" | "none";

type LocalModelRuntimeHealthStatus =
  | "not_configured"
  | "config_present"
  | "model_missing"
  | "executable_missing"
  | "ready_to_test_later";

type LocalModelRuntimePathState = "not_configured" | "missing" | "exists";

type LocalModelRuntimeHealthWarning = {
  kind: string;
  message: string;
};

type LocalModelRuntimeConfig = {
  runtime_kind: LocalModelRuntimeKind;
  model_path: string | null;
  executable_path: string | null;
  context_window: number | null;
  gpu_layers: number | null;
  temperature: number | null;
};

type LocalRuntimeAdapterKind = "llama_cpp";

type LocalRuntimeAdapterContractStatus =
  | "blocked"
  | "needs_review"
  | "contract_ready_later";

type LocalRuntimeAdapterContractBlocker = {
  kind: string;
  message: string;
};

type LocalRuntimeAdapterContractWarning = {
  kind: string;
  message: string;
};

type LocalRuntimeAdapterContractPreviewRequest = {
  adapter_kind: LocalRuntimeAdapterKind;
  executable_path: string | null;
  model_path: string | null;
  model_family: string | null;
  model_format: string | null;
  context_window_tokens: number | null;
  gpu_layers: number | null;
  threads: number | null;
  batch_size: number | null;
  chat_template: string | null;
};

type LocalRuntimeAdapterContractPreview = {
  status: LocalRuntimeAdapterContractStatus;
  adapter_kind: LocalRuntimeAdapterKind;
  normalized_model_family: string | null;
  normalized_model_format: string;
  executable_path_present: boolean;
  model_path_present: boolean;
  context_window_tokens: number | null;
  gpu_layers: number | null;
  threads: number | null;
  batch_size: number | null;
  chat_template_present: boolean;
  required_inputs: string[];
  missing_inputs: string[];
  contract_reasons: string[];
  blockers: LocalRuntimeAdapterContractBlocker[];
  warnings: LocalRuntimeAdapterContractWarning[];
  next_required_actions: string[];
  summary: string;
  preview_only: boolean;
  no_process_spawn: boolean;
  no_model_load: boolean;
  no_llm_call: boolean;
  no_runtime_execution: boolean;
  no_persistence: boolean;
  no_artifact_write: boolean;
  no_registry_status_change: boolean;
  no_audit_write: boolean;
};

type LocalRuntimeValidationStatus =
  | "blocked"
  | "needs_review"
  | "validation_ready_later";

type LocalRuntimeValidationPreviewRequest = {
  adapter_contract_request: LocalRuntimeAdapterContractPreviewRequest;
};

type LocalRuntimeValidationPreview = {
  status: LocalRuntimeValidationStatus;
  adapter_contract_status: LocalRuntimeAdapterContractStatus;
  adapter_kind: LocalRuntimeAdapterKind;
  normalized_model_family: string | null;
  normalized_model_format: string;
  executable_path_present: boolean;
  model_path_present: boolean;
  executable_exists: boolean;
  model_exists: boolean;
  executable_is_file: boolean;
  model_is_file: boolean;
  model_extension_valid: boolean;
  safe_executable_file_name?: string | null;
  safe_model_file_name?: string | null;
  context_window_tokens: number | null;
  gpu_layers: number | null;
  threads: number | null;
  batch_size: number | null;
  chat_template_present: boolean;
  missing_inputs: string[];
  validation_reasons: string[];
  blockers: LocalRuntimeAdapterContractBlocker[];
  warnings: LocalRuntimeAdapterContractWarning[];
  next_required_actions: string[];
  summary: string;
  preview_only: boolean;
  no_process_spawn: boolean;
  no_binary_probe: boolean;
  no_model_load: boolean;
  no_llm_call: boolean;
  no_runtime_execution: boolean;
  no_persistence: boolean;
  no_artifact_write: boolean;
  no_registry_status_change: boolean;
  no_audit_write: boolean;
};

type LocalRuntimeProbeReadinessStatus =
  | "blocked"
  | "needs_review"
  | "probe_ready_later";

type LocalRuntimeProbeReadinessPreviewRequest = {
  validation_preview_request: LocalRuntimeValidationPreviewRequest;
  probe_consent: boolean;
};

type LocalRuntimeProbeReadinessPreview = {
  status: LocalRuntimeProbeReadinessStatus;
  validation_status: LocalRuntimeValidationStatus;
  adapter_contract_status: LocalRuntimeAdapterContractStatus;
  adapter_kind: LocalRuntimeAdapterKind;
  normalized_model_family: string | null;
  normalized_model_format: string;
  executable_path_present: boolean;
  model_path_present: boolean;
  executable_exists: boolean;
  model_exists: boolean;
  executable_is_file: boolean;
  model_is_file: boolean;
  model_extension_valid: boolean;
  safe_executable_file_name?: string | null;
  safe_model_file_name?: string | null;
  probe_consent: boolean;
  required_inputs: string[];
  missing_inputs: string[];
  readiness_reasons: string[];
  blockers: LocalRuntimeAdapterContractBlocker[];
  warnings: LocalRuntimeAdapterContractWarning[];
  next_required_actions: string[];
  summary: string;
  preview_only: boolean;
  no_process_spawn: boolean;
  no_binary_probe: boolean;
  no_model_load: boolean;
  no_llm_call: boolean;
  no_runtime_execution: boolean;
  no_persistence: boolean;
  no_artifact_write: boolean;
  no_registry_status_change: boolean;
  no_audit_write: boolean;
};

type LocalModelRuntimeHealthPreview = {
  status: LocalModelRuntimeHealthStatus;
  runtime_kind: LocalModelRuntimeKind;
  model_state: LocalModelRuntimePathState;
  executable_state: LocalModelRuntimePathState;
  model_extension_valid: boolean;
  model_file_name?: string | null;
  context_window?: number | null;
  gpu_layers?: number | null;
  temperature?: number | null;
  warnings: LocalModelRuntimeHealthWarning[];
};

type ManagedLlamaServerLaunchPlanStatus = "blocked" | "launch_ready_later";
type ManagedLlamaServerLifecycleStatus =
  | "not_started"
  | "starting"
  | "running"
  | "stopped"
  | "failed"
  | "blocked"
  | "already_running"
  | "port_occupied"
  | "external_server_detected";
type ManagedLlamaServerHealthStatus = "not_started" | "loading" | "ready" | "unreachable" | "failed";
type ManagedLlamaServerPortOccupancyStatus = "free" | "managed_owned" | "external_server_detected" | "port_occupied" | "unknown_owner";

type ManagedLlamaServerNotice = {
  kind: string;
  message: string;
};

type ManagedLlamaServerLaunchPlanRequest = {
  executable_path: string | null;
  model_path: string | null;
  host: string | null;
  port: number | null;
  alias: string | null;
  context_window: number | null;
  gpu_layers: number | null;
};

type ManagedLlamaServerStartRequest = {
  allow_server_start: boolean;
  launch_plan_request: ManagedLlamaServerLaunchPlanRequest;
};

type ManagedLlamaServerLaunchPlanPreview = {
  status: ManagedLlamaServerLaunchPlanStatus;
  executable_path_present: boolean;
  model_path_present: boolean;
  executable_is_file: boolean;
  model_is_file: boolean;
  model_extension_valid: boolean;
  safe_executable_file_name?: string | null;
  safe_model_file_name?: string | null;
  host: string;
  port: number;
  alias: string;
  context_window: number;
  gpu_layers: number;
  blockers: ManagedLlamaServerNotice[];
  warnings: ManagedLlamaServerNotice[];
  next_required_actions: string[];
  summary: string;
  preview_only: boolean;
  no_process_spawn: boolean;
  no_model_output_used: boolean;
  no_answer_generation: boolean;
  no_persistence: boolean;
  no_artifact_write: boolean;
  no_lan_binding_by_default: boolean;
  no_auto_start_on_launch: boolean;
};

type ManagedLlamaServerStatusPreview = {
  lifecycle_status: ManagedLlamaServerLifecycleStatus;
  health_status: ManagedLlamaServerHealthStatus;
  owns_active_server: boolean;
  port_occupied: boolean;
  port_occupied_by_unmanaged_process: boolean;
  port_occupancy_status: ManagedLlamaServerPortOccupancyStatus;
  host?: string | null;
  port?: number | null;
  alias?: string | null;
  process_id?: number | null;
  exit_code?: number | null;
  safe_executable_file_name?: string | null;
  safe_model_file_name?: string | null;
  health_url?: string | null;
  response_body_preview: string;
  response_body_truncated: boolean;
  blockers: ManagedLlamaServerNotice[];
  warnings: ManagedLlamaServerNotice[];
  next_required_actions: string[];
  summary: string;
  preview_only: boolean;
  no_process_spawn: boolean;
  no_model_output_used: boolean;
  no_answer_generation: boolean;
  no_persistence: boolean;
  no_artifact_write: boolean;
  no_lan_binding_by_default: boolean;
};

type ManagedLlamaServerChatDiagnosticStatus =
  | "blocked"
  | "server_not_ready"
  | "diagnostic_succeeded"
  | "diagnostic_failed"
  | "timed_out";

type ManagedLlamaServerChatDiagnosticRequest = {
  allow_chat_diagnostic: boolean;
  prompt: string | null;
  max_tokens: number | null;
  temperature: number | null;
  timeout_ms: number | null;
};

type ManagedLlamaServerChatDiagnosticPreview = {
  status: ManagedLlamaServerChatDiagnosticStatus;
  request_attempted: boolean;
  lifecycle_status: ManagedLlamaServerLifecycleStatus;
  health_status: ManagedLlamaServerHealthStatus;
  host?: string | null;
  port?: number | null;
  alias?: string | null;
  safe_model_file_name?: string | null;
  prompt_char_count: number;
  max_tokens: number;
  temperature: number;
  timeout_ms: number;
  http_status?: number | null;
  response_preview: string;
  response_preview_truncated: boolean;
  extracted_message_preview?: string | null;
  duration_ms: number;
  blockers: ManagedLlamaServerNotice[];
  warnings: ManagedLlamaServerNotice[];
  next_required_actions: string[];
  summary: string;
  diagnostic_only: boolean;
  not_scholar_chat_answer: boolean;
  no_final_answer_created: boolean;
  no_grounding_applied: boolean;
  no_artifact_write: boolean;
  no_persistence: boolean;
};

type ManagedLlamaServerSmokeDiagnosticStatus = "blocked" | "server_not_running" | "smoke_succeeded" | "smoke_failed" | "timed_out";

type ManagedLlamaServerSmokeDiagnosticRequest = {
  allow_smoke_execution: boolean;
  prompt: string | null;
  max_output_tokens: number | null;
  timeout_ms: number | null;
};

type ManagedLlamaServerSmokeDiagnosticPreview = {
  status: ManagedLlamaServerSmokeDiagnosticStatus;
  execution_attempted: boolean;
  lifecycle_status: ManagedLlamaServerLifecycleStatus;
  health_status: ManagedLlamaServerHealthStatus;
  owns_active_server: boolean;
  port_occupied: boolean;
  port_occupied_by_unmanaged_process: boolean;
  port_occupancy_status: ManagedLlamaServerPortOccupancyStatus;
  host?: string | null;
  port?: number | null;
  alias?: string | null;
  safe_model_file_name?: string | null;
  prompt_char_count: number;
  max_output_tokens: number;
  timeout_ms: number;
  http_status?: number | null;
  response_preview: string;
  response_preview_truncated: boolean;
  extracted_output_preview?: string | null;
  error_preview: string;
  error_preview_truncated: boolean;
  duration_ms: number;
  blockers: ManagedLlamaServerNotice[];
  warnings: ManagedLlamaServerNotice[];
  next_required_actions: string[];
  summary: string;
  diagnostic_only: boolean;
  not_scholar_chat_answer: boolean;
  no_grounding_applied: boolean;
  no_evidence_pack_used: boolean;
  no_artifact_write: boolean;
  no_audit_write: boolean;
  no_persistence: boolean;
  no_final_answer_created: boolean;
};

type LocalRuntimeInvocationPlanStatus =
  | "not_configured"
  | "blocked"
  | "ready_to_invoke_later"
  | "preview_only";

type LocalRuntimeInvocationBlocker = {
  kind: string;
  message: string;
};

type LocalRuntimeInvocationPlan = {
  runtime_health_status: LocalModelRuntimeHealthStatus;
  prompt_char_count: number;
  estimated_context_char_count: number;
  max_output_tokens?: number | null;
  safe_model_file_name?: string | null;
  safe_executable_file_name?: string | null;
  invocation_steps: string[];
  safe_argument_preview: string[];
  blockers: LocalRuntimeInvocationBlocker[];
  warnings: LocalModelRuntimeHealthWarning[];
};

type LocalRuntimeInvocationPlanPreview = {
  status: LocalRuntimeInvocationPlanStatus;
  runtime_kind: LocalModelRuntimeKind;
  plan: LocalRuntimeInvocationPlan;
};

type LocalRuntimeProbeStatus = "blocked" | "completed" | "timed_out";

type LocalRuntimeProbeWarning = {
  kind: string;
  message: string;
};

type LocalRuntimeProbeRequest = {
  executable_path: string | null;
  allow_execution: boolean;
  timeout_ms: number | null;
};

type LocalRuntimeProbeResult = {
  status: LocalRuntimeProbeStatus;
  allow_execution: boolean;
  execution_attempted: boolean;
  probe_argument: string;
  timeout_ms: number;
  duration_ms: number;
  safe_executable_file_name?: string | null;
  exit_code?: number | null;
  stdout_preview: string;
  stderr_preview: string;
  blockers: LocalRuntimeProbeWarning[];
  warnings: LocalRuntimeProbeWarning[];
};

type LocalRuntimeSmokeInferenceStatus =
  | "blocked"
  | "not_configured"
  | "model_missing"
  | "executable_missing"
  | "inference_succeeded"
  | "inference_failed"
  | "timed_out";

type LocalRuntimeSmokeInferenceWarning = {
  kind: string;
  message: string;
};

type LocalRuntimeSmokeInferenceOutputClassification = "runtime_diagnostic";

type LocalRuntimeSmokeInferenceBlocker = {
  kind: string;
  message: string;
};

type LocalRuntimeSmokeInferenceRequest = {
  runtime_config: LocalModelRuntimeConfig;
  allow_execution: boolean;
  prompt: string | null;
  timeout_ms: number | null;
  max_output_tokens: number | null;
};

type LocalRuntimeSmokeInferenceResult = {
  status: LocalRuntimeSmokeInferenceStatus;
  allow_execution: boolean;
  execution_attempted: boolean;
  runtime_kind: LocalModelRuntimeKind;
  safe_model_file_name?: string | null;
  safe_executable_file_name?: string | null;
  normalized_prompt: string;
  prompt_char_count: number;
  max_output_tokens: number;
  timeout_ms: number;
  exit_code?: number | null;
  stdout_preview: string;
  stderr_preview: string;
  duration_ms: number;
  warnings: LocalRuntimeSmokeInferenceWarning[];
  blockers: LocalRuntimeSmokeInferenceBlocker[];
  diagnostic_only: boolean;
  no_answer_generated: boolean;
  no_grounding_applied: boolean;
  no_evidence_pack_used: boolean;
  not_scholar_chat_answer: boolean;
  output_classification: LocalRuntimeSmokeInferenceOutputClassification;
};

type LocalRuntimeSmokeDiagnosticStatus = "blocked" | "smoke_succeeded" | "smoke_failed" | "timed_out";

type LocalRuntimeSmokeDiagnosticRequest = {
  smoke_execution_plan_preview_request: LocalRuntimeSmokeExecutionPlanPreviewRequest;
  allow_smoke_execution: boolean;
};

type LocalRuntimeSmokeDiagnosticPreview = {
  status: LocalRuntimeSmokeDiagnosticStatus;
  smoke_execution_plan_status: LocalRuntimeSmokeExecutionPlanStatus;
  smoke_readiness_status: LocalRuntimeSmokeReadinessStatus;
  capability_status: LocalRuntimeCapabilityStatus;
  version_probe_status: LocalRuntimeVersionProbeStatus;
  probe_readiness_status: LocalRuntimeProbeReadinessStatus;
  validation_status: LocalRuntimeValidationStatus;
  adapter_contract_status: LocalRuntimeAdapterContractStatus;
  adapter_kind: LocalRuntimeAdapterKind;
  normalized_model_family?: string | null;
  normalized_model_format: string;
  safe_executable_file_name?: string | null;
  safe_model_file_name?: string | null;
  probe_consent: boolean;
  allow_probe_execution: boolean;
  smoke_consent: boolean;
  allow_smoke_execution: boolean;
  execution_attempted: boolean;
  normalized_diagnostic_prompt: string;
  diagnostic_prompt_char_count: number;
  max_output_tokens: number;
  timeout_ms: number;
  duration_ms: number;
  exit_code?: number | null;
  stdout_preview: string;
  stderr_preview: string;
  stdout_truncated: boolean;
  stderr_truncated: boolean;
  blockers: LocalRuntimeSmokeInferenceBlocker[];
  warnings: LocalRuntimeSmokeInferenceWarning[];
  next_required_actions: string[];
  summary: string;
  diagnostic_only: boolean;
  not_scholar_chat_answer: boolean;
  no_answer_generated: boolean;
  no_grounding_applied: boolean;
  no_evidence_pack_used: boolean;
  no_persistence: boolean;
  no_artifact_write: boolean;
  no_registry_status_change: boolean;
  no_audit_write: boolean;
};

type LocalRuntimeInvocationPlanRequest = {
  runtime_config: LocalModelRuntimeConfig;
  prompt_text: string | null;
  estimated_input_char_count: number | null;
  max_output_tokens: number | null;
  stop_sequences: string[] | null;
};

type RetrievalIndexEntry = {
  chunk_id: string;
  source_id: string;
  version_id: string;
  locator: CitationLocator;
  text_hash: string;
  normalized_terms: string[];
};

type RetrievalIndex = {
  source_id: string;
  version_id: string;
  indexed_at: string;
  chunk_count: number;
  index_version: string;
  chunk_report_hash: string;
  entries: RetrievalIndexEntry[];
  warnings: string[];
};

type RetrievalSearchResult = {
  chunk_id: string;
  source_id: string;
  version_id: string;
  locator: CitationLocator;
  score: number;
  matched_terms: string[];
  text_hash: string;
  preview: string;
};

type RetrievalSearchResponse = {
  query: string;
  normalized_query_terms: string[];
  result_count: number;
  results: RetrievalSearchResult[];
};

type CitationLocator = {
  label: string;
  section?: string | null;
  paragraph_index?: number | null;
  start_char: number;
  end_char: number;
  [key: string]: unknown;
};

type FinalAnswerStatement = {
  statement_id: string;
  grounded_statement_id: string;
  status: "supported" | "needs_evidence" | "unsupported";
  text: string;
  claim_ids: string[];
  evidence_ids: string[];
  chunk_ids: string[];
  locators: CitationLocator[];
  support_level: "direct_grounded_statement" | "missing_evidence";
};

type FinalAnswer = {
  final_answer_id: string;
  grounded_answer_id: string;
  source_id: string;
  version_id: string;
  query: string;
  created_at: string;
  answer_mode: "contract_only";
  statement_count: number;
  unsupported_count: number;
  statements: FinalAnswerStatement[];
  warnings: string[];
};

type FinalAnswerMetadata = {
  final_answer_id: string;
  grounded_answer_id: string;
  statement_count: number;
  unsupported_count: number;
  needs_evidence_count: number;
};

type AnswerArtifactOverview = {
  source_id: string;
  draft_count: number;
  grounded_answer_count: number;
  final_answer_count: number;
  final_answers: FinalAnswerMetadata[];
};

type AnswerArtifactSourceMetadata = {
  source_id: string;
  draft_count: number;
  grounded_answer_count: number;
  final_answer_count: number;
};

type AnswerArtifactSourceHealth = {
  source_id: string;
  draft_count: number;
  grounded_answer_count: number;
  final_answer_count: number;
  malformed_final_answer_count: number;
  unsupported_statement_count: number;
  needs_evidence_statement_count: number;
};

type AnswerArtifactHealth = {
  source_count: number;
  draft_count: number;
  grounded_answer_count: number;
  final_answer_count: number;
  malformed_final_answer_count: number;
  unsupported_statement_count: number;
  needs_evidence_statement_count: number;
  sources: AnswerArtifactSourceHealth[];
};

type AnswerArtifactIssue = {
  source_id: string;
  issue_kind: "malformed_final_answer" | "unsupported_statement" | "needs_evidence_statement";
  final_answer_id?: string | null;
  grounded_answer_id?: string | null;
  statement_index?: number | null;
  statement_status?: string | null;
  message: string;
};

type EvidencePackMetadata = {
  source_id: string;
  version_id: string;
  evidence_pack_id: string;
  query: string;
  created_at: string;
  retrieval_index_version: string;
  result_count: number;
  item_count: number;
  warning_count: number;
  evidence_pack_version: string;
};

type AnswerArtifactExportSource = {
  source_id: string;
  draft_count: number;
  grounded_answer_count: number;
  final_answer_count: number;
  final_answers: FinalAnswerMetadata[];
  issue_count: number;
};

type AnswerArtifactExportManifest = {
  schema_version: string;
  source_count: number;
  draft_count: number;
  grounded_answer_count: number;
  final_answer_count: number;
  issue_count: number;
  sources: AnswerArtifactExportSource[];
};

type ExportedArtifactFile = {
  relative_path: string;
  artifact_kind: "manifest" | "issues" | "summary" | "integrity" | "answer_draft" | "grounded_answer" | "final_answer";
  source_id?: string | null;
  artifact_id?: string | null;
};

type AnswerArtifactExportIntegrityFile = {
  relative_path: string;
  byte_count: number;
  sha256: string;
};

type AnswerArtifactExportIntegrity = {
  schema_version: string;
  algorithm: string;
  files: AnswerArtifactExportIntegrityFile[];
};

type AnswerArtifactExportResult = {
  schema_version: string;
  manifest: AnswerArtifactExportManifest;
  integrity: AnswerArtifactExportIntegrity;
  exported_source_count: number;
  exported_draft_count: number;
  exported_grounded_answer_count: number;
  exported_final_answer_count: number;
  exported_issue_count: number;
  export_id: string;
  written_files: ExportedArtifactFile[];
};

type AnswerArtifactExportIssueKindCount = {
  issue_kind: "malformed_final_answer" | "needs_evidence_statement" | "unsupported_statement";
  count: number;
};

type AnswerArtifactExportBundleInspectionIssueKindCount = {
  kind:
    | "missing_manifest"
    | "manifest_read_failed"
    | "missing_issues"
    | "issues_read_failed"
    | "missing_summary"
    | "summary_read_failed"
    | "missing_integrity"
    | "integrity_read_failed"
    | "integrity_schema_version_missing"
    | "integrity_schema_version_unsupported"
    | "integrity_algorithm_missing"
    | "integrity_algorithm_unsupported"
    | "integrity_duplicate_path"
    | "integrity_path_invalid"
    | "integrity_missing_file"
    | "integrity_byte_count_mismatch"
    | "integrity_digest_mismatch"
    | "schema_version_missing"
    | "schema_version_unsupported"
    | "schema_version_mismatch"
    | "summary_counts_mismatch"
    | "summary_issue_count_mismatch"
    | "summary_issue_kind_counts_mismatch"
    | "summary_export_id_mismatch"
    | "summary_metadata_mismatch";
  count: number;
};

type AnswerArtifactExportBundleInspectionSummary = {
  is_consistent: boolean;
  schema_supported: boolean;
  integrity_verified: boolean;
  issue_count: number;
  warning_count: number;
  issue_counts_by_kind: AnswerArtifactExportBundleInspectionIssueKindCount[];
  checked_file_count: number;
  integrity_file_count: number;
};

type AnswerArtifactExportBundleInspectionReportSection = {
  heading: string;
  lines: string[];
};

type AnswerArtifactExportBundleInspectionReportPreview = {
  title: string;
  schema_version: string;
  is_consistent: boolean;
  integrity_verified: boolean;
  issue_count: number;
  warning_count: number;
  issue_counts_by_kind: AnswerArtifactExportBundleInspectionIssueKindCount[];
  sections: AnswerArtifactExportBundleInspectionReportSection[];
};

type AnswerArtifactExportBundleInspectionIssueGroup = {
  kind: AnswerArtifactExportBundleInspectionIssueKind;
  count: number;
  lines: string[];
};

type AnswerArtifactExportBundleInspectionStatus = {
  code: string;
  label: string;
  severity: string;
  reason: string;
};

type AnswerArtifactExportBundleFileStatus = {
  file_label: string;
  present: boolean;
  parsed: boolean;
  malformed: boolean;
  schema_version?: string | null;
  schema_status: string;
  integrity_status: string;
  issue_count: number;
  status: string;
};

type AnswerArtifactExportSummarySource = {
  source_id: string;
  draft_count: number;
  grounded_answer_count: number;
  final_answer_count: number;
  issue_count: number;
};

type AnswerArtifactExportSummary = {
  schema_version: string;
  export_id: string;
  generated_from: string;
  export_scope: string;
  non_goals: string[];
  source_count: number;
  draft_count: number;
  grounded_answer_count: number;
  final_answer_count: number;
  issue_count: number;
  issue_kinds: AnswerArtifactExportIssueKindCount[];
  sources: AnswerArtifactExportSummarySource[];
};

type AnswerArtifactExportBundleInspectionIssueKind =
  | "missing_manifest"
  | "manifest_read_failed"
  | "missing_issues"
  | "issues_read_failed"
  | "missing_summary"
  | "summary_read_failed"
  | "missing_integrity"
  | "integrity_read_failed"
  | "integrity_schema_version_missing"
  | "integrity_schema_version_unsupported"
  | "integrity_algorithm_missing"
  | "integrity_algorithm_unsupported"
  | "integrity_duplicate_path"
  | "integrity_path_invalid"
  | "integrity_missing_file"
  | "integrity_byte_count_mismatch"
  | "integrity_digest_mismatch"
  | "schema_version_missing"
  | "schema_version_unsupported"
  | "schema_version_mismatch"
  | "summary_counts_mismatch"
  | "summary_issue_count_mismatch"
  | "summary_issue_kind_counts_mismatch"
  | "summary_export_id_mismatch"
  | "summary_metadata_mismatch";

type AnswerArtifactExportBundleInspectionIssue = {
  kind: AnswerArtifactExportBundleInspectionIssueKind;
  message: string;
  relative_path?: string | null;
};

type AnswerArtifactExportBundleInspection = {
  schema_version?: string | null;
  manifest_schema_version?: string | null;
  issues_schema_version?: string | null;
  summary_schema_version?: string | null;
  integrity_schema_version?: string | null;
  integrity_algorithm?: string | null;
  inspection_status: AnswerArtifactExportBundleInspectionStatus;
  inspection_summary: AnswerArtifactExportBundleInspectionSummary;
  report_preview: AnswerArtifactExportBundleInspectionReportPreview;
  issue_groups: AnswerArtifactExportBundleInspectionIssueGroup[];
  file_statuses: AnswerArtifactExportBundleFileStatus[];
  has_manifest: boolean;
  has_issues: boolean;
  has_summary: boolean;
  has_integrity: boolean;
  is_consistent: boolean;
  issue_count: number;
  warning_count: number;
  errors: AnswerArtifactExportBundleInspectionIssue[];
  warnings: AnswerArtifactExportBundleInspectionIssue[];
  manifest_counts?: AnswerArtifactExportManifest | null;
  summary_counts?: AnswerArtifactExportSummary | null;
  integrity_counts?: AnswerArtifactExportIntegrity | null;
  issue_kind_counts?: AnswerArtifactExportIssueKindCount[] | null;
};

function sanitizeBackendError(error: unknown) {
  const message = String(error);
  return message.replace(/[A-Za-z]:\\[^"]+/g, "[path hidden]").replace(/E:\\[^"]+/g, "[path hidden]");
}

function locatorSummary(locator: CitationLocator) {
  const section = locator.section ? `section=${locator.section}` : null;
  const paragraph = locator.paragraph_index !== null && locator.paragraph_index !== undefined ? `paragraph=${locator.paragraph_index}` : null;
  const range = `chars=${locator.start_char}-${locator.end_char}`;
  return [locator.label, section, paragraph, range].filter(Boolean).join(" | ");
}

function compactTextPreview(text: string, maxChars = 240) {
  const compacted = text.split(/\s+/).filter(Boolean).join(" ").trim();
  if (compacted.length <= maxChars) {
    return compacted;
  }
  return `${compacted.slice(0, Math.max(0, maxChars - 1)).trimEnd()}...`;
}

function renderMetricGrid(entries: { label: string; value: string | number }[]) {
  return (
    <div class="contract-meta">
      {entries.map((entry) => (
        <div>
          <span>{entry.label}</span>
          <strong>{entry.value}</strong>
        </div>
      ))}
    </div>
  );
}

export default function App() {
  const [activeWorkspace, setActiveWorkspace] = createSignal<WorkspaceSection>("scholar_chat");
  const [status, setStatus] = createSignal<CorpusStatus | null>(null);
  const [statusError, setStatusError] = createSignal<string | null>(null);
  const [scholarChatPrompt, setScholarChatPrompt] = createSignal("");
  const [scholarChatMode, setScholarChatMode] = createSignal<ScholarChatMode>("lecture_learning");
  const [scholarChatGroundingPolicy, setScholarChatGroundingPolicy] = createSignal<GroundingPolicy>("local_first");
  const [scholarChatPreview, setScholarChatPreview] = createSignal<ScholarChatAgenticWorkflowPlanPreview | null>(null);
  const [scholarChatError, setScholarChatError] = createSignal<string | null>(null);
  const [scholarChatValidationError, setScholarChatValidationError] = createSignal<string | null>(null);
  const [scholarChatLoading, setScholarChatLoading] = createSignal(false);
  const [scholarChatExecutionGatePreview, setScholarChatExecutionGatePreview] = createSignal<ScholarChatAgenticWorkflowExecutionGatePreview | null>(null);
  const [scholarChatExecutionGateError, setScholarChatExecutionGateError] = createSignal<string | null>(null);
  const [scholarChatExecutionGateValidationError, setScholarChatExecutionGateValidationError] = createSignal<string | null>(null);
  const [scholarChatExecutionGateLoading, setScholarChatExecutionGateLoading] = createSignal(false);
  const [scholarChatSourceContext, setScholarChatSourceContext] = createSignal<RegisteredSource[]>([]);
  const [scholarChatSourceContextLoading, setScholarChatSourceContextLoading] = createSignal(false);
  const [scholarChatSourceContextError, setScholarChatSourceContextError] = createSignal<string | null>(null);
  const [scholarChatSourceContextSelectedIds, setScholarChatSourceContextSelectedIds] = createSignal<string[]>([]);
  const [scholarChatSourceContextTouched, setScholarChatSourceContextTouched] = createSignal(false);
  const [scholarChatRetrievalPreview, setScholarChatRetrievalPreview] = createSignal<ScholarChatRetrievalPreviewResponse | null>(null);
  const [scholarChatRetrievalError, setScholarChatRetrievalError] = createSignal<string | null>(null);
  const [scholarChatRetrievalLoading, setScholarChatRetrievalLoading] = createSignal(false);
  const [scholarChatRetrievalHasRun, setScholarChatRetrievalHasRun] = createSignal(false);
  const [scholarChatEvidencePlanPreview, setScholarChatEvidencePlanPreview] = createSignal<ScholarChatEvidencePlanResponse | null>(null);
  const [scholarChatEvidencePlanError, setScholarChatEvidencePlanError] = createSignal<string | null>(null);
  const [scholarChatEvidencePlanLoading, setScholarChatEvidencePlanLoading] = createSignal(false);
  const [scholarChatEvidencePlanHasRun, setScholarChatEvidencePlanHasRun] = createSignal(false);
  const [scholarChatPromptPackPreview, setScholarChatPromptPackPreview] = createSignal<ScholarChatPromptPackPreviewResponse | null>(null);
  const [scholarChatPromptPackError, setScholarChatPromptPackError] = createSignal<string | null>(null);
  const [scholarChatPromptPackLoading, setScholarChatPromptPackLoading] = createSignal(false);
  const [scholarChatPromptPackHasRun, setScholarChatPromptPackHasRun] = createSignal(false);
  const [scholarChatTranscript, setScholarChatTranscript] = createSignal<ScholarChatTranscriptMessage[]>([]);
  const [scholarChatScientificMetadataProviderRequestPreview, setScholarChatScientificMetadataProviderRequestPreview] = createSignal<ScholarChatScientificMetadataProviderRequestPreview | null>(null);
  const [scholarChatScientificMetadataProviderRequestError, setScholarChatScientificMetadataProviderRequestError] = createSignal<string | null>(null);
  const [scholarChatScientificMetadataProviderRequestValidationError, setScholarChatScientificMetadataProviderRequestValidationError] = createSignal<string | null>(null);
  const [scholarChatScientificMetadataProviderRequestLoading, setScholarChatScientificMetadataProviderRequestLoading] = createSignal(false);
  const [scholarChatScientificMetadataProviderRequestHasRun, setScholarChatScientificMetadataProviderRequestHasRun] = createSignal(false);
  const scientificMetadataProviderRequestPreview = scholarChatScientificMetadataProviderRequestPreview();
  const [scholarChatAnswerReadinessAllowModelExecution, setScholarChatAnswerReadinessAllowModelExecution] = createSignal(false);
  const [scholarChatAnswerReadinessPreview, setScholarChatAnswerReadinessPreview] = createSignal<ScholarChatAnswerReadinessPreview | null>(null);
  const [scholarChatAnswerReadinessError, setScholarChatAnswerReadinessError] = createSignal<string | null>(null);
  const [scholarChatAnswerReadinessValidationError, setScholarChatAnswerReadinessValidationError] = createSignal<string | null>(null);
  const [scholarChatAnswerReadinessLoading, setScholarChatAnswerReadinessLoading] = createSignal(false);
  const [scholarChatAnswerReadinessHasRun, setScholarChatAnswerReadinessHasRun] = createSignal(false);
  const [scholarChatDraftInferencePreview, setScholarChatDraftInferencePreview] = createSignal<ScholarChatDraftInferencePreview | null>(null);
  const [scholarChatDraftInferenceError, setScholarChatDraftInferenceError] = createSignal<string | null>(null);
  const [scholarChatDraftInferenceValidationError, setScholarChatDraftInferenceValidationError] = createSignal<string | null>(null);
  const [scholarChatDraftInferenceLoading, setScholarChatDraftInferenceLoading] = createSignal(false);
  const [scholarChatDraftInferenceHasRun, setScholarChatDraftInferenceHasRun] = createSignal(false);
  const [scholarChatDraftGroundingInspectionDraftText, setScholarChatDraftGroundingInspectionDraftText] = createSignal("");
  const [scholarChatDraftGroundingInspectionPreview, setScholarChatDraftGroundingInspectionPreview] = createSignal<ScholarChatDraftGroundingInspectionPreview | null>(null);
  const [scholarChatDraftGroundingInspectionError, setScholarChatDraftGroundingInspectionError] = createSignal<string | null>(null);
  const [scholarChatDraftGroundingInspectionValidationError, setScholarChatDraftGroundingInspectionValidationError] = createSignal<string | null>(null);
  const [scholarChatDraftGroundingInspectionLoading, setScholarChatDraftGroundingInspectionLoading] = createSignal(false);
  const [scholarChatDraftGroundingInspectionHasRun, setScholarChatDraftGroundingInspectionHasRun] = createSignal(false);
  const [scholarChatGroundedDraftReadinessPreview, setScholarChatGroundedDraftReadinessPreview] = createSignal<ScholarChatGroundedDraftReadinessPreview | null>(null);
  const [scholarChatGroundedDraftReadinessError, setScholarChatGroundedDraftReadinessError] = createSignal<string | null>(null);
  const [scholarChatGroundedDraftReadinessValidationError, setScholarChatGroundedDraftReadinessValidationError] = createSignal<string | null>(null);
  const [scholarChatGroundedDraftReadinessLoading, setScholarChatGroundedDraftReadinessLoading] = createSignal(false);
  const [scholarChatGroundedDraftReadinessHasRun, setScholarChatGroundedDraftReadinessHasRun] = createSignal(false);
  const [scholarChatGroundedAnswerBuildPlanPreview, setScholarChatGroundedAnswerBuildPlanPreview] = createSignal<ScholarChatGroundedAnswerBuildPlanPreview | null>(null);
  const [scholarChatGroundedAnswerBuildPlanError, setScholarChatGroundedAnswerBuildPlanError] = createSignal<string | null>(null);
  const [scholarChatGroundedAnswerBuildPlanValidationError, setScholarChatGroundedAnswerBuildPlanValidationError] = createSignal<string | null>(null);
  const [scholarChatGroundedAnswerBuildPlanLoading, setScholarChatGroundedAnswerBuildPlanLoading] = createSignal(false);
  const [scholarChatGroundedAnswerBuildPlanHasRun, setScholarChatGroundedAnswerBuildPlanHasRun] = createSignal(false);
  const [scholarChatGroundedAnswerCandidatePreview, setScholarChatGroundedAnswerCandidatePreview] = createSignal<ScholarChatGroundedAnswerCandidatePreview | null>(null);
  const [scholarChatGroundedAnswerCandidateError, setScholarChatGroundedAnswerCandidateError] = createSignal<string | null>(null);
  const [scholarChatGroundedAnswerCandidateValidationError, setScholarChatGroundedAnswerCandidateValidationError] = createSignal<string | null>(null);
  const [scholarChatGroundedAnswerCandidateLoading, setScholarChatGroundedAnswerCandidateLoading] = createSignal(false);
  const [scholarChatGroundedAnswerCandidateHasRun, setScholarChatGroundedAnswerCandidateHasRun] = createSignal(false);
  const [scholarChatGroundedAnswerWriteEligibilityPreview, setScholarChatGroundedAnswerWriteEligibilityPreview] = createSignal<ScholarChatGroundedAnswerWriteEligibilityPreview | null>(null);
  const [scholarChatGroundedAnswerWriteEligibilityError, setScholarChatGroundedAnswerWriteEligibilityError] = createSignal<string | null>(null);
  const [scholarChatGroundedAnswerWriteEligibilityValidationError, setScholarChatGroundedAnswerWriteEligibilityValidationError] = createSignal<string | null>(null);
  const [scholarChatGroundedAnswerWriteEligibilityLoading, setScholarChatGroundedAnswerWriteEligibilityLoading] = createSignal(false);
  const [scholarChatGroundedAnswerWriteEligibilityHasRun, setScholarChatGroundedAnswerWriteEligibilityHasRun] = createSignal(false);
  const [scholarChatGroundedAnswerBuildIntentAnswerDraftId, setScholarChatGroundedAnswerBuildIntentAnswerDraftId] = createSignal("");
  const [scholarChatGroundedAnswerBuildIntentExplicitUserIntent, setScholarChatGroundedAnswerBuildIntentExplicitUserIntent] = createSignal(false);
  const [scholarChatGroundedAnswerBuildIntentPreview, setScholarChatGroundedAnswerBuildIntentPreview] = createSignal<ScholarChatGroundedAnswerBuildIntentPreview | null>(null);
  const [scholarChatGroundedAnswerBuildIntentError, setScholarChatGroundedAnswerBuildIntentError] = createSignal<string | null>(null);
  const [scholarChatGroundedAnswerBuildIntentValidationError, setScholarChatGroundedAnswerBuildIntentValidationError] = createSignal<string | null>(null);
  const [scholarChatGroundedAnswerBuildIntentLoading, setScholarChatGroundedAnswerBuildIntentLoading] = createSignal(false);
  const [scholarChatGroundedAnswerBuildIntentHasRun, setScholarChatGroundedAnswerBuildIntentHasRun] = createSignal(false);
  const [scholarChatGroundedAnswerBuildRequestPreview, setScholarChatGroundedAnswerBuildRequestPreview] = createSignal<ScholarChatGroundedAnswerBuildRequestPreview | null>(null);
  const [scholarChatGroundedAnswerBuildRequestError, setScholarChatGroundedAnswerBuildRequestError] = createSignal<string | null>(null);
  const [scholarChatGroundedAnswerBuildRequestValidationError, setScholarChatGroundedAnswerBuildRequestValidationError] = createSignal<string | null>(null);
  const [scholarChatGroundedAnswerBuildRequestLoading, setScholarChatGroundedAnswerBuildRequestLoading] = createSignal(false);
  const [scholarChatGroundedAnswerBuildRequestHasRun, setScholarChatGroundedAnswerBuildRequestHasRun] = createSignal(false);
  const [scholarChatGroundedAnswerBuildPreflightPreview, setScholarChatGroundedAnswerBuildPreflightPreview] = createSignal<ScholarChatGroundedAnswerBuildPreflightPreview | null>(null);
  const [scholarChatGroundedAnswerBuildPreflightError, setScholarChatGroundedAnswerBuildPreflightError] = createSignal<string | null>(null);
  const [scholarChatGroundedAnswerBuildPreflightValidationError, setScholarChatGroundedAnswerBuildPreflightValidationError] = createSignal<string | null>(null);
  const [scholarChatGroundedAnswerBuildPreflightLoading, setScholarChatGroundedAnswerBuildPreflightLoading] = createSignal(false);
  const [scholarChatGroundedAnswerBuildPreflightHasRun, setScholarChatGroundedAnswerBuildPreflightHasRun] = createSignal(false);
  const [scholarChatGroundedAnswerExecutionReadinessExecutionConsent, setScholarChatGroundedAnswerExecutionReadinessExecutionConsent] = createSignal(false);
  const [scholarChatGroundedAnswerExecutionReadinessPreview, setScholarChatGroundedAnswerExecutionReadinessPreview] = createSignal<ScholarChatGroundedAnswerExecutionReadinessPreview | null>(null);
  const [scholarChatGroundedAnswerExecutionReadinessError, setScholarChatGroundedAnswerExecutionReadinessError] = createSignal<string | null>(null);
  const [scholarChatGroundedAnswerExecutionReadinessValidationError, setScholarChatGroundedAnswerExecutionReadinessValidationError] = createSignal<string | null>(null);
  const [scholarChatGroundedAnswerExecutionReadinessLoading, setScholarChatGroundedAnswerExecutionReadinessLoading] = createSignal(false);
  const [scholarChatGroundedAnswerExecutionReadinessHasRun, setScholarChatGroundedAnswerExecutionReadinessHasRun] = createSignal(false);
  const [scholarChatGroundedAnswerExecutionPlanPreview, setScholarChatGroundedAnswerExecutionPlanPreview] = createSignal<ScholarChatGroundedAnswerExecutionPlanPreview | null>(null);
  const [scholarChatGroundedAnswerExecutionPlanError, setScholarChatGroundedAnswerExecutionPlanError] = createSignal<string | null>(null);
  const [scholarChatGroundedAnswerExecutionPlanValidationError, setScholarChatGroundedAnswerExecutionPlanValidationError] = createSignal<string | null>(null);
  const [scholarChatGroundedAnswerExecutionPlanLoading, setScholarChatGroundedAnswerExecutionPlanLoading] = createSignal(false);
  const [scholarChatGroundedAnswerExecutionPlanHasRun, setScholarChatGroundedAnswerExecutionPlanHasRun] = createSignal(false);
  const [scholarChatRuntimeDiagnosticBridgePreview, setScholarChatRuntimeDiagnosticBridgePreview] = createSignal<ScholarChatRuntimeDiagnosticBridgePreview | null>(null);
  const [scholarChatRuntimeDiagnosticBridgeError, setScholarChatRuntimeDiagnosticBridgeError] = createSignal<string | null>(null);
  const [scholarChatRuntimeDiagnosticBridgeValidationError, setScholarChatRuntimeDiagnosticBridgeValidationError] = createSignal<string | null>(null);
  const [scholarChatRuntimeDiagnosticBridgeLoading, setScholarChatRuntimeDiagnosticBridgeLoading] = createSignal(false);
  const [scholarChatRuntimeDiagnosticBridgeHasRun, setScholarChatRuntimeDiagnosticBridgeHasRun] = createSignal(false);
  const [scholarChatRuntimeDiagnosticResultPreview, setScholarChatRuntimeDiagnosticResultPreview] = createSignal<ScholarChatRuntimeDiagnosticResultPreview | null>(null);
  const [scholarChatRuntimeDiagnosticResultError, setScholarChatRuntimeDiagnosticResultError] = createSignal<string | null>(null);
  const [scholarChatRuntimeDiagnosticResultValidationError, setScholarChatRuntimeDiagnosticResultValidationError] = createSignal<string | null>(null);
  const [scholarChatRuntimeDiagnosticResultLoading, setScholarChatRuntimeDiagnosticResultLoading] = createSignal(false);
  const [scholarChatRuntimeDiagnosticResultHasRun, setScholarChatRuntimeDiagnosticResultHasRun] = createSignal(false);
  const [localRuntimeSmokeExecutionPlanPreview, setLocalRuntimeSmokeExecutionPlanPreview] = createSignal<LocalRuntimeSmokeExecutionPlanPreview | null>(null);
  const [localRuntimeSmokeExecutionPlanError, setLocalRuntimeSmokeExecutionPlanError] = createSignal<string | null>(null);
  const [localRuntimeSmokeExecutionPlanValidationError, setLocalRuntimeSmokeExecutionPlanValidationError] = createSignal<string | null>(null);
  const [localRuntimeSmokeExecutionPlanLoading, setLocalRuntimeSmokeExecutionPlanLoading] = createSignal(false);
  const [localRuntimeSmokeExecutionPlanHasRun, setLocalRuntimeSmokeExecutionPlanHasRun] = createSignal(false);
  const [localRuntimeKind, setLocalRuntimeKind] = createSignal<LocalModelRuntimeKind>("llama_cpp");
  const [localRuntimeModelPath, setLocalRuntimeModelPath] = createSignal("");
  const [localRuntimeExecutablePath, setLocalRuntimeExecutablePath] = createSignal("");
  const [localRuntimeContextWindow, setLocalRuntimeContextWindow] = createSignal("4096");
  const [localRuntimeGpuLayers, setLocalRuntimeGpuLayers] = createSignal("0");
  const [localRuntimeTemperature, setLocalRuntimeTemperature] = createSignal("0.2");
  const [managedLlamaServerExecutablePath, setManagedLlamaServerExecutablePath] = createSignal("");
  const [managedLlamaServerModelPath, setManagedLlamaServerModelPath] = createSignal("");
  const [managedLlamaServerHost, setManagedLlamaServerHost] = createSignal("127.0.0.1");
  const [managedLlamaServerPort, setManagedLlamaServerPort] = createSignal("48921");
  const [managedLlamaServerAlias, setManagedLlamaServerAlias] = createSignal("aegis-local-gemma");
  const [managedLlamaServerContextWindow, setManagedLlamaServerContextWindow] = createSignal("4096");
  const [managedLlamaServerGpuLayers, setManagedLlamaServerGpuLayers] = createSignal("0");
  const [managedLlamaServerAllowStart, setManagedLlamaServerAllowStart] = createSignal(false);
  const [managedLlamaServerLaunchPreview, setManagedLlamaServerLaunchPreview] = createSignal<ManagedLlamaServerLaunchPlanPreview | null>(null);
  const [managedLlamaServerLaunchLoading, setManagedLlamaServerLaunchLoading] = createSignal(false);
  const [managedLlamaServerLaunchError, setManagedLlamaServerLaunchError] = createSignal<string | null>(null);
  const [managedLlamaServerLaunchHasRun, setManagedLlamaServerLaunchHasRun] = createSignal(false);
  const [managedLlamaServerStatusPreview, setManagedLlamaServerStatusPreview] = createSignal<ManagedLlamaServerStatusPreview | null>(null);
  const [managedLlamaServerStatusLoading, setManagedLlamaServerStatusLoading] = createSignal(false);
  const [managedLlamaServerStatusError, setManagedLlamaServerStatusError] = createSignal<string | null>(null);
  const [managedLlamaServerStatusHasRun, setManagedLlamaServerStatusHasRun] = createSignal(false);
  const [managedLlamaServerChatDiagnosticPrompt, setManagedLlamaServerChatDiagnosticPrompt] = createSignal("Say READY in one short sentence.");
  const [managedLlamaServerChatDiagnosticMaxTokens, setManagedLlamaServerChatDiagnosticMaxTokens] = createSignal("16");
  const [managedLlamaServerChatDiagnosticTemperature, setManagedLlamaServerChatDiagnosticTemperature] = createSignal("0.2");
  const [managedLlamaServerChatDiagnosticTimeoutMs, setManagedLlamaServerChatDiagnosticTimeoutMs] = createSignal("5000");
  const [managedLlamaServerChatDiagnosticAllowRun, setManagedLlamaServerChatDiagnosticAllowRun] = createSignal(false);
  const [managedLlamaServerChatDiagnosticPreview, setManagedLlamaServerChatDiagnosticPreview] = createSignal<ManagedLlamaServerChatDiagnosticPreview | null>(null);
  const [managedLlamaServerChatDiagnosticLoading, setManagedLlamaServerChatDiagnosticLoading] = createSignal(false);
  const [managedLlamaServerChatDiagnosticError, setManagedLlamaServerChatDiagnosticError] = createSignal<string | null>(null);
  const [managedLlamaServerChatDiagnosticHasRun, setManagedLlamaServerChatDiagnosticHasRun] = createSignal(false);
  const [managedLlamaServerSmokeDiagnosticPrompt, setManagedLlamaServerSmokeDiagnosticPrompt] = createSignal("Say READY in one short sentence.");
  const [managedLlamaServerSmokeDiagnosticMaxOutputTokens, setManagedLlamaServerSmokeDiagnosticMaxOutputTokens] = createSignal("16");
  const [managedLlamaServerSmokeDiagnosticTimeoutMs, setManagedLlamaServerSmokeDiagnosticTimeoutMs] = createSignal("5000");
  const [managedLlamaServerSmokeDiagnosticAllowRun, setManagedLlamaServerSmokeDiagnosticAllowRun] = createSignal(false);
  const [managedLlamaServerSmokeDiagnosticPreview, setManagedLlamaServerSmokeDiagnosticPreview] = createSignal<ManagedLlamaServerSmokeDiagnosticPreview | null>(null);
  const [managedLlamaServerSmokeDiagnosticLoading, setManagedLlamaServerSmokeDiagnosticLoading] = createSignal(false);
  const [managedLlamaServerSmokeDiagnosticError, setManagedLlamaServerSmokeDiagnosticError] = createSignal<string | null>(null);
  const [managedLlamaServerSmokeDiagnosticHasRun, setManagedLlamaServerSmokeDiagnosticHasRun] = createSignal(false);
  const [localRuntimeAdapterExecutablePath, setLocalRuntimeAdapterExecutablePath] = createSignal("");
  const [localRuntimeAdapterModelPath, setLocalRuntimeAdapterModelPath] = createSignal("");
  const [localRuntimeAdapterModelFamily, setLocalRuntimeAdapterModelFamily] = createSignal("");
  const [localRuntimeAdapterModelFormat, setLocalRuntimeAdapterModelFormat] = createSignal("");
  const [localRuntimeAdapterContextWindowTokens, setLocalRuntimeAdapterContextWindowTokens] = createSignal("");
  const [localRuntimeAdapterGpuLayers, setLocalRuntimeAdapterGpuLayers] = createSignal("");
  const [localRuntimeAdapterThreads, setLocalRuntimeAdapterThreads] = createSignal("");
  const [localRuntimeAdapterBatchSize, setLocalRuntimeAdapterBatchSize] = createSignal("");
  const [localRuntimeAdapterChatTemplate, setLocalRuntimeAdapterChatTemplate] = createSignal("");
  const [localRuntimeAdapterPreview, setLocalRuntimeAdapterPreview] = createSignal<LocalRuntimeAdapterContractPreview | null>(null);
  const [localRuntimeAdapterError, setLocalRuntimeAdapterError] = createSignal<string | null>(null);
  const [localRuntimeAdapterValidationError, setLocalRuntimeAdapterValidationError] = createSignal<string | null>(null);
  const [localRuntimeAdapterLoading, setLocalRuntimeAdapterLoading] = createSignal(false);
  const [localRuntimeAdapterHasRun, setLocalRuntimeAdapterHasRun] = createSignal(false);
  const [localRuntimeValidationPreview, setLocalRuntimeValidationPreview] = createSignal<LocalRuntimeValidationPreview | null>(null);
  const [localRuntimeValidationPreviewError, setLocalRuntimeValidationPreviewError] = createSignal<string | null>(null);
  const [localRuntimeValidationPreviewInputError, setLocalRuntimeValidationPreviewInputError] = createSignal<string | null>(null);
  const [localRuntimeValidationPreviewLoading, setLocalRuntimeValidationPreviewLoading] = createSignal(false);
  const [localRuntimeValidationPreviewHasRun, setLocalRuntimeValidationPreviewHasRun] = createSignal(false);
  const [localRuntimeProbeReadinessConsent, setLocalRuntimeProbeReadinessConsent] = createSignal(false);
  const [localRuntimeProbeReadinessPreview, setLocalRuntimeProbeReadinessPreview] = createSignal<LocalRuntimeProbeReadinessPreview | null>(null);
  const [localRuntimeProbeReadinessPreviewError, setLocalRuntimeProbeReadinessPreviewError] = createSignal<string | null>(null);
  const [localRuntimeProbeReadinessPreviewInputError, setLocalRuntimeProbeReadinessPreviewInputError] = createSignal<string | null>(null);
  const [localRuntimeProbeReadinessPreviewLoading, setLocalRuntimeProbeReadinessPreviewLoading] = createSignal(false);
  const [localRuntimeProbeReadinessPreviewHasRun, setLocalRuntimeProbeReadinessPreviewHasRun] = createSignal(false);
  const [localRuntimePreview, setLocalRuntimePreview] = createSignal<LocalModelRuntimeHealthPreview | null>(null);
  const [localRuntimeError, setLocalRuntimeError] = createSignal<string | null>(null);
  const [localRuntimeValidationError, setLocalRuntimeValidationError] = createSignal<string | null>(null);
  const [localRuntimeLoading, setLocalRuntimeLoading] = createSignal(false);
  const [localRuntimeHasRun, setLocalRuntimeHasRun] = createSignal(false);
  const [localRuntimeInvocationMaxOutputTokens, setLocalRuntimeInvocationMaxOutputTokens] = createSignal("");
  const [localRuntimeInvocationStopSequences, setLocalRuntimeInvocationStopSequences] = createSignal("");
  const [localRuntimeInvocationPreview, setLocalRuntimeInvocationPreview] = createSignal<LocalRuntimeInvocationPlanPreview | null>(null);
  const [localRuntimeInvocationError, setLocalRuntimeInvocationError] = createSignal<string | null>(null);
  const [localRuntimeInvocationValidationError, setLocalRuntimeInvocationValidationError] = createSignal<string | null>(null);
  const [localRuntimeInvocationLoading, setLocalRuntimeInvocationLoading] = createSignal(false);
  const [localRuntimeInvocationHasRun, setLocalRuntimeInvocationHasRun] = createSignal(false);
  const [localRuntimeProbeAllowExecution, setLocalRuntimeProbeAllowExecution] = createSignal(false);
  const [localRuntimeProbeTimeoutMs, setLocalRuntimeProbeTimeoutMs] = createSignal("1500");
  const [localRuntimeProbeResult, setLocalRuntimeProbeResult] = createSignal<LocalRuntimeVersionProbePreview | null>(null);
  const [localRuntimeProbeError, setLocalRuntimeProbeError] = createSignal<string | null>(null);
  const [localRuntimeProbeValidationError, setLocalRuntimeProbeValidationError] = createSignal<string | null>(null);
  const [localRuntimeProbeLoading, setLocalRuntimeProbeLoading] = createSignal(false);
  const [localRuntimeProbeHasRun, setLocalRuntimeProbeHasRun] = createSignal(false);
  const [localRuntimeCapabilityResult, setLocalRuntimeCapabilityResult] = createSignal<LocalRuntimeCapabilityPreview | null>(null);
  const [localRuntimeCapabilityError, setLocalRuntimeCapabilityError] = createSignal<string | null>(null);
  const [localRuntimeCapabilityLoading, setLocalRuntimeCapabilityLoading] = createSignal(false);
  const [localRuntimeCapabilityHasRun, setLocalRuntimeCapabilityHasRun] = createSignal(false);
  const [localRuntimeSmokeReadinessConsent, setLocalRuntimeSmokeReadinessConsent] = createSignal(false);
  const [localRuntimeSmokeReadinessResult, setLocalRuntimeSmokeReadinessResult] = createSignal<LocalRuntimeSmokeReadinessPreview | null>(null);
  const [localRuntimeSmokeReadinessError, setLocalRuntimeSmokeReadinessError] = createSignal<string | null>(null);
  const [localRuntimeSmokeReadinessValidationError, setLocalRuntimeSmokeReadinessValidationError] = createSignal<string | null>(null);
  const [localRuntimeSmokeReadinessLoading, setLocalRuntimeSmokeReadinessLoading] = createSignal(false);
  const [localRuntimeSmokeReadinessHasRun, setLocalRuntimeSmokeReadinessHasRun] = createSignal(false);
  const [localRuntimeSmokePrompt, setLocalRuntimeSmokePrompt] = createSignal("Say READY in one short sentence.");
  const [localRuntimeSmokeAllowExecution, setLocalRuntimeSmokeAllowExecution] = createSignal(false);
  const [localRuntimeSmokeTimeoutMs, setLocalRuntimeSmokeTimeoutMs] = createSignal("5000");
  const [localRuntimeSmokeMaxOutputTokens, setLocalRuntimeSmokeMaxOutputTokens] = createSignal("16");
  const [localRuntimeSmokeResult, setLocalRuntimeSmokeResult] = createSignal<LocalRuntimeSmokeDiagnosticPreview | null>(null);
  const [localRuntimeSmokeError, setLocalRuntimeSmokeError] = createSignal<string | null>(null);
  const [localRuntimeSmokeValidationError, setLocalRuntimeSmokeValidationError] = createSignal<string | null>(null);
  const [localRuntimeSmokeLoading, setLocalRuntimeSmokeLoading] = createSignal(false);
  const [localRuntimeSmokeHasRun, setLocalRuntimeSmokeHasRun] = createSignal(false);
  const [sourceId, setSourceId] = createSignal("");
  const [finalAnswerId, setFinalAnswerId] = createSignal("");
  const [finalAnswer, setFinalAnswer] = createSignal<FinalAnswer | null>(null);
  const [finalAnswerError, setFinalAnswerError] = createSignal<string | null>(null);
  const [finalAnswerLoading, setFinalAnswerLoading] = createSignal(false);
  const [artifactOverview, setArtifactOverview] = createSignal<AnswerArtifactOverview | null>(null);
  const [artifactOverviewError, setArtifactOverviewError] = createSignal<string | null>(null);
  const [artifactOverviewLoading, setArtifactOverviewLoading] = createSignal(false);
  const [retrievalIndex, setRetrievalIndex] = createSignal<RetrievalIndex | null>(null);
  const [retrievalIndexError, setRetrievalIndexError] = createSignal<string | null>(null);
  const [retrievalIndexLoading, setRetrievalIndexLoading] = createSignal(false);
  const [retrievalSearchSourceId, setRetrievalSearchSourceId] = createSignal("");
  const [retrievalSearchQuery, setRetrievalSearchQuery] = createSignal("");
  const [retrievalSearch, setRetrievalSearch] = createSignal<RetrievalSearchResponse | null>(null);
  const [retrievalSearchHasRun, setRetrievalSearchHasRun] = createSignal(false);
  const [retrievalSearchValidationError, setRetrievalSearchValidationError] = createSignal<string | null>(null);
  const [retrievalSearchError, setRetrievalSearchError] = createSignal<string | null>(null);
  const [retrievalSearchLoading, setRetrievalSearchLoading] = createSignal(false);
  const [artifactSources, setArtifactSources] = createSignal<AnswerArtifactSourceMetadata[]>([]);
  const [artifactSourcesError, setArtifactSourcesError] = createSignal<string | null>(null);
  const [artifactSourcesLoading, setArtifactSourcesLoading] = createSignal(false);
  const [artifactHealth, setArtifactHealth] = createSignal<AnswerArtifactHealth | null>(null);
  const [artifactHealthError, setArtifactHealthError] = createSignal<string | null>(null);
  const [artifactHealthLoading, setArtifactHealthLoading] = createSignal(false);
  const [artifactIssues, setArtifactIssues] = createSignal<AnswerArtifactIssue[]>([]);
  const [artifactIssuesError, setArtifactIssuesError] = createSignal<string | null>(null);
  const [artifactIssuesLoading, setArtifactIssuesLoading] = createSignal(false);
  const [artifactIssuesHasRun, setArtifactIssuesHasRun] = createSignal(false);
  const [evidencePacks, setEvidencePacks] = createSignal<EvidencePackMetadata[] | null>(null);
  const [evidencePacksError, setEvidencePacksError] = createSignal<string | null>(null);
  const [evidencePacksLoading, setEvidencePacksLoading] = createSignal(false);
  const [evidencePacksSourceId, setEvidencePacksSourceId] = createSignal("");
  const [artifactManifest, setArtifactManifest] = createSignal<AnswerArtifactExportManifest | null>(null);
  const [artifactManifestError, setArtifactManifestError] = createSignal<string | null>(null);
  const [artifactManifestLoading, setArtifactManifestLoading] = createSignal(false);
  const [exportRoot, setExportRoot] = createSignal("");
  const [artifactExportResult, setArtifactExportResult] = createSignal<AnswerArtifactExportResult | null>(null);
  const [artifactExportError, setArtifactExportError] = createSignal<string | null>(null);
  const [artifactExportLoading, setArtifactExportLoading] = createSignal(false);
  const [exportBundleRoot, setExportBundleRoot] = createSignal("");
  const [artifactBundleInspection, setArtifactBundleInspection] = createSignal<AnswerArtifactExportBundleInspection | null>(null);
  const [artifactBundleInspectionError, setArtifactBundleInspectionError] = createSignal<string | null>(null);
  const [artifactBundleInspectionLoading, setArtifactBundleInspectionLoading] = createSignal(false);

  async function loadStatus() {
    setStatusError(null);
    try {
      const result = await invoke<CorpusStatus>("get_corpus_status", {
        root: ".",
      });
      setStatus(result);
    } catch (err) {
      setStatusError(sanitizeBackendError(err));
    }
  }

  async function loadFinalAnswer() {
    await loadFinalAnswerByIds(sourceId().trim(), finalAnswerId().trim());
  }

  async function loadFinalAnswerByIds(trimmedSourceId: string, trimmedFinalAnswerId: string) {
    if (!trimmedSourceId || !trimmedFinalAnswerId) {
      setFinalAnswerError("Source ID and final answer ID are required.");
      return;
    }

    if (finalAnswerLoading()) {
      return;
    }

    setFinalAnswerLoading(true);
    setFinalAnswerError(null);
    try {
      const result = await invoke<FinalAnswer>("get_final_answer", {
        root: ".",
        source_id: trimmedSourceId,
        final_answer_id: trimmedFinalAnswerId,
      });
      setFinalAnswer(result);
    } catch (err) {
      setFinalAnswerError(sanitizeBackendError(err));
    } finally {
      setFinalAnswerLoading(false);
    }
  }

  async function loadArtifactOverview() {
    const selectedSourceId = selectedAnswerArtifactSourceId();
    if (!selectedSourceId) {
      setArtifactOverview(null);
      return;
    }
    await loadArtifactOverviewBySourceId(selectedSourceId);
  }

  async function loadRetrievalIndex(preserveSelection = false, sourceIdOverride?: string) {
    const trimmedSourceId = (sourceIdOverride ?? sourceId()).trim();
    if (!trimmedSourceId) {
      setRetrievalIndexError("Source ID is required to load the retrieval index.");
      return;
    }
    if (retrievalIndexLoading()) {
      return;
    }
    setRetrievalIndexLoading(true);
    setRetrievalIndexError(null);
    try {
      const result = await invoke<RetrievalIndex>("get_retrieval_index", {
        root: ".",
        source_id: trimmedSourceId,
      });
      setRetrievalIndex(result);
      const sourceIds = Array.from(new Set(result.entries.map((entry) => entry.source_id))).sort();
      if (preserveSelection && sourceIds.includes(retrievalSearchSourceId().trim())) {
        setRetrievalSearchSourceId(retrievalSearchSourceId().trim());
      } else {
        setRetrievalSearchSourceId(sourceIds[0] ?? "");
      }
      setRetrievalSearch(null);
      setRetrievalSearchHasRun(false);
      setRetrievalSearchError(null);
      setRetrievalSearchValidationError(null);
      clearScholarChatDraftInferencePreview();
    } catch (err) {
      if (!preserveSelection) {
        setRetrievalSearchSourceId("");
        setRetrievalSearch(null);
        setRetrievalSearchHasRun(false);
        setRetrievalSearchError(null);
        setRetrievalSearchValidationError(null);
        setRetrievalIndex(null);
      }
      setRetrievalIndexError(sanitizeBackendError(err));
    } finally {
      setRetrievalIndexLoading(false);
    }
  }

  async function loadScholarChatSourceContext(preserveSelection = false) {
    if (scholarChatSourceContextLoading()) {
      return;
    }
    setScholarChatSourceContextLoading(true);
    setScholarChatSourceContextError(null);
    try {
      const result = await invoke<RegisteredSource[]>("list_sources", {
        root: ".",
      });
      const nextSourceContext = [...result].sort((left, right) => left.source_id.localeCompare(right.source_id));
      setScholarChatSourceContext(nextSourceContext);
      const availableSourceIds = new Set(nextSourceContext.map((item) => item.source_id));
      setScholarChatSourceContextSelectedIds((current) =>
        current
          .filter((sourceId) => availableSourceIds.has(sourceId))
          .sort((left, right) => left.localeCompare(right)),
      );
      if (!preserveSelection) {
        setScholarChatSourceContextTouched(false);
      }
      clearScholarChatDraftInferencePreview();
    } catch (err) {
      if (!preserveSelection) {
        setScholarChatSourceContext([]);
        setScholarChatSourceContextSelectedIds([]);
        setScholarChatSourceContextTouched(false);
      }
      setScholarChatSourceContextError(sanitizeBackendError(err));
    } finally {
      setScholarChatSourceContextLoading(false);
    }
  }

  async function runRetrievalSearch() {
    const trimmedSourceId = retrievalSearchSourceId().trim();
    const trimmedQuery = retrievalSearchQuery().trim();
    if (!retrievalSearchSourceIds().length || !trimmedSourceId) {
      setRetrievalSearchHasRun(true);
      setRetrievalSearch(null);
      setRetrievalSearchValidationError("Select a source ID from the loaded retrieval index.");
      setRetrievalSearchError(null);
      return;
    }
    if (!trimmedQuery) {
      setRetrievalSearchHasRun(true);
      setRetrievalSearch(null);
      setRetrievalSearchValidationError("Query is required to run retrieval search.");
      setRetrievalSearchError(null);
      return;
    }
    if (retrievalSearchLoading()) {
      return;
    }
    setRetrievalSearchHasRun(true);
    setRetrievalSearchLoading(true);
    setRetrievalSearchValidationError(null);
    setRetrievalSearchError(null);
    setRetrievalSearch(null);
    try {
      const result = await invoke<RetrievalSearchResponse>("search_source", {
        root: ".",
        source_id: trimmedSourceId,
        query: trimmedQuery,
        max_results: 25,
      });
      setRetrievalSearch(result);
    } catch (err) {
      setRetrievalSearchError(sanitizeBackendError(err));
    } finally {
      setRetrievalSearchLoading(false);
    }
  }

  function retrievalSearchSourceIds() {
    return Array.from(new Set((retrievalIndex()?.entries ?? []).map((entry) => entry.source_id))).sort();
  }

  function selectRetrievalSearchSourceId(nextSourceId: string) {
    setRetrievalSearchSourceId(nextSourceId);
    setRetrievalSearch(null);
    setRetrievalSearchHasRun(false);
    setRetrievalSearchError(null);
    setRetrievalSearchValidationError(null);
    clearScholarChatDraftInferencePreview();
  }

  async function loadArtifactSources(preserveExisting = false) {
    if (artifactSourcesLoading()) {
      return;
    }
    setArtifactSourcesLoading(true);
    setArtifactSourcesError(null);
    try {
      const result = await invoke<AnswerArtifactSourceMetadata[]>("list_answer_artifact_sources", {
        root: ".",
      });
      setArtifactSources(result);
      clearScholarChatDraftInferencePreview();
    } catch (err) {
      if (!preserveExisting) {
        setArtifactSources([]);
      }
      setArtifactSourcesError(sanitizeBackendError(err));
    } finally {
      setArtifactSourcesLoading(false);
    }
  }

  async function loadArtifactHealth(preserveExisting = false) {
    if (artifactHealthLoading()) {
      return;
    }
    setArtifactHealthLoading(true);
    setArtifactHealthError(null);
    try {
      const result = await invoke<AnswerArtifactHealth>("get_answer_artifact_health", {
        root: ".",
      });
      setArtifactHealth(result);
    } catch (err) {
      if (!preserveExisting) {
        setArtifactHealth(null);
      }
      setArtifactHealthError(sanitizeBackendError(err));
    } finally {
      setArtifactHealthLoading(false);
    }
  }

  async function loadArtifactIssues(preserveExisting = false) {
    if (artifactIssuesLoading()) {
      return;
    }
    setArtifactIssuesHasRun(true);
    setArtifactIssuesLoading(true);
    setArtifactIssuesError(null);
    try {
      const result = await invoke<AnswerArtifactIssue[]>("list_answer_artifact_issues", {
        root: ".",
      });
      setArtifactIssues(result);
    } catch (err) {
      if (!preserveExisting) {
        setArtifactIssues([]);
      }
      setArtifactIssuesError(sanitizeBackendError(err));
    } finally {
      setArtifactIssuesLoading(false);
    }
  }

  async function loadEvidencePacksBySourceId(trimmedSourceId: string) {
    if (!trimmedSourceId) {
      return;
    }
    if (evidencePacksLoading()) {
      return;
    }
    setEvidencePacksLoading(true);
    setEvidencePacksError(null);
    setEvidencePacksSourceId(trimmedSourceId);
    try {
      const result = await invoke<EvidencePackMetadata[]>("list_evidence_packs", {
        root: ".",
        source_id: trimmedSourceId,
      });
      setEvidencePacks(result);
    } catch (err) {
      setEvidencePacks(null);
      setEvidencePacksError(sanitizeBackendError(err));
    } finally {
      setEvidencePacksLoading(false);
    }
  }

  async function loadEvidencePacks() {
    const selectedSourceId = selectedEvidencePackSourceId();
    if (!selectedSourceId) {
      setEvidencePacks(null);
      setEvidencePacksError(null);
      setEvidencePacksSourceId("");
      return;
    }
    await loadEvidencePacksBySourceId(selectedSourceId);
  }

  async function loadArtifactManifest() {
    if (artifactManifestLoading()) {
      return;
    }
    setArtifactManifestLoading(true);
    setArtifactManifestError(null);
    try {
      const result = await invoke<AnswerArtifactExportManifest>("get_answer_artifact_export_manifest", {
        root: ".",
      });
      setArtifactManifest(result);
    } catch (err) {
      setArtifactManifest(null);
      setArtifactManifestError(sanitizeBackendError(err));
    } finally {
      setArtifactManifestLoading(false);
    }
  }

  async function exportArtifacts() {
    const trimmedExportRoot = exportRoot().trim();
    if (!trimmedExportRoot) {
      setArtifactExportError("Export destination is required.");
      return;
    }
    if (artifactExportLoading()) {
      return;
    }
    setArtifactExportLoading(true);
    setArtifactExportError(null);
    try {
      const result = await invoke<AnswerArtifactExportResult>("export_answer_artifacts", {
        root: ".",
        export_root: trimmedExportRoot,
      });
      setArtifactExportResult(result);
    } catch (err) {
      setArtifactExportResult(null);
      setArtifactExportError(sanitizeBackendError(err));
    } finally {
      setArtifactExportLoading(false);
    }
  }

  async function inspectExportBundle() {
    const trimmedExportBundleRoot = exportBundleRoot().trim();
    if (!trimmedExportBundleRoot) {
      setArtifactBundleInspectionError("Export bundle root is required.");
      return;
    }
    if (artifactBundleInspectionLoading()) {
      return;
    }
    setArtifactBundleInspectionLoading(true);
    setArtifactBundleInspectionError(null);
    try {
      const result = await invoke<AnswerArtifactExportBundleInspection>("inspect_answer_artifact_export_bundle", {
        export_root: trimmedExportBundleRoot,
      });
      setArtifactBundleInspection(result);
    } catch (err) {
      setArtifactBundleInspection(null);
      setArtifactBundleInspectionError(sanitizeBackendError(err));
    } finally {
      setArtifactBundleInspectionLoading(false);
    }
  }

  async function selectArtifactSource(item: AnswerArtifactSourceMetadata) {
    setSourceId(item.source_id);
    setFinalAnswerId("");
    setFinalAnswer(null);
    setFinalAnswerError(null);
    clearScholarChatDraftInferencePreview();
    await loadArtifactOverviewBySourceId(item.source_id);
  }

  function diagnosticsAreLoading() {
    return retrievalIndexLoading() || artifactSourcesLoading() || artifactHealthLoading() || artifactIssuesLoading() || evidencePacksLoading() || scholarChatSourceContextLoading();
  }

  function selectedAnswerArtifactSourceId() {
    const trimmedSourceId = sourceId().trim();
    return artifactSources().some((item) => item.source_id === trimmedSourceId) ? trimmedSourceId : "";
  }

  function selectedEvidencePackSourceId() {
    const retrievalSourceId = retrievalSearchSourceId().trim();
    if (retrievalSearchSourceIds().includes(retrievalSourceId)) {
      return retrievalSourceId;
    }
    const answerSourceId = selectedAnswerArtifactSourceId();
    return answerSourceId || "";
  }

  function selectedScholarChatSourceIds() {
    const explicitSourceIds = [...new Set(scholarChatSourceContextSelectedIds())].sort((left, right) => left.localeCompare(right));
    if (scholarChatSourceContextTouched()) {
      return explicitSourceIds;
    }
    const selectedSourceId = selectedEvidencePackSourceId();
    return selectedSourceId ? [selectedSourceId] : [];
  }

  function scholarChatSelectedSourceIdsSummary() {
    const explicitSourceIds = [...new Set(scholarChatSourceContextSelectedIds())].sort((left, right) => left.localeCompare(right));
    if (explicitSourceIds.length > 0) {
      return `Selected source context: ${explicitSourceIds.join(", ")}`;
    }
    if (scholarChatSourceContextTouched()) {
      return "No Scholar Chat source context selected; preview will be unscoped.";
    }
    if (selectedEvidencePackSourceId()) {
      return "No Scholar Chat source context selected yet; using existing diagnostic source context.";
    }
    return "No Scholar Chat source context selected; preview will be unscoped.";
  }

  function managedLlamaServerReadinessSummary() {
    const status = managedLlamaServerStatusPreview();
    if (!status) {
      return "AEGIS can start and stop only the local llama-server it owns. External servers stay outside its control.";
    }
    const occupancyLabel = formatSnakeCaseLabel(status.port_occupancy_status);
    const ownershipLabel = status.owns_active_server ? "owned by AEGIS" : status.port_occupied_by_unmanaged_process ? "external / unmanaged" : "not active";
    if (status.port_occupied_by_unmanaged_process) {
      return `Managed llama-server: ${formatSnakeCaseLabel(status.health_status)} / ${formatSnakeCaseLabel(status.lifecycle_status)}. Port occupancy: ${occupancyLabel} (${ownershipLabel}). AEGIS will not stop external servers.`;
    }
    return `Managed llama-server: ${formatSnakeCaseLabel(status.health_status)} / ${formatSnakeCaseLabel(status.lifecycle_status)}. Port occupancy: ${occupancyLabel} (${ownershipLabel}). AEGIS can start and stop only the server it owns.`;
  }

  function formatSnakeCaseLabel(value: string) {
    return value
      .replace(/_/g, " ")
      .split(" ")
      .filter(Boolean)
      .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
      .join(" ");
  }

  function renderFirstRunSourceReadiness() {
    return (
      <div class="warning-box">
        <h4>No local sources yet</h4>
        <p>Use the Source Import Wizard to register a local file and then move through extraction, chunking, and retrieval indexing step by step. AEGIS Scholar can later create Evidence Packs from already indexed sources.</p>
        <div class="contract-meta">
          <div>
            <span>Supported now</span>
            <strong>Markdown / text notes, dataset notes, web snapshots, PDF text-layer extraction</strong>
          </div>
          <div>
            <span>Not yet</span>
            <strong>OCR for scanned PDFs, drag-and-drop import, automatic literature sync</strong>
          </div>
        </div>
        <h4>Next actions</h4>
        <ul>
          <li>Use the Source Import Wizard to register a local file.</li>
          <li>Check corpus status.</li>
          <li>Then run extraction, chunking, and retrieval.</li>
        </ul>
      </div>
    );
  }

  function renderSourceWorkflowActionHints() {
    return (
      <details class="muted">
        <summary>Manual source workflow hints</summary>
        <p>Use this help text after registering or selecting a source. The chat workflow plan above remains the primary entry point, and the Source Import Wizard handles the local onboarding slice.</p>
        <ol>
          <li><strong>Register a local source</strong> - Markdown / text notes, PDF text-layer extraction, dataset notes, and web snapshots are supported now.</li>
          <li><strong>Extract text</strong> - scanned PDF OCR is not supported yet.</li>
          <li><strong>Chunk the source</strong> - keep source locators and provenance intact.</li>
          <li><strong>Build / inspect retrieval</strong> - preview retrieval candidates and retrieval index health.</li>
          <li><strong>Build / read Evidence Packs</strong> - where supported by the current source and preview flow.</li>
        </ol>
        <p>Broad PDF ingestion beyond text-layer extraction is not yet supported.</p>
      </details>
    );
  }

  function toggleScholarChatSourceContext(sourceId: string) {
    setScholarChatSourceContextTouched(true);
    setScholarChatSourceContextSelectedIds((current) => {
      const nextIds = new Set(current);
      if (nextIds.has(sourceId)) {
        nextIds.delete(sourceId);
      } else {
        nextIds.add(sourceId);
      }
      return Array.from(nextIds).sort((left, right) => left.localeCompare(right));
    });
    setScholarChatPreview(null);
    setScholarChatError(null);
    setScholarChatValidationError(null);
    setScholarChatRetrievalPreview(null);
    setScholarChatRetrievalError(null);
    setScholarChatRetrievalHasRun(false);
    setScholarChatEvidencePlanPreview(null);
    setScholarChatEvidencePlanError(null);
    setScholarChatEvidencePlanHasRun(false);
    setScholarChatPromptPackPreview(null);
    setScholarChatPromptPackError(null);
    setScholarChatPromptPackHasRun(false);
    clearScholarChatGroundedAnswerBuildIntentPreview();
    clearScholarChatAnswerReadinessPreview();
    clearScholarChatDraftInferencePreview();
    clearLocalRuntimeInvocationPreview();
  }

  function clearScholarChatPromptPackPreview() {
    setScholarChatPromptPackPreview(null);
    setScholarChatPromptPackError(null);
    setScholarChatPromptPackHasRun(false);
    clearScholarChatGroundedAnswerBuildIntentPreview();
    clearScholarChatAnswerReadinessPreview();
    clearScholarChatDraftInferencePreview();
    clearLocalRuntimeInvocationPreview();
  }

  function clearLocalRuntimePreview() {
    setLocalRuntimePreview(null);
    setLocalRuntimeError(null);
    setLocalRuntimeValidationError(null);
    setLocalRuntimeHasRun(false);
    clearScholarChatAnswerReadinessPreview();
    clearScholarChatDraftInferencePreview();
  }

  function clearManagedLlamaServerLaunchPreview() {
    setManagedLlamaServerLaunchPreview(null);
    setManagedLlamaServerLaunchError(null);
    setManagedLlamaServerLaunchHasRun(false);
  }

  function clearManagedLlamaServerStatusPreview() {
    setManagedLlamaServerStatusPreview(null);
    setManagedLlamaServerStatusError(null);
    setManagedLlamaServerStatusHasRun(false);
  }

  function buildManagedLlamaServerLaunchPlanRequest(): ManagedLlamaServerLaunchPlanRequest | null {
    const port = parseOptionalIntegerInput(
      managedLlamaServerPort(),
      "Managed llama-server port",
      setManagedLlamaServerLaunchError,
    );
    const contextWindow = parseOptionalIntegerInput(
      managedLlamaServerContextWindow(),
      "Managed llama-server context window",
      setManagedLlamaServerLaunchError,
    );
    const gpuLayers = parseOptionalIntegerInput(
      managedLlamaServerGpuLayers(),
      "Managed llama-server GPU layers",
      setManagedLlamaServerLaunchError,
    );

    if (port === undefined || contextWindow === undefined || gpuLayers === undefined) {
      return null;
    }

    return {
      executable_path: normalizeOptionalTextInput(managedLlamaServerExecutablePath()),
      model_path: normalizeOptionalTextInput(managedLlamaServerModelPath()),
      host: normalizeOptionalTextInput(managedLlamaServerHost()),
      port,
      alias: normalizeOptionalTextInput(managedLlamaServerAlias()),
      context_window: contextWindow,
      gpu_layers: gpuLayers,
    };
  }

  async function loadManagedLlamaServerStatus() {
    if (managedLlamaServerStatusLoading()) {
      return;
    }

    clearManagedLlamaServerChatDiagnosticPreview();
    clearManagedLlamaServerSmokeDiagnosticPreview();
    setManagedLlamaServerStatusLoading(true);
    setManagedLlamaServerStatusError(null);
    try {
      const result = await invoke<ManagedLlamaServerStatusPreview>("inspect_managed_llama_server_status");
      setManagedLlamaServerStatusPreview(result);
      setManagedLlamaServerStatusHasRun(true);
    } catch (err) {
      setManagedLlamaServerStatusError(sanitizeBackendError(err));
    } finally {
      setManagedLlamaServerStatusLoading(false);
    }
  }

  function clearManagedLlamaServerSmokeDiagnosticPreview() {
    setManagedLlamaServerSmokeDiagnosticPreview(null);
    setManagedLlamaServerSmokeDiagnosticError(null);
    setManagedLlamaServerSmokeDiagnosticHasRun(false);
  }

  async function previewManagedLlamaServerLaunchPlan() {
    if (managedLlamaServerLaunchLoading()) {
      return;
    }

    const request = buildManagedLlamaServerLaunchPlanRequest();
    if (!request) {
      setManagedLlamaServerLaunchHasRun(true);
      setManagedLlamaServerLaunchPreview(null);
      return;
    }

    setManagedLlamaServerLaunchLoading(true);
    setManagedLlamaServerLaunchError(null);
    try {
      const result = await invoke<ManagedLlamaServerLaunchPlanPreview>("preview_managed_llama_server_launch_plan", {
        root: ".",
        request,
      });
      setManagedLlamaServerLaunchPreview(result);
      setManagedLlamaServerLaunchHasRun(true);
    } catch (err) {
      setManagedLlamaServerLaunchError(sanitizeBackendError(err));
    } finally {
      setManagedLlamaServerLaunchLoading(false);
    }
  }

  async function startManagedLlamaServer() {
    if (managedLlamaServerStatusLoading()) {
      return;
    }

    clearManagedLlamaServerChatDiagnosticPreview();
    clearManagedLlamaServerSmokeDiagnosticPreview();
    const request = buildManagedLlamaServerLaunchPlanRequest();
    if (!request) {
      setManagedLlamaServerStatusHasRun(true);
      setManagedLlamaServerStatusPreview(null);
      return;
    }

    setManagedLlamaServerStatusLoading(true);
    setManagedLlamaServerStatusError(null);
    try {
      const result = await invoke<ManagedLlamaServerStatusPreview>("start_managed_llama_server", {
        root: ".",
        request: {
          allow_server_start: managedLlamaServerAllowStart(),
          launch_plan_request: request,
        } satisfies ManagedLlamaServerStartRequest,
      });
      setManagedLlamaServerStatusPreview(result);
      setManagedLlamaServerStatusHasRun(true);
    } catch (err) {
      setManagedLlamaServerStatusError(sanitizeBackendError(err));
    } finally {
      setManagedLlamaServerStatusLoading(false);
    }
  }

  async function checkManagedLlamaServerHealth() {
    if (managedLlamaServerStatusLoading()) {
      return;
    }

    clearManagedLlamaServerChatDiagnosticPreview();
    clearManagedLlamaServerSmokeDiagnosticPreview();
    setManagedLlamaServerStatusLoading(true);
    setManagedLlamaServerStatusError(null);
    try {
      const result = await invoke<ManagedLlamaServerStatusPreview>("check_managed_llama_server_health");
      setManagedLlamaServerStatusPreview(result);
      setManagedLlamaServerStatusHasRun(true);
    } catch (err) {
      setManagedLlamaServerStatusError(sanitizeBackendError(err));
    } finally {
      setManagedLlamaServerStatusLoading(false);
    }
  }

  async function stopManagedLlamaServer() {
    if (managedLlamaServerStatusLoading()) {
      return;
    }

    clearManagedLlamaServerChatDiagnosticPreview();
    clearManagedLlamaServerSmokeDiagnosticPreview();
    setManagedLlamaServerStatusLoading(true);
    setManagedLlamaServerStatusError(null);
    try {
      const result = await invoke<ManagedLlamaServerStatusPreview>("stop_managed_llama_server");
      setManagedLlamaServerStatusPreview(result);
      setManagedLlamaServerStatusHasRun(true);
    } catch (err) {
      setManagedLlamaServerStatusError(sanitizeBackendError(err));
    } finally {
      setManagedLlamaServerStatusLoading(false);
    }
  }

  function clearManagedLlamaServerChatDiagnosticPreview() {
    setManagedLlamaServerChatDiagnosticPreview(null);
    setManagedLlamaServerChatDiagnosticError(null);
    setManagedLlamaServerChatDiagnosticHasRun(false);
  }

  function buildManagedLlamaServerSmokeDiagnosticRequest(): ManagedLlamaServerSmokeDiagnosticRequest | null {
    const prompt = normalizeOptionalTextInput(managedLlamaServerSmokeDiagnosticPrompt()) ?? "Say READY in one short sentence.";
    const maxOutputTokens = parseOptionalIntegerInput(
      managedLlamaServerSmokeDiagnosticMaxOutputTokens(),
      "Managed llama-server smoke diagnostic max output tokens",
      setManagedLlamaServerSmokeDiagnosticError,
    );
    const timeoutMs = parseOptionalIntegerInput(
      managedLlamaServerSmokeDiagnosticTimeoutMs(),
      "Managed llama-server smoke diagnostic timeout",
      setManagedLlamaServerSmokeDiagnosticError,
    );

    if (maxOutputTokens === undefined || timeoutMs === undefined) {
      return null;
    }

    return {
      allow_smoke_execution: managedLlamaServerSmokeDiagnosticAllowRun(),
      prompt,
      max_output_tokens: maxOutputTokens,
      timeout_ms: timeoutMs,
    };
  }

  async function runManagedLlamaServerSmokeDiagnostic() {
    if (managedLlamaServerSmokeDiagnosticLoading()) {
      return;
    }

    const request = buildManagedLlamaServerSmokeDiagnosticRequest();
    if (!request) {
      setManagedLlamaServerSmokeDiagnosticHasRun(true);
      setManagedLlamaServerSmokeDiagnosticPreview(null);
      return;
    }

    setManagedLlamaServerSmokeDiagnosticLoading(true);
    setManagedLlamaServerSmokeDiagnosticError(null);
    try {
      const result = await invoke<ManagedLlamaServerSmokeDiagnosticPreview>("run_managed_llama_server_smoke_diagnostic", {
        request,
      });
      setManagedLlamaServerSmokeDiagnosticPreview(result);
      setManagedLlamaServerSmokeDiagnosticHasRun(true);
    } catch (err) {
      setManagedLlamaServerSmokeDiagnosticError(sanitizeBackendError(err));
    } finally {
      setManagedLlamaServerSmokeDiagnosticLoading(false);
    }
  }

  function buildManagedLlamaServerChatDiagnosticRequest(): ManagedLlamaServerChatDiagnosticRequest | null {
    const prompt = normalizeOptionalTextInput(managedLlamaServerChatDiagnosticPrompt()) ?? "Say READY in one short sentence.";
    const maxTokens = parseOptionalIntegerInput(
      managedLlamaServerChatDiagnosticMaxTokens(),
      "Managed llama-server chat diagnostic max tokens",
      setManagedLlamaServerChatDiagnosticError,
    );
    const temperature = parseOptionalNumberInput(
      managedLlamaServerChatDiagnosticTemperature(),
      "Managed llama-server chat diagnostic temperature",
      setManagedLlamaServerChatDiagnosticError,
    );
    const timeoutMs = parseOptionalIntegerInput(
      managedLlamaServerChatDiagnosticTimeoutMs(),
      "Managed llama-server chat diagnostic timeout",
      setManagedLlamaServerChatDiagnosticError,
    );

    if (maxTokens === undefined || temperature === undefined || timeoutMs === undefined) {
      return null;
    }

    return {
      allow_chat_diagnostic: managedLlamaServerChatDiagnosticAllowRun(),
      prompt,
      max_tokens: maxTokens,
      temperature,
      timeout_ms: timeoutMs,
    };
  }

  async function runManagedLlamaServerChatDiagnostic() {
    if (managedLlamaServerChatDiagnosticLoading()) {
      return;
    }

    const request = buildManagedLlamaServerChatDiagnosticRequest();
    if (!request) {
      setManagedLlamaServerChatDiagnosticHasRun(true);
      setManagedLlamaServerChatDiagnosticPreview(null);
      return;
    }

    setManagedLlamaServerChatDiagnosticLoading(true);
    setManagedLlamaServerChatDiagnosticError(null);
    try {
      const result = await invoke<ManagedLlamaServerChatDiagnosticPreview>("run_managed_llama_server_chat_diagnostic", {
        request,
      });
      setManagedLlamaServerChatDiagnosticPreview(result);
      setManagedLlamaServerChatDiagnosticHasRun(true);
    } catch (err) {
      setManagedLlamaServerChatDiagnosticError(sanitizeBackendError(err));
    } finally {
      setManagedLlamaServerChatDiagnosticLoading(false);
    }
  }

  function clearLocalRuntimeInvocationPreview() {
    setLocalRuntimeInvocationPreview(null);
    setLocalRuntimeInvocationError(null);
    setLocalRuntimeInvocationValidationError(null);
    setLocalRuntimeInvocationHasRun(false);
    clearScholarChatDraftInferencePreview();
  }

  function clearLocalRuntimeProbePreview() {
    setLocalRuntimeProbeResult(null);
    setLocalRuntimeProbeError(null);
    setLocalRuntimeProbeValidationError(null);
    setLocalRuntimeProbeHasRun(false);
    clearLocalRuntimeCapabilityPreview();
    clearScholarChatDraftInferencePreview();
  }

  function clearLocalRuntimeCapabilityPreview() {
    setLocalRuntimeCapabilityResult(null);
    setLocalRuntimeCapabilityError(null);
    setLocalRuntimeCapabilityHasRun(false);
    clearLocalRuntimeSmokeReadinessPreview();
  }

  function clearLocalRuntimeSmokePreview() {
    setLocalRuntimeSmokeResult(null);
    setLocalRuntimeSmokeError(null);
    setLocalRuntimeSmokeValidationError(null);
    setLocalRuntimeSmokeHasRun(false);
    clearScholarChatRuntimeDiagnosticResultPreview();
  }

  function clearLocalRuntimeSmokeReadinessPreview() {
    setLocalRuntimeSmokeReadinessResult(null);
    setLocalRuntimeSmokeReadinessError(null);
    setLocalRuntimeSmokeReadinessValidationError(null);
    setLocalRuntimeSmokeReadinessHasRun(false);
    clearLocalRuntimeSmokeExecutionPlanPreview();
  }

  function clearLocalRuntimeSmokeExecutionPlanPreview() {
    setLocalRuntimeSmokeExecutionPlanPreview(null);
    setLocalRuntimeSmokeExecutionPlanError(null);
    setLocalRuntimeSmokeExecutionPlanValidationError(null);
    setLocalRuntimeSmokeExecutionPlanHasRun(false);
    clearScholarChatRuntimeDiagnosticBridgePreview();
  }

  function clearLocalRuntimeAdapterContractPreview() {
    setLocalRuntimeAdapterPreview(null);
    setLocalRuntimeAdapterError(null);
    setLocalRuntimeAdapterValidationError(null);
    setLocalRuntimeAdapterHasRun(false);
    clearLocalRuntimeValidationPreview();
  }

  function clearLocalRuntimeValidationPreview() {
    setLocalRuntimeValidationPreview(null);
    setLocalRuntimeValidationPreviewError(null);
    setLocalRuntimeValidationPreviewInputError(null);
    setLocalRuntimeValidationPreviewHasRun(false);
    clearLocalRuntimeProbeReadinessPreview();
  }

  function clearLocalRuntimeProbeReadinessPreview() {
    setLocalRuntimeProbeReadinessPreview(null);
    setLocalRuntimeProbeReadinessPreviewError(null);
    setLocalRuntimeProbeReadinessPreviewInputError(null);
    setLocalRuntimeProbeReadinessPreviewHasRun(false);
  }

  function clearScholarChatAnswerReadinessPreview() {
    setScholarChatAnswerReadinessPreview(null);
    setScholarChatAnswerReadinessError(null);
    setScholarChatAnswerReadinessValidationError(null);
    setScholarChatAnswerReadinessHasRun(false);
    clearScholarChatDraftInferencePreview();
  }

  function clearScholarChatDraftInferencePreview() {
    setScholarChatDraftInferencePreview(null);
    setScholarChatDraftInferenceError(null);
    setScholarChatDraftInferenceValidationError(null);
    setScholarChatDraftInferenceHasRun(false);
    clearScholarChatDraftGroundingInspectionPreview();
  }

  function clearScholarChatDraftGroundingInspectionPreview() {
    setScholarChatDraftGroundingInspectionPreview(null);
    setScholarChatDraftGroundingInspectionError(null);
    setScholarChatDraftGroundingInspectionValidationError(null);
    setScholarChatDraftGroundingInspectionHasRun(false);
    setScholarChatGroundedDraftReadinessPreview(null);
    setScholarChatGroundedDraftReadinessError(null);
    setScholarChatGroundedDraftReadinessValidationError(null);
    setScholarChatGroundedDraftReadinessHasRun(false);
    setScholarChatGroundedAnswerBuildPlanPreview(null);
    setScholarChatGroundedAnswerBuildPlanError(null);
    setScholarChatGroundedAnswerBuildPlanValidationError(null);
    setScholarChatGroundedAnswerBuildPlanHasRun(false);
    setScholarChatGroundedAnswerCandidatePreview(null);
    setScholarChatGroundedAnswerCandidateError(null);
    setScholarChatGroundedAnswerCandidateValidationError(null);
    setScholarChatGroundedAnswerCandidateHasRun(false);
    setScholarChatGroundedAnswerWriteEligibilityPreview(null);
    setScholarChatGroundedAnswerWriteEligibilityError(null);
    setScholarChatGroundedAnswerWriteEligibilityValidationError(null);
    setScholarChatGroundedAnswerWriteEligibilityHasRun(false);
    clearScholarChatRuntimeDiagnosticBridgePreview();
  }

  function clearScholarChatGroundedAnswerBuildIntentPreview() {
    setScholarChatGroundedAnswerBuildIntentPreview(null);
    setScholarChatGroundedAnswerBuildIntentError(null);
    setScholarChatGroundedAnswerBuildIntentValidationError(null);
    setScholarChatGroundedAnswerBuildIntentHasRun(false);
    clearScholarChatGroundedAnswerBuildRequestPreview();
  }

  function resetScholarChatConversationPreviewState() {
    setScholarChatValidationError(null);
    setScholarChatError(null);
    setScholarChatPreview(null);
    setScholarChatExecutionGatePreview(null);
    clearScholarChatPromptPackPreview();
    clearScholarChatDraftInferencePreview();
    clearScholarChatGroundedAnswerBuildIntentPreview();
  }

  function applyScholarChatPromptSuggestion(prompt: string) {
    setScholarChatPrompt(prompt);
    resetScholarChatConversationPreviewState();
  }

  function nextScholarChatTranscriptId() {
    return scholarChatTranscript().reduce((max, message) => Math.max(max, message.id), 0) + 1;
  }

  function updateScholarChatTranscriptMessage(message: ScholarChatTranscriptMessage) {
    setScholarChatTranscript((current) => {
      const index = current.findIndex((item) => item.kind === message.kind && item.prompt === message.prompt);
      if (index >= 0) {
        const next = [...current];
        next[index] = message;
        return next;
      }
      return [...current, message];
    });
  }

  function ensureScholarChatUserTranscriptMessage(prompt: string) {
    const normalizedPrompt = prompt.trim();
    if (!normalizedPrompt) {
      return;
    }
    const lastUserMessage = [...scholarChatTranscript()].reverse().find((item) => item.role === "user");
    if (lastUserMessage && lastUserMessage.prompt === normalizedPrompt) {
      return;
    }
    setScholarChatTranscript((current) => [
      ...current,
      {
        id: nextScholarChatTranscriptId(),
        role: "user",
        kind: "prompt",
        prompt: normalizedPrompt,
        title: "You",
        content: normalizedPrompt,
        created_at: Date.now(),
      },
    ]);
  }

  function recordScholarChatWorkflowPreview(prompt: string, result: ScholarChatAgenticWorkflowPlanPreview) {
    ensureScholarChatUserTranscriptMessage(prompt);
    updateScholarChatTranscriptMessage({
      id: nextScholarChatTranscriptId(),
      role: "assistant",
      kind: "workflow_preview",
      prompt,
      title: "AEGIS",
      content: result.summary,
      created_at: Date.now(),
      workflow_preview: result,
    });
  }

  function recordScholarChatExecutionGatePreview(prompt: string, result: ScholarChatAgenticWorkflowExecutionGatePreview) {
    ensureScholarChatUserTranscriptMessage(prompt);
    updateScholarChatTranscriptMessage({
      id: nextScholarChatTranscriptId(),
      role: "assistant",
      kind: "execution_gate",
      prompt,
      title: "AEGIS",
      content: result.blocked_reason || "The next safe step is ready to review.",
      created_at: Date.now(),
      execution_gate_preview: result,
    });
  }

  function clearScholarChatGroundedAnswerBuildRequestPreview() {
    setScholarChatGroundedAnswerBuildRequestPreview(null);
    setScholarChatGroundedAnswerBuildRequestError(null);
    setScholarChatGroundedAnswerBuildRequestValidationError(null);
    setScholarChatGroundedAnswerBuildRequestHasRun(false);
    clearScholarChatGroundedAnswerBuildPreflightPreview();
  }

  function clearScholarChatGroundedAnswerBuildPreflightPreview() {
    setScholarChatGroundedAnswerBuildPreflightPreview(null);
    setScholarChatGroundedAnswerBuildPreflightError(null);
    setScholarChatGroundedAnswerBuildPreflightValidationError(null);
    setScholarChatGroundedAnswerBuildPreflightHasRun(false);
    clearScholarChatGroundedAnswerExecutionReadinessPreview();
  }

  function clearScholarChatGroundedAnswerExecutionReadinessPreview() {
    setScholarChatGroundedAnswerExecutionReadinessPreview(null);
    setScholarChatGroundedAnswerExecutionReadinessError(null);
    setScholarChatGroundedAnswerExecutionReadinessValidationError(null);
    setScholarChatGroundedAnswerExecutionReadinessHasRun(false);
    clearScholarChatGroundedAnswerExecutionPlanPreview();
  }

  function clearScholarChatGroundedAnswerExecutionPlanPreview() {
    setScholarChatGroundedAnswerExecutionPlanPreview(null);
    setScholarChatGroundedAnswerExecutionPlanError(null);
    setScholarChatGroundedAnswerExecutionPlanValidationError(null);
    setScholarChatGroundedAnswerExecutionPlanHasRun(false);
  }

  function clearScholarChatRuntimeDiagnosticBridgePreview() {
    setScholarChatRuntimeDiagnosticBridgePreview(null);
    setScholarChatRuntimeDiagnosticBridgeError(null);
    setScholarChatRuntimeDiagnosticBridgeValidationError(null);
    setScholarChatRuntimeDiagnosticBridgeHasRun(false);
    clearScholarChatRuntimeDiagnosticResultPreview();
  }

  function clearScholarChatRuntimeDiagnosticResultPreview() {
    setScholarChatRuntimeDiagnosticResultPreview(null);
    setScholarChatRuntimeDiagnosticResultError(null);
    setScholarChatRuntimeDiagnosticResultValidationError(null);
    setScholarChatRuntimeDiagnosticResultHasRun(false);
  }

  function normalizeOptionalTextInput(value: string) {
    const trimmed = value.trim();
    return trimmed ? trimmed : null;
  }

  function buildScholarChatDraftGroundingInspectionRequest(
    trimmedPrompt: string,
  ): ScholarChatDraftGroundingInspectionRequest {
    const draftText = scholarChatDraftGroundingInspectionDraftText().trim();
    return {
      scholar_chat_request: {
        prompt: trimmedPrompt,
        mode: scholarChatMode(),
        grounding_policy: scholarChatGroundingPolicy(),
        selected_source_ids: selectedScholarChatSourceIds(),
      },
      draft_text: draftText ? draftText : null,
      max_items: 8,
    };
  }

  function buildScholarChatGroundedAnswerBuildIntentRequest(
    trimmedPrompt: string,
  ): ScholarChatGroundedAnswerBuildIntentRequest {
    return {
      grounding_request: buildScholarChatDraftGroundingInspectionRequest(trimmedPrompt),
      answer_draft_id: normalizeOptionalTextInput(scholarChatGroundedAnswerBuildIntentAnswerDraftId()),
      explicit_user_intent: scholarChatGroundedAnswerBuildIntentExplicitUserIntent(),
    };
  }

  function buildScholarChatGroundedAnswerBuildRequestRequest(
    trimmedPrompt: string,
  ): ScholarChatGroundedAnswerBuildRequestPreviewRequest {
    return {
      build_intent_request: buildScholarChatGroundedAnswerBuildIntentRequest(trimmedPrompt),
    };
  }

  function buildScholarChatGroundedAnswerBuildPreflightRequest(
    trimmedPrompt: string,
  ): ScholarChatGroundedAnswerBuildPreflightPreviewRequest {
    return {
      build_request_preview_request: buildScholarChatGroundedAnswerBuildRequestRequest(trimmedPrompt),
    };
  }

  function parseOptionalIntegerInput(
    value: string,
    label: string,
    setValidationError = setLocalRuntimeValidationError,
  ) {
    const trimmed = value.trim();
    if (!trimmed) {
      return null;
    }
    const parsed = Number(trimmed);
    if (!Number.isInteger(parsed)) {
      setValidationError(`${label} must be a whole number.`);
      return undefined;
    }
    return parsed;
  }

  function parseOptionalNumberInput(
    value: string,
    label: string,
    setValidationError = setLocalRuntimeValidationError,
  ) {
    const trimmed = value.trim();
    if (!trimmed) {
      return null;
    }
    const parsed = Number(trimmed);
    if (!Number.isFinite(parsed)) {
      setValidationError(`${label} must be a number.`);
      return undefined;
    }
    return parsed;
  }

  function parseOptionalCommaSeparatedListInput(value: string) {
    const trimmed = value.trim();
    if (!trimmed) {
      return null;
    }
    return trimmed
      .split(",")
      .map((item) => item.trim())
      .filter(Boolean);
  }

  function selectedFinalAnswerDetail() {
    const selectedSourceId = selectedAnswerArtifactSourceId();
    const selectedFinalAnswerId = finalAnswerId().trim();
    const currentFinalAnswer = finalAnswer();
    if (!selectedSourceId || !selectedFinalAnswerId || !currentFinalAnswer) {
      return null;
    }
    if (currentFinalAnswer.source_id !== selectedSourceId || currentFinalAnswer.final_answer_id !== selectedFinalAnswerId) {
      return null;
    }
    return currentFinalAnswer;
  }

  function safeFileNameFromPathInput(value: string) {
    const trimmed = value.trim();
    if (!trimmed) {
      return null;
    }
    const normalized = trimmed.replace(/\\/g, "/");
    const fileName = normalized.split("/").filter(Boolean).pop();
    return fileName || null;
  }

  function localRuntimeProbeExecutableSummary() {
    const executablePath = localRuntimeExecutablePath().trim();
    if (!executablePath) {
      return "No executable path configured yet.";
    }
    const fileName = safeFileNameFromPathInput(executablePath);
    return fileName ? `Configured executable: ${fileName}` : "Configured executable: not readable yet.";
  }

  function answerArtifactSourceTotals() {
    return artifactSources().reduce(
      (totals, item) => ({
        source_count: totals.source_count + 1,
        draft_count: totals.draft_count + item.draft_count,
        grounded_answer_count: totals.grounded_answer_count + item.grounded_answer_count,
        final_answer_count: totals.final_answer_count + item.final_answer_count,
      }),
      {
        source_count: 0,
        draft_count: 0,
        grounded_answer_count: 0,
        final_answer_count: 0,
      },
    );
  }

  function answerArtifactIssueTotals() {
    return artifactIssues().reduce(
      (totals, item) => ({
        issue_count: totals.issue_count + 1,
        source_count: item.source_id && !totals.source_ids.includes(item.source_id) ? totals.source_count + 1 : totals.source_count,
        malformed_final_answer_count: totals.malformed_final_answer_count + (item.issue_kind === "malformed_final_answer" ? 1 : 0),
        unsupported_statement_count: totals.unsupported_statement_count + (item.issue_kind === "unsupported_statement" ? 1 : 0),
        needs_evidence_statement_count: totals.needs_evidence_statement_count + (item.issue_kind === "needs_evidence_statement" ? 1 : 0),
        source_ids: item.source_id && !totals.source_ids.includes(item.source_id) ? [...totals.source_ids, item.source_id] : totals.source_ids,
      }),
      {
        issue_count: 0,
        source_count: 0,
        malformed_final_answer_count: 0,
        unsupported_statement_count: 0,
        needs_evidence_statement_count: 0,
        source_ids: [] as string[],
      },
    );
  }

  async function refreshDiagnostics() {
    const sourceIdForRetrieval = sourceId().trim() || retrievalIndex()?.source_id?.trim() || "";
    const sourceIdForEvidence = selectedEvidencePackSourceId();
    await Promise.all([
      loadScholarChatSourceContext(true),
      sourceIdForRetrieval ? loadRetrievalIndex(true, sourceIdForRetrieval) : Promise.resolve(),
      loadArtifactSources(true),
      loadArtifactHealth(true),
      loadArtifactIssues(true),
      sourceIdForEvidence ? loadEvidencePacksBySourceId(sourceIdForEvidence) : Promise.resolve(),
    ]);
  }

  async function selectArtifactSourceId(source_id: string) {
    setSourceId(source_id);
    setFinalAnswerId("");
    setFinalAnswer(null);
    setFinalAnswerError(null);
    clearScholarChatDraftInferencePreview();
    await loadArtifactOverviewBySourceId(source_id);
  }

  async function loadArtifactOverviewBySourceId(trimmedSourceId: string) {
    if (!trimmedSourceId) {
      setArtifactOverviewError("Source ID is required to load the artifact overview.");
      return;
    }
    if (artifactOverviewLoading()) {
      return;
    }
    setArtifactOverviewLoading(true);
    setArtifactOverviewError(null);
    try {
      const result = await invoke<AnswerArtifactOverview>("get_answer_artifact_overview", {
        root: ".",
        source_id: trimmedSourceId,
      });
      setArtifactOverview(result);
    } catch (err) {
      setArtifactOverview(null);
      setArtifactOverviewError(sanitizeBackendError(err));
    } finally {
      setArtifactOverviewLoading(false);
    }
  }

  async function selectFinalAnswer(finalAnswerMetadata: FinalAnswerMetadata) {
    setFinalAnswerId(finalAnswerMetadata.final_answer_id);
    await loadFinalAnswerByIds(sourceId().trim(), finalAnswerMetadata.final_answer_id);
  }

  async function previewScholarChatAgenticWorkflowPlan() {
    const trimmedPrompt = scholarChatPrompt().trim();
    if (!trimmedPrompt) {
      setScholarChatPreview(null);
      setScholarChatError(null);
      setScholarChatValidationError("Prompt is required to preview a Scholar Chat workflow plan.");
      return;
    }
    if (scholarChatLoading()) {
      return;
    }
    setScholarChatLoading(true);
    setScholarChatError(null);
    setScholarChatValidationError(null);
    setScholarChatPreview(null);
    try {
      const result = await invoke<ScholarChatAgenticWorkflowPlanPreview>("preview_scholar_chat_agentic_workflow_plan", {
        root: ".",
        request: {
          prompt: trimmedPrompt,
          mode: scholarChatMode(),
          grounding_policy: scholarChatGroundingPolicy(),
          selected_source_ids: selectedScholarChatSourceIds(),
        },
      });
      setScholarChatPreview(result);
      recordScholarChatWorkflowPreview(trimmedPrompt, result);
    } catch (err) {
      setScholarChatError(sanitizeBackendError(err));
    } finally {
      setScholarChatLoading(false);
    }
  }

  async function previewScholarChatAgenticWorkflowExecutionGate() {
    const trimmedPrompt = scholarChatPrompt().trim();
    if (!trimmedPrompt) {
      setScholarChatExecutionGatePreview(null);
      setScholarChatExecutionGateError(null);
      setScholarChatExecutionGateValidationError("Prompt is required to preview Scholar Chat execution readiness.");
      return;
    }
    if (scholarChatExecutionGateLoading()) {
      return;
    }
    setScholarChatExecutionGateLoading(true);
    setScholarChatExecutionGateError(null);
    setScholarChatExecutionGateValidationError(null);
    setScholarChatExecutionGatePreview(null);
    try {
      const result = await invoke<ScholarChatAgenticWorkflowExecutionGatePreview>("preview_scholar_chat_agentic_workflow_execution_gate", {
        root: ".",
        request: {
          scholar_chat_request: {
            prompt: trimmedPrompt,
            mode: scholarChatMode(),
            grounding_policy: scholarChatGroundingPolicy(),
            selected_source_ids: selectedScholarChatSourceIds(),
          },
          user_consent_present: false,
        },
      });
      setScholarChatExecutionGatePreview(result);
      recordScholarChatExecutionGatePreview(trimmedPrompt, result);
    } catch (err) {
      setScholarChatExecutionGateError(sanitizeBackendError(err));
    } finally {
      setScholarChatExecutionGateLoading(false);
    }
  }

  async function previewScholarChatRetrieval() {
    const trimmedPrompt = scholarChatPrompt().trim();
    if (!trimmedPrompt) {
      setScholarChatRetrievalPreview(null);
      setScholarChatRetrievalError(null);
      setScholarChatValidationError("Prompt is required to preview Scholar Chat retrieval candidates.");
      return;
    }
    if (scholarChatRetrievalLoading()) {
      return;
    }
    setScholarChatRetrievalHasRun(true);
    setScholarChatRetrievalLoading(true);
    setScholarChatRetrievalError(null);
    setScholarChatValidationError(null);
    setScholarChatRetrievalPreview(null);
    try {
      const result = await invoke<ScholarChatRetrievalPreviewResponse>("preview_scholar_chat_retrieval", {
        root: ".",
        request: {
          prompt: trimmedPrompt,
          mode: scholarChatMode(),
          grounding_policy: scholarChatGroundingPolicy(),
          selected_source_ids: selectedScholarChatSourceIds(),
        },
      });
      setScholarChatRetrievalPreview(result);
    } catch (err) {
      setScholarChatRetrievalError(sanitizeBackendError(err));
    } finally {
      setScholarChatRetrievalLoading(false);
    }
  }

  async function previewScholarChatEvidencePlan() {
    const trimmedPrompt = scholarChatPrompt().trim();
    if (!trimmedPrompt) {
      setScholarChatEvidencePlanPreview(null);
      setScholarChatEvidencePlanError(null);
      setScholarChatValidationError("Prompt is required to preview a Scholar Chat evidence plan.");
      return;
    }
    if (scholarChatEvidencePlanLoading()) {
      return;
    }
    setScholarChatEvidencePlanHasRun(true);
    setScholarChatEvidencePlanLoading(true);
    setScholarChatEvidencePlanError(null);
    setScholarChatValidationError(null);
    setScholarChatEvidencePlanPreview(null);
    try {
      const result = await invoke<ScholarChatEvidencePlanResponse>("preview_scholar_chat_evidence_plan", {
        root: ".",
        request: {
          prompt: trimmedPrompt,
          mode: scholarChatMode(),
          grounding_policy: scholarChatGroundingPolicy(),
          selected_source_ids: selectedScholarChatSourceIds(),
        },
      });
      setScholarChatEvidencePlanPreview(result);
    } catch (err) {
      setScholarChatEvidencePlanError(sanitizeBackendError(err));
    } finally {
      setScholarChatEvidencePlanLoading(false);
    }
  }

  async function previewScholarChatPromptPack() {
    const trimmedPrompt = scholarChatPrompt().trim();
    if (!trimmedPrompt) {
      setScholarChatPromptPackPreview(null);
      setScholarChatPromptPackError("Prompt is required to preview a Scholar Chat prompt pack.");
      setScholarChatPromptPackHasRun(true);
      return;
    }
    if (scholarChatPromptPackLoading()) {
      return;
    }
    setScholarChatPromptPackHasRun(true);
    setScholarChatPromptPackLoading(true);
    setScholarChatPromptPackError(null);
    setScholarChatPromptPackPreview(null);
    try {
      const result = await invoke<ScholarChatPromptPackPreviewResponse>("preview_scholar_chat_prompt_pack", {
        root: ".",
        request: {
          prompt: trimmedPrompt,
          mode: scholarChatMode(),
          grounding_policy: scholarChatGroundingPolicy(),
          selected_source_ids: selectedScholarChatSourceIds(),
        },
      });
      setScholarChatPromptPackPreview(result);
    } catch (err) {
      setScholarChatPromptPackError(sanitizeBackendError(err));
    } finally {
      setScholarChatPromptPackLoading(false);
    }
  }

  function buildScholarChatScientificMetadataProviderRequestPreviewRequest(
    trimmedPrompt: string,
  ): ScholarChatScientificMetadataProviderRequestPreviewRequest {
    return {
      query_plan_preview_request: {
        query: trimmedPrompt,
        mode: "scientific_paper",
        context_tags: ["science", "metadata"],
        preferred_metadata_sources: ["openalex"],
        provider_override: ["openalex"],
        query_goal: "OpenAlex metadata preview",
        require_open_access: false,
        require_doi: false,
        year_from: 2018,
        year_to: 2026,
        include_disabled_providers: false,
        include_institutional_providers: false,
        include_rate_limit_notes: true,
        include_attribution_requirements: true,
        include_query_templates: true,
        include_filter_plan: true,
        include_result_field_plan: true,
        execution_requested: false,
        allow_network: false,
        allow_provider_terms_unreviewed: false,
        allow_metadata_record_write: false,
      },
      include_request_templates: true,
      include_header_plan: true,
      include_param_plan: true,
      include_body_plan: true,
    };
  }

  async function previewScholarChatScientificMetadataProviderRequest() {
    const trimmedPrompt = scholarChatPrompt().trim();
    if (!trimmedPrompt) {
      setScholarChatScientificMetadataProviderRequestPreview(null);
      setScholarChatScientificMetadataProviderRequestError(null);
      setScholarChatScientificMetadataProviderRequestValidationError("Prompt is required to preview the OpenAlex metadata provider request.");
      setScholarChatScientificMetadataProviderRequestHasRun(true);
      return;
    }
    if (scholarChatScientificMetadataProviderRequestLoading()) {
      return;
    }

    setScholarChatScientificMetadataProviderRequestHasRun(true);
    setScholarChatScientificMetadataProviderRequestLoading(true);
    setScholarChatScientificMetadataProviderRequestError(null);
    setScholarChatScientificMetadataProviderRequestValidationError(null);
    setScholarChatScientificMetadataProviderRequestPreview(null);
    try {
      // Phase 114.1 keeps only the read-only provider request preview wired here.
      // OpenAlex execution and cache/write gate remain intentionally unwired from this panel.
      const result = await invoke<ScholarChatScientificMetadataProviderRequestPreview>("preview_scholar_chat_scientific_metadata_provider_request", {
        root: ".",
        request: buildScholarChatScientificMetadataProviderRequestPreviewRequest(trimmedPrompt),
      });
      setScholarChatScientificMetadataProviderRequestPreview(result);
    } catch (err) {
      setScholarChatScientificMetadataProviderRequestError(sanitizeBackendError(err));
    } finally {
      setScholarChatScientificMetadataProviderRequestLoading(false);
    }
  }

  async function previewScholarChatAnswerReadiness() {
    const trimmedPrompt = scholarChatPrompt().trim();
    if (!trimmedPrompt) {
      setScholarChatAnswerReadinessPreview(null);
      setScholarChatAnswerReadinessError(null);
      setScholarChatAnswerReadinessValidationError("Prompt is required to preview answer readiness.");
      return;
    }
    if (scholarChatAnswerReadinessLoading()) {
      return;
    }
    const contextWindow = parseOptionalIntegerInput(localRuntimeContextWindow(), "Context window");
    const gpuLayers = parseOptionalIntegerInput(localRuntimeGpuLayers(), "GPU layers");
    const temperature = parseOptionalNumberInput(localRuntimeTemperature(), "Temperature");
    if (contextWindow === undefined || gpuLayers === undefined || temperature === undefined) {
      setScholarChatAnswerReadinessPreview(null);
      setScholarChatAnswerReadinessValidationError("Local runtime inputs must be valid before previewing answer readiness.");
      return;
    }
    setScholarChatAnswerReadinessHasRun(true);
    setScholarChatAnswerReadinessLoading(true);
    setScholarChatAnswerReadinessError(null);
    setScholarChatAnswerReadinessValidationError(null);
    setScholarChatAnswerReadinessPreview(null);
    try {
      const result = await invoke<ScholarChatAnswerReadinessPreview>("preview_scholar_chat_answer_readiness", {
        root: ".",
        request: {
          scholar_chat_request: {
            prompt: trimmedPrompt,
            mode: scholarChatMode(),
            grounding_policy: scholarChatGroundingPolicy(),
            selected_source_ids: selectedScholarChatSourceIds(),
          },
          runtime_config: {
            runtime_kind: localRuntimeKind(),
            model_path: normalizeOptionalTextInput(localRuntimeModelPath()),
            executable_path: normalizeOptionalTextInput(localRuntimeExecutablePath()),
            context_window: contextWindow,
            gpu_layers: gpuLayers,
            temperature: temperature,
          },
          allow_model_execution: scholarChatAnswerReadinessAllowModelExecution(),
        } satisfies ScholarChatAnswerReadinessRequest,
      });
      setScholarChatAnswerReadinessPreview(result);
    } catch (err) {
      setScholarChatAnswerReadinessError(sanitizeBackendError(err));
    } finally {
      setScholarChatAnswerReadinessLoading(false);
    }
  }

  async function previewScholarChatDraftInference() {
    const trimmedPrompt = scholarChatPrompt().trim();
    if (!trimmedPrompt) {
      setScholarChatDraftInferencePreview(null);
      setScholarChatDraftInferenceError(null);
      setScholarChatDraftInferenceValidationError("Prompt is required to preview Scholar Chat draft inference.");
      return;
    }
    if (scholarChatDraftInferenceLoading()) {
      return;
    }
    const contextWindow = parseOptionalIntegerInput(localRuntimeContextWindow(), "Context window", setScholarChatDraftInferenceValidationError);
    const gpuLayers = parseOptionalIntegerInput(localRuntimeGpuLayers(), "GPU layers", setScholarChatDraftInferenceValidationError);
    const temperature = parseOptionalNumberInput(localRuntimeTemperature(), "Temperature", setScholarChatDraftInferenceValidationError);
    const timeoutMs = parseOptionalIntegerInput(localRuntimeProbeTimeoutMs(), "Draft inference timeout", setScholarChatDraftInferenceValidationError);
    const maxOutputTokens = parseOptionalIntegerInput(localRuntimeInvocationMaxOutputTokens(), "Max output tokens", setScholarChatDraftInferenceValidationError);
    if (
      contextWindow === undefined ||
      gpuLayers === undefined ||
      temperature === undefined ||
      timeoutMs === undefined ||
      maxOutputTokens === undefined
    ) {
      setScholarChatDraftInferencePreview(null);
      return;
    }

    setScholarChatDraftInferenceHasRun(true);
    setScholarChatDraftInferenceLoading(true);
    setScholarChatDraftInferenceError(null);
    setScholarChatDraftInferenceValidationError(null);
    setScholarChatDraftInferencePreview(null);
    clearScholarChatDraftGroundingInspectionPreview();
    try {
      const result = await invoke<ScholarChatDraftInferencePreview>("preview_scholar_chat_draft_inference", {
        root: ".",
        request: {
          scholar_chat_request: {
            prompt: trimmedPrompt,
            mode: scholarChatMode(),
            grounding_policy: scholarChatGroundingPolicy(),
            selected_source_ids: selectedScholarChatSourceIds(),
          },
          runtime_config: {
            runtime_kind: localRuntimeKind(),
            model_path: normalizeOptionalTextInput(localRuntimeModelPath()),
            executable_path: normalizeOptionalTextInput(localRuntimeExecutablePath()),
            context_window: contextWindow,
            gpu_layers: gpuLayers,
            temperature: temperature,
          },
          allow_model_execution: scholarChatAnswerReadinessAllowModelExecution(),
          timeout_ms: timeoutMs,
          max_output_tokens: maxOutputTokens,
        } satisfies ScholarChatDraftInferenceRequest,
      });
      setScholarChatDraftInferencePreview(result);
    } catch (err) {
      setScholarChatDraftInferenceError(sanitizeBackendError(err));
    } finally {
      setScholarChatDraftInferenceLoading(false);
    }
  }

  async function previewScholarChatDraftGroundingInspection() {
    const trimmedPrompt = scholarChatPrompt().trim();
    if (!trimmedPrompt) {
      setScholarChatDraftGroundingInspectionPreview(null);
      setScholarChatDraftGroundingInspectionError(null);
      setScholarChatDraftGroundingInspectionValidationError("Prompt is required to preview draft grounding inspection.");
      return;
    }
    if (scholarChatDraftGroundingInspectionLoading()) {
      return;
    }

    setScholarChatDraftGroundingInspectionHasRun(true);
    setScholarChatDraftGroundingInspectionLoading(true);
    setScholarChatDraftGroundingInspectionError(null);
    setScholarChatDraftGroundingInspectionValidationError(null);
    setScholarChatDraftGroundingInspectionPreview(null);
    try {
      const result = await invoke<ScholarChatDraftGroundingInspectionPreview>("preview_scholar_chat_draft_grounding_inspection", {
        root: ".",
        request: buildScholarChatDraftGroundingInspectionRequest(trimmedPrompt),
      });
      setScholarChatDraftGroundingInspectionPreview(result);
    } catch (err) {
      setScholarChatDraftGroundingInspectionError(sanitizeBackendError(err));
    } finally {
      setScholarChatDraftGroundingInspectionLoading(false);
    }
  }

  async function previewScholarChatGroundedDraftReadiness() {
    const trimmedPrompt = scholarChatPrompt().trim();
    if (!trimmedPrompt) {
      setScholarChatGroundedDraftReadinessPreview(null);
      setScholarChatGroundedDraftReadinessError(null);
      setScholarChatGroundedDraftReadinessValidationError("Prompt is required to preview grounded draft readiness.");
      return;
    }
    if (scholarChatGroundedDraftReadinessLoading()) {
      return;
    }

    setScholarChatGroundedDraftReadinessHasRun(true);
    setScholarChatGroundedDraftReadinessLoading(true);
    setScholarChatGroundedDraftReadinessError(null);
    setScholarChatGroundedDraftReadinessValidationError(null);
    setScholarChatGroundedDraftReadinessPreview(null);
    try {
      const result = await invoke<ScholarChatGroundedDraftReadinessPreview>("preview_scholar_chat_grounded_draft_readiness", {
        root: ".",
        request: buildScholarChatDraftGroundingInspectionRequest(trimmedPrompt),
      });
      setScholarChatGroundedDraftReadinessPreview(result);
    } catch (err) {
      setScholarChatGroundedDraftReadinessError(sanitizeBackendError(err));
    } finally {
      setScholarChatGroundedDraftReadinessLoading(false);
    }
  }

  async function previewScholarChatGroundedAnswerBuildPlan() {
    const trimmedPrompt = scholarChatPrompt().trim();
    if (!trimmedPrompt) {
      setScholarChatGroundedAnswerBuildPlanPreview(null);
      setScholarChatGroundedAnswerBuildPlanError(null);
      setScholarChatGroundedAnswerBuildPlanValidationError("Prompt is required to preview grounded answer build plan.");
      return;
    }
    if (scholarChatGroundedAnswerBuildPlanLoading()) {
      return;
    }

    setScholarChatGroundedAnswerBuildPlanHasRun(true);
    setScholarChatGroundedAnswerBuildPlanLoading(true);
    setScholarChatGroundedAnswerBuildPlanError(null);
    setScholarChatGroundedAnswerBuildPlanValidationError(null);
    setScholarChatGroundedAnswerBuildPlanPreview(null);
    try {
      const result = await invoke<ScholarChatGroundedAnswerBuildPlanPreview>("preview_scholar_chat_grounded_answer_build_plan", {
        root: ".",
        request: buildScholarChatDraftGroundingInspectionRequest(trimmedPrompt),
      });
      setScholarChatGroundedAnswerBuildPlanPreview(result);
    } catch (err) {
      setScholarChatGroundedAnswerBuildPlanError(sanitizeBackendError(err));
    } finally {
      setScholarChatGroundedAnswerBuildPlanLoading(false);
    }
  }

  async function previewScholarChatGroundedAnswerCandidate() {
    const trimmedPrompt = scholarChatPrompt().trim();
    if (!trimmedPrompt) {
      setScholarChatGroundedAnswerCandidatePreview(null);
      setScholarChatGroundedAnswerCandidateError(null);
      setScholarChatGroundedAnswerCandidateValidationError("Prompt is required to preview grounded answer candidate.");
      return;
    }
    if (scholarChatGroundedAnswerCandidateLoading()) {
      return;
    }

    setScholarChatGroundedAnswerCandidateHasRun(true);
    setScholarChatGroundedAnswerCandidateLoading(true);
    setScholarChatGroundedAnswerCandidateError(null);
    setScholarChatGroundedAnswerCandidateValidationError(null);
    setScholarChatGroundedAnswerCandidatePreview(null);
    try {
      const result = await invoke<ScholarChatGroundedAnswerCandidatePreview>("preview_scholar_chat_grounded_answer_candidate", {
        root: ".",
        request: buildScholarChatDraftGroundingInspectionRequest(trimmedPrompt),
      });
      setScholarChatGroundedAnswerCandidatePreview(result);
    } catch (err) {
      setScholarChatGroundedAnswerCandidateError(sanitizeBackendError(err));
    } finally {
      setScholarChatGroundedAnswerCandidateLoading(false);
    }
  }

  async function previewScholarChatGroundedAnswerWriteEligibility() {
    const trimmedPrompt = scholarChatPrompt().trim();
    if (!trimmedPrompt) {
      setScholarChatGroundedAnswerWriteEligibilityPreview(null);
      setScholarChatGroundedAnswerWriteEligibilityError(null);
      setScholarChatGroundedAnswerWriteEligibilityValidationError("Prompt is required to preview grounded answer write eligibility.");
      return;
    }
    if (scholarChatGroundedAnswerWriteEligibilityLoading()) {
      return;
    }

    setScholarChatGroundedAnswerWriteEligibilityHasRun(true);
    setScholarChatGroundedAnswerWriteEligibilityLoading(true);
    setScholarChatGroundedAnswerWriteEligibilityError(null);
    setScholarChatGroundedAnswerWriteEligibilityValidationError(null);
    setScholarChatGroundedAnswerWriteEligibilityPreview(null);
    try {
      const result = await invoke<ScholarChatGroundedAnswerWriteEligibilityPreview>("preview_scholar_chat_grounded_answer_write_eligibility", {
        root: ".",
        request: buildScholarChatDraftGroundingInspectionRequest(trimmedPrompt),
      });
      setScholarChatGroundedAnswerWriteEligibilityPreview(result);
    } catch (err) {
      setScholarChatGroundedAnswerWriteEligibilityError(sanitizeBackendError(err));
    } finally {
      setScholarChatGroundedAnswerWriteEligibilityLoading(false);
    }
  }

  async function previewScholarChatGroundedAnswerBuildIntent() {
    const trimmedPrompt = scholarChatPrompt().trim();
    if (!trimmedPrompt) {
      setScholarChatGroundedAnswerBuildIntentPreview(null);
      setScholarChatGroundedAnswerBuildIntentError(null);
      setScholarChatGroundedAnswerBuildIntentValidationError("Prompt is required to preview grounded answer build intent.");
      return;
    }
    if (scholarChatGroundedAnswerBuildIntentLoading()) {
      return;
    }

    setScholarChatGroundedAnswerBuildIntentHasRun(true);
    setScholarChatGroundedAnswerBuildIntentLoading(true);
    setScholarChatGroundedAnswerBuildIntentError(null);
    setScholarChatGroundedAnswerBuildIntentValidationError(null);
    setScholarChatGroundedAnswerBuildIntentPreview(null);
    try {
      const result = await invoke<ScholarChatGroundedAnswerBuildIntentPreview>("preview_scholar_chat_grounded_answer_build_intent", {
        root: ".",
        request: buildScholarChatGroundedAnswerBuildIntentRequest(trimmedPrompt),
      });
      setScholarChatGroundedAnswerBuildIntentPreview(result);
    } catch (err) {
      setScholarChatGroundedAnswerBuildIntentError(sanitizeBackendError(err));
    } finally {
      setScholarChatGroundedAnswerBuildIntentLoading(false);
    }
  }

  async function previewScholarChatGroundedAnswerBuildRequest() {
    const trimmedPrompt = scholarChatPrompt().trim();
    if (!trimmedPrompt) {
      setScholarChatGroundedAnswerBuildRequestPreview(null);
      setScholarChatGroundedAnswerBuildRequestError(null);
      setScholarChatGroundedAnswerBuildRequestValidationError("Prompt is required to preview grounded answer build request.");
      return;
    }
    if (scholarChatGroundedAnswerBuildRequestLoading()) {
      return;
    }

    setScholarChatGroundedAnswerBuildRequestHasRun(true);
    setScholarChatGroundedAnswerBuildRequestLoading(true);
    setScholarChatGroundedAnswerBuildRequestError(null);
    setScholarChatGroundedAnswerBuildRequestValidationError(null);
    setScholarChatGroundedAnswerBuildRequestPreview(null);
    clearScholarChatGroundedAnswerBuildPreflightPreview();
    try {
      const result = await invoke<ScholarChatGroundedAnswerBuildRequestPreview>("preview_scholar_chat_grounded_answer_build_request", {
        root: ".",
        request: buildScholarChatGroundedAnswerBuildRequestRequest(trimmedPrompt),
      });
      setScholarChatGroundedAnswerBuildRequestPreview(result);
    } catch (err) {
      setScholarChatGroundedAnswerBuildRequestError(sanitizeBackendError(err));
    } finally {
      setScholarChatGroundedAnswerBuildRequestLoading(false);
    }
  }

  async function previewScholarChatGroundedAnswerBuildPreflight() {
    const trimmedPrompt = scholarChatPrompt().trim();
    if (!trimmedPrompt) {
      setScholarChatGroundedAnswerBuildPreflightPreview(null);
      setScholarChatGroundedAnswerBuildPreflightError(null);
      setScholarChatGroundedAnswerBuildPreflightValidationError("Prompt is required to preview grounded answer build preflight.");
      return;
    }
    if (scholarChatGroundedAnswerBuildPreflightLoading()) {
      return;
    }

    setScholarChatGroundedAnswerBuildPreflightHasRun(true);
    setScholarChatGroundedAnswerBuildPreflightLoading(true);
    setScholarChatGroundedAnswerBuildPreflightError(null);
    setScholarChatGroundedAnswerBuildPreflightValidationError(null);
    setScholarChatGroundedAnswerBuildPreflightPreview(null);
    try {
      const result = await invoke<ScholarChatGroundedAnswerBuildPreflightPreview>("preview_scholar_chat_grounded_answer_build_preflight", {
        root: ".",
        request: buildScholarChatGroundedAnswerBuildPreflightRequest(trimmedPrompt),
      });
      setScholarChatGroundedAnswerBuildPreflightPreview(result);
    } catch (err) {
      setScholarChatGroundedAnswerBuildPreflightError(sanitizeBackendError(err));
    } finally {
      setScholarChatGroundedAnswerBuildPreflightLoading(false);
    }
  }

  function buildScholarChatGroundedAnswerExecutionReadinessRequest(
    trimmedPrompt: string,
  ): ScholarChatGroundedAnswerExecutionReadinessPreviewRequest {
    return {
      build_preflight_preview_request: buildScholarChatGroundedAnswerBuildPreflightRequest(trimmedPrompt),
      execution_consent: scholarChatGroundedAnswerExecutionReadinessExecutionConsent(),
    };
  }

  async function previewScholarChatGroundedAnswerExecutionReadiness() {
    const trimmedPrompt = scholarChatPrompt().trim();
    if (!trimmedPrompt) {
      setScholarChatGroundedAnswerExecutionReadinessPreview(null);
      setScholarChatGroundedAnswerExecutionReadinessError(null);
      setScholarChatGroundedAnswerExecutionReadinessValidationError("Prompt is required to preview grounded answer execution readiness.");
      return;
    }
    if (scholarChatGroundedAnswerExecutionReadinessLoading()) {
      return;
    }

    setScholarChatGroundedAnswerExecutionReadinessHasRun(true);
    setScholarChatGroundedAnswerExecutionReadinessLoading(true);
    setScholarChatGroundedAnswerExecutionReadinessError(null);
    setScholarChatGroundedAnswerExecutionReadinessValidationError(null);
    setScholarChatGroundedAnswerExecutionReadinessPreview(null);
    try {
      const result = await invoke<ScholarChatGroundedAnswerExecutionReadinessPreview>("preview_scholar_chat_grounded_answer_execution_readiness", {
        root: ".",
        request: buildScholarChatGroundedAnswerExecutionReadinessRequest(trimmedPrompt),
      });
      setScholarChatGroundedAnswerExecutionReadinessPreview(result);
    } catch (err) {
      setScholarChatGroundedAnswerExecutionReadinessError(sanitizeBackendError(err));
    } finally {
      setScholarChatGroundedAnswerExecutionReadinessLoading(false);
    }
  }

  async function previewScholarChatGroundedAnswerExecutionPlan() {
    const trimmedPrompt = scholarChatPrompt().trim();
    if (!trimmedPrompt) {
      setScholarChatGroundedAnswerExecutionPlanPreview(null);
      setScholarChatGroundedAnswerExecutionPlanError(null);
      setScholarChatGroundedAnswerExecutionPlanValidationError("Prompt is required to preview grounded answer execution plan.");
      return;
    }
    if (scholarChatGroundedAnswerExecutionPlanLoading()) {
      return;
    }

    setScholarChatGroundedAnswerExecutionPlanHasRun(true);
    setScholarChatGroundedAnswerExecutionPlanLoading(true);
    setScholarChatGroundedAnswerExecutionPlanError(null);
    setScholarChatGroundedAnswerExecutionPlanValidationError(null);
    setScholarChatGroundedAnswerExecutionPlanPreview(null);
    try {
      const result = await invoke<ScholarChatGroundedAnswerExecutionPlanPreview>("preview_scholar_chat_grounded_answer_execution_plan", {
        root: ".",
        request: {
          execution_readiness_preview_request: {
            build_preflight_preview_request: buildScholarChatGroundedAnswerExecutionReadinessRequest(trimmedPrompt).build_preflight_preview_request,
            execution_consent: scholarChatGroundedAnswerExecutionReadinessExecutionConsent(),
          },
        } satisfies ScholarChatGroundedAnswerExecutionPlanPreviewRequest,
      });
      setScholarChatGroundedAnswerExecutionPlanPreview(result);
    } catch (err) {
      setScholarChatGroundedAnswerExecutionPlanError(sanitizeBackendError(err));
    } finally {
      setScholarChatGroundedAnswerExecutionPlanLoading(false);
    }
  }

  function buildScholarChatRuntimeDiagnosticBridgeRequest(
    trimmedPrompt: string,
  ): ScholarChatRuntimeDiagnosticBridgePreviewRequest | null {
    const smokeExecutionPlanPreviewRequest = buildLocalRuntimeSmokeExecutionPlanPreviewRequest();
    if (!smokeExecutionPlanPreviewRequest) {
      return null;
    }

    return {
      scholar_chat_request: {
        prompt: trimmedPrompt,
        mode: scholarChatMode(),
        grounding_policy: scholarChatGroundingPolicy(),
        selected_source_ids: selectedScholarChatSourceIds(),
      },
      smoke_execution_plan_preview_request: smokeExecutionPlanPreviewRequest,
    };
  }

  async function previewScholarChatRuntimeDiagnosticBridge() {
    const trimmedPrompt = scholarChatPrompt().trim();
    if (!trimmedPrompt) {
      setScholarChatRuntimeDiagnosticBridgePreview(null);
      setScholarChatRuntimeDiagnosticBridgeError(null);
      setScholarChatRuntimeDiagnosticBridgeValidationError("Prompt is required to preview the Scholar Chat runtime diagnostic bridge.");
      return;
    }
    if (scholarChatRuntimeDiagnosticBridgeLoading()) {
      return;
    }

    clearScholarChatRuntimeDiagnosticResultPreview();
    const request = buildScholarChatRuntimeDiagnosticBridgeRequest(trimmedPrompt);
    if (!request) {
      setScholarChatRuntimeDiagnosticBridgePreview(null);
      setScholarChatRuntimeDiagnosticBridgeError(null);
      setScholarChatRuntimeDiagnosticBridgeValidationError("Local runtime inputs must be valid before previewing the Scholar Chat runtime diagnostic bridge.");
      return;
    }

    setScholarChatRuntimeDiagnosticBridgeHasRun(true);
    setScholarChatRuntimeDiagnosticBridgeLoading(true);
    setScholarChatRuntimeDiagnosticBridgeError(null);
    setScholarChatRuntimeDiagnosticBridgeValidationError(null);
    setScholarChatRuntimeDiagnosticBridgePreview(null);
    try {
      const result = await invoke<ScholarChatRuntimeDiagnosticBridgePreview>("preview_scholar_chat_runtime_diagnostic_bridge", {
        root: ".",
        request,
      });
      setScholarChatRuntimeDiagnosticBridgePreview(result);
    } catch (err) {
      setScholarChatRuntimeDiagnosticBridgeError(sanitizeBackendError(err));
    } finally {
      setScholarChatRuntimeDiagnosticBridgeLoading(false);
    }
  }

  function buildScholarChatRuntimeDiagnosticResultRequest(
    trimmedPrompt: string,
    diagnosticPreview: LocalRuntimeSmokeDiagnosticPreview,
  ): ScholarChatRuntimeDiagnosticResultPreviewRequest | null {
    const bridgePreviewRequest = buildScholarChatRuntimeDiagnosticBridgeRequest(trimmedPrompt);
    if (!bridgePreviewRequest) {
      return null;
    }

    return {
      bridge_preview_request: bridgePreviewRequest,
      diagnostic_preview: diagnosticPreview,
    };
  }

  async function previewScholarChatRuntimeDiagnosticResult() {
    const trimmedPrompt = scholarChatPrompt().trim();
    if (!trimmedPrompt) {
      setScholarChatRuntimeDiagnosticResultPreview(null);
      setScholarChatRuntimeDiagnosticResultError(null);
      setScholarChatRuntimeDiagnosticResultValidationError("Prompt is required to preview the Scholar Chat runtime diagnostic result.");
      return;
    }
    if (scholarChatRuntimeDiagnosticResultLoading()) {
      return;
    }

    const diagnosticPreview = localRuntimeSmokeResult();
    if (!diagnosticPreview) {
      setScholarChatRuntimeDiagnosticResultPreview(null);
      setScholarChatRuntimeDiagnosticResultError(null);
      setScholarChatRuntimeDiagnosticResultValidationError("A loaded smoke diagnostic preview is required before previewing the Scholar Chat runtime diagnostic result.");
      return;
    }

    const request = buildScholarChatRuntimeDiagnosticResultRequest(trimmedPrompt, diagnosticPreview);
    if (!request) {
      setScholarChatRuntimeDiagnosticResultPreview(null);
      setScholarChatRuntimeDiagnosticResultError(null);
      setScholarChatRuntimeDiagnosticResultValidationError("Local runtime inputs must be valid before previewing the Scholar Chat runtime diagnostic result.");
      return;
    }

    setScholarChatRuntimeDiagnosticResultHasRun(true);
    setScholarChatRuntimeDiagnosticResultLoading(true);
    setScholarChatRuntimeDiagnosticResultError(null);
    setScholarChatRuntimeDiagnosticResultValidationError(null);
    setScholarChatRuntimeDiagnosticResultPreview(null);
    try {
      const result = await invoke<ScholarChatRuntimeDiagnosticResultPreview>("preview_scholar_chat_runtime_diagnostic_result", {
        root: ".",
        request,
      });
      setScholarChatRuntimeDiagnosticResultPreview(result);
    } catch (err) {
      setScholarChatRuntimeDiagnosticResultError(sanitizeBackendError(err));
    } finally {
      setScholarChatRuntimeDiagnosticResultLoading(false);
    }
  }

  async function previewLocalRuntimeHealth() {
    if (localRuntimeLoading()) {
      return;
    }

    const contextWindow = parseOptionalIntegerInput(localRuntimeContextWindow(), "Context window");
    const gpuLayers = parseOptionalIntegerInput(localRuntimeGpuLayers(), "GPU layers");
    const temperature = parseOptionalNumberInput(localRuntimeTemperature(), "Temperature");
    if (
      contextWindow === undefined ||
      gpuLayers === undefined ||
      temperature === undefined
    ) {
      setLocalRuntimeHasRun(true);
      setLocalRuntimePreview(null);
      return;
    }

    setLocalRuntimeHasRun(true);
    setLocalRuntimeLoading(true);
    setLocalRuntimeError(null);
    setLocalRuntimeValidationError(null);
    setLocalRuntimePreview(null);
    try {
      const result = await invoke<LocalModelRuntimeHealthPreview>("preview_local_model_runtime_health", {
        root: ".",
        config: {
          runtime_kind: localRuntimeKind(),
          model_path: normalizeOptionalTextInput(localRuntimeModelPath()),
          executable_path: normalizeOptionalTextInput(localRuntimeExecutablePath()),
          context_window: contextWindow,
          gpu_layers: gpuLayers,
          temperature: temperature,
        },
      });
      setLocalRuntimePreview(result);
    } catch (err) {
      setLocalRuntimeError(sanitizeBackendError(err));
    } finally {
      setLocalRuntimeLoading(false);
    }
  }

  async function previewLocalRuntimeInvocationPlan() {
    if (localRuntimeInvocationLoading()) {
      return;
    }

    const contextWindow = parseOptionalIntegerInput(
      localRuntimeContextWindow(),
      "Context window",
      setLocalRuntimeInvocationValidationError,
    );
    const gpuLayers = parseOptionalIntegerInput(
      localRuntimeGpuLayers(),
      "GPU layers",
      setLocalRuntimeInvocationValidationError,
    );
    const temperature = parseOptionalNumberInput(
      localRuntimeTemperature(),
      "Temperature",
      setLocalRuntimeInvocationValidationError,
    );
    const maxOutputTokens = parseOptionalIntegerInput(
      localRuntimeInvocationMaxOutputTokens(),
      "Max output tokens",
      setLocalRuntimeInvocationValidationError,
    );
    const stopSequences = parseOptionalCommaSeparatedListInput(localRuntimeInvocationStopSequences());
    if (
      contextWindow === undefined ||
      gpuLayers === undefined ||
      temperature === undefined ||
      maxOutputTokens === undefined
    ) {
      setLocalRuntimeInvocationHasRun(true);
      setLocalRuntimeInvocationPreview(null);
      setLocalRuntimeInvocationError(null);
      return;
    }

    const trimmedPromptText = scholarChatPrompt().trim();
    const promptPackEstimate = scholarChatPromptPackPreview()?.prompt_pack.estimated_input_char_count ?? null;
    const estimatedInputCharCount = promptPackEstimate ?? (trimmedPromptText ? Array.from(trimmedPromptText).length : null);

    if (localRuntimeInvocationLoading()) {
      return;
    }

    setLocalRuntimeInvocationHasRun(true);
    setLocalRuntimeInvocationLoading(true);
    setLocalRuntimeInvocationError(null);
    setLocalRuntimeInvocationValidationError(null);
    setLocalRuntimeInvocationPreview(null);
    try {
      const request: LocalRuntimeInvocationPlanRequest = {
        runtime_config: {
          runtime_kind: localRuntimeKind(),
          model_path: normalizeOptionalTextInput(localRuntimeModelPath()),
          executable_path: normalizeOptionalTextInput(localRuntimeExecutablePath()),
          context_window: contextWindow,
          gpu_layers: gpuLayers,
          temperature: temperature,
        },
        prompt_text: trimmedPromptText || null,
        estimated_input_char_count: estimatedInputCharCount,
        max_output_tokens: maxOutputTokens,
        stop_sequences: stopSequences,
      };
      const result = await invoke<LocalRuntimeInvocationPlanPreview>("preview_local_runtime_invocation_plan", {
        root: ".",
        request,
      });
      setLocalRuntimeInvocationPreview(result);
    } catch (err) {
      setLocalRuntimeInvocationError(sanitizeBackendError(err));
    } finally {
      setLocalRuntimeInvocationLoading(false);
    }
  }

  function buildLocalRuntimeAdapterContractPreviewRequest(
    setValidationError: Setter<string | null>,
  ): LocalRuntimeAdapterContractPreviewRequest | null {
    const contextWindowTokens = parseOptionalIntegerInput(
      localRuntimeAdapterContextWindowTokens(),
      "Context window tokens",
      setValidationError,
    );
    const gpuLayers = parseOptionalIntegerInput(
      localRuntimeAdapterGpuLayers(),
      "GPU layers",
      setValidationError,
    );
    const threads = parseOptionalIntegerInput(
      localRuntimeAdapterThreads(),
      "Threads",
      setValidationError,
    );
    const batchSize = parseOptionalIntegerInput(
      localRuntimeAdapterBatchSize(),
      "Batch size",
      setValidationError,
    );

    if (
      contextWindowTokens === undefined ||
      gpuLayers === undefined ||
      threads === undefined ||
      batchSize === undefined
    ) {
      return null;
    }

    return {
      adapter_kind: "llama_cpp",
      executable_path: normalizeOptionalTextInput(localRuntimeAdapterExecutablePath()),
      model_path: normalizeOptionalTextInput(localRuntimeAdapterModelPath()),
      model_family: normalizeOptionalTextInput(localRuntimeAdapterModelFamily()),
      model_format: normalizeOptionalTextInput(localRuntimeAdapterModelFormat()),
      context_window_tokens: contextWindowTokens,
      gpu_layers: gpuLayers,
      threads,
      batch_size: batchSize,
      chat_template: normalizeOptionalTextInput(localRuntimeAdapterChatTemplate()),
    };
  }

  async function previewLocalRuntimeAdapterContract() {
    if (localRuntimeAdapterLoading()) {
      return;
    }

    const request = buildLocalRuntimeAdapterContractPreviewRequest(setLocalRuntimeAdapterValidationError);
    if (!request) {
      setLocalRuntimeAdapterHasRun(true);
      setLocalRuntimeAdapterPreview(null);
      setLocalRuntimeAdapterError(null);
      return;
    }

    setLocalRuntimeAdapterHasRun(true);
    setLocalRuntimeAdapterLoading(true);
    setLocalRuntimeAdapterError(null);
    setLocalRuntimeAdapterValidationError(null);
    setLocalRuntimeAdapterPreview(null);
    try {
      const result = await invoke<LocalRuntimeAdapterContractPreview>("preview_llama_runtime_adapter_contract", {
        root: ".",
        request,
      });
      setLocalRuntimeAdapterPreview(result);
    } catch (err) {
      setLocalRuntimeAdapterError(sanitizeBackendError(err));
    } finally {
      setLocalRuntimeAdapterLoading(false);
    }
  }

  function buildLocalRuntimeValidationPreviewRequest(
    setValidationError: Setter<string | null>,
  ): LocalRuntimeValidationPreviewRequest | null {
    const adapterContractRequest = buildLocalRuntimeAdapterContractPreviewRequest(
      setValidationError,
    );
    if (!adapterContractRequest) {
      return null;
    }

    return {
      adapter_contract_request: adapterContractRequest,
    };
  }

  async function previewLocalRuntimeValidation() {
    if (localRuntimeValidationPreviewLoading()) {
      return;
    }

    const request = buildLocalRuntimeValidationPreviewRequest(setLocalRuntimeValidationPreviewInputError);
    if (!request) {
      setLocalRuntimeValidationPreviewHasRun(true);
      setLocalRuntimeValidationPreview(null);
      setLocalRuntimeValidationPreviewError(null);
      return;
    }

    setLocalRuntimeValidationPreviewHasRun(true);
    setLocalRuntimeValidationPreviewLoading(true);
    setLocalRuntimeValidationPreviewError(null);
    setLocalRuntimeValidationPreviewInputError(null);
    setLocalRuntimeValidationPreview(null);
    try {
      const result = await invoke<LocalRuntimeValidationPreview>("preview_llama_runtime_validation", {
        root: ".",
        request,
      });
      setLocalRuntimeValidationPreview(result);
    } catch (err) {
      setLocalRuntimeValidationPreviewError(sanitizeBackendError(err));
    } finally {
      setLocalRuntimeValidationPreviewLoading(false);
    }
  }

  function buildLocalRuntimeProbeReadinessPreviewRequest(): LocalRuntimeProbeReadinessPreviewRequest | null {
    const validationPreviewRequest = buildLocalRuntimeValidationPreviewRequest(
      setLocalRuntimeProbeReadinessPreviewInputError,
    );
    if (!validationPreviewRequest) {
      return null;
    }

    return {
      validation_preview_request: validationPreviewRequest,
      probe_consent: localRuntimeProbeReadinessConsent(),
    };
  }

  function buildLocalRuntimeVersionProbePreviewRequest(): LocalRuntimeVersionProbePreviewRequest | null {
    const probeReadinessPreviewRequest = buildLocalRuntimeProbeReadinessPreviewRequest();
    if (!probeReadinessPreviewRequest) {
      return null;
    }

    const timeoutMs = parseOptionalIntegerInput(localRuntimeProbeTimeoutMs(), "Version probe timeout", setLocalRuntimeProbeValidationError);
    if (timeoutMs === undefined) {
      return null;
    }

    return {
      probe_readiness_preview_request: probeReadinessPreviewRequest,
      allow_probe_execution: localRuntimeProbeAllowExecution(),
      timeout_ms: timeoutMs,
    };
  }

  function buildLocalRuntimeCapabilityPreviewRequest(): LocalRuntimeCapabilityPreviewRequest | null {
    const versionProbePreviewRequest = buildLocalRuntimeVersionProbePreviewRequest();
    if (!versionProbePreviewRequest) {
      return null;
    }

    return {
      version_probe_preview_request: versionProbePreviewRequest,
    };
  }

  function buildLocalRuntimeSmokeReadinessPreviewRequest(): LocalRuntimeSmokeReadinessPreviewRequest | null {
    const capabilityPreviewRequest = buildLocalRuntimeCapabilityPreviewRequest();
    if (!capabilityPreviewRequest) {
      return null;
    }

    const maxOutputTokens = parseOptionalIntegerInput(
      localRuntimeSmokeMaxOutputTokens(),
      "Smoke readiness max output tokens",
      setLocalRuntimeSmokeReadinessValidationError,
    );
    const timeoutMs = parseOptionalIntegerInput(
      localRuntimeSmokeTimeoutMs(),
      "Smoke readiness timeout",
      setLocalRuntimeSmokeReadinessValidationError,
    );
    if (maxOutputTokens === undefined || timeoutMs === undefined) {
      return null;
    }

    const diagnosticPrompt = localRuntimeSmokePrompt().trim();

    return {
      capability_preview_request: capabilityPreviewRequest,
      smoke_consent: localRuntimeSmokeReadinessConsent(),
      diagnostic_prompt: diagnosticPrompt ? diagnosticPrompt : null,
      max_output_tokens: maxOutputTokens,
      timeout_ms: timeoutMs,
    };
  }

  function buildLocalRuntimeSmokeExecutionPlanPreviewRequest(): LocalRuntimeSmokeExecutionPlanPreviewRequest | null {
    const smokeReadinessPreviewRequest = buildLocalRuntimeSmokeReadinessPreviewRequest();
    if (!smokeReadinessPreviewRequest) {
      return null;
    }

    return {
      smoke_readiness_preview_request: smokeReadinessPreviewRequest,
    };
  }

  async function previewLocalRuntimeSmokeExecutionPlan() {
    if (localRuntimeSmokeExecutionPlanLoading()) {
      return;
    }

    const request = buildLocalRuntimeSmokeExecutionPlanPreviewRequest();
    if (!request) {
      setLocalRuntimeSmokeExecutionPlanHasRun(true);
      setLocalRuntimeSmokeExecutionPlanPreview(null);
      setLocalRuntimeSmokeExecutionPlanError(null);
      setLocalRuntimeSmokeExecutionPlanValidationError(null);
      return;
    }

    setLocalRuntimeSmokeExecutionPlanHasRun(true);
    setLocalRuntimeSmokeExecutionPlanLoading(true);
    setLocalRuntimeSmokeExecutionPlanError(null);
    setLocalRuntimeSmokeExecutionPlanValidationError(null);
    setLocalRuntimeSmokeExecutionPlanPreview(null);
    try {
      const result = await invoke<LocalRuntimeSmokeExecutionPlanPreview>("preview_llama_runtime_smoke_execution_plan", {
        root: ".",
        request,
      });
      setLocalRuntimeSmokeExecutionPlanPreview(result);
    } catch (err) {
      setLocalRuntimeSmokeExecutionPlanError(sanitizeBackendError(err));
    } finally {
      setLocalRuntimeSmokeExecutionPlanLoading(false);
    }
  }

  function buildLocalRuntimeSmokeDiagnosticRequest(): LocalRuntimeSmokeDiagnosticRequest | null {
    const smokeExecutionPlanPreviewRequest = buildLocalRuntimeSmokeExecutionPlanPreviewRequest();
    if (!smokeExecutionPlanPreviewRequest) {
      return null;
    }

    return {
      smoke_execution_plan_preview_request: smokeExecutionPlanPreviewRequest,
      allow_smoke_execution: localRuntimeSmokeAllowExecution(),
    };
  }

  async function previewLocalRuntimeSmokeDiagnostic() {
    if (localRuntimeSmokeLoading()) {
      return;
    }

    const request = buildLocalRuntimeSmokeDiagnosticRequest();
    if (!request) {
      setLocalRuntimeSmokeHasRun(true);
      setLocalRuntimeSmokeResult(null);
      setLocalRuntimeSmokeError(null);
      setLocalRuntimeSmokeValidationError("Diagnostic smoke inputs could not be prepared.");
      return;
    }

    setLocalRuntimeSmokeHasRun(true);
    setLocalRuntimeSmokeLoading(true);
    setLocalRuntimeSmokeError(null);
    setLocalRuntimeSmokeValidationError(null);
    setLocalRuntimeSmokeResult(null);
    try {
      const result = await invoke<LocalRuntimeSmokeDiagnosticPreview>("run_llama_runtime_smoke_diagnostic", {
        root: ".",
        request,
      });
      setLocalRuntimeSmokeResult(result);
    } catch (err) {
      setLocalRuntimeSmokeError(sanitizeBackendError(err));
    } finally {
      setLocalRuntimeSmokeLoading(false);
    }
  }

  async function previewLocalRuntimeProbeReadiness() {
    if (localRuntimeProbeReadinessPreviewLoading()) {
      return;
    }

    const request = buildLocalRuntimeProbeReadinessPreviewRequest();
    if (!request) {
      setLocalRuntimeProbeReadinessPreviewHasRun(true);
      setLocalRuntimeProbeReadinessPreview(null);
      setLocalRuntimeProbeReadinessPreviewError(null);
      return;
    }

    setLocalRuntimeProbeReadinessPreviewHasRun(true);
    setLocalRuntimeProbeReadinessPreviewLoading(true);
    setLocalRuntimeProbeReadinessPreviewError(null);
    setLocalRuntimeProbeReadinessPreviewInputError(null);
    setLocalRuntimeProbeReadinessPreview(null);
    try {
      const result = await invoke<LocalRuntimeProbeReadinessPreview>("preview_llama_runtime_probe_readiness", {
        root: ".",
        request,
      });
      setLocalRuntimeProbeReadinessPreview(result);
    } catch (err) {
      setLocalRuntimeProbeReadinessPreviewError(sanitizeBackendError(err));
    } finally {
      setLocalRuntimeProbeReadinessPreviewLoading(false);
    }
  }

  async function previewLocalRuntimeVersionProbe() {
    if (localRuntimeProbeLoading()) {
      return;
    }

    setLocalRuntimeProbeValidationError(null);
    const request = buildLocalRuntimeVersionProbePreviewRequest();
    if (!request) {
      setLocalRuntimeProbeHasRun(true);
      setLocalRuntimeProbeResult(null);
      setLocalRuntimeProbeError(null);
      return;
    }

    setLocalRuntimeProbeHasRun(true);
    setLocalRuntimeProbeLoading(true);
    setLocalRuntimeProbeError(null);
    setLocalRuntimeProbeValidationError(null);
    setLocalRuntimeProbeResult(null);
    clearLocalRuntimeCapabilityPreview();
    try {
      const result = await invoke<LocalRuntimeVersionProbePreview>("run_llama_runtime_version_probe", {
        root: ".",
        request,
      });
      setLocalRuntimeProbeResult(result);
    } catch (err) {
      setLocalRuntimeProbeError(sanitizeBackendError(err));
    } finally {
      setLocalRuntimeProbeLoading(false);
    }
  }

  async function previewLocalRuntimeCapability() {
    if (localRuntimeCapabilityLoading()) {
      return;
    }

    const request = buildLocalRuntimeCapabilityPreviewRequest();
    if (!request) {
      setLocalRuntimeCapabilityHasRun(true);
      setLocalRuntimeCapabilityResult(null);
      setLocalRuntimeCapabilityError(null);
      return;
    }

    setLocalRuntimeCapabilityHasRun(true);
    setLocalRuntimeCapabilityLoading(true);
    setLocalRuntimeCapabilityError(null);
    setLocalRuntimeCapabilityResult(null);
    try {
      const result = await invoke<LocalRuntimeCapabilityPreview>("preview_llama_runtime_capability", {
        root: ".",
        request,
      });
      setLocalRuntimeCapabilityResult(result);
    } catch (err) {
      setLocalRuntimeCapabilityError(sanitizeBackendError(err));
    } finally {
      setLocalRuntimeCapabilityLoading(false);
    }
  }

  async function previewLocalRuntimeSmokeReadiness() {
    if (localRuntimeSmokeReadinessLoading()) {
      return;
    }

    setLocalRuntimeSmokeReadinessValidationError(null);
    const request = buildLocalRuntimeSmokeReadinessPreviewRequest();
    if (!request) {
      setLocalRuntimeSmokeReadinessHasRun(true);
      setLocalRuntimeSmokeReadinessResult(null);
      setLocalRuntimeSmokeReadinessError(null);
      return;
    }

    setLocalRuntimeSmokeReadinessHasRun(true);
    setLocalRuntimeSmokeReadinessLoading(true);
    setLocalRuntimeSmokeReadinessError(null);
    setLocalRuntimeSmokeReadinessValidationError(null);
    setLocalRuntimeSmokeReadinessResult(null);
    try {
      const result = await invoke<LocalRuntimeSmokeReadinessPreview>("preview_llama_runtime_smoke_readiness", {
        root: ".",
        request,
      });
      setLocalRuntimeSmokeReadinessResult(result);
    } catch (err) {
      setLocalRuntimeSmokeReadinessError(sanitizeBackendError(err));
    } finally {
      setLocalRuntimeSmokeReadinessLoading(false);
    }
  }

  onMount(() => {
    void loadScholarChatSourceContext();
    void loadStatus();
    void loadManagedLlamaServerStatus();
  });

  function activateWorkspace(workspace: WorkspaceSection) {
    const target = WORKSPACE_SECTIONS.find((item) => item.value === workspace);
    setActiveWorkspace(workspace);
    if (target) {
      queueMicrotask(() => {
        document.getElementById(target.targetId)?.scrollIntoView({ behavior: "smooth", block: "start" });
      });
    }
  }

  const activeWorkspaceItem = () => WORKSPACE_SECTIONS.find((item) => item.value === activeWorkspace());

  return (
    <WorkspaceShell
      activeWorkspace={activeWorkspace()}
      activeLabel={activeWorkspaceItem()?.label ?? "Scholar Chat"}
      activeDescription={activeWorkspaceItem()?.description ?? "Chat-first academic workflow"}
      workspaceSections={WORKSPACE_SECTIONS}
      onActivate={(workspace) => activateWorkspace(workspace as WorkspaceSection)}
    >
      <div class="workspace-stack">
        <SourcesWorkspace
        status={status()}
        statusError={statusError()}
        sourceContextLoading={scholarChatSourceContextLoading()}
        sourceContextError={scholarChatSourceContextError()}
        sourceContext={scholarChatSourceContext()}
        sourceContextSelectedIds={scholarChatSourceContextSelectedIds()}
        renderFirstRunSourceReadiness={renderFirstRunSourceReadiness}
        renderSourceWorkflowActionHints={renderSourceWorkflowActionHints}
        selectedSourceSummary={scholarChatSelectedSourceIdsSummary()}
        toggleSourceContext={toggleScholarChatSourceContext}
        setScholarChatPreview={setScholarChatPreview}
        setScholarChatExecutionGatePreview={setScholarChatExecutionGatePreview}
        formatSnakeCaseLabel={formatSnakeCaseLabel}
        refreshCorpusStatus={loadStatus}
        refreshSourceContext={loadScholarChatSourceContext}
      />

        <ScholarChatWorkspace
          transcript={scholarChatTranscript()}
          suggestions={SCHOLAR_CHAT_PROMPT_SUGGESTIONS}
          runtimeReadinessNote={managedLlamaServerReadinessSummary()}
          prompt={scholarChatPrompt()}
          validationError={scholarChatValidationError()}
          error={scholarChatError()}
        previewLoading={scholarChatLoading()}
        executionGateLoading={scholarChatExecutionGateLoading()}
        selectedSourceSummary={scholarChatSelectedSourceIdsSummary()}
        mode={scholarChatMode()}
        groundingPolicy={scholarChatGroundingPolicy()}
        modes={SCHOLAR_CHAT_MODES}
        groundingPolicies={GROUNDING_POLICIES}
        onApplySuggestion={applyScholarChatPromptSuggestion}
        onPromptInput={(value: string) => {
          setScholarChatPrompt(value);
          resetScholarChatConversationPreviewState();
        }}
        onPreviewPlan={previewScholarChatAgenticWorkflowPlan}
        onCheckNextStep={previewScholarChatAgenticWorkflowExecutionGate}
        onModeChange={(value: string) => {
          setScholarChatMode(value as ScholarChatMode);
          resetScholarChatConversationPreviewState();
        }}
        onGroundingPolicyChange={(value: string) => {
          setScholarChatGroundingPolicy(value as GroundingPolicy);
          resetScholarChatConversationPreviewState();
        }}
        renderMetricGrid={renderMetricGrid}
        formatSnakeCaseLabel={formatSnakeCaseLabel}
      />

      <details class="advanced-panels">
            <summary>Advanced preview panels</summary>
        <div class="artifact-overview">
          <h3>Retrieval preview</h3>
          <p class="muted">
            Read-only retrieval candidate preview for the selected source context. It does not build indexes or generate answers.
          </p>
          <div class="hero-actions">
            <button onClick={previewScholarChatRetrieval} disabled={scholarChatRetrievalLoading()}>
              {scholarChatRetrievalLoading() ? "Previewing..." : "Preview retrieval candidates"}
            </button>
          </div>
          {scholarChatRetrievalError() && <p class="error">{scholarChatRetrievalError()}</p>}
          {scholarChatRetrievalLoading() ? (
            <p>Previewing retrieval candidates...</p>
          ) : scholarChatRetrievalHasRun() ? (
            scholarChatRetrievalPreview() ? (
              <>
                <div class="contract-meta">
                  <div><span>Status</span><strong>{scholarChatRetrievalPreview()!.status}</strong></div>
                  <div><span>Mode</span><strong>{scholarChatRetrievalPreview()!.mode}</strong></div>
                  <div><span>Grounding policy</span><strong>{scholarChatRetrievalPreview()!.grounding_policy}</strong></div>
                  <div><span>Selected sources</span><strong>{scholarChatRetrievalPreview()!.selected_source_count}</strong></div>
                  <div><span>Candidates</span><strong>{scholarChatRetrievalPreview()!.candidate_count}</strong></div>
                </div>
                <p><strong>Prompt:</strong> {scholarChatRetrievalPreview()!.normalized_prompt}</p>
                {scholarChatRetrievalPreview()!.warnings.length > 0 ? (
                  <div class="warning-box">
                    <h4>Warnings</h4>
                    <ul>
                      {scholarChatRetrievalPreview()!.warnings.map((warning) => (
                        <li>{warning}</li>
                      ))}
                    </ul>
                  </div>
                ) : null}
                {scholarChatRetrievalPreview()!.candidates.length > 0 ? (
                  <ul class="final-answer-list-items">
                    {scholarChatRetrievalPreview()!.candidates.map((item) => (
                      <li>
                        <div class="final-answer-list-item">
                          <span>{item.chunk_id}</span>
                          <small>
                            source={item.source_id} | version={item.version_id} | score={item.score.toFixed(3)} | matched={item.matched_terms.join(", ") || "none"}
                          </small>
                          <small>{locatorSummary(item.locator)}</small>
                          <p>{item.preview}</p>
                        </div>
                      </li>
                    ))}
                  </ul>
                ) : (
                  <p>
                    {scholarChatRetrievalPreview()!.selected_source_count > 0
                      ? "No retrieval candidates matched the selected context."
                      : "No source selected yet; preview was unscoped."}
                  </p>
                )}
              </>
            ) : (
              <p>No Scholar Chat retrieval preview loaded yet.</p>
            )
          ) : (
            <p>No Scholar Chat retrieval preview loaded yet.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>Evidence plan preview</h3>
          <p class="muted">
            Read-only evidence-plan preview for retrieval candidates that would be eligible for Evidence Pack assembly later. It does not build an Evidence Pack or generate an answer.
          </p>
          <div class="hero-actions">
            <button onClick={previewScholarChatEvidencePlan} disabled={scholarChatEvidencePlanLoading()}>
              {scholarChatEvidencePlanLoading() ? "Previewing..." : "Preview evidence plan"}
            </button>
          </div>
          {scholarChatEvidencePlanError() && <p class="error">{scholarChatEvidencePlanError()}</p>}
          {scholarChatEvidencePlanLoading() ? (
            <p>Previewing evidence plan...</p>
          ) : scholarChatEvidencePlanHasRun() ? (
            scholarChatEvidencePlanPreview() ? (
              <>
                <div class="contract-meta">
                  <div><span>Status</span><strong>{scholarChatEvidencePlanPreview()!.status}</strong></div>
                  <div><span>Mode</span><strong>{scholarChatEvidencePlanPreview()!.mode}</strong></div>
                  <div><span>Grounding policy</span><strong>{scholarChatEvidencePlanPreview()!.grounding_policy}</strong></div>
                  <div><span>Selected sources</span><strong>{scholarChatEvidencePlanPreview()!.selected_source_count}</strong></div>
                  <div><span>Retrieval candidates</span><strong>{scholarChatEvidencePlanPreview()!.retrieval_candidate_count}</strong></div>
                  <div><span>Evidence candidates</span><strong>{scholarChatEvidencePlanPreview()!.evidence_candidate_count}</strong></div>
                </div>
                <p><strong>Prompt:</strong> {scholarChatEvidencePlanPreview()!.normalized_prompt}</p>
                <p>{scholarChatEvidencePlanPreview()!.evidence_plan.summary}</p>
                <div class="contract-meta">
                  <div><span>Evidence required</span><strong>{scholarChatEvidencePlanPreview()!.evidence_plan.evidence_required ? "yes" : "no"}</strong></div>
                  <div><span>Evidence Pack later</span><strong>{scholarChatEvidencePlanPreview()!.evidence_plan.evidence_pack_would_be_built_later ? "yes" : "no"}</strong></div>
                </div>
                <h4>Plan steps</h4>
                <ul>
                  {scholarChatEvidencePlanPreview()!.evidence_plan.steps.map((step) => (
                    <li>{step}</li>
                  ))}
                </ul>
                {scholarChatEvidencePlanPreview()!.warnings.length > 0 ? (
                  <div class="warning-box">
                    <h4>Warnings</h4>
                    <ul>
                      {scholarChatEvidencePlanPreview()!.warnings.map((warning) => (
                        <li>{warning}</li>
                      ))}
                    </ul>
                  </div>
                ) : null}
                {scholarChatEvidencePlanPreview()!.candidates.length > 0 ? (
                  <ul class="final-answer-list-items">
                    {scholarChatEvidencePlanPreview()!.candidates.map((item) => (
                      <li>
                        <div class="final-answer-list-item">
                          <span>{item.chunk_id}</span>
                          <small>
                            source={item.source_id} | version={item.version_id} | score={item.score.toFixed(3)} | matched={item.matched_terms.join(", ") || "none"}
                          </small>
                          <small>{locatorSummary(item.locator)}</small>
                          <p>{item.preview}</p>
                        </div>
                      </li>
                    ))}
                  </ul>
                ) : (
                  <p>
                    {scholarChatEvidencePlanPreview()!.selected_source_count > 0
                      ? "No retrieval candidates were eligible for Evidence Pack assembly yet."
                      : "No Scholar Chat source context selected; preview is unscoped."}
                  </p>
                )}
              </>
            ) : (
              <p>No Scholar Chat evidence plan preview loaded yet.</p>
            )
          ) : (
            <p>No Scholar Chat evidence plan preview loaded yet.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>Prompt pack preview</h3>
          <p class="muted">
            Read-only prompt-pack preview for the current Scholar Chat request. It shows the prompt assembly that would be sent later, but it does not call a model or generate an answer.
          </p>
          <p class="muted">{scholarChatSelectedSourceIdsSummary()}</p>
          <div class="hero-actions">
            <button onClick={previewScholarChatPromptPack} disabled={scholarChatPromptPackLoading()}>
              {scholarChatPromptPackLoading() ? "Previewing..." : "Preview prompt pack"}
            </button>
          </div>
          {scholarChatPromptPackError() && <p class="error">{scholarChatPromptPackError()}</p>}
          {scholarChatPromptPackLoading() ? (
            <p>Previewing prompt pack...</p>
          ) : scholarChatPromptPackHasRun() ? (
            scholarChatPromptPackPreview() ? (
              <>
                <div class="contract-meta">
                  <div><span>Status</span><strong>{scholarChatPromptPackPreview()!.status}</strong></div>
                  <div><span>Mode</span><strong>{scholarChatPromptPackPreview()!.mode}</strong></div>
                  <div><span>Grounding policy</span><strong>{scholarChatPromptPackPreview()!.grounding_policy}</strong></div>
                  <div><span>Selected sources</span><strong>{scholarChatPromptPackPreview()!.selected_source_count}</strong></div>
                  <div><span>Evidence candidates</span><strong>{scholarChatPromptPackPreview()!.evidence_candidate_count}</strong></div>
                  <div><span>Sections</span><strong>{scholarChatPromptPackPreview()!.prompt_pack.section_count}</strong></div>
                  <div><span>Context items</span><strong>{scholarChatPromptPackPreview()!.prompt_pack.context_item_count}</strong></div>
                  <div><span>Estimated chars</span><strong>{scholarChatPromptPackPreview()!.prompt_pack.estimated_input_char_count}</strong></div>
                </div>
                <p><strong>Prompt:</strong> {scholarChatPromptPackPreview()!.normalized_prompt}</p>
                {scholarChatPromptPackPreview()!.warnings.length > 0 ? (
                  <div class="warning-box">
                    <h4>Warnings</h4>
                    <ul>
                      {scholarChatPromptPackPreview()!.warnings.map((warning) => (
                        <li>{warning}</li>
                      ))}
                    </ul>
                  </div>
                ) : null}
                {scholarChatPromptPackPreview()!.prompt_pack.sections.length > 0 ? (
                  scholarChatPromptPackPreview()!.prompt_pack.sections.map((section) => (
                    <div class="artifact-overview">
                      <h4>{section.title}</h4>
                      <p class="muted">Kind: {section.kind}</p>
                      {section.lines.length > 0 ? (
                        <ul class="final-answer-list-items">
                          {section.lines.map((line) => (
                            <li>
                              <div class="final-answer-list-item">
                                <span>{line}</span>
                              </div>
                            </li>
                          ))}
                        </ul>
                      ) : (
                        <p>No lines available.</p>
                      )}
                    </div>
                  ))
                ) : (
                  <p>No prompt-pack sections available.</p>
                )}
                {scholarChatPromptPackPreview()!.context_items.length > 0 ? (
                  <ul class="final-answer-list-items">
                    {scholarChatPromptPackPreview()!.context_items.map((item) => (
                      <li>
                        <div class="final-answer-list-item">
                          <span>{item.chunk_id}</span>
                          <small>
                            source={item.source_id} | version={item.version_id} | score={item.score.toFixed(3)} | matched={item.matched_terms.join(", ") || "none"}
                          </small>
                          <small>{locatorSummary(item.locator)}</small>
                          <p>{item.preview}</p>
                        </div>
                      </li>
                    ))}
                  </ul>
                ) : (
                  <p>No context items available for this prompt pack.</p>
                )}
              </>
            ) : (
              <p>No Scholar Chat prompt pack preview loaded yet.</p>
            )
          ) : (
            <p>No Scholar Chat prompt pack preview loaded yet.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>OpenAlex metadata provider request preview</h3>
          <p class="muted">
            Read-only preview. Only provider request preview is wired. No OpenAlex execution. No cache/write gate execution. No write button. No Evidence Pack, citations, Literature Review, or answer generation. No network call from this preview.
          </p>
          <p class="muted">
            Execution requires explicit advanced consent; cache/write remains diagnostics-only. This uses the current Scholar Chat prompt as the query preview.
          </p>
          <div class="hero-actions">
            <button onClick={previewScholarChatScientificMetadataProviderRequest} disabled={scholarChatScientificMetadataProviderRequestLoading()}>
              {scholarChatScientificMetadataProviderRequestLoading() ? "Previewing..." : "Preview OpenAlex metadata provider request"}
            </button>
          </div>
          {scholarChatScientificMetadataProviderRequestValidationError() && <p class="error">{scholarChatScientificMetadataProviderRequestValidationError()}</p>}
          {scholarChatScientificMetadataProviderRequestError() && <p class="error">{scholarChatScientificMetadataProviderRequestError()}</p>}
          {scholarChatScientificMetadataProviderRequestLoading() ? (
            <p>Previewing OpenAlex metadata provider request...</p>
          ) : scholarChatScientificMetadataProviderRequestHasRun() ? (
            scientificMetadataProviderRequestPreview ? (
              <>
                {renderMetricGrid([
                  { label: "Status", value: formatSnakeCaseLabel(scientificMetadataProviderRequestPreview.status) },
                  { label: "Query plan status", value: formatSnakeCaseLabel(scientificMetadataProviderRequestPreview.query_plan_status) },
                  { label: "Query plan strategy", value: formatSnakeCaseLabel(scientificMetadataProviderRequestPreview.query_plan_strategy) },
                  { label: "Provider request strategy", value: formatSnakeCaseLabel(scientificMetadataProviderRequestPreview.provider_request_strategy) },
                  { label: "Selected providers", value: scientificMetadataProviderRequestPreview.selected_provider_count },
                  { label: "Execution requested", value: scientificMetadataProviderRequestPreview.execution_requested ? "yes" : "no" },
                  { label: "Allow network", value: scientificMetadataProviderRequestPreview.allow_network ? "yes" : "no" },
                  { label: "Allow provider terms unreviewed", value: scientificMetadataProviderRequestPreview.allow_provider_terms_unreviewed ? "yes" : "no" },
                  { label: "Allow metadata record write", value: scientificMetadataProviderRequestPreview.allow_metadata_record_write ? "yes" : "no" },
                ])}
                <p><strong>Query:</strong> {scientificMetadataProviderRequestPreview.normalized_query}</p>
                <p><strong>Query goal:</strong> {scientificMetadataProviderRequestPreview.normalized_query_goal ?? "none"}</p>
                <p><strong>Normalized mode:</strong> {scientificMetadataProviderRequestPreview.normalized_mode ?? "none"}</p>
                <p><strong>Selected provider IDs:</strong> {scientificMetadataProviderRequestPreview.selected_provider_ids.length > 0 ? scientificMetadataProviderRequestPreview.selected_provider_ids.join(", ") : "none"}</p>
                <p><strong>Public providers:</strong> {scientificMetadataProviderRequestPreview.public_metadata_provider_ids.length > 0 ? scientificMetadataProviderRequestPreview.public_metadata_provider_ids.join(", ") : "none"}</p>
                <p><strong>Institutional providers:</strong> {scientificMetadataProviderRequestPreview.institutional_boundary_provider_ids.length > 0 ? scientificMetadataProviderRequestPreview.institutional_boundary_provider_ids.join(", ") : "none"}</p>
                <p><strong>Normalized provider override:</strong> {scientificMetadataProviderRequestPreview.normalized_provider_override && scientificMetadataProviderRequestPreview.normalized_provider_override.length > 0 ? scientificMetadataProviderRequestPreview.normalized_provider_override.join(", ") : "none"}</p>
                <p class="muted">{scientificMetadataProviderRequestPreview.summary}</p>
                {renderMetricGrid([...OPENALEX_READONLY_PANEL_BOUNDARY_CHECKLIST])}
                <div class="contract-meta">
                  <div><span>Preview only</span><strong>{scientificMetadataProviderRequestPreview.preview_only ? "yes" : "no"}</strong></div>
                  <div><span>Metadata provider request preview only</span><strong>{scientificMetadataProviderRequestPreview.metadata_provider_request_preview_only ? "yes" : "no"}</strong></div>
                  <div><span>Dry run only</span><strong>{scientificMetadataProviderRequestPreview.dry_run_only ? "yes" : "no"}</strong></div>
                  <div><span>Execution disabled</span><strong>{scientificMetadataProviderRequestPreview.execution_disabled ? "yes" : "no"}</strong></div>
                  <div><span>No network call</span><strong>{scientificMetadataProviderRequestPreview.no_network_call ? "yes" : "no"}</strong></div>
                  <div><span>No HTTP client</span><strong>{scientificMetadataProviderRequestPreview.no_http_client ? "yes" : "no"}</strong></div>
                  <div><span>No API key read</span><strong>{scientificMetadataProviderRequestPreview.no_api_key_read ? "yes" : "no"}</strong></div>
                  <div><span>No environment read</span><strong>{scientificMetadataProviderRequestPreview.no_environment_read ? "yes" : "no"}</strong></div>
                  <div><span>No scraping</span><strong>{scientificMetadataProviderRequestPreview.no_scraping ? "yes" : "no"}</strong></div>
                  <div><span>No connector call</span><strong>{scientificMetadataProviderRequestPreview.no_connector_call ? "yes" : "no"}</strong></div>
                  <div><span>No source import</span><strong>{scientificMetadataProviderRequestPreview.no_source_import ? "yes" : "no"}</strong></div>
                  <div><span>No metadata write</span><strong>{scientificMetadataProviderRequestPreview.no_metadata_record_write ? "yes" : "no"}</strong></div>
                  <div><span>No metadata persistence</span><strong>{scientificMetadataProviderRequestPreview.no_metadata_persistence ? "yes" : "no"}</strong></div>
                  <div><span>No retrieval execution</span><strong>{scientificMetadataProviderRequestPreview.no_retrieval_execution ? "yes" : "no"}</strong></div>
                  <div><span>No model loading</span><strong>{scientificMetadataProviderRequestPreview.no_model_loading ? "yes" : "no"}</strong></div>
                  <div><span>No runtime inference</span><strong>{scientificMetadataProviderRequestPreview.no_runtime_inference ? "yes" : "no"}</strong></div>
                  <div><span>No LLM call</span><strong>{scientificMetadataProviderRequestPreview.no_llm_call ? "yes" : "no"}</strong></div>
                  <div><span>No answer generated</span><strong>{scientificMetadataProviderRequestPreview.no_answer_generated ? "yes" : "no"}</strong></div>
                  <div><span>No Literature Review created</span><strong>{scientificMetadataProviderRequestPreview.no_literature_review_created ? "yes" : "no"}</strong></div>
                  <div><span>No Evidence Pack created</span><strong>{scientificMetadataProviderRequestPreview.no_evidence_pack_created ? "yes" : "no"}</strong></div>
                  <div><span>No artifact write</span><strong>{scientificMetadataProviderRequestPreview.no_artifact_write ? "yes" : "no"}</strong></div>
                  <div><span>No registry status change</span><strong>{scientificMetadataProviderRequestPreview.no_registry_status_change ? "yes" : "no"}</strong></div>
                  <div><span>No audit write</span><strong>{scientificMetadataProviderRequestPreview.no_audit_write ? "yes" : "no"}</strong></div>
                </div>
                {scientificMetadataProviderRequestPreview.unknown_provider_ids.length > 0 ? (
                  <div class="warning-box">
                    <h4>Unknown provider IDs</h4>
                    <ul>
                      {scientificMetadataProviderRequestPreview.unknown_provider_ids.map((providerId) => (
                        <li>{providerId}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No unknown provider IDs.</p>
                )}
                {scientificMetadataProviderRequestPreview.blockers.length > 0 ? (
                  <div class="warning-box">
                    <h4>Blockers</h4>
                    <ul>
                      {scientificMetadataProviderRequestPreview.blockers.map((blocker) => (
                        <li>{blocker}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No provider request blockers.</p>
                )}
                {scientificMetadataProviderRequestPreview.warnings.length > 0 ? (
                  <div class="warning-box">
                    <h4>Warnings</h4>
                    <ul>
                      {scientificMetadataProviderRequestPreview.warnings.map((warning) => (
                        <li>{warning}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No provider request warnings.</p>
                )}
                {scientificMetadataProviderRequestPreview.next_required_actions.length > 0 ? (
                  <div class="warning-box">
                    <h4>Next required actions</h4>
                    <ul>
                      {scientificMetadataProviderRequestPreview.next_required_actions.map((action) => (
                        <li>{action}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No next required actions.</p>
                )}
              </>
            ) : (
              <p>No OpenAlex metadata provider request preview loaded yet.</p>
            )
          ) : (
            <p>No OpenAlex metadata provider request preview loaded yet.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>Answer readiness</h3>
          <p class="muted">
            Read-only readiness gate for the future Scholar Chat draft path. It uses the current request preview, source context, and local runtime configuration, but it never generates an answer or runs the model.
          </p>
          <div class="form-row">
            <label class="inline-field">
              Allow future model execution
              <input
                type="checkbox"
                checked={scholarChatAnswerReadinessAllowModelExecution()}
                onChange={(event) => {
                  setScholarChatAnswerReadinessAllowModelExecution(event.currentTarget.checked);
                  clearScholarChatAnswerReadinessPreview();
                  clearScholarChatDraftInferencePreview();
                }}
              />
            </label>
          </div>
          <p class="muted">{scholarChatSelectedSourceIdsSummary()}</p>
          <div class="hero-actions">
            <button onClick={previewScholarChatAnswerReadiness} disabled={scholarChatAnswerReadinessLoading()}>
              {scholarChatAnswerReadinessLoading() ? "Previewing..." : "Preview answer readiness"}
            </button>
          </div>
          <p class="muted">No answer was generated. No model was executed. No Evidence Pack or final answer was created.</p>
          {scholarChatAnswerReadinessValidationError() && <p class="error">{scholarChatAnswerReadinessValidationError()}</p>}
          {scholarChatAnswerReadinessError() && <p class="error">{scholarChatAnswerReadinessError()}</p>}
          {scholarChatAnswerReadinessLoading() ? (
            <p>Previewing answer readiness...</p>
          ) : scholarChatAnswerReadinessHasRun() ? (
            scholarChatAnswerReadinessPreview() ? (
              <>
                {renderMetricGrid([
                  { label: "Status", value: formatSnakeCaseLabel(scholarChatAnswerReadinessPreview()!.status) },
                  { label: "Future output classification", value: formatSnakeCaseLabel(scholarChatAnswerReadinessPreview()!.future_output_classification) },
                  { label: "Mode", value: formatSnakeCaseLabel(scholarChatAnswerReadinessPreview()!.mode) },
                  { label: "Grounding policy", value: formatSnakeCaseLabel(scholarChatAnswerReadinessPreview()!.grounding_policy) },
                  { label: "Selected sources", value: scholarChatAnswerReadinessPreview()!.selected_source_count },
                  { label: "Retrieval candidates", value: scholarChatAnswerReadinessPreview()!.retrieval_candidate_count },
                  { label: "Evidence candidates", value: scholarChatAnswerReadinessPreview()!.evidence_candidate_count },
                  { label: "Prompt pack ready", value: scholarChatAnswerReadinessPreview()!.prompt_pack_ready ? "yes" : "no" },
                  { label: "Runtime health", value: formatSnakeCaseLabel(scholarChatAnswerReadinessPreview()!.runtime_health_status) },
                  { label: "Invocation plan", value: formatSnakeCaseLabel(scholarChatAnswerReadinessPreview()!.invocation_plan_status) },
                  { label: "Allow future model execution", value: scholarChatAnswerReadinessPreview()!.allow_model_execution ? "yes" : "no" },
                  { label: "Would generate answer now", value: scholarChatAnswerReadinessPreview()!.would_generate_answer_now ? "yes" : "no" },
                  { label: "Would build Evidence Pack now", value: scholarChatAnswerReadinessPreview()!.would_build_evidence_pack_now ? "yes" : "no" },
                  { label: "Would create final answer now", value: scholarChatAnswerReadinessPreview()!.would_create_final_answer_now ? "yes" : "no" },
                ])}
                <p><strong>Prompt:</strong> {scholarChatAnswerReadinessPreview()!.normalized_prompt}</p>
                {scholarChatAnswerReadinessPreview()!.blockers.length > 0 ? (
                  <div class="warning-box">
                    <h4>Blockers</h4>
                    <ul>
                      {scholarChatAnswerReadinessPreview()!.blockers.map((blocker) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(blocker.kind)}</strong>
                          <div>{blocker.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No readiness blockers.</p>
                )}
                {scholarChatAnswerReadinessPreview()!.warnings.length > 0 ? (
                  <div class="warning-box">
                    <h4>Warnings</h4>
                    <ul>
                      {scholarChatAnswerReadinessPreview()!.warnings.map((warning) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(warning.kind)}</strong>
                          <div>{warning.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No readiness warnings.</p>
                )}
                {scholarChatAnswerReadinessPreview()!.next_required_actions.length > 0 ? (
                  <div class="warning-box">
                    <h4>Next required actions</h4>
                    <ul>
                      {scholarChatAnswerReadinessPreview()!.next_required_actions.map((action) => (
                        <li>{action}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No next required actions.</p>
                )}
              </>
            ) : (
              <p>No answer readiness preview loaded yet.</p>
            )
          ) : (
            <p>No answer readiness preview loaded yet.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>Draft inference preview</h3>
          <p class="muted">
            Read-only draft-only preview of the future Scholar Chat draft path. It may run the local model only when consent and readiness allow it, but it is not a Scholar Chat answer, grounded answer, or final answer, and nothing is persisted.
          </p>
          <p class="muted">Uses the same execution consent toggle shown in Answer readiness.</p>
          <p class="muted">{scholarChatSelectedSourceIdsSummary()}</p>
          <div class="hero-actions">
            <button onClick={previewScholarChatDraftInference} disabled={scholarChatDraftInferenceLoading()}>
              {scholarChatDraftInferenceLoading() ? "Previewing..." : "Run draft inference preview"}
            </button>
          </div>
          <p class="muted">No Scholar Chat answer is generated. No Evidence Pack is built. No final answer is created.</p>
          {scholarChatDraftInferenceValidationError() && <p class="error">{scholarChatDraftInferenceValidationError()}</p>}
          {scholarChatDraftInferenceError() && <p class="error">{scholarChatDraftInferenceError()}</p>}
          {scholarChatDraftInferenceLoading() ? (
            <p>Running draft inference preview...</p>
          ) : scholarChatDraftInferenceHasRun() ? (
            scholarChatDraftInferencePreview() ? (
              <>
                {renderMetricGrid([
                  { label: "Status", value: formatSnakeCaseLabel(scholarChatDraftInferencePreview()!.status) },
                  { label: "Output classification", value: formatSnakeCaseLabel(scholarChatDraftInferencePreview()!.output_classification) },
                  { label: "Mode", value: formatSnakeCaseLabel(scholarChatDraftInferencePreview()!.mode) },
                  { label: "Grounding policy", value: formatSnakeCaseLabel(scholarChatDraftInferencePreview()!.grounding_policy) },
                  { label: "Selected sources", value: scholarChatDraftInferencePreview()!.selected_source_count },
                  { label: "Retrieval candidates", value: scholarChatDraftInferencePreview()!.retrieval_candidate_count },
                  { label: "Evidence candidates", value: scholarChatDraftInferencePreview()!.evidence_candidate_count },
                  { label: "Prompt sections", value: scholarChatDraftInferencePreview()!.prompt_pack_section_count },
                  { label: "Prompt chars", value: scholarChatDraftInferencePreview()!.prompt_char_count },
                  { label: "Runtime health", value: formatSnakeCaseLabel(scholarChatDraftInferencePreview()!.runtime_health_status) },
                  { label: "Invocation plan", value: formatSnakeCaseLabel(scholarChatDraftInferencePreview()!.invocation_plan_status) },
                  { label: "Allow future model execution", value: scholarChatDraftInferencePreview()!.allow_model_execution ? "yes" : "no" },
                  { label: "Execution attempted", value: scholarChatDraftInferencePreview()!.execution_attempted ? "yes" : "no" },
                  { label: "Duration ms", value: scholarChatDraftInferencePreview()!.duration_ms },
                  { label: "Exit code", value: scholarChatDraftInferencePreview()!.exit_code ?? "missing" },
                  { label: "Draft only", value: scholarChatDraftInferencePreview()!.draft_only ? "yes" : "no" },
                  { label: "Preview only", value: scholarChatDraftInferencePreview()!.preview_only ? "yes" : "no" },
                  { label: "Not final answer", value: scholarChatDraftInferencePreview()!.not_final_answer ? "yes" : "no" },
                  { label: "Not grounded answer", value: scholarChatDraftInferencePreview()!.not_grounded_answer ? "yes" : "no" },
                  { label: "No answer artifact created", value: scholarChatDraftInferencePreview()!.no_answer_artifact_created ? "yes" : "no" },
                  { label: "No Evidence Pack built", value: scholarChatDraftInferencePreview()!.no_evidence_pack_built ? "yes" : "no" },
                  { label: "No persistence", value: scholarChatDraftInferencePreview()!.no_persistence ? "yes" : "no" },
                ])}
                <p><strong>Prompt:</strong> {scholarChatDraftInferencePreview()!.normalized_prompt}</p>
                <div class="contract-meta">
                  <div><span>Model file</span><strong>{scholarChatDraftInferencePreview()!.safe_model_file_name ?? "not configured"}</strong></div>
                  <div><span>Executable file</span><strong>{scholarChatDraftInferencePreview()!.safe_executable_file_name ?? "not configured"}</strong></div>
                </div>
                {scholarChatDraftInferencePreview()!.stdout_preview ? (
                  <div class="artifact-overview">
                    <h4>Runtime stdout diagnostic</h4>
                    <pre>{scholarChatDraftInferencePreview()!.stdout_preview}</pre>
                    <div class="hero-actions">
                      <button
                        onClick={() => {
                          setScholarChatDraftGroundingInspectionDraftText(scholarChatDraftInferencePreview()!.stdout_preview);
                          clearScholarChatDraftGroundingInspectionPreview();
                        }}
                      >
                        Use stdout for grounding inspection
                      </button>
                    </div>
                    <p class="muted">Runtime stdout remains diagnostic draft text, not a verified answer.</p>
                  </div>
                ) : (
                  <p>No runtime stdout preview.</p>
                )}
                {scholarChatDraftInferencePreview()!.stderr_preview ? (
                  <div class="artifact-overview">
                    <h4>Runtime stderr diagnostic</h4>
                    <pre>{scholarChatDraftInferencePreview()!.stderr_preview}</pre>
                  </div>
                ) : (
                  <p>No runtime stderr preview.</p>
                )}
                {scholarChatDraftInferencePreview()!.blockers.length > 0 ? (
                  <div class="warning-box">
                    <h4>Blockers</h4>
                    <ul>
                      {scholarChatDraftInferencePreview()!.blockers.map((blocker) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(blocker.kind)}</strong>
                          <div>{blocker.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No draft inference blockers.</p>
                )}
                {scholarChatDraftInferencePreview()!.warnings.length > 0 ? (
                  <div class="warning-box">
                    <h4>Warnings</h4>
                    <ul>
                      {scholarChatDraftInferencePreview()!.warnings.map((warning) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(warning.kind)}</strong>
                          <div>{warning.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No draft inference warnings.</p>
                )}
              </>
            ) : (
              <p>No Scholar Chat draft inference preview loaded yet.</p>
            )
          ) : (
            <p>No Scholar Chat draft inference preview loaded yet.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>Draft grounding inspection</h3>
          <p class="muted">
            Read-only diagnostic inspection of draft text against local evidence candidates only. It does not read runtime stdout/stderr, does not create a grounded answer, and does not persist anything.
          </p>
          <p class="muted">{scholarChatSelectedSourceIdsSummary()}</p>
          <label>
            Draft text preview
            <textarea
              rows={5}
              value={scholarChatDraftGroundingInspectionDraftText()}
              onInput={(event) => {
                setScholarChatDraftGroundingInspectionDraftText(event.currentTarget.value);
                clearScholarChatDraftGroundingInspectionPreview();
                clearScholarChatGroundedAnswerBuildIntentPreview();
              }}
              placeholder="Paste generated_text_preview or draft text here."
            />
          </label>
          <div class="hero-actions">
            <button onClick={previewScholarChatDraftGroundingInspection} disabled={scholarChatDraftGroundingInspectionLoading()}>
              {scholarChatDraftGroundingInspectionLoading() ? "Inspecting..." : "Inspect draft grounding"}
            </button>
          </div>
          <p class="muted">Diagnostic only - no answer was generated. No grounded answer was created. No Evidence Pack was built. No final answer was created. "Supported" means deterministic local lexical overlap, not verified truth. Weak and unsupported items need review.</p>
          {scholarChatDraftGroundingInspectionValidationError() && <p class="error">{scholarChatDraftGroundingInspectionValidationError()}</p>}
          {scholarChatDraftGroundingInspectionError() && <p class="error">{scholarChatDraftGroundingInspectionError()}</p>}
          {scholarChatDraftGroundingInspectionLoading() ? (
            <p>Inspecting draft grounding...</p>
          ) : scholarChatDraftGroundingInspectionHasRun() ? (
            scholarChatDraftGroundingInspectionPreview() ? (
              <>
                {renderMetricGrid([
                  { label: "Status", value: formatSnakeCaseLabel(scholarChatDraftGroundingInspectionPreview()!.status) },
                  { label: "Draft chars", value: scholarChatDraftGroundingInspectionPreview()!.draft_char_count },
                  { label: "Selected sources", value: scholarChatDraftGroundingInspectionPreview()!.selected_source_count },
                  { label: "Evidence candidates", value: scholarChatDraftGroundingInspectionPreview()!.evidence_candidate_count },
                  { label: "Inspected items", value: scholarChatDraftGroundingInspectionPreview()!.inspected_item_count },
                  { label: "Supported items (local overlap)", value: scholarChatDraftGroundingInspectionPreview()!.supported_item_count },
                  { label: "Weakly supported items", value: scholarChatDraftGroundingInspectionPreview()!.weakly_supported_item_count },
                  { label: "Unsupported items", value: scholarChatDraftGroundingInspectionPreview()!.unsupported_item_count },
                ])}
                {scholarChatDraftGroundingInspectionPreview()!.blockers.length > 0 ? (
                  <div class="warning-box">
                    <h4>Blockers</h4>
                    <ul>
                      {scholarChatDraftGroundingInspectionPreview()!.blockers.map((blocker) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(blocker.kind)}</strong>
                          <div>{blocker.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No draft grounding blockers.</p>
                )}
                {scholarChatDraftGroundingInspectionPreview()!.warnings.length > 0 ? (
                  <div class="warning-box">
                    <h4>Warnings</h4>
                    <ul>
                      {scholarChatDraftGroundingInspectionPreview()!.warnings.map((warning) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(warning.kind)}</strong>
                          <div>{warning.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No draft grounding warnings.</p>
                )}
                {scholarChatDraftGroundingInspectionPreview()!.items.length > 0 ? (
                  <ul class="final-answer-list-items">
                    {scholarChatDraftGroundingInspectionPreview()!.items.map((item) => (
                      <li>
                        <div class="final-answer-list-item">
                          <span>
                            <strong>Item {item.item_index + 1}</strong> {compactTextPreview(item.text_preview, 140)}
                          </span>
                          <small>
                            Status: {formatSnakeCaseLabel(item.support_status)} | matched={item.matched_evidence_count} | sources={item.source_ids.join(", ") || "none"}
                          </small>
                        </div>
                        {item.locator_previews.length > 0 ? (
                          <small>{item.locator_previews.join(" | ")}</small>
                        ) : (
                          <small>No locator previews.</small>
                        )}
                      </li>
                    ))}
                  </ul>
                ) : (
                  <p>No draft items were inspected yet.</p>
                )}
              </>
            ) : (
              <p>No draft grounding inspection preview loaded yet.</p>
            )
          ) : (
            <p>No draft grounding inspection preview loaded yet.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>Grounded draft readiness</h3>
          <p class="muted">
            Read-only readiness preview for the current draft grounding inspection text. It summarizes whether the draft looks ready for a future grounded-answer path, but it is not a verified grounded answer, final answer, or Evidence Pack.
          </p>
          <p class="muted">{scholarChatSelectedSourceIdsSummary()}</p>
          <p class="muted">Uses the current Scholar Chat request and the draft text from the inspection card above.</p>
          <div class="hero-actions">
            <button onClick={previewScholarChatGroundedDraftReadiness} disabled={scholarChatGroundedDraftReadinessLoading()}>
              {scholarChatGroundedDraftReadinessLoading() ? "Previewing..." : "Preview grounded draft readiness"}
            </button>
          </div>
          <p class="muted">Readiness only - not a verified grounded answer. No Evidence Pack, grounded answer, or final answer was created.</p>
          {scholarChatGroundedDraftReadinessValidationError() && <p class="error">{scholarChatGroundedDraftReadinessValidationError()}</p>}
          {scholarChatGroundedDraftReadinessError() && <p class="error">{scholarChatGroundedDraftReadinessError()}</p>}
          {scholarChatGroundedDraftReadinessLoading() ? (
            <p>Previewing grounded draft readiness...</p>
          ) : scholarChatGroundedDraftReadinessHasRun() ? (
            scholarChatGroundedDraftReadinessPreview() ? (
              <>
                {renderMetricGrid([
                  { label: "Status", value: formatSnakeCaseLabel(scholarChatGroundedDraftReadinessPreview()!.status) },
                  { label: "Inspection status", value: formatSnakeCaseLabel(scholarChatGroundedDraftReadinessPreview()!.inspection_status) },
                  { label: "Selected sources", value: scholarChatGroundedDraftReadinessPreview()!.selected_source_count },
                  { label: "Evidence candidates", value: scholarChatGroundedDraftReadinessPreview()!.evidence_candidate_count },
                  { label: "Inspected items", value: scholarChatGroundedDraftReadinessPreview()!.inspected_item_count },
                  { label: "Supported items", value: scholarChatGroundedDraftReadinessPreview()!.supported_item_count },
                  { label: "Weakly supported items", value: scholarChatGroundedDraftReadinessPreview()!.weakly_supported_item_count },
                  { label: "Unsupported items", value: scholarChatGroundedDraftReadinessPreview()!.unsupported_item_count },
                ])}
                <p><strong>Prompt:</strong> {scholarChatGroundedDraftReadinessPreview()!.normalized_prompt}</p>
                <p>{scholarChatGroundedDraftReadinessPreview()!.summary}</p>
                <div class="contract-meta">
                  <div><span>Preview only</span><strong>{scholarChatGroundedDraftReadinessPreview()!.preview_only ? "yes" : "no"}</strong></div>
                  <div><span>Not grounded answer</span><strong>{scholarChatGroundedDraftReadinessPreview()!.not_grounded_answer ? "yes" : "no"}</strong></div>
                  <div><span>Not final answer</span><strong>{scholarChatGroundedDraftReadinessPreview()!.not_final_answer ? "yes" : "no"}</strong></div>
                  <div><span>No answer artifact created</span><strong>{scholarChatGroundedDraftReadinessPreview()!.no_answer_artifact_created ? "yes" : "no"}</strong></div>
                  <div><span>No Evidence Pack built</span><strong>{scholarChatGroundedDraftReadinessPreview()!.no_evidence_pack_built ? "yes" : "no"}</strong></div>
                  <div><span>No persistence</span><strong>{scholarChatGroundedDraftReadinessPreview()!.no_persistence ? "yes" : "no"}</strong></div>
                  <div><span>No LLM call</span><strong>{scholarChatGroundedDraftReadinessPreview()!.no_llm_call ? "yes" : "no"}</strong></div>
                  <div><span>No runtime execution</span><strong>{scholarChatGroundedDraftReadinessPreview()!.no_runtime_execution ? "yes" : "no"}</strong></div>
                </div>
                {scholarChatGroundedDraftReadinessPreview()!.blockers.length > 0 ? (
                  <div class="warning-box">
                    <h4>Blockers</h4>
                    <ul>
                      {scholarChatGroundedDraftReadinessPreview()!.blockers.map((blocker) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(blocker.kind)}</strong>
                          <div>{blocker.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No grounded draft readiness blockers.</p>
                )}
                {scholarChatGroundedDraftReadinessPreview()!.warnings.length > 0 ? (
                  <div class="warning-box">
                    <h4>Warnings</h4>
                    <ul>
                      {scholarChatGroundedDraftReadinessPreview()!.warnings.map((warning) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(warning.kind)}</strong>
                          <div>{warning.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No grounded draft readiness warnings.</p>
                )}
                {scholarChatGroundedDraftReadinessPreview()!.next_required_actions.length > 0 ? (
                  <div class="warning-box">
                    <h4>Next required actions</h4>
                    <ul>
                      {scholarChatGroundedDraftReadinessPreview()!.next_required_actions.map((action) => (
                        <li>{action}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No next required actions.</p>
                )}
              </>
            ) : (
              <p>No grounded draft readiness preview loaded yet.</p>
            )
          ) : (
            <p>No grounded draft readiness preview loaded yet.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>Grounded answer build plan</h3>
          <p class="muted">
            Read-only plan preview for the current draft grounding inspection text. It summarizes what would still be needed before a future GroundedAnswer could be written, but it is not a GroundedAnswer, final answer, Evidence Pack, or persisted artifact.
          </p>
          <p class="muted">{scholarChatSelectedSourceIdsSummary()}</p>
          <p class="muted">Uses the current Scholar Chat request and the draft text from the inspection card above.</p>
          <div class="hero-actions">
            <button onClick={previewScholarChatGroundedAnswerBuildPlan} disabled={scholarChatGroundedAnswerBuildPlanLoading()}>
              {scholarChatGroundedAnswerBuildPlanLoading() ? "Previewing..." : "Preview grounded answer build plan"}
            </button>
          </div>
          <p class="muted">Plan only - not a GroundedAnswer. No Evidence Pack, final answer, persistence, or runtime execution was created.</p>
          {scholarChatGroundedAnswerBuildPlanValidationError() && <p class="error">{scholarChatGroundedAnswerBuildPlanValidationError()}</p>}
          {scholarChatGroundedAnswerBuildPlanError() && <p class="error">{scholarChatGroundedAnswerBuildPlanError()}</p>}
          {scholarChatGroundedAnswerBuildPlanLoading() ? (
            <p>Previewing grounded answer build plan...</p>
          ) : scholarChatGroundedAnswerBuildPlanHasRun() ? (
            scholarChatGroundedAnswerBuildPlanPreview() ? (
              <>
                {renderMetricGrid([
                  { label: "Status", value: formatSnakeCaseLabel(scholarChatGroundedAnswerBuildPlanPreview()!.status) },
                  { label: "Readiness status", value: formatSnakeCaseLabel(scholarChatGroundedAnswerBuildPlanPreview()!.readiness_status) },
                  { label: "Selected sources", value: scholarChatGroundedAnswerBuildPlanPreview()!.selected_source_count },
                  { label: "Evidence candidates", value: scholarChatGroundedAnswerBuildPlanPreview()!.evidence_candidate_count },
                  { label: "Inspected items", value: scholarChatGroundedAnswerBuildPlanPreview()!.inspected_item_count },
                  { label: "Supported items", value: scholarChatGroundedAnswerBuildPlanPreview()!.supported_item_count },
                  { label: "Weakly supported items", value: scholarChatGroundedAnswerBuildPlanPreview()!.weakly_supported_item_count },
                  { label: "Unsupported items", value: scholarChatGroundedAnswerBuildPlanPreview()!.unsupported_item_count },
                  { label: "Planned steps", value: scholarChatGroundedAnswerBuildPlanPreview()!.planned_steps.length },
                ])}
                <p><strong>Prompt:</strong> {scholarChatGroundedAnswerBuildPlanPreview()!.normalized_prompt}</p>
                <p>{scholarChatGroundedAnswerBuildPlanPreview()!.summary}</p>
                <div class="contract-meta">
                  <div><span>Preview only</span><strong>{scholarChatGroundedAnswerBuildPlanPreview()!.preview_only ? "yes" : "no"}</strong></div>
                  <div><span>Not answer draft</span><strong>{scholarChatGroundedAnswerBuildPlanPreview()!.not_answer_draft ? "yes" : "no"}</strong></div>
                  <div><span>Not grounded answer</span><strong>{scholarChatGroundedAnswerBuildPlanPreview()!.not_grounded_answer ? "yes" : "no"}</strong></div>
                  <div><span>Not final answer</span><strong>{scholarChatGroundedAnswerBuildPlanPreview()!.not_final_answer ? "yes" : "no"}</strong></div>
                  <div><span>No answer artifact created</span><strong>{scholarChatGroundedAnswerBuildPlanPreview()!.no_answer_artifact_created ? "yes" : "no"}</strong></div>
                  <div><span>No Evidence Pack built</span><strong>{scholarChatGroundedAnswerBuildPlanPreview()!.no_evidence_pack_built ? "yes" : "no"}</strong></div>
                  <div><span>No persistence</span><strong>{scholarChatGroundedAnswerBuildPlanPreview()!.no_persistence ? "yes" : "no"}</strong></div>
                  <div><span>No LLM call</span><strong>{scholarChatGroundedAnswerBuildPlanPreview()!.no_llm_call ? "yes" : "no"}</strong></div>
                  <div><span>No runtime execution</span><strong>{scholarChatGroundedAnswerBuildPlanPreview()!.no_runtime_execution ? "yes" : "no"}</strong></div>
                </div>
                {scholarChatGroundedAnswerBuildPlanPreview()!.planned_steps.length > 0 ? (
                  <div class="warning-box">
                    <h4>Planned steps</h4>
                    <ol>
                      {scholarChatGroundedAnswerBuildPlanPreview()!.planned_steps.map((step) => (
                        <li>{step}</li>
                      ))}
                    </ol>
                  </div>
                ) : (
                  <p>No planned steps.</p>
                )}
                {scholarChatGroundedAnswerBuildPlanPreview()!.blockers.length > 0 ? (
                  <div class="warning-box">
                    <h4>Blockers</h4>
                    <ul>
                      {scholarChatGroundedAnswerBuildPlanPreview()!.blockers.map((blocker) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(blocker.kind)}</strong>
                          <div>{blocker.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No grounded answer build plan blockers.</p>
                )}
                {scholarChatGroundedAnswerBuildPlanPreview()!.warnings.length > 0 ? (
                  <div class="warning-box">
                    <h4>Warnings</h4>
                    <ul>
                      {scholarChatGroundedAnswerBuildPlanPreview()!.warnings.map((warning) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(warning.kind)}</strong>
                          <div>{warning.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No grounded answer build plan warnings.</p>
                )}
                {scholarChatGroundedAnswerBuildPlanPreview()!.next_required_actions.length > 0 ? (
                  <div class="warning-box">
                    <h4>Next required actions</h4>
                    <ul>
                      {scholarChatGroundedAnswerBuildPlanPreview()!.next_required_actions.map((action) => (
                        <li>{action}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No next required actions.</p>
                )}
              </>
            ) : (
              <p>No grounded answer build plan preview loaded yet.</p>
            )
          ) : (
            <p>No grounded answer build plan preview loaded yet.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>Grounded answer candidate</h3>
          <p class="muted">
            Read-only candidate preview for the current draft grounding inspection text. It shows what a future GroundedAnswer could look like later, but it is candidate only - not a GroundedAnswer, final answer, Evidence Pack, persistence, runtime execution, or LLM call.
          </p>
          <p class="muted">{scholarChatSelectedSourceIdsSummary()}</p>
          <p class="muted">Uses the current Scholar Chat request and the draft text from the inspection card above.</p>
          <div class="hero-actions">
            <button onClick={previewScholarChatGroundedAnswerCandidate} disabled={scholarChatGroundedAnswerCandidateLoading()}>
              {scholarChatGroundedAnswerCandidateLoading() ? "Previewing..." : "Preview grounded answer candidate"}
            </button>
          </div>
          <p class="muted">Candidate only - not a GroundedAnswer. No Evidence Pack, final answer, persistence, runtime execution, or LLM call was created.</p>
          {scholarChatGroundedAnswerCandidateValidationError() && <p class="error">{scholarChatGroundedAnswerCandidateValidationError()}</p>}
          {scholarChatGroundedAnswerCandidateError() && <p class="error">{scholarChatGroundedAnswerCandidateError()}</p>}
          {scholarChatGroundedAnswerCandidateLoading() ? (
            <p>Previewing grounded answer candidate...</p>
          ) : scholarChatGroundedAnswerCandidateHasRun() ? (
            scholarChatGroundedAnswerCandidatePreview() ? (
              <>
                {renderMetricGrid([
                  { label: "Status", value: formatSnakeCaseLabel(scholarChatGroundedAnswerCandidatePreview()!.status) },
                  { label: "Selected sources", value: scholarChatGroundedAnswerCandidatePreview()!.selected_source_count },
                  { label: "Evidence candidates", value: scholarChatGroundedAnswerCandidatePreview()!.evidence_candidate_count },
                  { label: "Inspected items", value: scholarChatGroundedAnswerCandidatePreview()!.inspected_item_count },
                  { label: "Supported items", value: scholarChatGroundedAnswerCandidatePreview()!.supported_item_count },
                  { label: "Weakly supported items", value: scholarChatGroundedAnswerCandidatePreview()!.weakly_supported_item_count },
                  { label: "Unsupported items", value: scholarChatGroundedAnswerCandidatePreview()!.unsupported_item_count },
                  { label: "Candidate statements", value: scholarChatGroundedAnswerCandidatePreview()!.candidate_statement_count },
                ])}
                <p><strong>Prompt:</strong> {scholarChatGroundedAnswerCandidatePreview()!.normalized_prompt}</p>
                <p>{scholarChatGroundedAnswerCandidatePreview()!.summary}</p>
                <div class="contract-meta">
                  <div><span>Preview only</span><strong>{scholarChatGroundedAnswerCandidatePreview()!.preview_only ? "yes" : "no"}</strong></div>
                  <div><span>Not answer draft</span><strong>{scholarChatGroundedAnswerCandidatePreview()!.not_answer_draft ? "yes" : "no"}</strong></div>
                  <div><span>Not grounded answer</span><strong>{scholarChatGroundedAnswerCandidatePreview()!.not_grounded_answer ? "yes" : "no"}</strong></div>
                  <div><span>Not final answer</span><strong>{scholarChatGroundedAnswerCandidatePreview()!.not_final_answer ? "yes" : "no"}</strong></div>
                  <div><span>No answer artifact created</span><strong>{scholarChatGroundedAnswerCandidatePreview()!.no_answer_artifact_created ? "yes" : "no"}</strong></div>
                  <div><span>No Evidence Pack built</span><strong>{scholarChatGroundedAnswerCandidatePreview()!.no_evidence_pack_built ? "yes" : "no"}</strong></div>
                  <div><span>No persistence</span><strong>{scholarChatGroundedAnswerCandidatePreview()!.no_persistence ? "yes" : "no"}</strong></div>
                  <div><span>No LLM call</span><strong>{scholarChatGroundedAnswerCandidatePreview()!.no_llm_call ? "yes" : "no"}</strong></div>
                  <div><span>No runtime execution</span><strong>{scholarChatGroundedAnswerCandidatePreview()!.no_runtime_execution ? "yes" : "no"}</strong></div>
                </div>
                {scholarChatGroundedAnswerCandidatePreview()!.candidate_items.length > 0 ? (
                  <ul class="final-answer-list-items">
                    {scholarChatGroundedAnswerCandidatePreview()!.candidate_items.map((item) => (
                      <li>
                        <div class="final-answer-list-item">
                          <span>
                            <strong>Item {item.item_index + 1}</strong> {compactTextPreview(item.statement_preview, 140)}
                          </span>
                          <small>
                            Status: {formatSnakeCaseLabel(item.support_status)} | matched={item.matched_evidence_count} | sources={item.source_ids.join(", ") || "none"}
                          </small>
                        </div>
                        {item.locator_previews.length > 0 ? (
                          <small>{item.locator_previews.join(" | ")}</small>
                        ) : (
                          <small>No locator previews.</small>
                        )}
                      </li>
                    ))}
                  </ul>
                ) : (
                  <p>
                    {scholarChatGroundedAnswerCandidatePreview()!.status === "candidate_ready_later"
                      ? "No candidate items were returned."
                      : scholarChatGroundedAnswerCandidatePreview()!.status === "needs_review"
                        ? "No candidate items yet because weakly supported or unsupported items remain."
                        : "No candidate items because grounded-answer candidate preview is blocked."}
                  </p>
                )}
                {scholarChatGroundedAnswerCandidatePreview()!.blockers.length > 0 ? (
                  <div class="warning-box">
                    <h4>Blockers</h4>
                    <ul>
                      {scholarChatGroundedAnswerCandidatePreview()!.blockers.map((blocker) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(blocker.kind)}</strong>
                          <div>{blocker.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No grounded answer candidate blockers.</p>
                )}
                {scholarChatGroundedAnswerCandidatePreview()!.warnings.length > 0 ? (
                  <div class="warning-box">
                    <h4>Warnings</h4>
                    <ul>
                      {scholarChatGroundedAnswerCandidatePreview()!.warnings.map((warning) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(warning.kind)}</strong>
                          <div>{warning.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No grounded answer candidate warnings.</p>
                )}
                {scholarChatGroundedAnswerCandidatePreview()!.next_required_actions.length > 0 ? (
                  <div class="warning-box">
                    <h4>Next required actions</h4>
                    <ul>
                      {scholarChatGroundedAnswerCandidatePreview()!.next_required_actions.map((action) => (
                        <li>{action}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No next required actions.</p>
                )}
              </>
            ) : (
              <p>No grounded answer candidate preview loaded yet.</p>
            )
          ) : (
            <p>No grounded answer candidate preview loaded yet.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>Grounded answer write eligibility</h3>
          <p class="muted">
            Eligibility only - no GroundedAnswer was written. No Evidence Pack, final answer, persistence, registry status change, audit write, runtime execution, or LLM call occurred.
          </p>
          <p class="muted">{scholarChatSelectedSourceIdsSummary()}</p>
          <p class="muted">Uses the current Scholar Chat request and the draft text from the inspection card above.</p>
          <div class="hero-actions">
            <button onClick={previewScholarChatGroundedAnswerWriteEligibility} disabled={scholarChatGroundedAnswerWriteEligibilityLoading()}>
              {scholarChatGroundedAnswerWriteEligibilityLoading() ? "Previewing..." : "Preview grounded answer write eligibility"}
            </button>
          </div>
          {scholarChatGroundedAnswerWriteEligibilityValidationError() && <p class="error">{scholarChatGroundedAnswerWriteEligibilityValidationError()}</p>}
          {scholarChatGroundedAnswerWriteEligibilityError() && <p class="error">{scholarChatGroundedAnswerWriteEligibilityError()}</p>}
          {scholarChatGroundedAnswerWriteEligibilityLoading() ? (
            <p>Previewing grounded answer write eligibility...</p>
          ) : scholarChatGroundedAnswerWriteEligibilityHasRun() ? (
            scholarChatGroundedAnswerWriteEligibilityPreview() ? (
              <>
                {renderMetricGrid([
                  { label: "Status", value: formatSnakeCaseLabel(scholarChatGroundedAnswerWriteEligibilityPreview()!.status) },
                  { label: "Candidate status", value: formatSnakeCaseLabel(scholarChatGroundedAnswerWriteEligibilityPreview()!.candidate_status) },
                  { label: "Selected sources", value: scholarChatGroundedAnswerWriteEligibilityPreview()!.selected_source_count },
                  { label: "Evidence candidates", value: scholarChatGroundedAnswerWriteEligibilityPreview()!.evidence_candidate_count },
                  { label: "Inspected items", value: scholarChatGroundedAnswerWriteEligibilityPreview()!.inspected_item_count },
                  { label: "Supported items", value: scholarChatGroundedAnswerWriteEligibilityPreview()!.supported_item_count },
                  { label: "Weakly supported items", value: scholarChatGroundedAnswerWriteEligibilityPreview()!.weakly_supported_item_count },
                  { label: "Unsupported items", value: scholarChatGroundedAnswerWriteEligibilityPreview()!.unsupported_item_count },
                  { label: "Candidate statements", value: scholarChatGroundedAnswerWriteEligibilityPreview()!.candidate_statement_count },
                ])}
                <p><strong>Prompt:</strong> {scholarChatGroundedAnswerWriteEligibilityPreview()!.normalized_prompt}</p>
                <p>{scholarChatGroundedAnswerWriteEligibilityPreview()!.summary}</p>
                <div class="contract-meta">
                  <div><span>Preview only</span><strong>{scholarChatGroundedAnswerWriteEligibilityPreview()!.preview_only ? "yes" : "no"}</strong></div>
                  <div><span>Not answer draft</span><strong>{scholarChatGroundedAnswerWriteEligibilityPreview()!.not_answer_draft ? "yes" : "no"}</strong></div>
                  <div><span>Not grounded answer</span><strong>{scholarChatGroundedAnswerWriteEligibilityPreview()!.not_grounded_answer ? "yes" : "no"}</strong></div>
                  <div><span>Not final answer</span><strong>{scholarChatGroundedAnswerWriteEligibilityPreview()!.not_final_answer ? "yes" : "no"}</strong></div>
                  <div><span>No answer artifact created</span><strong>{scholarChatGroundedAnswerWriteEligibilityPreview()!.no_answer_artifact_created ? "yes" : "no"}</strong></div>
                  <div><span>No Evidence Pack built</span><strong>{scholarChatGroundedAnswerWriteEligibilityPreview()!.no_evidence_pack_built ? "yes" : "no"}</strong></div>
                  <div><span>No persistence</span><strong>{scholarChatGroundedAnswerWriteEligibilityPreview()!.no_persistence ? "yes" : "no"}</strong></div>
                  <div><span>No LLM call</span><strong>{scholarChatGroundedAnswerWriteEligibilityPreview()!.no_llm_call ? "yes" : "no"}</strong></div>
                  <div><span>No runtime execution</span><strong>{scholarChatGroundedAnswerWriteEligibilityPreview()!.no_runtime_execution ? "yes" : "no"}</strong></div>
                  <div><span>No registry status change</span><strong>{scholarChatGroundedAnswerWriteEligibilityPreview()!.no_registry_status_change ? "yes" : "no"}</strong></div>
                  <div><span>No audit write</span><strong>{scholarChatGroundedAnswerWriteEligibilityPreview()!.no_audit_write ? "yes" : "no"}</strong></div>
                </div>
                {scholarChatGroundedAnswerWriteEligibilityPreview()!.eligibility_reasons.length > 0 ? (
                  <div class="warning-box">
                    <h4>Eligibility reasons</h4>
                    <ul>
                      {scholarChatGroundedAnswerWriteEligibilityPreview()!.eligibility_reasons.map((reason) => (
                        <li>{reason}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No eligibility reasons.</p>
                )}
                {scholarChatGroundedAnswerWriteEligibilityPreview()!.blockers.length > 0 ? (
                  <div class="warning-box">
                    <h4>Blockers</h4>
                    <ul>
                      {scholarChatGroundedAnswerWriteEligibilityPreview()!.blockers.map((blocker) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(blocker.kind)}</strong>
                          <div>{blocker.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No grounded answer write eligibility blockers.</p>
                )}
                {scholarChatGroundedAnswerWriteEligibilityPreview()!.warnings.length > 0 ? (
                  <div class="warning-box">
                    <h4>Warnings</h4>
                    <ul>
                      {scholarChatGroundedAnswerWriteEligibilityPreview()!.warnings.map((warning) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(warning.kind)}</strong>
                          <div>{warning.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No grounded answer write eligibility warnings.</p>
                )}
                {scholarChatGroundedAnswerWriteEligibilityPreview()!.next_required_actions.length > 0 ? (
                  <div class="warning-box">
                    <h4>Next required actions</h4>
                    <ul>
                      {scholarChatGroundedAnswerWriteEligibilityPreview()!.next_required_actions.map((action) => (
                        <li>{action}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No next required actions.</p>
                )}
              </>
            ) : (
              <p>No grounded answer write eligibility preview loaded yet.</p>
            )
          ) : (
            <p>No grounded answer write eligibility preview loaded yet.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>Grounded answer build intent</h3>
          <p class="muted">
            Build intent only - not an AnswerDraft, GroundedAnswer, or FinalAnswer. No Evidence Pack, persistence, registry status change, audit write, runtime execution, or LLM call occurred.
          </p>
          <p class="muted">{scholarChatSelectedSourceIdsSummary()}</p>
          <p class="muted">Uses the current Scholar Chat request and the draft text from the inspection card above. Provide an answer draft ID and explicit user intent to preview the future intent gate.</p>
          <label>
            Answer draft ID
            <input
              type="text"
              value={scholarChatGroundedAnswerBuildIntentAnswerDraftId()}
              onInput={(event) => {
                setScholarChatGroundedAnswerBuildIntentAnswerDraftId(event.currentTarget.value);
                clearScholarChatGroundedAnswerBuildIntentPreview();
              }}
              placeholder="answer draft id"
            />
          </label>
          <label>
            <input
              type="checkbox"
              checked={scholarChatGroundedAnswerBuildIntentExplicitUserIntent()}
              onChange={(event) => {
                setScholarChatGroundedAnswerBuildIntentExplicitUserIntent(event.currentTarget.checked);
                clearScholarChatGroundedAnswerBuildIntentPreview();
              }}
            />
            Explicit user intent
          </label>
          <div class="hero-actions">
            <button onClick={previewScholarChatGroundedAnswerBuildIntent} disabled={scholarChatGroundedAnswerBuildIntentLoading()}>
              {scholarChatGroundedAnswerBuildIntentLoading() ? "Previewing..." : "Preview grounded answer build intent"}
            </button>
          </div>
          {scholarChatGroundedAnswerBuildIntentValidationError() && <p class="error">{scholarChatGroundedAnswerBuildIntentValidationError()}</p>}
          {scholarChatGroundedAnswerBuildIntentError() && <p class="error">{scholarChatGroundedAnswerBuildIntentError()}</p>}
          {scholarChatGroundedAnswerBuildIntentLoading() ? (
            <p>Previewing grounded answer build intent...</p>
          ) : scholarChatGroundedAnswerBuildIntentHasRun() ? (
            scholarChatGroundedAnswerBuildIntentPreview() ? (
              <>
                {renderMetricGrid([
                  { label: "Status", value: formatSnakeCaseLabel(scholarChatGroundedAnswerBuildIntentPreview()!.status) },
                  { label: "Write eligibility", value: formatSnakeCaseLabel(scholarChatGroundedAnswerBuildIntentPreview()!.write_eligibility_status) },
                  { label: "Candidate status", value: formatSnakeCaseLabel(scholarChatGroundedAnswerBuildIntentPreview()!.candidate_status) },
                  { label: "Selected sources", value: scholarChatGroundedAnswerBuildIntentPreview()!.selected_source_count },
                  { label: "Evidence candidates", value: scholarChatGroundedAnswerBuildIntentPreview()!.evidence_candidate_count },
                  { label: "Inspected items", value: scholarChatGroundedAnswerBuildIntentPreview()!.inspected_item_count },
                  { label: "Supported items", value: scholarChatGroundedAnswerBuildIntentPreview()!.supported_item_count },
                  { label: "Weakly supported items", value: scholarChatGroundedAnswerBuildIntentPreview()!.weakly_supported_item_count },
                  { label: "Unsupported items", value: scholarChatGroundedAnswerBuildIntentPreview()!.unsupported_item_count },
                  { label: "Candidate statements", value: scholarChatGroundedAnswerBuildIntentPreview()!.candidate_statement_count },
                ])}
                <p><strong>Prompt:</strong> {scholarChatGroundedAnswerBuildIntentPreview()!.normalized_prompt}</p>
                <p>{scholarChatGroundedAnswerBuildIntentPreview()!.summary}</p>
                <div class="contract-meta">
                  <div><span>Preview only</span><strong>{scholarChatGroundedAnswerBuildIntentPreview()!.preview_only ? "yes" : "no"}</strong></div>
                  <div><span>Not answer draft</span><strong>{scholarChatGroundedAnswerBuildIntentPreview()!.not_answer_draft ? "yes" : "no"}</strong></div>
                  <div><span>Not grounded answer</span><strong>{scholarChatGroundedAnswerBuildIntentPreview()!.not_grounded_answer ? "yes" : "no"}</strong></div>
                  <div><span>Not final answer</span><strong>{scholarChatGroundedAnswerBuildIntentPreview()!.not_final_answer ? "yes" : "no"}</strong></div>
                  <div><span>No answer artifact created</span><strong>{scholarChatGroundedAnswerBuildIntentPreview()!.no_answer_artifact_created ? "yes" : "no"}</strong></div>
                  <div><span>No Evidence Pack built</span><strong>{scholarChatGroundedAnswerBuildIntentPreview()!.no_evidence_pack_built ? "yes" : "no"}</strong></div>
                  <div><span>No persistence</span><strong>{scholarChatGroundedAnswerBuildIntentPreview()!.no_persistence ? "yes" : "no"}</strong></div>
                  <div><span>No LLM call</span><strong>{scholarChatGroundedAnswerBuildIntentPreview()!.no_llm_call ? "yes" : "no"}</strong></div>
                  <div><span>No runtime execution</span><strong>{scholarChatGroundedAnswerBuildIntentPreview()!.no_runtime_execution ? "yes" : "no"}</strong></div>
                  <div><span>No registry status change</span><strong>{scholarChatGroundedAnswerBuildIntentPreview()!.no_registry_status_change ? "yes" : "no"}</strong></div>
                  <div><span>No audit write</span><strong>{scholarChatGroundedAnswerBuildIntentPreview()!.no_audit_write ? "yes" : "no"}</strong></div>
                  <div><span>No grounded answer service call</span><strong>{scholarChatGroundedAnswerBuildIntentPreview()!.no_grounded_answer_service_call ? "yes" : "no"}</strong></div>
                </div>
                {scholarChatGroundedAnswerBuildIntentPreview()!.required_inputs.length > 0 ? (
                  <div class="warning-box">
                    <h4>Required inputs</h4>
                    <ul>
                      {scholarChatGroundedAnswerBuildIntentPreview()!.required_inputs.map((input) => (
                        <li>{input}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No required inputs.</p>
                )}
                {scholarChatGroundedAnswerBuildIntentPreview()!.missing_inputs.length > 0 ? (
                  <div class="warning-box">
                    <h4>Missing inputs</h4>
                    <ul>
                      {scholarChatGroundedAnswerBuildIntentPreview()!.missing_inputs.map((input) => (
                        <li>{input}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No missing inputs.</p>
                )}
                {scholarChatGroundedAnswerBuildIntentPreview()!.intent_reasons.length > 0 ? (
                  <div class="warning-box">
                    <h4>Intent reasons</h4>
                    <ul>
                      {scholarChatGroundedAnswerBuildIntentPreview()!.intent_reasons.map((reason) => (
                        <li>{reason}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No intent reasons.</p>
                )}
                {scholarChatGroundedAnswerBuildIntentPreview()!.blockers.length > 0 ? (
                  <div class="warning-box">
                    <h4>Blockers</h4>
                    <ul>
                      {scholarChatGroundedAnswerBuildIntentPreview()!.blockers.map((blocker) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(blocker.kind)}</strong>
                          <div>{blocker.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No grounded answer build intent blockers.</p>
                )}
                {scholarChatGroundedAnswerBuildIntentPreview()!.warnings.length > 0 ? (
                  <div class="warning-box">
                    <h4>Warnings</h4>
                    <ul>
                      {scholarChatGroundedAnswerBuildIntentPreview()!.warnings.map((warning) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(warning.kind)}</strong>
                          <div>{warning.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No grounded answer build intent warnings.</p>
                )}
                {scholarChatGroundedAnswerBuildIntentPreview()!.next_required_actions.length > 0 ? (
                  <div class="warning-box">
                    <h4>Next required actions</h4>
                    <ul>
                      {scholarChatGroundedAnswerBuildIntentPreview()!.next_required_actions.map((action) => (
                        <li>{action}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No next required actions.</p>
                )}
              </>
            ) : (
              <p>No grounded answer build intent preview loaded yet.</p>
            )
          ) : (
            <p>No grounded answer build intent preview loaded yet.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>Grounded answer build request</h3>
          <p class="muted">
            Request preview only - no GroundedAnswer service call was made. No answer artifact, Evidence Pack, final answer, persistence, registry status change, audit write, runtime execution, or LLM call occurred.
          </p>
          <p class="muted">{scholarChatSelectedSourceIdsSummary()}</p>
          <p class="muted">Uses the current Scholar Chat request, the draft text from the inspection card above, the build-intent answer draft ID, and the explicit user intent checkbox.</p>
          <div class="hero-actions">
            <button onClick={previewScholarChatGroundedAnswerBuildRequest} disabled={scholarChatGroundedAnswerBuildRequestLoading()}>
              {scholarChatGroundedAnswerBuildRequestLoading() ? "Previewing..." : "Preview grounded answer build request"}
            </button>
          </div>
          {scholarChatGroundedAnswerBuildRequestValidationError() && <p class="error">{scholarChatGroundedAnswerBuildRequestValidationError()}</p>}
          {scholarChatGroundedAnswerBuildRequestError() && <p class="error">{scholarChatGroundedAnswerBuildRequestError()}</p>}
          {scholarChatGroundedAnswerBuildRequestLoading() ? (
            <p>Previewing grounded answer build request...</p>
          ) : scholarChatGroundedAnswerBuildRequestHasRun() ? (
            scholarChatGroundedAnswerBuildRequestPreview() ? (
              <>
                {renderMetricGrid([
                  { label: "Status", value: formatSnakeCaseLabel(scholarChatGroundedAnswerBuildRequestPreview()!.status) },
                  { label: "Build intent status", value: formatSnakeCaseLabel(scholarChatGroundedAnswerBuildRequestPreview()!.build_intent_status) },
                  { label: "Write eligibility status", value: formatSnakeCaseLabel(scholarChatGroundedAnswerBuildRequestPreview()!.write_eligibility_status) },
                  { label: "Candidate status", value: formatSnakeCaseLabel(scholarChatGroundedAnswerBuildRequestPreview()!.candidate_status) },
                  { label: "Selected sources", value: scholarChatGroundedAnswerBuildRequestPreview()!.selected_source_count },
                  { label: "Evidence candidates", value: scholarChatGroundedAnswerBuildRequestPreview()!.evidence_candidate_count },
                  { label: "Inspected items", value: scholarChatGroundedAnswerBuildRequestPreview()!.inspected_item_count },
                  { label: "Supported items", value: scholarChatGroundedAnswerBuildRequestPreview()!.supported_item_count },
                  { label: "Weakly supported items", value: scholarChatGroundedAnswerBuildRequestPreview()!.weakly_supported_item_count },
                  { label: "Unsupported items", value: scholarChatGroundedAnswerBuildRequestPreview()!.unsupported_item_count },
                  { label: "Candidate statements", value: scholarChatGroundedAnswerBuildRequestPreview()!.candidate_statement_count },
                ])}
                <p><strong>Prompt:</strong> {scholarChatGroundedAnswerBuildRequestPreview()!.normalized_prompt}</p>
                <p><strong>Answer draft ID:</strong> {scholarChatGroundedAnswerBuildRequestPreview()!.answer_draft_id ?? "none"}</p>
                <p><strong>Selected source IDs:</strong> {scholarChatGroundedAnswerBuildRequestPreview()!.selected_source_ids.length > 0 ? scholarChatGroundedAnswerBuildRequestPreview()!.selected_source_ids.join(", ") : "none"}</p>
                <p>{scholarChatGroundedAnswerBuildRequestPreview()!.summary}</p>
                <div class="contract-meta">
                  <div><span>Preview only</span><strong>{scholarChatGroundedAnswerBuildRequestPreview()!.preview_only ? "yes" : "no"}</strong></div>
                  <div><span>Not answer draft</span><strong>{scholarChatGroundedAnswerBuildRequestPreview()!.not_answer_draft ? "yes" : "no"}</strong></div>
                  <div><span>Not grounded answer</span><strong>{scholarChatGroundedAnswerBuildRequestPreview()!.not_grounded_answer ? "yes" : "no"}</strong></div>
                  <div><span>Not final answer</span><strong>{scholarChatGroundedAnswerBuildRequestPreview()!.not_final_answer ? "yes" : "no"}</strong></div>
                  <div><span>No answer artifact created</span><strong>{scholarChatGroundedAnswerBuildRequestPreview()!.no_answer_artifact_created ? "yes" : "no"}</strong></div>
                  <div><span>No Evidence Pack built</span><strong>{scholarChatGroundedAnswerBuildRequestPreview()!.no_evidence_pack_built ? "yes" : "no"}</strong></div>
                  <div><span>No persistence</span><strong>{scholarChatGroundedAnswerBuildRequestPreview()!.no_persistence ? "yes" : "no"}</strong></div>
                  <div><span>No LLM call</span><strong>{scholarChatGroundedAnswerBuildRequestPreview()!.no_llm_call ? "yes" : "no"}</strong></div>
                  <div><span>No runtime execution</span><strong>{scholarChatGroundedAnswerBuildRequestPreview()!.no_runtime_execution ? "yes" : "no"}</strong></div>
                  <div><span>No registry status change</span><strong>{scholarChatGroundedAnswerBuildRequestPreview()!.no_registry_status_change ? "yes" : "no"}</strong></div>
                  <div><span>No audit write</span><strong>{scholarChatGroundedAnswerBuildRequestPreview()!.no_audit_write ? "yes" : "no"}</strong></div>
                  <div><span>No grounded answer service call</span><strong>{scholarChatGroundedAnswerBuildRequestPreview()!.no_grounded_answer_service_call ? "yes" : "no"}</strong></div>
                </div>
                {scholarChatGroundedAnswerBuildRequestPreview()!.required_inputs.length > 0 ? (
                  <div class="warning-box">
                    <h4>Required inputs</h4>
                    <ul>
                      {scholarChatGroundedAnswerBuildRequestPreview()!.required_inputs.map((input) => (
                        <li>{input}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No required inputs.</p>
                )}
                {scholarChatGroundedAnswerBuildRequestPreview()!.missing_inputs.length > 0 ? (
                  <div class="warning-box">
                    <h4>Missing inputs</h4>
                    <ul>
                      {scholarChatGroundedAnswerBuildRequestPreview()!.missing_inputs.map((input) => (
                        <li>{input}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No missing inputs.</p>
                )}
                {scholarChatGroundedAnswerBuildRequestPreview()!.request_reasons.length > 0 ? (
                  <div class="warning-box">
                    <h4>Request reasons</h4>
                    <ul>
                      {scholarChatGroundedAnswerBuildRequestPreview()!.request_reasons.map((reason) => (
                        <li>{reason}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No request reasons.</p>
                )}
                {scholarChatGroundedAnswerBuildRequestPreview()!.blockers.length > 0 ? (
                  <div class="warning-box">
                    <h4>Blockers</h4>
                    <ul>
                      {scholarChatGroundedAnswerBuildRequestPreview()!.blockers.map((blocker) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(blocker.kind)}</strong>
                          <div>{blocker.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No grounded answer build request blockers.</p>
                )}
                {scholarChatGroundedAnswerBuildRequestPreview()!.warnings.length > 0 ? (
                  <div class="warning-box">
                    <h4>Warnings</h4>
                    <ul>
                      {scholarChatGroundedAnswerBuildRequestPreview()!.warnings.map((warning) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(warning.kind)}</strong>
                          <div>{warning.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No grounded answer build request warnings.</p>
                )}
                {scholarChatGroundedAnswerBuildRequestPreview()!.next_required_actions.length > 0 ? (
                  <div class="warning-box">
                    <h4>Next required actions</h4>
                    <ul>
                      {scholarChatGroundedAnswerBuildRequestPreview()!.next_required_actions.map((action) => (
                        <li>{action}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No next required actions.</p>
                )}
              </>
            ) : (
              <p>No grounded answer build request preview loaded yet.</p>
            )
          ) : (
            <p>No grounded answer build request preview loaded yet.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>Grounded answer build preflight</h3>
          <p class="muted">
            Preflight only - no GroundedAnswer service call was made. Not an AnswerDraft, GroundedAnswer, FinalAnswer, Evidence Pack, or persisted artifact. No registry status change, audit write, runtime execution, or LLM call occurred.
          </p>
          <p class="muted">{scholarChatSelectedSourceIdsSummary()}</p>
          <p class="muted">Uses the current Scholar Chat request, the draft text from the inspection card above, and the build request preview inputs.</p>
          <div class="hero-actions">
            <button onClick={previewScholarChatGroundedAnswerBuildPreflight} disabled={scholarChatGroundedAnswerBuildPreflightLoading()}>
              {scholarChatGroundedAnswerBuildPreflightLoading() ? "Previewing..." : "Preview grounded answer build preflight"}
            </button>
          </div>
          <p class="muted">Preflight preview only - not a verified grounded answer. No AnswerDraft, GroundedAnswer, FinalAnswer, Evidence Pack, persistence, runtime execution, or LLM call was created.</p>
          {scholarChatGroundedAnswerBuildPreflightValidationError() && <p class="error">{scholarChatGroundedAnswerBuildPreflightValidationError()}</p>}
          {scholarChatGroundedAnswerBuildPreflightError() && <p class="error">{scholarChatGroundedAnswerBuildPreflightError()}</p>}
          {scholarChatGroundedAnswerBuildPreflightLoading() ? (
            <p>Previewing grounded answer build preflight...</p>
          ) : scholarChatGroundedAnswerBuildPreflightHasRun() ? (
            scholarChatGroundedAnswerBuildPreflightPreview() ? (
              <>
                {renderMetricGrid([
                  { label: "Status", value: formatSnakeCaseLabel(scholarChatGroundedAnswerBuildPreflightPreview()!.status) },
                  { label: "Build request status", value: formatSnakeCaseLabel(scholarChatGroundedAnswerBuildPreflightPreview()!.build_request_status) },
                  { label: "Build intent status", value: formatSnakeCaseLabel(scholarChatGroundedAnswerBuildPreflightPreview()!.build_intent_status) },
                  { label: "Write eligibility status", value: formatSnakeCaseLabel(scholarChatGroundedAnswerBuildPreflightPreview()!.write_eligibility_status) },
                  { label: "Candidate status", value: formatSnakeCaseLabel(scholarChatGroundedAnswerBuildPreflightPreview()!.candidate_status) },
                  { label: "Selected sources", value: scholarChatGroundedAnswerBuildPreflightPreview()!.selected_source_count },
                  { label: "Evidence candidates", value: scholarChatGroundedAnswerBuildPreflightPreview()!.evidence_candidate_count },
                  { label: "Inspected items", value: scholarChatGroundedAnswerBuildPreflightPreview()!.inspected_item_count },
                  { label: "Supported items", value: scholarChatGroundedAnswerBuildPreflightPreview()!.supported_item_count },
                  { label: "Weakly supported items", value: scholarChatGroundedAnswerBuildPreflightPreview()!.weakly_supported_item_count },
                  { label: "Unsupported items", value: scholarChatGroundedAnswerBuildPreflightPreview()!.unsupported_item_count },
                  { label: "Candidate statements", value: scholarChatGroundedAnswerBuildPreflightPreview()!.candidate_statement_count },
                  { label: "Answer draft claims", value: scholarChatGroundedAnswerBuildPreflightPreview()!.answer_draft_claim_count },
                ])}
                <p><strong>Prompt:</strong> {scholarChatGroundedAnswerBuildPreflightPreview()!.normalized_prompt}</p>
                <p><strong>Answer draft ID:</strong> {scholarChatGroundedAnswerBuildPreflightPreview()!.answer_draft_id ?? "none"}</p>
                <p><strong>Selected source IDs:</strong> {scholarChatGroundedAnswerBuildPreflightPreview()!.selected_source_ids.length > 0 ? scholarChatGroundedAnswerBuildPreflightPreview()!.selected_source_ids.join(", ") : "none"}</p>
                <p>{scholarChatGroundedAnswerBuildPreflightPreview()!.summary}</p>
                <div class="contract-meta">
                  <div><span>Preview only</span><strong>{scholarChatGroundedAnswerBuildPreflightPreview()!.preview_only ? "yes" : "no"}</strong></div>
                  <div><span>Not answer draft</span><strong>{scholarChatGroundedAnswerBuildPreflightPreview()!.not_answer_draft ? "yes" : "no"}</strong></div>
                  <div><span>Not grounded answer</span><strong>{scholarChatGroundedAnswerBuildPreflightPreview()!.not_grounded_answer ? "yes" : "no"}</strong></div>
                  <div><span>Not final answer</span><strong>{scholarChatGroundedAnswerBuildPreflightPreview()!.not_final_answer ? "yes" : "no"}</strong></div>
                  <div><span>No answer artifact created</span><strong>{scholarChatGroundedAnswerBuildPreflightPreview()!.no_answer_artifact_created ? "yes" : "no"}</strong></div>
                  <div><span>No Evidence Pack built</span><strong>{scholarChatGroundedAnswerBuildPreflightPreview()!.no_evidence_pack_built ? "yes" : "no"}</strong></div>
                  <div><span>No persistence</span><strong>{scholarChatGroundedAnswerBuildPreflightPreview()!.no_persistence ? "yes" : "no"}</strong></div>
                  <div><span>No LLM call</span><strong>{scholarChatGroundedAnswerBuildPreflightPreview()!.no_llm_call ? "yes" : "no"}</strong></div>
                  <div><span>No runtime execution</span><strong>{scholarChatGroundedAnswerBuildPreflightPreview()!.no_runtime_execution ? "yes" : "no"}</strong></div>
                  <div><span>No registry status change</span><strong>{scholarChatGroundedAnswerBuildPreflightPreview()!.no_registry_status_change ? "yes" : "no"}</strong></div>
                  <div><span>No audit write</span><strong>{scholarChatGroundedAnswerBuildPreflightPreview()!.no_audit_write ? "yes" : "no"}</strong></div>
                  <div><span>No grounded answer service call</span><strong>{scholarChatGroundedAnswerBuildPreflightPreview()!.no_grounded_answer_service_call ? "yes" : "no"}</strong></div>
                  <div><span>No grounded answer write</span><strong>{scholarChatGroundedAnswerBuildPreflightPreview()!.no_grounded_answer_write ? "yes" : "no"}</strong></div>
                  <div><span>Answer draft present</span><strong>{scholarChatGroundedAnswerBuildPreflightPreview()!.answer_draft_present ? "yes" : "no"}</strong></div>
                  <div><span>Answer draft readable</span><strong>{scholarChatGroundedAnswerBuildPreflightPreview()!.answer_draft_readable ? "yes" : "no"}</strong></div>
                </div>
                {scholarChatGroundedAnswerBuildPreflightPreview()!.required_inputs.length > 0 ? (
                  <div class="warning-box">
                    <h4>Required inputs</h4>
                    <ul>
                      {scholarChatGroundedAnswerBuildPreflightPreview()!.required_inputs.map((input) => (
                        <li>{input}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No required inputs.</p>
                )}
                {scholarChatGroundedAnswerBuildPreflightPreview()!.missing_inputs.length > 0 ? (
                  <div class="warning-box">
                    <h4>Missing inputs</h4>
                    <ul>
                      {scholarChatGroundedAnswerBuildPreflightPreview()!.missing_inputs.map((input) => (
                        <li>{input}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No missing inputs.</p>
                )}
                {scholarChatGroundedAnswerBuildPreflightPreview()!.preflight_reasons.length > 0 ? (
                  <div class="warning-box">
                    <h4>Preflight reasons</h4>
                    <ul>
                      {scholarChatGroundedAnswerBuildPreflightPreview()!.preflight_reasons.map((reason) => (
                        <li>{reason}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No preflight reasons.</p>
                )}
                {scholarChatGroundedAnswerBuildPreflightPreview()!.blockers.length > 0 ? (
                  <div class="warning-box">
                    <h4>Blockers</h4>
                    <ul>
                      {scholarChatGroundedAnswerBuildPreflightPreview()!.blockers.map((blocker) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(blocker.kind)}</strong>
                          <div>{blocker.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No grounded answer build preflight blockers.</p>
                )}
                {scholarChatGroundedAnswerBuildPreflightPreview()!.warnings.length > 0 ? (
                  <div class="warning-box">
                    <h4>Warnings</h4>
                    <ul>
                      {scholarChatGroundedAnswerBuildPreflightPreview()!.warnings.map((warning) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(warning.kind)}</strong>
                          <div>{warning.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No grounded answer build preflight warnings.</p>
                )}
                {scholarChatGroundedAnswerBuildPreflightPreview()!.next_required_actions.length > 0 ? (
                  <div class="warning-box">
                    <h4>Next required actions</h4>
                    <ul>
                      {scholarChatGroundedAnswerBuildPreflightPreview()!.next_required_actions.map((action) => (
                        <li>{action}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No next required actions.</p>
                )}
              </>
            ) : (
              <p>No grounded answer build preflight preview loaded yet.</p>
            )
          ) : (
            <p>No grounded answer build preflight preview loaded yet.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>Grounded answer execution readiness</h3>
          <p class="muted">
            Execution-readiness preview only - no GroundedAnswer service call was made. No answer artifact, Evidence Pack, final answer, persistence, registry status change, audit write, runtime execution, or LLM call occurred.
          </p>
          <p class="muted">{scholarChatSelectedSourceIdsSummary()}</p>
          <p class="muted">Uses the current Scholar Chat request, the draft text from the inspection card above, the build preflight preview, and the execution consent checkbox.</p>
          <div class="form-row">
            <label class="inline-field">
              <input
                type="checkbox"
                checked={scholarChatGroundedAnswerExecutionReadinessExecutionConsent()}
                onChange={(event) => {
                  setScholarChatGroundedAnswerExecutionReadinessExecutionConsent(event.currentTarget.checked);
                  clearScholarChatGroundedAnswerExecutionReadinessPreview();
                }}
              />
              I understand this is still only an execution-readiness preview.
            </label>
          </div>
          <div class="hero-actions">
            <button onClick={previewScholarChatGroundedAnswerExecutionReadiness} disabled={scholarChatGroundedAnswerExecutionReadinessLoading()}>
              {scholarChatGroundedAnswerExecutionReadinessLoading() ? "Previewing..." : "Preview grounded answer execution readiness"}
            </button>
          </div>
          <p class="muted">Readiness only - not a verified grounded answer. No AnswerDraft, GroundedAnswer, FinalAnswer, Evidence Pack, persistence, runtime execution, or LLM call was created.</p>
          {scholarChatGroundedAnswerExecutionReadinessValidationError() && <p class="error">{scholarChatGroundedAnswerExecutionReadinessValidationError()}</p>}
          {scholarChatGroundedAnswerExecutionReadinessError() && <p class="error">{scholarChatGroundedAnswerExecutionReadinessError()}</p>}
          {scholarChatGroundedAnswerExecutionReadinessLoading() ? (
            <p>Previewing grounded answer execution readiness...</p>
          ) : scholarChatGroundedAnswerExecutionReadinessHasRun() ? (
            scholarChatGroundedAnswerExecutionReadinessPreview() ? (
              <>
                {renderMetricGrid([
                  { label: "Status", value: formatSnakeCaseLabel(scholarChatGroundedAnswerExecutionReadinessPreview()!.status) },
                  { label: "Build preflight status", value: formatSnakeCaseLabel(scholarChatGroundedAnswerExecutionReadinessPreview()!.build_preflight_status) },
                  { label: "Build request status", value: formatSnakeCaseLabel(scholarChatGroundedAnswerExecutionReadinessPreview()!.build_request_status) },
                  { label: "Build intent status", value: formatSnakeCaseLabel(scholarChatGroundedAnswerExecutionReadinessPreview()!.build_intent_status) },
                  { label: "Write eligibility status", value: formatSnakeCaseLabel(scholarChatGroundedAnswerExecutionReadinessPreview()!.write_eligibility_status) },
                  { label: "Candidate status", value: formatSnakeCaseLabel(scholarChatGroundedAnswerExecutionReadinessPreview()!.candidate_status) },
                  { label: "Selected sources", value: scholarChatGroundedAnswerExecutionReadinessPreview()!.selected_source_count },
                  { label: "Evidence candidates", value: scholarChatGroundedAnswerExecutionReadinessPreview()!.evidence_candidate_count },
                  { label: "Inspected items", value: scholarChatGroundedAnswerExecutionReadinessPreview()!.inspected_item_count },
                  { label: "Supported items", value: scholarChatGroundedAnswerExecutionReadinessPreview()!.supported_item_count },
                  { label: "Weakly supported items", value: scholarChatGroundedAnswerExecutionReadinessPreview()!.weakly_supported_item_count },
                  { label: "Unsupported items", value: scholarChatGroundedAnswerExecutionReadinessPreview()!.unsupported_item_count },
                  { label: "Candidate statements", value: scholarChatGroundedAnswerExecutionReadinessPreview()!.candidate_statement_count },
                  { label: "Answer draft claims", value: scholarChatGroundedAnswerExecutionReadinessPreview()!.answer_draft_claim_count },
                ])}
                <p><strong>Prompt:</strong> {scholarChatGroundedAnswerExecutionReadinessPreview()!.normalized_prompt}</p>
                <p><strong>Answer draft ID:</strong> {scholarChatGroundedAnswerExecutionReadinessPreview()!.answer_draft_id ?? "none"}</p>
                <p><strong>Selected source IDs:</strong> {scholarChatGroundedAnswerExecutionReadinessPreview()!.selected_source_ids.length > 0 ? scholarChatGroundedAnswerExecutionReadinessPreview()!.selected_source_ids.join(", ") : "none"}</p>
                <p><strong>Execution consent:</strong> {scholarChatGroundedAnswerExecutionReadinessPreview()!.execution_consent ? "yes" : "no"}</p>
                <p>{scholarChatGroundedAnswerExecutionReadinessPreview()!.summary}</p>
                <div class="contract-meta">
                  <div><span>Preview only</span><strong>{scholarChatGroundedAnswerExecutionReadinessPreview()!.preview_only ? "yes" : "no"}</strong></div>
                  <div><span>Not answer draft</span><strong>{scholarChatGroundedAnswerExecutionReadinessPreview()!.not_answer_draft ? "yes" : "no"}</strong></div>
                  <div><span>Not grounded answer</span><strong>{scholarChatGroundedAnswerExecutionReadinessPreview()!.not_grounded_answer ? "yes" : "no"}</strong></div>
                  <div><span>Not final answer</span><strong>{scholarChatGroundedAnswerExecutionReadinessPreview()!.not_final_answer ? "yes" : "no"}</strong></div>
                  <div><span>No answer artifact created</span><strong>{scholarChatGroundedAnswerExecutionReadinessPreview()!.no_answer_artifact_created ? "yes" : "no"}</strong></div>
                  <div><span>No Evidence Pack built</span><strong>{scholarChatGroundedAnswerExecutionReadinessPreview()!.no_evidence_pack_built ? "yes" : "no"}</strong></div>
                  <div><span>No persistence</span><strong>{scholarChatGroundedAnswerExecutionReadinessPreview()!.no_persistence ? "yes" : "no"}</strong></div>
                  <div><span>No LLM call</span><strong>{scholarChatGroundedAnswerExecutionReadinessPreview()!.no_llm_call ? "yes" : "no"}</strong></div>
                  <div><span>No runtime execution</span><strong>{scholarChatGroundedAnswerExecutionReadinessPreview()!.no_runtime_execution ? "yes" : "no"}</strong></div>
                  <div><span>No registry status change</span><strong>{scholarChatGroundedAnswerExecutionReadinessPreview()!.no_registry_status_change ? "yes" : "no"}</strong></div>
                  <div><span>No audit write</span><strong>{scholarChatGroundedAnswerExecutionReadinessPreview()!.no_audit_write ? "yes" : "no"}</strong></div>
                  <div><span>No grounded answer service call</span><strong>{scholarChatGroundedAnswerExecutionReadinessPreview()!.no_grounded_answer_service_call ? "yes" : "no"}</strong></div>
                  <div><span>No grounded answer write</span><strong>{scholarChatGroundedAnswerExecutionReadinessPreview()!.no_grounded_answer_write ? "yes" : "no"}</strong></div>
                  <div><span>Answer draft present</span><strong>{scholarChatGroundedAnswerExecutionReadinessPreview()!.answer_draft_present ? "yes" : "no"}</strong></div>
                  <div><span>Answer draft readable</span><strong>{scholarChatGroundedAnswerExecutionReadinessPreview()!.answer_draft_readable ? "yes" : "no"}</strong></div>
                </div>
                {scholarChatGroundedAnswerExecutionReadinessPreview()!.required_inputs.length > 0 ? (
                  <div class="warning-box">
                    <h4>Required inputs</h4>
                    <ul>
                      {scholarChatGroundedAnswerExecutionReadinessPreview()!.required_inputs.map((input) => (
                        <li>{input}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No required inputs.</p>
                )}
                {scholarChatGroundedAnswerExecutionReadinessPreview()!.missing_inputs.length > 0 ? (
                  <div class="warning-box">
                    <h4>Missing inputs</h4>
                    <ul>
                      {scholarChatGroundedAnswerExecutionReadinessPreview()!.missing_inputs.map((input) => (
                        <li>{input}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No missing inputs.</p>
                )}
                {scholarChatGroundedAnswerExecutionReadinessPreview()!.readiness_reasons.length > 0 ? (
                  <div class="warning-box">
                    <h4>Readiness reasons</h4>
                    <ul>
                      {scholarChatGroundedAnswerExecutionReadinessPreview()!.readiness_reasons.map((reason) => (
                        <li>{reason}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No readiness reasons.</p>
                )}
                {scholarChatGroundedAnswerExecutionReadinessPreview()!.blockers.length > 0 ? (
                  <div class="warning-box">
                    <h4>Blockers</h4>
                    <ul>
                      {scholarChatGroundedAnswerExecutionReadinessPreview()!.blockers.map((blocker) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(blocker.kind)}</strong>
                          <div>{blocker.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No execution-readiness blockers.</p>
                )}
                {scholarChatGroundedAnswerExecutionReadinessPreview()!.warnings.length > 0 ? (
                  <div class="warning-box">
                    <h4>Warnings</h4>
                    <ul>
                      {scholarChatGroundedAnswerExecutionReadinessPreview()!.warnings.map((warning) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(warning.kind)}</strong>
                          <div>{warning.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No execution-readiness warnings.</p>
                )}
                {scholarChatGroundedAnswerExecutionReadinessPreview()!.next_required_actions.length > 0 ? (
                  <div class="warning-box">
                    <h4>Next required actions</h4>
                    <ul>
                      {scholarChatGroundedAnswerExecutionReadinessPreview()!.next_required_actions.map((action) => (
                        <li>{action}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No next required actions.</p>
                )}
              </>
            ) : (
              <p>No grounded answer execution readiness preview loaded yet.</p>
            )
          ) : (
            <p>No grounded answer execution readiness preview loaded yet.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>Grounded answer execution plan</h3>
          <p class="muted">
            Execution plan preview only - no GroundedAnswer service call was made. No answer artifact, Evidence Pack, final answer, persistence, registry status change, audit write, runtime execution, or LLM call occurred.
          </p>
          <p class="muted">{scholarChatSelectedSourceIdsSummary()}</p>
          <p class="muted">Uses the current Scholar Chat request, the draft text from the inspection card above, and the execution-readiness preview.</p>
          <div class="hero-actions">
            <button onClick={previewScholarChatGroundedAnswerExecutionPlan} disabled={scholarChatGroundedAnswerExecutionPlanLoading()}>
              {scholarChatGroundedAnswerExecutionPlanLoading() ? "Previewing..." : "Preview grounded answer execution plan"}
            </button>
          </div>
          <p class="muted">Plan preview only - not a grounded answer. No AnswerDraft, GroundedAnswer, FinalAnswer, Evidence Pack, persistence, runtime execution, or LLM call was created.</p>
          {scholarChatGroundedAnswerExecutionPlanValidationError() && <p class="error">{scholarChatGroundedAnswerExecutionPlanValidationError()}</p>}
          {scholarChatGroundedAnswerExecutionPlanError() && <p class="error">{scholarChatGroundedAnswerExecutionPlanError()}</p>}
          {scholarChatGroundedAnswerExecutionPlanLoading() ? (
            <p>Previewing grounded answer execution plan...</p>
          ) : scholarChatGroundedAnswerExecutionPlanHasRun() ? (
            scholarChatGroundedAnswerExecutionPlanPreview() ? (
              <>
                {renderMetricGrid([
                  { label: "Status", value: formatSnakeCaseLabel(scholarChatGroundedAnswerExecutionPlanPreview()!.status) },
                  { label: "Readiness status", value: formatSnakeCaseLabel(scholarChatGroundedAnswerExecutionPlanPreview()!.readiness_status) },
                  { label: "Build preflight status", value: formatSnakeCaseLabel(scholarChatGroundedAnswerExecutionPlanPreview()!.build_preflight_status) },
                  { label: "Build request status", value: formatSnakeCaseLabel(scholarChatGroundedAnswerExecutionPlanPreview()!.build_request_status) },
                  { label: "Build intent status", value: formatSnakeCaseLabel(scholarChatGroundedAnswerExecutionPlanPreview()!.build_intent_status) },
                  { label: "Write eligibility status", value: formatSnakeCaseLabel(scholarChatGroundedAnswerExecutionPlanPreview()!.write_eligibility_status) },
                  { label: "Candidate status", value: formatSnakeCaseLabel(scholarChatGroundedAnswerExecutionPlanPreview()!.candidate_status) },
                  { label: "Selected sources", value: scholarChatGroundedAnswerExecutionPlanPreview()!.selected_source_count },
                  { label: "Evidence candidates", value: scholarChatGroundedAnswerExecutionPlanPreview()!.evidence_candidate_count },
                  { label: "Inspected items", value: scholarChatGroundedAnswerExecutionPlanPreview()!.inspected_item_count },
                  { label: "Supported items", value: scholarChatGroundedAnswerExecutionPlanPreview()!.supported_item_count },
                  { label: "Weakly supported items", value: scholarChatGroundedAnswerExecutionPlanPreview()!.weakly_supported_item_count },
                  { label: "Unsupported items", value: scholarChatGroundedAnswerExecutionPlanPreview()!.unsupported_item_count },
                  { label: "Candidate statements", value: scholarChatGroundedAnswerExecutionPlanPreview()!.candidate_statement_count },
                  { label: "Answer draft claims", value: scholarChatGroundedAnswerExecutionPlanPreview()!.answer_draft_claim_count },
                ])}
                <p><strong>Prompt:</strong> {scholarChatGroundedAnswerExecutionPlanPreview()!.normalized_prompt}</p>
                <p><strong>Answer draft ID:</strong> {scholarChatGroundedAnswerExecutionPlanPreview()!.answer_draft_id ?? "none"}</p>
                <p><strong>Selected source IDs:</strong> {scholarChatGroundedAnswerExecutionPlanPreview()!.selected_source_ids.length > 0 ? scholarChatGroundedAnswerExecutionPlanPreview()!.selected_source_ids.join(", ") : "none"}</p>
                <p><strong>Execution consent:</strong> {scholarChatGroundedAnswerExecutionPlanPreview()!.execution_consent ? "yes" : "no"}</p>
                <p><strong>Planned operation:</strong> {scholarChatGroundedAnswerExecutionPlanPreview()!.planned_operation}</p>
                <p>{scholarChatGroundedAnswerExecutionPlanPreview()!.summary}</p>
                <div class="contract-meta">
                  <div><span>Preview only</span><strong>{scholarChatGroundedAnswerExecutionPlanPreview()!.preview_only ? "yes" : "no"}</strong></div>
                  <div><span>Not answer draft</span><strong>{scholarChatGroundedAnswerExecutionPlanPreview()!.not_answer_draft ? "yes" : "no"}</strong></div>
                  <div><span>Not grounded answer</span><strong>{scholarChatGroundedAnswerExecutionPlanPreview()!.not_grounded_answer ? "yes" : "no"}</strong></div>
                  <div><span>Not final answer</span><strong>{scholarChatGroundedAnswerExecutionPlanPreview()!.not_final_answer ? "yes" : "no"}</strong></div>
                  <div><span>No answer artifact created</span><strong>{scholarChatGroundedAnswerExecutionPlanPreview()!.no_answer_artifact_created ? "yes" : "no"}</strong></div>
                  <div><span>No Evidence Pack built</span><strong>{scholarChatGroundedAnswerExecutionPlanPreview()!.no_evidence_pack_built ? "yes" : "no"}</strong></div>
                  <div><span>No persistence</span><strong>{scholarChatGroundedAnswerExecutionPlanPreview()!.no_persistence ? "yes" : "no"}</strong></div>
                  <div><span>No LLM call</span><strong>{scholarChatGroundedAnswerExecutionPlanPreview()!.no_llm_call ? "yes" : "no"}</strong></div>
                  <div><span>No runtime execution</span><strong>{scholarChatGroundedAnswerExecutionPlanPreview()!.no_runtime_execution ? "yes" : "no"}</strong></div>
                  <div><span>No registry status change</span><strong>{scholarChatGroundedAnswerExecutionPlanPreview()!.no_registry_status_change ? "yes" : "no"}</strong></div>
                  <div><span>No audit write</span><strong>{scholarChatGroundedAnswerExecutionPlanPreview()!.no_audit_write ? "yes" : "no"}</strong></div>
                  <div><span>No grounded answer service call</span><strong>{scholarChatGroundedAnswerExecutionPlanPreview()!.no_grounded_answer_service_call ? "yes" : "no"}</strong></div>
                  <div><span>No grounded answer write</span><strong>{scholarChatGroundedAnswerExecutionPlanPreview()!.no_grounded_answer_write ? "yes" : "no"}</strong></div>
                </div>
                <div class="warning-box">
                  <h4>Planned inputs</h4>
                  <ul>
                    {scholarChatGroundedAnswerExecutionPlanPreview()!.planned_inputs.map((item) => (
                      <li>{item}</li>
                    ))}
                  </ul>
                </div>
                <div class="warning-box">
                  <h4>Planned outputs</h4>
                  <ul>
                    {scholarChatGroundedAnswerExecutionPlanPreview()!.planned_outputs.map((item) => (
                      <li>{item}</li>
                    ))}
                  </ul>
                </div>
                <div class="warning-box">
                  <h4>Planned write targets</h4>
                  <ul>
                    {scholarChatGroundedAnswerExecutionPlanPreview()!.planned_write_targets.map((item) => (
                      <li>{item}</li>
                    ))}
                  </ul>
                </div>
                {scholarChatGroundedAnswerExecutionPlanPreview()!.required_inputs.length > 0 ? (
                  <div class="warning-box">
                    <h4>Required inputs</h4>
                    <ul>
                      {scholarChatGroundedAnswerExecutionPlanPreview()!.required_inputs.map((input) => (
                        <li>{input}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No required inputs.</p>
                )}
                {scholarChatGroundedAnswerExecutionPlanPreview()!.missing_inputs.length > 0 ? (
                  <div class="warning-box">
                    <h4>Missing inputs</h4>
                    <ul>
                      {scholarChatGroundedAnswerExecutionPlanPreview()!.missing_inputs.map((input) => (
                        <li>{input}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No missing inputs.</p>
                )}
                {scholarChatGroundedAnswerExecutionPlanPreview()!.plan_reasons.length > 0 ? (
                  <div class="warning-box">
                    <h4>Plan reasons</h4>
                    <ul>
                      {scholarChatGroundedAnswerExecutionPlanPreview()!.plan_reasons.map((reason) => (
                        <li>{reason}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No plan reasons.</p>
                )}
                {scholarChatGroundedAnswerExecutionPlanPreview()!.blockers.length > 0 ? (
                  <div class="warning-box">
                    <h4>Blockers</h4>
                    <ul>
                      {scholarChatGroundedAnswerExecutionPlanPreview()!.blockers.map((blocker) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(blocker.kind)}</strong>
                          <div>{blocker.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No execution-plan blockers.</p>
                )}
                {scholarChatGroundedAnswerExecutionPlanPreview()!.warnings.length > 0 ? (
                  <div class="warning-box">
                    <h4>Warnings</h4>
                    <ul>
                      {scholarChatGroundedAnswerExecutionPlanPreview()!.warnings.map((warning) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(warning.kind)}</strong>
                          <div>{warning.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No execution-plan warnings.</p>
                )}
                {scholarChatGroundedAnswerExecutionPlanPreview()!.next_required_actions.length > 0 ? (
                  <div class="warning-box">
                    <h4>Next required actions</h4>
                    <ul>
                      {scholarChatGroundedAnswerExecutionPlanPreview()!.next_required_actions.map((action) => (
                        <li>{action}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No next required actions.</p>
                )}
              </>
            ) : (
              <p>No grounded answer execution plan preview loaded yet.</p>
            )
          ) : (
            <p>No grounded answer execution plan preview loaded yet.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>Runtime diagnostic bridge</h3>
          <p class="muted">
            Bridge preview only - this does not run smoke diagnostics, does not run inference, does not call an LLM, does not generate a Scholar Chat answer, and does not write artifacts.
          </p>
          <p class="muted">{scholarChatSelectedSourceIdsSummary()}</p>
          <p class="muted">Uses the current Scholar Chat request and the local runtime smoke execution plan preview inputs.</p>
          <div class="hero-actions">
            <button onClick={previewScholarChatRuntimeDiagnosticBridge} disabled={scholarChatRuntimeDiagnosticBridgeLoading()}>
              {scholarChatRuntimeDiagnosticBridgeLoading() ? "Previewing..." : "Preview Scholar Chat runtime diagnostic bridge"}
            </button>
          </div>
          {scholarChatRuntimeDiagnosticBridgeValidationError() && <p class="error">{scholarChatRuntimeDiagnosticBridgeValidationError()}</p>}
          {scholarChatRuntimeDiagnosticBridgeError() && <p class="error">{scholarChatRuntimeDiagnosticBridgeError()}</p>}
          {scholarChatRuntimeDiagnosticBridgeLoading() ? (
            <p>Previewing Scholar Chat runtime diagnostic bridge...</p>
          ) : scholarChatRuntimeDiagnosticBridgeHasRun() ? (
            scholarChatRuntimeDiagnosticBridgePreview() ? (
              <>
                {renderMetricGrid([
                  { label: "Status", value: formatSnakeCaseLabel(scholarChatRuntimeDiagnosticBridgePreview()!.status) },
                  { label: "Selected sources", value: scholarChatRuntimeDiagnosticBridgePreview()!.selected_source_count },
                  { label: "Smoke execution plan", value: formatSnakeCaseLabel(scholarChatRuntimeDiagnosticBridgePreview()!.smoke_execution_plan_status) },
                  { label: "Smoke readiness", value: formatSnakeCaseLabel(scholarChatRuntimeDiagnosticBridgePreview()!.smoke_readiness_status) },
                  { label: "Capability status", value: formatSnakeCaseLabel(scholarChatRuntimeDiagnosticBridgePreview()!.capability_status) },
                  { label: "Version probe status", value: formatSnakeCaseLabel(scholarChatRuntimeDiagnosticBridgePreview()!.version_probe_status) },
                  { label: "Probe readiness", value: formatSnakeCaseLabel(scholarChatRuntimeDiagnosticBridgePreview()!.probe_readiness_status) },
                  { label: "Validation status", value: formatSnakeCaseLabel(scholarChatRuntimeDiagnosticBridgePreview()!.validation_status) },
                  { label: "Adapter contract status", value: formatSnakeCaseLabel(scholarChatRuntimeDiagnosticBridgePreview()!.adapter_contract_status) },
                  { label: "Adapter kind", value: formatSnakeCaseLabel(scholarChatRuntimeDiagnosticBridgePreview()!.adapter_kind) },
                  { label: "Normalized model family", value: scholarChatRuntimeDiagnosticBridgePreview()!.normalized_model_family ?? "none" },
                  { label: "Normalized model format", value: scholarChatRuntimeDiagnosticBridgePreview()!.normalized_model_format },
                  { label: "Prompt chars", value: scholarChatRuntimeDiagnosticBridgePreview()!.diagnostic_prompt_char_count },
                  { label: "Max output tokens", value: scholarChatRuntimeDiagnosticBridgePreview()!.max_output_tokens },
                  { label: "Timeout ms", value: scholarChatRuntimeDiagnosticBridgePreview()!.timeout_ms },
                ])}
                {scholarChatRuntimeDiagnosticBridgePreview()!.safe_executable_file_name ? (
                  <p><strong>Executable file name:</strong> {scholarChatRuntimeDiagnosticBridgePreview()!.safe_executable_file_name}</p>
                ) : null}
                {scholarChatRuntimeDiagnosticBridgePreview()!.safe_model_file_name ? (
                  <p><strong>Model file name:</strong> {scholarChatRuntimeDiagnosticBridgePreview()!.safe_model_file_name}</p>
                ) : null}
                <p><strong>Prompt:</strong> {scholarChatRuntimeDiagnosticBridgePreview()!.normalized_prompt}</p>
                <p class="muted">{scholarChatRuntimeDiagnosticBridgePreview()!.summary}</p>
                <div class="contract-meta">
                  <div><span>Preview only</span><strong>{scholarChatRuntimeDiagnosticBridgePreview()!.preview_only ? "yes" : "no"}</strong></div>
                  <div><span>No smoke execution</span><strong>{scholarChatRuntimeDiagnosticBridgePreview()!.no_smoke_execution ? "yes" : "no"}</strong></div>
                  <div><span>No runtime inference</span><strong>{scholarChatRuntimeDiagnosticBridgePreview()!.no_runtime_inference ? "yes" : "no"}</strong></div>
                  <div><span>No LLM call</span><strong>{scholarChatRuntimeDiagnosticBridgePreview()!.no_llm_call ? "yes" : "no"}</strong></div>
                  <div><span>No answer generated</span><strong>{scholarChatRuntimeDiagnosticBridgePreview()!.no_answer_generated ? "yes" : "no"}</strong></div>
                  <div><span>No answer draft created</span><strong>{scholarChatRuntimeDiagnosticBridgePreview()!.no_answer_draft_created ? "yes" : "no"}</strong></div>
                  <div><span>No grounded answer created</span><strong>{scholarChatRuntimeDiagnosticBridgePreview()!.no_grounded_answer_created ? "yes" : "no"}</strong></div>
                  <div><span>No final answer created</span><strong>{scholarChatRuntimeDiagnosticBridgePreview()!.no_final_answer_created ? "yes" : "no"}</strong></div>
                  <div><span>No grounding applied</span><strong>{scholarChatRuntimeDiagnosticBridgePreview()!.no_grounding_applied ? "yes" : "no"}</strong></div>
                  <div><span>No Evidence Pack built</span><strong>{scholarChatRuntimeDiagnosticBridgePreview()!.no_evidence_pack_built ? "yes" : "no"}</strong></div>
                  <div><span>No persistence</span><strong>{scholarChatRuntimeDiagnosticBridgePreview()!.no_persistence ? "yes" : "no"}</strong></div>
                  <div><span>No artifact write</span><strong>{scholarChatRuntimeDiagnosticBridgePreview()!.no_artifact_write ? "yes" : "no"}</strong></div>
                  <div><span>No registry status change</span><strong>{scholarChatRuntimeDiagnosticBridgePreview()!.no_registry_status_change ? "yes" : "no"}</strong></div>
                  <div><span>No audit write</span><strong>{scholarChatRuntimeDiagnosticBridgePreview()!.no_audit_write ? "yes" : "no"}</strong></div>
                </div>
                {scholarChatRuntimeDiagnosticBridgePreview()!.runtime_diagnostic_reasons.length > 0 ? (
                  <div class="warning-box">
                    <h4>Runtime diagnostic reasons</h4>
                    <ul>
                      {scholarChatRuntimeDiagnosticBridgePreview()!.runtime_diagnostic_reasons.map((reason) => (
                        <li>{reason}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No runtime diagnostic reasons.</p>
                )}
                {scholarChatRuntimeDiagnosticBridgePreview()!.blockers.length > 0 ? (
                  <div class="warning-box">
                    <h4>Blockers</h4>
                    <ul>
                      {scholarChatRuntimeDiagnosticBridgePreview()!.blockers.map((blocker) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(blocker.kind)}</strong>
                          <div>{blocker.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No runtime diagnostic bridge blockers.</p>
                )}
                {scholarChatRuntimeDiagnosticBridgePreview()!.warnings.length > 0 ? (
                  <div class="warning-box">
                    <h4>Warnings</h4>
                    <ul>
                      {scholarChatRuntimeDiagnosticBridgePreview()!.warnings.map((warning) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(warning.kind)}</strong>
                          <div>{warning.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No runtime diagnostic bridge warnings.</p>
                )}
                {scholarChatRuntimeDiagnosticBridgePreview()!.next_required_actions.length > 0 ? (
                  <div class="warning-box">
                    <h4>Next required actions</h4>
                    <ul>
                      {scholarChatRuntimeDiagnosticBridgePreview()!.next_required_actions.map((action) => (
                        <li>{action}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No next required actions.</p>
                )}
              </>
            ) : (
              <p>No Scholar Chat runtime diagnostic bridge preview loaded yet.</p>
            )
          ) : (
            <p>No Scholar Chat runtime diagnostic bridge preview loaded yet.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>Runtime diagnostic result</h3>
          <p class="muted">
            Result preview only - this classifies an already-loaded smoke diagnostic preview for future Scholar Chat use. It does not run smoke diagnostics, does not run inference, does not call an LLM, does not generate a Scholar Chat answer, and does not write artifacts.
          </p>
          <p class="muted">It uses the current Scholar Chat request and the latest loaded smoke diagnostic preview.</p>
          <div class="hero-actions">
            <button onClick={previewScholarChatRuntimeDiagnosticResult} disabled={scholarChatRuntimeDiagnosticResultLoading()}>
              {scholarChatRuntimeDiagnosticResultLoading() ? "Previewing..." : "Preview Scholar Chat runtime diagnostic result"}
            </button>
          </div>
          {scholarChatRuntimeDiagnosticResultValidationError() && <p class="error">{scholarChatRuntimeDiagnosticResultValidationError()}</p>}
          {scholarChatRuntimeDiagnosticResultError() && <p class="error">{scholarChatRuntimeDiagnosticResultError()}</p>}
          {scholarChatRuntimeDiagnosticResultLoading() ? (
            <p>Previewing Scholar Chat runtime diagnostic result...</p>
          ) : scholarChatRuntimeDiagnosticResultHasRun() ? (
            scholarChatRuntimeDiagnosticResultPreview() ? (
              <>
                {renderMetricGrid([
                  { label: "Status", value: formatSnakeCaseLabel(scholarChatRuntimeDiagnosticResultPreview()!.status) },
                  { label: "Bridge status", value: formatSnakeCaseLabel(scholarChatRuntimeDiagnosticResultPreview()!.bridge_status) },
                  { label: "Smoke diagnostic status", value: formatSnakeCaseLabel(scholarChatRuntimeDiagnosticResultPreview()!.smoke_diagnostic_status) },
                  { label: "Smoke execution plan", value: formatSnakeCaseLabel(scholarChatRuntimeDiagnosticResultPreview()!.smoke_execution_plan_status) },
                  { label: "Smoke readiness", value: formatSnakeCaseLabel(scholarChatRuntimeDiagnosticResultPreview()!.smoke_readiness_status) },
                  { label: "Capability status", value: formatSnakeCaseLabel(scholarChatRuntimeDiagnosticResultPreview()!.capability_status) },
                  { label: "Version probe status", value: formatSnakeCaseLabel(scholarChatRuntimeDiagnosticResultPreview()!.version_probe_status) },
                  { label: "Probe readiness", value: formatSnakeCaseLabel(scholarChatRuntimeDiagnosticResultPreview()!.probe_readiness_status) },
                  { label: "Validation status", value: formatSnakeCaseLabel(scholarChatRuntimeDiagnosticResultPreview()!.validation_status) },
                  { label: "Adapter contract status", value: formatSnakeCaseLabel(scholarChatRuntimeDiagnosticResultPreview()!.adapter_contract_status) },
                  { label: "Adapter kind", value: formatSnakeCaseLabel(scholarChatRuntimeDiagnosticResultPreview()!.adapter_kind) },
                  { label: "Prompt chars", value: scholarChatRuntimeDiagnosticResultPreview()!.diagnostic_prompt_char_count },
                  { label: "Max output tokens", value: scholarChatRuntimeDiagnosticResultPreview()!.max_output_tokens },
                  { label: "Timeout ms", value: scholarChatRuntimeDiagnosticResultPreview()!.timeout_ms },
                  { label: "Exit code", value: scholarChatRuntimeDiagnosticResultPreview()!.exit_code ?? "missing" },
                  { label: "Stdout truncated", value: scholarChatRuntimeDiagnosticResultPreview()!.stdout_truncated ? "yes" : "no" },
                  { label: "Stderr truncated", value: scholarChatRuntimeDiagnosticResultPreview()!.stderr_truncated ? "yes" : "no" },
                ])}
                {scholarChatRuntimeDiagnosticResultPreview()!.safe_executable_file_name ? (
                  <p><strong>Executable file name:</strong> {scholarChatRuntimeDiagnosticResultPreview()!.safe_executable_file_name}</p>
                ) : null}
                {scholarChatRuntimeDiagnosticResultPreview()!.safe_model_file_name ? (
                  <p><strong>Model file name:</strong> {scholarChatRuntimeDiagnosticResultPreview()!.safe_model_file_name}</p>
                ) : null}
                <p><strong>Normalized model family:</strong> {scholarChatRuntimeDiagnosticResultPreview()!.normalized_model_family ?? "none"}</p>
                <p><strong>Normalized model format:</strong> {scholarChatRuntimeDiagnosticResultPreview()!.normalized_model_format}</p>
                <p class="muted">{scholarChatRuntimeDiagnosticResultPreview()!.summary}</p>
                {scholarChatRuntimeDiagnosticResultPreview()!.runtime_result_reasons.length > 0 ? (
                  <div class="warning-box">
                    <h4>Runtime diagnostic result reasons</h4>
                    <ul>
                      {scholarChatRuntimeDiagnosticResultPreview()!.runtime_result_reasons.map((reason) => (
                        <li>{reason}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No runtime diagnostic result reasons.</p>
                )}
                {scholarChatRuntimeDiagnosticResultPreview()!.stdout_preview ? (
                  <>
                    <h4>Stdout preview</h4>
                    <pre>{scholarChatRuntimeDiagnosticResultPreview()!.stdout_preview}</pre>
                  </>
                ) : (
                  <p>No stdout captured.</p>
                )}
                {scholarChatRuntimeDiagnosticResultPreview()!.stderr_preview ? (
                  <>
                    <h4>Stderr preview</h4>
                    <pre>{scholarChatRuntimeDiagnosticResultPreview()!.stderr_preview}</pre>
                  </>
                ) : (
                  <p>No stderr captured.</p>
                )}
                {scholarChatRuntimeDiagnosticResultPreview()!.blockers.length > 0 ? (
                  <div class="warning-box">
                    <h4>Blockers</h4>
                    <ul>
                      {scholarChatRuntimeDiagnosticResultPreview()!.blockers.map((blocker) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(blocker.kind)}</strong>
                          <div>{blocker.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No runtime diagnostic result blockers.</p>
                )}
                {scholarChatRuntimeDiagnosticResultPreview()!.warnings.length > 0 ? (
                  <div class="warning-box">
                    <h4>Warnings</h4>
                    <ul>
                      {scholarChatRuntimeDiagnosticResultPreview()!.warnings.map((warning) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(warning.kind)}</strong>
                          <div>{warning.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No runtime diagnostic result warnings.</p>
                )}
                {scholarChatRuntimeDiagnosticResultPreview()!.next_required_actions.length > 0 ? (
                  <div class="warning-box">
                    <h4>Next required actions</h4>
                    <ul>
                      {scholarChatRuntimeDiagnosticResultPreview()!.next_required_actions.map((action) => (
                        <li>{action}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No next required actions.</p>
                )}
                <div class="contract-meta">
                  <div><span>Preview only</span><strong>{scholarChatRuntimeDiagnosticResultPreview()!.preview_only ? "yes" : "no"}</strong></div>
                  <div><span>Diagnostic result only</span><strong>{scholarChatRuntimeDiagnosticResultPreview()!.diagnostic_result_only ? "yes" : "no"}</strong></div>
                  <div><span>No smoke execution</span><strong>{scholarChatRuntimeDiagnosticResultPreview()!.no_smoke_execution ? "yes" : "no"}</strong></div>
                  <div><span>No runtime inference</span><strong>{scholarChatRuntimeDiagnosticResultPreview()!.no_runtime_inference ? "yes" : "no"}</strong></div>
                  <div><span>No new process spawn</span><strong>{scholarChatRuntimeDiagnosticResultPreview()!.no_new_process_spawn ? "yes" : "no"}</strong></div>
                  <div><span>No LLM call</span><strong>{scholarChatRuntimeDiagnosticResultPreview()!.no_llm_call ? "yes" : "no"}</strong></div>
                  <div><span>No answer generated</span><strong>{scholarChatRuntimeDiagnosticResultPreview()!.no_answer_generated ? "yes" : "no"}</strong></div>
                  <div><span>No answer draft created</span><strong>{scholarChatRuntimeDiagnosticResultPreview()!.no_answer_draft_created ? "yes" : "no"}</strong></div>
                  <div><span>No grounded answer created</span><strong>{scholarChatRuntimeDiagnosticResultPreview()!.no_grounded_answer_created ? "yes" : "no"}</strong></div>
                  <div><span>No final answer created</span><strong>{scholarChatRuntimeDiagnosticResultPreview()!.no_final_answer_created ? "yes" : "no"}</strong></div>
                  <div><span>No grounding applied</span><strong>{scholarChatRuntimeDiagnosticResultPreview()!.no_grounding_applied ? "yes" : "no"}</strong></div>
                  <div><span>No Evidence Pack built</span><strong>{scholarChatRuntimeDiagnosticResultPreview()!.no_evidence_pack_built ? "yes" : "no"}</strong></div>
                  <div><span>No persistence</span><strong>{scholarChatRuntimeDiagnosticResultPreview()!.no_persistence ? "yes" : "no"}</strong></div>
                  <div><span>No artifact write</span><strong>{scholarChatRuntimeDiagnosticResultPreview()!.no_artifact_write ? "yes" : "no"}</strong></div>
                  <div><span>No registry status change</span><strong>{scholarChatRuntimeDiagnosticResultPreview()!.no_registry_status_change ? "yes" : "no"}</strong></div>
                  <div><span>No audit write</span><strong>{scholarChatRuntimeDiagnosticResultPreview()!.no_audit_write ? "yes" : "no"}</strong></div>
                </div>
              </>
            ) : (
              <p>No Scholar Chat runtime diagnostic result preview loaded yet.</p>
            )
          ) : (
            <p>No Scholar Chat runtime diagnostic result preview loaded yet.</p>
          )}
        </div>
        <div class="artifact-overview runtime-setup-card">
          <h3>Managed llama-server lifecycle</h3>
          <p class="muted">
            Backend-owned lifecycle preview and local launch control for llama-server. The server stays on localhost, remains consent-gated, and does not route output into Scholar Chat answers yet.
          </p>
          <div class="warning-box">
            <p><strong>AEGIS can start and stop the local llama-server it owns.</strong></p>
            <p><strong>AEGIS will not stop external servers it did not start.</strong></p>
            <p><strong>If the configured port is already occupied, start is blocked until you stop the external process or change the port.</strong></p>
            <p><strong>Server is stopped when the app exits where possible.</strong></p>
          </div>
          <ol class="runtime-sequence">
            <li>Preview the managed launch plan.</li>
            <li>Check the exact executable and `.gguf` model file.</li>
            <li>Confirm explicit consent before starting the server.</li>
            <li>Start the AEGIS-managed server on localhost.</li>
            <li>Check `/health` and stop only the managed process when needed.</li>
          </ol>
          <div class="form-row">
            <label>
              Executable path
              <input
                type="text"
                value={managedLlamaServerExecutablePath()}
                  onInput={(event) => {
                    setManagedLlamaServerExecutablePath(event.currentTarget.value);
                    clearManagedLlamaServerLaunchPreview();
                    clearManagedLlamaServerStatusPreview();
                    clearManagedLlamaServerChatDiagnosticPreview();
                    clearManagedLlamaServerSmokeDiagnosticPreview();
                  }}
                placeholder="C:\\path\\to\\llama-server.exe"
              />
            </label>
            <label>
              Model path
              <input
                type="text"
                value={managedLlamaServerModelPath()}
                  onInput={(event) => {
                    setManagedLlamaServerModelPath(event.currentTarget.value);
                    clearManagedLlamaServerLaunchPreview();
                    clearManagedLlamaServerStatusPreview();
                    clearManagedLlamaServerChatDiagnosticPreview();
                    clearManagedLlamaServerSmokeDiagnosticPreview();
                  }}
                placeholder="C:\\path\\to\\model.gguf"
              />
            </label>
          </div>
          <div class="form-row">
            <label>
              Host
              <input
                type="text"
                value={managedLlamaServerHost()}
                  onInput={(event) => {
                    setManagedLlamaServerHost(event.currentTarget.value);
                    clearManagedLlamaServerLaunchPreview();
                    clearManagedLlamaServerStatusPreview();
                    clearManagedLlamaServerChatDiagnosticPreview();
                    clearManagedLlamaServerSmokeDiagnosticPreview();
                  }}
                placeholder="127.0.0.1"
              />
            </label>
            <label>
              Port
              <input
                type="number"
                value={managedLlamaServerPort()}
                  onInput={(event) => {
                    setManagedLlamaServerPort(event.currentTarget.value);
                    clearManagedLlamaServerLaunchPreview();
                    clearManagedLlamaServerStatusPreview();
                    clearManagedLlamaServerChatDiagnosticPreview();
                    clearManagedLlamaServerSmokeDiagnosticPreview();
                  }}
                placeholder="48921"
              />
            </label>
          </div>
          <div class="form-row">
            <label>
              Alias
              <input
                type="text"
                value={managedLlamaServerAlias()}
                  onInput={(event) => {
                    setManagedLlamaServerAlias(event.currentTarget.value);
                    clearManagedLlamaServerLaunchPreview();
                    clearManagedLlamaServerStatusPreview();
                    clearManagedLlamaServerChatDiagnosticPreview();
                    clearManagedLlamaServerSmokeDiagnosticPreview();
                  }}
                placeholder="aegis-local-gemma"
              />
            </label>
            <label>
              Context window
              <input
                type="number"
                value={managedLlamaServerContextWindow()}
                  onInput={(event) => {
                    setManagedLlamaServerContextWindow(event.currentTarget.value);
                    clearManagedLlamaServerLaunchPreview();
                    clearManagedLlamaServerStatusPreview();
                    clearManagedLlamaServerChatDiagnosticPreview();
                    clearManagedLlamaServerSmokeDiagnosticPreview();
                  }}
                placeholder="4096"
              />
            </label>
            <label>
              GPU layers
              <input
                type="number"
                value={managedLlamaServerGpuLayers()}
                  onInput={(event) => {
                    setManagedLlamaServerGpuLayers(event.currentTarget.value);
                    clearManagedLlamaServerLaunchPreview();
                    clearManagedLlamaServerStatusPreview();
                    clearManagedLlamaServerChatDiagnosticPreview();
                    clearManagedLlamaServerSmokeDiagnosticPreview();
                  }}
                placeholder="0"
              />
            </label>
            <label class="inline-field">
              <input
                type="checkbox"
                checked={managedLlamaServerAllowStart()}
                onChange={(event) => {
                  setManagedLlamaServerAllowStart(event.currentTarget.checked);
                  clearManagedLlamaServerChatDiagnosticPreview();
                  clearManagedLlamaServerSmokeDiagnosticPreview();
                }}
              />
              I understand server start is consent-gated and local only.
            </label>
          </div>
          <div class="hero-actions">
            <button onClick={previewManagedLlamaServerLaunchPlan} disabled={managedLlamaServerLaunchLoading()}>
              {managedLlamaServerLaunchLoading() ? "Previewing..." : "Preview managed launch plan"}
            </button>
            <button onClick={startManagedLlamaServer} disabled={managedLlamaServerStatusLoading() || !managedLlamaServerAllowStart()}>
              {managedLlamaServerStatusLoading() ? "Starting..." : "Start managed server"}
            </button>
            <button onClick={checkManagedLlamaServerHealth} disabled={managedLlamaServerStatusLoading()}>
              {managedLlamaServerStatusLoading() ? "Checking..." : "Check health"}
            </button>
            <button onClick={stopManagedLlamaServer} disabled={managedLlamaServerStatusLoading()}>
              {managedLlamaServerStatusLoading() ? "Stopping..." : "Stop managed server"}
            </button>
            <button onClick={loadManagedLlamaServerStatus} disabled={managedLlamaServerStatusLoading()}>
              {managedLlamaServerStatusLoading() ? "Refreshing..." : "Refresh status"}
            </button>
          </div>
          <p class="muted">No Scholar Chat answer will use this output yet. The managed server stays secondary to the chat workflow.</p>
          {managedLlamaServerLaunchError() && <p class="error">{managedLlamaServerLaunchError()}</p>}
          {managedLlamaServerStatusError() && <p class="error">{managedLlamaServerStatusError()}</p>}
          {managedLlamaServerLaunchLoading() ? (
            <p>Previewing managed launch plan...</p>
          ) : managedLlamaServerLaunchHasRun() ? (
            managedLlamaServerLaunchPreview() ? (
              <>
                {renderMetricGrid([
                  { label: "Status", value: formatSnakeCaseLabel(managedLlamaServerLaunchPreview()!.status) },
                  { label: "Executable present", value: managedLlamaServerLaunchPreview()!.executable_path_present ? "yes" : "no" },
                  { label: "Executable is file", value: managedLlamaServerLaunchPreview()!.executable_is_file ? "yes" : "no" },
                  { label: "Model present", value: managedLlamaServerLaunchPreview()!.model_path_present ? "yes" : "no" },
                  { label: "Model is file", value: managedLlamaServerLaunchPreview()!.model_is_file ? "yes" : "no" },
                  { label: "Model extension valid", value: managedLlamaServerLaunchPreview()!.model_extension_valid ? "yes" : "no" },
                  { label: "Host", value: managedLlamaServerLaunchPreview()!.host },
                  { label: "Port", value: managedLlamaServerLaunchPreview()!.port },
                  { label: "Alias", value: managedLlamaServerLaunchPreview()!.alias },
                  { label: "Context window", value: managedLlamaServerLaunchPreview()!.context_window },
                  { label: "GPU layers", value: managedLlamaServerLaunchPreview()!.gpu_layers },
                ])}
                {managedLlamaServerLaunchPreview()!.safe_executable_file_name ? (
                  <p><strong>Executable file name:</strong> {managedLlamaServerLaunchPreview()!.safe_executable_file_name}</p>
                ) : null}
                {managedLlamaServerLaunchPreview()!.safe_model_file_name ? (
                  <p><strong>Model file name:</strong> {managedLlamaServerLaunchPreview()!.safe_model_file_name}</p>
                ) : null}
                <p><strong>Summary:</strong> {managedLlamaServerLaunchPreview()!.summary}</p>
                <div class="contract-meta">
                  <div><span>Preview only</span><strong>{managedLlamaServerLaunchPreview()!.preview_only ? "yes" : "no"}</strong></div>
                  <div><span>No process spawn</span><strong>{managedLlamaServerLaunchPreview()!.no_process_spawn ? "yes" : "no"}</strong></div>
                  <div><span>No model output used</span><strong>{managedLlamaServerLaunchPreview()!.no_model_output_used ? "yes" : "no"}</strong></div>
                  <div><span>No answer generation</span><strong>{managedLlamaServerLaunchPreview()!.no_answer_generation ? "yes" : "no"}</strong></div>
                  <div><span>No persistence</span><strong>{managedLlamaServerLaunchPreview()!.no_persistence ? "yes" : "no"}</strong></div>
                  <div><span>No artifact write</span><strong>{managedLlamaServerLaunchPreview()!.no_artifact_write ? "yes" : "no"}</strong></div>
                  <div><span>Localhost only</span><strong>{managedLlamaServerLaunchPreview()!.no_lan_binding_by_default ? "yes" : "no"}</strong></div>
                  <div><span>No auto start on launch</span><strong>{managedLlamaServerLaunchPreview()!.no_auto_start_on_launch ? "yes" : "no"}</strong></div>
                </div>
                {managedLlamaServerLaunchPreview()!.blockers.length > 0 ? (
                  <div class="warning-box">
                    <h4>Blockers</h4>
                    <ul>
                      {managedLlamaServerLaunchPreview()!.blockers.map((blocker) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(blocker.kind)}</strong>
                          <div>{blocker.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No launch blockers.</p>
                )}
                {managedLlamaServerLaunchPreview()!.warnings.length > 0 ? (
                  <div class="warning-box">
                    <h4>Warnings</h4>
                    <ul>
                      {managedLlamaServerLaunchPreview()!.warnings.map((warning) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(warning.kind)}</strong>
                          <div>{warning.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No launch warnings.</p>
                )}
                {managedLlamaServerLaunchPreview()!.next_required_actions.length > 0 ? (
                  <div class="warning-box">
                    <h4>Next required actions</h4>
                    <ul>
                      {managedLlamaServerLaunchPreview()!.next_required_actions.map((action) => (
                        <li>{action}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No next required actions.</p>
                )}
              </>
            ) : (
              <p>No managed launch plan preview loaded yet.</p>
            )
          ) : (
            <p>No managed launch plan preview loaded yet.</p>
          )}
          {managedLlamaServerStatusLoading() ? (
            <p>Updating managed llama-server status...</p>
          ) : managedLlamaServerStatusHasRun() ? (
            managedLlamaServerStatusPreview() ? (
              <>
                {renderMetricGrid([
                  { label: "Lifecycle", value: formatSnakeCaseLabel(managedLlamaServerStatusPreview()!.lifecycle_status) },
                  { label: "Health", value: formatSnakeCaseLabel(managedLlamaServerStatusPreview()!.health_status) },
                  { label: "Ownership", value: managedLlamaServerStatusPreview()!.owns_active_server ? "AEGIS-owned" : managedLlamaServerStatusPreview()!.port_occupied_by_unmanaged_process ? "external / unmanaged" : "not active" },
                  { label: "Port occupancy", value: formatSnakeCaseLabel(managedLlamaServerStatusPreview()!.port_occupancy_status) },
                  { label: "Port occupied", value: managedLlamaServerStatusPreview()!.port_occupied ? "yes" : "no" },
                  { label: "Port unmanaged", value: managedLlamaServerStatusPreview()!.port_occupied_by_unmanaged_process ? "yes" : "no" },
                  { label: "Host", value: managedLlamaServerStatusPreview()!.host ?? "missing" },
                  { label: "Port", value: managedLlamaServerStatusPreview()!.port ?? "missing" },
                  { label: "Alias", value: managedLlamaServerStatusPreview()!.alias ?? "missing" },
                  { label: "Process ID", value: managedLlamaServerStatusPreview()!.process_id ?? "missing" },
                  { label: "Exit code", value: managedLlamaServerStatusPreview()!.exit_code ?? "missing" },
                  { label: "Executable file", value: managedLlamaServerStatusPreview()!.safe_executable_file_name ?? "missing" },
                  { label: "Model file", value: managedLlamaServerStatusPreview()!.safe_model_file_name ?? "missing" },
                  { label: "Health URL", value: managedLlamaServerStatusPreview()!.health_url ?? "missing" },
                  { label: "Response body truncated", value: managedLlamaServerStatusPreview()!.response_body_truncated ? "yes" : "no" },
                ])}
                {managedLlamaServerStatusPreview()!.summary && <p><strong>Summary:</strong> {managedLlamaServerStatusPreview()!.summary}</p>}
                <details class="warning-box">
                  <summary>Health preview</summary>
                  {managedLlamaServerStatusPreview()!.response_body_preview ? (
                    <pre>{managedLlamaServerStatusPreview()!.response_body_preview}</pre>
                  ) : (
                    <p>No health preview captured.</p>
                  )}
                  {managedLlamaServerStatusPreview()!.response_body_truncated ? <p class="muted">Preview truncated.</p> : null}
                </details>
                <div class="contract-meta">
                  <div><span>Preview only</span><strong>{managedLlamaServerStatusPreview()!.preview_only ? "yes" : "no"}</strong></div>
                  <div><span>No process spawn</span><strong>{managedLlamaServerStatusPreview()!.no_process_spawn ? "yes" : "no"}</strong></div>
                  <div><span>No model output used</span><strong>{managedLlamaServerStatusPreview()!.no_model_output_used ? "yes" : "no"}</strong></div>
                  <div><span>No answer generation</span><strong>{managedLlamaServerStatusPreview()!.no_answer_generation ? "yes" : "no"}</strong></div>
                  <div><span>No persistence</span><strong>{managedLlamaServerStatusPreview()!.no_persistence ? "yes" : "no"}</strong></div>
                  <div><span>No artifact write</span><strong>{managedLlamaServerStatusPreview()!.no_artifact_write ? "yes" : "no"}</strong></div>
                  <div><span>Localhost only</span><strong>{managedLlamaServerStatusPreview()!.no_lan_binding_by_default ? "yes" : "no"}</strong></div>
                </div>
                {managedLlamaServerStatusPreview()!.blockers.length > 0 ? (
                  <div class="warning-box">
                    <h4>Blockers</h4>
                    <ul>
                      {managedLlamaServerStatusPreview()!.blockers.map((blocker) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(blocker.kind)}</strong>
                          <div>{blocker.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No status blockers.</p>
                )}
                {managedLlamaServerStatusPreview()!.warnings.length > 0 ? (
                  <div class="warning-box">
                    <h4>Warnings</h4>
                    <ul>
                      {managedLlamaServerStatusPreview()!.warnings.map((warning) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(warning.kind)}</strong>
                          <div>{warning.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No status warnings.</p>
                )}
                {managedLlamaServerStatusPreview()!.next_required_actions.length > 0 ? (
                  <div class="warning-box">
                    <h4>Next required actions</h4>
                    <ul>
                      {managedLlamaServerStatusPreview()!.next_required_actions.map((action) => (
                        <li>{action}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No next required actions.</p>
                )}
              </>
            ) : (
              <p>No managed server status preview loaded yet.</p>
            )
          ) : (
            <p>No managed server status preview loaded yet.</p>
          )}
        </div>
        <div class="artifact-overview runtime-setup-card">
          <h3>Chat diagnostic</h3>
          <p class="muted">
            Diagnostic-only local request. This does not create a Scholar Chat answer. Requires managed server health to be ready.
          </p>
          <p class="muted">{managedLlamaServerReadinessSummary()}</p>
          <div class="form-row">
            <label>
              Prompt
              <textarea
                value={managedLlamaServerChatDiagnosticPrompt()}
                onInput={(event) => {
                  setManagedLlamaServerChatDiagnosticPrompt(event.currentTarget.value);
                  clearManagedLlamaServerChatDiagnosticPreview();
                }}
                rows={3}
                placeholder="Say READY in one short sentence."
              />
            </label>
          </div>
          <div class="form-row">
            <label>
              Max tokens
              <input
                type="number"
                value={managedLlamaServerChatDiagnosticMaxTokens()}
                onInput={(event) => {
                  setManagedLlamaServerChatDiagnosticMaxTokens(event.currentTarget.value);
                  clearManagedLlamaServerChatDiagnosticPreview();
                }}
                placeholder="16"
              />
            </label>
            <label>
              Temperature
              <input
                type="number"
                step="0.1"
                value={managedLlamaServerChatDiagnosticTemperature()}
                onInput={(event) => {
                  setManagedLlamaServerChatDiagnosticTemperature(event.currentTarget.value);
                  clearManagedLlamaServerChatDiagnosticPreview();
                }}
                placeholder="0.2"
              />
            </label>
            <label>
              Timeout ms
              <input
                type="number"
                value={managedLlamaServerChatDiagnosticTimeoutMs()}
                onInput={(event) => {
                  setManagedLlamaServerChatDiagnosticTimeoutMs(event.currentTarget.value);
                  clearManagedLlamaServerChatDiagnosticPreview();
                }}
                placeholder="5000"
              />
            </label>
            <label class="inline-field">
              <input
                type="checkbox"
                checked={managedLlamaServerChatDiagnosticAllowRun()}
                onChange={(event) => {
                  setManagedLlamaServerChatDiagnosticAllowRun(event.currentTarget.checked);
                  clearManagedLlamaServerChatDiagnosticPreview();
                }}
              />
              I understand this is diagnostic-only.
            </label>
          </div>
          <div class="hero-actions">
            <button onClick={runManagedLlamaServerChatDiagnostic} disabled={managedLlamaServerChatDiagnosticLoading()}>
              {managedLlamaServerChatDiagnosticLoading() ? "Running..." : "Run chat diagnostic"}
            </button>
          </div>
          <p class="muted">This does not create a Scholar Chat answer and stays inside the managed localhost server boundary.</p>
          {managedLlamaServerChatDiagnosticError() && <p class="error">{managedLlamaServerChatDiagnosticError()}</p>}
          {managedLlamaServerChatDiagnosticLoading() ? (
            <p>Running managed chat diagnostic...</p>
          ) : managedLlamaServerChatDiagnosticHasRun() ? (
            managedLlamaServerChatDiagnosticPreview() ? (
              <>
                {renderMetricGrid([
                  { label: "Status", value: formatSnakeCaseLabel(managedLlamaServerChatDiagnosticPreview()!.status) },
                  { label: "Lifecycle", value: formatSnakeCaseLabel(managedLlamaServerChatDiagnosticPreview()!.lifecycle_status) },
                  { label: "Health", value: formatSnakeCaseLabel(managedLlamaServerChatDiagnosticPreview()!.health_status) },
                  { label: "Host", value: managedLlamaServerChatDiagnosticPreview()!.host ?? "missing" },
                  { label: "Port", value: managedLlamaServerChatDiagnosticPreview()!.port ?? "missing" },
                  { label: "Alias", value: managedLlamaServerChatDiagnosticPreview()!.alias ?? "missing" },
                  { label: "Model", value: managedLlamaServerChatDiagnosticPreview()!.safe_model_file_name ?? "missing" },
                  { label: "Prompt chars", value: managedLlamaServerChatDiagnosticPreview()!.prompt_char_count },
                  { label: "Max tokens", value: managedLlamaServerChatDiagnosticPreview()!.max_tokens },
                  { label: "Temperature", value: managedLlamaServerChatDiagnosticPreview()!.temperature },
                  { label: "Timeout ms", value: managedLlamaServerChatDiagnosticPreview()!.timeout_ms },
                  { label: "HTTP status", value: managedLlamaServerChatDiagnosticPreview()!.http_status ?? "missing" },
                  { label: "Duration ms", value: managedLlamaServerChatDiagnosticPreview()!.duration_ms },
                ])}
                <p><strong>Summary:</strong> {managedLlamaServerChatDiagnosticPreview()!.summary}</p>
                {managedLlamaServerChatDiagnosticPreview()!.extracted_message_preview ? (
                  <p><strong>Extracted message preview:</strong> {managedLlamaServerChatDiagnosticPreview()!.extracted_message_preview}</p>
                ) : (
                  <p>No extracted assistant message preview.</p>
                )}
                <div class="contract-meta">
                  <div><span>Diagnostic only</span><strong>{managedLlamaServerChatDiagnosticPreview()!.diagnostic_only ? "yes" : "no"}</strong></div>
                  <div><span>Not Scholar Chat answer</span><strong>{managedLlamaServerChatDiagnosticPreview()!.not_scholar_chat_answer ? "yes" : "no"}</strong></div>
                  <div><span>No final answer</span><strong>{managedLlamaServerChatDiagnosticPreview()!.no_final_answer_created ? "yes" : "no"}</strong></div>
                  <div><span>No grounding applied</span><strong>{managedLlamaServerChatDiagnosticPreview()!.no_grounding_applied ? "yes" : "no"}</strong></div>
                  <div><span>No artifact write</span><strong>{managedLlamaServerChatDiagnosticPreview()!.no_artifact_write ? "yes" : "no"}</strong></div>
                  <div><span>No persistence</span><strong>{managedLlamaServerChatDiagnosticPreview()!.no_persistence ? "yes" : "no"}</strong></div>
                  <div><span>Request attempted</span><strong>{managedLlamaServerChatDiagnosticPreview()!.request_attempted ? "yes" : "no"}</strong></div>
                </div>
                {managedLlamaServerChatDiagnosticPreview()!.blockers.length > 0 ? (
                  <div class="warning-box">
                    <h4>Blockers</h4>
                    <ul>
                      {managedLlamaServerChatDiagnosticPreview()!.blockers.map((blocker) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(blocker.kind)}</strong>
                          <div>{blocker.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No diagnostic blockers.</p>
                )}
                {managedLlamaServerChatDiagnosticPreview()!.warnings.length > 0 ? (
                  <div class="warning-box">
                    <h4>Warnings</h4>
                    <ul>
                      {managedLlamaServerChatDiagnosticPreview()!.warnings.map((warning) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(warning.kind)}</strong>
                          <div>{warning.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No diagnostic warnings.</p>
                )}
                {managedLlamaServerChatDiagnosticPreview()!.next_required_actions.length > 0 ? (
                  <div class="warning-box">
                    <h4>Next required actions</h4>
                    <ul>
                      {managedLlamaServerChatDiagnosticPreview()!.next_required_actions.map((action) => (
                        <li>{action}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No next required actions.</p>
                )}
                <details class="warning-box">
                  <summary>Raw response preview</summary>
                  {managedLlamaServerChatDiagnosticPreview()!.response_preview ? (
                    <pre>{managedLlamaServerChatDiagnosticPreview()!.response_preview}</pre>
                  ) : (
                    <p>No raw response preview captured.</p>
                  )}
                  {managedLlamaServerChatDiagnosticPreview()!.response_preview_truncated ? <p class="muted">Preview truncated.</p> : null}
                </details>
              </>
            ) : (
              <p>No managed chat diagnostic preview loaded yet.</p>
            )
          ) : (
            <p>No managed chat diagnostic preview loaded yet.</p>
          )}
        </div>
        <div class="artifact-overview runtime-setup-card">
          <h3>Local model setup</h3>
          <p class="muted">
            Secondary runtime setup and readiness preview. Select the exact `.gguf` model file, not just the folder, and select a llama.cpp executable such as `llama-cli.exe`.
          </p>
          <p class="muted">
            Gemma models may need chat-template review before future answer generation. Diagnostics do not create final answers.
          </p>
          <ol class="runtime-sequence">
            <li>Enter the exact `.gguf` model file path.</li>
            <li>Enter the `llama-cli.exe` executable path.</li>
            <li>Preview local runtime readiness.</li>
            <li>Validate the adapter setup.</li>
            <li>Run the version probe only after explicit consent.</li>
            <li>Run the smoke diagnostic only after explicit consent.</li>
          </ol>
          <div class="form-row">
            <label>
              Runtime kind
              <select
                value={localRuntimeKind()}
              onChange={(event) => {
                setLocalRuntimeKind(event.currentTarget.value as LocalModelRuntimeKind);
                clearLocalRuntimePreview();
                clearLocalRuntimeInvocationPreview();
                clearLocalRuntimeProbePreview();
                clearLocalRuntimeSmokePreview();
                clearScholarChatDraftInferencePreview();
              }}
              >
                <option value="none">None</option>
                <option value="llama_cpp">llama.cpp</option>
              </select>
            </label>
            <label>
              Context window
              <input
                type="number"
                value={localRuntimeContextWindow()}
                onInput={(event) => {
                  setLocalRuntimeContextWindow(event.currentTarget.value);
                  clearLocalRuntimePreview();
                  clearLocalRuntimeInvocationPreview();
                  clearLocalRuntimeProbePreview();
                  clearLocalRuntimeSmokePreview();
                  clearScholarChatDraftInferencePreview();
                }}
                placeholder="4096"
              />
            </label>
            <label>
              GPU layers
              <input
                type="number"
                value={localRuntimeGpuLayers()}
                onInput={(event) => {
                  setLocalRuntimeGpuLayers(event.currentTarget.value);
                  clearLocalRuntimePreview();
                  clearLocalRuntimeInvocationPreview();
                  clearLocalRuntimeProbePreview();
                  clearLocalRuntimeSmokePreview();
                  clearScholarChatDraftInferencePreview();
                }}
                placeholder="0"
              />
            </label>
            <label>
              Temperature
              <input
                type="number"
                step="0.1"
                value={localRuntimeTemperature()}
                onInput={(event) => {
                  setLocalRuntimeTemperature(event.currentTarget.value);
                  clearLocalRuntimePreview();
                  clearLocalRuntimeInvocationPreview();
                  clearLocalRuntimeProbePreview();
                  clearLocalRuntimeSmokePreview();
                  clearScholarChatDraftInferencePreview();
                }}
                placeholder="0.2"
              />
            </label>
          </div>
          <div class="form-row">
            <label>
              Model path
              <input
                type="text"
                value={localRuntimeModelPath()}
                onInput={(event) => {
                  setLocalRuntimeModelPath(event.currentTarget.value);
                  clearLocalRuntimePreview();
                  clearLocalRuntimeInvocationPreview();
                  clearLocalRuntimeProbePreview();
                  clearLocalRuntimeSmokePreview();
                  clearScholarChatDraftInferencePreview();
                }}
                placeholder="C:\\path\\to\\model.gguf"
              />
            </label>
            <label>
              Executable path
              <input
                type="text"
                value={localRuntimeExecutablePath()}
                onInput={(event) => {
                  setLocalRuntimeExecutablePath(event.currentTarget.value);
                  clearLocalRuntimePreview();
                  clearLocalRuntimeInvocationPreview();
                  clearLocalRuntimeProbePreview();
                  clearLocalRuntimeSmokePreview();
                  clearScholarChatDraftInferencePreview();
                }}
                placeholder="C:\\path\\to\\llama-cli.exe"
              />
            </label>
          </div>
          <div class="hero-actions">
            <button onClick={previewLocalRuntimeHealth} disabled={localRuntimeLoading()}>
              {localRuntimeLoading() ? "Previewing..." : "Preview local runtime readiness"}
            </button>
          </div>
          <p class="muted">Use the exact `.gguf` file, not only the model folder. Use `llama-cli.exe` from a llama.cpp Windows release.</p>
          <p class="muted">No model is executed. No answer will be generated. Configuration is not persisted.</p>
          {localRuntimeValidationError() && <p class="error">{localRuntimeValidationError()}</p>}
          {localRuntimeError() && <p class="error">{localRuntimeError()}</p>}
          {localRuntimeLoading() ? (
            <p>Previewing local runtime health...</p>
          ) : localRuntimeHasRun() ? (
            localRuntimePreview() ? (
              <>
                <div class="contract-meta">
                  <div><span>Status</span><strong>{formatSnakeCaseLabel(localRuntimePreview()!.status)}</strong></div>
                  <div><span>Runtime kind</span><strong>{formatSnakeCaseLabel(localRuntimePreview()!.runtime_kind)}</strong></div>
                  <div><span>Model state</span><strong>{formatSnakeCaseLabel(localRuntimePreview()!.model_state)}</strong></div>
                  <div><span>Executable state</span><strong>{formatSnakeCaseLabel(localRuntimePreview()!.executable_state)}</strong></div>
                  <div><span>Model extension valid</span><strong>{localRuntimePreview()!.model_extension_valid ? "yes" : "no"}</strong></div>
                  <div><span>Context window</span><strong>{localRuntimePreview()!.context_window ?? "missing"}</strong></div>
                  <div><span>GPU layers</span><strong>{localRuntimePreview()!.gpu_layers ?? "missing"}</strong></div>
                  <div><span>Temperature</span><strong>{localRuntimePreview()!.temperature ?? "missing"}</strong></div>
                </div>
                {localRuntimePreview()!.model_file_name ? (
                  <p><strong>Model file name:</strong> {localRuntimePreview()!.model_file_name}</p>
                ) : null}
                {localRuntimePreview()!.warnings.length > 0 ? (
                  <div class="warning-box">
                    <h4>Warnings</h4>
                    <ul>
                      {localRuntimePreview()!.warnings.map((warning) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(warning.kind)}</strong>
                          <div>{warning.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No runtime warnings.</p>
                )}
              </>
            ) : (
              <p class="muted">Run readiness preview to see status, blockers, warnings, and next actions.</p>
            )
          ) : (
            <p class="muted">Run readiness preview to see status, blockers, warnings, and next actions.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>llama.cpp adapter contract</h3>
          <p class="muted">
            Adapter contract preview only - no process was started, no model was loaded, no runtime execution or LLM call occurred, and no settings or artifacts were persisted.
          </p>
          <p class="muted">Previews a future llama.cpp / GGUF adapter contract only. Gemma and other families remain preview metadata, not bundled models.</p>
          <div class="form-row">
            <label>
              Executable path
              <input
                type="text"
                value={localRuntimeAdapterExecutablePath()}
                onInput={(event) => {
                  setLocalRuntimeAdapterExecutablePath(event.currentTarget.value);
                  clearLocalRuntimeAdapterContractPreview();
                }}
                placeholder="C:\\path\\to\\llama-cli.exe"
              />
            </label>
            <label>
              Model path
              <input
                type="text"
                value={localRuntimeAdapterModelPath()}
                onInput={(event) => {
                  setLocalRuntimeAdapterModelPath(event.currentTarget.value);
                  clearLocalRuntimeAdapterContractPreview();
                }}
                placeholder="C:\\path\\to\\model.gguf"
              />
            </label>
          </div>
          <p class="muted">Select the exact `.gguf` file and a matching llama.cpp executable before using preview or probe actions.</p>
          <div class="form-row">
            <label>
              Model family
              <input
                type="text"
                value={localRuntimeAdapterModelFamily()}
                onInput={(event) => {
                  setLocalRuntimeAdapterModelFamily(event.currentTarget.value);
                  clearLocalRuntimeAdapterContractPreview();
                }}
                placeholder="gemma"
              />
            </label>
            <label>
              Model format
              <input
                type="text"
                value={localRuntimeAdapterModelFormat()}
                onInput={(event) => {
                  setLocalRuntimeAdapterModelFormat(event.currentTarget.value);
                  clearLocalRuntimeAdapterContractPreview();
                }}
                placeholder="gguf"
              />
            </label>
          </div>
          <div class="form-row">
            <label>
              Context window tokens
              <input
                type="number"
                value={localRuntimeAdapterContextWindowTokens()}
                onInput={(event) => {
                  setLocalRuntimeAdapterContextWindowTokens(event.currentTarget.value);
                  clearLocalRuntimeAdapterContractPreview();
                }}
                placeholder="8192"
              />
            </label>
            <label>
              GPU layers
              <input
                type="number"
                value={localRuntimeAdapterGpuLayers()}
                onInput={(event) => {
                  setLocalRuntimeAdapterGpuLayers(event.currentTarget.value);
                  clearLocalRuntimeAdapterContractPreview();
                }}
                placeholder="0"
              />
            </label>
            <label>
              Threads
              <input
                type="number"
                value={localRuntimeAdapterThreads()}
                onInput={(event) => {
                  setLocalRuntimeAdapterThreads(event.currentTarget.value);
                  clearLocalRuntimeAdapterContractPreview();
                }}
                placeholder="8"
              />
            </label>
            <label>
              Batch size
              <input
                type="number"
                value={localRuntimeAdapterBatchSize()}
                onInput={(event) => {
                  setLocalRuntimeAdapterBatchSize(event.currentTarget.value);
                  clearLocalRuntimeAdapterContractPreview();
                }}
                placeholder="256"
              />
            </label>
          </div>
          <label>
            Chat template
            <textarea
              value={localRuntimeAdapterChatTemplate()}
              onInput={(event) => {
                setLocalRuntimeAdapterChatTemplate(event.currentTarget.value);
                clearLocalRuntimeAdapterContractPreview();
              }}
              rows={3}
              placeholder="<start_of_turn>user ...</start_of_turn>"
            />
          </label>
          <div class="hero-actions">
            <button onClick={previewLocalRuntimeAdapterContract} disabled={localRuntimeAdapterLoading()}>
              {localRuntimeAdapterLoading() ? "Previewing..." : "Preview llama.cpp adapter contract"}
            </button>
          </div>
          <p class="muted">No process is executed. No model is loaded. No settings or artifacts are persisted.</p>
          {localRuntimeAdapterValidationError() && <p class="error">{localRuntimeAdapterValidationError()}</p>}
          {localRuntimeAdapterError() && <p class="error">{localRuntimeAdapterError()}</p>}
          {localRuntimeAdapterLoading() ? (
            <p>Previewing llama.cpp adapter contract...</p>
          ) : localRuntimeAdapterHasRun() ? (
            localRuntimeAdapterPreview() ? (
              <>
                {renderMetricGrid([
                  { label: "Status", value: formatSnakeCaseLabel(localRuntimeAdapterPreview()!.status) },
                  { label: "Adapter kind", value: formatSnakeCaseLabel(localRuntimeAdapterPreview()!.adapter_kind) },
                  { label: "Normalized model family", value: localRuntimeAdapterPreview()!.normalized_model_family ?? "missing" },
                  { label: "Normalized model format", value: localRuntimeAdapterPreview()!.normalized_model_format },
                  { label: "Executable path present", value: localRuntimeAdapterPreview()!.executable_path_present ? "yes" : "no" },
                  { label: "Model path present", value: localRuntimeAdapterPreview()!.model_path_present ? "yes" : "no" },
                  { label: "Context window tokens", value: localRuntimeAdapterPreview()!.context_window_tokens ?? "missing" },
                  { label: "GPU layers", value: localRuntimeAdapterPreview()!.gpu_layers ?? "missing" },
                  { label: "Threads", value: localRuntimeAdapterPreview()!.threads ?? "missing" },
                  { label: "Batch size", value: localRuntimeAdapterPreview()!.batch_size ?? "missing" },
                  { label: "Chat template present", value: localRuntimeAdapterPreview()!.chat_template_present ? "yes" : "no" },
                ])}
                {localRuntimeAdapterPreview()!.summary && <p><strong>Summary:</strong> {localRuntimeAdapterPreview()!.summary}</p>}
                <div class="contract-meta">
                  <div><span>Preview only</span><strong>{localRuntimeAdapterPreview()!.preview_only ? "yes" : "no"}</strong></div>
                  <div><span>No process spawn</span><strong>{localRuntimeAdapterPreview()!.no_process_spawn ? "yes" : "no"}</strong></div>
                  <div><span>No model load</span><strong>{localRuntimeAdapterPreview()!.no_model_load ? "yes" : "no"}</strong></div>
                  <div><span>No runtime execution</span><strong>{localRuntimeAdapterPreview()!.no_runtime_execution ? "yes" : "no"}</strong></div>
                  <div><span>No LLM call</span><strong>{localRuntimeAdapterPreview()!.no_llm_call ? "yes" : "no"}</strong></div>
                  <div><span>No persistence</span><strong>{localRuntimeAdapterPreview()!.no_persistence ? "yes" : "no"}</strong></div>
                  <div><span>No artifact write</span><strong>{localRuntimeAdapterPreview()!.no_artifact_write ? "yes" : "no"}</strong></div>
                  <div><span>No registry status change</span><strong>{localRuntimeAdapterPreview()!.no_registry_status_change ? "yes" : "no"}</strong></div>
                  <div><span>No audit write</span><strong>{localRuntimeAdapterPreview()!.no_audit_write ? "yes" : "no"}</strong></div>
                </div>
                <h4>Required inputs</h4>
                {localRuntimeAdapterPreview()!.required_inputs.length > 0 ? (
                  <ul>
                    {localRuntimeAdapterPreview()!.required_inputs.map((item) => (
                      <li>{item}</li>
                    ))}
                  </ul>
                ) : (
                  <p>No required inputs.</p>
                )}
                <h4>Missing inputs</h4>
                {localRuntimeAdapterPreview()!.missing_inputs.length > 0 ? (
                  <ul>
                    {localRuntimeAdapterPreview()!.missing_inputs.map((item) => (
                      <li>{item}</li>
                    ))}
                  </ul>
                ) : (
                  <p>No missing inputs.</p>
                )}
                <h4>Contract reasons</h4>
                {localRuntimeAdapterPreview()!.contract_reasons.length > 0 ? (
                  <ul>
                    {localRuntimeAdapterPreview()!.contract_reasons.map((item) => (
                      <li>{item}</li>
                    ))}
                  </ul>
                ) : (
                  <p>No contract reasons.</p>
                )}
                {localRuntimeAdapterPreview()!.blockers.length > 0 ? (
                  <div class="warning-box">
                    <h4>Blockers</h4>
                    <ul>
                      {localRuntimeAdapterPreview()!.blockers.map((blocker) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(blocker.kind)}</strong>
                          <div>{blocker.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No adapter blockers.</p>
                )}
                {localRuntimeAdapterPreview()!.warnings.length > 0 ? (
                  <div class="warning-box">
                    <h4>Warnings</h4>
                    <ul>
                      {localRuntimeAdapterPreview()!.warnings.map((warning) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(warning.kind)}</strong>
                          <div>{warning.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No adapter warnings.</p>
                )}
                <h4>Next required actions</h4>
                {localRuntimeAdapterPreview()!.next_required_actions.length > 0 ? (
                  <ul>
                    {localRuntimeAdapterPreview()!.next_required_actions.map((item) => (
                      <li>{item}</li>
                    ))}
                  </ul>
                ) : (
                  <p>No next required actions.</p>
                )}
              </>
            ) : (
              <p class="muted">Run adapter preview to see compatibility, blockers, warnings, and next actions.</p>
            )
          ) : (
            <p class="muted">Run adapter preview to see compatibility, blockers, warnings, and next actions.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>llama.cpp validation</h3>
          <p class="muted">
            Validation preview only - no binary was probed, no process was started, no model was loaded, no runtime execution or LLM call occurred, and no settings or artifacts were persisted.
          </p>
          <p class="muted">
            Checks path presence plus lightweight metadata for a future GGUF runtime. It reuses the adapter contract inputs above.
          </p>
          <div class="hero-actions">
            <button onClick={previewLocalRuntimeValidation} disabled={localRuntimeValidationPreviewLoading()}>
              {localRuntimeValidationPreviewLoading() ? "Previewing..." : "Preview llama.cpp validation"}
            </button>
          </div>
          {localRuntimeValidationPreviewInputError() && <p class="error">{localRuntimeValidationPreviewInputError()}</p>}
          {localRuntimeValidationPreviewError() && <p class="error">{localRuntimeValidationPreviewError()}</p>}
          {localRuntimeValidationPreviewLoading() ? (
            <p>Previewing llama.cpp validation...</p>
          ) : localRuntimeValidationPreviewHasRun() ? (
            localRuntimeValidationPreview() ? (
              <>
                {renderMetricGrid([
                  { label: "Status", value: formatSnakeCaseLabel(localRuntimeValidationPreview()!.status) },
                  { label: "Adapter contract status", value: formatSnakeCaseLabel(localRuntimeValidationPreview()!.adapter_contract_status) },
                  { label: "Adapter kind", value: formatSnakeCaseLabel(localRuntimeValidationPreview()!.adapter_kind) },
                  { label: "Normalized model family", value: localRuntimeValidationPreview()!.normalized_model_family ?? "missing" },
                  { label: "Normalized model format", value: localRuntimeValidationPreview()!.normalized_model_format },
                  { label: "Executable path present", value: localRuntimeValidationPreview()!.executable_path_present ? "yes" : "no" },
                  { label: "Executable exists", value: localRuntimeValidationPreview()!.executable_exists ? "yes" : "no" },
                  { label: "Executable is file", value: localRuntimeValidationPreview()!.executable_is_file ? "yes" : "no" },
                  { label: "Model path present", value: localRuntimeValidationPreview()!.model_path_present ? "yes" : "no" },
                  { label: "Model exists", value: localRuntimeValidationPreview()!.model_exists ? "yes" : "no" },
                  { label: "Model is file", value: localRuntimeValidationPreview()!.model_is_file ? "yes" : "no" },
                  { label: "Model extension valid", value: localRuntimeValidationPreview()!.model_extension_valid ? "yes" : "no" },
                  { label: "Context window tokens", value: localRuntimeValidationPreview()!.context_window_tokens ?? "missing" },
                  { label: "GPU layers", value: localRuntimeValidationPreview()!.gpu_layers ?? "missing" },
                  { label: "Threads", value: localRuntimeValidationPreview()!.threads ?? "missing" },
                  { label: "Batch size", value: localRuntimeValidationPreview()!.batch_size ?? "missing" },
                  { label: "Chat template present", value: localRuntimeValidationPreview()!.chat_template_present ? "yes" : "no" },
                  { label: "Safe executable file name", value: localRuntimeValidationPreview()!.safe_executable_file_name ?? "not configured" },
                  { label: "Safe model file name", value: localRuntimeValidationPreview()!.safe_model_file_name ?? "not configured" },
                ])}
                {localRuntimeValidationPreview()!.summary && <p><strong>Summary:</strong> {localRuntimeValidationPreview()!.summary}</p>}
                <div class="contract-meta">
                  <div><span>Preview only</span><strong>{localRuntimeValidationPreview()!.preview_only ? "yes" : "no"}</strong></div>
                  <div><span>No process spawn</span><strong>{localRuntimeValidationPreview()!.no_process_spawn ? "yes" : "no"}</strong></div>
                  <div><span>No binary probe</span><strong>{localRuntimeValidationPreview()!.no_binary_probe ? "yes" : "no"}</strong></div>
                  <div><span>No model load</span><strong>{localRuntimeValidationPreview()!.no_model_load ? "yes" : "no"}</strong></div>
                  <div><span>No runtime execution</span><strong>{localRuntimeValidationPreview()!.no_runtime_execution ? "yes" : "no"}</strong></div>
                  <div><span>No LLM call</span><strong>{localRuntimeValidationPreview()!.no_llm_call ? "yes" : "no"}</strong></div>
                  <div><span>No persistence</span><strong>{localRuntimeValidationPreview()!.no_persistence ? "yes" : "no"}</strong></div>
                  <div><span>No artifact write</span><strong>{localRuntimeValidationPreview()!.no_artifact_write ? "yes" : "no"}</strong></div>
                  <div><span>No registry status change</span><strong>{localRuntimeValidationPreview()!.no_registry_status_change ? "yes" : "no"}</strong></div>
                  <div><span>No audit write</span><strong>{localRuntimeValidationPreview()!.no_audit_write ? "yes" : "no"}</strong></div>
                </div>
                <h4>Missing inputs</h4>
                {localRuntimeValidationPreview()!.missing_inputs.length > 0 ? (
                  <ul>
                    {localRuntimeValidationPreview()!.missing_inputs.map((item) => (
                      <li>{item}</li>
                    ))}
                  </ul>
                ) : (
                  <p>No missing inputs.</p>
                )}
                <h4>Validation reasons</h4>
                {localRuntimeValidationPreview()!.validation_reasons.length > 0 ? (
                  <ul>
                    {localRuntimeValidationPreview()!.validation_reasons.map((item) => (
                      <li>{item}</li>
                    ))}
                  </ul>
                ) : (
                  <p>No validation reasons.</p>
                )}
                <h4>Blockers</h4>
                {localRuntimeValidationPreview()!.blockers.length > 0 ? (
                  <ul>
                    {localRuntimeValidationPreview()!.blockers.map((blocker) => (
                      <li>
                        <strong>{formatSnakeCaseLabel(blocker.kind)}</strong>: {blocker.message}
                      </li>
                    ))}
                  </ul>
                ) : (
                  <p>No validation blockers.</p>
                )}
                <h4>Warnings</h4>
                {localRuntimeValidationPreview()!.warnings.length > 0 ? (
                  <ul>
                    {localRuntimeValidationPreview()!.warnings.map((warning) => (
                      <li>
                        <strong>{formatSnakeCaseLabel(warning.kind)}</strong>: {warning.message}
                      </li>
                    ))}
                  </ul>
                ) : (
                  <p>No validation warnings.</p>
                )}
                <h4>Next required actions</h4>
                {localRuntimeValidationPreview()!.next_required_actions.length > 0 ? (
                  <ul>
                    {localRuntimeValidationPreview()!.next_required_actions.map((item) => (
                      <li>{item}</li>
                    ))}
                  </ul>
                ) : (
                  <p>No next required actions.</p>
                )}
              </>
            ) : (
              <p class="muted">Run validation preview to see path checks, blockers, warnings, and next actions.</p>
            )
          ) : (
            <p class="muted">Run validation preview to see path checks, blockers, warnings, and next actions.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>llama.cpp probe readiness</h3>
          <p class="muted">
            Probe readiness preview only - no binary was probed, no process was started, no model was loaded, no runtime execution or LLM call occurred, and no settings or artifacts were persisted.
          </p>
          <p class="muted">Uses the current validation inputs above and only prepares a future binary probe preview.</p>
          <div class="form-row">
            <label class="inline-field">
              <input
                type="checkbox"
                checked={localRuntimeProbeReadinessConsent()}
                onChange={(event) => {
                  setLocalRuntimeProbeReadinessConsent(event.currentTarget.checked);
                  clearLocalRuntimeProbeReadinessPreview();
                  clearLocalRuntimeProbePreview();
                }}
              />
              I understand this only prepares a future binary probe preview.
            </label>
          </div>
          <p class="muted">Version probe actions remain consent-gated and do not load a model or generate answers.</p>
          <div class="hero-actions">
            <button onClick={previewLocalRuntimeProbeReadiness} disabled={localRuntimeProbeReadinessPreviewLoading()}>
              {localRuntimeProbeReadinessPreviewLoading() ? "Previewing..." : "Preview llama.cpp probe readiness"}
            </button>
          </div>
          {localRuntimeProbeReadinessPreviewInputError() && <p class="error">{localRuntimeProbeReadinessPreviewInputError()}</p>}
          {localRuntimeProbeReadinessPreviewError() && <p class="error">{localRuntimeProbeReadinessPreviewError()}</p>}
          {localRuntimeProbeReadinessPreviewLoading() ? (
            <p>Previewing llama.cpp probe readiness...</p>
          ) : localRuntimeProbeReadinessPreviewHasRun() ? (
            localRuntimeProbeReadinessPreview() ? (
              <>
                {renderMetricGrid([
                  { label: "Status", value: formatSnakeCaseLabel(localRuntimeProbeReadinessPreview()!.status) },
                  { label: "Validation status", value: formatSnakeCaseLabel(localRuntimeProbeReadinessPreview()!.validation_status) },
                  { label: "Adapter contract status", value: formatSnakeCaseLabel(localRuntimeProbeReadinessPreview()!.adapter_contract_status) },
                  { label: "Adapter kind", value: formatSnakeCaseLabel(localRuntimeProbeReadinessPreview()!.adapter_kind) },
                  { label: "Normalized model family", value: localRuntimeProbeReadinessPreview()!.normalized_model_family ?? "missing" },
                  { label: "Normalized model format", value: localRuntimeProbeReadinessPreview()!.normalized_model_format },
                  { label: "Executable path present", value: localRuntimeProbeReadinessPreview()!.executable_path_present ? "yes" : "no" },
                  { label: "Executable exists", value: localRuntimeProbeReadinessPreview()!.executable_exists ? "yes" : "no" },
                  { label: "Executable is file", value: localRuntimeProbeReadinessPreview()!.executable_is_file ? "yes" : "no" },
                  { label: "Model path present", value: localRuntimeProbeReadinessPreview()!.model_path_present ? "yes" : "no" },
                  { label: "Model exists", value: localRuntimeProbeReadinessPreview()!.model_exists ? "yes" : "no" },
                  { label: "Model is file", value: localRuntimeProbeReadinessPreview()!.model_is_file ? "yes" : "no" },
                  { label: "Model extension valid", value: localRuntimeProbeReadinessPreview()!.model_extension_valid ? "yes" : "no" },
                  { label: "Probe consent", value: localRuntimeProbeReadinessPreview()!.probe_consent ? "yes" : "no" },
                  { label: "Safe executable file name", value: localRuntimeProbeReadinessPreview()!.safe_executable_file_name ?? "not configured" },
                  { label: "Safe model file name", value: localRuntimeProbeReadinessPreview()!.safe_model_file_name ?? "not configured" },
                ])}
                {localRuntimeProbeReadinessPreview()!.summary && <p><strong>Summary:</strong> {localRuntimeProbeReadinessPreview()!.summary}</p>}
                <div class="contract-meta">
                  <div><span>Preview only</span><strong>{localRuntimeProbeReadinessPreview()!.preview_only ? "yes" : "no"}</strong></div>
                  <div><span>No process spawn</span><strong>{localRuntimeProbeReadinessPreview()!.no_process_spawn ? "yes" : "no"}</strong></div>
                  <div><span>No binary probe</span><strong>{localRuntimeProbeReadinessPreview()!.no_binary_probe ? "yes" : "no"}</strong></div>
                  <div><span>No model load</span><strong>{localRuntimeProbeReadinessPreview()!.no_model_load ? "yes" : "no"}</strong></div>
                  <div><span>No runtime execution</span><strong>{localRuntimeProbeReadinessPreview()!.no_runtime_execution ? "yes" : "no"}</strong></div>
                  <div><span>No LLM call</span><strong>{localRuntimeProbeReadinessPreview()!.no_llm_call ? "yes" : "no"}</strong></div>
                  <div><span>No persistence</span><strong>{localRuntimeProbeReadinessPreview()!.no_persistence ? "yes" : "no"}</strong></div>
                  <div><span>No artifact write</span><strong>{localRuntimeProbeReadinessPreview()!.no_artifact_write ? "yes" : "no"}</strong></div>
                  <div><span>No registry status change</span><strong>{localRuntimeProbeReadinessPreview()!.no_registry_status_change ? "yes" : "no"}</strong></div>
                  <div><span>No audit write</span><strong>{localRuntimeProbeReadinessPreview()!.no_audit_write ? "yes" : "no"}</strong></div>
                </div>
                <h4>Required inputs</h4>
                {localRuntimeProbeReadinessPreview()!.required_inputs.length > 0 ? (
                  <ul>
                    {localRuntimeProbeReadinessPreview()!.required_inputs.map((item) => (
                      <li>{item}</li>
                    ))}
                  </ul>
                ) : (
                  <p>No required inputs.</p>
                )}
                <h4>Missing inputs</h4>
                {localRuntimeProbeReadinessPreview()!.missing_inputs.length > 0 ? (
                  <ul>
                    {localRuntimeProbeReadinessPreview()!.missing_inputs.map((item) => (
                      <li>{item}</li>
                    ))}
                  </ul>
                ) : (
                  <p>No missing inputs.</p>
                )}
                <h4>Readiness reasons</h4>
                {localRuntimeProbeReadinessPreview()!.readiness_reasons.length > 0 ? (
                  <ul>
                    {localRuntimeProbeReadinessPreview()!.readiness_reasons.map((item) => (
                      <li>{item}</li>
                    ))}
                  </ul>
                ) : (
                  <p>No readiness reasons.</p>
                )}
                {localRuntimeProbeReadinessPreview()!.blockers.length > 0 ? (
                  <div class="warning-box">
                    <h4>Blockers</h4>
                    <ul>
                      {localRuntimeProbeReadinessPreview()!.blockers.map((blocker) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(blocker.kind)}</strong>
                          <div>{blocker.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No probe-readiness blockers.</p>
                )}
                {localRuntimeProbeReadinessPreview()!.warnings.length > 0 ? (
                  <div class="warning-box">
                    <h4>Warnings</h4>
                    <ul>
                      {localRuntimeProbeReadinessPreview()!.warnings.map((warning) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(warning.kind)}</strong>
                          <div>{warning.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No probe-readiness warnings.</p>
                )}
                <h4>Next required actions</h4>
                {localRuntimeProbeReadinessPreview()!.next_required_actions.length > 0 ? (
                  <ul>
                    {localRuntimeProbeReadinessPreview()!.next_required_actions.map((item) => (
                      <li>{item}</li>
                    ))}
                  </ul>
                ) : (
                  <p>No next required actions.</p>
                )}
              </>
            ) : (
              <p class="muted">Run probe readiness preview to see readiness reasons, blockers, warnings, and next actions.</p>
            )
          ) : (
            <p class="muted">Run probe readiness preview to see readiness reasons, blockers, warnings, and next actions.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>llama.cpp version probe</h3>
          <p class="muted">
            Version probe only - this may run the configured binary with a version argument, but it does not pass a model path, load a model, run inference, call an LLM, or persist settings/artifacts.
          </p>
          <p class="muted">Uses the current validation and probe-readiness inputs above, plus explicit version-probe execution consent.</p>
          <div class="form-row">
            <label class="inline-field">
              <input
                type="checkbox"
                checked={localRuntimeProbeAllowExecution()}
                onChange={(event) => {
                  setLocalRuntimeProbeAllowExecution(event.currentTarget.checked);
                  clearLocalRuntimeProbePreview();
                }}
              />
              I allow running the configured llama.cpp binary with a version-only probe.
            </label>
          </div>
          <div class="form-row">
            <label>
              Timeout ms
              <input
                type="number"
                value={localRuntimeProbeTimeoutMs()}
                onInput={(event) => {
                  setLocalRuntimeProbeTimeoutMs(event.currentTarget.value);
                  clearLocalRuntimeProbePreview();
                  clearScholarChatDraftInferencePreview();
                }}
                placeholder="1500"
              />
            </label>
          </div>
          <p class="muted">{localRuntimeProbeExecutableSummary()}</p>
          <div class="hero-actions">
            <button onClick={previewLocalRuntimeVersionProbe} disabled={localRuntimeProbeLoading()}>
              {localRuntimeProbeLoading() ? "Probing..." : "Run llama.cpp version probe"}
            </button>
          </div>
          <p class="muted">
            Version probe preview only - no model path was passed, no model was loaded, no inference was run, no LLM call occurred, and no settings or artifacts were persisted.
          </p>
          {localRuntimeProbeValidationError() && <p class="error">{localRuntimeProbeValidationError()}</p>}
          {localRuntimeProbeError() && <p class="error">{localRuntimeProbeError()}</p>}
          {localRuntimeProbeLoading() ? (
            <p>Running llama.cpp version probe...</p>
          ) : localRuntimeProbeHasRun() ? (
            localRuntimeProbeResult() ? (
              <>
                {renderMetricGrid([
                  { label: "Status", value: formatSnakeCaseLabel(localRuntimeProbeResult()!.status) },
                  { label: "Probe readiness", value: formatSnakeCaseLabel(localRuntimeProbeResult()!.probe_readiness_status) },
                  { label: "Validation status", value: formatSnakeCaseLabel(localRuntimeProbeResult()!.validation_status) },
                  { label: "Adapter contract status", value: formatSnakeCaseLabel(localRuntimeProbeResult()!.adapter_contract_status) },
                  { label: "Adapter kind", value: formatSnakeCaseLabel(localRuntimeProbeResult()!.adapter_kind) },
                  { label: "Probe consent", value: localRuntimeProbeResult()!.probe_consent ? "yes" : "no" },
                  { label: "Allow probe execution", value: localRuntimeProbeResult()!.allow_probe_execution ? "yes" : "no" },
                  { label: "Execution attempted", value: localRuntimeProbeResult()!.execution_attempted ? "yes" : "no" },
                  { label: "Probe argument", value: localRuntimeProbeResult()!.probe_argument },
                  { label: "Timeout ms", value: localRuntimeProbeResult()!.timeout_ms },
                  { label: "Duration ms", value: localRuntimeProbeResult()!.duration_ms },
                  { label: "Exit code", value: localRuntimeProbeResult()!.exit_code ?? "missing" },
                  { label: "Stdout truncated", value: localRuntimeProbeResult()!.stdout_truncated ? "yes" : "no" },
                  { label: "Stderr truncated", value: localRuntimeProbeResult()!.stderr_truncated ? "yes" : "no" },
                ])}
                <div class="contract-meta">
                  <div><span>Executable file</span><strong>{localRuntimeProbeResult()!.safe_executable_file_name ?? "not configured"}</strong></div>
                  <div><span>Model file</span><strong>{localRuntimeProbeResult()!.safe_model_file_name ?? "not configured"}</strong></div>
                </div>
                {localRuntimeProbeResult()!.summary && <p><strong>Summary:</strong> {localRuntimeProbeResult()!.summary}</p>}
                <div class="contract-meta">
                  <div><span>Preview only</span><strong>{localRuntimeProbeResult()!.preview_only ? "yes" : "no"}</strong></div>
                  <div><span>No model load</span><strong>{localRuntimeProbeResult()!.no_model_load ? "yes" : "no"}</strong></div>
                  <div><span>No model path argument</span><strong>{localRuntimeProbeResult()!.no_model_path_argument ? "yes" : "no"}</strong></div>
                  <div><span>No LLM call</span><strong>{localRuntimeProbeResult()!.no_llm_call ? "yes" : "no"}</strong></div>
                  <div><span>No runtime inference</span><strong>{localRuntimeProbeResult()!.no_runtime_inference ? "yes" : "no"}</strong></div>
                  <div><span>No persistence</span><strong>{localRuntimeProbeResult()!.no_persistence ? "yes" : "no"}</strong></div>
                  <div><span>No artifact write</span><strong>{localRuntimeProbeResult()!.no_artifact_write ? "yes" : "no"}</strong></div>
                  <div><span>No registry status change</span><strong>{localRuntimeProbeResult()!.no_registry_status_change ? "yes" : "no"}</strong></div>
                  <div><span>No audit write</span><strong>{localRuntimeProbeResult()!.no_audit_write ? "yes" : "no"}</strong></div>
                </div>
                <h4>Stdout preview</h4>
                {localRuntimeProbeResult()!.stdout_preview ? (
                  <pre>{localRuntimeProbeResult()!.stdout_preview}</pre>
                ) : (
                  <p>No stdout captured.</p>
                )}
                <h4>Stderr preview</h4>
                {localRuntimeProbeResult()!.stderr_preview ? (
                  <pre>{localRuntimeProbeResult()!.stderr_preview}</pre>
                ) : (
                  <p>No stderr captured.</p>
                )}
                {localRuntimeProbeResult()!.blockers.length > 0 ? (
                  <div class="warning-box">
                    <h4>Blockers</h4>
                    <ul>
                      {localRuntimeProbeResult()!.blockers.map((blocker) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(blocker.kind)}</strong>
                          <div>{blocker.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No version-probe blockers.</p>
                )}
                {localRuntimeProbeResult()!.warnings.length > 0 ? (
                  <div class="warning-box">
                    <h4>Warnings</h4>
                    <ul>
                      {localRuntimeProbeResult()!.warnings.map((warning) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(warning.kind)}</strong>
                          <div>{warning.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No version-probe warnings.</p>
                )}
                <h4>Next required actions</h4>
                {localRuntimeProbeResult()!.next_required_actions.length > 0 ? (
                  <ul>
                    {localRuntimeProbeResult()!.next_required_actions.map((item) => (
                      <li>{item}</li>
                    ))}
                  </ul>
                ) : (
                  <p>No next required actions.</p>
                )}
              </>
            ) : (
              <p class="muted">Run the version probe to confirm the executable can start with `--version`.</p>
            )
          ) : (
              <p class="muted">Run the version probe to confirm the executable can start with `--version`.</p>
          )}
        </div>
      <div class="artifact-overview">
        <h3>llama.cpp capability preview</h3>
        <p class="muted">
          Capability preview only - this derives a path-free diagnostic summary from the version probe result. It does not run inference, pass a model path, load/read a model, call an LLM, or persist settings/artifacts.
        </p>
        <p class="muted">Uses the current validation and version-probe inputs above, plus explicit version-probe execution consent.</p>
        <div class="hero-actions">
          <button onClick={previewLocalRuntimeCapability} disabled={localRuntimeCapabilityLoading()}>
            {localRuntimeCapabilityLoading() ? "Previewing..." : "Preview llama.cpp runtime capability"}
          </button>
        </div>
        <p class="muted">{localRuntimeProbeExecutableSummary()}</p>
        {localRuntimeProbeValidationError() && <p class="error">{localRuntimeProbeValidationError()}</p>}
        {localRuntimeCapabilityError() && <p class="error">{localRuntimeCapabilityError()}</p>}
        {localRuntimeCapabilityLoading() ? (
          <p>Previewing llama.cpp runtime capability...</p>
        ) : localRuntimeCapabilityHasRun() ? (
          localRuntimeCapabilityResult() ? (
            <>
              {renderMetricGrid([
                { label: "Status", value: formatSnakeCaseLabel(localRuntimeCapabilityResult()!.status) },
                { label: "Version probe status", value: formatSnakeCaseLabel(localRuntimeCapabilityResult()!.version_probe_status) },
                { label: "Probe readiness", value: formatSnakeCaseLabel(localRuntimeCapabilityResult()!.probe_readiness_status) },
                { label: "Validation status", value: formatSnakeCaseLabel(localRuntimeCapabilityResult()!.validation_status) },
                { label: "Adapter contract status", value: formatSnakeCaseLabel(localRuntimeCapabilityResult()!.adapter_contract_status) },
                { label: "Adapter kind", value: formatSnakeCaseLabel(localRuntimeCapabilityResult()!.adapter_kind) },
                { label: "Probe consent", value: localRuntimeCapabilityResult()!.probe_consent ? "yes" : "no" },
                { label: "Allow probe execution", value: localRuntimeCapabilityResult()!.allow_probe_execution ? "yes" : "no" },
                { label: "Version probe attempted", value: localRuntimeCapabilityResult()!.version_probe_execution_attempted ? "yes" : "no" },
                { label: "Version probe timed out", value: localRuntimeCapabilityResult()!.version_probe_timed_out ? "yes" : "no" },
                { label: "Version probe exit code", value: localRuntimeCapabilityResult()!.version_probe_exit_code ?? "missing" },
                { label: "Inferred runtime available", value: localRuntimeCapabilityResult()!.inferred_runtime_available ? "yes" : "no" },
                { label: "Inferred version text", value: localRuntimeCapabilityResult()!.inferred_version_text ?? "none" },
              ])}
              <div class="contract-meta">
                <div><span>Executable file</span><strong>{localRuntimeCapabilityResult()!.safe_executable_file_name ?? "not configured"}</strong></div>
                <div><span>Model file</span><strong>{localRuntimeCapabilityResult()!.safe_model_file_name ?? "not configured"}</strong></div>
                <div><span>Preview only</span><strong>{localRuntimeCapabilityResult()!.preview_only ? "yes" : "no"}</strong></div>
                <div><span>No new process spawn</span><strong>{localRuntimeCapabilityResult()!.no_new_process_spawn ? "yes" : "no"}</strong></div>
                <div><span>No binary probe beyond wrapped version probe</span><strong>{localRuntimeCapabilityResult()!.no_binary_probe_beyond_wrapped_version_probe ? "yes" : "no"}</strong></div>
                <div><span>No model path argument</span><strong>{localRuntimeCapabilityResult()!.no_model_path_argument ? "yes" : "no"}</strong></div>
                <div><span>No model file read</span><strong>{localRuntimeCapabilityResult()!.no_model_file_read ? "yes" : "no"}</strong></div>
                <div><span>No model load</span><strong>{localRuntimeCapabilityResult()!.no_model_load ? "yes" : "no"}</strong></div>
                <div><span>No runtime inference</span><strong>{localRuntimeCapabilityResult()!.no_runtime_inference ? "yes" : "no"}</strong></div>
                <div><span>No smoke inference</span><strong>{localRuntimeCapabilityResult()!.no_smoke_inference ? "yes" : "no"}</strong></div>
                <div><span>No LLM call</span><strong>{localRuntimeCapabilityResult()!.no_llm_call ? "yes" : "no"}</strong></div>
                <div><span>No persistence</span><strong>{localRuntimeCapabilityResult()!.no_persistence ? "yes" : "no"}</strong></div>
                <div><span>No artifact write</span><strong>{localRuntimeCapabilityResult()!.no_artifact_write ? "yes" : "no"}</strong></div>
                <div><span>No registry status change</span><strong>{localRuntimeCapabilityResult()!.no_registry_status_change ? "yes" : "no"}</strong></div>
                <div><span>No audit write</span><strong>{localRuntimeCapabilityResult()!.no_audit_write ? "yes" : "no"}</strong></div>
              </div>
              {localRuntimeCapabilityResult()!.summary && <p><strong>Summary:</strong> {localRuntimeCapabilityResult()!.summary}</p>}
              {localRuntimeCapabilityResult()!.capability_reasons.length > 0 ? (
                <div class="warning-box">
                  <h4>Capability reasons</h4>
                  <ul>
                    {localRuntimeCapabilityResult()!.capability_reasons.map((reason) => (
                      <li>{reason}</li>
                    ))}
                  </ul>
                </div>
              ) : (
                <p>No capability reasons.</p>
              )}
              <h4>Version probe stdout preview</h4>
              {localRuntimeCapabilityResult()!.version_probe_stdout_preview ? (
                <pre>{localRuntimeCapabilityResult()!.version_probe_stdout_preview}</pre>
              ) : (
                <p>No stdout captured.</p>
              )}
              <h4>Version probe stderr preview</h4>
              {localRuntimeCapabilityResult()!.version_probe_stderr_preview ? (
                <pre>{localRuntimeCapabilityResult()!.version_probe_stderr_preview}</pre>
              ) : (
                <p>No stderr captured.</p>
              )}
              {localRuntimeCapabilityResult()!.blockers.length > 0 ? (
                <div class="warning-box">
                  <h4>Blockers</h4>
                  <ul>
                    {localRuntimeCapabilityResult()!.blockers.map((blocker) => (
                      <li>
                        <strong>{formatSnakeCaseLabel(blocker.kind)}</strong>
                        <div>{blocker.message}</div>
                      </li>
                    ))}
                  </ul>
                </div>
              ) : (
                <p>No capability blockers.</p>
              )}
              {localRuntimeCapabilityResult()!.warnings.length > 0 ? (
                <div class="warning-box">
                  <h4>Warnings</h4>
                  <ul>
                    {localRuntimeCapabilityResult()!.warnings.map((warning) => (
                      <li>
                        <strong>{formatSnakeCaseLabel(warning.kind)}</strong>
                        <div>{warning.message}</div>
                      </li>
                    ))}
                  </ul>
                </div>
              ) : (
                <p>No capability warnings.</p>
              )}
              <h4>Next required actions</h4>
              {localRuntimeCapabilityResult()!.next_required_actions.length > 0 ? (
                <ul>
                  {localRuntimeCapabilityResult()!.next_required_actions.map((item) => (
                    <li>{item}</li>
                  ))}
                </ul>
              ) : (
                <p>No next required actions.</p>
              )}
            </>
          ) : (
            <p>No llama.cpp capability preview loaded yet.</p>
          )
        ) : (
          <p>No llama.cpp capability preview loaded yet.</p>
        )}
      </div>
        <div class="artifact-overview">
          <h3>llama.cpp smoke readiness</h3>
        <p class="muted">
          Smoke readiness only - this does not run inference, does not pass a model path to a process, does not load or read a model, does not call an LLM, and does not persist settings or artifacts.
        </p>
        <p class="muted">Uses the current capability preview inputs above and the shared smoke prompt and timeout settings, plus explicit smoke consent.</p>
        <div class="form-row">
          <label>
            Diagnostic prompt
            <textarea
              rows={3}
              value={localRuntimeSmokePrompt()}
              onInput={(event) => {
                setLocalRuntimeSmokePrompt(event.currentTarget.value);
                clearLocalRuntimeSmokePreview();
                clearLocalRuntimeSmokeReadinessPreview();
              }}
              placeholder="Describe the diagnostic smoke-check prompt..."
            />
          </label>
        </div>
        <div class="form-row">
          <label class="inline-field">
            <input
              type="checkbox"
              checked={localRuntimeSmokeReadinessConsent()}
              onChange={(event) => {
                setLocalRuntimeSmokeReadinessConsent(event.currentTarget.checked);
                clearLocalRuntimeSmokeReadinessPreview();
              }}
            />
            I understand this only prepares a future diagnostic smoke inference and does not run inference now.
          </label>
        </div>
        <div class="hero-actions">
          <button onClick={previewLocalRuntimeSmokeReadiness} disabled={localRuntimeSmokeReadinessLoading()}>
            {localRuntimeSmokeReadinessLoading() ? "Previewing..." : "Preview llama.cpp smoke readiness"}
          </button>
        </div>
        <p class="muted">This preview is read-only and shares the current smoke prompt, timeout, and max-output-token inputs.</p>
        {localRuntimeSmokeReadinessValidationError() && <p class="error">{localRuntimeSmokeReadinessValidationError()}</p>}
        {localRuntimeSmokeReadinessError() && <p class="error">{localRuntimeSmokeReadinessError()}</p>}
        {localRuntimeSmokeReadinessLoading() ? (
          <p>Previewing llama.cpp smoke readiness...</p>
        ) : localRuntimeSmokeReadinessHasRun() ? (
          localRuntimeSmokeReadinessResult() ? (
            <>
              {renderMetricGrid([
                { label: "Status", value: formatSnakeCaseLabel(localRuntimeSmokeReadinessResult()!.status) },
                { label: "Capability status", value: formatSnakeCaseLabel(localRuntimeSmokeReadinessResult()!.capability_status) },
                { label: "Version probe status", value: formatSnakeCaseLabel(localRuntimeSmokeReadinessResult()!.version_probe_status) },
                { label: "Probe readiness", value: formatSnakeCaseLabel(localRuntimeSmokeReadinessResult()!.probe_readiness_status) },
                { label: "Validation status", value: formatSnakeCaseLabel(localRuntimeSmokeReadinessResult()!.validation_status) },
                { label: "Adapter contract status", value: formatSnakeCaseLabel(localRuntimeSmokeReadinessResult()!.adapter_contract_status) },
                { label: "Adapter kind", value: formatSnakeCaseLabel(localRuntimeSmokeReadinessResult()!.adapter_kind) },
                { label: "Probe consent", value: localRuntimeSmokeReadinessResult()!.probe_consent ? "yes" : "no" },
                { label: "Allow probe execution", value: localRuntimeSmokeReadinessResult()!.allow_probe_execution ? "yes" : "no" },
                { label: "Smoke consent", value: localRuntimeSmokeReadinessResult()!.smoke_consent ? "yes" : "no" },
                { label: "Diagnostic prompt chars", value: localRuntimeSmokeReadinessResult()!.diagnostic_prompt_char_count },
                { label: "Max output tokens", value: localRuntimeSmokeReadinessResult()!.max_output_tokens },
                { label: "Timeout ms", value: localRuntimeSmokeReadinessResult()!.timeout_ms },
              ])}
              <div class="contract-meta">
                <div><span>Executable file</span><strong>{localRuntimeSmokeReadinessResult()!.safe_executable_file_name ?? "not configured"}</strong></div>
                <div><span>Model file</span><strong>{localRuntimeSmokeReadinessResult()!.safe_model_file_name ?? "not configured"}</strong></div>
                <div><span>Normalized prompt</span><strong>{localRuntimeSmokeReadinessResult()!.normalized_diagnostic_prompt || "missing"}</strong></div>
                <div><span>Preview only</span><strong>{localRuntimeSmokeReadinessResult()!.preview_only ? "yes" : "no"}</strong></div>
                <div><span>No new process spawn</span><strong>{localRuntimeSmokeReadinessResult()!.no_new_process_spawn ? "yes" : "no"}</strong></div>
                <div><span>No smoke inference execution</span><strong>{localRuntimeSmokeReadinessResult()!.no_smoke_inference_execution ? "yes" : "no"}</strong></div>
                <div><span>No model path argument</span><strong>{localRuntimeSmokeReadinessResult()!.no_model_path_argument ? "yes" : "no"}</strong></div>
                <div><span>No model file read</span><strong>{localRuntimeSmokeReadinessResult()!.no_model_file_read ? "yes" : "no"}</strong></div>
                <div><span>No model load</span><strong>{localRuntimeSmokeReadinessResult()!.no_model_load ? "yes" : "no"}</strong></div>
                <div><span>No LLM call</span><strong>{localRuntimeSmokeReadinessResult()!.no_llm_call ? "yes" : "no"}</strong></div>
                <div><span>No persistence</span><strong>{localRuntimeSmokeReadinessResult()!.no_persistence ? "yes" : "no"}</strong></div>
                <div><span>No artifact write</span><strong>{localRuntimeSmokeReadinessResult()!.no_artifact_write ? "yes" : "no"}</strong></div>
                <div><span>No registry status change</span><strong>{localRuntimeSmokeReadinessResult()!.no_registry_status_change ? "yes" : "no"}</strong></div>
                <div><span>No audit write</span><strong>{localRuntimeSmokeReadinessResult()!.no_audit_write ? "yes" : "no"}</strong></div>
                <div><span>Diagnostic only</span><strong>{localRuntimeSmokeReadinessResult()!.diagnostic_only ? "yes" : "no"}</strong></div>
                <div><span>Not Scholar Chat answer</span><strong>{localRuntimeSmokeReadinessResult()!.not_scholar_chat_answer ? "yes" : "no"}</strong></div>
                <div><span>No answer generated</span><strong>{localRuntimeSmokeReadinessResult()!.no_answer_generated ? "yes" : "no"}</strong></div>
                <div><span>No grounding applied</span><strong>{localRuntimeSmokeReadinessResult()!.no_grounding_applied ? "yes" : "no"}</strong></div>
                <div><span>No Evidence Pack used</span><strong>{localRuntimeSmokeReadinessResult()!.no_evidence_pack_used ? "yes" : "no"}</strong></div>
              </div>
              {localRuntimeSmokeReadinessResult()!.summary && <p><strong>Summary:</strong> {localRuntimeSmokeReadinessResult()!.summary}</p>}
              <div class="contract-meta">
                <div><span>Required inputs</span><strong>{localRuntimeSmokeReadinessResult()!.required_inputs.join(", ") || "none"}</strong></div>
                <div><span>Missing inputs</span><strong>{localRuntimeSmokeReadinessResult()!.missing_inputs.join(", ") || "none"}</strong></div>
              </div>
              {localRuntimeSmokeReadinessResult()!.readiness_reasons.length > 0 ? (
                <div class="warning-box">
                  <h4>Readiness reasons</h4>
                  <ul>
                    {localRuntimeSmokeReadinessResult()!.readiness_reasons.map((reason) => (
                      <li>{reason}</li>
                    ))}
                  </ul>
                </div>
              ) : (
                <p>No readiness reasons.</p>
              )}
              {localRuntimeSmokeReadinessResult()!.blockers.length > 0 ? (
                <div class="warning-box">
                  <h4>Blockers</h4>
                  <ul>
                    {localRuntimeSmokeReadinessResult()!.blockers.map((blocker) => (
                      <li>
                        <strong>{formatSnakeCaseLabel(blocker.kind)}</strong>
                        <div>{blocker.message}</div>
                      </li>
                    ))}
                  </ul>
                </div>
              ) : (
                <p>No smoke readiness blockers.</p>
              )}
              {localRuntimeSmokeReadinessResult()!.warnings.length > 0 ? (
                <div class="warning-box">
                  <h4>Warnings</h4>
                  <ul>
                    {localRuntimeSmokeReadinessResult()!.warnings.map((warning) => (
                      <li>
                        <strong>{formatSnakeCaseLabel(warning.kind)}</strong>
                        <div>{warning.message}</div>
                      </li>
                    ))}
                  </ul>
                </div>
              ) : (
                <p>No smoke readiness warnings.</p>
              )}
              <h4>Next required actions</h4>
              {localRuntimeSmokeReadinessResult()!.next_required_actions.length > 0 ? (
                <ul>
                  {localRuntimeSmokeReadinessResult()!.next_required_actions.map((item) => (
                    <li>{item}</li>
                  ))}
                </ul>
              ) : (
                <p>No next required actions.</p>
              )}
            </>
            ) : (
              <p class="muted">Run smoke readiness preview to see readiness reasons, blockers, warnings, and next actions.</p>
            )
          ) : (
          <p class="muted">Run smoke readiness preview to see readiness reasons, blockers, warnings, and next actions.</p>
        )}
      </div>
      <div class="artifact-overview">
        <h3>llama.cpp smoke execution plan</h3>
        <p class="muted">
          Execution plan only - this does not run smoke inference, does not spawn a process, does not load/read a model, does not call an LLM, and does not persist settings or artifacts.
        </p>
        <p class="muted">Uses the smoke readiness inputs above and only previews a future diagnostic smoke inference plan.</p>
        <div class="hero-actions">
          <button onClick={previewLocalRuntimeSmokeExecutionPlan} disabled={localRuntimeSmokeExecutionPlanLoading()}>
            {localRuntimeSmokeExecutionPlanLoading() ? "Previewing..." : "Preview llama.cpp smoke execution plan"}
          </button>
        </div>
        {localRuntimeSmokeExecutionPlanValidationError() && <p class="error">{localRuntimeSmokeExecutionPlanValidationError()}</p>}
        {localRuntimeSmokeExecutionPlanError() && <p class="error">{localRuntimeSmokeExecutionPlanError()}</p>}
        {localRuntimeSmokeExecutionPlanLoading() ? (
          <p>Previewing llama.cpp smoke execution plan...</p>
        ) : localRuntimeSmokeExecutionPlanHasRun() ? (
          localRuntimeSmokeExecutionPlanPreview() ? (
            <>
              {renderMetricGrid([
                { label: "Status", value: formatSnakeCaseLabel(localRuntimeSmokeExecutionPlanPreview()!.status) },
                { label: "Smoke readiness", value: formatSnakeCaseLabel(localRuntimeSmokeExecutionPlanPreview()!.smoke_readiness_status) },
                { label: "Capability status", value: formatSnakeCaseLabel(localRuntimeSmokeExecutionPlanPreview()!.capability_status) },
                { label: "Version probe status", value: formatSnakeCaseLabel(localRuntimeSmokeExecutionPlanPreview()!.version_probe_status) },
                { label: "Probe readiness", value: formatSnakeCaseLabel(localRuntimeSmokeExecutionPlanPreview()!.probe_readiness_status) },
                { label: "Validation status", value: formatSnakeCaseLabel(localRuntimeSmokeExecutionPlanPreview()!.validation_status) },
                { label: "Adapter contract status", value: formatSnakeCaseLabel(localRuntimeSmokeExecutionPlanPreview()!.adapter_contract_status) },
                { label: "Adapter kind", value: formatSnakeCaseLabel(localRuntimeSmokeExecutionPlanPreview()!.adapter_kind) },
                { label: "Probe consent", value: localRuntimeSmokeExecutionPlanPreview()!.probe_consent ? "yes" : "no" },
                { label: "Allow probe execution", value: localRuntimeSmokeExecutionPlanPreview()!.allow_probe_execution ? "yes" : "no" },
                { label: "Smoke consent", value: localRuntimeSmokeExecutionPlanPreview()!.smoke_consent ? "yes" : "no" },
                { label: "Diagnostic prompt chars", value: localRuntimeSmokeExecutionPlanPreview()!.diagnostic_prompt_char_count },
                { label: "Max output tokens", value: localRuntimeSmokeExecutionPlanPreview()!.max_output_tokens },
                { label: "Timeout ms", value: localRuntimeSmokeExecutionPlanPreview()!.timeout_ms },
              ])}
              <div class="contract-meta">
                <div><span>Executable file</span><strong>{localRuntimeSmokeExecutionPlanPreview()!.safe_executable_file_name ?? "not configured"}</strong></div>
                <div><span>Model file</span><strong>{localRuntimeSmokeExecutionPlanPreview()!.safe_model_file_name ?? "not configured"}</strong></div>
                <div><span>Normalized prompt</span><strong>{localRuntimeSmokeExecutionPlanPreview()!.normalized_diagnostic_prompt || "missing"}</strong></div>
                <div><span>Planned operation</span><strong>{localRuntimeSmokeExecutionPlanPreview()!.planned_operation}</strong></div>
                <div><span>Preview only</span><strong>{localRuntimeSmokeExecutionPlanPreview()!.preview_only ? "yes" : "no"}</strong></div>
                <div><span>No process spawn</span><strong>{localRuntimeSmokeExecutionPlanPreview()!.no_process_spawn ? "yes" : "no"}</strong></div>
                <div><span>No smoke inference execution</span><strong>{localRuntimeSmokeExecutionPlanPreview()!.no_smoke_inference_execution ? "yes" : "no"}</strong></div>
                <div><span>No model file read</span><strong>{localRuntimeSmokeExecutionPlanPreview()!.no_model_file_read ? "yes" : "no"}</strong></div>
                <div><span>No model load</span><strong>{localRuntimeSmokeExecutionPlanPreview()!.no_model_load ? "yes" : "no"}</strong></div>
                <div><span>No LLM call</span><strong>{localRuntimeSmokeExecutionPlanPreview()!.no_llm_call ? "yes" : "no"}</strong></div>
                <div><span>No persistence</span><strong>{localRuntimeSmokeExecutionPlanPreview()!.no_persistence ? "yes" : "no"}</strong></div>
                <div><span>No artifact write</span><strong>{localRuntimeSmokeExecutionPlanPreview()!.no_artifact_write ? "yes" : "no"}</strong></div>
                <div><span>No registry status change</span><strong>{localRuntimeSmokeExecutionPlanPreview()!.no_registry_status_change ? "yes" : "no"}</strong></div>
                <div><span>No audit write</span><strong>{localRuntimeSmokeExecutionPlanPreview()!.no_audit_write ? "yes" : "no"}</strong></div>
                <div><span>Diagnostic only</span><strong>{localRuntimeSmokeExecutionPlanPreview()!.diagnostic_only ? "yes" : "no"}</strong></div>
                <div><span>Not Scholar Chat answer</span><strong>{localRuntimeSmokeExecutionPlanPreview()!.not_scholar_chat_answer ? "yes" : "no"}</strong></div>
                <div><span>No answer generated</span><strong>{localRuntimeSmokeExecutionPlanPreview()!.no_answer_generated ? "yes" : "no"}</strong></div>
                <div><span>No grounding applied</span><strong>{localRuntimeSmokeExecutionPlanPreview()!.no_grounding_applied ? "yes" : "no"}</strong></div>
                <div><span>No Evidence Pack used</span><strong>{localRuntimeSmokeExecutionPlanPreview()!.no_evidence_pack_used ? "yes" : "no"}</strong></div>
              </div>
              {localRuntimeSmokeExecutionPlanPreview()!.summary && <p><strong>Summary:</strong> {localRuntimeSmokeExecutionPlanPreview()!.summary}</p>}
              <div class="contract-meta">
                <div><span>Required inputs</span><strong>{localRuntimeSmokeExecutionPlanPreview()!.required_inputs.join(", ") || "none"}</strong></div>
                <div><span>Missing inputs</span><strong>{localRuntimeSmokeExecutionPlanPreview()!.missing_inputs.join(", ") || "none"}</strong></div>
                <div><span>Planned inputs</span><strong>{localRuntimeSmokeExecutionPlanPreview()!.planned_inputs.join(", ") || "none"}</strong></div>
                <div><span>Planned safe arguments</span><strong>{localRuntimeSmokeExecutionPlanPreview()!.planned_safe_arguments.join(" ") || "none"}</strong></div>
                <div><span>Planned outputs</span><strong>{localRuntimeSmokeExecutionPlanPreview()!.planned_outputs.join(", ") || "none"}</strong></div>
              </div>
              {localRuntimeSmokeExecutionPlanPreview()!.plan_reasons.length > 0 ? (
                <div class="warning-box">
                  <h4>Plan reasons</h4>
                  <ul>
                    {localRuntimeSmokeExecutionPlanPreview()!.plan_reasons.map((reason) => (
                      <li>{reason}</li>
                    ))}
                  </ul>
                </div>
              ) : (
                <p>No smoke execution plan reasons.</p>
              )}
              {localRuntimeSmokeExecutionPlanPreview()!.blockers.length > 0 ? (
                <div class="warning-box">
                  <h4>Blockers</h4>
                  <ul>
                    {localRuntimeSmokeExecutionPlanPreview()!.blockers.map((blocker) => (
                      <li>
                        <strong>{formatSnakeCaseLabel(blocker.kind)}</strong>
                        <div>{blocker.message}</div>
                      </li>
                    ))}
                  </ul>
                </div>
              ) : (
                <p>No smoke execution plan blockers.</p>
              )}
              {localRuntimeSmokeExecutionPlanPreview()!.warnings.length > 0 ? (
                <div class="warning-box">
                  <h4>Warnings</h4>
                  <ul>
                    {localRuntimeSmokeExecutionPlanPreview()!.warnings.map((warning) => (
                      <li>
                        <strong>{formatSnakeCaseLabel(warning.kind)}</strong>
                        <div>{warning.message}</div>
                      </li>
                    ))}
                  </ul>
                </div>
              ) : (
                <p>No smoke execution plan warnings.</p>
              )}
              <h4>Next required actions</h4>
              {localRuntimeSmokeExecutionPlanPreview()!.next_required_actions.length > 0 ? (
                <ul>
                  {localRuntimeSmokeExecutionPlanPreview()!.next_required_actions.map((item) => (
                    <li>{item}</li>
                  ))}
                </ul>
              ) : (
                <p>No next required actions.</p>
              )}
            </>
            ) : (
              <p class="muted">Run the smoke execution plan preview to see the diagnostic-only plan and consent gate.</p>
            )
          ) : (
            <p class="muted">Run the smoke execution plan preview to see the diagnostic-only plan and consent gate.</p>
          )}
        </div>
      <div class="artifact-overview">
        <h3>Runtime invocation plan</h3>
        <p class="muted">
          Read-only preview of the future local runtime invocation. It uses the current Scholar Chat prompt and optional prompt-pack estimate, but it never starts a process or persists configuration.
        </p>
          <div class="form-row">
            <label>
              Max output tokens
              <input
                type="number"
                value={localRuntimeInvocationMaxOutputTokens()}
                onInput={(event) => {
                  setLocalRuntimeInvocationMaxOutputTokens(event.currentTarget.value);
                  clearLocalRuntimeInvocationPreview();
                  clearScholarChatDraftInferencePreview();
                }}
                placeholder="1024"
              />
            </label>
            <label>
              Stop sequences
              <input
                type="text"
                value={localRuntimeInvocationStopSequences()}
                onInput={(event) => {
                  setLocalRuntimeInvocationStopSequences(event.currentTarget.value);
                  clearLocalRuntimeInvocationPreview();
                }}
                placeholder="</s>, <|end|>"
              />
            </label>
          </div>
          <div class="hero-actions">
            <button onClick={previewLocalRuntimeInvocationPlan} disabled={localRuntimeInvocationLoading()}>
              {localRuntimeInvocationLoading() ? "Previewing..." : "Preview invocation plan"}
            </button>
          </div>
          <p class="muted">No process is executed. No tokens are generated. Configuration is not persisted.</p>
          {localRuntimeInvocationValidationError() && <p class="error">{localRuntimeInvocationValidationError()}</p>}
          {localRuntimeInvocationError() && <p class="error">{localRuntimeInvocationError()}</p>}
          {localRuntimeInvocationLoading() ? (
            <p>Previewing invocation plan...</p>
          ) : localRuntimeInvocationHasRun() ? (
            localRuntimeInvocationPreview() ? (
              <>
                {renderMetricGrid([
                  { label: "Status", value: formatSnakeCaseLabel(localRuntimeInvocationPreview()!.status) },
                  { label: "Runtime kind", value: formatSnakeCaseLabel(localRuntimeInvocationPreview()!.runtime_kind) },
                  { label: "Runtime health", value: formatSnakeCaseLabel(localRuntimeInvocationPreview()!.plan.runtime_health_status) },
                  { label: "Prompt chars", value: localRuntimeInvocationPreview()!.plan.prompt_char_count },
                  { label: "Estimated context chars", value: localRuntimeInvocationPreview()!.plan.estimated_context_char_count },
                  { label: "Max output tokens", value: localRuntimeInvocationPreview()!.plan.max_output_tokens ?? "missing" },
                ])}
                <div class="contract-meta">
                  <div><span>Model file</span><strong>{localRuntimeInvocationPreview()!.plan.safe_model_file_name ?? "not configured"}</strong></div>
                  <div><span>Executable file</span><strong>{localRuntimeInvocationPreview()!.plan.safe_executable_file_name ?? "not configured"}</strong></div>
                </div>
                <h4>Invocation steps</h4>
                {localRuntimeInvocationPreview()!.plan.invocation_steps.length > 0 ? (
                  <ul>
                    {localRuntimeInvocationPreview()!.plan.invocation_steps.map((step) => (
                      <li>{step}</li>
                    ))}
                  </ul>
                ) : (
                  <p>No invocation steps planned.</p>
                )}
                <h4>Safe argument preview</h4>
                {localRuntimeInvocationPreview()!.plan.safe_argument_preview.length > 0 ? (
                  <ul class="final-answer-list-items">
                    {localRuntimeInvocationPreview()!.plan.safe_argument_preview.map((item) => (
                      <li>
                        <div class="final-answer-list-item">
                          <span>{item}</span>
                        </div>
                      </li>
                    ))}
                  </ul>
                ) : (
                  <p>No safe argument preview available.</p>
                )}
                {localRuntimeInvocationPreview()!.plan.blockers.length > 0 ? (
                  <div class="warning-box">
                    <h4>Blockers</h4>
                    <ul>
                      {localRuntimeInvocationPreview()!.plan.blockers.map((blocker) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(blocker.kind)}</strong>
                          <div>{blocker.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No invocation blockers.</p>
                )}
                {localRuntimeInvocationPreview()!.plan.warnings.length > 0 ? (
                  <div class="warning-box">
                    <h4>Warnings</h4>
                    <ul>
                      {localRuntimeInvocationPreview()!.plan.warnings.map((warning) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(warning.kind)}</strong>
                          <div>{warning.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No invocation warnings.</p>
                )}
              </>
            ) : (
              <p>No runtime invocation plan preview loaded yet.</p>
            )
          ) : (
            <p>No runtime invocation plan preview loaded yet.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>llama.cpp smoke diagnostic</h3>
          <p class="muted">
            Diagnostic smoke only - this may run the configured llama.cpp executable with the configured model, but it does not generate a Scholar Chat answer, does not ground claims, does not use an Evidence Pack, and does not persist settings or artifacts.
          </p>
          <p class="muted">
            It uses the current smoke execution plan preview as its consent gate and only runs when that plan is ready later and execution consent is enabled.
          </p>
          <label>
            Diagnostic prompt
            <textarea
              rows={3}
              value={localRuntimeSmokePrompt()}
              onInput={(event) => {
                setLocalRuntimeSmokePrompt(event.currentTarget.value);
                clearLocalRuntimeSmokePreview();
                clearLocalRuntimeSmokeReadinessPreview();
              }}
              placeholder="Say READY in one short sentence."
            />
          </label>
          <div class="form-row">
            <label class="inline-field">
              I understand this will run the configured llama.cpp executable for a diagnostic smoke inference only.
              <input
                type="checkbox"
                checked={localRuntimeSmokeAllowExecution()}
                onChange={(event) => {
                  setLocalRuntimeSmokeAllowExecution(event.currentTarget.checked);
                  clearLocalRuntimeSmokePreview();
                }}
              />
            </label>
            <label>
              Timeout ms
              <input
                type="number"
                value={localRuntimeSmokeTimeoutMs()}
                onInput={(event) => {
                  setLocalRuntimeSmokeTimeoutMs(event.currentTarget.value);
                  clearLocalRuntimeSmokePreview();
                  clearLocalRuntimeSmokeReadinessPreview();
                }}
                placeholder="5000"
              />
            </label>
            <label>
              Max output tokens
              <input
                type="number"
                value={localRuntimeSmokeMaxOutputTokens()}
                onInput={(event) => {
                  setLocalRuntimeSmokeMaxOutputTokens(event.currentTarget.value);
                  clearLocalRuntimeSmokePreview();
                  clearLocalRuntimeSmokeReadinessPreview();
                }}
                placeholder="16"
              />
            </label>
          </div>
          <p class="muted">Smoke diagnostic actions remain consent-gated and do not create Scholar Chat answers.</p>
          <div class="hero-actions">
            <button onClick={previewLocalRuntimeSmokeDiagnostic} disabled={localRuntimeSmokeLoading()}>
              {localRuntimeSmokeLoading() ? "Running..." : "Run llama.cpp smoke diagnostic"}
            </button>
          </div>
          {localRuntimeSmokeValidationError() && <p class="error">{localRuntimeSmokeValidationError()}</p>}
          {localRuntimeSmokeError() && <p class="error">{localRuntimeSmokeError()}</p>}
          {localRuntimeSmokeLoading() ? (
            <p>Running llama.cpp smoke diagnostic...</p>
          ) : localRuntimeSmokeHasRun() ? (
            localRuntimeSmokeResult() ? (
              <>
                {renderMetricGrid([
                  { label: "Status", value: formatSnakeCaseLabel(localRuntimeSmokeResult()!.status) },
                  { label: "Smoke execution plan", value: formatSnakeCaseLabel(localRuntimeSmokeResult()!.smoke_execution_plan_status) },
                  { label: "Smoke readiness", value: formatSnakeCaseLabel(localRuntimeSmokeResult()!.smoke_readiness_status) },
                  { label: "Capability status", value: formatSnakeCaseLabel(localRuntimeSmokeResult()!.capability_status) },
                  { label: "Version probe status", value: formatSnakeCaseLabel(localRuntimeSmokeResult()!.version_probe_status) },
                  { label: "Probe readiness", value: formatSnakeCaseLabel(localRuntimeSmokeResult()!.probe_readiness_status) },
                  { label: "Validation status", value: formatSnakeCaseLabel(localRuntimeSmokeResult()!.validation_status) },
                  { label: "Adapter contract status", value: formatSnakeCaseLabel(localRuntimeSmokeResult()!.adapter_contract_status) },
                  { label: "Adapter kind", value: formatSnakeCaseLabel(localRuntimeSmokeResult()!.adapter_kind) },
                  { label: "Probe consent", value: localRuntimeSmokeResult()!.probe_consent ? "yes" : "no" },
                  { label: "Allow probe execution", value: localRuntimeSmokeResult()!.allow_probe_execution ? "yes" : "no" },
                  { label: "Smoke consent", value: localRuntimeSmokeResult()!.smoke_consent ? "yes" : "no" },
                  { label: "Allow smoke execution", value: localRuntimeSmokeResult()!.allow_smoke_execution ? "yes" : "no" },
                  { label: "Execution attempted", value: localRuntimeSmokeResult()!.execution_attempted ? "yes" : "no" },
                  { label: "Prompt chars", value: localRuntimeSmokeResult()!.diagnostic_prompt_char_count },
                  { label: "Max output tokens", value: localRuntimeSmokeResult()!.max_output_tokens },
                  { label: "Timeout ms", value: localRuntimeSmokeResult()!.timeout_ms },
                  { label: "Duration ms", value: localRuntimeSmokeResult()!.duration_ms },
                  { label: "Exit code", value: localRuntimeSmokeResult()!.exit_code ?? "missing" },
                  { label: "Stdout truncated", value: localRuntimeSmokeResult()!.stdout_truncated ? "yes" : "no" },
                  { label: "Stderr truncated", value: localRuntimeSmokeResult()!.stderr_truncated ? "yes" : "no" },
                  { label: "Diagnostic only", value: localRuntimeSmokeResult()!.diagnostic_only ? "yes" : "no" },
                  { label: "Not Scholar Chat answer", value: localRuntimeSmokeResult()!.not_scholar_chat_answer ? "yes" : "no" },
                  { label: "No answer generated", value: localRuntimeSmokeResult()!.no_answer_generated ? "yes" : "no" },
                  { label: "No grounding applied", value: localRuntimeSmokeResult()!.no_grounding_applied ? "yes" : "no" },
                  { label: "No Evidence Pack used", value: localRuntimeSmokeResult()!.no_evidence_pack_used ? "yes" : "no" },
                  { label: "No persistence", value: localRuntimeSmokeResult()!.no_persistence ? "yes" : "no" },
                  { label: "No artifact write", value: localRuntimeSmokeResult()!.no_artifact_write ? "yes" : "no" },
                  { label: "No registry status change", value: localRuntimeSmokeResult()!.no_registry_status_change ? "yes" : "no" },
                  { label: "No audit write", value: localRuntimeSmokeResult()!.no_audit_write ? "yes" : "no" },
                ])}
                <p class="muted">{localRuntimeSmokeResult()!.summary}</p>
                {localRuntimeSmokeResult()!.safe_model_file_name ? (
                  <p><strong>Model file name:</strong> {localRuntimeSmokeResult()!.safe_model_file_name}</p>
                ) : null}
                {localRuntimeSmokeResult()!.safe_executable_file_name ? (
                  <p><strong>Executable file name:</strong> {localRuntimeSmokeResult()!.safe_executable_file_name}</p>
                ) : null}
                <p><strong>Normalized prompt:</strong> {localRuntimeSmokeResult()!.normalized_diagnostic_prompt}</p>
                <h4>Stdout preview</h4>
                {localRuntimeSmokeResult()!.stdout_preview ? (
                  <pre>{localRuntimeSmokeResult()!.stdout_preview}</pre>
                ) : (
                  <p>No stdout captured.</p>
                )}
                <h4>Stderr preview</h4>
                {localRuntimeSmokeResult()!.stderr_preview ? (
                  <pre>{localRuntimeSmokeResult()!.stderr_preview}</pre>
                ) : (
                  <p>No stderr captured.</p>
                )}
                {localRuntimeSmokeResult()!.blockers.length > 0 ? (
                  <div class="warning-box">
                    <h4>Blockers</h4>
                    <ul>
                      {localRuntimeSmokeResult()!.blockers.map((blocker) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(blocker.kind)}</strong>
                          <div>{blocker.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No smoke diagnostic blockers.</p>
                )}
                {localRuntimeSmokeResult()!.warnings.length > 0 ? (
                  <div class="warning-box">
                    <h4>Warnings</h4>
                    <ul>
                      {localRuntimeSmokeResult()!.warnings.map((warning) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(warning.kind)}</strong>
                          <div>{warning.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No smoke diagnostic warnings.</p>
                )}
                <h4>Next required actions</h4>
                {localRuntimeSmokeResult()!.next_required_actions.length > 0 ? (
                  <ul>
                    {localRuntimeSmokeResult()!.next_required_actions.map((item) => (
                      <li>{item}</li>
                    ))}
                  </ul>
                ) : (
                  <p>No next required actions.</p>
                )}
              </>
            ) : (
              <p class="muted">Run the smoke diagnostic to see diagnostic-only status, blockers, warnings, and next actions.</p>
            )
          ) : (
            <p class="muted">Run the smoke diagnostic to see diagnostic-only status, blockers, warnings, and next actions.</p>
          )}
        </div>
        </details>
      <section class="card workspace-panel" id="developer-diagnostics" data-workspace="developer_diagnostics">
        <h2>Final answer inspector</h2>
        <p class="muted">
          Developer diagnostics and contract inspection stay available here. This read-only display covers an already-built FinalAnswer contract.
        </p>
        <div class="artifact-overview runtime-setup-card">
          <h3>Diagnostic-only local model smoke test</h3>
          <p class="muted">
            Consent-gated local smoke request against the managed localhost llama-server. This stays diagnostic-only, does not create a Scholar Chat answer, and is secondary to the chat workflow.
          </p>
          <p class="muted">{managedLlamaServerReadinessSummary()}</p>
          <div class="form-row">
            <label>
              Prompt
              <textarea
                value={managedLlamaServerSmokeDiagnosticPrompt()}
                onInput={(event) => {
                  setManagedLlamaServerSmokeDiagnosticPrompt(event.currentTarget.value);
                  clearManagedLlamaServerSmokeDiagnosticPreview();
                }}
                rows={3}
                placeholder="Say READY in one short sentence."
              />
            </label>
          </div>
          <div class="form-row">
            <label>
              Max output tokens
              <input
                type="number"
                value={managedLlamaServerSmokeDiagnosticMaxOutputTokens()}
                onInput={(event) => {
                  setManagedLlamaServerSmokeDiagnosticMaxOutputTokens(event.currentTarget.value);
                  clearManagedLlamaServerSmokeDiagnosticPreview();
                }}
                placeholder="16"
              />
            </label>
            <label>
              Timeout ms
              <input
                type="number"
                value={managedLlamaServerSmokeDiagnosticTimeoutMs()}
                onInput={(event) => {
                  setManagedLlamaServerSmokeDiagnosticTimeoutMs(event.currentTarget.value);
                  clearManagedLlamaServerSmokeDiagnosticPreview();
                }}
                placeholder="5000"
              />
            </label>
            <label class="inline-field">
              <input
                type="checkbox"
                checked={managedLlamaServerSmokeDiagnosticAllowRun()}
                onChange={(event) => {
                  setManagedLlamaServerSmokeDiagnosticAllowRun(event.currentTarget.checked);
                  clearManagedLlamaServerSmokeDiagnosticPreview();
                }}
              />
              I understand this is a diagnostic-only local smoke test against the managed localhost server.
            </label>
          </div>
          <div class="hero-actions">
            <button onClick={runManagedLlamaServerSmokeDiagnostic} disabled={managedLlamaServerSmokeDiagnosticLoading()}>
              {managedLlamaServerSmokeDiagnosticLoading() ? "Running..." : "Run smoke test"}
            </button>
          </div>
          <p class="muted">This remains diagnostic-only and does not create a Scholar Chat answer, Evidence Pack, or persisted artifact.</p>
          {managedLlamaServerSmokeDiagnosticError() && <p class="error">{managedLlamaServerSmokeDiagnosticError()}</p>}
          {managedLlamaServerSmokeDiagnosticLoading() ? (
            <p>Running managed smoke test...</p>
          ) : managedLlamaServerSmokeDiagnosticHasRun() ? (
            managedLlamaServerSmokeDiagnosticPreview() ? (
              <>
                {renderMetricGrid([
                  { label: "Status", value: formatSnakeCaseLabel(managedLlamaServerSmokeDiagnosticPreview()!.status) },
                  { label: "Lifecycle", value: formatSnakeCaseLabel(managedLlamaServerSmokeDiagnosticPreview()!.lifecycle_status) },
                  { label: "Health", value: formatSnakeCaseLabel(managedLlamaServerSmokeDiagnosticPreview()!.health_status) },
                  { label: "Ownership", value: managedLlamaServerSmokeDiagnosticPreview()!.owns_active_server ? "AEGIS-owned" : managedLlamaServerSmokeDiagnosticPreview()!.port_occupied_by_unmanaged_process ? "external / unmanaged" : "not active" },
                  { label: "Port occupancy", value: formatSnakeCaseLabel(managedLlamaServerSmokeDiagnosticPreview()!.port_occupancy_status) },
                  { label: "Port occupied", value: managedLlamaServerSmokeDiagnosticPreview()!.port_occupied ? "yes" : "no" },
                  { label: "Port unmanaged", value: managedLlamaServerSmokeDiagnosticPreview()!.port_occupied_by_unmanaged_process ? "yes" : "no" },
                  { label: "Host", value: managedLlamaServerSmokeDiagnosticPreview()!.host ?? "missing" },
                  { label: "Port", value: managedLlamaServerSmokeDiagnosticPreview()!.port ?? "missing" },
                  { label: "Alias", value: managedLlamaServerSmokeDiagnosticPreview()!.alias ?? "missing" },
                  { label: "Model", value: managedLlamaServerSmokeDiagnosticPreview()!.safe_model_file_name ?? "missing" },
                  { label: "Prompt chars", value: managedLlamaServerSmokeDiagnosticPreview()!.prompt_char_count },
                  { label: "Max output tokens", value: managedLlamaServerSmokeDiagnosticPreview()!.max_output_tokens },
                  { label: "Timeout ms", value: managedLlamaServerSmokeDiagnosticPreview()!.timeout_ms },
                  { label: "HTTP status", value: managedLlamaServerSmokeDiagnosticPreview()!.http_status ?? "missing" },
                  { label: "Duration ms", value: managedLlamaServerSmokeDiagnosticPreview()!.duration_ms },
                ])}
                <p><strong>Summary:</strong> {managedLlamaServerSmokeDiagnosticPreview()!.summary}</p>
                <div class="contract-meta">
                  <div><span>Diagnostic only</span><strong>{managedLlamaServerSmokeDiagnosticPreview()!.diagnostic_only ? "yes" : "no"}</strong></div>
                  <div><span>Not Scholar Chat answer</span><strong>{managedLlamaServerSmokeDiagnosticPreview()!.not_scholar_chat_answer ? "yes" : "no"}</strong></div>
                  <div><span>No grounding applied</span><strong>{managedLlamaServerSmokeDiagnosticPreview()!.no_grounding_applied ? "yes" : "no"}</strong></div>
                  <div><span>No Evidence Pack used</span><strong>{managedLlamaServerSmokeDiagnosticPreview()!.no_evidence_pack_used ? "yes" : "no"}</strong></div>
                  <div><span>No final answer</span><strong>{managedLlamaServerSmokeDiagnosticPreview()!.no_final_answer_created ? "yes" : "no"}</strong></div>
                  <div><span>No artifact write</span><strong>{managedLlamaServerSmokeDiagnosticPreview()!.no_artifact_write ? "yes" : "no"}</strong></div>
                  <div><span>No audit write</span><strong>{managedLlamaServerSmokeDiagnosticPreview()!.no_audit_write ? "yes" : "no"}</strong></div>
                  <div><span>No persistence</span><strong>{managedLlamaServerSmokeDiagnosticPreview()!.no_persistence ? "yes" : "no"}</strong></div>
                  <div><span>Execution attempted</span><strong>{managedLlamaServerSmokeDiagnosticPreview()!.execution_attempted ? "yes" : "no"}</strong></div>
                </div>
                {managedLlamaServerSmokeDiagnosticPreview()!.extracted_output_preview ? (
                  <p><strong>Output preview:</strong> {managedLlamaServerSmokeDiagnosticPreview()!.extracted_output_preview}</p>
                ) : (
                  <p>No extracted output preview.</p>
                )}
                {managedLlamaServerSmokeDiagnosticPreview()!.response_preview ? (
                  <details class="warning-box">
                    <summary>Raw response preview</summary>
                    <pre>{managedLlamaServerSmokeDiagnosticPreview()!.response_preview}</pre>
                    {managedLlamaServerSmokeDiagnosticPreview()!.response_preview_truncated ? <p class="muted">Preview truncated.</p> : null}
                  </details>
                ) : (
                  <p>No raw response preview captured.</p>
                )}
                {managedLlamaServerSmokeDiagnosticPreview()!.error_preview ? (
                  <details class="warning-box">
                    <summary>Error preview</summary>
                    <pre>{managedLlamaServerSmokeDiagnosticPreview()!.error_preview}</pre>
                    {managedLlamaServerSmokeDiagnosticPreview()!.error_preview_truncated ? <p class="muted">Preview truncated.</p> : null}
                  </details>
                ) : null}
                {managedLlamaServerSmokeDiagnosticPreview()!.blockers.length > 0 ? (
                  <div class="warning-box">
                    <h4>Blockers</h4>
                    <ul>
                      {managedLlamaServerSmokeDiagnosticPreview()!.blockers.map((blocker) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(blocker.kind)}</strong>
                          <div>{blocker.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No smoke test blockers.</p>
                )}
                {managedLlamaServerSmokeDiagnosticPreview()!.warnings.length > 0 ? (
                  <div class="warning-box">
                    <h4>Warnings</h4>
                    <ul>
                      {managedLlamaServerSmokeDiagnosticPreview()!.warnings.map((warning) => (
                        <li>
                          <strong>{formatSnakeCaseLabel(warning.kind)}</strong>
                          <div>{warning.message}</div>
                        </li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No smoke test warnings.</p>
                )}
                {managedLlamaServerSmokeDiagnosticPreview()!.next_required_actions.length > 0 ? (
                  <div class="warning-box">
                    <h4>Next required actions</h4>
                    <ul>
                      {managedLlamaServerSmokeDiagnosticPreview()!.next_required_actions.map((action) => (
                        <li>{action}</li>
                      ))}
                    </ul>
                  </div>
                ) : (
                  <p>No next required actions.</p>
                )}
              </>
            ) : (
              <p>No managed smoke test preview loaded yet.</p>
            )
          ) : (
            <p>No managed smoke test preview loaded yet.</p>
          )}
        </div>
        <div class="form-row">
          <label>
            Source ID
              <input
                type="text"
                value={sourceId()}
                onInput={(event) => {
                  setSourceId(event.currentTarget.value);
                  clearScholarChatDraftInferencePreview();
                }}
                placeholder="src_..."
              />
          </label>
          <label>
            Final answer ID
            <input
              type="text"
              value={finalAnswerId()}
              onInput={(event) => setFinalAnswerId(event.currentTarget.value)}
              placeholder="fan_..."
            />
          </label>
        </div>
        <div class="hero-actions">
          <button onClick={loadFinalAnswer} disabled={finalAnswerLoading()}>
            {finalAnswerLoading() ? "Loading..." : "Load final answer"}
          </button>
          <button onClick={loadArtifactOverview} disabled={artifactOverviewLoading() || !selectedAnswerArtifactSourceId()}>
            {artifactOverviewLoading() ? "Loading..." : "Load final answers"}
          </button>
          <button onClick={() => loadArtifactSources()} disabled={artifactSourcesLoading()}>
            {artifactSourcesLoading() ? "Loading..." : "Load source index"}
          </button>
          <button onClick={() => loadArtifactHealth()} disabled={artifactHealthLoading()}>
            {artifactHealthLoading() ? "Loading..." : "Load answer artifacts"}
          </button>
          <button onClick={() => loadArtifactIssues()} disabled={artifactIssuesLoading()}>
            {artifactIssuesLoading() ? "Loading..." : "Load artifact issues"}
          </button>
          <button onClick={() => refreshDiagnostics()} disabled={diagnosticsAreLoading()}>
            {diagnosticsAreLoading() ? "Refreshing..." : "Refresh diagnostics"}
          </button>
          <button onClick={loadArtifactManifest} disabled={artifactManifestLoading()}>
            {artifactManifestLoading() ? "Loading..." : "Load export manifest"}
          </button>
          <label class="inline-field">
            Export destination
            <input
              type="text"
              value={exportRoot()}
              onInput={(event) => setExportRoot(event.currentTarget.value)}
              placeholder="E:\\path\\to\\export"
            />
          </label>
          <button onClick={exportArtifacts} disabled={artifactExportLoading()}>
            {artifactExportLoading() ? "Loading..." : "Export artifacts"}
          </button>
        </div>
        {finalAnswerError() && <p class="error">{finalAnswerError()}</p>}
        {artifactOverviewError() && <p class="error">{artifactOverviewError()}</p>}
        {retrievalIndexError() && <p class="error">{retrievalIndexError()}</p>}
        {artifactSourcesError() && <p class="error">{artifactSourcesError()}</p>}
        {artifactHealthError() && <p class="error">{artifactHealthError()}</p>}
        {artifactIssuesError() && <p class="error">{artifactIssuesError()}</p>}
        {artifactManifestError() && <p class="error">{artifactManifestError()}</p>}
        {artifactExportError() && <p class="error">{artifactExportError()}</p>}
        {artifactBundleInspectionError() && <p class="error">{artifactBundleInspectionError()}</p>}
        <div class="artifact-overview">
          <h3>Answer artifact sources</h3>
          <p class="muted">Read-only source index for existing answer artifacts.</p>
          {artifactSources().length > 0 ? (
            <div class="contract-meta">
              <div><span>Sources</span><strong>{answerArtifactSourceTotals().source_count}</strong></div>
              <div><span>Drafts</span><strong>{answerArtifactSourceTotals().draft_count}</strong></div>
              <div><span>Grounded answers</span><strong>{answerArtifactSourceTotals().grounded_answer_count}</strong></div>
              <div><span>Final answers</span><strong>{answerArtifactSourceTotals().final_answer_count}</strong></div>
            </div>
          ) : null}
          {artifactSources().length > 0 ? (
            <ul class="final-answer-list-items">
              {artifactSources().map((item) => (
                <li>
                  <button class="final-answer-list-item" onClick={() => selectArtifactSource(item)}>
                    <span>{item.source_id}</span>
                    <small>
                      drafts={item.draft_count} | grounded={item.grounded_answer_count} | final={item.final_answer_count}
                    </small>
                  </button>
                </li>
              ))}
            </ul>
          ) : (
            <p>No answer artifact sources loaded yet.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>Answer artifacts</h3>
          <p class="muted">Read-only health and overview counts for existing answer artifacts.</p>
          {artifactHealth() ? (
            <>
              <div class="contract-meta">
                <div><span>Sources</span><strong>{artifactHealth()!.source_count}</strong></div>
                <div><span>Drafts</span><strong>{artifactHealth()!.draft_count}</strong></div>
                <div><span>Grounded answers</span><strong>{artifactHealth()!.grounded_answer_count}</strong></div>
                <div><span>Final answers</span><strong>{artifactHealth()!.final_answer_count}</strong></div>
                <div><span>Malformed finals</span><strong>{artifactHealth()!.malformed_final_answer_count}</strong></div>
                <div><span>Unsupported statements</span><strong>{artifactHealth()!.unsupported_statement_count}</strong></div>
                <div><span>Needs evidence</span><strong>{artifactHealth()!.needs_evidence_statement_count}</strong></div>
              </div>
              {artifactHealth()!.sources.length > 0 ? (
                <ul class="final-answer-list-items">
                  {artifactHealth()!.sources.map((item) => (
                    <li>
                      <button class="final-answer-list-item" onClick={() => selectArtifactSource(item)}>
                        <span>{item.source_id}</span>
                        <small>
                          drafts={item.draft_count} | grounded={item.grounded_answer_count} | final={item.final_answer_count} | malformed={item.malformed_final_answer_count} | needs_evidence={item.needs_evidence_statement_count} | unsupported={item.unsupported_statement_count}
                        </small>
                      </button>
                    </li>
                  ))}
                </ul>
              ) : (
                <p>No answer artifact health entries yet.</p>
              )}
            </>
          ) : (
            <p>No answer artifacts loaded yet.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>Answer artifact issues</h3>
          <p class="muted">Read-only issue list for existing answer artifacts.</p>
          {artifactIssuesLoading() ? (
            <p>Loading answer artifact issues...</p>
          ) : artifactIssuesHasRun() ? (
            <>
              <div class="contract-meta">
                <div><span>Issues</span><strong>{answerArtifactIssueTotals().issue_count}</strong></div>
                <div><span>Sources</span><strong>{answerArtifactIssueTotals().source_count}</strong></div>
                <div><span>Malformed finals</span><strong>{answerArtifactIssueTotals().malformed_final_answer_count}</strong></div>
                <div><span>Needs evidence</span><strong>{answerArtifactIssueTotals().needs_evidence_statement_count}</strong></div>
                <div><span>Unsupported</span><strong>{answerArtifactIssueTotals().unsupported_statement_count}</strong></div>
              </div>
              {artifactIssues().length > 0 ? (
                <ul class="final-answer-list-items">
                  {artifactIssues().map((item) => (
                    <li>
                      <div class="final-answer-list-item">
                        <span>
                          <button class="link-button" onClick={() => selectArtifactSourceId(item.source_id)}>
                            {item.source_id}
                          </button>
                        </span>
                        <small>
                          {item.issue_kind}
                          {item.final_answer_id ? ` | ${item.final_answer_id}` : ""}
                          {item.statement_index !== null && item.statement_index !== undefined ? ` | statement=${item.statement_index}` : ""}
                          {item.statement_status ? ` | status=${item.statement_status}` : ""}
                          {item.grounded_answer_id ? ` | grounded=${item.grounded_answer_id}` : ""}
                        </small>
                        <p>{item.message}</p>
                      </div>
                    </li>
                  ))}
                </ul>
              ) : (
                <p>No answer artifact issues reported.</p>
              )}
            </>
          ) : (
            <p>No answer artifact issues loaded yet.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>Export manifest</h3>
          {artifactManifest() ? (
            <>
              <div class="contract-meta">
                <div><span>Schema</span><strong>{artifactManifest()!.schema_version || "missing"}</strong></div>
                <div><span>Sources</span><strong>{artifactManifest()!.source_count}</strong></div>
                <div><span>Drafts</span><strong>{artifactManifest()!.draft_count}</strong></div>
                <div><span>Grounded answers</span><strong>{artifactManifest()!.grounded_answer_count}</strong></div>
                <div><span>Final answers</span><strong>{artifactManifest()!.final_answer_count}</strong></div>
                <div><span>Issues</span><strong>{artifactManifest()!.issue_count}</strong></div>
              </div>
              {artifactManifest()!.sources.length > 0 ? (
                <ul class="final-answer-list-items">
                  {artifactManifest()!.sources.map((item) => (
                    <li>
                      <button class="final-answer-list-item" onClick={() => selectArtifactSourceId(item.source_id)}>
                        <span>{item.source_id}</span>
                        <small>
                          drafts={item.draft_count} | grounded={item.grounded_answer_count} | final={item.final_answer_count} | issues={item.issue_count}
                        </small>
                        {item.final_answers.length > 0 && (
                          <small>
                            {item.final_answers.map((answer) => answer.final_answer_id).join(", ")}
                          </small>
                        )}
                      </button>
                    </li>
                  ))}
                </ul>
              ) : (
                <p>No export manifest entries yet.</p>
              )}
            </>
          ) : (
            <p>No export manifest loaded yet.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>Export result</h3>
          {artifactExportResult() ? (
            <>
              <div class="contract-meta">
                <div><span>Schema</span><strong>{artifactExportResult()!.schema_version || "missing"}</strong></div>
                <div><span>Export ID</span><strong>{artifactExportResult()!.export_id}</strong></div>
                <div><span>Sources</span><strong>{artifactExportResult()!.exported_source_count}</strong></div>
                <div><span>Drafts</span><strong>{artifactExportResult()!.exported_draft_count}</strong></div>
                <div><span>Grounded answers</span><strong>{artifactExportResult()!.exported_grounded_answer_count}</strong></div>
                <div><span>Final answers</span><strong>{artifactExportResult()!.exported_final_answer_count}</strong></div>
                <div><span>Issues</span><strong>{artifactExportResult()!.exported_issue_count}</strong></div>
                <div><span>Integrity</span><strong>{artifactExportResult()!.integrity.schema_version ? `${artifactExportResult()!.integrity.algorithm} | ${artifactExportResult()!.integrity.files.length} files` : "missing"}</strong></div>
              </div>
              {artifactExportResult()!.written_files.length > 0 ? (
                <ul class="final-answer-list-items">
                  {artifactExportResult()!.written_files.map((item) => (
                    <li>
                      <div class="final-answer-list-item">
                        <span>{item.relative_path}</span>
                        <small>
                          {item.artifact_kind}
                          {item.source_id ? ` | ${item.source_id}` : ""}
                          {item.artifact_id ? ` | ${item.artifact_id}` : ""}
                        </small>
                      </div>
                    </li>
                  ))}
                </ul>
              ) : (
                <p>No exported files listed yet.</p>
              )}
            </>
          ) : (
            <p>No export result loaded yet.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>Export bundle inspector</h3>
          <p class="muted">
            Read-only validation for an existing manual export bundle.
          </p>
          <div class="form-row">
            <label class="inline-field">
              Export bundle root
              <input
                type="text"
                value={exportBundleRoot()}
                onInput={(event) => setExportBundleRoot(event.currentTarget.value)}
                placeholder="E:\\path\\to\\export-bundle"
              />
            </label>
          </div>
          <div class="hero-actions">
            <button onClick={inspectExportBundle} disabled={artifactBundleInspectionLoading()}>
              {artifactBundleInspectionLoading() ? "Loading..." : "Inspect export bundle"}
            </button>
          </div>
          {artifactBundleInspection() ? (
            <>
              <div class="artifact-overview">
                <h4>Inspection status</h4>
                {renderMetricGrid([
                  { label: "Status code", value: artifactBundleInspection()!.inspection_status.code },
                  { label: "Status label", value: artifactBundleInspection()!.inspection_status.label },
                  { label: "Severity", value: artifactBundleInspection()!.inspection_status.severity },
                ])}
                <p class="muted">{artifactBundleInspection()!.inspection_status.reason}</p>
              </div>
              <div class="artifact-overview">
                <h4>Inspection summary</h4>
                {renderMetricGrid([
                  { label: "Consistent", value: artifactBundleInspection()!.inspection_summary.is_consistent ? "yes" : "no" },
                  { label: "Schema supported", value: artifactBundleInspection()!.inspection_summary.schema_supported ? "yes" : "no" },
                  { label: "Integrity verified", value: artifactBundleInspection()!.inspection_summary.integrity_verified ? "yes" : "no" },
                  { label: "Issues", value: artifactBundleInspection()!.inspection_summary.issue_count },
                  { label: "Warnings", value: artifactBundleInspection()!.inspection_summary.warning_count },
                  { label: "Checked files", value: artifactBundleInspection()!.inspection_summary.checked_file_count },
                  { label: "Integrity files", value: artifactBundleInspection()!.inspection_summary.integrity_file_count },
                ])}
                {artifactBundleInspection()!.inspection_summary.issue_counts_by_kind.length > 0 ? (
                  <ul class="final-answer-list-items">
                    {artifactBundleInspection()!.inspection_summary.issue_counts_by_kind.map((item) => (
                      <li>
                        <div class="final-answer-list-item">
                          <span>{item.kind}</span>
                          <small>count={item.count}</small>
                        </div>
                      </li>
                    ))}
                  </ul>
                ) : null}
              </div>
              <div class="artifact-overview">
                <h4>File statuses</h4>
                {artifactBundleInspection()!.file_statuses.length > 0 ? (
                  <ul class="final-answer-list-items">
                    {artifactBundleInspection()!.file_statuses.map((fileStatus) => (
                      <li>
                        <div class="final-answer-list-item">
                          <span>{fileStatus.file_label}</span>
                          <small>
                            status={fileStatus.status} | present={fileStatus.present ? "yes" : "no"} | parsed={fileStatus.parsed ? "yes" : "no"} | malformed={fileStatus.malformed ? "yes" : "no"} | schema={fileStatus.schema_status} | integrity={fileStatus.integrity_status} | issues={fileStatus.issue_count}
                          </small>
                          {fileStatus.schema_version ? <small>schema_version={fileStatus.schema_version}</small> : null}
                        </div>
                      </li>
                    ))}
                  </ul>
                ) : (
                  <p>No file statuses available.</p>
                )}
              </div>
              <div class="artifact-overview">
                <h4>Issue detail groups</h4>
                {artifactBundleInspection()!.issue_groups.length > 0 ? (
                  artifactBundleInspection()!.issue_groups.map((group) => (
                    <div class="artifact-overview">
                      <h5>{group.kind}</h5>
                      {renderMetricGrid([{ label: "Count", value: group.count }])}
                      {group.lines.length > 0 ? (
                        <ul class="final-answer-list-items">
                          {group.lines.map((line) => (
                            <li>
                              <div class="final-answer-list-item">
                                <span>{line}</span>
                              </div>
                            </li>
                          ))}
                        </ul>
                      ) : (
                        <p>No lines available.</p>
                      )}
                    </div>
                  ))
                ) : (
                  <p>No issue groups available.</p>
                )}
              </div>
              <div class="artifact-overview">
                <h4>{artifactBundleInspection()!.report_preview.title}</h4>
                {renderMetricGrid([
                  { label: "Schema", value: artifactBundleInspection()!.report_preview.schema_version },
                  { label: "Consistent", value: artifactBundleInspection()!.report_preview.is_consistent ? "yes" : "no" },
                  { label: "Integrity verified", value: artifactBundleInspection()!.report_preview.integrity_verified ? "yes" : "no" },
                  { label: "Issues", value: artifactBundleInspection()!.report_preview.issue_count },
                  { label: "Warnings", value: artifactBundleInspection()!.report_preview.warning_count },
                ])}
                {artifactBundleInspection()!.report_preview.sections.length > 0 ? (
                  artifactBundleInspection()!.report_preview.sections.map((section) => (
                    <div class="artifact-overview">
                      <h5>{section.heading}</h5>
                      {section.lines.length > 0 ? (
                        <ul class="final-answer-list-items">
                          {section.lines.map((line) => (
                            <li>
                              <div class="final-answer-list-item">
                                <span>{line}</span>
                              </div>
                            </li>
                          ))}
                        </ul>
                      ) : (
                        <p>No lines available.</p>
                      )}
                    </div>
                  ))
                ) : (
                  <p>No preview sections available.</p>
                )}
              </div>
              <div class="contract-meta">
                <div><span>Schema</span><strong>{artifactBundleInspection()!.schema_version || "mixed / missing"}</strong></div>
                <div><span>Has manifest</span><strong>{artifactBundleInspection()!.has_manifest ? "yes" : "no"}</strong></div>
                <div><span>Has issues</span><strong>{artifactBundleInspection()!.has_issues ? "yes" : "no"}</strong></div>
                <div><span>Has summary</span><strong>{artifactBundleInspection()!.has_summary ? "yes" : "no"}</strong></div>
                <div><span>Has integrity</span><strong>{artifactBundleInspection()!.has_integrity ? "yes" : "no"}</strong></div>
                <div><span>Consistent</span><strong>{artifactBundleInspection()!.is_consistent ? "yes" : "no"}</strong></div>
                <div><span>Issues</span><strong>{artifactBundleInspection()!.issue_count}</strong></div>
                <div><span>Warnings</span><strong>{artifactBundleInspection()!.warning_count}</strong></div>
              </div>
              {artifactBundleInspection()!.manifest_counts ? (
                <div class="artifact-overview">
                  <h4>Manifest counts</h4>
                  {renderMetricGrid([
                    { label: "Schema", value: artifactBundleInspection()!.manifest_counts!.schema_version || "missing" },
                    { label: "Sources", value: artifactBundleInspection()!.manifest_counts!.source_count },
                    { label: "Drafts", value: artifactBundleInspection()!.manifest_counts!.draft_count },
                    { label: "Grounded answers", value: artifactBundleInspection()!.manifest_counts!.grounded_answer_count },
                    { label: "Final answers", value: artifactBundleInspection()!.manifest_counts!.final_answer_count },
                    { label: "Issues", value: artifactBundleInspection()!.manifest_counts!.issue_count },
                  ])}
                  {artifactBundleInspection()!.manifest_counts!.sources.length > 0 ? (
                    <ul class="final-answer-list-items">
                      {artifactBundleInspection()!.manifest_counts!.sources.map((item) => (
                        <li>
                          <div class="final-answer-list-item">
                            <span>{item.source_id}</span>
                            <small>
                              drafts={item.draft_count} | grounded={item.grounded_answer_count} | final={item.final_answer_count} | issues={item.issue_count}
                            </small>
                            {item.final_answers.length > 0 && (
                              <small>
                                {item.final_answers.map((answer) => answer.final_answer_id).join(", ")}
                              </small>
                            )}
                          </div>
                        </li>
                      ))}
                    </ul>
                  ) : (
                    <p>No manifest sources listed yet.</p>
                  )}
                </div>
              ) : null}
              {artifactBundleInspection()!.summary_counts ? (
                <div class="artifact-overview">
                  <h4>Summary counts</h4>
                  {renderMetricGrid([
                    { label: "Schema", value: artifactBundleInspection()!.summary_counts!.schema_version || "missing" },
                    { label: "Export ID", value: artifactBundleInspection()!.summary_counts!.export_id },
                    { label: "Generated from", value: artifactBundleInspection()!.summary_counts!.generated_from },
                    { label: "Scope", value: artifactBundleInspection()!.summary_counts!.export_scope },
                    { label: "Sources", value: artifactBundleInspection()!.summary_counts!.source_count },
                    { label: "Drafts", value: artifactBundleInspection()!.summary_counts!.draft_count },
                    { label: "Grounded answers", value: artifactBundleInspection()!.summary_counts!.grounded_answer_count },
                    { label: "Final answers", value: artifactBundleInspection()!.summary_counts!.final_answer_count },
                    { label: "Issues", value: artifactBundleInspection()!.summary_counts!.issue_count },
                  ])}
                  {artifactBundleInspection()!.summary_counts!.issue_kinds.length > 0 && (
                    <ul class="final-answer-list-items">
                      {artifactBundleInspection()!.summary_counts!.issue_kinds.map((item) => (
                        <li>
                          <div class="final-answer-list-item">
                            <span>{item.issue_kind}</span>
                            <small>count={item.count}</small>
                          </div>
                        </li>
                      ))}
                    </ul>
                  )}
                </div>
              ) : null}
              {artifactBundleInspection()!.integrity_counts ? (
                <div class="artifact-overview">
                  <h4>Integrity metadata</h4>
                  {renderMetricGrid([
                    { label: "Schema", value: artifactBundleInspection()!.integrity_counts!.schema_version || "missing" },
                    { label: "Algorithm", value: artifactBundleInspection()!.integrity_counts!.algorithm },
                    { label: "Files", value: artifactBundleInspection()!.integrity_counts!.files.length },
                  ])}
                </div>
              ) : null}
              {artifactBundleInspection()!.issue_kind_counts && artifactBundleInspection()!.issue_kind_counts!.length > 0 ? (
                <div class="artifact-overview">
                  <h4>Issue kind counts</h4>
                  <ul class="final-answer-list-items">
                    {artifactBundleInspection()!.issue_kind_counts!.map((item) => (
                      <li>
                        <div class="final-answer-list-item">
                          <span>{item.issue_kind}</span>
                          <small>count={item.count}</small>
                        </div>
                      </li>
                    ))}
                  </ul>
                </div>
              ) : null}
              {artifactBundleInspection()!.errors.length > 0 ? (
                <div class="warning-box">
                  <h4>Inspection errors</h4>
                  <ul>
                    {artifactBundleInspection()!.errors.map((item) => (
                      <li>
                        <strong>{item.kind}</strong>
                        <div>{item.message}</div>
                        {item.relative_path && <small>{item.relative_path}</small>}
                      </li>
                    ))}
                  </ul>
                </div>
              ) : null}
              {artifactBundleInspection()!.warnings.length > 0 ? (
                <div class="warning-box">
                  <h4>Inspection warnings</h4>
                  <ul>
                    {artifactBundleInspection()!.warnings.map((item) => (
                      <li>
                        <strong>{item.kind}</strong>
                        <div>{item.message}</div>
                        {item.relative_path && <small>{item.relative_path}</small>}
                      </li>
                    ))}
                  </ul>
                </div>
              ) : null}
            </>
          ) : (
            <p>No export bundle inspection loaded yet.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>Final answers</h3>
          <p class="muted">Read-only final-answer metadata for the selected answer artifact source.</p>
          {selectedAnswerArtifactSourceId() ? (
            artifactOverview() ? (
              <>
                <div class="contract-meta">
                  <div><span>Source ID</span><strong>{artifactOverview()!.source_id}</strong></div>
                  <div><span>Answer drafts</span><strong>{artifactOverview()!.draft_count}</strong></div>
                  <div><span>Grounded answers</span><strong>{artifactOverview()!.grounded_answer_count}</strong></div>
                  <div><span>Final answers</span><strong>{artifactOverview()!.final_answer_count}</strong></div>
                </div>
                {artifactOverview()!.final_answers.length > 0 ? (
                  <ul class="final-answer-list-items">
                    {artifactOverview()!.final_answers.map((item) => (
                      <li>
                        <button class="final-answer-list-item" onClick={() => selectFinalAnswer(item)}>
                          <span>{item.final_answer_id}</span>
                          <small>
                            {item.grounded_answer_id} | statements={item.statement_count} | needs_evidence={item.needs_evidence_count} | unsupported={item.unsupported_count}
                          </small>
                        </button>
                      </li>
                    ))}
                  </ul>
                ) : (
                  <p>No final answers listed yet for this source.</p>
                )}
                <div class="artifact-overview">
                  <h4>Selected final answer preview</h4>
                  {finalAnswerLoading() ? (
                    <p>Loading selected final answer...</p>
                  ) : finalAnswerId().trim() ? (
                    selectedFinalAnswerDetail() ? (
                      <>
                        <div class="contract-meta">
                          <div><span>Final answer ID</span><strong>{selectedFinalAnswerDetail()!.final_answer_id}</strong></div>
                          <div><span>Grounded answer ID</span><strong>{selectedFinalAnswerDetail()!.grounded_answer_id}</strong></div>
                          <div><span>Mode</span><strong>{selectedFinalAnswerDetail()!.answer_mode}</strong></div>
                          <div><span>Statements</span><strong>{selectedFinalAnswerDetail()!.statement_count}</strong></div>
                          <div><span>Needs evidence</span><strong>{selectedFinalAnswerDetail()!.statements.filter((statement) => statement.status === "needs_evidence").length}</strong></div>
                          <div><span>Unsupported</span><strong>{selectedFinalAnswerDetail()!.unsupported_count}</strong></div>
                        </div>
                        {selectedFinalAnswerDetail()!.statements.length > 0 ? (
                          <ul class="final-answer-list-items">
                            {selectedFinalAnswerDetail()!.statements.map((statement, index) => (
                              <li>
                                <div class="final-answer-list-item">
                                  <span>
                                    Statement {index + 1}
                                  </span>
                                  <small>
                                    <span class={`status-pill status-${statement.status}`}>{statement.status}</span> | {statement.support_level}
                                  </small>
                                  <p>{statement.text}</p>
                                  <small>
                                    evidence={statement.evidence_ids.join(", ") || "none"} | chunks={statement.chunk_ids.join(", ") || "none"} | locators={statement.locators.length}
                                  </small>
                                </div>
                              </li>
                            ))}
                          </ul>
                        ) : (
                          <p>No statements in this final answer.</p>
                        )}
                        {selectedFinalAnswerDetail()!.warnings.length > 0 && (
                          <div class="warning-box">
                            <h4>Warnings</h4>
                            <ul>
                              {selectedFinalAnswerDetail()!.warnings.map((warning) => (
                                <li>{warning}</li>
                              ))}
                            </ul>
                          </div>
                        )}
                      </>
                    ) : finalAnswerError() ? (
                      <p class="error">Selected final answer unavailable: {finalAnswerError()}</p>
                    ) : (
                      <p>Selected final answer unavailable.</p>
                    )
                  ) : (
                    <p>Select a final answer to preview details.</p>
                  )}
                </div>
              </>
            ) : (
              <p>Loading final answers for the selected source...</p>
            )
          ) : (
            <p>Select an answer artifact source to load final answers.</p>
          )}
        </div>
        <EvidencePacksWorkspace
          selectedEvidencePackSourceId={selectedEvidencePackSourceId()}
          loadEvidencePacks={loadEvidencePacks}
          evidencePacksLoading={evidencePacksLoading()}
          evidencePacksError={evidencePacksError()}
          evidencePacksSourceId={evidencePacksSourceId()}
          evidencePacks={evidencePacks()}
        />
        <div class="artifact-overview">
          <h3>Retrieval index</h3>
          <p class="muted">Read-only retrieval metadata for the source ID above.</p>
          <div class="hero-actions">
            <button onClick={() => loadRetrievalIndex()} disabled={retrievalIndexLoading()}>
              {retrievalIndexLoading() ? "Loading..." : "Load retrieval index"}
            </button>
          </div>
          {retrievalIndex() ? (
            <>
              <div class="contract-meta">
                <div><span>Source ID</span><strong>{retrievalIndex()!.source_id}</strong></div>
                <div><span>Version ID</span><strong>{retrievalIndex()!.version_id}</strong></div>
                <div><span>Indexed at</span><strong>{retrievalIndex()!.indexed_at}</strong></div>
                <div><span>Chunk count</span><strong>{retrievalIndex()!.chunk_count}</strong></div>
                <div><span>Index version</span><strong>{retrievalIndex()!.index_version}</strong></div>
                <div><span>Chunk report hash</span><strong>{retrievalIndex()!.chunk_report_hash}</strong></div>
                <div><span>Entries</span><strong>{retrievalIndex()!.entries.length}</strong></div>
                <div><span>Warnings</span><strong>{retrievalIndex()!.warnings.length}</strong></div>
              </div>
              {retrievalIndex()!.warnings.length > 0 ? (
                <ul class="final-answer-list-items">
                  {retrievalIndex()!.warnings.map((warning) => (
                    <li>
                      <div class="final-answer-list-item">
                        <span>{warning}</span>
                      </div>
                    </li>
                  ))}
                </ul>
              ) : null}
            </>
          ) : (
            <p>No retrieval index loaded yet.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>Retrieval search</h3>
          <p class="muted">Read-only lexical search using the selected source from the loaded retrieval index.</p>
          {retrievalIndex() ? (
            <div class="form-row">
              <label>
                Source ID
                <select
                  value={retrievalSearchSourceId()}
                  onChange={(event) => selectRetrievalSearchSourceId(event.currentTarget.value)}
                  disabled={retrievalSearchSourceIds().length === 0}
                >
                  {retrievalSearchSourceIds().length > 0 ? (
                    retrievalSearchSourceIds().map((item) => <option value={item}>{item}</option>)
                  ) : (
                    <option value="">No source IDs available</option>
                  )}
                </select>
              </label>
            </div>
          ) : (
            <p class="muted">Load a retrieval index to choose a source.</p>
          )}
          <div class="form-row">
            <label>
              Query
              <input
                type="text"
                value={retrievalSearchQuery()}
                onInput={(event) => {
                  setRetrievalSearchQuery(event.currentTarget.value);
                  setRetrievalSearchValidationError(null);
                }}
                placeholder="alpha beta"
              />
            </label>
          </div>
          {retrievalSearchValidationError() && <p class="error">{retrievalSearchValidationError()}</p>}
          <div class="hero-actions">
            <button
              onClick={runRetrievalSearch}
              disabled={retrievalSearchLoading() || retrievalSearchSourceIds().length === 0 || !retrievalSearchSourceId().trim()}
            >
              {retrievalSearchLoading() ? "Loading..." : "Run retrieval search"}
            </button>
          </div>
          {retrievalSearchError() && <p class="error">{retrievalSearchError()}</p>}
          {retrievalSearchSourceIds().length === 0 && retrievalIndex() ? (
            <p class="muted">No source IDs are available in the loaded retrieval index.</p>
          ) : null}
          {retrievalSearchHasRun() ? (
            retrievalSearch() ? (
              <>
                <div class="contract-meta">
                  <div><span>Query</span><strong>{retrievalSearch()!.query}</strong></div>
                  <div><span>Result count</span><strong>{retrievalSearch()!.result_count}</strong></div>
                  <div>
                    <span>Normalized terms</span>
                    <strong>{retrievalSearch()!.normalized_query_terms.length}</strong>
                  </div>
                </div>
                {retrievalSearch()!.results.length > 0 ? (
                  <>
                    {retrievalSearch()!.results.length > RETRIEVAL_SEARCH_DISPLAY_LIMIT ? (
                      <p class="muted">
                        Showing first {RETRIEVAL_SEARCH_DISPLAY_LIMIT} of {retrievalSearch()!.results.length} results returned.
                      </p>
                    ) : null}
                    <ul class="final-answer-list-items">
                      {retrievalSearch()!.results.slice(0, RETRIEVAL_SEARCH_DISPLAY_LIMIT).map((item) => (
                        <li>
                          <div class="final-answer-list-item">
                            <span>{item.chunk_id}</span>
                            <small>
                              source={item.source_id} | score={item.score.toFixed(3)} | matched={item.matched_terms.join(", ") || "none"}
                            </small>
                            <small>{locatorSummary(item.locator)}</small>
                            <p>{item.preview}</p>
                          </div>
                        </li>
                      ))}
                    </ul>
                  </>
                ) : (
                  <p>No retrieval results matched the query.</p>
                )}
              </>
            ) : null
          ) : (
            retrievalSearchSourceIds().length > 0 ? <p>No retrieval search run yet.</p> : null
          )}
        </div>
        {finalAnswer() ? (
          <div class="contract-view">
            <div class="contract-meta">
              <div><span>Final answer ID</span><strong>{finalAnswer()!.final_answer_id}</strong></div>
              <div><span>Grounded answer ID</span><strong>{finalAnswer()!.grounded_answer_id}</strong></div>
              <div><span>Mode</span><strong>{finalAnswer()!.answer_mode}</strong></div>
              <div><span>Statements</span><strong>{finalAnswer()!.statement_count}</strong></div>
              <div><span>Needs evidence</span><strong>{finalAnswer()!.statements.filter((statement) => statement.status === "needs_evidence").length}</strong></div>
              <div><span>Unsupported</span><strong>{finalAnswer()!.unsupported_count}</strong></div>
            </div>
            {finalAnswer()!.statements.length > 0 ? (
              <div class="statement-list">
                {finalAnswer()!.statements.map((statement, index) => (
                <article class="statement-card">
                  <div class="statement-header">
                    <h3>
                      Statement {index + 1}
                    </h3>
                    <span class={`status-pill status-${statement.status}`}>
                      {statement.status}
                    </span>
                  </div>
                  <p>{statement.text}</p>
                  <div class="reference-grid">
                    <div><span>Statement ID</span><code>{statement.statement_id}</code></div>
                    <div><span>Grounded statement ID</span><code>{statement.grounded_statement_id}</code></div>
                    <div><span>Support level</span><code>{statement.support_level}</code></div>
                    <div><span>Claim IDs</span><code>{statement.claim_ids.join(", ") || "none"}</code></div>
                    <div><span>Evidence IDs</span><code>{statement.evidence_ids.join(", ") || "none"}</code></div>
                    <div><span>Chunk IDs</span><code>{statement.chunk_ids.join(", ") || "none"}</code></div>
                    <div class="full-span">
                      <span>Locators</span>
                      <div class="locator-list">
                        {statement.locators.length > 0 ? (
                          statement.locators.map((locator) => <code>{locatorSummary(locator)}</code>)
                        ) : (
                          <code>none</code>
                        )}
                      </div>
                    </div>
                  </div>
                </article>
              ))}
              </div>
            ) : (
              <p>No statements in this final answer.</p>
            )}
            {finalAnswer()!.warnings.length > 0 && (
              <div class="warning-box">
                <h3>Warnings</h3>
                <ul>
                  {finalAnswer()!.warnings.map((warning) => (
                    <li>{warning}</li>
                  ))}
                </ul>
              </div>
            )}
          </div>
        ) : (
          <p>No final answer loaded yet.</p>
        )}
      </section>
      </div>
    </WorkspaceShell>
  );
}
