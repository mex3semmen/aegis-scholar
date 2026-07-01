export type CorpusStatus = {
  source_count: number;
  registered_count: number;
  extracted_count: number;
  failed_count: number;
};

export type RegisteredSource = {
  source_id: string;
  version_id: string;
  title: string;
  source_type: string;
  ingestion_status: string;
};

export type ScholarChatMode =
  | "lecture_learning"
  | "thesis_writing"
  | "literature_review"
  | "flashcards"
  | "statistics_methods"
  | "general_scholar";

export type GroundingPolicy =
  | "local_only"
  | "local_first"
  | "allow_marked_model_knowledge"
  | "external_adapters_later";

export type ScholarChatRequest = {
  prompt: string;
  mode: ScholarChatMode;
  grounding_policy: GroundingPolicy;
  selected_source_ids: string[];
};

export type WorkspaceSection = "scholar_chat" | "sources" | "evidence_packs" | "developer_diagnostics";
export type ScholarChatTranscriptRole = "user" | "assistant" | "system";
export type ScholarChatTranscriptKind = "prompt" | "workflow_preview" | "execution_gate" | "system";

export type ScholarChatTranscriptMessage = {
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

export type ScholarChatGroundingPlan = {
  selected_source_count: number;
  local_corpus_required: boolean;
  retrieval_would_run: boolean;
  evidence_pack_would_be_required: boolean;
  model_knowledge_allowed: boolean;
  external_adapters_available: boolean;
  summary: string;
  steps: string[];
};

export type ScholarChatResponse = {
  status: "preview_only";
  normalized_prompt: string;
  mode: ScholarChatMode;
  grounding_policy: GroundingPolicy;
  selected_source_ids: string[];
  selected_source_count: number;
  grounding_plan: ScholarChatGroundingPlan;
  warnings: string[];
};

export type ScholarChatAgenticWorkflowPlanStatus = "blocked" | "needs_review" | "plan_ready_later";

export type ScholarChatAgenticWorkflowIntent =
  | "source_registration_needed"
  | "extract_text"
  | "chunk_source"
  | "build_or_inspect_retrieval"
  | "build_evidence_pack"
  | "inspect_evidence_pack"
  | "ask_local_sources"
  | "explain_blocker"
  | "unknown_or_unsupported";

export type ScholarChatAgenticWorkflowPlanPreview = {
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

export type ScholarChatAgenticWorkflowExecutionGateStatus = "blocked" | "needs_review" | "execution_ready_later";

export type ScholarChatAgenticWorkflowExecutionGateDecision =
  | "blocked"
  | "needs_context"
  | "needs_consent"
  | "ready_later"
  | "no_action_available";

export type ScholarChatAgenticWorkflowFutureAction =
  | "register_source_later"
  | "extract_text_later"
  | "chunk_source_later"
  | "inspect_retrieval_later"
  | "build_evidence_pack_later"
  | "inspect_evidence_pack_later"
  | "ask_local_sources_later"
  | "no_action_available";

export type ScholarChatAgenticWorkflowExecutionGatePreview = {
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

export type ScholarChatRetrievalCandidate = {
  source_id: string;
  version_id: string;
  chunk_id: string;
  score: number;
  matched_terms: string[];
  preview: string;
  locator: CitationLocator;
};

export type ScholarChatRetrievalPreviewResponse = {
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

export type ScholarChatEvidenceCandidate = {
  source_id: string;
  version_id: string;
  chunk_id: string;
  score: number;
  matched_terms: string[];
  preview: string;
  locator: CitationLocator;
};

export type ScholarChatEvidencePlan = {
  retrieval_candidate_count: number;
  evidence_candidate_count: number;
  evidence_required: boolean;
  evidence_pack_would_be_built_later: boolean;
  summary: string;
  steps: string[];
};

export type ScholarChatEvidencePlanResponse = {
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

export type ScholarChatPromptPackStatus = "prompt_pack_preview";

export type ScholarChatPromptPackSectionKind =
  | "system_or_policy_instructions"
  | "mode_instructions"
  | "grounding_instructions"
  | "source_context"
  | "user_prompt";

export type ScholarChatPromptPackSection = {
  kind: ScholarChatPromptPackSectionKind;
  title: string;
  lines: string[];
};

export type ScholarChatPromptContextItem = {
  source_id: string;
  version_id: string;
  chunk_id: string;
  score: number;
  matched_terms: string[];
  preview: string;
  locator: CitationLocator;
};

export type ScholarChatPromptPack = {
  section_count: number;
  context_item_count: number;
  estimated_input_char_count: number;
  sections: ScholarChatPromptPackSection[];
};

export type ScholarChatPromptPackPreviewResponse = {
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

export type ScholarChatScientificMetadataQueryPlanStatus =
  | "blocked"
  | "query_plan_ready"
  | "needs_provider_config"
  | "needs_provider_selection"
  | "needs_query_goal"
  | "needs_network_policy_review"
  | "needs_provider_terms_review"
  | "needs_institutional_access_review"
  | "unsupported_provider";

export type ScholarChatScientificMetadataProviderRequestStatus =
  | "blocked"
  | "needs_provider_config"
  | "needs_provider_selection"
  | "needs_query_goal"
  | "needs_network_policy_review"
  | "needs_provider_terms_review"
  | "needs_institutional_access_review"
  | "unsupported_provider"
  | "provider_request_ready_later";

export type ScholarChatScientificMetadataProviderRequestStrategy =
  | "blocked"
  | "provider_policy_review_first"
  | "public_metadata_request_preview"
  | "institutional_boundary_request_preview"
  | "provider_request_preview_only";

export type ScholarChatScientificMetadataQueryPlanPreviewRequest = {
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

export type ScholarChatScientificMetadataProviderRequestPreviewRequest = {
  query_plan_preview_request: ScholarChatScientificMetadataQueryPlanPreviewRequest;
  include_request_templates?: boolean;
  include_header_plan?: boolean;
  include_param_plan?: boolean;
  include_body_plan?: boolean;
};

export type ScholarChatScientificMetadataProviderRequestPreview = {
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

export type ScholarChatAnswerReadinessStatus =
  | "blocked"
  | "needs_sources"
  | "needs_retrieval_index"
  | "needs_evidence_candidates"
  | "needs_runtime_config"
  | "needs_execution_consent"
  | "ready_for_draft_inference_later"
  | "ready_for_grounded_draft_later";

export type ScholarChatAnswerReadinessOutputClassification =
  | "blocked"
  | "ungrounded_draft"
  | "source_context_draft"
  | "grounded_draft_candidate";

export type ScholarChatAnswerReadinessBlocker = {
  kind: string;
  message: string;
};

export type ScholarChatAnswerReadinessWarning = {
  kind: string;
  message: string;
};

export type ScholarChatAnswerReadinessRequest = {
  scholar_chat_request: ScholarChatRequest;
  runtime_config: LocalModelRuntimeConfig;
  allow_model_execution: boolean;
};

export type ScholarChatAnswerReadinessPreview = {
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

export type ScholarChatDraftInferenceStatus =
  | "blocked"
  | "needs_sources"
  | "needs_evidence"
  | "needs_runtime_config"
  | "needs_execution_consent"
  | "inference_succeeded"
  | "inference_failed"
  | "timed_out";

export type ScholarChatDraftOutputClassification =
  | "blocked"
  | "ungrounded_model_draft"
  | "source_context_draft"
  | "grounded_draft_candidate";

export type ScholarChatDraftInferenceBlocker = {
  kind: string;
  message: string;
};

export type ScholarChatDraftInferenceWarning = {
  kind: string;
  message: string;
};

export type ScholarChatDraftInferenceRequest = {
  scholar_chat_request: ScholarChatRequest;
  runtime_config: LocalModelRuntimeConfig;
  allow_model_execution: boolean;
  timeout_ms: number | null;
  max_output_tokens: number | null;
};

export type ScholarChatDraftInferencePreview = {
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

export type ScholarChatDraftGroundingInspectionStatus =
  | "blocked"
  | "no_draft_text"
  | "no_evidence_candidates"
  | "inspected";

export type ScholarChatDraftGroundingSupportStatus =
  | "not_inspected"
  | "unsupported"
  | "weakly_supported"
  | "supported_by_local_evidence";

export type ScholarChatDraftGroundingInspectionBlocker = {
  kind: string;
  message: string;
};

export type ScholarChatDraftGroundingInspectionWarning = {
  kind: string;
  message: string;
};

export type ScholarChatDraftGroundingInspectionRequest = {
  scholar_chat_request: ScholarChatRequest;
  draft_text: string | null;
  max_items: number | null;
};

export type ScholarChatDraftGroundingInspectionItem = {
  item_index: number;
  text_preview: string;
  support_status: ScholarChatDraftGroundingSupportStatus;
  matched_evidence_count: number;
  source_ids: string[];
  locator_previews: string[];
};

export type ScholarChatDraftGroundingInspectionPreview = {
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

export type ScholarChatGroundedDraftReadinessStatus =
  | "blocked"
  | "needs_review"
  | "ready_for_grounded_draft_later";

export type ScholarChatGroundedDraftReadinessPreview = {
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

export type LocalRuntimeVersionProbeStatus = "blocked" | "probe_succeeded" | "probe_failed" | "timed_out";

export type LocalRuntimeVersionProbePreviewRequest = {
  probe_readiness_preview_request: LocalRuntimeProbeReadinessPreviewRequest;
  allow_probe_execution: boolean;
  timeout_ms: number | null;
};

export type LocalRuntimeVersionProbePreview = {
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

export type LocalRuntimeCapabilityStatus = "blocked" | "needs_review" | "capability_ready_later";

export type LocalRuntimeCapabilityPreviewRequest = {
  version_probe_preview_request: LocalRuntimeVersionProbePreviewRequest;
};

export type LocalRuntimeCapabilityPreview = {
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

export type LocalRuntimeSmokeReadinessStatus = "blocked" | "needs_review" | "smoke_ready_later";

export type LocalRuntimeSmokeReadinessPreviewRequest = {
  capability_preview_request: LocalRuntimeCapabilityPreviewRequest;
  smoke_consent: boolean;
  diagnostic_prompt: string | null;
  max_output_tokens: number | null;
  timeout_ms: number | null;
};

export type LocalRuntimeSmokeReadinessPreview = {
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

export type LocalRuntimeSmokeExecutionPlanStatus = "blocked" | "needs_review" | "plan_ready_later";

export type LocalRuntimeSmokeExecutionPlanPreviewRequest = {
  smoke_readiness_preview_request: LocalRuntimeSmokeReadinessPreviewRequest;
};

export type LocalRuntimeSmokeExecutionPlanPreview = {
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

export type ScholarChatGroundedAnswerBuildPlanStatus =
  | "blocked"
  | "needs_review"
  | "plan_ready_later";

export type ScholarChatGroundedAnswerBuildPlanPreview = {
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

export type ScholarChatGroundedAnswerCandidateStatus =
  | "blocked"
  | "needs_review"
  | "candidate_ready_later";

export type ScholarChatGroundedAnswerCandidateItem = {
  item_index: number;
  statement_preview: string;
  support_status: ScholarChatDraftGroundingSupportStatus;
  source_ids: string[];
  locator_previews: string[];
  matched_evidence_count: number;
};

export type ScholarChatGroundedAnswerCandidatePreview = {
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

export type ScholarChatGroundedAnswerWriteEligibilityStatus =
  | "blocked"
  | "needs_review"
  | "write_eligible_later";

export type ScholarChatGroundedAnswerWriteEligibilityPreview = {
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

export type ScholarChatGroundedAnswerBuildIntentStatus =
  | "blocked"
  | "needs_review"
  | "intent_ready_later";

export type ScholarChatGroundedAnswerBuildIntentRequest = {
  grounding_request: ScholarChatDraftGroundingInspectionRequest;
  answer_draft_id: string | null;
  explicit_user_intent: boolean;
};

export type ScholarChatGroundedAnswerBuildIntentPreview = {
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

export type ScholarChatGroundedAnswerBuildRequestStatus =
  | "blocked"
  | "needs_review"
  | "request_ready_later";

export type ScholarChatGroundedAnswerBuildRequestPreviewRequest = {
  build_intent_request: ScholarChatGroundedAnswerBuildIntentRequest;
};

export type ScholarChatGroundedAnswerBuildRequestPreview = {
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

export type ScholarChatGroundedAnswerBuildPreflightStatus =
  | "blocked"
  | "needs_review"
  | "preflight_ready_later";

export type ScholarChatGroundedAnswerBuildPreflightPreviewRequest = {
  build_request_preview_request: ScholarChatGroundedAnswerBuildRequestPreviewRequest;
};

export type ScholarChatGroundedAnswerBuildPreflightPreview = {
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

export type ScholarChatGroundedAnswerExecutionReadinessStatus =
  | "blocked"
  | "needs_review"
  | "execution_ready_later";

export type ScholarChatGroundedAnswerExecutionReadinessPreviewRequest = {
  build_preflight_preview_request: ScholarChatGroundedAnswerBuildPreflightPreviewRequest;
  execution_consent: boolean;
};

export type ScholarChatGroundedAnswerExecutionReadinessPreview = {
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

export type ScholarChatGroundedAnswerExecutionPlanStatus =
  | "blocked"
  | "needs_review"
  | "plan_ready_later";

export type ScholarChatGroundedAnswerExecutionPlanPreviewRequest = {
  execution_readiness_preview_request: ScholarChatGroundedAnswerExecutionReadinessPreviewRequest;
};

export type ScholarChatGroundedAnswerExecutionPlanPreview = {
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

export type ScholarChatRuntimeDiagnosticBridgeStatus =
  | "blocked"
  | "needs_review"
  | "runtime_diagnostic_ready_later";

export type ScholarChatRuntimeDiagnosticBridgePreviewRequest = {
  scholar_chat_request: ScholarChatRequest;
  smoke_execution_plan_preview_request: LocalRuntimeSmokeExecutionPlanPreviewRequest;
};

export type ScholarChatRuntimeDiagnosticBridgePreview = {
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

export type ScholarChatRuntimeDiagnosticResultStatus =
  | "blocked"
  | "needs_review"
  | "runtime_diagnostic_failed"
  | "runtime_diagnostic_succeeded_later";

export type ScholarChatRuntimeDiagnosticResultPreviewRequest = {
  bridge_preview_request: ScholarChatRuntimeDiagnosticBridgePreviewRequest;
  diagnostic_preview: LocalRuntimeSmokeDiagnosticPreview;
};

export type ScholarChatRuntimeDiagnosticResultPreview = {
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

export type LocalModelRuntimeKind = "llama_cpp" | "none";

export type LocalModelRuntimeHealthStatus =
  | "not_configured"
  | "config_present"
  | "model_missing"
  | "executable_missing"
  | "ready_to_test_later";

export type LocalModelRuntimePathState = "not_configured" | "missing" | "exists";

export type LocalModelRuntimeHealthWarning = {
  kind: string;
  message: string;
};

export type LocalModelRuntimeConfig = {
  runtime_kind: LocalModelRuntimeKind;
  model_path: string | null;
  executable_path: string | null;
  context_window: number | null;
  gpu_layers: number | null;
  temperature: number | null;
};

export type LocalRuntimeAdapterKind = "llama_cpp";

export type LocalRuntimeAdapterContractStatus =
  | "blocked"
  | "needs_review"
  | "contract_ready_later";

export type LocalRuntimeAdapterContractBlocker = {
  kind: string;
  message: string;
};

export type LocalRuntimeAdapterContractWarning = {
  kind: string;
  message: string;
};

export type LocalRuntimeAdapterContractPreviewRequest = {
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

export type LocalRuntimeAdapterContractPreview = {
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

export type LocalRuntimeValidationStatus =
  | "blocked"
  | "needs_review"
  | "validation_ready_later";

export type LocalRuntimeValidationPreviewRequest = {
  adapter_contract_request: LocalRuntimeAdapterContractPreviewRequest;
};

export type LocalRuntimeValidationPreview = {
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

export type LocalRuntimeProbeReadinessStatus =
  | "blocked"
  | "needs_review"
  | "probe_ready_later";

export type LocalRuntimeProbeReadinessPreviewRequest = {
  validation_preview_request: LocalRuntimeValidationPreviewRequest;
  probe_consent: boolean;
};

export type LocalRuntimeProbeReadinessPreview = {
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

export type LocalModelRuntimeHealthPreview = {
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

export type ManagedLlamaServerLaunchPlanStatus = "blocked" | "launch_ready_later";
export type ManagedLlamaServerLifecycleStatus =
  | "not_started"
  | "starting"
  | "running"
  | "stopped"
  | "failed"
  | "blocked"
  | "already_running"
  | "port_occupied"
  | "external_server_detected";
export type ManagedLlamaServerHealthStatus = "not_started" | "loading" | "ready" | "unreachable" | "failed";
export type ManagedLlamaServerPortOccupancyStatus = "free" | "managed_owned" | "external_server_detected" | "port_occupied" | "unknown_owner";

export type ManagedLlamaServerNotice = {
  kind: string;
  message: string;
};

export type ManagedLlamaServerLaunchPlanRequest = {
  executable_path: string | null;
  model_path: string | null;
  host: string | null;
  port: number | null;
  alias: string | null;
  context_window: number | null;
  gpu_layers: number | null;
};

export type ManagedLlamaServerStartRequest = {
  allow_server_start: boolean;
  launch_plan_request: ManagedLlamaServerLaunchPlanRequest;
};

export type ManagedLlamaServerLaunchPlanPreview = {
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

export type ManagedLlamaServerStatusPreview = {
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

export type ManagedLlamaServerChatDiagnosticStatus =
  | "blocked"
  | "server_not_ready"
  | "diagnostic_succeeded"
  | "diagnostic_failed"
  | "timed_out";

export type ManagedLlamaServerChatDiagnosticRequest = {
  allow_chat_diagnostic: boolean;
  prompt: string | null;
  max_tokens: number | null;
  temperature: number | null;
  timeout_ms: number | null;
};

export type ManagedLlamaServerChatDiagnosticPreview = {
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

export type ManagedLlamaServerSmokeDiagnosticStatus = "blocked" | "server_not_running" | "smoke_succeeded" | "smoke_failed" | "timed_out";

export type ManagedLlamaServerSmokeDiagnosticRequest = {
  allow_smoke_execution: boolean;
  prompt: string | null;
  max_output_tokens: number | null;
  timeout_ms: number | null;
};

export type ManagedLlamaServerSmokeDiagnosticPreview = {
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

export type LocalRuntimeInvocationPlanStatus =
  | "not_configured"
  | "blocked"
  | "ready_to_invoke_later"
  | "preview_only";

export type LocalRuntimeInvocationBlocker = {
  kind: string;
  message: string;
};

export type LocalRuntimeInvocationPlan = {
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

export type LocalRuntimeInvocationPlanPreview = {
  status: LocalRuntimeInvocationPlanStatus;
  runtime_kind: LocalModelRuntimeKind;
  plan: LocalRuntimeInvocationPlan;
};

export type LocalRuntimeProbeStatus = "blocked" | "completed" | "timed_out";

export type LocalRuntimeProbeWarning = {
  kind: string;
  message: string;
};

export type LocalRuntimeProbeRequest = {
  executable_path: string | null;
  allow_execution: boolean;
  timeout_ms: number | null;
};

export type LocalRuntimeProbeResult = {
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

export type LocalRuntimeSmokeInferenceStatus =
  | "blocked"
  | "not_configured"
  | "model_missing"
  | "executable_missing"
  | "inference_succeeded"
  | "inference_failed"
  | "timed_out";

export type LocalRuntimeSmokeInferenceWarning = {
  kind: string;
  message: string;
};

export type LocalRuntimeSmokeInferenceOutputClassification = "runtime_diagnostic";

export type LocalRuntimeSmokeInferenceBlocker = {
  kind: string;
  message: string;
};

export type LocalRuntimeSmokeInferenceRequest = {
  runtime_config: LocalModelRuntimeConfig;
  allow_execution: boolean;
  prompt: string | null;
  timeout_ms: number | null;
  max_output_tokens: number | null;
};

export type LocalRuntimeSmokeInferenceResult = {
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

export type LocalRuntimeSmokeDiagnosticStatus = "blocked" | "smoke_succeeded" | "smoke_failed" | "timed_out";

export type LocalRuntimeSmokeDiagnosticRequest = {
  smoke_execution_plan_preview_request: LocalRuntimeSmokeExecutionPlanPreviewRequest;
  allow_smoke_execution: boolean;
};

export type LocalRuntimeSmokeDiagnosticPreview = {
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

export type LocalRuntimeInvocationPlanRequest = {
  runtime_config: LocalModelRuntimeConfig;
  prompt_text: string | null;
  estimated_input_char_count: number | null;
  max_output_tokens: number | null;
  stop_sequences: string[] | null;
};

export type RetrievalIndexEntry = {
  chunk_id: string;
  source_id: string;
  version_id: string;
  locator: CitationLocator;
  text_hash: string;
  normalized_terms: string[];
};

export type RetrievalIndex = {
  source_id: string;
  version_id: string;
  indexed_at: string;
  chunk_count: number;
  index_version: string;
  chunk_report_hash: string;
  entries: RetrievalIndexEntry[];
  warnings: string[];
};

export type RetrievalSearchResult = {
  chunk_id: string;
  source_id: string;
  version_id: string;
  locator: CitationLocator;
  score: number;
  matched_terms: string[];
  text_hash: string;
  preview: string;
};

export type RetrievalSearchResponse = {
  query: string;
  normalized_query_terms: string[];
  result_count: number;
  results: RetrievalSearchResult[];
};

export type CitationLocator = {
  label: string;
  section?: string | null;
  paragraph_index?: number | null;
  start_char: number;
  end_char: number;
  [key: string]: unknown;
};

export type FinalAnswerStatement = {
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

export type FinalAnswer = {
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

export type FinalAnswerMetadata = {
  final_answer_id: string;
  grounded_answer_id: string;
  statement_count: number;
  unsupported_count: number;
  needs_evidence_count: number;
};

export type AnswerArtifactOverview = {
  source_id: string;
  draft_count: number;
  grounded_answer_count: number;
  final_answer_count: number;
  final_answers: FinalAnswerMetadata[];
};

export type AnswerArtifactSourceMetadata = {
  source_id: string;
  draft_count: number;
  grounded_answer_count: number;
  final_answer_count: number;
};

export type AnswerArtifactSourceHealth = {
  source_id: string;
  draft_count: number;
  grounded_answer_count: number;
  final_answer_count: number;
  malformed_final_answer_count: number;
  unsupported_statement_count: number;
  needs_evidence_statement_count: number;
};

export type AnswerArtifactHealth = {
  source_count: number;
  draft_count: number;
  grounded_answer_count: number;
  final_answer_count: number;
  malformed_final_answer_count: number;
  unsupported_statement_count: number;
  needs_evidence_statement_count: number;
  sources: AnswerArtifactSourceHealth[];
};

export type AnswerArtifactIssue = {
  source_id: string;
  issue_kind: "malformed_final_answer" | "unsupported_statement" | "needs_evidence_statement";
  final_answer_id?: string | null;
  grounded_answer_id?: string | null;
  statement_index?: number | null;
  statement_status?: string | null;
  message: string;
};

export type EvidencePackMetadata = {
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

export type AnswerArtifactExportSource = {
  source_id: string;
  draft_count: number;
  grounded_answer_count: number;
  final_answer_count: number;
  final_answers: FinalAnswerMetadata[];
  issue_count: number;
};

export type AnswerArtifactExportManifest = {
  schema_version: string;
  source_count: number;
  draft_count: number;
  grounded_answer_count: number;
  final_answer_count: number;
  issue_count: number;
  sources: AnswerArtifactExportSource[];
};

export type ExportedArtifactFile = {
  relative_path: string;
  artifact_kind: "manifest" | "issues" | "summary" | "integrity" | "answer_draft" | "grounded_answer" | "final_answer";
  source_id?: string | null;
  artifact_id?: string | null;
};

export type AnswerArtifactExportIntegrityFile = {
  relative_path: string;
  byte_count: number;
  sha256: string;
};

export type AnswerArtifactExportIntegrity = {
  schema_version: string;
  algorithm: string;
  files: AnswerArtifactExportIntegrityFile[];
};

export type AnswerArtifactExportResult = {
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

export type AnswerArtifactExportIssueKindCount = {
  issue_kind: "malformed_final_answer" | "needs_evidence_statement" | "unsupported_statement";
  count: number;
};

export type AnswerArtifactExportBundleInspectionIssueKindCount = {
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

export type AnswerArtifactExportBundleInspectionSummary = {
  is_consistent: boolean;
  schema_supported: boolean;
  integrity_verified: boolean;
  issue_count: number;
  warning_count: number;
  issue_counts_by_kind: AnswerArtifactExportBundleInspectionIssueKindCount[];
  checked_file_count: number;
  integrity_file_count: number;
};

export type AnswerArtifactExportBundleInspectionReportSection = {
  heading: string;
  lines: string[];
};

export type AnswerArtifactExportBundleInspectionReportPreview = {
  title: string;
  schema_version: string;
  is_consistent: boolean;
  integrity_verified: boolean;
  issue_count: number;
  warning_count: number;
  issue_counts_by_kind: AnswerArtifactExportBundleInspectionIssueKindCount[];
  sections: AnswerArtifactExportBundleInspectionReportSection[];
};

export type AnswerArtifactExportBundleInspectionIssueGroup = {
  kind: AnswerArtifactExportBundleInspectionIssueKind;
  count: number;
  lines: string[];
};

export type AnswerArtifactExportBundleInspectionStatus = {
  code: string;
  label: string;
  severity: string;
  reason: string;
};

export type AnswerArtifactExportBundleFileStatus = {
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

export type AnswerArtifactExportSummarySource = {
  source_id: string;
  draft_count: number;
  grounded_answer_count: number;
  final_answer_count: number;
  issue_count: number;
};

export type AnswerArtifactExportSummary = {
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

export type AnswerArtifactExportBundleInspectionIssueKind =
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

export type AnswerArtifactExportBundleInspectionIssue = {
  kind: AnswerArtifactExportBundleInspectionIssueKind;
  message: string;
  relative_path?: string | null;
};

export type AnswerArtifactExportBundleInspection = {
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
