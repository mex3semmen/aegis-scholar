import { createSignal, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/core";

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
  const [status, setStatus] = createSignal<CorpusStatus | null>(null);
  const [statusError, setStatusError] = createSignal<string | null>(null);
  const [scholarChatPrompt, setScholarChatPrompt] = createSignal("");
  const [scholarChatMode, setScholarChatMode] = createSignal<ScholarChatMode>("lecture_learning");
  const [scholarChatGroundingPolicy, setScholarChatGroundingPolicy] = createSignal<GroundingPolicy>("local_first");
  const [scholarChatPreview, setScholarChatPreview] = createSignal<ScholarChatResponse | null>(null);
  const [scholarChatError, setScholarChatError] = createSignal<string | null>(null);
  const [scholarChatValidationError, setScholarChatValidationError] = createSignal<string | null>(null);
  const [scholarChatLoading, setScholarChatLoading] = createSignal(false);
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
  const [localRuntimeKind, setLocalRuntimeKind] = createSignal<LocalModelRuntimeKind>("none");
  const [localRuntimeModelPath, setLocalRuntimeModelPath] = createSignal("");
  const [localRuntimeExecutablePath, setLocalRuntimeExecutablePath] = createSignal("");
  const [localRuntimeContextWindow, setLocalRuntimeContextWindow] = createSignal("");
  const [localRuntimeGpuLayers, setLocalRuntimeGpuLayers] = createSignal("");
  const [localRuntimeTemperature, setLocalRuntimeTemperature] = createSignal("");
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
  const [localRuntimeProbeResult, setLocalRuntimeProbeResult] = createSignal<LocalRuntimeProbeResult | null>(null);
  const [localRuntimeProbeError, setLocalRuntimeProbeError] = createSignal<string | null>(null);
  const [localRuntimeProbeValidationError, setLocalRuntimeProbeValidationError] = createSignal<string | null>(null);
  const [localRuntimeProbeLoading, setLocalRuntimeProbeLoading] = createSignal(false);
  const [localRuntimeProbeHasRun, setLocalRuntimeProbeHasRun] = createSignal(false);
  const [localRuntimeSmokePrompt, setLocalRuntimeSmokePrompt] = createSignal("Say READY in one short sentence.");
  const [localRuntimeSmokeAllowExecution, setLocalRuntimeSmokeAllowExecution] = createSignal(false);
  const [localRuntimeSmokeTimeoutMs, setLocalRuntimeSmokeTimeoutMs] = createSignal("3000");
  const [localRuntimeSmokeMaxOutputTokens, setLocalRuntimeSmokeMaxOutputTokens] = createSignal("8");
  const [localRuntimeSmokeResult, setLocalRuntimeSmokeResult] = createSignal<LocalRuntimeSmokeInferenceResult | null>(null);
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
      setStatusError(String(err));
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

  function formatSnakeCaseLabel(value: string) {
    return value
      .replace(/_/g, " ")
      .split(" ")
      .filter(Boolean)
      .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
      .join(" ");
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
    clearScholarChatAnswerReadinessPreview();
    clearScholarChatDraftInferencePreview();
    clearLocalRuntimeInvocationPreview();
  }

  function clearScholarChatPromptPackPreview() {
    setScholarChatPromptPackPreview(null);
    setScholarChatPromptPackError(null);
    setScholarChatPromptPackHasRun(false);
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
    clearScholarChatDraftInferencePreview();
  }

  function clearLocalRuntimeSmokePreview() {
    setLocalRuntimeSmokeResult(null);
    setLocalRuntimeSmokeError(null);
    setLocalRuntimeSmokeValidationError(null);
    setLocalRuntimeSmokeHasRun(false);
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

  async function previewScholarChatRequest() {
    const trimmedPrompt = scholarChatPrompt().trim();
    if (!trimmedPrompt) {
      setScholarChatPreview(null);
      setScholarChatError(null);
      setScholarChatValidationError("Prompt is required to preview a Scholar Chat request.");
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
      const result = await invoke<ScholarChatResponse>("preview_scholar_chat_request", {
        root: ".",
        request: {
          prompt: trimmedPrompt,
          mode: scholarChatMode(),
          grounding_policy: scholarChatGroundingPolicy(),
          selected_source_ids: selectedScholarChatSourceIds(),
        },
      });
      setScholarChatPreview(result);
    } catch (err) {
      setScholarChatError(sanitizeBackendError(err));
    } finally {
      setScholarChatLoading(false);
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

  async function previewLocalRuntimeVersionProbe() {
    const timeoutMs = parseOptionalIntegerInput(
      localRuntimeProbeTimeoutMs(),
      "Probe timeout",
      setLocalRuntimeProbeValidationError,
    );
    if (timeoutMs === undefined) {
      setLocalRuntimeProbeHasRun(true);
      setLocalRuntimeProbeResult(null);
      setLocalRuntimeProbeError(null);
      return;
    }
    if (localRuntimeProbeLoading()) {
      return;
    }

    setLocalRuntimeProbeHasRun(true);
    setLocalRuntimeProbeLoading(true);
    setLocalRuntimeProbeError(null);
    setLocalRuntimeProbeValidationError(null);
    setLocalRuntimeProbeResult(null);
    try {
      const result = await invoke<LocalRuntimeProbeResult>("probe_local_runtime_version", {
        root: ".",
        request: {
          executable_path: normalizeOptionalTextInput(localRuntimeExecutablePath()),
          allow_execution: localRuntimeProbeAllowExecution(),
          timeout_ms: timeoutMs,
        } satisfies LocalRuntimeProbeRequest,
      });
      setLocalRuntimeProbeResult(result);
    } catch (err) {
      setLocalRuntimeProbeError(sanitizeBackendError(err));
    } finally {
      setLocalRuntimeProbeLoading(false);
    }
  }

  async function previewLocalRuntimeSmokeInference() {
    if (localRuntimeSmokeLoading()) {
      return;
    }

    const contextWindow = parseOptionalIntegerInput(
      localRuntimeContextWindow(),
      "Context window",
      setLocalRuntimeSmokeValidationError,
    );
    const gpuLayers = parseOptionalIntegerInput(
      localRuntimeGpuLayers(),
      "GPU layers",
      setLocalRuntimeSmokeValidationError,
    );
    const temperature = parseOptionalNumberInput(
      localRuntimeTemperature(),
      "Temperature",
      setLocalRuntimeSmokeValidationError,
    );
    const timeoutMs = parseOptionalIntegerInput(
      localRuntimeSmokeTimeoutMs(),
      "Smoke timeout",
      setLocalRuntimeSmokeValidationError,
    );
    const maxOutputTokens = parseOptionalIntegerInput(
      localRuntimeSmokeMaxOutputTokens(),
      "Max output tokens",
      setLocalRuntimeSmokeValidationError,
    );
    if (
      contextWindow === undefined ||
      gpuLayers === undefined ||
      temperature === undefined ||
      timeoutMs === undefined ||
      maxOutputTokens === undefined
    ) {
      setLocalRuntimeSmokeHasRun(true);
      setLocalRuntimeSmokeResult(null);
      setLocalRuntimeSmokeError(null);
      return;
    }

    const trimmedPrompt = localRuntimeSmokePrompt().trim();
    if (!trimmedPrompt) {
      setLocalRuntimeSmokeHasRun(true);
      setLocalRuntimeSmokeResult(null);
      setLocalRuntimeSmokeError(null);
      setLocalRuntimeSmokeValidationError("Prompt is required to run a smoke inference probe.");
      return;
    }

    setLocalRuntimeSmokeHasRun(true);
    setLocalRuntimeSmokeLoading(true);
    setLocalRuntimeSmokeError(null);
    setLocalRuntimeSmokeValidationError(null);
    setLocalRuntimeSmokeResult(null);
    try {
      const result = await invoke<LocalRuntimeSmokeInferenceResult>("smoke_test_local_runtime_inference", {
        root: ".",
        request: {
          runtime_config: {
            runtime_kind: localRuntimeKind(),
            model_path: normalizeOptionalTextInput(localRuntimeModelPath()),
            executable_path: normalizeOptionalTextInput(localRuntimeExecutablePath()),
            context_window: contextWindow,
            gpu_layers: gpuLayers,
            temperature,
          },
          allow_execution: localRuntimeSmokeAllowExecution(),
          prompt: trimmedPrompt,
          timeout_ms: timeoutMs,
          max_output_tokens: maxOutputTokens,
        } satisfies LocalRuntimeSmokeInferenceRequest,
      });
      setLocalRuntimeSmokeResult(result);
    } catch (err) {
      setLocalRuntimeSmokeError(sanitizeBackendError(err));
    } finally {
      setLocalRuntimeSmokeLoading(false);
    }
  }

  onMount(() => {
    void loadScholarChatSourceContext();
  });

  return (
    <main class="app-shell">
      <section class="hero">
        <p class="eyebrow">AEGIS Scholar</p>
        <h1>Scientific knowledge OS</h1>
        <p>
          Minimal Phase 1 scaffold for Corpus Authority and Source Registry.
          No RAG, no model runtime, no coding-agent behavior.
        </p>
        <div class="hero-actions">
          <button onClick={loadStatus}>Check corpus status</button>
        </div>
      </section>

      <section class="card">
        <h2>Corpus status</h2>
        {status() ? (
          <pre>{JSON.stringify(status(), null, 2)}</pre>
        ) : (
          <p>No status loaded yet.</p>
        )}
        {statusError() && <p class="error">{statusError()}</p>}
      </section>

      <section class="card">
        <h2>Scholar Chat</h2>
        <p class="muted">
          Preview-only request shell for the future local-first Scholar Chat workflow. It can preview grounding plans and retrieval candidates from existing local data, but it does not run answer generation, model calls, or Evidence Pack builds.
        </p>
        <div class="form-row">
          <label>
            Mode
            <select
              value={scholarChatMode()}
              onChange={(event) => {
                setScholarChatMode(event.currentTarget.value as ScholarChatMode);
                clearScholarChatPromptPackPreview();
                clearScholarChatDraftInferencePreview();
              }}
            >
              {SCHOLAR_CHAT_MODES.map((item) => (
                <option value={item.value}>{item.label}</option>
              ))}
            </select>
          </label>
          <label>
            Grounding policy
            <select
              value={scholarChatGroundingPolicy()}
              onChange={(event) => {
                setScholarChatGroundingPolicy(event.currentTarget.value as GroundingPolicy);
                clearScholarChatPromptPackPreview();
                clearScholarChatDraftInferencePreview();
              }}
            >
              {GROUNDING_POLICIES.map((item) => (
                <option value={item.value}>{item.label}</option>
              ))}
            </select>
          </label>
        </div>
        <div class="artifact-overview">
          <h3>Source context</h3>
          <p class="muted">
            Choose Scholar Chat source IDs to scope the preview. Existing diagnostic source selection remains a fallback until you set Scholar Chat context.
          </p>
          {scholarChatSourceContextLoading() ? (
            <p>Loading registered sources...</p>
          ) : scholarChatSourceContextError() ? (
            <p class="error">{scholarChatSourceContextError()}</p>
          ) : scholarChatSourceContext().length === 0 ? (
            <p>No registered sources yet.</p>
          ) : (
            <>
              <p class="muted">Selected source count: {scholarChatSourceContextSelectedIds().length}</p>
              <ul class="final-answer-list-items">
                {scholarChatSourceContext().map((item) => (
                  <li>
                    <label class="final-answer-list-item">
                      <span>
                        <input
                          type="checkbox"
                          checked={scholarChatSourceContextSelectedIds().includes(item.source_id)}
                          onChange={() => toggleScholarChatSourceContext(item.source_id)}
                        />
                        <strong> {item.title || item.source_id}</strong>
                      </span>
                      <small>
                        source_id={item.source_id} | type={formatSnakeCaseLabel(item.source_type)} | version={item.version_id} | status={formatSnakeCaseLabel(item.ingestion_status)}
                      </small>
                    </label>
                  </li>
                ))}
              </ul>
            </>
          )}
          <p class="muted">{scholarChatSelectedSourceIdsSummary()}</p>
        </div>
        <label>
          Prompt
          <textarea
            rows={4}
            value={scholarChatPrompt()}
            onInput={(event) => {
              setScholarChatPrompt(event.currentTarget.value);
              setScholarChatValidationError(null);
              clearScholarChatPromptPackPreview();
              clearScholarChatDraftInferencePreview();
            }}
            placeholder="Ask about a lecture, paper, method, or thesis task..."
          />
        </label>
        {scholarChatValidationError() && <p class="error">{scholarChatValidationError()}</p>}
        {scholarChatError() && <p class="error">{scholarChatError()}</p>}
        <div class="hero-actions">
          <button onClick={previewScholarChatRequest} disabled={scholarChatLoading()}>
            {scholarChatLoading() ? "Previewing..." : "Preview grounding plan"}
          </button>
        </div>
        {scholarChatPreview() ? (
          <div class="artifact-overview">
            <h3>Preview result</h3>
            {renderMetricGrid([
              { label: "Status", value: scholarChatPreview()!.status },
              { label: "Mode", value: scholarChatPreview()!.mode },
              { label: "Grounding policy", value: scholarChatPreview()!.grounding_policy },
              { label: "Selected sources", value: scholarChatPreview()!.selected_source_count },
            ])}
            <p><strong>Prompt:</strong> {scholarChatPreview()!.normalized_prompt}</p>
            <p>{scholarChatPreview()!.grounding_plan.summary}</p>
            <div class="contract-meta">
              <div><span>Retrieval would run</span><strong>{scholarChatPreview()!.grounding_plan.retrieval_would_run ? "yes" : "no"}</strong></div>
              <div><span>Evidence Pack required</span><strong>{scholarChatPreview()!.grounding_plan.evidence_pack_would_be_required ? "yes" : "no"}</strong></div>
              <div><span>Model knowledge allowed</span><strong>{scholarChatPreview()!.grounding_plan.model_knowledge_allowed ? "yes" : "no"}</strong></div>
              <div><span>External adapters</span><strong>{scholarChatPreview()!.grounding_plan.external_adapters_available ? "available" : "not available"}</strong></div>
            </div>
            <h4>Plan steps</h4>
            <ul>
              {scholarChatPreview()!.grounding_plan.steps.map((step) => (
                <li>{step}</li>
              ))}
            </ul>
            {scholarChatPreview()!.warnings.length > 0 ? (
              <div class="warning-box">
                <h4>Warnings</h4>
                <ul>
                  {scholarChatPreview()!.warnings.map((warning) => (
                    <li>{warning}</li>
                  ))}
                </ul>
              </div>
            ) : null}
          </div>
        ) : (
          <p>No Scholar Chat preview loaded yet.</p>
        )}
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
          <h3>Local model runtime</h3>
          <p class="muted">
            Read-only local model runtime readiness preview. It only checks configured paths and preview settings; no model is executed, no answer is generated, and nothing is persisted.
          </p>
          <div class="form-row">
            <label>
              Runtime kind
              <select
                value={localRuntimeKind()}
              onChange={(event) => {
                setLocalRuntimeKind(event.currentTarget.value as LocalModelRuntimeKind);
                clearLocalRuntimePreview();
                clearLocalRuntimeInvocationPreview();
                clearLocalRuntimeSmokePreview();
                clearScholarChatDraftInferencePreview();
              }}
              >
                <option value="none">None</option>
                <option value="llama_cpp">llama_cpp</option>
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
                  clearLocalRuntimeSmokePreview();
                  clearScholarChatDraftInferencePreview();
                }}
                placeholder="0.7"
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
                  clearLocalRuntimeSmokePreview();
                  clearScholarChatDraftInferencePreview();
                }}
                placeholder="E:\\models\\scholar.gguf"
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
                placeholder="E:\\bin\\llama-server.exe"
              />
            </label>
          </div>
          <div class="hero-actions">
            <button onClick={previewLocalRuntimeHealth} disabled={localRuntimeLoading()}>
              {localRuntimeLoading() ? "Previewing..." : "Preview runtime health"}
            </button>
          </div>
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
              <p>No local model runtime preview loaded yet.</p>
            )
          ) : (
            <p>No local model runtime preview loaded yet.</p>
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
          <h3>Runtime version probe</h3>
          <p class="muted">
            Read-only version/help probe for the configured executable. It uses a fixed <code>--version</code> argument, does not load a model, and does not send a prompt or generate an answer.
          </p>
          <div class="form-row">
            <label class="inline-field">
              Allow execution
              <input
                type="checkbox"
                checked={localRuntimeProbeAllowExecution()}
                onChange={(event) => {
                  setLocalRuntimeProbeAllowExecution(event.currentTarget.checked);
                  clearLocalRuntimeProbePreview();
                }}
              />
            </label>
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
              {localRuntimeProbeLoading() ? "Probing..." : "Run version probe"}
            </button>
          </div>
          <p class="muted">No model is loaded. No prompt is sent. No answer is generated.</p>
          {localRuntimeProbeValidationError() && <p class="error">{localRuntimeProbeValidationError()}</p>}
          {localRuntimeProbeError() && <p class="error">{localRuntimeProbeError()}</p>}
          {localRuntimeProbeLoading() ? (
            <p>Running version probe...</p>
          ) : localRuntimeProbeHasRun() ? (
            localRuntimeProbeResult() ? (
              <>
                {renderMetricGrid([
                  { label: "Status", value: formatSnakeCaseLabel(localRuntimeProbeResult()!.status) },
                  { label: "Allow execution", value: localRuntimeProbeResult()!.allow_execution ? "yes" : "no" },
                  { label: "Execution attempted", value: localRuntimeProbeResult()!.execution_attempted ? "yes" : "no" },
                  { label: "Timeout ms", value: localRuntimeProbeResult()!.timeout_ms },
                  { label: "Duration ms", value: localRuntimeProbeResult()!.duration_ms },
                  { label: "Exit code", value: localRuntimeProbeResult()!.exit_code ?? "missing" },
                ])}
                <div class="contract-meta">
                  <div><span>Executable file</span><strong>{localRuntimeProbeResult()!.safe_executable_file_name ?? "not configured"}</strong></div>
                  <div><span>Probe argument</span><strong>{localRuntimeProbeResult()!.probe_argument}</strong></div>
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
                  <p>No probe blockers.</p>
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
                  <p>No probe warnings.</p>
                )}
              </>
            ) : (
              <p>No runtime version probe preview loaded yet.</p>
            )
          ) : (
            <p>No runtime version probe preview loaded yet.</p>
          )}
        </div>
        <div class="artifact-overview">
          <h3>Local runtime smoke test</h3>
          <p class="muted">
            Read-only smoke probe for the configured local runtime. It uses a tiny prompt only when execution is allowed, and it never persists a result, generates a Scholar Chat answer, or builds an Evidence Pack.
          </p>
          <label>
            Prompt
            <textarea
              rows={3}
              value={localRuntimeSmokePrompt()}
              onInput={(event) => {
                setLocalRuntimeSmokePrompt(event.currentTarget.value);
                clearLocalRuntimeSmokePreview();
              }}
              placeholder="Say READY in one short sentence."
            />
          </label>
          <div class="form-row">
            <label class="inline-field">
              Allow execution
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
                }}
                placeholder="3000"
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
                }}
                placeholder="8"
              />
            </label>
          </div>
          <p class="muted">Runtime diagnostic only - not a Scholar Chat answer.</p>
          <div class="hero-actions">
            <button onClick={previewLocalRuntimeSmokeInference} disabled={localRuntimeSmokeLoading()}>
              {localRuntimeSmokeLoading() ? "Previewing..." : "Run smoke inference probe"}
            </button>
          </div>
          {localRuntimeSmokeValidationError() && <p class="error">{localRuntimeSmokeValidationError()}</p>}
          {localRuntimeSmokeError() && <p class="error">{localRuntimeSmokeError()}</p>}
          {localRuntimeSmokeLoading() ? (
            <p>Running smoke inference probe...</p>
          ) : localRuntimeSmokeHasRun() ? (
            localRuntimeSmokeResult() ? (
              <>
                {renderMetricGrid([
                  { label: "Status", value: formatSnakeCaseLabel(localRuntimeSmokeResult()!.status) },
                  { label: "Output classification", value: formatSnakeCaseLabel(localRuntimeSmokeResult()!.output_classification) },
                  { label: "Allow execution", value: localRuntimeSmokeResult()!.allow_execution ? "yes" : "no" },
                  { label: "Execution attempted", value: localRuntimeSmokeResult()!.execution_attempted ? "yes" : "no" },
                  { label: "Runtime kind", value: formatSnakeCaseLabel(localRuntimeSmokeResult()!.runtime_kind) },
                  { label: "Prompt chars", value: localRuntimeSmokeResult()!.prompt_char_count },
                  { label: "Max output tokens", value: localRuntimeSmokeResult()!.max_output_tokens },
                  { label: "Timeout ms", value: localRuntimeSmokeResult()!.timeout_ms },
                  { label: "Duration ms", value: localRuntimeSmokeResult()!.duration_ms },
                  { label: "Exit code", value: localRuntimeSmokeResult()!.exit_code ?? "missing" },
                  { label: "Diagnostic only", value: localRuntimeSmokeResult()!.diagnostic_only ? "yes" : "no" },
                  { label: "No answer generated", value: localRuntimeSmokeResult()!.no_answer_generated ? "yes" : "no" },
                  { label: "No grounding applied", value: localRuntimeSmokeResult()!.no_grounding_applied ? "yes" : "no" },
                  { label: "No Evidence Pack used", value: localRuntimeSmokeResult()!.no_evidence_pack_used ? "yes" : "no" },
                  { label: "Not Scholar Chat answer", value: localRuntimeSmokeResult()!.not_scholar_chat_answer ? "yes" : "no" },
                ])}
                <p class="muted">This preview shows runtime diagnostics only; no grounded answer, Evidence Pack, or final answer is produced.</p>
                {localRuntimeSmokeResult()!.safe_model_file_name ? (
                  <p><strong>Model file name:</strong> {localRuntimeSmokeResult()!.safe_model_file_name}</p>
                ) : null}
                {localRuntimeSmokeResult()!.safe_executable_file_name ? (
                  <p><strong>Executable file name:</strong> {localRuntimeSmokeResult()!.safe_executable_file_name}</p>
                ) : null}
                <p><strong>Normalized prompt:</strong> {localRuntimeSmokeResult()!.normalized_prompt}</p>
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
                  <p>No smoke probe blockers.</p>
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
                  <p>No smoke probe warnings.</p>
                )}
              </>
            ) : (
              <p>No smoke inference preview loaded yet.</p>
            )
          ) : (
            <p>No smoke inference preview loaded yet.</p>
          )}
        </div>
      </section>

      <section class="card">
        <h2>Final answer inspector</h2>
        <p class="muted">
          Read-only display for an already-built FinalAnswer contract.
        </p>
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
        <div class="artifact-overview">
          <h3>Evidence packs</h3>
          <p class="muted">Read-only evidence-pack metadata for the selected retrieval or answer-artifact source.</p>
          {selectedEvidencePackSourceId() ? (
            <>
              <div class="hero-actions">
                <button onClick={() => loadEvidencePacks()} disabled={evidencePacksLoading()}>
                  {evidencePacksLoading() ? "Loading..." : "Load evidence packs"}
                </button>
              </div>
              {evidencePacksError() && evidencePacksSourceId() === selectedEvidencePackSourceId() && (
                <p class="error">{evidencePacksError()}</p>
              )}
              {evidencePacksSourceId() === selectedEvidencePackSourceId() ? (
                evidencePacks() ? (
                  <>
                    <div class="contract-meta">
                      <div><span>Source ID</span><strong>{selectedEvidencePackSourceId()}</strong></div>
                      <div><span>Packs</span><strong>{evidencePacks()!.length}</strong></div>
                    </div>
                    {evidencePacks()!.length > 0 ? (
                      <ul class="final-answer-list-items">
                        {evidencePacks()!.map((item) => (
                          <li>
                            <div class="final-answer-list-item">
                              <span>{item.evidence_pack_id}</span>
                              <small>
                                version={item.version_id} | created={item.created_at} | items={item.item_count} | results={item.result_count} | warnings={item.warning_count}
                              </small>
                              <small>
                                query={item.query} | retrieval_index_version={item.retrieval_index_version} | pack_version={item.evidence_pack_version}
                              </small>
                            </div>
                          </li>
                        ))}
                      </ul>
                    ) : (
                      <p>No evidence packs listed yet for this source.</p>
                    )}
                  </>
                ) : evidencePacksLoading() ? (
                  <p>Loading evidence packs...</p>
                ) : evidencePacksError() ? null : (
                  <p>No evidence packs loaded yet for this source.</p>
                )
              ) : (
                <p>No evidence packs loaded yet for this source.</p>
              )}
            </>
          ) : (
            <p>Select a retrieval or answer-artifact source to load evidence packs.</p>
          )}
        </div>
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
    </main>
  );
}
