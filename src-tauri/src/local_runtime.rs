use crate::errors::{AegisError, AegisResult};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::fs;
use std::process::{Command, Stdio};
use std::path::{Component, Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LocalModelRuntimeKind {
    LlamaCpp,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalModelRuntimeConfig {
    pub runtime_kind: LocalModelRuntimeKind,
    pub model_path: Option<String>,
    pub executable_path: Option<String>,
    pub context_window: Option<u32>,
    pub gpu_layers: Option<i32>,
    pub temperature: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LocalRuntimeAdapterKind {
    LlamaCpp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LocalRuntimeAdapterContractStatus {
    Blocked,
    NeedsReview,
    ContractReadyLater,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalRuntimeAdapterContractBlocker {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalRuntimeAdapterContractWarning {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalRuntimeAdapterContractPreviewRequest {
    pub adapter_kind: LocalRuntimeAdapterKind,
    pub executable_path: Option<String>,
    pub model_path: Option<String>,
    pub model_family: Option<String>,
    pub model_format: Option<String>,
    pub context_window_tokens: Option<u32>,
    pub gpu_layers: Option<i32>,
    pub threads: Option<u32>,
    pub batch_size: Option<u32>,
    pub chat_template: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalRuntimeAdapterContractPreview {
    pub status: LocalRuntimeAdapterContractStatus,
    pub adapter_kind: LocalRuntimeAdapterKind,
    pub normalized_model_family: Option<String>,
    pub normalized_model_format: String,
    pub executable_path_present: bool,
    pub model_path_present: bool,
    pub context_window_tokens: Option<u32>,
    pub gpu_layers: Option<i32>,
    pub threads: Option<u32>,
    pub batch_size: Option<u32>,
    pub chat_template_present: bool,
    pub required_inputs: Vec<String>,
    pub missing_inputs: Vec<String>,
    pub contract_reasons: Vec<String>,
    pub blockers: Vec<LocalRuntimeAdapterContractBlocker>,
    pub warnings: Vec<LocalRuntimeAdapterContractWarning>,
    pub next_required_actions: Vec<String>,
    pub summary: String,
    pub preview_only: bool,
    pub no_process_spawn: bool,
    pub no_model_load: bool,
    pub no_llm_call: bool,
    pub no_runtime_execution: bool,
    pub no_persistence: bool,
    pub no_artifact_write: bool,
    pub no_registry_status_change: bool,
    pub no_audit_write: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LocalRuntimeValidationStatus {
    Blocked,
    NeedsReview,
    ValidationReadyLater,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalRuntimeValidationPreviewRequest {
    pub adapter_contract_request: LocalRuntimeAdapterContractPreviewRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalRuntimeValidationPreview {
    pub status: LocalRuntimeValidationStatus,
    pub adapter_contract_status: LocalRuntimeAdapterContractStatus,
    pub adapter_kind: LocalRuntimeAdapterKind,
    pub normalized_model_family: Option<String>,
    pub normalized_model_format: String,
    pub executable_path_present: bool,
    pub model_path_present: bool,
    pub executable_exists: bool,
    pub model_exists: bool,
    pub executable_is_file: bool,
    pub model_is_file: bool,
    pub model_extension_valid: bool,
    pub safe_executable_file_name: Option<String>,
    pub safe_model_file_name: Option<String>,
    pub context_window_tokens: Option<u32>,
    pub gpu_layers: Option<i32>,
    pub threads: Option<u32>,
    pub batch_size: Option<u32>,
    pub chat_template_present: bool,
    pub missing_inputs: Vec<String>,
    pub validation_reasons: Vec<String>,
    pub blockers: Vec<LocalRuntimeAdapterContractBlocker>,
    pub warnings: Vec<LocalRuntimeAdapterContractWarning>,
    pub next_required_actions: Vec<String>,
    pub summary: String,
    pub preview_only: bool,
    pub no_process_spawn: bool,
    pub no_binary_probe: bool,
    pub no_model_load: bool,
    pub no_llm_call: bool,
    pub no_runtime_execution: bool,
    pub no_persistence: bool,
    pub no_artifact_write: bool,
    pub no_registry_status_change: bool,
    pub no_audit_write: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LocalRuntimeProbeReadinessStatus {
    Blocked,
    NeedsReview,
    ProbeReadyLater,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalRuntimeProbeReadinessPreviewRequest {
    pub validation_preview_request: LocalRuntimeValidationPreviewRequest,
    pub probe_consent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalRuntimeProbeReadinessPreview {
    pub status: LocalRuntimeProbeReadinessStatus,
    pub validation_status: LocalRuntimeValidationStatus,
    pub adapter_contract_status: LocalRuntimeAdapterContractStatus,
    pub adapter_kind: LocalRuntimeAdapterKind,
    pub normalized_model_family: Option<String>,
    pub normalized_model_format: String,
    pub executable_path_present: bool,
    pub model_path_present: bool,
    pub executable_exists: bool,
    pub model_exists: bool,
    pub executable_is_file: bool,
    pub model_is_file: bool,
    pub model_extension_valid: bool,
    pub safe_executable_file_name: Option<String>,
    pub safe_model_file_name: Option<String>,
    pub probe_consent: bool,
    pub required_inputs: Vec<String>,
    pub missing_inputs: Vec<String>,
    pub readiness_reasons: Vec<String>,
    pub blockers: Vec<LocalRuntimeAdapterContractBlocker>,
    pub warnings: Vec<LocalRuntimeAdapterContractWarning>,
    pub next_required_actions: Vec<String>,
    pub summary: String,
    pub preview_only: bool,
    pub no_process_spawn: bool,
    pub no_binary_probe: bool,
    pub no_model_load: bool,
    pub no_llm_call: bool,
    pub no_runtime_execution: bool,
    pub no_persistence: bool,
    pub no_artifact_write: bool,
    pub no_registry_status_change: bool,
    pub no_audit_write: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LocalRuntimeVersionProbeStatus {
    Blocked,
    ProbeSucceeded,
    ProbeFailed,
    TimedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalRuntimeVersionProbePreviewRequest {
    pub probe_readiness_preview_request: LocalRuntimeProbeReadinessPreviewRequest,
    pub allow_probe_execution: bool,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalRuntimeVersionProbePreview {
    pub status: LocalRuntimeVersionProbeStatus,
    pub probe_readiness_status: LocalRuntimeProbeReadinessStatus,
    pub validation_status: LocalRuntimeValidationStatus,
    pub adapter_contract_status: LocalRuntimeAdapterContractStatus,
    pub adapter_kind: LocalRuntimeAdapterKind,
    pub normalized_model_family: Option<String>,
    pub normalized_model_format: String,
    pub safe_executable_file_name: Option<String>,
    pub safe_model_file_name: Option<String>,
    pub probe_consent: bool,
    pub allow_probe_execution: bool,
    pub execution_attempted: bool,
    pub probe_argument: String,
    pub timeout_ms: u64,
    pub duration_ms: u64,
    pub exit_code: Option<i32>,
    pub stdout_preview: String,
    pub stderr_preview: String,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
    pub blockers: Vec<LocalRuntimeProbeWarning>,
    pub warnings: Vec<LocalRuntimeProbeWarning>,
    pub next_required_actions: Vec<String>,
    pub summary: String,
    pub preview_only: bool,
    pub no_model_load: bool,
    pub no_model_path_argument: bool,
    pub no_llm_call: bool,
    pub no_runtime_inference: bool,
    pub no_persistence: bool,
    pub no_artifact_write: bool,
    pub no_registry_status_change: bool,
    pub no_audit_write: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LocalRuntimeCapabilityStatus {
    Blocked,
    NeedsReview,
    CapabilityReadyLater,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalRuntimeCapabilityPreviewRequest {
    pub version_probe_preview_request: LocalRuntimeVersionProbePreviewRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalRuntimeCapabilityPreview {
    pub status: LocalRuntimeCapabilityStatus,
    pub version_probe_status: LocalRuntimeVersionProbeStatus,
    pub probe_readiness_status: LocalRuntimeProbeReadinessStatus,
    pub validation_status: LocalRuntimeValidationStatus,
    pub adapter_contract_status: LocalRuntimeAdapterContractStatus,
    pub adapter_kind: LocalRuntimeAdapterKind,
    pub normalized_model_family: Option<String>,
    pub normalized_model_format: String,
    pub safe_executable_file_name: Option<String>,
    pub safe_model_file_name: Option<String>,
    pub probe_consent: bool,
    pub allow_probe_execution: bool,
    pub version_probe_execution_attempted: bool,
    pub version_probe_exit_code: Option<i32>,
    pub version_probe_timed_out: bool,
    pub version_probe_stdout_preview: String,
    pub version_probe_stderr_preview: String,
    pub inferred_runtime_available: bool,
    pub inferred_version_text: Option<String>,
    pub capability_reasons: Vec<String>,
    pub blockers: Vec<LocalRuntimeProbeWarning>,
    pub warnings: Vec<LocalRuntimeProbeWarning>,
    pub next_required_actions: Vec<String>,
    pub summary: String,
    pub preview_only: bool,
    pub no_new_process_spawn: bool,
    pub no_binary_probe_beyond_wrapped_version_probe: bool,
    pub no_model_path_argument: bool,
    pub no_model_file_read: bool,
    pub no_model_load: bool,
    pub no_runtime_inference: bool,
    pub no_smoke_inference: bool,
    pub no_llm_call: bool,
    pub no_persistence: bool,
    pub no_artifact_write: bool,
    pub no_registry_status_change: bool,
    pub no_audit_write: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LocalRuntimeSmokeReadinessStatus {
    Blocked,
    NeedsReview,
    SmokeReadyLater,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalRuntimeSmokeReadinessPreviewRequest {
    pub capability_preview_request: LocalRuntimeCapabilityPreviewRequest,
    pub smoke_consent: bool,
    pub diagnostic_prompt: Option<String>,
    pub max_output_tokens: Option<u32>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalRuntimeSmokeReadinessPreview {
    pub status: LocalRuntimeSmokeReadinessStatus,
    pub capability_status: LocalRuntimeCapabilityStatus,
    pub version_probe_status: LocalRuntimeVersionProbeStatus,
    pub probe_readiness_status: LocalRuntimeProbeReadinessStatus,
    pub validation_status: LocalRuntimeValidationStatus,
    pub adapter_contract_status: LocalRuntimeAdapterContractStatus,
    pub adapter_kind: LocalRuntimeAdapterKind,
    pub normalized_model_family: Option<String>,
    pub normalized_model_format: String,
    pub safe_executable_file_name: Option<String>,
    pub safe_model_file_name: Option<String>,
    pub probe_consent: bool,
    pub allow_probe_execution: bool,
    pub smoke_consent: bool,
    pub normalized_diagnostic_prompt: String,
    pub diagnostic_prompt_char_count: usize,
    pub max_output_tokens: u32,
    pub timeout_ms: u64,
    pub required_inputs: Vec<String>,
    pub missing_inputs: Vec<String>,
    pub readiness_reasons: Vec<String>,
    pub blockers: Vec<LocalRuntimeProbeWarning>,
    pub warnings: Vec<LocalRuntimeProbeWarning>,
    pub next_required_actions: Vec<String>,
    pub summary: String,
    pub preview_only: bool,
    pub no_new_process_spawn: bool,
    pub no_smoke_inference_execution: bool,
    pub no_model_path_argument: bool,
    pub no_model_file_read: bool,
    pub no_model_load: bool,
    pub no_llm_call: bool,
    pub no_persistence: bool,
    pub no_artifact_write: bool,
    pub no_registry_status_change: bool,
    pub no_audit_write: bool,
    pub diagnostic_only: bool,
    pub not_scholar_chat_answer: bool,
    pub no_answer_generated: bool,
    pub no_grounding_applied: bool,
    pub no_evidence_pack_used: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LocalRuntimeSmokeExecutionPlanStatus {
    Blocked,
    NeedsReview,
    PlanReadyLater,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalRuntimeSmokeExecutionPlanPreviewRequest {
    pub smoke_readiness_preview_request: LocalRuntimeSmokeReadinessPreviewRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalRuntimeSmokeExecutionPlanPreview {
    pub status: LocalRuntimeSmokeExecutionPlanStatus,
    pub smoke_readiness_status: LocalRuntimeSmokeReadinessStatus,
    pub capability_status: LocalRuntimeCapabilityStatus,
    pub version_probe_status: LocalRuntimeVersionProbeStatus,
    pub probe_readiness_status: LocalRuntimeProbeReadinessStatus,
    pub validation_status: LocalRuntimeValidationStatus,
    pub adapter_contract_status: LocalRuntimeAdapterContractStatus,
    pub adapter_kind: LocalRuntimeAdapterKind,
    pub normalized_model_family: Option<String>,
    pub normalized_model_format: String,
    pub safe_executable_file_name: Option<String>,
    pub safe_model_file_name: Option<String>,
    pub probe_consent: bool,
    pub allow_probe_execution: bool,
    pub smoke_consent: bool,
    pub normalized_diagnostic_prompt: String,
    pub diagnostic_prompt_char_count: usize,
    pub max_output_tokens: u32,
    pub timeout_ms: u64,
    pub planned_operation: String,
    pub planned_inputs: Vec<String>,
    pub planned_safe_arguments: Vec<String>,
    pub planned_outputs: Vec<String>,
    pub required_inputs: Vec<String>,
    pub missing_inputs: Vec<String>,
    pub plan_reasons: Vec<String>,
    pub blockers: Vec<LocalRuntimeProbeWarning>,
    pub warnings: Vec<LocalRuntimeProbeWarning>,
    pub next_required_actions: Vec<String>,
    pub summary: String,
    pub preview_only: bool,
    pub no_process_spawn: bool,
    pub no_smoke_inference_execution: bool,
    pub no_model_file_read: bool,
    pub no_model_load: bool,
    pub no_llm_call: bool,
    pub no_persistence: bool,
    pub no_artifact_write: bool,
    pub no_registry_status_change: bool,
    pub no_audit_write: bool,
    pub diagnostic_only: bool,
    pub not_scholar_chat_answer: bool,
    pub no_answer_generated: bool,
    pub no_grounding_applied: bool,
    pub no_evidence_pack_used: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LocalModelRuntimeHealthStatus {
    NotConfigured,
    ConfigPresent,
    ModelMissing,
    ExecutableMissing,
    ReadyToTestLater,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LocalModelRuntimePathState {
    NotConfigured,
    Missing,
    Exists,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalModelRuntimeHealthWarning {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalModelRuntimeHealthPreview {
    pub status: LocalModelRuntimeHealthStatus,
    pub runtime_kind: LocalModelRuntimeKind,
    pub model_state: LocalModelRuntimePathState,
    pub executable_state: LocalModelRuntimePathState,
    pub model_extension_valid: bool,
    pub model_file_name: Option<String>,
    pub context_window: Option<u32>,
    pub gpu_layers: Option<i32>,
    pub temperature: Option<f64>,
    pub warnings: Vec<LocalModelRuntimeHealthWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LocalRuntimeInvocationPlanStatus {
    NotConfigured,
    Blocked,
    ReadyToInvokeLater,
    PreviewOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalRuntimeInvocationPlanRequest {
    pub runtime_config: LocalModelRuntimeConfig,
    pub prompt_text: Option<String>,
    pub estimated_input_char_count: Option<u32>,
    pub max_output_tokens: Option<u32>,
    pub stop_sequences: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalRuntimeInvocationBlocker {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalRuntimeInvocationPlan {
    pub runtime_health_status: LocalModelRuntimeHealthStatus,
    pub prompt_char_count: u32,
    pub estimated_context_char_count: u32,
    pub max_output_tokens: Option<u32>,
    pub safe_model_file_name: Option<String>,
    pub safe_executable_file_name: Option<String>,
    pub invocation_steps: Vec<String>,
    pub safe_argument_preview: Vec<String>,
    pub blockers: Vec<LocalRuntimeInvocationBlocker>,
    pub warnings: Vec<LocalModelRuntimeHealthWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalRuntimeInvocationPlanPreview {
    pub status: LocalRuntimeInvocationPlanStatus,
    pub runtime_kind: LocalModelRuntimeKind,
    pub plan: LocalRuntimeInvocationPlan,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LocalRuntimeProbeStatus {
    Blocked,
    Completed,
    TimedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalRuntimeProbeWarning {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalRuntimeProbeRequest {
    pub executable_path: Option<String>,
    pub allow_execution: bool,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalRuntimeProbeResult {
    pub status: LocalRuntimeProbeStatus,
    pub allow_execution: bool,
    pub execution_attempted: bool,
    pub probe_argument: String,
    pub timeout_ms: u64,
    pub duration_ms: u64,
    pub safe_executable_file_name: Option<String>,
    pub exit_code: Option<i32>,
    pub stdout_preview: String,
    pub stderr_preview: String,
    pub blockers: Vec<LocalRuntimeProbeWarning>,
    pub warnings: Vec<LocalRuntimeProbeWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LocalRuntimeSmokeInferenceStatus {
    Blocked,
    NotConfigured,
    ModelMissing,
    ExecutableMissing,
    InferenceSucceeded,
    InferenceFailed,
    TimedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LocalRuntimeSmokeInferenceOutputClassification {
    RuntimeDiagnostic,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalRuntimeSmokeInferenceWarning {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalRuntimeSmokeInferenceBlocker {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalRuntimeSmokeInferenceRequest {
    pub runtime_config: LocalModelRuntimeConfig,
    pub allow_execution: bool,
    pub prompt: Option<String>,
    pub timeout_ms: Option<u64>,
    pub max_output_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalRuntimeSmokeInferenceResult {
    pub status: LocalRuntimeSmokeInferenceStatus,
    pub allow_execution: bool,
    pub execution_attempted: bool,
    pub runtime_kind: LocalModelRuntimeKind,
    pub safe_model_file_name: Option<String>,
    pub safe_executable_file_name: Option<String>,
    pub normalized_prompt: String,
    pub prompt_char_count: u32,
    pub max_output_tokens: u32,
    pub timeout_ms: u64,
    pub exit_code: Option<i32>,
    pub stdout_preview: String,
    pub stderr_preview: String,
    pub duration_ms: u64,
    pub warnings: Vec<LocalRuntimeSmokeInferenceWarning>,
    pub blockers: Vec<LocalRuntimeSmokeInferenceBlocker>,
    pub diagnostic_only: bool,
    pub no_answer_generated: bool,
    pub no_grounding_applied: bool,
    pub no_evidence_pack_used: bool,
    pub not_scholar_chat_answer: bool,
    pub output_classification: LocalRuntimeSmokeInferenceOutputClassification,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LocalRuntimeSmokeDiagnosticStatus {
    Blocked,
    SmokeSucceeded,
    SmokeFailed,
    TimedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalRuntimeSmokeDiagnosticRequest {
    pub smoke_execution_plan_preview_request: LocalRuntimeSmokeExecutionPlanPreviewRequest,
    pub allow_smoke_execution: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LocalRuntimeSmokeDiagnosticPreview {
    pub status: LocalRuntimeSmokeDiagnosticStatus,
    pub smoke_execution_plan_status: LocalRuntimeSmokeExecutionPlanStatus,
    pub smoke_readiness_status: LocalRuntimeSmokeReadinessStatus,
    pub capability_status: LocalRuntimeCapabilityStatus,
    pub version_probe_status: LocalRuntimeVersionProbeStatus,
    pub probe_readiness_status: LocalRuntimeProbeReadinessStatus,
    pub validation_status: LocalRuntimeValidationStatus,
    pub adapter_contract_status: LocalRuntimeAdapterContractStatus,
    pub adapter_kind: LocalRuntimeAdapterKind,
    pub normalized_model_family: Option<String>,
    pub normalized_model_format: String,
    pub safe_executable_file_name: Option<String>,
    pub safe_model_file_name: Option<String>,
    pub probe_consent: bool,
    pub allow_probe_execution: bool,
    pub smoke_consent: bool,
    pub allow_smoke_execution: bool,
    pub execution_attempted: bool,
    pub normalized_diagnostic_prompt: String,
    pub diagnostic_prompt_char_count: usize,
    pub max_output_tokens: u32,
    pub timeout_ms: u64,
    pub duration_ms: u64,
    pub exit_code: Option<i32>,
    pub stdout_preview: String,
    pub stderr_preview: String,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
    pub blockers: Vec<LocalRuntimeSmokeInferenceBlocker>,
    pub warnings: Vec<LocalRuntimeSmokeInferenceWarning>,
    pub next_required_actions: Vec<String>,
    pub summary: String,
    pub diagnostic_only: bool,
    pub not_scholar_chat_answer: bool,
    pub no_answer_generated: bool,
    pub no_grounding_applied: bool,
    pub no_evidence_pack_used: bool,
    pub no_persistence: bool,
    pub no_artifact_write: bool,
    pub no_registry_status_change: bool,
    pub no_audit_write: bool,
}

pub fn preview_local_model_runtime_health(
    root: impl Into<PathBuf>,
    config: LocalModelRuntimeConfig,
) -> AegisResult<LocalModelRuntimeHealthPreview> {
    let root = root.into();
    let model_path = normalize_optional_path(config.model_path)?;
    let executable_path = normalize_optional_path(config.executable_path)?;
    let context_present =
        config.context_window.is_some() || config.gpu_layers.is_some() || config.temperature.is_some();

    let mut warnings = Vec::new();
    push_warning(&mut warnings, "This is preview only; no process was started.");
    push_warning(&mut warnings, "Configuration is not persisted.");

    let model_lookup = inspect_configured_path(&root, model_path.as_deref())?;
    let executable_lookup = inspect_configured_path(&root, executable_path.as_deref())?;

    let model_state = model_lookup.state;
    let executable_state = executable_lookup.state;
    let model_extension_valid = model_lookup.extension_valid;
    let model_file_name = model_lookup.file_name;

    if matches!(config.runtime_kind, LocalModelRuntimeKind::None)
        && model_state == LocalModelRuntimePathState::NotConfigured
        && executable_state == LocalModelRuntimePathState::NotConfigured
        && !context_present
    {
        push_warning(&mut warnings, "No local runtime is configured yet.");
    } else if matches!(config.runtime_kind, LocalModelRuntimeKind::None) {
        push_warning(&mut warnings, "Runtime kind is set to none; choose llama_cpp for a local runtime preview.");
    }

    if matches!(model_state, LocalModelRuntimePathState::Missing) {
        push_warning(&mut warnings, "Configured model file is missing.");
    }
    if matches!(executable_state, LocalModelRuntimePathState::Missing) {
        push_warning(&mut warnings, "Configured executable file is missing.");
    }
    if model_state == LocalModelRuntimePathState::Exists && !model_extension_valid {
        push_warning(&mut warnings, "Configured model file does not use a .gguf extension.");
    }
    if model_state == LocalModelRuntimePathState::Exists
        && matches!(config.runtime_kind, LocalModelRuntimeKind::LlamaCpp)
        && model_extension_valid
        && (executable_state == LocalModelRuntimePathState::NotConfigured || executable_state == LocalModelRuntimePathState::Exists)
    {
        push_warning(&mut warnings, "Local runtime looks ready for a later test run.");
    }

    let status = if model_state == LocalModelRuntimePathState::Missing {
        LocalModelRuntimeHealthStatus::ModelMissing
    } else if executable_state == LocalModelRuntimePathState::Missing {
        LocalModelRuntimeHealthStatus::ExecutableMissing
    } else if matches!(config.runtime_kind, LocalModelRuntimeKind::LlamaCpp)
        && model_state == LocalModelRuntimePathState::Exists
        && model_extension_valid
        && (executable_state == LocalModelRuntimePathState::NotConfigured || executable_state == LocalModelRuntimePathState::Exists)
    {
        LocalModelRuntimeHealthStatus::ReadyToTestLater
    } else if model_state == LocalModelRuntimePathState::NotConfigured
        && executable_state == LocalModelRuntimePathState::NotConfigured
        && !context_present
        && matches!(config.runtime_kind, LocalModelRuntimeKind::None)
    {
        LocalModelRuntimeHealthStatus::NotConfigured
    } else {
        LocalModelRuntimeHealthStatus::ConfigPresent
    };

    Ok(LocalModelRuntimeHealthPreview {
        status,
        runtime_kind: config.runtime_kind,
        model_state,
        executable_state,
        model_extension_valid,
        model_file_name,
        context_window: config.context_window,
        gpu_layers: config.gpu_layers,
        temperature: config.temperature,
        warnings,
    })
}

pub fn preview_local_runtime_invocation_plan(
    root: impl Into<PathBuf>,
    request: LocalRuntimeInvocationPlanRequest,
) -> AegisResult<LocalRuntimeInvocationPlanPreview> {
    let root = root.into();
    let runtime_config = request.runtime_config;
    let normalized_model_path = normalize_optional_path(runtime_config.model_path.clone())?;
    let normalized_executable_path = normalize_optional_path(runtime_config.executable_path.clone())?;
    let prompt_text = normalize_optional_text(request.prompt_text);
    let prompt_char_count = prompt_text
        .as_deref()
        .map(|value| value.chars().count() as u32)
        .unwrap_or(0);
    let estimated_context_char_count = request
        .estimated_input_char_count
        .unwrap_or(prompt_char_count);
    let max_output_tokens = request.max_output_tokens;
    let stop_sequence_count = normalize_optional_text_list(request.stop_sequences).len() as u32;

    let health = preview_local_model_runtime_health(
        root,
        LocalModelRuntimeConfig {
            runtime_kind: runtime_config.runtime_kind.clone(),
            model_path: normalized_model_path.clone(),
            executable_path: normalized_executable_path.clone(),
            context_window: runtime_config.context_window,
            gpu_layers: runtime_config.gpu_layers,
            temperature: runtime_config.temperature,
        },
    )?;

    let safe_model_file_name = safe_file_name_from_path(normalized_model_path.as_deref());
    let safe_executable_file_name = safe_file_name_from_path(normalized_executable_path.as_deref());

    let mut warnings = health.warnings.clone();
    let mut blockers = Vec::new();

    match health.status {
        LocalModelRuntimeHealthStatus::NotConfigured => {
            push_blocker(
                &mut blockers,
                "runtime_not_configured",
                "No local runtime is configured yet.",
            );
        }
        LocalModelRuntimeHealthStatus::ModelMissing => {
            push_blocker(
                &mut blockers,
                "model_missing",
                "Configured model file is missing.",
            );
        }
        LocalModelRuntimeHealthStatus::ExecutableMissing => {
            push_blocker(
                &mut blockers,
                "executable_missing",
                "Configured executable file is missing.",
            );
        }
        LocalModelRuntimeHealthStatus::ConfigPresent | LocalModelRuntimeHealthStatus::ReadyToTestLater => {}
    }

    if health.model_state == LocalModelRuntimePathState::Exists && !health.model_extension_valid {
        push_blocker(
            &mut blockers,
            "model_extension_invalid",
            "Configured model file does not use a .gguf extension.",
        );
    }

    if prompt_char_count == 0 {
        push_warning(
            &mut warnings,
            "No prompt text was provided; preview is using runtime configuration only.",
        );
    }

    if max_output_tokens.is_none() {
        push_warning(
            &mut warnings,
            "No max output tokens were provided yet.",
        );
    }

    if stop_sequence_count > 0 {
        push_warning(
            &mut warnings,
            "Stop sequences are captured as a count only in this preview.",
        );
    }

    let status = if matches!(health.status, LocalModelRuntimeHealthStatus::NotConfigured) {
        LocalRuntimeInvocationPlanStatus::NotConfigured
    } else if !blockers.is_empty() {
        LocalRuntimeInvocationPlanStatus::Blocked
    } else if matches!(health.status, LocalModelRuntimeHealthStatus::ReadyToTestLater)
        && prompt_char_count > 0
        && max_output_tokens.unwrap_or(0) > 0
    {
        LocalRuntimeInvocationPlanStatus::ReadyToInvokeLater
    } else {
        LocalRuntimeInvocationPlanStatus::PreviewOnly
    };

    let mut invocation_steps = vec![
        "Validate runtime readiness and prompt inputs.".to_string(),
        "Prepare redacted invocation arguments.".to_string(),
        "Stop before any process execution.".to_string(),
    ];
    if prompt_char_count > 0 {
        invocation_steps.insert(
            1,
            format!("Use the trimmed prompt text ({prompt_char_count} characters)."),
        );
    }
    invocation_steps.insert(
        1,
        format!("Estimate prompt/context size as {estimated_context_char_count} characters."),
    );

    let mut safe_argument_preview = vec![
        format!("--runtime-kind={}", format_runtime_kind(&runtime_config.runtime_kind)),
        format!("--prompt-chars={prompt_char_count}"),
        format!("--estimated-context-chars={estimated_context_char_count}"),
        format!("--stop-sequences={stop_sequence_count}"),
    ];
    if let Some(max_output_tokens) = max_output_tokens {
        safe_argument_preview.push(format!("--max-output-tokens={max_output_tokens}"));
    } else {
        safe_argument_preview.push("--max-output-tokens=<missing>".to_string());
    }
    if let Some(file_name) = safe_model_file_name.as_deref() {
        safe_argument_preview.push(format!("--model-file-name={file_name}"));
    }
    if let Some(file_name) = safe_executable_file_name.as_deref() {
        safe_argument_preview.push(format!("--executable-file-name={file_name}"));
    }
    if let Some(context_window) = runtime_config.context_window {
        safe_argument_preview.push(format!("--ctx-size={context_window}"));
    }
    if let Some(gpu_layers) = runtime_config.gpu_layers {
        safe_argument_preview.push(format!("--gpu-layers={gpu_layers}"));
    }
    if let Some(temperature) = runtime_config.temperature {
        safe_argument_preview.push(format!("--temperature={temperature}"));
    }

    Ok(LocalRuntimeInvocationPlanPreview {
        status,
        runtime_kind: runtime_config.runtime_kind,
        plan: LocalRuntimeInvocationPlan {
            runtime_health_status: health.status,
            prompt_char_count,
            estimated_context_char_count,
            max_output_tokens,
            safe_model_file_name,
            safe_executable_file_name,
            invocation_steps,
            safe_argument_preview,
            blockers,
            warnings,
        },
    })
}

pub fn preview_llama_runtime_adapter_contract(
    root: impl Into<PathBuf>,
    request: LocalRuntimeAdapterContractPreviewRequest,
) -> AegisResult<LocalRuntimeAdapterContractPreview> {
    let _root = root.into();
    let executable_path = normalize_optional_path(request.executable_path)?;
    let model_path = normalize_optional_path(request.model_path)?;
    let normalized_model_family = normalize_llama_runtime_adapter_model_family(request.model_family);
    let (normalized_model_format, model_format_supported) =
        normalize_llama_runtime_adapter_model_format(request.model_format);
    let chat_template_present = normalize_optional_text(request.chat_template).is_some();

    let mut blockers = Vec::new();
    let mut warnings = Vec::new();

    let executable_path_present = executable_path.is_some();
    if !executable_path_present {
        push_adapter_contract_blocker(
            &mut blockers,
            "executable_path_missing",
            "An executable_path is required for the llama.cpp adapter contract.",
        );
    }

    let model_path_present = model_path.is_some();
    if !model_path_present {
        push_adapter_contract_blocker(
            &mut blockers,
            "model_path_missing",
            "A model_path is required for the llama.cpp adapter contract.",
        );
    }

    if let Some(context_window_tokens) = request.context_window_tokens {
        if context_window_tokens < LOCAL_RUNTIME_ADAPTER_MIN_CONTEXT_WINDOW_TOKENS
            || context_window_tokens > LOCAL_RUNTIME_ADAPTER_MAX_CONTEXT_WINDOW_TOKENS
        {
            push_adapter_contract_blocker(
                &mut blockers,
                "context_window_tokens_out_of_range",
                "context_window_tokens must be a positive value within the safe preview range.",
            );
        }
    }

    if let Some(gpu_layers) = request.gpu_layers {
        if gpu_layers < -1 {
            push_adapter_contract_blocker(
                &mut blockers,
                "gpu_layers_out_of_range",
                "gpu_layers must be -1 or a non-negative value.",
            );
        }
    }

    if let Some(threads) = request.threads {
        if threads == 0 {
            push_adapter_contract_blocker(
                &mut blockers,
                "threads_out_of_range",
                "threads must be a positive value.",
            );
        }
    }

    if let Some(batch_size) = request.batch_size {
        if batch_size == 0 {
            push_adapter_contract_blocker(
                &mut blockers,
                "batch_size_out_of_range",
                "batch_size must be a positive value.",
            );
        }
    }

    if !model_format_supported {
        push_adapter_contract_blocker(
            &mut blockers,
            "model_format_unsupported",
            "Only gguf model_format is supported for the llama.cpp adapter preview.",
        );
    }

    if let Some(model_family) = normalized_model_family.as_deref() {
        if !is_known_llama_runtime_adapter_model_family(model_family) {
            push_adapter_contract_warning(
                &mut warnings,
                "model_family_needs_review",
                "The model family is not one of the previewed common families and needs review.",
            );
        }
    }

    if normalized_model_family.as_deref() == Some("gemma") {
        if !chat_template_present {
            push_adapter_contract_warning(
                &mut warnings,
                "chat_template_missing_for_gemma",
                "Gemma adapters usually need a chat template before a future runtime implementation.",
            );
        }
    }
    let status = if !blockers.is_empty() {
        LocalRuntimeAdapterContractStatus::Blocked
    } else if warnings.is_empty() {
        LocalRuntimeAdapterContractStatus::ContractReadyLater
    } else {
        LocalRuntimeAdapterContractStatus::NeedsReview
    };
    let contract_reasons = llama_runtime_adapter_contract_reasons(
        &status,
        &normalized_model_family,
        &normalized_model_format,
        chat_template_present,
        &blockers,
        &warnings,
    );
    let required_inputs = llama_runtime_adapter_contract_required_inputs();
    let missing_inputs = llama_runtime_adapter_contract_missing_inputs(
        executable_path_present,
        model_path_present,
        &blockers,
    );
    let next_required_actions = llama_runtime_adapter_contract_next_required_actions(
        &status,
        executable_path_present,
        model_path_present,
        &warnings,
    );
    let summary = llama_runtime_adapter_contract_summary(
        &status,
        executable_path_present,
        model_path_present,
        &normalized_model_family,
        &normalized_model_format,
        chat_template_present,
    );

    Ok(LocalRuntimeAdapterContractPreview {
        status,
        adapter_kind: request.adapter_kind,
        normalized_model_family,
        normalized_model_format,
        executable_path_present,
        model_path_present,
        context_window_tokens: request.context_window_tokens,
        gpu_layers: request.gpu_layers,
        threads: request.threads,
        batch_size: request.batch_size,
        chat_template_present,
        required_inputs,
        missing_inputs,
        contract_reasons,
        blockers,
        warnings,
        next_required_actions,
        summary,
        preview_only: true,
        no_process_spawn: true,
        no_model_load: true,
        no_llm_call: true,
        no_runtime_execution: true,
        no_persistence: true,
        no_artifact_write: true,
        no_registry_status_change: true,
        no_audit_write: true,
    })
}

pub fn preview_llama_runtime_validation(
    root: impl Into<PathBuf>,
    request: LocalRuntimeValidationPreviewRequest,
) -> AegisResult<LocalRuntimeValidationPreview> {
    let root = root.into();
    let adapter_contract_request = request.adapter_contract_request;
    let adapter_contract_preview =
        preview_llama_runtime_adapter_contract(root.clone(), adapter_contract_request.clone())?;
    let normalized_executable_path = normalize_optional_path(adapter_contract_request.executable_path)?;
    let normalized_model_path = normalize_optional_path(adapter_contract_request.model_path)?;
    let executable_file_name = safe_file_name_from_path(normalized_executable_path.as_deref());
    let model_file_name = safe_file_name_from_path(normalized_model_path.as_deref());
    let executable_path_present = adapter_contract_preview.executable_path_present;
    let model_path_present = adapter_contract_preview.model_path_present;
    let executable_metadata = inspect_runtime_validation_path(&root, normalized_executable_path.as_deref())?;
    let model_metadata = inspect_runtime_validation_path(&root, normalized_model_path.as_deref())?;
    let model_extension_valid = normalized_model_path
        .as_deref()
        .map(|path| has_gguf_extension(Path::new(path)))
        .unwrap_or(false);

    let mut blockers = adapter_contract_preview.blockers.clone();
    let mut warnings = adapter_contract_preview.warnings.clone();
    push_adapter_contract_warning(
        &mut warnings,
        "validation_boundary",
        "This is a preview-only llama.cpp runtime validation; no binary was probed, no process was started, no model was loaded, no runtime execution or LLM call occurred, and no settings or artifacts were persisted.",
    );

    if executable_path_present {
        if !executable_metadata.exists {
            push_adapter_contract_blocker(
                &mut blockers,
                "executable_missing",
                "The configured executable file is missing.",
            );
        } else if !executable_metadata.is_file {
            push_adapter_contract_blocker(
                &mut blockers,
                "executable_not_file",
                "The configured executable path points to a directory instead of a file.",
            );
        }
    }

    if model_path_present {
        if !model_metadata.exists {
            push_adapter_contract_blocker(
                &mut blockers,
                "model_missing",
                "The configured model file is missing.",
            );
        } else if !model_metadata.is_file {
            push_adapter_contract_blocker(
                &mut blockers,
                "model_not_file",
                "The configured model path points to a directory instead of a file.",
            );
        } else if !model_extension_valid {
            push_adapter_contract_blocker(
                &mut blockers,
                "model_extension_invalid",
                "The configured model file does not use a .gguf extension.",
            );
        }
    }

    let status = if matches!(adapter_contract_preview.status, LocalRuntimeAdapterContractStatus::Blocked)
        || !blockers.is_empty()
    {
        LocalRuntimeValidationStatus::Blocked
    } else if matches!(
        adapter_contract_preview.status,
        LocalRuntimeAdapterContractStatus::NeedsReview
    ) {
        LocalRuntimeValidationStatus::NeedsReview
    } else {
        LocalRuntimeValidationStatus::ValidationReadyLater
    };

    let missing_inputs = llama_runtime_validation_missing_inputs(
        executable_path_present,
        model_path_present,
        &executable_metadata,
        &model_metadata,
        model_extension_valid,
    );
    let validation_reasons = llama_runtime_validation_reasons(
        &status,
        &adapter_contract_preview,
        executable_path_present,
        model_path_present,
        &executable_metadata,
        &model_metadata,
        model_extension_valid,
        executable_file_name.as_deref(),
        model_file_name.as_deref(),
    );
    let next_required_actions = llama_runtime_validation_next_required_actions(
        &status,
        &adapter_contract_preview,
        executable_path_present,
        model_path_present,
        executable_metadata.exists,
        model_metadata.exists,
        executable_metadata.is_file,
        model_metadata.is_file,
        &executable_metadata,
        &model_metadata,
        model_extension_valid,
    );
    let summary = llama_runtime_validation_summary(
        &status,
        &adapter_contract_preview,
        executable_path_present,
        model_path_present,
        &executable_metadata,
        &model_metadata,
        model_extension_valid,
    );

    Ok(LocalRuntimeValidationPreview {
        status,
        adapter_contract_status: adapter_contract_preview.status,
        adapter_kind: adapter_contract_preview.adapter_kind,
        normalized_model_family: adapter_contract_preview.normalized_model_family,
        normalized_model_format: adapter_contract_preview.normalized_model_format,
        executable_path_present,
        model_path_present,
        executable_exists: executable_metadata.exists,
        model_exists: model_metadata.exists,
        executable_is_file: executable_metadata.is_file,
        model_is_file: model_metadata.is_file,
        model_extension_valid,
        safe_executable_file_name: executable_file_name,
        safe_model_file_name: model_file_name,
        context_window_tokens: adapter_contract_preview.context_window_tokens,
        gpu_layers: adapter_contract_preview.gpu_layers,
        threads: adapter_contract_preview.threads,
        batch_size: adapter_contract_preview.batch_size,
        chat_template_present: adapter_contract_preview.chat_template_present,
        missing_inputs,
        validation_reasons,
        blockers,
        warnings,
        next_required_actions,
        summary,
        preview_only: true,
        no_process_spawn: true,
        no_binary_probe: true,
        no_model_load: true,
        no_llm_call: true,
        no_runtime_execution: true,
        no_persistence: true,
        no_artifact_write: true,
        no_registry_status_change: true,
        no_audit_write: true,
    })
}

pub fn preview_llama_runtime_probe_readiness(
    root: impl Into<PathBuf>,
    request: LocalRuntimeProbeReadinessPreviewRequest,
) -> AegisResult<LocalRuntimeProbeReadinessPreview> {
    let root = root.into();
    let validation_preview =
        preview_llama_runtime_validation(root, request.validation_preview_request)?;
    let status = match validation_preview.status {
        LocalRuntimeValidationStatus::Blocked => LocalRuntimeProbeReadinessStatus::Blocked,
        LocalRuntimeValidationStatus::NeedsReview => LocalRuntimeProbeReadinessStatus::NeedsReview,
        LocalRuntimeValidationStatus::ValidationReadyLater => {
            if request.probe_consent {
                LocalRuntimeProbeReadinessStatus::ProbeReadyLater
            } else {
                LocalRuntimeProbeReadinessStatus::Blocked
            }
        }
    };
    let required_inputs = llama_runtime_probe_readiness_required_inputs();
    let missing_inputs = llama_runtime_probe_readiness_missing_inputs(
        &validation_preview.status,
        request.probe_consent,
    );
    let readiness_reasons = llama_runtime_probe_readiness_reasons(
        &validation_preview,
        request.probe_consent,
        &status,
    );
    let mut blockers = validation_preview.blockers.clone();
    let mut warnings = validation_preview.warnings.clone();

    push_adapter_contract_warning(
        &mut warnings,
        "probe_readiness_boundary",
        "This is a preview-only llama.cpp probe readiness; no binary was probed, no process was started, no model was loaded, no runtime execution or LLM call occurred, and no settings or artifacts were persisted.",
    );
    if !matches!(
        validation_preview.status,
        LocalRuntimeValidationStatus::ValidationReadyLater
    ) {
        push_adapter_contract_blocker(
            &mut blockers,
            "runtime_validation_not_ready_later",
            "The llama.cpp runtime validation preview is not ready later yet.",
        );
    } else if !request.probe_consent {
        push_adapter_contract_blocker(
            &mut blockers,
            "probe_consent_missing",
            "Probe consent was not given.",
        );
    }

    let next_required_actions = llama_runtime_probe_readiness_next_required_actions(
        &status,
        &validation_preview,
        request.probe_consent,
    );
    let summary = llama_runtime_probe_readiness_summary(
        &status,
        &validation_preview,
        request.probe_consent,
    );

    Ok(LocalRuntimeProbeReadinessPreview {
        status,
        validation_status: validation_preview.status,
        adapter_contract_status: validation_preview.adapter_contract_status,
        adapter_kind: validation_preview.adapter_kind,
        normalized_model_family: validation_preview.normalized_model_family,
        normalized_model_format: validation_preview.normalized_model_format,
        executable_path_present: validation_preview.executable_path_present,
        model_path_present: validation_preview.model_path_present,
        executable_exists: validation_preview.executable_exists,
        model_exists: validation_preview.model_exists,
        executable_is_file: validation_preview.executable_is_file,
        model_is_file: validation_preview.model_is_file,
        model_extension_valid: validation_preview.model_extension_valid,
        safe_executable_file_name: validation_preview.safe_executable_file_name,
        safe_model_file_name: validation_preview.safe_model_file_name,
        probe_consent: request.probe_consent,
        required_inputs,
        missing_inputs,
        readiness_reasons,
        blockers,
        warnings,
        next_required_actions,
        summary,
        preview_only: true,
        no_process_spawn: true,
        no_binary_probe: true,
        no_model_load: true,
        no_llm_call: true,
        no_runtime_execution: true,
        no_persistence: true,
        no_artifact_write: true,
        no_registry_status_change: true,
        no_audit_write: true,
    })
}

pub fn run_llama_runtime_version_probe(
    root: impl Into<PathBuf>,
    request: LocalRuntimeVersionProbePreviewRequest,
) -> AegisResult<LocalRuntimeVersionProbePreview> {
    let root = root.into();
    let LocalRuntimeVersionProbePreviewRequest {
        probe_readiness_preview_request,
        allow_probe_execution,
        timeout_ms,
    } = request;
    let readiness_preview = preview_llama_runtime_probe_readiness(root.clone(), probe_readiness_preview_request.clone())?;
    let adapter_contract_request = &probe_readiness_preview_request.validation_preview_request.adapter_contract_request;
    let normalized_executable_path = normalize_optional_path(adapter_contract_request.executable_path.clone())?;
    let normalized_model_path = normalize_optional_path(adapter_contract_request.model_path.clone())?;
    let timeout_ms = clamp_probe_timeout_ms(timeout_ms);
    let probe_argument = "--version".to_string();
    let safe_executable_file_name = readiness_preview.safe_executable_file_name.clone();
    let safe_model_file_name = readiness_preview.safe_model_file_name.clone();
    let mut blockers: Vec<LocalRuntimeProbeWarning> = readiness_preview
        .blockers
        .iter()
        .map(probe_warning_blocker_from_adapter_contract)
        .collect();
    let mut warnings: Vec<LocalRuntimeProbeWarning> = readiness_preview
        .warnings
        .iter()
        .map(probe_warning_from_adapter_contract)
        .collect();
    let mut execution_attempted = false;
    let mut duration_ms = 0;
    let mut exit_code = None;
    let mut stdout_preview = String::new();
    let mut stderr_preview = String::new();
    let mut stdout_truncated = false;
    let mut stderr_truncated = false;

    let status = match readiness_preview.status {
        LocalRuntimeProbeReadinessStatus::Blocked | LocalRuntimeProbeReadinessStatus::NeedsReview => {
            push_probe_blocker(
                &mut blockers,
                "probe_readiness_not_ready_later",
                "The llama.cpp probe-readiness preview must be ready later before a version probe can run.",
            );
            LocalRuntimeVersionProbeStatus::Blocked
        }
        LocalRuntimeProbeReadinessStatus::ProbeReadyLater => {
            if !allow_probe_execution {
                push_probe_blocker(
                    &mut blockers,
                    "probe_execution_not_allowed",
                    "Probe execution was not allowed for this version probe.",
                );
                LocalRuntimeVersionProbeStatus::Blocked
            } else {
                match normalized_executable_path.as_deref() {
                    Some(executable_path) => {
                        let resolved_executable_path = resolve_runtime_path(&root, executable_path);
                        match run_version_probe(&resolved_executable_path, timeout_ms) {
                            Ok(execution) => {
                                execution_attempted = true;
                                let redactions = version_probe_output_redactions(
                                    Some(resolved_executable_path.to_string_lossy().as_ref()),
                                    safe_executable_file_name.as_deref(),
                                    normalized_model_path.as_deref(),
                                    safe_model_file_name.as_deref(),
                                );
                                let (stdout, stdout_is_truncated) = preview_probe_output_with_redactions(
                                    &execution.stdout,
                                    LOCAL_RUNTIME_PROBE_PREVIEW_LIMIT,
                                    &redactions,
                                );
                                let (stderr, stderr_is_truncated) = preview_probe_output_with_redactions(
                                    &execution.stderr,
                                    LOCAL_RUNTIME_PROBE_PREVIEW_LIMIT,
                                    &redactions,
                                );
                                stdout_preview = stdout;
                                stderr_preview = stderr;
                                stdout_truncated = stdout_is_truncated;
                                stderr_truncated = stderr_is_truncated;
                                if stdout_truncated {
                                    push_probe_warning(
                                        &mut warnings,
                                        "stdout_truncated",
                                        "Standard output was truncated to keep the version probe preview compact.",
                                    );
                                }
                                if stderr_truncated {
                                    push_probe_warning(
                                        &mut warnings,
                                        "stderr_truncated",
                                        "Standard error was truncated to keep the version probe preview compact.",
                                    );
                                }
                                if execution.timed_out {
                                    push_probe_warning(
                                        &mut warnings,
                                        "timed_out",
                                        "The version probe reached its timeout and was stopped.",
                                    );
                                }
                                if let Some(exit_code_value) = execution.exit_code {
                                    if exit_code_value != 0 {
                                        push_probe_warning(
                                            &mut warnings,
                                            "non_zero_exit",
                                            "The version probe exited with a non-zero status.",
                                        );
                                    }
                                } else {
                                    push_probe_warning(
                                        &mut warnings,
                                        "no_exit_code",
                                        "The version probe did not report an exit code.",
                                    );
                                }
                                duration_ms = execution.duration_ms;
                                exit_code = execution.exit_code;
                                if execution.timed_out {
                                    LocalRuntimeVersionProbeStatus::TimedOut
                                } else if execution.exit_code == Some(0) {
                                    LocalRuntimeVersionProbeStatus::ProbeSucceeded
                                } else {
                                    LocalRuntimeVersionProbeStatus::ProbeFailed
                                }
                            }
                            Err(_) => {
                                push_probe_blocker(
                                    &mut blockers,
                                    "probe_start_failed",
                                    "The version probe could not start.",
                                );
                                LocalRuntimeVersionProbeStatus::Blocked
                            }
                        }
                    }
                    None => {
                        push_probe_blocker(
                            &mut blockers,
                            "executable_missing",
                            "An executable path is required for this version probe.",
                        );
                        LocalRuntimeVersionProbeStatus::Blocked
                    }
                }
            }
        }
    };

    let summary = llama_runtime_version_probe_summary(
        &status,
        &readiness_preview.status,
        allow_probe_execution,
        safe_executable_file_name.as_deref(),
        safe_model_file_name.as_deref(),
    );

    let next_required_actions = llama_runtime_version_probe_required_actions(
        &readiness_preview.status,
        allow_probe_execution,
        &blockers,
    );

    Ok(LocalRuntimeVersionProbePreview {
        status,
        probe_readiness_status: readiness_preview.status,
        validation_status: readiness_preview.validation_status,
        adapter_contract_status: readiness_preview.adapter_contract_status,
        adapter_kind: readiness_preview.adapter_kind,
        normalized_model_family: readiness_preview.normalized_model_family,
        normalized_model_format: readiness_preview.normalized_model_format,
        safe_executable_file_name,
        safe_model_file_name,
        probe_consent: readiness_preview.probe_consent,
        allow_probe_execution,
        execution_attempted,
        probe_argument,
        timeout_ms,
        duration_ms,
        exit_code,
        stdout_preview,
        stderr_preview,
        stdout_truncated,
        stderr_truncated,
        blockers,
        warnings,
        next_required_actions,
        summary,
        preview_only: true,
        no_model_load: true,
        no_model_path_argument: true,
        no_llm_call: true,
        no_runtime_inference: true,
        no_persistence: true,
        no_artifact_write: true,
        no_registry_status_change: true,
        no_audit_write: true,
    })
}

pub fn preview_llama_runtime_capability(
    root: impl Into<PathBuf>,
    request: LocalRuntimeCapabilityPreviewRequest,
) -> AegisResult<LocalRuntimeCapabilityPreview> {
    let root = root.into();
    let LocalRuntimeCapabilityPreviewRequest {
        version_probe_preview_request,
    } = request;
    let version_probe_result = run_llama_runtime_version_probe(root, version_probe_preview_request)?;
    let version_probe_status = version_probe_result.status.clone();
    let status = llama_runtime_capability_status(&version_probe_status);
    let inferred_version_text = match status {
        LocalRuntimeCapabilityStatus::CapabilityReadyLater => infer_version_text_from_probe_result(
            &version_probe_result.stdout_preview,
            &version_probe_result.stderr_preview,
        ),
        LocalRuntimeCapabilityStatus::Blocked | LocalRuntimeCapabilityStatus::NeedsReview => None,
    };
    let inferred_runtime_available = matches!(
        status,
        LocalRuntimeCapabilityStatus::CapabilityReadyLater
    );
    let capability_reasons = llama_runtime_capability_reasons(
        &status,
        &version_probe_result,
        inferred_version_text.as_deref(),
    );
    let blockers = version_probe_result.blockers.clone();
    let warnings = version_probe_result.warnings.clone();
    let next_required_actions = llama_runtime_capability_next_required_actions(
        &status,
        &version_probe_result,
    );
    let summary = llama_runtime_capability_summary(
        &status,
        &version_probe_result,
        inferred_version_text.as_deref(),
    );
    let version_probe_timed_out = matches!(version_probe_status, LocalRuntimeVersionProbeStatus::TimedOut);

    Ok(LocalRuntimeCapabilityPreview {
        status,
        version_probe_status: version_probe_status.clone(),
        probe_readiness_status: version_probe_result.probe_readiness_status,
        validation_status: version_probe_result.validation_status,
        adapter_contract_status: version_probe_result.adapter_contract_status,
        adapter_kind: version_probe_result.adapter_kind,
        normalized_model_family: version_probe_result.normalized_model_family,
        normalized_model_format: version_probe_result.normalized_model_format,
        safe_executable_file_name: version_probe_result.safe_executable_file_name,
        safe_model_file_name: version_probe_result.safe_model_file_name,
        probe_consent: version_probe_result.probe_consent,
        allow_probe_execution: version_probe_result.allow_probe_execution,
        version_probe_execution_attempted: version_probe_result.execution_attempted,
        version_probe_exit_code: version_probe_result.exit_code,
        version_probe_timed_out,
        version_probe_stdout_preview: version_probe_result.stdout_preview,
        version_probe_stderr_preview: version_probe_result.stderr_preview,
        inferred_runtime_available,
        inferred_version_text,
        capability_reasons,
        blockers,
        warnings,
        next_required_actions,
        summary,
        preview_only: true,
        no_new_process_spawn: true,
        no_binary_probe_beyond_wrapped_version_probe: true,
        no_model_path_argument: true,
        no_model_file_read: true,
        no_model_load: true,
        no_runtime_inference: true,
        no_smoke_inference: true,
        no_llm_call: true,
        no_persistence: true,
        no_artifact_write: true,
        no_registry_status_change: true,
        no_audit_write: true,
    })
}

pub fn preview_llama_runtime_smoke_readiness(
    root: impl Into<PathBuf>,
    request: LocalRuntimeSmokeReadinessPreviewRequest,
) -> AegisResult<LocalRuntimeSmokeReadinessPreview> {
    let root = root.into();
    let LocalRuntimeSmokeReadinessPreviewRequest {
        capability_preview_request,
        smoke_consent,
        diagnostic_prompt,
        max_output_tokens,
        timeout_ms,
    } = request;
    let capability_preview = preview_llama_runtime_capability(&root, capability_preview_request)?;
    let capability_status = capability_preview.status.clone();
    let mut warnings = capability_preview.warnings.clone();
    push_probe_warning(
        &mut warnings,
        "boundary",
        "This is a smoke readiness preview only; it does not run inference, pass a model path to a process, load or read a model, call an LLM, or persist settings or artifacts.",
    );
    let normalized_diagnostic_prompt = normalize_smoke_readiness_prompt(
        diagnostic_prompt.as_deref(),
        &mut warnings,
    );
    let diagnostic_prompt_char_count = normalized_diagnostic_prompt
        .as_deref()
        .map(|value| value.chars().count())
        .unwrap_or(0);
    let max_output_tokens = clamp_smoke_max_output_tokens(max_output_tokens);
    let timeout_ms = clamp_smoke_timeout_ms(timeout_ms);
    let status = llama_runtime_smoke_readiness_status(
        &capability_status,
        smoke_consent,
        normalized_diagnostic_prompt.is_some(),
    );
    let mut readiness_blockers = capability_preview.blockers.clone();
    if matches!(
        capability_status,
        LocalRuntimeCapabilityStatus::CapabilityReadyLater
    ) {
        if !smoke_consent {
            push_probe_blocker(
                &mut readiness_blockers,
                "smoke_consent_missing",
                "Smoke consent was not given.",
            );
        } else if normalized_diagnostic_prompt.is_none() {
            push_probe_blocker(
                &mut readiness_blockers,
                "diagnostic_prompt_missing",
                "A diagnostic prompt is required for smoke readiness.",
            );
        }
    }
    let required_inputs = llama_runtime_smoke_readiness_required_inputs();
    let missing_inputs = llama_runtime_smoke_readiness_missing_inputs(
        &capability_status,
        smoke_consent,
        normalized_diagnostic_prompt.is_some(),
    );
    let mut readiness_reasons = llama_runtime_smoke_readiness_reasons(
        &status,
        &capability_preview,
        smoke_consent,
        diagnostic_prompt_char_count,
        max_output_tokens,
        timeout_ms,
    );
    let next_required_actions = llama_runtime_smoke_readiness_next_required_actions(
        &status,
        &capability_preview,
        smoke_consent,
        normalized_diagnostic_prompt.is_some(),
    );
    let summary = llama_runtime_smoke_readiness_summary(
        &status,
        &capability_preview,
        smoke_consent,
        diagnostic_prompt_char_count,
        max_output_tokens,
        timeout_ms,
    );

    if matches!(status, LocalRuntimeSmokeReadinessStatus::NeedsReview) {
        push_unique_text(
            &mut readiness_reasons,
            "The capability preview still needs review before smoke readiness can be accepted.",
        );
    }
    if matches!(status, LocalRuntimeSmokeReadinessStatus::SmokeReadyLater) {
        push_unique_text(
            &mut readiness_reasons,
            "A future diagnostic smoke inference preview can be added later without changing this preview.",
        );
    }

    Ok(LocalRuntimeSmokeReadinessPreview {
        status,
        capability_status,
        version_probe_status: capability_preview.version_probe_status,
        probe_readiness_status: capability_preview.probe_readiness_status,
        validation_status: capability_preview.validation_status,
        adapter_contract_status: capability_preview.adapter_contract_status,
        adapter_kind: capability_preview.adapter_kind,
        normalized_model_family: capability_preview.normalized_model_family,
        normalized_model_format: capability_preview.normalized_model_format,
        safe_executable_file_name: capability_preview.safe_executable_file_name,
        safe_model_file_name: capability_preview.safe_model_file_name,
        probe_consent: capability_preview.probe_consent,
        allow_probe_execution: capability_preview.allow_probe_execution,
        smoke_consent,
        normalized_diagnostic_prompt: normalized_diagnostic_prompt.unwrap_or_default(),
        diagnostic_prompt_char_count,
        max_output_tokens,
        timeout_ms,
        required_inputs,
        missing_inputs,
        readiness_reasons,
        blockers: readiness_blockers,
        warnings,
        next_required_actions,
        summary,
        preview_only: true,
        no_new_process_spawn: true,
        no_smoke_inference_execution: true,
        no_model_path_argument: true,
        no_model_file_read: true,
        no_model_load: true,
        no_llm_call: true,
        no_persistence: true,
        no_artifact_write: true,
        no_registry_status_change: true,
        no_audit_write: true,
        diagnostic_only: true,
        not_scholar_chat_answer: true,
        no_answer_generated: true,
        no_grounding_applied: true,
        no_evidence_pack_used: true,
    })
}

pub fn probe_local_runtime_version(
    root: impl Into<PathBuf>,
    request: LocalRuntimeProbeRequest,
) -> AegisResult<LocalRuntimeProbeResult> {
    let root = root.into();
    let normalized_executable_path = normalize_optional_path(request.executable_path)?;
    let timeout_ms = clamp_probe_timeout_ms(request.timeout_ms);
    let safe_executable_file_name = safe_file_name_from_path(normalized_executable_path.as_deref());
    let probe_argument = "--version".to_string();
    let mut blockers = Vec::new();
    let mut warnings = vec![
        LocalRuntimeProbeWarning {
            kind: "preview_only".to_string(),
            message: "This is a preview-only runtime probe; no model is loaded and no answer is generated.".to_string(),
        },
        LocalRuntimeProbeWarning {
            kind: "no_persistence".to_string(),
            message: "Probe configuration is not persisted.".to_string(),
        },
    ];

    if !request.allow_execution {
        push_probe_blocker(
            &mut blockers,
            "execution_disabled",
            "Execution is disabled for this runtime probe.",
        );
        return Ok(build_probe_result(
            LocalRuntimeProbeStatus::Blocked,
            request.allow_execution,
            false,
            probe_argument,
            timeout_ms,
            0,
            safe_executable_file_name,
            None,
            String::new(),
            String::new(),
            blockers,
            warnings,
        ));
    }

    let Some(executable_path) = normalized_executable_path.as_deref() else {
        push_probe_blocker(
            &mut blockers,
            "executable_missing",
            "An executable path is required for this runtime probe.",
        );
        return Ok(build_probe_result(
            LocalRuntimeProbeStatus::Blocked,
            request.allow_execution,
            false,
            probe_argument,
            timeout_ms,
            0,
            safe_executable_file_name,
            None,
            String::new(),
            String::new(),
            blockers,
            warnings,
        ));
    };

    let resolved_executable_path = resolve_runtime_path(&root, executable_path);
    match fs::metadata(&resolved_executable_path) {
        Ok(metadata) if metadata.is_file() => {}
        Ok(_) => {
            push_probe_blocker(
                &mut blockers,
                "executable_not_a_file",
                "The configured executable path does not point to a file.",
            );
            return Ok(build_probe_result(
                LocalRuntimeProbeStatus::Blocked,
                request.allow_execution,
                false,
                probe_argument,
                timeout_ms,
                0,
                safe_executable_file_name,
                None,
                String::new(),
                String::new(),
                blockers,
                warnings,
            ));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            push_probe_blocker(
                &mut blockers,
                "executable_missing",
                "The configured executable file is missing.",
            );
            return Ok(build_probe_result(
                LocalRuntimeProbeStatus::Blocked,
                request.allow_execution,
                false,
                probe_argument,
                timeout_ms,
                0,
                safe_executable_file_name,
                None,
                String::new(),
                String::new(),
                blockers,
                warnings,
            ));
        }
        Err(_) => {
            push_probe_blocker(
                &mut blockers,
                "executable_unavailable",
                "The configured executable could not be inspected.",
            );
            return Ok(build_probe_result(
                LocalRuntimeProbeStatus::Blocked,
                request.allow_execution,
                false,
                probe_argument,
                timeout_ms,
                0,
                safe_executable_file_name,
                None,
                String::new(),
                String::new(),
                blockers,
                warnings,
            ));
        }
    }

    match run_version_probe(&resolved_executable_path, timeout_ms) {
        Ok(execution) => {
            let (stdout_preview, stdout_truncated) = preview_probe_output(&execution.stdout, LOCAL_RUNTIME_PROBE_PREVIEW_LIMIT);
            let (stderr_preview, stderr_truncated) = preview_probe_output(&execution.stderr, LOCAL_RUNTIME_PROBE_PREVIEW_LIMIT);
            if stdout_truncated {
                push_probe_warning(
                    &mut warnings,
                    "stdout_truncated",
                    "Standard output was truncated to keep the preview compact.",
                );
            }
            if stderr_truncated {
                push_probe_warning(
                    &mut warnings,
                    "stderr_truncated",
                    "Standard error was truncated to keep the preview compact.",
                );
            }
            if execution.timed_out {
                push_probe_warning(
                    &mut warnings,
                    "timed_out",
                    "The runtime probe reached its timeout and was stopped.",
                );
            }
            if let Some(exit_code) = execution.exit_code {
                if exit_code != 0 {
                    push_probe_warning(
                        &mut warnings,
                        "non_zero_exit",
                        "The runtime probe exited with a non-zero status.",
                    );
                }
            } else {
                push_probe_warning(
                    &mut warnings,
                    "no_exit_code",
                    "The runtime probe did not report an exit code.",
                );
            }
            Ok(build_probe_result(
                if execution.timed_out {
                    LocalRuntimeProbeStatus::TimedOut
                } else {
                    LocalRuntimeProbeStatus::Completed
                },
                request.allow_execution,
                true,
                probe_argument,
                timeout_ms,
                execution.duration_ms,
                safe_executable_file_name,
                execution.exit_code,
                stdout_preview,
                stderr_preview,
                blockers,
                warnings,
            ))
        }
        Err(_) => {
            push_probe_blocker(
                &mut blockers,
                "probe_start_failed",
                "The runtime probe could not start.",
            );
            Ok(build_probe_result(
                LocalRuntimeProbeStatus::Blocked,
                request.allow_execution,
                false,
                probe_argument,
                timeout_ms,
                0,
                safe_executable_file_name,
                None,
                String::new(),
                String::new(),
                blockers,
                warnings,
            ))
        }
    }
}

pub fn smoke_test_local_runtime_inference(
    root: impl Into<PathBuf>,
    request: LocalRuntimeSmokeInferenceRequest,
) -> AegisResult<LocalRuntimeSmokeInferenceResult> {
    let root = root.into();
    let runtime_config = request.runtime_config;
    let trimmed_model_path = normalize_optional_text(runtime_config.model_path.clone());
    let trimmed_executable_path = normalize_optional_text(runtime_config.executable_path.clone());
    let timeout_ms = clamp_smoke_timeout_ms(request.timeout_ms);
    let max_output_tokens = clamp_smoke_max_output_tokens(request.max_output_tokens);
    let safe_model_file_name = safe_file_name_from_path(trimmed_model_path.as_deref());
    let safe_executable_file_name = safe_file_name_from_path(trimmed_executable_path.as_deref());
    let mut warnings = vec![
        LocalRuntimeSmokeInferenceWarning {
            kind: "preview_only".to_string(),
            message: "This is a preview-only smoke probe; no answer is generated.".to_string(),
        },
        LocalRuntimeSmokeInferenceWarning {
            kind: "no_persistence".to_string(),
            message: "Smoke probe configuration is not persisted.".to_string(),
        },
        LocalRuntimeSmokeInferenceWarning {
            kind: "no_grounding".to_string(),
            message: "This smoke probe does not use source or evidence grounding.".to_string(),
        },
    ];
    if request.timeout_ms.is_some() && request.timeout_ms != Some(timeout_ms) {
        push_smoke_warning(
            &mut warnings,
            "timeout_clamped",
            "The smoke probe timeout was clamped to a safe range.",
        );
    }
    if request.max_output_tokens.is_some() && request.max_output_tokens != Some(max_output_tokens) {
        push_smoke_warning(
            &mut warnings,
            "max_output_tokens_clamped",
            "The smoke probe output token limit was clamped to a safe range.",
        );
    }
    let prompt_text = normalize_smoke_prompt(request.prompt.as_deref(), &mut warnings);
    let normalized_prompt = prompt_text.clone().unwrap_or_default();
    let prompt_char_count = normalized_prompt.chars().count() as u32;
    let mut blockers = Vec::new();

    if !request.allow_execution {
        push_smoke_blocker(
            &mut blockers,
            "execution_disabled",
            "Execution is disabled for this smoke probe.",
        );
        return Ok(build_smoke_result(
            LocalRuntimeSmokeInferenceStatus::Blocked,
            request.allow_execution,
            false,
            runtime_config.runtime_kind,
            safe_model_file_name,
            safe_executable_file_name,
            normalized_prompt,
            prompt_char_count,
            max_output_tokens,
            timeout_ms,
            None,
            String::new(),
            String::new(),
            0,
            warnings,
            blockers,
        ));
    }

    let Some(prompt) = prompt_text else {
        push_smoke_blocker(
            &mut blockers,
            "prompt_empty",
            "A smoke-test prompt is required.",
        );
        return Ok(build_smoke_result(
            LocalRuntimeSmokeInferenceStatus::Blocked,
            request.allow_execution,
            false,
            runtime_config.runtime_kind,
            safe_model_file_name,
            safe_executable_file_name,
            normalized_prompt,
            prompt_char_count,
            max_output_tokens,
            timeout_ms,
            None,
            String::new(),
            String::new(),
            0,
            warnings,
            blockers,
        ));
    };

    let normalized_model_path = normalize_optional_path(trimmed_model_path.clone())?;
    let normalized_executable_path = normalize_optional_path(trimmed_executable_path.clone())?;
    let model_lookup = inspect_configured_path(&root, normalized_model_path.as_deref())?;
    let executable_lookup = inspect_configured_path(&root, normalized_executable_path.as_deref())?;
    let model_state = model_lookup.state;
    let executable_state = executable_lookup.state;
    let model_extension_valid = model_lookup.extension_valid;

    match runtime_config.runtime_kind {
        LocalModelRuntimeKind::None => {
            push_smoke_blocker(
                &mut blockers,
                "runtime_not_configured",
                "A local runtime kind is required for smoke inference.",
            );
            return Ok(build_smoke_result(
                LocalRuntimeSmokeInferenceStatus::NotConfigured,
                request.allow_execution,
                false,
                runtime_config.runtime_kind,
                safe_model_file_name,
                safe_executable_file_name,
                normalized_prompt,
                prompt_char_count,
                max_output_tokens,
                timeout_ms,
                None,
                String::new(),
                String::new(),
                0,
                warnings,
                blockers,
            ));
        }
        LocalModelRuntimeKind::LlamaCpp => {}
    }

    if !matches!(model_state, LocalModelRuntimePathState::Exists) {
        push_smoke_blocker(
            &mut blockers,
            "model_missing",
            "Configured model file is missing.",
        );
        return Ok(build_smoke_result(
            LocalRuntimeSmokeInferenceStatus::ModelMissing,
            request.allow_execution,
            false,
            runtime_config.runtime_kind,
            safe_model_file_name,
            safe_executable_file_name,
            normalized_prompt,
            prompt_char_count,
            max_output_tokens,
            timeout_ms,
            None,
            String::new(),
            String::new(),
            0,
            warnings,
            blockers,
        ));
    }

    if !matches!(executable_state, LocalModelRuntimePathState::Exists) {
        push_smoke_blocker(
            &mut blockers,
            "executable_missing",
            "Configured executable file is missing.",
        );
        return Ok(build_smoke_result(
            LocalRuntimeSmokeInferenceStatus::ExecutableMissing,
            request.allow_execution,
            false,
            runtime_config.runtime_kind,
            safe_model_file_name,
            safe_executable_file_name,
            normalized_prompt,
            prompt_char_count,
            max_output_tokens,
            timeout_ms,
            None,
            String::new(),
            String::new(),
            0,
            warnings,
            blockers,
        ));
    }

    if !model_extension_valid {
        push_smoke_blocker(
            &mut blockers,
            "model_extension_invalid",
            "Configured model file does not use a .gguf extension.",
        );
        return Ok(build_smoke_result(
            LocalRuntimeSmokeInferenceStatus::Blocked,
            request.allow_execution,
            false,
            runtime_config.runtime_kind,
            safe_model_file_name,
            safe_executable_file_name,
            normalized_prompt,
            prompt_char_count,
            max_output_tokens,
            timeout_ms,
            None,
            String::new(),
            String::new(),
            0,
            warnings,
            blockers,
        ));
    }

    let resolved_model_path = resolve_runtime_path(&root, normalized_model_path.as_deref().unwrap());
    let resolved_executable_path = resolve_runtime_path(&root, normalized_executable_path.as_deref().unwrap());

    match run_smoke_inference_probe(
        &resolved_executable_path,
        &resolved_model_path,
        &prompt,
        max_output_tokens,
        timeout_ms,
        runtime_config.context_window,
        runtime_config.gpu_layers,
        runtime_config.temperature,
    ) {
        Ok(execution) => {
            let redactions = [
                (
                    root.to_string_lossy().to_string(),
                    "<root>".to_string(),
                ),
                (
                    resolved_model_path.to_string_lossy().to_string(),
                    safe_model_file_name.clone().unwrap_or_else(|| "<model>".to_string()),
                ),
                (
                    resolved_executable_path.to_string_lossy().to_string(),
                    safe_executable_file_name
                        .clone()
                        .unwrap_or_else(|| "<executable>".to_string()),
                ),
            ];
            let (stdout_preview, stdout_truncated) = preview_probe_output_with_redactions(
                &execution.stdout,
                LOCAL_RUNTIME_SMOKE_PREVIEW_LIMIT,
                &redactions,
            );
            let (stderr_preview, stderr_truncated) = preview_probe_output_with_redactions(
                &execution.stderr,
                LOCAL_RUNTIME_SMOKE_PREVIEW_LIMIT,
                &redactions,
            );
            if stdout_truncated {
                push_smoke_warning(
                    &mut warnings,
                    "stdout_truncated",
                    "Standard output was truncated to keep the preview compact.",
                );
            }
            if stderr_truncated {
                push_smoke_warning(
                    &mut warnings,
                    "stderr_truncated",
                    "Standard error was truncated to keep the preview compact.",
                );
            }
            if execution.timed_out {
                push_smoke_warning(
                    &mut warnings,
                    "timed_out",
                    "The smoke probe reached its timeout and was stopped.",
                );
            }
            if let Some(exit_code) = execution.exit_code {
                if exit_code != 0 {
                    push_smoke_warning(
                        &mut warnings,
                        "non_zero_exit",
                        "The smoke probe exited with a non-zero status.",
                    );
                }
            } else {
                push_smoke_warning(
                    &mut warnings,
                    "no_exit_code",
                    "The smoke probe did not report an exit code.",
                );
            }
            Ok(build_smoke_result(
                if execution.timed_out {
                    LocalRuntimeSmokeInferenceStatus::TimedOut
                } else if execution.exit_code.unwrap_or(1) == 0 {
                    LocalRuntimeSmokeInferenceStatus::InferenceSucceeded
                } else {
                    LocalRuntimeSmokeInferenceStatus::InferenceFailed
                },
                request.allow_execution,
                true,
                runtime_config.runtime_kind,
                safe_model_file_name,
                safe_executable_file_name,
                normalized_prompt,
                prompt_char_count,
                max_output_tokens,
                timeout_ms,
                execution.exit_code,
                stdout_preview,
                stderr_preview,
                execution.duration_ms,
                warnings,
                blockers,
            ))
        }
        Err(_) => {
            push_smoke_blocker(
                &mut blockers,
                "probe_start_failed",
                "The smoke probe could not start.",
            );
            Ok(build_smoke_result(
                LocalRuntimeSmokeInferenceStatus::InferenceFailed,
                request.allow_execution,
                true,
                runtime_config.runtime_kind,
                safe_model_file_name,
                safe_executable_file_name,
                normalized_prompt,
                prompt_char_count,
                max_output_tokens,
                timeout_ms,
                None,
                String::new(),
                String::new(),
                0,
                warnings,
                blockers,
            ))
        }
    }
}

pub fn run_llama_runtime_smoke_diagnostic(
    root: impl Into<PathBuf>,
    request: LocalRuntimeSmokeDiagnosticRequest,
) -> AegisResult<LocalRuntimeSmokeDiagnosticPreview> {
    let root = root.into();
    let LocalRuntimeSmokeDiagnosticRequest {
        smoke_execution_plan_preview_request,
        allow_smoke_execution,
    } = request;
    let smoke_execution_plan_preview =
        preview_llama_runtime_smoke_execution_plan(&root, smoke_execution_plan_preview_request.clone())?;
    let smoke_execution_plan_status = smoke_execution_plan_preview.status.clone();
    let smoke_readiness_status = smoke_execution_plan_preview.smoke_readiness_status.clone();
    let capability_status = smoke_execution_plan_preview.capability_status.clone();
    let version_probe_status = smoke_execution_plan_preview.version_probe_status.clone();
    let probe_readiness_status = smoke_execution_plan_preview.probe_readiness_status.clone();
    let validation_status = smoke_execution_plan_preview.validation_status.clone();
    let adapter_contract_status = smoke_execution_plan_preview.adapter_contract_status.clone();
    let adapter_kind = smoke_execution_plan_preview.adapter_kind.clone();
    let normalized_model_family = smoke_execution_plan_preview.normalized_model_family.clone();
    let normalized_model_format = smoke_execution_plan_preview.normalized_model_format.clone();
    let safe_executable_file_name = smoke_execution_plan_preview.safe_executable_file_name.clone();
    let safe_model_file_name = smoke_execution_plan_preview.safe_model_file_name.clone();
    let probe_consent = smoke_execution_plan_preview.probe_consent;
    let allow_probe_execution = smoke_execution_plan_preview.allow_probe_execution;
    let smoke_consent = smoke_execution_plan_preview.smoke_consent;
    let normalized_diagnostic_prompt = smoke_execution_plan_preview.normalized_diagnostic_prompt.clone();
    let diagnostic_prompt_char_count = smoke_execution_plan_preview.diagnostic_prompt_char_count;
    let max_output_tokens = smoke_execution_plan_preview.max_output_tokens;
    let timeout_ms = smoke_execution_plan_preview.timeout_ms;

    let mut blockers: Vec<LocalRuntimeSmokeInferenceBlocker> = smoke_execution_plan_preview
        .blockers
        .iter()
        .map(|blocker| LocalRuntimeSmokeInferenceBlocker {
            kind: blocker.kind.clone(),
            message: blocker.message.clone(),
        })
        .collect();
    let mut warnings: Vec<LocalRuntimeSmokeInferenceWarning> = smoke_execution_plan_preview
        .warnings
        .iter()
        .map(|warning| LocalRuntimeSmokeInferenceWarning {
            kind: warning.kind.clone(),
            message: warning.message.clone(),
        })
        .collect();
    push_smoke_warning(
        &mut warnings,
        "boundary",
        "This is a diagnostic smoke execution only; it does not generate a Scholar Chat answer, ground claims, build an Evidence Pack, or persist settings or artifacts.",
    );

    let mut execution_attempted = false;
    let mut duration_ms = 0;
    let mut exit_code = None;
    let mut stdout_preview = String::new();
    let mut stderr_preview = String::new();
    let mut stdout_truncated = false;
    let mut stderr_truncated = false;
    let mut status = LocalRuntimeSmokeDiagnosticStatus::Blocked;
    let mut next_required_actions = smoke_execution_plan_preview.next_required_actions.clone();

    if !matches!(smoke_execution_plan_status, LocalRuntimeSmokeExecutionPlanStatus::PlanReadyLater) {
        push_smoke_blocker(
            &mut blockers,
            "smoke_execution_plan_not_ready_later",
            "The smoke execution plan must be ready later before diagnostic smoke execution can begin.",
        );
        push_unique_text(
            &mut next_required_actions,
            "Resolve the smoke execution plan before running diagnostic smoke execution.",
        );
    } else if !allow_smoke_execution {
        push_smoke_blocker(
            &mut blockers,
            "smoke_execution_consent_missing",
            "Diagnostic smoke execution consent was not given.",
        );
        push_unique_text(
            &mut next_required_actions,
            "Grant diagnostic smoke execution consent.",
        );
    } else {
        let smoke_inference_request = LocalRuntimeSmokeInferenceRequest {
            runtime_config: smoke_runtime_config_from_execution_plan_request(
                &smoke_execution_plan_preview_request,
            ),
            allow_execution: true,
            prompt: Some(normalized_diagnostic_prompt.clone()),
            timeout_ms: Some(timeout_ms),
            max_output_tokens: Some(max_output_tokens),
        };
        let smoke_result = smoke_test_local_runtime_inference(&root, smoke_inference_request)?;
        execution_attempted = smoke_result.execution_attempted;
        duration_ms = smoke_result.duration_ms;
        exit_code = smoke_result.exit_code;
        stdout_preview = smoke_result.stdout_preview;
        stderr_preview = smoke_result.stderr_preview;
        stdout_truncated = smoke_warning_has_kind(&smoke_result.warnings, "stdout_truncated");
        stderr_truncated = smoke_warning_has_kind(&smoke_result.warnings, "stderr_truncated");
        blockers.extend(smoke_result.blockers);
        warnings.extend(smoke_result.warnings);
        match smoke_result.status {
            LocalRuntimeSmokeInferenceStatus::InferenceSucceeded => {
                status = LocalRuntimeSmokeDiagnosticStatus::SmokeSucceeded;
                push_unique_text(
                    &mut next_required_actions,
                    "Review the diagnostic smoke output before proceeding.",
                );
            }
            LocalRuntimeSmokeInferenceStatus::TimedOut => {
                status = LocalRuntimeSmokeDiagnosticStatus::TimedOut;
                push_unique_text(
                    &mut next_required_actions,
                    "Review the diagnostic smoke output and timeout conditions before proceeding.",
                );
            }
            LocalRuntimeSmokeInferenceStatus::InferenceFailed
            | LocalRuntimeSmokeInferenceStatus::Blocked
            | LocalRuntimeSmokeInferenceStatus::NotConfigured
            | LocalRuntimeSmokeInferenceStatus::ModelMissing
            | LocalRuntimeSmokeInferenceStatus::ExecutableMissing => {
                status = LocalRuntimeSmokeDiagnosticStatus::SmokeFailed;
                push_unique_text(
                    &mut next_required_actions,
                    "Review the diagnostic smoke output and stderr before proceeding.",
                );
            }
        }
    }

    let summary = match status {
        LocalRuntimeSmokeDiagnosticStatus::Blocked => {
            if matches!(smoke_execution_plan_status, LocalRuntimeSmokeExecutionPlanStatus::Blocked)
            {
                "The diagnostic smoke execution is blocked because the smoke execution plan is blocked.".to_string()
            } else if matches!(
                smoke_execution_plan_status,
                LocalRuntimeSmokeExecutionPlanStatus::NeedsReview
            ) {
                "The diagnostic smoke execution is blocked because the smoke execution plan still needs review.".to_string()
            } else if !allow_smoke_execution {
                "The diagnostic smoke execution is blocked until explicit execution consent is granted.".to_string()
            } else {
                "The diagnostic smoke execution is blocked because the smoke probe could not run.".to_string()
            }
        }
        LocalRuntimeSmokeDiagnosticStatus::SmokeSucceeded => {
            "The diagnostic smoke execution succeeded and remains diagnostic-only.".to_string()
        }
        LocalRuntimeSmokeDiagnosticStatus::SmokeFailed => {
            "The diagnostic smoke execution failed and remains diagnostic-only.".to_string()
        }
        LocalRuntimeSmokeDiagnosticStatus::TimedOut => {
            "The diagnostic smoke execution timed out and remains diagnostic-only.".to_string()
        }
    };

    Ok(LocalRuntimeSmokeDiagnosticPreview {
        status,
        smoke_execution_plan_status,
        smoke_readiness_status,
        capability_status,
        version_probe_status,
        probe_readiness_status,
        validation_status,
        adapter_contract_status,
        adapter_kind,
        normalized_model_family,
        normalized_model_format,
        safe_executable_file_name,
        safe_model_file_name,
        probe_consent,
        allow_probe_execution,
        smoke_consent,
        allow_smoke_execution,
        execution_attempted,
        normalized_diagnostic_prompt,
        diagnostic_prompt_char_count,
        max_output_tokens,
        timeout_ms,
        duration_ms,
        exit_code,
        stdout_preview,
        stderr_preview,
        stdout_truncated,
        stderr_truncated,
        blockers,
        warnings,
        next_required_actions,
        summary,
        diagnostic_only: true,
        not_scholar_chat_answer: true,
        no_answer_generated: true,
        no_grounding_applied: true,
        no_evidence_pack_used: true,
        no_persistence: true,
        no_artifact_write: true,
        no_registry_status_change: true,
        no_audit_write: true,
    })
}

struct PathInspection {
    state: LocalModelRuntimePathState,
    extension_valid: bool,
    file_name: Option<String>,
}

struct RuntimeValidationPathInspection {
    exists: bool,
    is_file: bool,
}

fn normalize_optional_path(path: Option<String>) -> AegisResult<Option<String>> {
    match path {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                validate_runtime_path(trimmed)?;
                Ok(Some(trimmed.to_string()))
            }
        }
        None => Ok(None),
    }
}

fn normalize_optional_text(path: Option<String>) -> Option<String> {
    path.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn normalize_optional_text_list(values: Option<Vec<String>>) -> Vec<String> {
    let mut normalized = values
        .unwrap_or_default()
        .into_iter()
        .filter_map(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    normalized
}

fn normalize_llama_runtime_adapter_model_family(model_family: Option<String>) -> Option<String> {
    let Some(model_family) = normalize_optional_text(model_family) else {
        return None;
    };
    let compact = model_family
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .map(|ch| ch.to_ascii_lowercase())
        .collect::<String>();
    let normalized = if compact.starts_with("gemma") {
        "gemma"
    } else if compact.starts_with("llama") {
        "llama"
    } else if compact.starts_with("qwen") {
        "qwen"
    } else if compact.starts_with("mistral") {
        "mistral"
    } else {
        return Some(model_family.to_ascii_lowercase());
    };
    Some(normalized.to_string())
}

fn is_known_llama_runtime_adapter_model_family(model_family: &str) -> bool {
    matches!(model_family, "gemma" | "llama" | "qwen" | "mistral")
}

fn normalize_llama_runtime_adapter_model_format(model_format: Option<String>) -> (String, bool) {
    let Some(model_format) = normalize_optional_text(model_format) else {
        return ("gguf".to_string(), true);
    };
    let normalized = model_format.to_ascii_lowercase();
    let supported = normalized == "gguf";
    (if normalized.is_empty() { "gguf".to_string() } else { normalized }, supported)
}

const LOCAL_RUNTIME_ADAPTER_MIN_CONTEXT_WINDOW_TOKENS: u32 = 1;
const LOCAL_RUNTIME_ADAPTER_MAX_CONTEXT_WINDOW_TOKENS: u32 = 131_072;

fn push_adapter_contract_warning(
    warnings: &mut Vec<LocalRuntimeAdapterContractWarning>,
    kind: &str,
    message: &str,
) {
    if !warnings
        .iter()
        .any(|warning| warning.kind == kind && warning.message == message)
    {
        warnings.push(LocalRuntimeAdapterContractWarning {
            kind: kind.to_string(),
            message: message.to_string(),
        });
    }
}

fn push_adapter_contract_blocker(
    blockers: &mut Vec<LocalRuntimeAdapterContractBlocker>,
    kind: &str,
    message: &str,
) {
    if !blockers
        .iter()
        .any(|blocker| blocker.kind == kind && blocker.message == message)
    {
        blockers.push(LocalRuntimeAdapterContractBlocker {
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

fn llama_runtime_adapter_contract_required_inputs() -> Vec<String> {
    vec!["executable_path".to_string(), "model_path".to_string()]
}

fn llama_runtime_adapter_contract_missing_inputs(
    executable_path_present: bool,
    model_path_present: bool,
    blockers: &[LocalRuntimeAdapterContractBlocker],
) -> Vec<String> {
    let mut missing_inputs = Vec::new();
    if !executable_path_present
        && blockers.iter().any(|blocker| blocker.kind == "executable_path_missing")
    {
        missing_inputs.push("executable_path".to_string());
    }
    if !model_path_present && blockers.iter().any(|blocker| blocker.kind == "model_path_missing") {
        missing_inputs.push("model_path".to_string());
    }
    missing_inputs
}

fn llama_runtime_adapter_contract_next_required_actions(
    status: &LocalRuntimeAdapterContractStatus,
    executable_path_present: bool,
    model_path_present: bool,
    warnings: &[LocalRuntimeAdapterContractWarning],
) -> Vec<String> {
    let mut next_required_actions = Vec::new();
    match status {
        LocalRuntimeAdapterContractStatus::Blocked => {
            if !executable_path_present {
                push_unique_text(
                    &mut next_required_actions,
                    "Provide an executable_path for the future llama.cpp adapter.",
                );
            }
            if !model_path_present {
                push_unique_text(
                    &mut next_required_actions,
                    "Provide a model_path for the future llama.cpp adapter.",
                );
            }
            if warnings.iter().any(|warning| warning.kind == "chat_template_missing_for_gemma") {
                push_unique_text(
                    &mut next_required_actions,
                    "Provide a chat_template before treating Gemma as ready for a future adapter.",
                );
            }
        }
        LocalRuntimeAdapterContractStatus::NeedsReview => {
            push_unique_text(
                &mut next_required_actions,
                "Review the model family and adapter metadata before wiring execution.",
            );
            if warnings.iter().any(|warning| warning.kind == "chat_template_missing_for_gemma") {
                push_unique_text(
                    &mut next_required_actions,
                    "Provide a chat_template before treating Gemma as ready for a future adapter.",
                );
            }
        }
        LocalRuntimeAdapterContractStatus::ContractReadyLater => {
            push_unique_text(
                &mut next_required_actions,
                "Implement the future llama.cpp adapter later without changing this preview.",
            );
        }
    }
    next_required_actions
}

fn llama_runtime_adapter_contract_summary(
    status: &LocalRuntimeAdapterContractStatus,
    executable_path_present: bool,
    model_path_present: bool,
    normalized_model_family: &Option<String>,
    normalized_model_format: &str,
    chat_template_present: bool,
) -> String {
    match status {
        LocalRuntimeAdapterContractStatus::Blocked => {
            let mut missing = Vec::new();
            if !executable_path_present {
                missing.push("executable_path".to_string());
            }
            if !model_path_present {
                missing.push("model_path".to_string());
            }
            if missing.is_empty() {
                "The llama.cpp adapter contract preview is blocked.".to_string()
            } else {
                format!(
                    "The llama.cpp adapter contract preview is blocked until {} are provided.",
                    missing.join(" and ")
                )
            }
        }
        LocalRuntimeAdapterContractStatus::NeedsReview => {
            let mut parts = vec!["The llama.cpp adapter contract preview needs review.".to_string()];
            if let Some(model_family) = normalized_model_family.as_deref() {
                parts.push(format!("Model family: {model_family}."));
            }
            if normalized_model_format != "gguf" {
                parts.push("Only gguf is supported for the preview.".to_string());
            }
            if !chat_template_present && normalized_model_family.as_deref() == Some("gemma") {
                parts.push("Gemma usually needs a chat template.".to_string());
            }
            parts.join(" ")
        }
        LocalRuntimeAdapterContractStatus::ContractReadyLater => {
            let family_text = normalized_model_family
                .as_deref()
                .map(|value| format!("Model family: {value}. "))
                .unwrap_or_default();
            let template_text = if chat_template_present {
                "A chat template is present. "
            } else {
                ""
            };
            format!(
                "The llama.cpp adapter contract preview is ready later. {family_text}{template_text}Normalized model format: {normalized_model_format}. No process was started, no model was loaded, and nothing was persisted."
            )
        }
    }
}

fn llama_runtime_adapter_contract_reasons(
    status: &LocalRuntimeAdapterContractStatus,
    normalized_model_family: &Option<String>,
    normalized_model_format: &str,
    chat_template_present: bool,
    blockers: &[LocalRuntimeAdapterContractBlocker],
    warnings: &[LocalRuntimeAdapterContractWarning],
) -> Vec<String> {
    let mut contract_reasons = Vec::new();
    push_unique_text(
        &mut contract_reasons,
        "This is a preview-only llama.cpp adapter contract; no process was started and no model was loaded.",
    );
    if let Some(model_family) = normalized_model_family.as_deref() {
        push_unique_text(
            &mut contract_reasons,
            &format!("Normalized model family: {model_family}."),
        );
    }
    push_unique_text(
        &mut contract_reasons,
        &format!("Normalized model format: {normalized_model_format}."),
    );
    if chat_template_present {
        push_unique_text(
            &mut contract_reasons,
            "A chat template was provided for the preview.",
        );
    }
    for blocker in blockers {
        push_unique_text(&mut contract_reasons, &format!("Blocker: {}", blocker.message));
    }
    for warning in warnings {
        push_unique_text(&mut contract_reasons, &format!("Warning: {}", warning.message));
    }
    if matches!(status, LocalRuntimeAdapterContractStatus::ContractReadyLater) {
        push_unique_text(
            &mut contract_reasons,
            "The contract preview is ready later for a future llama.cpp adapter implementation.",
        );
    }
    contract_reasons
}

fn validate_runtime_path(path: &str) -> AegisResult<()> {
    if Path::new(path)
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(AegisError::LocalModelRuntimeInvalidPath);
    }
    Ok(())
}

fn safe_file_name_from_path(path: Option<&str>) -> Option<String> {
    path.and_then(|value| {
        Path::new(value)
            .file_name()
            .and_then(|component| component.to_str())
            .map(|component| component.to_string())
    })
}

fn inspect_configured_path(root: &Path, path: Option<&str>) -> AegisResult<PathInspection> {
    let Some(path) = path else {
        return Ok(PathInspection {
            state: LocalModelRuntimePathState::NotConfigured,
            extension_valid: false,
            file_name: None,
        });
    };

    let resolved = resolve_runtime_path(root, path);
    let metadata = fs::metadata(&resolved);
    match metadata {
        Ok(metadata) if metadata.is_file() => Ok(PathInspection {
            state: LocalModelRuntimePathState::Exists,
            extension_valid: has_gguf_extension(&resolved),
            file_name: resolved.file_name().and_then(|value| value.to_str()).map(|value| value.to_string()),
        }),
        Ok(_) => Ok(PathInspection {
            state: LocalModelRuntimePathState::Missing,
            extension_valid: false,
            file_name: resolved.file_name().and_then(|value| value.to_str()).map(|value| value.to_string()),
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(PathInspection {
            state: LocalModelRuntimePathState::Missing,
            extension_valid: has_gguf_extension(&resolved),
            file_name: resolved.file_name().and_then(|value| value.to_str()).map(|value| value.to_string()),
        }),
        Err(error) => Err(error.into()),
    }
}

fn inspect_runtime_validation_path(
    root: &Path,
    path: Option<&str>,
) -> AegisResult<RuntimeValidationPathInspection> {
    let Some(path) = path else {
        return Ok(RuntimeValidationPathInspection {
            exists: false,
            is_file: false,
        });
    };

    let resolved = resolve_runtime_path(root, path);
    let metadata = fs::metadata(&resolved);
    match metadata {
        Ok(metadata) => Ok(RuntimeValidationPathInspection {
            exists: true,
            is_file: metadata.is_file(),
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(RuntimeValidationPathInspection {
            exists: false,
            is_file: false,
        }),
        Err(error) => Err(error.into()),
    }
}

fn llama_runtime_validation_missing_inputs(
    executable_path_present: bool,
    model_path_present: bool,
    executable_metadata: &RuntimeValidationPathInspection,
    model_metadata: &RuntimeValidationPathInspection,
    model_extension_valid: bool,
) -> Vec<String> {
    let mut missing_inputs = Vec::new();
    if !executable_path_present {
        missing_inputs.push("executable_path".to_string());
    } else if !executable_metadata.exists {
        missing_inputs.push("executable_exists".to_string());
    } else if !executable_metadata.is_file {
        missing_inputs.push("executable_is_file".to_string());
    }

    if !model_path_present {
        missing_inputs.push("model_path".to_string());
    } else if !model_metadata.exists {
        missing_inputs.push("model_exists".to_string());
    } else if !model_metadata.is_file {
        missing_inputs.push("model_is_file".to_string());
    } else if !model_extension_valid {
        missing_inputs.push("model_extension_valid".to_string());
    }

    missing_inputs
}

fn llama_runtime_validation_reasons(
    status: &LocalRuntimeValidationStatus,
    adapter_contract_preview: &LocalRuntimeAdapterContractPreview,
    executable_path_present: bool,
    model_path_present: bool,
    executable_metadata: &RuntimeValidationPathInspection,
    model_metadata: &RuntimeValidationPathInspection,
    model_extension_valid: bool,
    safe_executable_file_name: Option<&str>,
    safe_model_file_name: Option<&str>,
) -> Vec<String> {
    let mut reasons = adapter_contract_preview.contract_reasons.clone();
    push_unique_text(
        &mut reasons,
        "This is a preview-only llama.cpp runtime validation; no binary was probed and no model was loaded.",
    );
    push_unique_text(
        &mut reasons,
        &format!(
            "Adapter contract status: {}.",
            validation_status_label(&adapter_contract_preview.status)
        ),
    );
    push_unique_text(
        &mut reasons,
        &format!(
            "Executable path present: {}.",
            yes_no_text(executable_path_present)
        ),
    );
    push_unique_text(
        &mut reasons,
        &format!("Executable exists: {}.", yes_no_text(executable_metadata.exists)),
    );
    push_unique_text(
        &mut reasons,
        &format!("Executable is file: {}.", yes_no_text(executable_metadata.is_file)),
    );
    push_unique_text(
        &mut reasons,
        &format!("Model path present: {}.", yes_no_text(model_path_present)),
    );
    push_unique_text(
        &mut reasons,
        &format!("Model exists: {}.", yes_no_text(model_metadata.exists)),
    );
    push_unique_text(
        &mut reasons,
        &format!("Model is file: {}.", yes_no_text(model_metadata.is_file)),
    );
    push_unique_text(
        &mut reasons,
        &format!("Model extension valid: {}.", yes_no_text(model_extension_valid)),
    );
    if let Some(file_name) = safe_executable_file_name {
        push_unique_text(&mut reasons, &format!("Safe executable file name: {file_name}."));
    }
    if let Some(file_name) = safe_model_file_name {
        push_unique_text(&mut reasons, &format!("Safe model file name: {file_name}."));
    }
    if matches!(status, LocalRuntimeValidationStatus::ValidationReadyLater) {
        push_unique_text(
            &mut reasons,
            "The llama.cpp runtime validation preview is ready later.",
        );
    }
    reasons
}

fn llama_runtime_validation_next_required_actions(
    status: &LocalRuntimeValidationStatus,
    adapter_contract_preview: &LocalRuntimeAdapterContractPreview,
    executable_path_present: bool,
    model_path_present: bool,
    executable_exists: bool,
    model_exists: bool,
    executable_is_file: bool,
    model_is_file: bool,
    executable_metadata: &RuntimeValidationPathInspection,
    model_metadata: &RuntimeValidationPathInspection,
    model_extension_valid: bool,
) -> Vec<String> {
    let mut next_required_actions = Vec::new();
    if !executable_path_present {
        push_unique_text(
            &mut next_required_actions,
            "Provide an executable_path for the future llama.cpp runtime validation.",
        );
    } else if !executable_metadata.exists {
        push_unique_text(
            &mut next_required_actions,
            "Place the executable file at the configured path before validation.",
        );
    } else if !executable_metadata.is_file {
        push_unique_text(
            &mut next_required_actions,
            "Point executable_path at a file, not a directory.",
        );
    }

    if !model_path_present {
        push_unique_text(
            &mut next_required_actions,
            "Provide a model_path for the future llama.cpp runtime validation.",
        );
    } else if !model_metadata.exists {
        push_unique_text(
            &mut next_required_actions,
            "Place the model file at the configured path before validation.",
        );
    } else if !model_metadata.is_file {
        push_unique_text(
            &mut next_required_actions,
            "Point model_path at a file, not a directory.",
        );
    } else if !model_extension_valid {
        push_unique_text(
            &mut next_required_actions,
            "Use a .gguf model file for the future runtime.",
        );
    }

    match status {
        LocalRuntimeValidationStatus::Blocked => {
            if executable_path_present
                && model_path_present
                && executable_exists
                && model_exists
                && executable_is_file
                && model_is_file
                && model_extension_valid
                && matches!(adapter_contract_preview.status, LocalRuntimeAdapterContractStatus::Blocked)
            {
                push_unique_text(
                    &mut next_required_actions,
                    "Review the adapter metadata before accepting validation.",
                );
            }
        }
        LocalRuntimeValidationStatus::NeedsReview => {
            push_unique_text(
                &mut next_required_actions,
                "Review the adapter metadata before accepting validation.",
            );
        }
        LocalRuntimeValidationStatus::ValidationReadyLater => {
            push_unique_text(
                &mut next_required_actions,
                "Implement the future llama.cpp runtime validation later without changing this preview.",
            );
        }
    }
    next_required_actions
}

fn llama_runtime_validation_summary(
    status: &LocalRuntimeValidationStatus,
    adapter_contract_preview: &LocalRuntimeAdapterContractPreview,
    executable_path_present: bool,
    model_path_present: bool,
    executable_metadata: &RuntimeValidationPathInspection,
    model_metadata: &RuntimeValidationPathInspection,
    model_extension_valid: bool,
) -> String {
    match status {
        LocalRuntimeValidationStatus::Blocked => {
            if !executable_path_present {
                if !model_path_present {
                    "Llama runtime validation preview is blocked until executable_path and model_path are provided.".to_string()
                } else {
                    "Llama runtime validation preview is blocked until executable_path is provided.".to_string()
                }
            } else if !model_path_present {
                "Llama runtime validation preview is blocked until model_path is provided.".to_string()
            } else if !executable_metadata.exists {
                "Llama runtime validation preview is blocked until the executable file exists.".to_string()
            } else if !model_metadata.exists {
                "Llama runtime validation preview is blocked until the model file exists.".to_string()
            } else if !executable_metadata.is_file {
                "Llama runtime validation preview is blocked until executable_path points to a file.".to_string()
            } else if !model_metadata.is_file {
                "Llama runtime validation preview is blocked until model_path points to a file.".to_string()
            } else if !model_extension_valid {
                "Llama runtime validation preview is blocked until the model file uses a .gguf extension.".to_string()
            } else if matches!(
                adapter_contract_preview.status,
                LocalRuntimeAdapterContractStatus::Blocked
            ) {
                "Llama runtime validation preview is blocked because the adapter contract preview is blocked.".to_string()
            } else {
                "Llama runtime validation preview is blocked.".to_string()
            }
        }
        LocalRuntimeValidationStatus::NeedsReview => {
            "Llama runtime validation preview needs review because the adapter contract preview still needs review."
                .to_string()
        }
        LocalRuntimeValidationStatus::ValidationReadyLater => {
            "The llama.cpp runtime validation preview is ready later: the adapter contract is ready later, both files exist and are files, and the model file uses a .gguf extension."
                .to_string()
        }
    }
}

fn llama_runtime_probe_readiness_required_inputs() -> Vec<String> {
    vec![
        "runtime_validation_ready_later".to_string(),
        "probe_consent".to_string(),
    ]
}

fn llama_runtime_probe_readiness_missing_inputs(
    validation_status: &LocalRuntimeValidationStatus,
    probe_consent: bool,
) -> Vec<String> {
    let mut missing_inputs = Vec::new();
    if !matches!(
        validation_status,
        LocalRuntimeValidationStatus::ValidationReadyLater
    ) {
        missing_inputs.push("runtime_validation_ready_later".to_string());
    } else if !probe_consent {
        missing_inputs.push("probe_consent".to_string());
    }
    missing_inputs
}

fn llama_runtime_probe_readiness_reasons(
    validation_preview: &LocalRuntimeValidationPreview,
    probe_consent: bool,
    status: &LocalRuntimeProbeReadinessStatus,
) -> Vec<String> {
    let mut reasons = validation_preview.validation_reasons.clone();
    push_unique_text(
        &mut reasons,
        &format!("Validation status: {:?}.", validation_preview.status),
    );
    push_unique_text(
        &mut reasons,
        &format!(
            "Adapter contract status: {:?}.",
            validation_preview.adapter_contract_status
        ),
    );
    push_unique_text(
        &mut reasons,
        &format!("Probe consent: {}.", yes_no_text(probe_consent)),
    );
    if let Some(file_name) = validation_preview.safe_executable_file_name.as_deref() {
        push_unique_text(&mut reasons, &format!("Safe executable file name: {file_name}."));
    }
    if let Some(file_name) = validation_preview.safe_model_file_name.as_deref() {
        push_unique_text(&mut reasons, &format!("Safe model file name: {file_name}."));
    }
    match status {
        LocalRuntimeProbeReadinessStatus::Blocked => {
            push_unique_text(
                &mut reasons,
                "The llama.cpp probe-readiness preview is blocked until the runtime validation preview is ready later and probe consent is given.",
            );
        }
        LocalRuntimeProbeReadinessStatus::NeedsReview => {
            push_unique_text(
                &mut reasons,
                "The llama.cpp probe-readiness preview needs review because the runtime validation preview still needs review.",
            );
        }
        LocalRuntimeProbeReadinessStatus::ProbeReadyLater => {
            push_unique_text(
                &mut reasons,
                "The llama.cpp probe-readiness preview is ready later and probe consent is true.",
            );
        }
    }
    reasons
}

fn llama_runtime_probe_readiness_next_required_actions(
    status: &LocalRuntimeProbeReadinessStatus,
    validation_preview: &LocalRuntimeValidationPreview,
    probe_consent: bool,
) -> Vec<String> {
    let mut next_required_actions = validation_preview.next_required_actions.clone();
    match status {
        LocalRuntimeProbeReadinessStatus::Blocked => {
            if !matches!(
                validation_preview.status,
                LocalRuntimeValidationStatus::ValidationReadyLater
            ) {
                push_unique_text(
                    &mut next_required_actions,
                    "Bring the llama.cpp runtime validation preview to validation_ready_later first.",
                );
            } else if !probe_consent {
                push_unique_text(
                    &mut next_required_actions,
                    "Confirm probe consent before a future binary probe can proceed.",
                );
            }
        }
        LocalRuntimeProbeReadinessStatus::NeedsReview => {
            push_unique_text(
                &mut next_required_actions,
                "Review the llama.cpp runtime validation preview before checking probe readiness again.",
            );
        }
        LocalRuntimeProbeReadinessStatus::ProbeReadyLater => {
            push_unique_text(
                &mut next_required_actions,
                "A future binary probe can be added later when execution is enabled.",
            );
        }
    }
    next_required_actions
}

fn llama_runtime_probe_readiness_summary(
    status: &LocalRuntimeProbeReadinessStatus,
    validation_preview: &LocalRuntimeValidationPreview,
    probe_consent: bool,
) -> String {
    match status {
        LocalRuntimeProbeReadinessStatus::Blocked => {
            if !matches!(
                validation_preview.status,
                LocalRuntimeValidationStatus::ValidationReadyLater
            ) {
                "Probe-readiness preview is blocked until the llama.cpp runtime validation preview is ready later."
                    .to_string()
            } else if !probe_consent {
                "Probe-readiness preview is blocked until probe consent is given.".to_string()
            } else {
                "Probe-readiness preview is blocked.".to_string()
            }
        }
        LocalRuntimeProbeReadinessStatus::NeedsReview => {
            "Probe-readiness preview needs review because the llama.cpp runtime validation preview still needs review."
                .to_string()
        }
        LocalRuntimeProbeReadinessStatus::ProbeReadyLater => {
            "Probe-readiness preview is ready later: the llama.cpp runtime validation preview is ready later and probe consent is true."
                .to_string()
        }
    }
}

fn llama_runtime_version_probe_required_actions(
    readiness_status: &LocalRuntimeProbeReadinessStatus,
    allow_probe_execution: bool,
    blockers: &[LocalRuntimeProbeWarning],
) -> Vec<String> {
    let mut next_required_actions = Vec::new();
    match readiness_status {
        LocalRuntimeProbeReadinessStatus::Blocked => {
            push_unique_text(
                &mut next_required_actions,
                "Review the llama.cpp probe-readiness preview before running a version-only probe.",
            );
        }
        LocalRuntimeProbeReadinessStatus::NeedsReview => {
            push_unique_text(
                &mut next_required_actions,
                "Review the llama.cpp probe-readiness preview before trying the version probe again.",
            );
        }
        LocalRuntimeProbeReadinessStatus::ProbeReadyLater => {
            if !allow_probe_execution {
                push_unique_text(
                    &mut next_required_actions,
                    "Enable version-probe execution to run the configured llama.cpp binary with --version only.",
                );
            } else if blockers.iter().any(|blocker| blocker.kind == "probe_start_failed") {
                push_unique_text(
                    &mut next_required_actions,
                    "Review the configured executable before trying the version probe again.",
                );
            } else {
                push_unique_text(
                    &mut next_required_actions,
                    "Review the bounded stdout and stderr previews before trying the version probe again later.",
                );
            }
        }
    }
    next_required_actions
}

fn llama_runtime_version_probe_summary(
    status: &LocalRuntimeVersionProbeStatus,
    readiness_status: &LocalRuntimeProbeReadinessStatus,
    allow_probe_execution: bool,
    safe_executable_file_name: Option<&str>,
    safe_model_file_name: Option<&str>,
) -> String {
    let executable_text = safe_executable_file_name
        .map(|file_name| format!(" Configured executable file: {file_name}."))
        .unwrap_or_default();
    let model_text = safe_model_file_name
        .map(|file_name| format!(" Configured model file: {file_name}."))
        .unwrap_or_default();
    match status {
        LocalRuntimeVersionProbeStatus::Blocked => {
            if !matches!(readiness_status, LocalRuntimeProbeReadinessStatus::ProbeReadyLater) {
                format!(
                    "The llama.cpp version probe preview is blocked until the probe-readiness preview is ready later.{executable_text}{model_text}"
                )
            } else if !allow_probe_execution {
                format!(
                    "The llama.cpp version probe preview is blocked until probe execution is allowed.{executable_text}{model_text}"
                )
            } else {
                format!("The llama.cpp version probe preview is blocked.{executable_text}{model_text}")
            }
        }
        LocalRuntimeVersionProbeStatus::ProbeSucceeded => {
            format!(
                "The llama.cpp version probe preview succeeded after running the configured executable with --version only.{executable_text}{model_text}"
            )
        }
        LocalRuntimeVersionProbeStatus::ProbeFailed => {
            format!(
                "The llama.cpp version probe preview failed after running the configured executable with --version only.{executable_text}{model_text}"
            )
        }
        LocalRuntimeVersionProbeStatus::TimedOut => {
            format!(
                "The llama.cpp version probe preview timed out after running the configured executable with --version only.{executable_text}{model_text}"
            )
        }
    }
}

fn llama_runtime_capability_status(
    version_probe_status: &LocalRuntimeVersionProbeStatus,
) -> LocalRuntimeCapabilityStatus {
    match version_probe_status {
        LocalRuntimeVersionProbeStatus::Blocked => LocalRuntimeCapabilityStatus::Blocked,
        LocalRuntimeVersionProbeStatus::TimedOut | LocalRuntimeVersionProbeStatus::ProbeFailed => {
            LocalRuntimeCapabilityStatus::NeedsReview
        }
        LocalRuntimeVersionProbeStatus::ProbeSucceeded => {
            LocalRuntimeCapabilityStatus::CapabilityReadyLater
        }
    }
}

fn extract_obvious_version_text_from_preview(preview: &str) -> Option<String> {
    for line in preview.lines() {
        let lower = line.to_ascii_lowercase();
        if !lower.contains("version") {
            continue;
        }
        for token in line.split_whitespace() {
            let token = token.trim_matches(|value: char| {
                !value.is_ascii_alphanumeric() && value != '.' && value != '-' && value != '+'
            });
            if token.is_empty() {
                continue;
            }
            let token = token.strip_prefix('v').unwrap_or(token);
            let normalized = token.trim_matches(|value: char| value == '.' || value == '-' || value == '+');
            if normalized.chars().any(|value| value.is_ascii_digit())
                && normalized
                    .chars()
                    .all(|value| value.is_ascii_alphanumeric() || matches!(value, '.' | '-' | '+'))
            {
                return Some(normalized.to_string());
            }
        }
    }
    None
}

fn infer_version_text_from_probe_result(
    stdout_preview: &str,
    stderr_preview: &str,
) -> Option<String> {
    extract_obvious_version_text_from_preview(stdout_preview)
        .or_else(|| extract_obvious_version_text_from_preview(stderr_preview))
}

fn llama_runtime_capability_reasons(
    status: &LocalRuntimeCapabilityStatus,
    version_probe_result: &LocalRuntimeVersionProbePreview,
    inferred_version_text: Option<&str>,
) -> Vec<String> {
    let mut reasons = Vec::new();
    match status {
        LocalRuntimeCapabilityStatus::Blocked => {
            push_unique_text(
                &mut reasons,
                "The llama.cpp capability preview is blocked until the wrapped version probe is ready later.",
            );
            push_unique_text(
                &mut reasons,
                "This capability preview does not add any new execution path beyond the wrapped version probe.",
            );
        }
        LocalRuntimeCapabilityStatus::NeedsReview => {
            if matches!(version_probe_result.status, LocalRuntimeVersionProbeStatus::TimedOut) {
                push_unique_text(
                    &mut reasons,
                    "The wrapped version probe timed out and the capability preview needs review.",
                );
            } else {
                push_unique_text(
                    &mut reasons,
                    "The wrapped version probe failed and the capability preview needs review.",
                );
            }
        }
        LocalRuntimeCapabilityStatus::CapabilityReadyLater => {
            push_unique_text(
                &mut reasons,
                "The wrapped version probe succeeded, so the capability preview is ready later.",
            );
            if let Some(version_text) = inferred_version_text {
                push_unique_text(
                    &mut reasons,
                    &format!("Detected obvious version text: {version_text}."),
                );
            } else {
                push_unique_text(
                    &mut reasons,
                    "No obvious version text was found in the sanitized version-probe output.",
                );
            }
        }
    }
    if let Some(model_family) = version_probe_result.normalized_model_family.as_deref() {
        push_unique_text(
            &mut reasons,
            &format!(
                "Configured model family context remains diagnostic only: {model_family}."
            ),
        );
    }
    reasons
}

fn llama_runtime_capability_next_required_actions(
    status: &LocalRuntimeCapabilityStatus,
    version_probe_result: &LocalRuntimeVersionProbePreview,
) -> Vec<String> {
    let mut next_required_actions = Vec::new();
    match status {
        LocalRuntimeCapabilityStatus::Blocked => {
            if !matches!(
                version_probe_result.probe_readiness_status,
                LocalRuntimeProbeReadinessStatus::ProbeReadyLater
            ) {
                push_unique_text(
                    &mut next_required_actions,
                    "Review the version-probe readiness preview before trying capability preview again.",
                );
            } else if !version_probe_result.allow_probe_execution {
                push_unique_text(
                    &mut next_required_actions,
                    "Enable version-probe execution before trying the capability preview again.",
                );
            } else {
                push_unique_text(
                    &mut next_required_actions,
                    "Review the version-probe blockers before trying the capability preview again.",
                );
            }
        }
        LocalRuntimeCapabilityStatus::NeedsReview => {
            push_unique_text(
                &mut next_required_actions,
                "Review the bounded version-probe output before using the capability preview later.",
            );
        }
        LocalRuntimeCapabilityStatus::CapabilityReadyLater => {
            push_unique_text(
                &mut next_required_actions,
                "Review the bounded version-probe output before integrating runtime capability later.",
            );
        }
    }
    next_required_actions
}

fn llama_runtime_capability_summary(
    status: &LocalRuntimeCapabilityStatus,
    version_probe_result: &LocalRuntimeVersionProbePreview,
    inferred_version_text: Option<&str>,
) -> String {
    let executable_text = version_probe_result
        .safe_executable_file_name
        .as_deref()
        .map(|file_name| format!(" Configured executable file: {file_name}."))
        .unwrap_or_default();
    let model_text = version_probe_result
        .safe_model_file_name
        .as_deref()
        .map(|file_name| format!(" Configured model file: {file_name}."))
        .unwrap_or_default();
    match status {
        LocalRuntimeCapabilityStatus::Blocked => {
            format!(
                "The llama.cpp capability preview is blocked until the wrapped version probe is ready later.{executable_text}{model_text}"
            )
        }
        LocalRuntimeCapabilityStatus::NeedsReview => {
            if matches!(version_probe_result.status, LocalRuntimeVersionProbeStatus::TimedOut) {
                format!(
                    "The llama.cpp capability preview needs review because the wrapped version probe timed out.{executable_text}{model_text}"
                )
            } else {
                format!(
                    "The llama.cpp capability preview needs review because the wrapped version probe failed.{executable_text}{model_text}"
                )
            }
        }
        LocalRuntimeCapabilityStatus::CapabilityReadyLater => {
            if let Some(version_text) = inferred_version_text {
                format!(
                    "The llama.cpp capability preview is ready later because the wrapped version probe succeeded. Inferred version text: {version_text}.{executable_text}{model_text}"
                )
            } else {
                format!(
                    "The llama.cpp capability preview is ready later because the wrapped version probe succeeded, but no obvious version text was found.{executable_text}{model_text}"
                )
            }
        }
    }
}

fn normalize_smoke_readiness_prompt(
    prompt: Option<&str>,
    warnings: &mut Vec<LocalRuntimeProbeWarning>,
) -> Option<String> {
    let prompt = normalize_optional_text(prompt.map(|value| value.to_string()));
    let Some(prompt) = prompt else {
        return None;
    };
    if prompt.chars().count() > LOCAL_RUNTIME_SMOKE_PROMPT_LIMIT {
        push_probe_warning(
            warnings,
            "prompt_truncated",
            "The smoke-readiness prompt was truncated to keep the preview compact.",
        );
        Some(prompt.chars().take(LOCAL_RUNTIME_SMOKE_PROMPT_LIMIT).collect::<String>())
    } else {
        Some(prompt)
    }
}

fn llama_runtime_smoke_readiness_status(
    capability_status: &LocalRuntimeCapabilityStatus,
    smoke_consent: bool,
    diagnostic_prompt_present: bool,
) -> LocalRuntimeSmokeReadinessStatus {
    match capability_status {
        LocalRuntimeCapabilityStatus::Blocked => LocalRuntimeSmokeReadinessStatus::Blocked,
        LocalRuntimeCapabilityStatus::NeedsReview => LocalRuntimeSmokeReadinessStatus::NeedsReview,
        LocalRuntimeCapabilityStatus::CapabilityReadyLater => {
            if smoke_consent && diagnostic_prompt_present {
                LocalRuntimeSmokeReadinessStatus::SmokeReadyLater
            } else {
                LocalRuntimeSmokeReadinessStatus::Blocked
            }
        }
    }
}

fn llama_runtime_smoke_readiness_required_inputs() -> Vec<String> {
    vec![
        "capability_ready_later".to_string(),
        "smoke_consent".to_string(),
        "diagnostic_prompt".to_string(),
    ]
}

fn llama_runtime_smoke_readiness_missing_inputs(
    capability_status: &LocalRuntimeCapabilityStatus,
    smoke_consent: bool,
    diagnostic_prompt_present: bool,
) -> Vec<String> {
    let mut missing_inputs = Vec::new();
    if !matches!(capability_status, LocalRuntimeCapabilityStatus::CapabilityReadyLater) {
        missing_inputs.push("capability_ready_later".to_string());
    } else if !smoke_consent {
        missing_inputs.push("smoke_consent".to_string());
    } else if !diagnostic_prompt_present {
        missing_inputs.push("diagnostic_prompt".to_string());
    }
    missing_inputs
}

fn llama_runtime_smoke_readiness_reasons(
    status: &LocalRuntimeSmokeReadinessStatus,
    capability_preview: &LocalRuntimeCapabilityPreview,
    smoke_consent: bool,
    diagnostic_prompt_char_count: usize,
    max_output_tokens: u32,
    timeout_ms: u64,
) -> Vec<String> {
    let mut reasons = vec![
        format!("Capability status: {:?}", capability_preview.status),
        format!("Version probe status: {:?}", capability_preview.version_probe_status),
        format!("Probe readiness status: {:?}", capability_preview.probe_readiness_status),
        format!("Validation status: {:?}", capability_preview.validation_status),
        format!(
            "Adapter contract status: {:?}",
            capability_preview.adapter_contract_status
        ),
        format!("Adapter kind: {:?}", capability_preview.adapter_kind),
        format!("Probe consent: {}", capability_preview.probe_consent),
        format!("Allow probe execution: {}", capability_preview.allow_probe_execution),
        format!("Smoke consent: {}", smoke_consent),
        format!("Diagnostic prompt char count: {diagnostic_prompt_char_count}"),
        format!("Max output tokens: {max_output_tokens}"),
        format!("Timeout ms: {timeout_ms}"),
        format!(
            "Normalized diagnostic prompt was {}.",
            if diagnostic_prompt_char_count > 0 {
                "provided"
            } else {
                "missing"
            }
        ),
    ];
    reasons.extend(capability_preview.capability_reasons.iter().cloned());
    match status {
        LocalRuntimeSmokeReadinessStatus::Blocked => {
            push_unique_text(
                &mut reasons,
                "The smoke readiness preview is blocked until the capability preview is ready later, smoke consent is given, and a diagnostic prompt is provided.",
            );
        }
        LocalRuntimeSmokeReadinessStatus::NeedsReview => {
            push_unique_text(
                &mut reasons,
                "The capability preview still needs review before smoke readiness can be accepted.",
            );
        }
        LocalRuntimeSmokeReadinessStatus::SmokeReadyLater => {
            push_unique_text(
                &mut reasons,
                "The capability preview is ready later, smoke consent is true, and the diagnostic prompt is bounded for a future smoke inference preview.",
            );
        }
    }
    reasons
}

fn llama_runtime_smoke_readiness_next_required_actions(
    status: &LocalRuntimeSmokeReadinessStatus,
    capability_preview: &LocalRuntimeCapabilityPreview,
    smoke_consent: bool,
    diagnostic_prompt_present: bool,
) -> Vec<String> {
    let mut next_required_actions = capability_preview.next_required_actions.clone();
    match status {
        LocalRuntimeSmokeReadinessStatus::Blocked => {
            if !matches!(
                capability_preview.status,
                LocalRuntimeCapabilityStatus::CapabilityReadyLater
            ) {
                push_unique_text(
                    &mut next_required_actions,
                    "Review the capability preview before trying smoke readiness again.",
                );
            } else if !smoke_consent {
                push_unique_text(
                    &mut next_required_actions,
                    "Confirm smoke consent before a future diagnostic smoke inference can be considered.",
                );
            } else if !diagnostic_prompt_present {
                push_unique_text(
                    &mut next_required_actions,
                    "Provide a diagnostic prompt before a future diagnostic smoke inference can be considered.",
                );
            }
        }
        LocalRuntimeSmokeReadinessStatus::NeedsReview => {
            push_unique_text(
                &mut next_required_actions,
                "Review the capability preview before checking smoke readiness again.",
            );
        }
        LocalRuntimeSmokeReadinessStatus::SmokeReadyLater => {
            push_unique_text(
                &mut next_required_actions,
                "A future diagnostic smoke inference preview can be added later without changing this preview.",
            );
        }
    }
    next_required_actions
}

fn llama_runtime_smoke_readiness_summary(
    status: &LocalRuntimeSmokeReadinessStatus,
    capability_preview: &LocalRuntimeCapabilityPreview,
    smoke_consent: bool,
    diagnostic_prompt_char_count: usize,
    max_output_tokens: u32,
    timeout_ms: u64,
) -> String {
    let executable_text = capability_preview
        .safe_executable_file_name
        .as_deref()
        .map(|file_name| format!(" Configured executable file: {file_name}."))
        .unwrap_or_default();
    let model_text = capability_preview
        .safe_model_file_name
        .as_deref()
        .map(|file_name| format!(" Configured model file: {file_name}."))
        .unwrap_or_default();
    match status {
        LocalRuntimeSmokeReadinessStatus::Blocked => {
            if !matches!(
                capability_preview.status,
                LocalRuntimeCapabilityStatus::CapabilityReadyLater
            ) {
                format!(
                    "Smoke readiness preview is blocked because the capability preview is {:?}.{executable_text}{model_text}",
                    capability_preview.status
                )
            } else if !smoke_consent {
                format!(
                    "Smoke readiness preview is blocked until smoke consent is given.{executable_text}{model_text}"
                )
            } else if diagnostic_prompt_char_count == 0 {
                format!(
                    "Smoke readiness preview is blocked until a diagnostic prompt is provided.{executable_text}{model_text}"
                )
            } else {
                format!(
                    "Smoke readiness preview is blocked.{executable_text}{model_text}"
                )
            }
        }
        LocalRuntimeSmokeReadinessStatus::NeedsReview => {
            format!(
                "Smoke readiness preview needs review because the capability preview still needs review.{executable_text}{model_text}"
            )
        }
        LocalRuntimeSmokeReadinessStatus::SmokeReadyLater => {
            format!(
                "The smoke readiness preview is ready later: capability is ready later, smoke consent is true, a bounded diagnostic prompt is present, max output tokens is {max_output_tokens}, and timeout ms is {timeout_ms}.{executable_text}{model_text}"
            )
        }
    }
}

fn llama_runtime_smoke_execution_plan_status(
    smoke_readiness_status: &LocalRuntimeSmokeReadinessStatus,
) -> LocalRuntimeSmokeExecutionPlanStatus {
    match smoke_readiness_status {
        LocalRuntimeSmokeReadinessStatus::Blocked => LocalRuntimeSmokeExecutionPlanStatus::Blocked,
        LocalRuntimeSmokeReadinessStatus::NeedsReview => {
            LocalRuntimeSmokeExecutionPlanStatus::NeedsReview
        }
        LocalRuntimeSmokeReadinessStatus::SmokeReadyLater => {
            LocalRuntimeSmokeExecutionPlanStatus::PlanReadyLater
        }
    }
}

fn llama_runtime_smoke_execution_plan_required_inputs(
    smoke_readiness_required_inputs: &[String],
) -> Vec<String> {
    smoke_readiness_required_inputs.to_vec()
}

fn llama_runtime_smoke_execution_plan_missing_inputs(
    smoke_readiness_status: &LocalRuntimeSmokeReadinessStatus,
    smoke_readiness_missing_inputs: &[String],
) -> Vec<String> {
    if matches!(
        smoke_readiness_status,
        LocalRuntimeSmokeReadinessStatus::SmokeReadyLater
    ) {
        Vec::new()
    } else {
        smoke_readiness_missing_inputs.to_vec()
    }
}

fn llama_runtime_smoke_execution_plan_planned_inputs() -> Vec<String> {
    vec![
        "llama_runtime_executable".to_string(),
        "gguf_model_file".to_string(),
        "diagnostic_prompt".to_string(),
        "max_output_tokens".to_string(),
        "timeout_ms".to_string(),
    ]
}

fn llama_runtime_smoke_execution_plan_planned_safe_arguments() -> Vec<String> {
    vec![
        "<executable>".to_string(),
        "-m".to_string(),
        "<model-file>".to_string(),
        "-p".to_string(),
        "<diagnostic-prompt>".to_string(),
        "-n".to_string(),
        "<max-output-tokens>".to_string(),
    ]
}

fn llama_runtime_smoke_execution_plan_planned_outputs() -> Vec<String> {
    vec![
        "diagnostic stdout preview".to_string(),
        "diagnostic stderr preview".to_string(),
        "exit status".to_string(),
        "duration_ms".to_string(),
        "diagnostic-only status".to_string(),
        "no artifact output".to_string(),
        "no answer output".to_string(),
    ]
}

fn llama_runtime_smoke_execution_plan_plan_reasons(
    status: &LocalRuntimeSmokeExecutionPlanStatus,
    smoke_readiness_preview: &LocalRuntimeSmokeReadinessPreview,
) -> Vec<String> {
    let mut reasons = vec![
        format!("Smoke readiness status: {:?}", smoke_readiness_preview.status),
        format!("Capability status: {:?}", smoke_readiness_preview.capability_status),
        format!("Version probe status: {:?}", smoke_readiness_preview.version_probe_status),
        format!(
            "Planned operation: future_diagnostic_smoke_inference"
        ),
        "Planned inputs are logical and path-free.".to_string(),
        "Planned safe arguments use placeholders only.".to_string(),
        "Planned outputs are diagnostic-only and do not include answer or artifact output.".to_string(),
    ];
    reasons.extend(smoke_readiness_preview.readiness_reasons.iter().cloned());
    match status {
        LocalRuntimeSmokeExecutionPlanStatus::Blocked => {
            push_unique_text(
                &mut reasons,
                "The smoke execution plan is blocked until smoke readiness is ready later.",
            );
        }
        LocalRuntimeSmokeExecutionPlanStatus::NeedsReview => {
            push_unique_text(
                &mut reasons,
                "The smoke readiness preview still needs review before a future execution plan can be accepted.",
            );
        }
        LocalRuntimeSmokeExecutionPlanStatus::PlanReadyLater => {
            push_unique_text(
                &mut reasons,
                "A future diagnostic smoke inference execution plan can be considered later without changing this preview.",
            );
        }
    }
    reasons
}

fn llama_runtime_smoke_execution_plan_next_required_actions(
    status: &LocalRuntimeSmokeExecutionPlanStatus,
    smoke_readiness_preview: &LocalRuntimeSmokeReadinessPreview,
) -> Vec<String> {
    let mut next_required_actions = smoke_readiness_preview.next_required_actions.clone();
    match status {
        LocalRuntimeSmokeExecutionPlanStatus::Blocked => {
            push_unique_text(
                &mut next_required_actions,
                "Review smoke readiness before trying the smoke execution plan again.",
            );
        }
        LocalRuntimeSmokeExecutionPlanStatus::NeedsReview => {
            push_unique_text(
                &mut next_required_actions,
                "Review the smoke readiness preview before checking the execution plan again.",
            );
        }
        LocalRuntimeSmokeExecutionPlanStatus::PlanReadyLater => {
            push_unique_text(
                &mut next_required_actions,
                "A future diagnostic smoke inference execution plan can be added later without changing this preview.",
            );
        }
    }
    next_required_actions
}

fn llama_runtime_smoke_execution_plan_summary(
    status: &LocalRuntimeSmokeExecutionPlanStatus,
    smoke_readiness_preview: &LocalRuntimeSmokeReadinessPreview,
) -> String {
    let executable_text = smoke_readiness_preview
        .safe_executable_file_name
        .as_deref()
        .map(|file_name| format!(" Configured executable file: {file_name}."))
        .unwrap_or_default();
    let model_text = smoke_readiness_preview
        .safe_model_file_name
        .as_deref()
        .map(|file_name| format!(" Configured model file: {file_name}."))
        .unwrap_or_default();
    match status {
        LocalRuntimeSmokeExecutionPlanStatus::Blocked => {
            format!(
                "Smoke execution plan preview is blocked because smoke readiness is {:?}.{executable_text}{model_text}",
                smoke_readiness_preview.status
            )
        }
        LocalRuntimeSmokeExecutionPlanStatus::NeedsReview => {
            format!(
                "Smoke execution plan preview needs review because smoke readiness still needs review.{executable_text}{model_text}"
            )
        }
        LocalRuntimeSmokeExecutionPlanStatus::PlanReadyLater => {
            format!(
                "The smoke execution plan preview is ready later: the readiness preview is ready later, the planned operation is future_diagnostic_smoke_inference, and the plan uses logical placeholders only.{executable_text}{model_text}"
            )
        }
    }
}

pub fn preview_llama_runtime_smoke_execution_plan(
    root: impl Into<PathBuf>,
    request: LocalRuntimeSmokeExecutionPlanPreviewRequest,
) -> AegisResult<LocalRuntimeSmokeExecutionPlanPreview> {
    let root = root.into();
    let LocalRuntimeSmokeExecutionPlanPreviewRequest {
        smoke_readiness_preview_request,
    } = request;
    let smoke_readiness_preview = preview_llama_runtime_smoke_readiness(&root, smoke_readiness_preview_request)?;
    let smoke_readiness_status = smoke_readiness_preview.status.clone();
    let capability_status = smoke_readiness_preview.capability_status.clone();
    let version_probe_status = smoke_readiness_preview.version_probe_status.clone();
    let probe_readiness_status = smoke_readiness_preview.probe_readiness_status.clone();
    let validation_status = smoke_readiness_preview.validation_status.clone();
    let adapter_contract_status = smoke_readiness_preview.adapter_contract_status.clone();
    let adapter_kind = smoke_readiness_preview.adapter_kind.clone();
    let normalized_model_family = smoke_readiness_preview.normalized_model_family.clone();
    let normalized_model_format = smoke_readiness_preview.normalized_model_format.clone();
    let safe_executable_file_name = smoke_readiness_preview.safe_executable_file_name.clone();
    let safe_model_file_name = smoke_readiness_preview.safe_model_file_name.clone();
    let probe_consent = smoke_readiness_preview.probe_consent;
    let allow_probe_execution = smoke_readiness_preview.allow_probe_execution;
    let smoke_consent = smoke_readiness_preview.smoke_consent;
    let normalized_diagnostic_prompt = smoke_readiness_preview.normalized_diagnostic_prompt.clone();
    let diagnostic_prompt_char_count = smoke_readiness_preview.diagnostic_prompt_char_count;
    let max_output_tokens = smoke_readiness_preview.max_output_tokens;
    let timeout_ms = smoke_readiness_preview.timeout_ms;
    let status = llama_runtime_smoke_execution_plan_status(&smoke_readiness_status);
    let required_inputs =
        llama_runtime_smoke_execution_plan_required_inputs(&smoke_readiness_preview.required_inputs);
    let missing_inputs = llama_runtime_smoke_execution_plan_missing_inputs(
        &smoke_readiness_status,
        &smoke_readiness_preview.missing_inputs,
    );
    let plan_reasons =
        llama_runtime_smoke_execution_plan_plan_reasons(&status, &smoke_readiness_preview);
    let next_required_actions =
        llama_runtime_smoke_execution_plan_next_required_actions(&status, &smoke_readiness_preview);
    let planned_operation = "future_diagnostic_smoke_inference".to_string();
    let planned_inputs = llama_runtime_smoke_execution_plan_planned_inputs();
    let planned_safe_arguments = llama_runtime_smoke_execution_plan_planned_safe_arguments();
    let planned_outputs = llama_runtime_smoke_execution_plan_planned_outputs();
    let summary = llama_runtime_smoke_execution_plan_summary(&status, &smoke_readiness_preview);
    let mut blockers = smoke_readiness_preview.blockers.clone();
    let mut warnings = smoke_readiness_preview.warnings.clone();
    push_probe_warning(
        &mut warnings,
        "boundary",
        "This is a smoke execution plan preview only; it does not run smoke inference, spawn a process, load or read a model, call an LLM, or persist settings or artifacts.",
    );
    if matches!(status, LocalRuntimeSmokeExecutionPlanStatus::Blocked) {
        push_probe_blocker(
            &mut blockers,
            "smoke_readiness_blocked",
            "Smoke readiness must be ready later before a future smoke execution plan can be considered.",
        );
    }
    if matches!(status, LocalRuntimeSmokeExecutionPlanStatus::NeedsReview) {
        push_probe_warning(
            &mut warnings,
            "needs_review",
            "The smoke readiness preview still needs review before a future smoke execution plan can be accepted.",
        );
    }
    if matches!(status, LocalRuntimeSmokeExecutionPlanStatus::PlanReadyLater) {
        push_probe_warning(
            &mut warnings,
            "plan_ready_later",
            "A future diagnostic smoke inference execution plan can be added later without changing this preview.",
        );
    }

    Ok(LocalRuntimeSmokeExecutionPlanPreview {
        status,
        smoke_readiness_status,
        capability_status,
        version_probe_status,
        probe_readiness_status,
        validation_status,
        adapter_contract_status,
        adapter_kind,
        normalized_model_family,
        normalized_model_format,
        safe_executable_file_name,
        safe_model_file_name,
        probe_consent,
        allow_probe_execution,
        smoke_consent,
        normalized_diagnostic_prompt,
        diagnostic_prompt_char_count,
        max_output_tokens,
        timeout_ms,
        planned_operation,
        planned_inputs,
        planned_safe_arguments,
        planned_outputs,
        required_inputs,
        missing_inputs,
        plan_reasons,
        blockers,
        warnings,
        next_required_actions,
        summary,
        preview_only: true,
        no_process_spawn: true,
        no_smoke_inference_execution: true,
        no_model_file_read: true,
        no_model_load: true,
        no_llm_call: true,
        no_persistence: true,
        no_artifact_write: true,
        no_registry_status_change: true,
        no_audit_write: true,
        diagnostic_only: true,
        not_scholar_chat_answer: true,
        no_answer_generated: true,
        no_grounding_applied: true,
        no_evidence_pack_used: true,
    })
}

fn version_probe_output_redactions(
    executable_path: Option<&str>,
    safe_executable_file_name: Option<&str>,
    model_path: Option<&str>,
    safe_model_file_name: Option<&str>,
) -> Vec<(String, String)> {
    let mut redactions = Vec::new();
    if let (Some(executable_path), Some(safe_executable_file_name)) = (executable_path, safe_executable_file_name) {
        redactions.push((executable_path.to_string(), safe_executable_file_name.to_string()));
    }
    if let (Some(model_path), Some(safe_model_file_name)) = (model_path, safe_model_file_name) {
        redactions.push((model_path.to_string(), safe_model_file_name.to_string()));
    }
    redactions
}

fn validation_status_label(status: &LocalRuntimeAdapterContractStatus) -> &'static str {
    match status {
        LocalRuntimeAdapterContractStatus::Blocked => "blocked",
        LocalRuntimeAdapterContractStatus::NeedsReview => "needs_review",
        LocalRuntimeAdapterContractStatus::ContractReadyLater => "contract_ready_later",
    }
}

fn yes_no_text(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn resolve_runtime_path(root: &Path, path: &str) -> PathBuf {
    let candidate = PathBuf::from(path);
    if candidate.is_absolute() {
        candidate
    } else {
        root.join(candidate)
    }
}

fn has_gguf_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|value| value.eq_ignore_ascii_case("gguf"))
        .unwrap_or(false)
}

fn push_warning(warnings: &mut Vec<LocalModelRuntimeHealthWarning>, message: &str) {
    if !warnings.iter().any(|warning| warning.message == message) {
        warnings.push(LocalModelRuntimeHealthWarning {
            kind: message
                .chars()
                .map(|ch| if ch.is_ascii_alphanumeric() { ch.to_ascii_lowercase() } else { '_' })
                .collect::<String>()
                .replace("__", "_"),
            message: message.to_string(),
        });
    }
}

fn push_blocker(blockers: &mut Vec<LocalRuntimeInvocationBlocker>, kind: &str, message: &str) {
    if !blockers.iter().any(|blocker| blocker.kind == kind && blocker.message == message) {
        blockers.push(LocalRuntimeInvocationBlocker {
            kind: kind.to_string(),
            message: message.to_string(),
        });
    }
}

fn format_runtime_kind(runtime_kind: &LocalModelRuntimeKind) -> &'static str {
    match runtime_kind {
        LocalModelRuntimeKind::LlamaCpp => "llama_cpp",
        LocalModelRuntimeKind::None => "none",
    }
}

const LOCAL_RUNTIME_PROBE_PREVIEW_LIMIT: usize = 1024;
const LOCAL_RUNTIME_PROBE_DEFAULT_TIMEOUT_MS: u64 = 1_500;
const LOCAL_RUNTIME_PROBE_MIN_TIMEOUT_MS: u64 = 250;
const LOCAL_RUNTIME_PROBE_MAX_TIMEOUT_MS: u64 = 5_000;

fn clamp_probe_timeout_ms(timeout_ms: Option<u64>) -> u64 {
    timeout_ms
        .unwrap_or(LOCAL_RUNTIME_PROBE_DEFAULT_TIMEOUT_MS)
        .clamp(LOCAL_RUNTIME_PROBE_MIN_TIMEOUT_MS, LOCAL_RUNTIME_PROBE_MAX_TIMEOUT_MS)
}

fn run_version_probe(
    executable_path: &Path,
    timeout_ms: u64,
) -> AegisResult<LocalRuntimeProbeExecution> {
    let mut child = Command::new(executable_path)
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let stdout_handle = stdout.map(spawn_pipe_reader);
    let stderr_handle = stderr.map(spawn_pipe_reader);
    let started_at = Instant::now();
    let deadline = Duration::from_millis(timeout_ms);
    let mut timed_out = false;

    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if started_at.elapsed() >= deadline {
            timed_out = true;
            let _ = child.kill();
            break child.wait()?;
        }
        thread::sleep(Duration::from_millis(10));
    };

    let stdout = join_pipe_reader(stdout_handle);
    let stderr = join_pipe_reader(stderr_handle);

    Ok(LocalRuntimeProbeExecution {
        exit_code: status.code(),
        duration_ms: bucket_probe_duration_ms(started_at.elapsed().as_millis() as u64),
        stdout: clean_probe_capture(stdout),
        stderr: clean_probe_capture(stderr),
        timed_out,
    })
}

fn spawn_pipe_reader<R>(mut reader: R) -> thread::JoinHandle<String>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut buffer = Vec::new();
        if reader.read_to_end(&mut buffer).is_err() {
            return String::new();
        }
        String::from_utf8_lossy(&buffer).into_owned()
    })
}

fn join_pipe_reader(handle: Option<thread::JoinHandle<String>>) -> String {
    match handle {
        Some(handle) => handle.join().unwrap_or_default(),
        None => String::new(),
    }
}

fn clean_probe_capture(value: String) -> String {
    value.replace("\r\n", "\n").replace('\r', "\n").trim().to_string()
}

fn bucket_probe_duration_ms(value: u64) -> u64 {
    (value / 100) * 100
}

const LOCAL_RUNTIME_SMOKE_PREVIEW_LIMIT: usize = 2048;
const LOCAL_RUNTIME_SMOKE_PROMPT_LIMIT: usize = 256;
const LOCAL_RUNTIME_SMOKE_DEFAULT_TIMEOUT_MS: u64 = 3_000;
const LOCAL_RUNTIME_SMOKE_MIN_TIMEOUT_MS: u64 = 250;
const LOCAL_RUNTIME_SMOKE_MAX_TIMEOUT_MS: u64 = 10_000;
const LOCAL_RUNTIME_SMOKE_DEFAULT_MAX_OUTPUT_TOKENS: u32 = 8;
const LOCAL_RUNTIME_SMOKE_MIN_MAX_OUTPUT_TOKENS: u32 = 1;
const LOCAL_RUNTIME_SMOKE_MAX_MAX_OUTPUT_TOKENS: u32 = 32;

fn clamp_smoke_timeout_ms(timeout_ms: Option<u64>) -> u64 {
    timeout_ms
        .unwrap_or(LOCAL_RUNTIME_SMOKE_DEFAULT_TIMEOUT_MS)
        .clamp(LOCAL_RUNTIME_SMOKE_MIN_TIMEOUT_MS, LOCAL_RUNTIME_SMOKE_MAX_TIMEOUT_MS)
}

fn clamp_smoke_max_output_tokens(max_output_tokens: Option<u32>) -> u32 {
    max_output_tokens
        .unwrap_or(LOCAL_RUNTIME_SMOKE_DEFAULT_MAX_OUTPUT_TOKENS)
        .clamp(
            LOCAL_RUNTIME_SMOKE_MIN_MAX_OUTPUT_TOKENS,
            LOCAL_RUNTIME_SMOKE_MAX_MAX_OUTPUT_TOKENS,
        )
}

fn normalize_smoke_prompt(prompt: Option<&str>, warnings: &mut Vec<LocalRuntimeSmokeInferenceWarning>) -> Option<String> {
    let prompt = normalize_optional_text(prompt.map(|value| value.to_string()));
    let Some(prompt) = prompt else {
        return None;
    };
    if prompt.chars().count() > LOCAL_RUNTIME_SMOKE_PROMPT_LIMIT {
        push_smoke_warning(
            warnings,
            "prompt_truncated",
            "The smoke-test prompt was truncated to keep the preview compact.",
        );
        Some(prompt.chars().take(LOCAL_RUNTIME_SMOKE_PROMPT_LIMIT).collect::<String>())
    } else {
        Some(prompt)
    }
}

fn preview_probe_output(value: &str, preview_limit: usize) -> (String, bool) {
    let normalized = clean_probe_capture(value.to_string());
    let mut chars = normalized.chars();
    let preview = chars.by_ref().take(preview_limit).collect::<String>();
    let truncated = chars.next().is_some();
    if truncated {
        (format!("{preview}…"), true)
    } else {
        (preview, false)
    }
}

fn preview_probe_output_with_redactions(
    value: &str,
    preview_limit: usize,
    redactions: &[(String, String)],
) -> (String, bool) {
    let mut normalized = clean_probe_capture(value.to_string());
    let mut redactions = redactions.to_vec();
    redactions.sort_by(|left, right| right.0.len().cmp(&left.0.len()).then_with(|| left.0.cmp(&right.0)));
    for (search, replacement) in redactions {
        if !search.is_empty() {
            normalized = normalized.replace(&search, &replacement);
        }
    }
    let mut chars = normalized.chars();
    let preview = chars.by_ref().take(preview_limit).collect::<String>();
    let truncated = chars.next().is_some();
    if truncated {
        (format!("{preview}…"), true)
    } else {
        (preview, false)
    }
}

fn build_probe_result(
    status: LocalRuntimeProbeStatus,
    allow_execution: bool,
    execution_attempted: bool,
    probe_argument: String,
    timeout_ms: u64,
    duration_ms: u64,
    safe_executable_file_name: Option<String>,
    exit_code: Option<i32>,
    stdout_preview: String,
    stderr_preview: String,
    blockers: Vec<LocalRuntimeProbeWarning>,
    warnings: Vec<LocalRuntimeProbeWarning>,
) -> LocalRuntimeProbeResult {
    LocalRuntimeProbeResult {
        status,
        allow_execution,
        execution_attempted,
        probe_argument,
        timeout_ms,
        duration_ms,
        safe_executable_file_name,
        exit_code,
        stdout_preview,
        stderr_preview,
        blockers,
        warnings,
    }
}

fn run_smoke_inference_probe(
    executable_path: &Path,
    model_path: &Path,
    prompt: &str,
    max_output_tokens: u32,
    timeout_ms: u64,
    context_window: Option<u32>,
    gpu_layers: Option<i32>,
    temperature: Option<f64>,
) -> AegisResult<LocalRuntimeSmokeInferenceExecution> {
    let mut command = Command::new(executable_path);
    command
        .arg("-m")
        .arg(model_path)
        .arg("-p")
        .arg(prompt)
        .arg("-n")
        .arg(max_output_tokens.to_string())
        .arg("--temp")
        .arg(temperature.unwrap_or(0.0).to_string())
        .arg("--no-display-prompt")
        .arg("--log-disable");
    if let Some(context_window) = context_window {
        command.arg("--ctx-size").arg(context_window.to_string());
    }
    if let Some(gpu_layers) = gpu_layers {
        command.arg("-ngl").arg(gpu_layers.to_string());
    }
    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = command.spawn()?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let stdout_handle = stdout.map(spawn_pipe_reader);
    let stderr_handle = stderr.map(spawn_pipe_reader);
    let started_at = Instant::now();
    let deadline = Duration::from_millis(timeout_ms);
    let mut timed_out = false;

    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if started_at.elapsed() >= deadline {
            timed_out = true;
            let _ = child.kill();
            break child.wait()?;
        }
        thread::sleep(Duration::from_millis(10));
    };

    let stdout = join_pipe_reader(stdout_handle);
    let stderr = join_pipe_reader(stderr_handle);

    Ok(LocalRuntimeSmokeInferenceExecution {
        exit_code: status.code(),
        duration_ms: bucket_probe_duration_ms(started_at.elapsed().as_millis() as u64),
        stdout: clean_probe_capture(stdout),
        stderr: clean_probe_capture(stderr),
        timed_out,
    })
}

fn push_probe_warning(warnings: &mut Vec<LocalRuntimeProbeWarning>, kind: &str, message: &str) {
    if !warnings.iter().any(|warning| warning.kind == kind && warning.message == message) {
        warnings.push(LocalRuntimeProbeWarning {
            kind: kind.to_string(),
            message: message.to_string(),
        });
    }
}

fn push_probe_blocker(blockers: &mut Vec<LocalRuntimeProbeWarning>, kind: &str, message: &str) {
    if !blockers.iter().any(|blocker| blocker.kind == kind && blocker.message == message) {
        blockers.push(LocalRuntimeProbeWarning {
            kind: kind.to_string(),
            message: message.to_string(),
        });
    }
}

fn probe_warning_from_adapter_contract(
    item: &LocalRuntimeAdapterContractWarning,
) -> LocalRuntimeProbeWarning {
    LocalRuntimeProbeWarning {
        kind: item.kind.clone(),
        message: item.message.clone(),
    }
}

fn probe_warning_blocker_from_adapter_contract(
    item: &LocalRuntimeAdapterContractBlocker,
) -> LocalRuntimeProbeWarning {
    LocalRuntimeProbeWarning {
        kind: item.kind.clone(),
        message: item.message.clone(),
    }
}

fn build_smoke_result(
    status: LocalRuntimeSmokeInferenceStatus,
    allow_execution: bool,
    execution_attempted: bool,
    runtime_kind: LocalModelRuntimeKind,
    safe_model_file_name: Option<String>,
    safe_executable_file_name: Option<String>,
    normalized_prompt: String,
    prompt_char_count: u32,
    max_output_tokens: u32,
    timeout_ms: u64,
    exit_code: Option<i32>,
    stdout_preview: String,
    stderr_preview: String,
    duration_ms: u64,
    warnings: Vec<LocalRuntimeSmokeInferenceWarning>,
    blockers: Vec<LocalRuntimeSmokeInferenceBlocker>,
) -> LocalRuntimeSmokeInferenceResult {
    LocalRuntimeSmokeInferenceResult {
        status,
        allow_execution,
        execution_attempted,
        runtime_kind,
        safe_model_file_name,
        safe_executable_file_name,
        normalized_prompt,
        prompt_char_count,
        max_output_tokens,
        timeout_ms,
        exit_code,
        stdout_preview,
        stderr_preview,
        duration_ms,
        warnings,
        blockers,
        diagnostic_only: true,
        no_answer_generated: true,
        no_grounding_applied: true,
        no_evidence_pack_used: true,
        not_scholar_chat_answer: true,
        output_classification: LocalRuntimeSmokeInferenceOutputClassification::RuntimeDiagnostic,
    }
}

fn smoke_warning_has_kind(warnings: &[LocalRuntimeSmokeInferenceWarning], kind: &str) -> bool {
    warnings.iter().any(|warning| warning.kind == kind)
}

fn smoke_runtime_config_from_execution_plan_request(
    request: &LocalRuntimeSmokeExecutionPlanPreviewRequest,
) -> LocalModelRuntimeConfig {
    let adapter_contract_request = &request
        .smoke_readiness_preview_request
        .capability_preview_request
        .version_probe_preview_request
        .probe_readiness_preview_request
        .validation_preview_request
        .adapter_contract_request;
    LocalModelRuntimeConfig {
        runtime_kind: LocalModelRuntimeKind::LlamaCpp,
        model_path: adapter_contract_request.model_path.clone(),
        executable_path: adapter_contract_request.executable_path.clone(),
        context_window: adapter_contract_request.context_window_tokens,
        gpu_layers: adapter_contract_request.gpu_layers,
        temperature: None,
    }
}

fn push_smoke_warning(warnings: &mut Vec<LocalRuntimeSmokeInferenceWarning>, kind: &str, message: &str) {
    if !warnings.iter().any(|warning| warning.kind == kind && warning.message == message) {
        warnings.push(LocalRuntimeSmokeInferenceWarning {
            kind: kind.to_string(),
            message: message.to_string(),
        });
    }
}

fn push_smoke_blocker(blockers: &mut Vec<LocalRuntimeSmokeInferenceBlocker>, kind: &str, message: &str) {
    if !blockers.iter().any(|blocker| blocker.kind == kind && blocker.message == message) {
        blockers.push(LocalRuntimeSmokeInferenceBlocker {
            kind: kind.to_string(),
            message: message.to_string(),
        });
    }
}

struct LocalRuntimeProbeExecution {
    exit_code: Option<i32>,
    duration_ms: u64,
    stdout: String,
    stderr: String,
    timed_out: bool,
}

struct LocalRuntimeSmokeInferenceExecution {
    exit_code: Option<i32>,
    duration_ms: u64,
    stdout: String,
    stderr: String,
    timed_out: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::env;

    fn config(model_path: Option<&str>, executable_path: Option<&str>) -> LocalModelRuntimeConfig {
        LocalModelRuntimeConfig {
            runtime_kind: LocalModelRuntimeKind::None,
            model_path: model_path.map(|value| value.to_string()),
            executable_path: executable_path.map(|value| value.to_string()),
            context_window: None,
            gpu_layers: None,
            temperature: None,
        }
    }

    fn probe_request(executable_path: Option<&str>, allow_execution: bool, timeout_ms: Option<u64>) -> LocalRuntimeProbeRequest {
        LocalRuntimeProbeRequest {
            executable_path: executable_path.map(|value| value.to_string()),
            allow_execution,
            timeout_ms,
        }
    }

    fn smoke_request_with_runtime_kind(
        runtime_kind: LocalModelRuntimeKind,
        model_path: Option<&str>,
        executable_path: Option<&str>,
        allow_execution: bool,
        prompt: Option<&str>,
        timeout_ms: Option<u64>,
        max_output_tokens: Option<u32>,
    ) -> LocalRuntimeSmokeInferenceRequest {
        LocalRuntimeSmokeInferenceRequest {
            runtime_config: LocalModelRuntimeConfig {
                runtime_kind,
                model_path: model_path.map(|value| value.to_string()),
                executable_path: executable_path.map(|value| value.to_string()),
                context_window: Some(512),
                gpu_layers: Some(0),
                temperature: Some(0.0),
            },
            allow_execution,
            prompt: prompt.map(|value| value.to_string()),
            timeout_ms,
            max_output_tokens,
        }
    }

    fn smoke_request(
        model_path: Option<&str>,
        executable_path: Option<&str>,
        allow_execution: bool,
        prompt: Option<&str>,
        timeout_ms: Option<u64>,
        max_output_tokens: Option<u32>,
    ) -> LocalRuntimeSmokeInferenceRequest {
        smoke_request_with_runtime_kind(
            LocalModelRuntimeKind::LlamaCpp,
            model_path,
            executable_path,
            allow_execution,
            prompt,
            timeout_ms,
            max_output_tokens,
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
            .arg("--edition=2021")
            .arg(&source_path)
            .arg("-o")
            .arg(&executable_path)
            .status()
            .unwrap();
        assert!(status.success());
        executable_path
    }

    fn assert_root_clean(root: &tempfile::TempDir, expected_entries: usize) {
        assert!(!root.path().join(".aegis").exists());
        assert_eq!(fs::read_dir(root.path()).unwrap().count(), expected_entries);
    }

    fn assert_smoke_boundary_fields(result: &LocalRuntimeSmokeInferenceResult) {
        assert!(result.diagnostic_only);
        assert!(result.no_answer_generated);
        assert!(result.no_grounding_applied);
        assert!(result.no_evidence_pack_used);
        assert!(result.not_scholar_chat_answer);
        assert_eq!(
            result.output_classification,
            LocalRuntimeSmokeInferenceOutputClassification::RuntimeDiagnostic
        );
    }

    #[test]
    fn local_runtime_health_preview_is_not_configured_for_default_config() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_local_model_runtime_health(temp.path(), config(None, None)).unwrap();
        assert_eq!(result.status, LocalModelRuntimeHealthStatus::NotConfigured);
        assert_eq!(result.model_state, LocalModelRuntimePathState::NotConfigured);
        assert_eq!(result.executable_state, LocalModelRuntimePathState::NotConfigured);
        assert!(result.warnings.iter().any(|warning| warning.message.contains("No local runtime is configured")));
        assert!(result.warnings.iter().any(|warning| warning.message.contains("preview only")));
        assert!(result.warnings.iter().any(|warning| warning.message.contains("not persisted")));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn local_runtime_health_preview_warns_about_non_gguf_extension() {
        let temp = tempfile::tempdir().unwrap();
        let model_path = temp.path().join("model.txt");
        fs::write(&model_path, "not a gguf model").unwrap();
        let result = preview_local_model_runtime_health(
            temp.path(),
            LocalModelRuntimeConfig {
                runtime_kind: LocalModelRuntimeKind::LlamaCpp,
                model_path: Some(model_path.to_string_lossy().to_string()),
                executable_path: None,
                context_window: Some(4096),
                gpu_layers: Some(12),
                temperature: Some(0.7),
            },
        )
        .unwrap();
        assert_eq!(result.status, LocalModelRuntimeHealthStatus::ConfigPresent);
        assert_eq!(result.model_state, LocalModelRuntimePathState::Exists);
        assert!(!result.model_extension_valid);
        assert!(result.warnings.iter().any(|warning| warning.message.contains(".gguf extension")));
        let debug = format!("{result:?}");
        let json = serde_json::to_string(&result).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
    }

    #[test]
    fn local_runtime_health_preview_reports_missing_model_without_paths() {
        let temp = tempfile::tempdir().unwrap();
        let model_path = temp.path().join("missing-model.gguf");
        let result = preview_local_model_runtime_health(
            temp.path(),
            LocalModelRuntimeConfig {
                runtime_kind: LocalModelRuntimeKind::LlamaCpp,
                model_path: Some(model_path.to_string_lossy().to_string()),
                executable_path: None,
                context_window: None,
                gpu_layers: None,
                temperature: None,
            },
        )
        .unwrap();
        assert_eq!(result.status, LocalModelRuntimeHealthStatus::ModelMissing);
        assert_eq!(result.model_state, LocalModelRuntimePathState::Missing);
        assert!(result.warnings.iter().any(|warning| warning.message.contains("missing")));
        let debug = format!("{result:?}");
        let json = serde_json::to_string(&result).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
    }

    #[test]
    fn local_runtime_health_preview_reports_missing_executable_without_paths() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_local_model_runtime_health(
            temp.path(),
            LocalModelRuntimeConfig {
                runtime_kind: LocalModelRuntimeKind::LlamaCpp,
                model_path: None,
                executable_path: Some("missing-llama-cpp.exe".to_string()),
                context_window: None,
                gpu_layers: None,
                temperature: None,
            },
        )
        .unwrap();
        assert_eq!(result.status, LocalModelRuntimeHealthStatus::ExecutableMissing);
        assert_eq!(result.executable_state, LocalModelRuntimePathState::Missing);
        assert!(result.warnings.iter().any(|warning| warning.message.contains("executable file is missing")));
        let debug = format!("{result:?}");
        let json = serde_json::to_string(&result).unwrap();
        assert!(!debug.contains("missing-llama-cpp.exe"));
        assert!(!json.contains("missing-llama-cpp.exe"));
    }

    #[test]
    fn local_runtime_health_preview_reports_ready_for_existing_gguf_file() {
        let temp = tempfile::tempdir().unwrap();
        let model_path = temp.path().join("ready-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        let result = preview_local_model_runtime_health(
            temp.path(),
            LocalModelRuntimeConfig {
                runtime_kind: LocalModelRuntimeKind::LlamaCpp,
                model_path: Some(model_path.to_string_lossy().to_string()),
                executable_path: None,
                context_window: Some(8192),
                gpu_layers: Some(0),
                temperature: Some(0.2),
            },
        )
        .unwrap();
        assert_eq!(result.status, LocalModelRuntimeHealthStatus::ReadyToTestLater);
        assert_eq!(result.model_state, LocalModelRuntimePathState::Exists);
        assert!(result.model_extension_valid);
        assert_eq!(result.model_file_name.as_deref(), Some("ready-model.gguf"));
        assert!(result.warnings.iter().any(|warning| warning.message.contains("ready for a later test run")));
    }

    #[test]
    fn local_runtime_health_preview_is_deterministic_and_path_free() {
        let temp = tempfile::tempdir().unwrap();
        let model_path = temp.path().join("deterministic.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        let executable_path = temp.path().join("llama-cli.exe");
        fs::write(&executable_path, "placeholder").unwrap();
        let request = LocalModelRuntimeConfig {
            runtime_kind: LocalModelRuntimeKind::LlamaCpp,
            model_path: Some(model_path.to_string_lossy().to_string()),
            executable_path: Some(executable_path.to_string_lossy().to_string()),
            context_window: Some(4096),
            gpu_layers: Some(8),
            temperature: Some(0.6),
        };
        let first = preview_local_model_runtime_health(temp.path(), request.clone()).unwrap();
        let second = preview_local_model_runtime_health(temp.path(), request).unwrap();
        assert_eq!(first, second);
        let debug = format!("{first:?}");
        let json = serde_json::to_string(&first).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert!(!temp.path().join(".aegis").exists());
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 2);
    }

    #[test]
    fn local_runtime_health_preview_rejects_traversal_like_paths_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["..", "../model.gguf", "nested/../model.gguf", "nested\\..\\model.gguf"] {
            let result = preview_local_model_runtime_health(
                temp.path(),
                LocalModelRuntimeConfig {
                    runtime_kind: LocalModelRuntimeKind::LlamaCpp,
                    model_path: Some(invalid.to_string()),
                    executable_path: None,
                    context_window: None,
                    gpu_layers: None,
                    temperature: None,
                },
            );
            assert!(matches!(result, Err(AegisError::LocalModelRuntimeInvalidPath)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    fn invocation_request(
        runtime_kind: LocalModelRuntimeKind,
        model_path: Option<&str>,
        executable_path: Option<&str>,
        prompt_text: Option<&str>,
        estimated_input_char_count: Option<u32>,
        max_output_tokens: Option<u32>,
        stop_sequences: Option<Vec<&str>>,
        context_window: Option<u32>,
        gpu_layers: Option<i32>,
        temperature: Option<f64>,
    ) -> LocalRuntimeInvocationPlanRequest {
        LocalRuntimeInvocationPlanRequest {
            runtime_config: LocalModelRuntimeConfig {
                runtime_kind,
                model_path: model_path.map(|value| value.to_string()),
                executable_path: executable_path.map(|value| value.to_string()),
                context_window,
                gpu_layers,
                temperature,
            },
            prompt_text: prompt_text.map(|value| value.to_string()),
            estimated_input_char_count,
            max_output_tokens,
            stop_sequences: stop_sequences.map(|values| values.into_iter().map(|value| value.to_string()).collect()),
        }
    }

    fn llama_runtime_adapter_contract_request(
        executable_path: Option<&str>,
        model_path: Option<&str>,
        model_family: Option<&str>,
        model_format: Option<&str>,
        context_window_tokens: Option<u32>,
        gpu_layers: Option<i32>,
        threads: Option<u32>,
        batch_size: Option<u32>,
        chat_template: Option<&str>,
    ) -> LocalRuntimeAdapterContractPreviewRequest {
        LocalRuntimeAdapterContractPreviewRequest {
            adapter_kind: LocalRuntimeAdapterKind::LlamaCpp,
            executable_path: executable_path.map(|value| value.to_string()),
            model_path: model_path.map(|value| value.to_string()),
            model_family: model_family.map(|value| value.to_string()),
            model_format: model_format.map(|value| value.to_string()),
            context_window_tokens,
            gpu_layers,
            threads,
            batch_size,
            chat_template: chat_template.map(|value| value.to_string()),
        }
    }

    fn assert_llama_runtime_adapter_contract_boundary_fields(
        preview: &LocalRuntimeAdapterContractPreview,
    ) {
        assert!(preview.preview_only);
        assert!(preview.no_process_spawn);
        assert!(preview.no_model_load);
        assert!(preview.no_llm_call);
        assert!(preview.no_runtime_execution);
        assert!(preview.no_persistence);
        assert!(preview.no_artifact_write);
        assert!(preview.no_registry_status_change);
        assert!(preview.no_audit_write);
    }

    fn assert_llama_runtime_adapter_contract_deterministic_and_path_free(
        temp: &tempfile::TempDir,
        request: LocalRuntimeAdapterContractPreviewRequest,
    ) -> LocalRuntimeAdapterContractPreview {
        let before_entries = fs::read_dir(temp.path()).unwrap().count();
        let first = preview_llama_runtime_adapter_contract(temp.path(), request.clone()).unwrap();
        let second = preview_llama_runtime_adapter_contract(temp.path(), request).unwrap();
        let after_entries = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_llama_runtime_adapter_contract_boundary_fields(preview);
            assert!(preview.no_process_spawn);
            assert!(preview.no_model_load);
            assert!(preview.no_llm_call);
            assert!(preview.no_runtime_execution);
            assert!(preview.no_persistence);
            assert!(preview.no_artifact_write);
            assert!(preview.no_registry_status_change);
            assert!(preview.no_audit_write);
        }
        first
    }

    fn llama_runtime_validation_request(
        executable_path: Option<&str>,
        model_path: Option<&str>,
        model_family: Option<&str>,
        model_format: Option<&str>,
        context_window_tokens: Option<u32>,
        gpu_layers: Option<i32>,
        threads: Option<u32>,
        batch_size: Option<u32>,
        chat_template: Option<&str>,
    ) -> LocalRuntimeValidationPreviewRequest {
        LocalRuntimeValidationPreviewRequest {
            adapter_contract_request: llama_runtime_adapter_contract_request(
                executable_path,
                model_path,
                model_family,
                model_format,
                context_window_tokens,
                gpu_layers,
                threads,
                batch_size,
                chat_template,
            ),
        }
    }

    fn assert_llama_runtime_validation_boundary_fields(preview: &LocalRuntimeValidationPreview) {
        assert!(preview.preview_only);
        assert!(preview.no_process_spawn);
        assert!(preview.no_binary_probe);
        assert!(preview.no_model_load);
        assert!(preview.no_llm_call);
        assert!(preview.no_runtime_execution);
        assert!(preview.no_persistence);
        assert!(preview.no_artifact_write);
        assert!(preview.no_registry_status_change);
        assert!(preview.no_audit_write);
    }

    fn assert_llama_runtime_validation_deterministic_and_path_free(
        temp: &tempfile::TempDir,
        request: LocalRuntimeValidationPreviewRequest,
    ) -> LocalRuntimeValidationPreview {
        let before_entries = fs::read_dir(temp.path()).unwrap().count();
        let first = preview_llama_runtime_validation(temp.path(), request.clone()).unwrap();
        let second = preview_llama_runtime_validation(temp.path(), request).unwrap();
        let after_entries = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_llama_runtime_validation_boundary_fields(preview);
        }
        first
    }

    fn probe_readiness_request(
        validation_preview_request: LocalRuntimeValidationPreviewRequest,
        probe_consent: bool,
    ) -> LocalRuntimeProbeReadinessPreviewRequest {
        LocalRuntimeProbeReadinessPreviewRequest {
            validation_preview_request,
            probe_consent,
        }
    }

    fn version_probe_request(
        validation_preview_request: LocalRuntimeValidationPreviewRequest,
        probe_consent: bool,
        allow_probe_execution: bool,
        timeout_ms: Option<u64>,
    ) -> LocalRuntimeVersionProbePreviewRequest {
        LocalRuntimeVersionProbePreviewRequest {
            probe_readiness_preview_request: probe_readiness_request(validation_preview_request, probe_consent),
            allow_probe_execution,
            timeout_ms,
        }
    }

    fn version_probe_helper_executable_with_source(
        temp: &tempfile::TempDir,
        executable_name: &str,
        source: &str,
    ) -> PathBuf {
        let source_path = temp.path().join(format!("{executable_name}.rs"));
        let executable_path = temp.path().join(executable_name);
        let crate_name = source_path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("version_probe_helper")
            .chars()
            .map(|value| if value.is_ascii_alphanumeric() || value == '_' { value } else { '_' })
            .collect::<String>();
        fs::write(&source_path, source).unwrap();
        let rustc = env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
        let status = Command::new(rustc)
            .arg("--crate-name")
            .arg(&crate_name)
            .arg("--edition=2021")
            .arg(&source_path)
            .arg("-o")
            .arg(&executable_path)
            .status()
            .unwrap();
        assert!(status.success());
        executable_path
    }

    fn version_probe_helper_executable(temp: &tempfile::TempDir, executable_name: &str) -> PathBuf {
        let source = r#"
use std::{env, thread, time::Duration};

fn main() {
    let args: Vec<String> = env::args().collect();
    let exe_name = env::current_exe()
        .ok()
        .and_then(|path| path.file_name().and_then(|value| value.to_str()).map(|value| value.to_string()))
        .unwrap_or_default();
    println!("stdout marker");
    println!("args={}", args.join(" | "));
    println!("exe_name={}", exe_name);
    println!("{}", "S".repeat(5000));
    eprintln!("stderr marker");
    eprintln!("args={}", args.join(" | "));
    eprintln!("exe_name={}", exe_name);
    eprintln!("{}", "E".repeat(5000));
    if exe_name.contains("slow") {
        thread::sleep(Duration::from_millis(750));
    }
    if exe_name.contains("fail") {
        std::process::exit(7);
    }
}
"#;
        version_probe_helper_executable_with_source(temp, executable_name, source)
    }

    fn assert_llama_runtime_probe_readiness_boundary_fields(
        preview: &LocalRuntimeProbeReadinessPreview,
    ) {
        assert!(preview.preview_only);
        assert!(preview.no_process_spawn);
        assert!(preview.no_binary_probe);
        assert!(preview.no_model_load);
        assert!(preview.no_llm_call);
        assert!(preview.no_runtime_execution);
        assert!(preview.no_persistence);
        assert!(preview.no_artifact_write);
        assert!(preview.no_registry_status_change);
        assert!(preview.no_audit_write);
    }

    fn assert_llama_runtime_version_probe_boundary_fields(
        preview: &LocalRuntimeVersionProbePreview,
    ) {
        assert!(preview.preview_only);
        assert!(preview.no_model_load);
        assert!(preview.no_model_path_argument);
        assert!(preview.no_llm_call);
        assert!(preview.no_runtime_inference);
        assert!(preview.no_persistence);
        assert!(preview.no_artifact_write);
        assert!(preview.no_registry_status_change);
        assert!(preview.no_audit_write);
    }

    fn capability_request(
        validation_preview_request: LocalRuntimeValidationPreviewRequest,
        probe_consent: bool,
        allow_probe_execution: bool,
        timeout_ms: Option<u64>,
    ) -> LocalRuntimeCapabilityPreviewRequest {
        LocalRuntimeCapabilityPreviewRequest {
            version_probe_preview_request: version_probe_request(
                validation_preview_request,
                probe_consent,
                allow_probe_execution,
                timeout_ms,
            ),
        }
    }

    fn assert_llama_runtime_capability_boundary_fields(
        preview: &LocalRuntimeCapabilityPreview,
    ) {
        assert!(preview.preview_only);
        assert!(preview.no_new_process_spawn);
        assert!(preview.no_binary_probe_beyond_wrapped_version_probe);
        assert!(preview.no_model_path_argument);
        assert!(preview.no_model_file_read);
        assert!(preview.no_model_load);
        assert!(preview.no_runtime_inference);
        assert!(preview.no_smoke_inference);
        assert!(preview.no_llm_call);
        assert!(preview.no_persistence);
        assert!(preview.no_artifact_write);
        assert!(preview.no_registry_status_change);
        assert!(preview.no_audit_write);
    }

    fn assert_llama_runtime_capability_deterministic_and_path_free(
        temp: &tempfile::TempDir,
        request: LocalRuntimeCapabilityPreviewRequest,
    ) -> LocalRuntimeCapabilityPreview {
        let before_entries = fs::read_dir(temp.path()).unwrap().count();
        let first = preview_llama_runtime_capability(temp.path(), request.clone()).unwrap();
        let second = preview_llama_runtime_capability(temp.path(), request).unwrap();
        let after_entries = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_llama_runtime_capability_boundary_fields(preview);
        }
        first
    }

    fn smoke_readiness_request(
        capability_preview_request: LocalRuntimeCapabilityPreviewRequest,
        smoke_consent: bool,
        diagnostic_prompt: Option<&str>,
        max_output_tokens: Option<u32>,
        timeout_ms: Option<u64>,
    ) -> LocalRuntimeSmokeReadinessPreviewRequest {
        LocalRuntimeSmokeReadinessPreviewRequest {
            capability_preview_request,
            smoke_consent,
            diagnostic_prompt: diagnostic_prompt.map(|value| value.to_string()),
            max_output_tokens,
            timeout_ms,
        }
    }

    fn smoke_execution_plan_request(
        capability_preview_request: LocalRuntimeCapabilityPreviewRequest,
        smoke_consent: bool,
        diagnostic_prompt: Option<&str>,
        max_output_tokens: Option<u32>,
        timeout_ms: Option<u64>,
    ) -> LocalRuntimeSmokeExecutionPlanPreviewRequest {
        LocalRuntimeSmokeExecutionPlanPreviewRequest {
            smoke_readiness_preview_request: smoke_readiness_request(
                capability_preview_request,
                smoke_consent,
                diagnostic_prompt,
                max_output_tokens,
                timeout_ms,
            ),
        }
    }

    fn smoke_diagnostic_request(
        capability_preview_request: LocalRuntimeCapabilityPreviewRequest,
        smoke_consent: bool,
        diagnostic_prompt: Option<&str>,
        max_output_tokens: Option<u32>,
        timeout_ms: Option<u64>,
        allow_smoke_execution: bool,
    ) -> LocalRuntimeSmokeDiagnosticRequest {
        LocalRuntimeSmokeDiagnosticRequest {
            smoke_execution_plan_preview_request: smoke_execution_plan_request(
                capability_preview_request,
                smoke_consent,
                diagnostic_prompt,
                max_output_tokens,
                timeout_ms,
            ),
            allow_smoke_execution,
        }
    }

    fn smoke_readiness_helper_executable(temp: &tempfile::TempDir, executable_name: &str) -> PathBuf {
        let marker_path = temp.path().join("smoke_readiness_unexpected_call.txt");
        let marker_literal = format!("{:?}", marker_path.to_string_lossy().to_string());
        let source = r#"
use std::{env, fs, path::PathBuf};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|arg| matches!(arg.as_str(), "-p" | "--prompt" | "-n" | "--max_output_tokens" | "-m" | "--ctx-size" | "-ngl")) {
        let marker = PathBuf::from(MARKER_PATH);
        let _ = fs::write(marker, args.join(" | "));
        std::process::exit(91);
    }
    println!("stdout marker");
    println!("args={}", args.join(" | "));
    println!("llama.cpp version 1.2.3");
    eprintln!("stderr marker");
    eprintln!("args={}", args.join(" | "));
    eprintln!("llama.cpp version 1.2.3");
}
"#
        .replace("MARKER_PATH", &marker_literal);
        version_probe_helper_executable_with_source(temp, executable_name, &source)
    }

    fn assert_llama_runtime_smoke_readiness_boundary_fields(
        preview: &LocalRuntimeSmokeReadinessPreview,
    ) {
        assert!(preview.preview_only);
        assert!(preview.no_new_process_spawn);
        assert!(preview.no_smoke_inference_execution);
        assert!(preview.no_model_path_argument);
        assert!(preview.no_model_file_read);
        assert!(preview.no_model_load);
        assert!(preview.no_llm_call);
        assert!(preview.no_persistence);
        assert!(preview.no_artifact_write);
        assert!(preview.no_registry_status_change);
        assert!(preview.no_audit_write);
        assert!(preview.diagnostic_only);
        assert!(preview.not_scholar_chat_answer);
        assert!(preview.no_answer_generated);
        assert!(preview.no_grounding_applied);
        assert!(preview.no_evidence_pack_used);
    }

    fn assert_llama_runtime_smoke_execution_plan_boundary_fields(
        preview: &LocalRuntimeSmokeExecutionPlanPreview,
    ) {
        assert!(preview.preview_only);
        assert!(preview.no_process_spawn);
        assert!(preview.no_smoke_inference_execution);
        assert!(preview.no_model_file_read);
        assert!(preview.no_model_load);
        assert!(preview.no_llm_call);
        assert!(preview.no_persistence);
        assert!(preview.no_artifact_write);
        assert!(preview.no_registry_status_change);
        assert!(preview.no_audit_write);
        assert!(preview.diagnostic_only);
        assert!(preview.not_scholar_chat_answer);
        assert!(preview.no_answer_generated);
        assert!(preview.no_grounding_applied);
        assert!(preview.no_evidence_pack_used);
    }

    fn assert_llama_runtime_smoke_diagnostic_boundary_fields(
        preview: &LocalRuntimeSmokeDiagnosticPreview,
    ) {
        assert!(preview.diagnostic_only);
        assert!(preview.not_scholar_chat_answer);
        assert!(preview.no_answer_generated);
        assert!(preview.no_grounding_applied);
        assert!(preview.no_evidence_pack_used);
        assert!(preview.no_persistence);
        assert!(preview.no_artifact_write);
        assert!(preview.no_registry_status_change);
        assert!(preview.no_audit_write);
    }

    fn assert_llama_runtime_smoke_readiness_deterministic_and_path_free(
        temp: &tempfile::TempDir,
        request: LocalRuntimeSmokeReadinessPreviewRequest,
    ) -> LocalRuntimeSmokeReadinessPreview {
        let before_entries = fs::read_dir(temp.path()).unwrap().count();
        let first = preview_llama_runtime_smoke_readiness(temp.path(), request.clone()).unwrap();
        let second = preview_llama_runtime_smoke_readiness(temp.path(), request).unwrap();
        let after_entries = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_llama_runtime_smoke_readiness_boundary_fields(preview);
        }
        first
    }

    fn assert_llama_runtime_smoke_execution_plan_deterministic_and_path_free(
        temp: &tempfile::TempDir,
        request: LocalRuntimeSmokeExecutionPlanPreviewRequest,
    ) -> LocalRuntimeSmokeExecutionPlanPreview {
        let before_entries = fs::read_dir(temp.path()).unwrap().count();
        let first = preview_llama_runtime_smoke_execution_plan(temp.path(), request.clone()).unwrap();
        let second = preview_llama_runtime_smoke_execution_plan(temp.path(), request).unwrap();
        let after_entries = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_llama_runtime_smoke_execution_plan_boundary_fields(preview);
        }
        first
    }

    fn assert_llama_runtime_smoke_diagnostic_deterministic_and_path_free(
        temp: &tempfile::TempDir,
        request: LocalRuntimeSmokeDiagnosticRequest,
    ) -> LocalRuntimeSmokeDiagnosticPreview {
        let before_entries = fs::read_dir(temp.path()).unwrap().count();
        let first = run_llama_runtime_smoke_diagnostic(temp.path(), request.clone()).unwrap();
        let second = run_llama_runtime_smoke_diagnostic(temp.path(), request).unwrap();
        let after_entries = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_llama_runtime_smoke_diagnostic_boundary_fields(preview);
        }
        first
    }

    fn assert_llama_runtime_probe_readiness_deterministic_and_path_free(
        temp: &tempfile::TempDir,
        request: LocalRuntimeProbeReadinessPreviewRequest,
    ) -> LocalRuntimeProbeReadinessPreview {
        let before_entries = fs::read_dir(temp.path()).unwrap().count();
        let first = preview_llama_runtime_probe_readiness(temp.path(), request.clone()).unwrap();
        let second = preview_llama_runtime_probe_readiness(temp.path(), request).unwrap();
        let after_entries = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_llama_runtime_probe_readiness_boundary_fields(preview);
        }
        first
    }

    fn assert_llama_runtime_version_probe_deterministic_and_path_free(
        temp: &tempfile::TempDir,
        request: LocalRuntimeVersionProbePreviewRequest,
    ) -> LocalRuntimeVersionProbePreview {
        let before_entries = fs::read_dir(temp.path()).unwrap().count();
        let first = run_llama_runtime_version_probe(temp.path(), request.clone()).unwrap();
        let second = run_llama_runtime_version_probe(temp.path(), request).unwrap();
        let after_entries = fs::read_dir(temp.path()).unwrap().count();
        assert_eq!(first, second);
        assert_eq!(before_entries, after_entries);
        let temp_path = temp.path().to_string_lossy();
        for preview in [&first, &second] {
            let debug = format!("{preview:?}");
            let json = serde_json::to_string(preview).unwrap();
            assert!(!debug.contains(temp_path.as_ref()));
            assert!(!json.contains(temp_path.as_ref()));
            assert_llama_runtime_version_probe_boundary_fields(preview);
        }
        first
    }

    #[test]
    fn local_runtime_invocation_plan_preview_is_not_configured_for_default_config() {
        let temp = tempfile::tempdir().unwrap();
        let result = preview_local_runtime_invocation_plan(
            temp.path(),
            invocation_request(LocalModelRuntimeKind::None, None, None, None, None, None, None, None, None, None),
        )
        .unwrap();
        assert_eq!(result.status, LocalRuntimeInvocationPlanStatus::NotConfigured);
        assert_eq!(result.plan.runtime_health_status, LocalModelRuntimeHealthStatus::NotConfigured);
        assert!(result.plan.blockers.iter().any(|blocker| blocker.kind == "runtime_not_configured"));
        assert!(result.plan.warnings.iter().any(|warning| warning.message.contains("preview only")));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn local_runtime_invocation_plan_preview_reports_missing_model_without_paths() {
        let temp = tempfile::tempdir().unwrap();
        let model_path = temp.path().join("missing-model.gguf");
        let result = preview_local_runtime_invocation_plan(
            temp.path(),
            invocation_request(
                LocalModelRuntimeKind::LlamaCpp,
                Some(model_path.to_string_lossy().as_ref()),
                None,
                Some("  question about runtime  "),
                Some(42),
                Some(128),
                None,
                Some(4096),
                Some(8),
                Some(0.6),
            ),
        )
        .unwrap();
        assert_eq!(result.status, LocalRuntimeInvocationPlanStatus::Blocked);
        assert_eq!(result.plan.runtime_health_status, LocalModelRuntimeHealthStatus::ModelMissing);
        assert!(result.plan.blockers.iter().any(|blocker| blocker.kind == "model_missing"));
        assert_eq!(result.plan.prompt_char_count, "question about runtime".chars().count() as u32);
        assert_eq!(result.plan.estimated_context_char_count, 42);
        let debug = format!("{result:?}");
        let json = serde_json::to_string(&result).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
    }

    #[test]
    fn local_runtime_invocation_plan_preview_blocks_non_gguf_extensions() {
        let temp = tempfile::tempdir().unwrap();
        let model_path = temp.path().join("model.txt");
        fs::write(&model_path, "not a gguf model").unwrap();
        let result = preview_local_runtime_invocation_plan(
            temp.path(),
            invocation_request(
                LocalModelRuntimeKind::LlamaCpp,
                Some(model_path.to_string_lossy().as_ref()),
                None,
                Some("runtime planning"),
                None,
                Some(256),
                None,
                Some(4096),
                Some(8),
                Some(0.6),
            ),
        )
        .unwrap();
        assert_eq!(result.status, LocalRuntimeInvocationPlanStatus::Blocked);
        assert_eq!(result.plan.runtime_health_status, LocalModelRuntimeHealthStatus::ConfigPresent);
        assert!(result.plan.blockers.iter().any(|blocker| blocker.kind == "model_extension_invalid"));
        let debug = format!("{result:?}");
        let json = serde_json::to_string(&result).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
    }

    #[test]
    fn local_runtime_invocation_plan_preview_reports_ready_to_invoke_later_for_existing_gguf_file() {
        let temp = tempfile::tempdir().unwrap();
        let model_path = temp.path().join("ready-model.gguf");
        let executable_path = temp.path().join("llama-cli.exe");
        fs::write(&model_path, "gguf placeholder").unwrap();
        fs::write(&executable_path, "placeholder").unwrap();
        let result = preview_local_runtime_invocation_plan(
            temp.path(),
            invocation_request(
                LocalModelRuntimeKind::LlamaCpp,
                Some(model_path.to_string_lossy().as_ref()),
                Some(executable_path.to_string_lossy().as_ref()),
                Some("  prompt with spaces  "),
                Some(512),
                Some(1024),
                Some(vec!["</s>", "<|end|>"]),
                Some(8192),
                Some(16),
                Some(0.2),
            ),
        )
        .unwrap();
        assert_eq!(result.status, LocalRuntimeInvocationPlanStatus::ReadyToInvokeLater);
        assert_eq!(result.plan.runtime_health_status, LocalModelRuntimeHealthStatus::ReadyToTestLater);
        assert_eq!(result.plan.safe_model_file_name.as_deref(), Some("ready-model.gguf"));
        assert_eq!(result.plan.safe_executable_file_name.as_deref(), Some("llama-cli.exe"));
        assert!(result.plan.safe_argument_preview.iter().any(|item| item == "--model-file-name=ready-model.gguf"));
        assert!(result.plan.safe_argument_preview.iter().any(|item| item == "--executable-file-name=llama-cli.exe"));
        assert!(result.plan.invocation_steps.iter().any(|step| step.contains("Prepare redacted invocation arguments")));
    }

    #[test]
    fn local_runtime_invocation_plan_preview_is_deterministic_and_path_free() {
        let temp = tempfile::tempdir().unwrap();
        let model_path = temp.path().join("deterministic.gguf");
        let executable_path = temp.path().join("llama-cli.exe");
        fs::write(&model_path, "gguf placeholder").unwrap();
        fs::write(&executable_path, "placeholder").unwrap();
        let request = invocation_request(
            LocalModelRuntimeKind::LlamaCpp,
            Some(model_path.to_string_lossy().as_ref()),
            Some(executable_path.to_string_lossy().as_ref()),
            Some("  trimmed prompt  "),
            Some(777),
            Some(2048),
            Some(vec!["stop", "stop", " end "]),
            Some(4096),
            Some(8),
            Some(0.6),
        );
        let first = preview_local_runtime_invocation_plan(temp.path(), request.clone()).unwrap();
        let second = preview_local_runtime_invocation_plan(temp.path(), request).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.plan.prompt_char_count, "trimmed prompt".chars().count() as u32);
        assert_eq!(first.plan.estimated_context_char_count, 777);
        assert!(first.plan.safe_argument_preview.iter().any(|item| item == "--stop-sequences=2"));
        let debug = format!("{first:?}");
        let json = serde_json::to_string(&first).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert!(!temp.path().join(".aegis").exists());
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 2);
    }

    #[test]
    fn local_runtime_invocation_plan_preview_rejects_traversal_like_paths_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["..", "../model.gguf", "nested/../model.gguf", "nested\\..\\model.gguf"] {
            let result = preview_local_runtime_invocation_plan(
                temp.path(),
                invocation_request(
                    LocalModelRuntimeKind::LlamaCpp,
                    Some(invalid),
                    Some("llama-cli.exe"),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ),
            );
            assert!(matches!(result, Err(AegisError::LocalModelRuntimeInvalidPath)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn local_runtime_adapter_contract_preview_blocks_when_executable_path_is_missing() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_llama_runtime_adapter_contract_deterministic_and_path_free(
            &temp,
            llama_runtime_adapter_contract_request(
                None,
                Some("adapter-model.gguf"),
                None,
                Some("gguf"),
                Some(8192),
                Some(0),
                Some(8),
                Some(256),
                Some("template"),
            ),
        );
        assert_eq!(result.status, LocalRuntimeAdapterContractStatus::Blocked);
        assert!(!result.executable_path_present);
        assert!(result.model_path_present);
        assert!(result.missing_inputs.contains(&"executable_path".to_string()));
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "executable_path_missing"));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("executable_path")));
        assert!(!temp.path().join(".aegis").exists());
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 0);
    }

    #[test]
    fn local_runtime_adapter_contract_preview_blocks_when_model_path_is_missing() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_llama_runtime_adapter_contract_deterministic_and_path_free(
            &temp,
            llama_runtime_adapter_contract_request(
                Some("llama-cli.exe"),
                None,
                None,
                Some("gguf"),
                Some(8192),
                Some(0),
                Some(8),
                Some(256),
                Some("template"),
            ),
        );
        assert_eq!(result.status, LocalRuntimeAdapterContractStatus::Blocked);
        assert!(result.executable_path_present);
        assert!(!result.model_path_present);
        assert!(result.missing_inputs.contains(&"model_path".to_string()));
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "model_path_missing"));
        assert!(!temp.path().join(".aegis").exists());
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 0);
    }

    #[test]
    fn local_runtime_adapter_contract_preview_normalizes_gguf_and_can_be_ready_later() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_llama_runtime_adapter_contract_deterministic_and_path_free(
            &temp,
            llama_runtime_adapter_contract_request(
                Some("llama-cli.exe"),
                Some("model.gguf"),
                Some("gemma-2"),
                Some("  GGUF  "),
                Some(8192),
                Some(0),
                Some(8),
                Some(256),
                Some("template"),
            ),
        );
        assert_eq!(result.status, LocalRuntimeAdapterContractStatus::ContractReadyLater);
        assert_eq!(result.normalized_model_family.as_deref(), Some("gemma"));
        assert_eq!(result.normalized_model_format, "gguf");
        assert!(result.missing_inputs.is_empty());
        assert!(result.contract_reasons.iter().any(|reason| reason.contains("Normalized model family: gemma")));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn local_runtime_adapter_contract_preview_marks_unknown_model_family_for_review() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_llama_runtime_adapter_contract_deterministic_and_path_free(
            &temp,
            llama_runtime_adapter_contract_request(
                Some("llama-cli.exe"),
                Some("model.gguf"),
                Some("experimental-family"),
                Some("gguf"),
                Some(8192),
                Some(0),
                Some(8),
                Some(256),
                Some("template"),
            ),
        );
        assert_eq!(result.status, LocalRuntimeAdapterContractStatus::NeedsReview);
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.kind == "model_family_needs_review"));
        assert!(result
            .contract_reasons
            .iter()
            .any(|reason| reason.contains("needs review")));
    }

    #[test]
    fn local_runtime_adapter_contract_preview_marks_gemma_without_chat_template_for_review() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_llama_runtime_adapter_contract_deterministic_and_path_free(
            &temp,
            llama_runtime_adapter_contract_request(
                Some("llama-cli.exe"),
                Some("model.gguf"),
                Some("gemma"),
                Some("gguf"),
                Some(8192),
                Some(0),
                Some(8),
                Some(256),
                None,
            ),
        );
        assert_eq!(result.status, LocalRuntimeAdapterContractStatus::NeedsReview);
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.kind == "chat_template_missing_for_gemma"));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("chat_template")));
    }

    #[test]
    fn local_runtime_adapter_contract_preview_rejects_out_of_range_numeric_inputs() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_llama_runtime_adapter_contract_deterministic_and_path_free(
            &temp,
            llama_runtime_adapter_contract_request(
                Some("llama-cli.exe"),
                Some("model.gguf"),
                Some("llama"),
                Some("gguf"),
                Some(0),
                Some(-2),
                Some(0),
                Some(0),
                Some("template"),
            ),
        );
        assert_eq!(result.status, LocalRuntimeAdapterContractStatus::Blocked);
        assert!(result
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "context_window_tokens_out_of_range"));
        assert!(result
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "gpu_layers_out_of_range"));
        assert!(result
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "threads_out_of_range"));
        assert!(result
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "batch_size_out_of_range"));
    }

    #[test]
    fn local_runtime_adapter_contract_preview_rejects_traversal_like_paths_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["..", "../llama-server.exe", "nested/../llama-server.exe", "nested\\..\\llama-server.exe"] {
            let result = preview_llama_runtime_adapter_contract(
                temp.path(),
                llama_runtime_adapter_contract_request(
                    Some(invalid),
                    Some("model.gguf"),
                    Some("llama"),
                    Some("gguf"),
                    Some(8192),
                    Some(0),
                    Some(8),
                    Some(256),
                    Some("template"),
                ),
            );
            assert!(matches!(result, Err(AegisError::LocalModelRuntimeInvalidPath)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn local_runtime_validation_preview_blocks_when_adapter_contract_is_blocked() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_llama_runtime_validation_deterministic_and_path_free(
            &temp,
            llama_runtime_validation_request(
                Some("   "),
                Some("\t"),
                None,
                Some("gguf"),
                Some(8192),
                Some(0),
                Some(8),
                Some(256),
                Some("template"),
            ),
        );
        assert_eq!(result.status, LocalRuntimeValidationStatus::Blocked);
        assert_eq!(result.adapter_contract_status, LocalRuntimeAdapterContractStatus::Blocked);
        assert_eq!(
            result.missing_inputs,
            vec!["executable_path".to_string(), "model_path".to_string()]
        );
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "executable_path_missing"));
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "model_path_missing"));
        assert!(!temp.path().join(".aegis").exists());
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 0);
    }

    #[test]
    fn local_runtime_validation_preview_rejects_traversal_like_paths_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["..", "../llama-server.exe", "nested/../llama-server.exe", "nested\\..\\llama-server.exe"] {
            for request in [
                llama_runtime_validation_request(
                    Some(invalid),
                    Some("model.gguf"),
                    Some("llama"),
                    Some("gguf"),
                    Some(8192),
                    Some(0),
                    Some(8),
                    Some(256),
                    Some("template"),
                ),
                llama_runtime_validation_request(
                    Some("llama-server.exe"),
                    Some(invalid),
                    Some("llama"),
                    Some("gguf"),
                    Some(8192),
                    Some(0),
                    Some(8),
                    Some(256),
                    Some("template"),
                ),
            ] {
                let result = preview_llama_runtime_validation(temp.path(), request);
                assert!(matches!(result, Err(AegisError::LocalModelRuntimeInvalidPath)));
                assert!(!temp.path().join(".aegis").exists());
            }
        }
    }

    #[test]
    fn local_runtime_validation_preview_blocks_when_executable_file_is_missing() {
        let temp = tempfile::tempdir().unwrap();
        let model_path = temp.path().join("ready-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        let result = assert_llama_runtime_validation_deterministic_and_path_free(
            &temp,
            llama_runtime_validation_request(
                Some("missing-executable.exe"),
                Some(model_path.to_string_lossy().as_ref()),
                Some("llama"),
                Some("gguf"),
                Some(8192),
                Some(0),
                Some(8),
                Some(256),
                Some("template"),
            ),
        );
        assert_eq!(result.status, LocalRuntimeValidationStatus::Blocked);
        assert!(!result.executable_exists);
        assert!(!result.executable_is_file);
        assert_eq!(result.safe_executable_file_name.as_deref(), Some("missing-executable.exe"));
        assert!(result.missing_inputs.contains(&"executable_exists".to_string()));
        assert!(!temp.path().join(".aegis").exists());
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 1);
    }

    #[test]
    fn local_runtime_validation_preview_blocks_when_model_file_is_missing() {
        let temp = tempfile::tempdir().unwrap();
        let executable_path = temp.path().join("llama-server.exe");
        fs::write(&executable_path, "placeholder").unwrap();
        let result = assert_llama_runtime_validation_deterministic_and_path_free(
            &temp,
            llama_runtime_validation_request(
                Some(executable_path.to_string_lossy().as_ref()),
                Some("missing-model.gguf"),
                Some("llama"),
                Some("gguf"),
                Some(8192),
                Some(0),
                Some(8),
                Some(256),
                Some("template"),
            ),
        );
        assert_eq!(result.status, LocalRuntimeValidationStatus::Blocked);
        assert!(result.executable_exists);
        assert!(result.executable_is_file);
        assert!(!result.model_exists);
        assert!(!result.model_is_file);
        assert_eq!(result.safe_model_file_name.as_deref(), Some("missing-model.gguf"));
        assert!(result.missing_inputs.contains(&"model_exists".to_string()));
        assert!(!temp.path().join(".aegis").exists());
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 1);
    }

    #[test]
    fn local_runtime_validation_preview_blocks_when_executable_path_points_to_a_directory() {
        let temp = tempfile::tempdir().unwrap();
        let executable_dir = temp.path().join("llama-server.exe");
        fs::create_dir(&executable_dir).unwrap();
        let model_path = temp.path().join("ready-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        let result = assert_llama_runtime_validation_deterministic_and_path_free(
            &temp,
            llama_runtime_validation_request(
                Some(executable_dir.to_string_lossy().as_ref()),
                Some(model_path.to_string_lossy().as_ref()),
                Some("llama"),
                Some("gguf"),
                Some(8192),
                Some(0),
                Some(8),
                Some(256),
                Some("template"),
            ),
        );
        assert_eq!(result.status, LocalRuntimeValidationStatus::Blocked);
        assert!(result.executable_exists);
        assert!(!result.executable_is_file);
        assert!(result.missing_inputs.contains(&"executable_is_file".to_string()));
        assert!(!temp.path().join(".aegis").exists());
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 2);
    }

    #[test]
    fn local_runtime_validation_preview_blocks_when_model_path_points_to_a_directory() {
        let temp = tempfile::tempdir().unwrap();
        let executable_path = temp.path().join("llama-server.exe");
        fs::write(&executable_path, "placeholder").unwrap();
        let model_dir = temp.path().join("ready-model.gguf");
        fs::create_dir(&model_dir).unwrap();
        let result = assert_llama_runtime_validation_deterministic_and_path_free(
            &temp,
            llama_runtime_validation_request(
                Some(executable_path.to_string_lossy().as_ref()),
                Some(model_dir.to_string_lossy().as_ref()),
                Some("llama"),
                Some("gguf"),
                Some(8192),
                Some(0),
                Some(8),
                Some(256),
                Some("template"),
            ),
        );
        assert_eq!(result.status, LocalRuntimeValidationStatus::Blocked);
        assert!(result.model_exists);
        assert!(!result.model_is_file);
        assert!(result.missing_inputs.contains(&"model_is_file".to_string()));
        assert!(!temp.path().join(".aegis").exists());
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 2);
    }

    #[test]
    fn local_runtime_validation_preview_blocks_when_model_extension_is_not_gguf() {
        let temp = tempfile::tempdir().unwrap();
        let executable_path = temp.path().join("llama-server.exe");
        let model_path = temp.path().join("ready-model.txt");
        fs::write(&executable_path, "placeholder").unwrap();
        fs::write(&model_path, "not gguf").unwrap();
        let result = assert_llama_runtime_validation_deterministic_and_path_free(
            &temp,
            llama_runtime_validation_request(
                Some(executable_path.to_string_lossy().as_ref()),
                Some(model_path.to_string_lossy().as_ref()),
                Some("llama"),
                Some("gguf"),
                Some(8192),
                Some(0),
                Some(8),
                Some(256),
                Some("template"),
            ),
        );
        assert_eq!(result.status, LocalRuntimeValidationStatus::Blocked);
        assert!(result.model_exists);
        assert!(result.model_is_file);
        assert!(!result.model_extension_valid);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "model_extension_invalid"));
        assert!(result.missing_inputs.contains(&"model_extension_valid".to_string()));
    }

    #[test]
    fn local_runtime_validation_preview_needs_review_when_adapter_contract_needs_review_and_files_are_ready() {
        let temp = tempfile::tempdir().unwrap();
        let executable_path = temp.path().join("llama-server.exe");
        let model_path = temp.path().join("ready-model.gguf");
        fs::write(&executable_path, "placeholder").unwrap();
        fs::write(&model_path, "gguf placeholder").unwrap();
        let result = assert_llama_runtime_validation_deterministic_and_path_free(
            &temp,
            llama_runtime_validation_request(
                Some(executable_path.to_string_lossy().as_ref()),
                Some(model_path.to_string_lossy().as_ref()),
                Some("experimental-family"),
                Some("gguf"),
                Some(8192),
                Some(0),
                Some(8),
                Some(256),
                Some("template"),
            ),
        );
        assert_eq!(result.status, LocalRuntimeValidationStatus::NeedsReview);
        assert_eq!(result.adapter_contract_status, LocalRuntimeAdapterContractStatus::NeedsReview);
        assert!(result.missing_inputs.is_empty());
        assert!(result
            .validation_reasons
            .iter()
            .any(|reason| reason.contains("Adapter contract status")));
        assert!(!temp.path().join(".aegis").exists());
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 2);
    }

    #[test]
    fn local_runtime_validation_preview_returns_ready_later_for_existing_gguf_files() {
        let temp = tempfile::tempdir().unwrap();
        let executable_path = temp.path().join("llama-server.exe");
        let model_path = temp.path().join("ready-model.gguf");
        fs::write(&executable_path, "placeholder").unwrap();
        fs::write(&model_path, "gguf placeholder").unwrap();
        let result = assert_llama_runtime_validation_deterministic_and_path_free(
            &temp,
            llama_runtime_validation_request(
                Some(executable_path.to_string_lossy().as_ref()),
                Some(model_path.to_string_lossy().as_ref()),
                Some("llama"),
                Some("gguf"),
                Some(8192),
                Some(0),
                Some(8),
                Some(256),
                Some("template"),
            ),
        );
        assert_eq!(result.status, LocalRuntimeValidationStatus::ValidationReadyLater);
        assert_eq!(result.adapter_contract_status, LocalRuntimeAdapterContractStatus::ContractReadyLater);
        assert!(result.missing_inputs.is_empty());
        assert_eq!(result.safe_executable_file_name.as_deref(), Some("llama-server.exe"));
        assert_eq!(result.safe_model_file_name.as_deref(), Some("ready-model.gguf"));
        assert!(result.validation_reasons.iter().any(|reason| reason.contains("ready later")));
        assert!(!temp.path().join(".aegis").exists());
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 2);
    }

    #[test]
    fn local_runtime_validation_preview_is_deterministic_and_path_free() {
        let temp = tempfile::tempdir().unwrap();
        let executable_path = temp.path().join("llama-server.exe");
        let model_path = temp.path().join("ready-model.gguf");
        fs::write(&executable_path, "placeholder").unwrap();
        fs::write(&model_path, "gguf placeholder").unwrap();
        let request = llama_runtime_validation_request(
            Some(executable_path.to_string_lossy().as_ref()),
            Some(model_path.to_string_lossy().as_ref()),
            Some("llama"),
            Some("gguf"),
            Some(8192),
            Some(0),
            Some(8),
            Some(256),
            Some("template"),
        );
        let first = preview_llama_runtime_validation(temp.path(), request.clone()).unwrap();
        let second = preview_llama_runtime_validation(temp.path(), request).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.status, LocalRuntimeValidationStatus::ValidationReadyLater);
        let debug = format!("{first:?}");
        let json = serde_json::to_string(&first).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert!(!temp.path().join(".aegis").exists());
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 2);
    }

    #[test]
    fn local_runtime_probe_readiness_preview_blocks_when_validation_is_blocked() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_llama_runtime_probe_readiness_deterministic_and_path_free(
            &temp,
            probe_readiness_request(
                llama_runtime_validation_request(
                    Some("   "),
                    Some("\t"),
                    None,
                    Some("gguf"),
                    Some(8192),
                    Some(0),
                    Some(8),
                    Some(256),
                    Some("template"),
                ),
                true,
            ),
        );
        assert_eq!(result.status, LocalRuntimeProbeReadinessStatus::Blocked);
        assert_eq!(result.validation_status, LocalRuntimeValidationStatus::Blocked);
        assert_eq!(
            result.missing_inputs,
            vec!["runtime_validation_ready_later".to_string()]
        );
        assert!(result
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "runtime_validation_not_ready_later"));
        assert!(!temp.path().join(".aegis").exists());
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 0);
    }

    #[test]
    fn local_runtime_probe_readiness_preview_needs_review_when_validation_needs_review() {
        let temp = tempfile::tempdir().unwrap();
        let executable_path = temp.path().join("llama-server.exe");
        let model_path = temp.path().join("ready-model.gguf");
        fs::write(&executable_path, "placeholder").unwrap();
        fs::write(&model_path, "gguf placeholder").unwrap();
        let result = assert_llama_runtime_probe_readiness_deterministic_and_path_free(
            &temp,
            probe_readiness_request(
                llama_runtime_validation_request(
                    Some(executable_path.to_string_lossy().as_ref()),
                    Some(model_path.to_string_lossy().as_ref()),
                    Some("experimental-family"),
                    Some("gguf"),
                    Some(8192),
                    Some(0),
                    Some(8),
                    Some(256),
                    Some("template"),
                ),
                true,
            ),
        );
        assert_eq!(result.status, LocalRuntimeProbeReadinessStatus::NeedsReview);
        assert_eq!(result.validation_status, LocalRuntimeValidationStatus::NeedsReview);
        assert_eq!(
            result.missing_inputs,
            vec!["runtime_validation_ready_later".to_string()]
        );
        assert!(result
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "runtime_validation_not_ready_later"));
        assert!(!temp.path().join(".aegis").exists());
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 2);
    }

    #[test]
    fn local_runtime_probe_readiness_preview_blocks_without_probe_consent_only_after_validation_is_ready_later() {
        let temp = tempfile::tempdir().unwrap();
        let executable_path = temp.path().join("llama-server.exe");
        let model_path = temp.path().join("ready-model.gguf");
        fs::write(&executable_path, "placeholder").unwrap();
        fs::write(&model_path, "gguf placeholder").unwrap();
        let result = assert_llama_runtime_probe_readiness_deterministic_and_path_free(
            &temp,
            probe_readiness_request(
                llama_runtime_validation_request(
                    Some(executable_path.to_string_lossy().as_ref()),
                    Some(model_path.to_string_lossy().as_ref()),
                    Some("llama"),
                    Some("gguf"),
                    Some(8192),
                    Some(0),
                    Some(8),
                    Some(256),
                    Some("template"),
                ),
                false,
            ),
        );
        assert_eq!(result.status, LocalRuntimeProbeReadinessStatus::Blocked);
        assert_eq!(result.validation_status, LocalRuntimeValidationStatus::ValidationReadyLater);
        assert_eq!(result.missing_inputs, vec!["probe_consent".to_string()]);
        assert!(result
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "probe_consent_missing"));
        assert!(!temp.path().join(".aegis").exists());
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 2);
    }

    #[test]
    fn local_runtime_probe_readiness_preview_returns_ready_later_when_validation_is_ready_later_and_probe_consent_is_true() {
        let temp = tempfile::tempdir().unwrap();
        let executable_path = temp.path().join("llama-server.exe");
        let model_path = temp.path().join("ready-model.gguf");
        fs::write(&executable_path, "placeholder").unwrap();
        fs::write(&model_path, "gguf placeholder").unwrap();
        let result = assert_llama_runtime_probe_readiness_deterministic_and_path_free(
            &temp,
            probe_readiness_request(
                llama_runtime_validation_request(
                    Some(executable_path.to_string_lossy().as_ref()),
                    Some(model_path.to_string_lossy().as_ref()),
                    Some("llama"),
                    Some("gguf"),
                    Some(8192),
                    Some(0),
                    Some(8),
                    Some(256),
                    Some("template"),
                ),
                true,
            ),
        );
        assert_eq!(result.status, LocalRuntimeProbeReadinessStatus::ProbeReadyLater);
        assert_eq!(result.validation_status, LocalRuntimeValidationStatus::ValidationReadyLater);
        assert!(result.missing_inputs.is_empty());
        assert_eq!(result.safe_executable_file_name.as_deref(), Some("llama-server.exe"));
        assert_eq!(result.safe_model_file_name.as_deref(), Some("ready-model.gguf"));
        assert_eq!(result.required_inputs, vec![
            "runtime_validation_ready_later".to_string(),
            "probe_consent".to_string(),
        ]);
        assert!(!temp.path().join(".aegis").exists());
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 2);
    }

    #[test]
    fn local_runtime_version_probe_preview_blocks_when_readiness_is_blocked() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_llama_runtime_version_probe_deterministic_and_path_free(
            &temp,
            version_probe_request(
                llama_runtime_validation_request(
                    Some(""),
                    Some(""),
                    None,
                    Some("gguf"),
                    Some(8192),
                    Some(0),
                    Some(8),
                    Some(256),
                    Some("template"),
                ),
                true,
                true,
                Some(1_500),
            ),
        );
        assert_eq!(result.status, LocalRuntimeVersionProbeStatus::Blocked);
        assert_eq!(result.probe_readiness_status, LocalRuntimeProbeReadinessStatus::Blocked);
        assert_eq!(result.validation_status, LocalRuntimeValidationStatus::Blocked);
        assert!(!result.execution_attempted);
        assert_eq!(result.probe_argument, "--version");
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "runtime_validation_not_ready_later"));
        assert_llama_runtime_version_probe_boundary_fields(&result);
        assert!(!temp.path().join(".aegis").exists());
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 0);
    }

    #[test]
    fn local_runtime_version_probe_preview_blocks_when_readiness_needs_review() {
        let temp = tempfile::tempdir().unwrap();
        let executable_path = temp.path().join("llama-server.exe");
        let model_path = temp.path().join("ready-model.gguf");
        fs::write(&executable_path, "placeholder").unwrap();
        fs::write(&model_path, "gguf placeholder").unwrap();
        let result = assert_llama_runtime_version_probe_deterministic_and_path_free(
            &temp,
            version_probe_request(
                llama_runtime_validation_request(
                    Some(executable_path.to_string_lossy().as_ref()),
                    Some(model_path.to_string_lossy().as_ref()),
                    Some("experimental-family"),
                    Some("gguf"),
                    Some(8192),
                    Some(0),
                    Some(8),
                    Some(256),
                    Some("template"),
                ),
                true,
                true,
                Some(1_500),
            ),
        );
        assert_eq!(result.status, LocalRuntimeVersionProbeStatus::Blocked);
        assert_eq!(result.probe_readiness_status, LocalRuntimeProbeReadinessStatus::NeedsReview);
        assert_eq!(result.validation_status, LocalRuntimeValidationStatus::NeedsReview);
        assert!(!result.execution_attempted);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "runtime_validation_not_ready_later"));
        assert_llama_runtime_version_probe_boundary_fields(&result);
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 2);
    }

    #[test]
    fn local_runtime_version_probe_preview_blocks_when_probe_execution_is_not_allowed() {
        let temp = tempfile::tempdir().unwrap();
        let executable_path = temp.path().join("llama-server.exe");
        let model_path = temp.path().join("ready-model.gguf");
        fs::write(&executable_path, "placeholder").unwrap();
        fs::write(&model_path, "gguf placeholder").unwrap();
        let result = assert_llama_runtime_version_probe_deterministic_and_path_free(
            &temp,
            version_probe_request(
                llama_runtime_validation_request(
                    Some(executable_path.to_string_lossy().as_ref()),
                    Some(model_path.to_string_lossy().as_ref()),
                    Some("llama"),
                    Some("gguf"),
                    Some(8192),
                    Some(0),
                    Some(8),
                    Some(256),
                    Some("template"),
                ),
                true,
                false,
                Some(1_500),
            ),
        );
        assert_eq!(result.status, LocalRuntimeVersionProbeStatus::Blocked);
        assert_eq!(result.probe_readiness_status, LocalRuntimeProbeReadinessStatus::ProbeReadyLater);
        assert_eq!(result.validation_status, LocalRuntimeValidationStatus::ValidationReadyLater);
        assert!(!result.execution_attempted);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "probe_execution_not_allowed"));
        assert!(result.next_required_actions.iter().any(|action| action.contains("Enable version-probe execution")));
        assert_llama_runtime_version_probe_boundary_fields(&result);
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 2);
    }

    #[test]
    fn local_runtime_version_probe_preview_reports_probe_succeeded_and_sanitizes_output() {
        let temp = tempfile::tempdir().unwrap();
        let helper = tempfile::tempdir().unwrap();
        let executable_path = version_probe_helper_executable(&helper, "version_probe_ok.exe");
        let model_path = temp.path().join("ready-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        let result = assert_llama_runtime_version_probe_deterministic_and_path_free(
            &temp,
            version_probe_request(
                llama_runtime_validation_request(
                    Some(executable_path.to_string_lossy().as_ref()),
                    Some(model_path.to_string_lossy().as_ref()),
                    Some("llama"),
                    Some("gguf"),
                    Some(8192),
                    Some(0),
                    Some(8),
                    Some(256),
                    Some("template"),
                ),
                true,
                true,
                Some(1_500),
            ),
        );
        assert_eq!(result.status, LocalRuntimeVersionProbeStatus::ProbeSucceeded);
        assert!(result.execution_attempted);
        assert_eq!(result.probe_argument, "--version");
        assert_eq!(result.exit_code, Some(0));
        assert!(result.stdout_truncated);
        assert!(result.stderr_truncated);
        assert!(result.stdout_preview.contains("stdout marker"));
        assert!(result.stderr_preview.contains("stderr marker"));
        let debug = format!("{result:?}");
        let json = serde_json::to_string(&result).unwrap();
        let temp_path = temp.path().to_string_lossy();
        let helper_path = helper.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert!(!debug.contains(helper_path.as_ref()));
        assert!(!json.contains(helper_path.as_ref()));
        assert!(!result.stdout_preview.contains(temp_path.as_ref()));
        assert!(!result.stderr_preview.contains(temp_path.as_ref()));
        assert!(!result.stdout_preview.contains(helper_path.as_ref()));
        assert!(!result.stderr_preview.contains(helper_path.as_ref()));
        assert_eq!(result.safe_executable_file_name.as_deref(), Some("version_probe_ok.exe"));
        assert_eq!(result.safe_model_file_name.as_deref(), Some("ready-model.gguf"));
        assert!(result.no_model_load);
        assert!(result.no_model_path_argument);
        assert!(result.no_llm_call);
        assert!(result.no_runtime_inference);
        assert!(result.no_persistence);
        assert!(result.no_artifact_write);
        assert!(result.no_registry_status_change);
        assert!(result.no_audit_write);
        assert_llama_runtime_version_probe_boundary_fields(&result);
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 1);
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn local_runtime_version_probe_preview_reports_probe_failed_for_nonzero_exit() {
        let temp = tempfile::tempdir().unwrap();
        let helper = tempfile::tempdir().unwrap();
        let executable_path = version_probe_helper_executable(&helper, "version_probe_fail.exe");
        let model_path = temp.path().join("ready-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        let result = assert_llama_runtime_version_probe_deterministic_and_path_free(
            &temp,
            version_probe_request(
                llama_runtime_validation_request(
                    Some(executable_path.to_string_lossy().as_ref()),
                    Some(model_path.to_string_lossy().as_ref()),
                    Some("llama"),
                    Some("gguf"),
                    Some(8192),
                    Some(0),
                    Some(8),
                    Some(256),
                    Some("template"),
                ),
                true,
                true,
                Some(1_500),
            ),
        );
        assert_eq!(result.status, LocalRuntimeVersionProbeStatus::ProbeFailed);
        assert!(result.execution_attempted);
        assert_eq!(result.exit_code, Some(7));
        assert!(result.warnings.iter().any(|warning| warning.kind == "non_zero_exit"));
        assert!(result.stdout_truncated);
        assert!(result.stderr_truncated);
        assert_llama_runtime_version_probe_boundary_fields(&result);
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 1);
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn local_runtime_version_probe_preview_reports_timed_out_for_slow_executable() {
        let temp = tempfile::tempdir().unwrap();
        let helper = tempfile::tempdir().unwrap();
        let executable_path = version_probe_helper_executable(&helper, "version_probe_slow.exe");
        let model_path = temp.path().join("ready-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        let result = assert_llama_runtime_version_probe_deterministic_and_path_free(
            &temp,
            version_probe_request(
                llama_runtime_validation_request(
                    Some(executable_path.to_string_lossy().as_ref()),
                    Some(model_path.to_string_lossy().as_ref()),
                    Some("llama"),
                    Some("gguf"),
                    Some(8192),
                    Some(0),
                    Some(8),
                    Some(256),
                    Some("template"),
                ),
                true,
                true,
                Some(50),
            ),
        );
        assert_eq!(result.status, LocalRuntimeVersionProbeStatus::TimedOut);
        assert!(result.execution_attempted);
        assert!(result.warnings.iter().any(|warning| warning.kind == "timed_out"));
        assert!(result.stdout_truncated);
        assert!(result.stderr_truncated);
        assert_llama_runtime_version_probe_boundary_fields(&result);
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 1);
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn local_runtime_version_probe_preview_reports_spawn_error_without_paths() {
        let temp = tempfile::tempdir().unwrap();
        let helper = tempfile::tempdir().unwrap();
        let executable_path = helper.path().join("broken-version-probe.exe");
        fs::write(&executable_path, "not an executable").unwrap();
        let model_path = temp.path().join("ready-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        let result = assert_llama_runtime_version_probe_deterministic_and_path_free(
            &temp,
            version_probe_request(
                llama_runtime_validation_request(
                    Some(executable_path.to_string_lossy().as_ref()),
                    Some(model_path.to_string_lossy().as_ref()),
                    Some("llama"),
                    Some("gguf"),
                    Some(8192),
                    Some(0),
                    Some(8),
                    Some(256),
                    Some("template"),
                ),
                true,
                true,
                Some(1_500),
            ),
        );
        assert_eq!(result.status, LocalRuntimeVersionProbeStatus::Blocked);
        assert!(!result.execution_attempted);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "probe_start_failed"));
        let debug = format!("{result:?}");
        let json = serde_json::to_string(&result).unwrap();
        let temp_path = temp.path().to_string_lossy();
        let helper_path = helper.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert!(!debug.contains(helper_path.as_ref()));
        assert!(!json.contains(helper_path.as_ref()));
        assert_llama_runtime_version_probe_boundary_fields(&result);
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn local_runtime_version_probe_preview_rejects_traversal_like_paths_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["..", "../probe.exe", "nested/../probe.exe", "nested\\..\\probe.exe"] {
            let result = run_llama_runtime_version_probe(
                temp.path(),
                version_probe_request(
                    llama_runtime_validation_request(
                        Some(invalid),
                        Some("model.gguf"),
                        Some("llama"),
                        Some("gguf"),
                        Some(8192),
                        Some(0),
                        Some(8),
                        Some(256),
                        Some("template"),
                    ),
                    true,
                    true,
                    Some(1_500),
                ),
            );
            assert!(matches!(result, Err(AegisError::LocalModelRuntimeInvalidPath)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn local_runtime_capability_preview_blocks_when_version_probe_is_blocked() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_llama_runtime_capability_deterministic_and_path_free(
            &temp,
            capability_request(
                llama_runtime_validation_request(
                    Some(""),
                    Some(""),
                    None,
                    Some("gguf"),
                    Some(8192),
                    Some(0),
                    Some(8),
                    Some(256),
                    Some("template"),
                ),
                true,
                true,
                Some(1_500),
            ),
        );
        assert_eq!(result.status, LocalRuntimeCapabilityStatus::Blocked);
        assert_eq!(result.version_probe_status, LocalRuntimeVersionProbeStatus::Blocked);
        assert!(!result.version_probe_execution_attempted);
        assert!(!result.version_probe_timed_out);
        assert!(!result.inferred_runtime_available);
        assert!(result
            .capability_reasons
            .iter()
            .any(|reason| reason.contains("wrapped version probe is ready later")));
        assert_llama_runtime_capability_boundary_fields(&result);
        assert!(!temp.path().join(".aegis").exists());
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 0);
    }

    #[test]
    fn local_runtime_capability_preview_needs_review_when_version_probe_failed() {
        let temp = tempfile::tempdir().unwrap();
        let helper = tempfile::tempdir().unwrap();
        let executable_path = version_probe_helper_executable(&helper, "version_probe_fail.exe");
        let model_path = temp.path().join("ready-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        let result = assert_llama_runtime_capability_deterministic_and_path_free(
            &temp,
            capability_request(
                llama_runtime_validation_request(
                    Some(executable_path.to_string_lossy().as_ref()),
                    Some(model_path.to_string_lossy().as_ref()),
                    Some("llama"),
                    Some("gguf"),
                    Some(8192),
                    Some(0),
                    Some(8),
                    Some(256),
                    Some("template"),
                ),
                true,
                true,
                Some(1_500),
            ),
        );
        assert_eq!(result.status, LocalRuntimeCapabilityStatus::NeedsReview);
        assert_eq!(result.version_probe_status, LocalRuntimeVersionProbeStatus::ProbeFailed);
        assert!(result.version_probe_execution_attempted);
        assert!(!result.version_probe_timed_out);
        assert_eq!(result.version_probe_exit_code, Some(7));
        assert!(!result.inferred_runtime_available);
        assert!(result
            .capability_reasons
            .iter()
            .any(|reason| reason.contains("wrapped version probe failed")));
        assert_llama_runtime_capability_boundary_fields(&result);
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 1);
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn local_runtime_capability_preview_needs_review_when_version_probe_times_out() {
        let temp = tempfile::tempdir().unwrap();
        let helper = tempfile::tempdir().unwrap();
        let executable_path = version_probe_helper_executable(&helper, "version_probe_slow.exe");
        let model_path = temp.path().join("ready-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        let result = assert_llama_runtime_capability_deterministic_and_path_free(
            &temp,
            capability_request(
                llama_runtime_validation_request(
                    Some(executable_path.to_string_lossy().as_ref()),
                    Some(model_path.to_string_lossy().as_ref()),
                    Some("llama"),
                    Some("gguf"),
                    Some(8192),
                    Some(0),
                    Some(8),
                    Some(256),
                    Some("template"),
                ),
                true,
                true,
                Some(50),
            ),
        );
        assert_eq!(result.status, LocalRuntimeCapabilityStatus::NeedsReview);
        assert_eq!(result.version_probe_status, LocalRuntimeVersionProbeStatus::TimedOut);
        assert!(result.version_probe_execution_attempted);
        assert!(result.version_probe_timed_out);
        assert!(!result.inferred_runtime_available);
        assert!(result
            .capability_reasons
            .iter()
            .any(|reason| reason.contains("timed out")));
        assert_llama_runtime_capability_boundary_fields(&result);
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 1);
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn local_runtime_capability_preview_is_ready_later_when_version_probe_succeeds_without_obvious_version_text() {
        let temp = tempfile::tempdir().unwrap();
        let helper = tempfile::tempdir().unwrap();
        let executable_path = version_probe_helper_executable(&helper, "version_probe_ok.exe");
        let model_path = temp.path().join("ready-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        let result = assert_llama_runtime_capability_deterministic_and_path_free(
            &temp,
            capability_request(
                llama_runtime_validation_request(
                    Some(executable_path.to_string_lossy().as_ref()),
                    Some(model_path.to_string_lossy().as_ref()),
                    Some("llama"),
                    Some("gguf"),
                    Some(8192),
                    Some(0),
                    Some(8),
                    Some(256),
                    Some("template"),
                ),
                true,
                true,
                Some(1_500),
            ),
        );
        assert_eq!(result.status, LocalRuntimeCapabilityStatus::CapabilityReadyLater);
        assert_eq!(result.version_probe_status, LocalRuntimeVersionProbeStatus::ProbeSucceeded);
        assert!(result.version_probe_execution_attempted);
        assert!(!result.version_probe_timed_out);
        assert!(result.inferred_runtime_available);
        assert!(result.inferred_version_text.is_none());
        assert!(result
            .capability_reasons
            .iter()
            .any(|reason| reason.contains("ready later")));
        assert_llama_runtime_capability_boundary_fields(&result);
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 1);
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn local_runtime_capability_preview_extracts_version_text_and_sanitizes_output() {
        let temp = tempfile::tempdir().unwrap();
        let helper = tempfile::tempdir().unwrap();
        let model_path = temp.path().join("ready-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        let model_path_literal = format!("{:?}", model_path.to_string_lossy().to_string());
        let source = r#"
use std::{env, thread, time::Duration};

fn main() {
    let args: Vec<String> = env::args().collect();
    let exe_name = env::current_exe()
        .ok()
        .and_then(|path| path.file_name().and_then(|value| value.to_str()).map(|value| value.to_string()))
        .unwrap_or_default();
    println!("stdout marker");
    println!("args={}", args.join(" | "));
    println!("model_path={}", MODEL_PATH);
    println!("llama.cpp version 1.2.3");
    println!("exe_name={}", exe_name);
    println!("{}", "S".repeat(5000));
    eprintln!("stderr marker");
    eprintln!("args={}", args.join(" | "));
    eprintln!("model_path={}", MODEL_PATH);
    eprintln!("llama.cpp version 1.2.3");
    eprintln!("exe_name={}", exe_name);
    eprintln!("{}", "E".repeat(5000));
    if exe_name.contains("slow") {
        thread::sleep(Duration::from_millis(750));
    }
    if exe_name.contains("fail") {
        std::process::exit(7);
    }
}
"#
        .replace("MODEL_PATH", &model_path_literal);
        let executable_path =
            version_probe_helper_executable_with_source(&helper, "version_probe_version.exe", &source);
        let result = assert_llama_runtime_capability_deterministic_and_path_free(
            &temp,
            capability_request(
                llama_runtime_validation_request(
                    Some(executable_path.to_string_lossy().as_ref()),
                    Some(model_path.to_string_lossy().as_ref()),
                    Some("llama"),
                    Some("gguf"),
                    Some(8192),
                    Some(0),
                    Some(8),
                    Some(256),
                    Some("template"),
                ),
                true,
                true,
                Some(1_500),
            ),
        );
        assert_eq!(result.status, LocalRuntimeCapabilityStatus::CapabilityReadyLater);
        assert_eq!(result.version_probe_status, LocalRuntimeVersionProbeStatus::ProbeSucceeded);
        assert!(result.inferred_runtime_available);
        assert_eq!(result.inferred_version_text.as_deref(), Some("1.2.3"));
        assert!(result.version_probe_stdout_preview.contains("args="));
        assert!(result.version_probe_stdout_preview.contains("--version"));
        assert!(result.version_probe_stdout_preview.contains("llama.cpp version 1.2.3"));
        assert!(result.version_probe_stderr_preview.contains("llama.cpp version 1.2.3"));
        assert!(!result.version_probe_stdout_preview.contains(model_path.to_string_lossy().as_ref()));
        assert!(!result.version_probe_stderr_preview.contains(model_path.to_string_lossy().as_ref()));
        let debug = format!("{result:?}");
        let json = serde_json::to_string(&result).unwrap();
        let temp_path = temp.path().to_string_lossy();
        let helper_path = helper.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert!(!debug.contains(helper_path.as_ref()));
        assert!(!json.contains(helper_path.as_ref()));
        assert!(result
            .capability_reasons
            .iter()
            .any(|reason| reason.contains("Detected obvious version text: 1.2.3")));
        assert_llama_runtime_capability_boundary_fields(&result);
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 1);
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn local_runtime_smoke_readiness_preview_blocks_when_capability_is_blocked() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_llama_runtime_smoke_readiness_deterministic_and_path_free(
            &temp,
            smoke_readiness_request(
                capability_request(
                    llama_runtime_validation_request(
                        Some(""),
                        Some(""),
                        None,
                        Some("gguf"),
                        Some(8192),
                        Some(0),
                        Some(8),
                        Some(256),
                        Some("template"),
                    ),
                    true,
                    true,
                    Some(1_500),
                ),
                true,
                Some("diagnostic smoke prompt"),
                Some(0),
                Some(0),
            ),
        );
        assert_eq!(result.status, LocalRuntimeSmokeReadinessStatus::Blocked);
        assert_eq!(result.capability_status, LocalRuntimeCapabilityStatus::Blocked);
        assert_eq!(result.version_probe_status, LocalRuntimeVersionProbeStatus::Blocked);
        assert!(result.missing_inputs.contains(&"capability_ready_later".to_string()));
        assert_llama_runtime_smoke_readiness_boundary_fields(&result);
        assert!(!temp.path().join(".aegis").exists());
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 0);
    }

    #[test]
    fn local_runtime_smoke_readiness_preview_needs_review_when_capability_needs_review() {
        let temp = tempfile::tempdir().unwrap();
        let helper = tempfile::tempdir().unwrap();
        let executable_path = version_probe_helper_executable(&helper, "version_probe_fail.exe");
        let model_path = temp.path().join("ready-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        let result = assert_llama_runtime_smoke_readiness_deterministic_and_path_free(
            &temp,
            smoke_readiness_request(
                capability_request(
                    llama_runtime_validation_request(
                        Some(executable_path.to_string_lossy().as_ref()),
                        Some(model_path.to_string_lossy().as_ref()),
                        Some("llama"),
                        Some("gguf"),
                        Some(8192),
                        Some(0),
                        Some(8),
                        Some(256),
                        Some("template"),
                    ),
                    true,
                    true,
                    Some(1_500),
                ),
                true,
                Some("diagnostic smoke prompt"),
                Some(8),
                Some(3_000),
            ),
        );
        assert_eq!(result.status, LocalRuntimeSmokeReadinessStatus::NeedsReview);
        assert_eq!(result.capability_status, LocalRuntimeCapabilityStatus::NeedsReview);
        assert_eq!(result.version_probe_status, LocalRuntimeVersionProbeStatus::ProbeFailed);
        assert_llama_runtime_smoke_readiness_boundary_fields(&result);
        assert!(result.missing_inputs.contains(&"capability_ready_later".to_string()));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn local_runtime_smoke_readiness_preview_blocks_when_smoke_consent_is_missing() {
        let temp = tempfile::tempdir().unwrap();
        let helper = tempfile::tempdir().unwrap();
        let executable_path = version_probe_helper_executable(&helper, "version_probe_ok.exe");
        let model_path = temp.path().join("ready-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        let result = assert_llama_runtime_smoke_readiness_deterministic_and_path_free(
            &temp,
            smoke_readiness_request(
                capability_request(
                    llama_runtime_validation_request(
                        Some(executable_path.to_string_lossy().as_ref()),
                        Some(model_path.to_string_lossy().as_ref()),
                        Some("llama"),
                        Some("gguf"),
                        Some(8192),
                        Some(0),
                        Some(8),
                        Some(256),
                        Some("template"),
                    ),
                    true,
                    true,
                    Some(1_500),
                ),
                false,
                Some("diagnostic smoke prompt"),
                Some(8),
                Some(3_000),
            ),
        );
        assert_eq!(result.status, LocalRuntimeSmokeReadinessStatus::Blocked);
        assert_eq!(result.capability_status, LocalRuntimeCapabilityStatus::CapabilityReadyLater);
        assert!(result.missing_inputs.contains(&"smoke_consent".to_string()));
        assert_llama_runtime_smoke_readiness_boundary_fields(&result);
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn local_runtime_smoke_readiness_preview_blocks_when_diagnostic_prompt_is_blank() {
        let temp = tempfile::tempdir().unwrap();
        let helper = tempfile::tempdir().unwrap();
        let executable_path = version_probe_helper_executable(&helper, "version_probe_ok.exe");
        let model_path = temp.path().join("ready-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        let result = assert_llama_runtime_smoke_readiness_deterministic_and_path_free(
            &temp,
            smoke_readiness_request(
                capability_request(
                    llama_runtime_validation_request(
                        Some(executable_path.to_string_lossy().as_ref()),
                        Some(model_path.to_string_lossy().as_ref()),
                        Some("llama"),
                        Some("gguf"),
                        Some(8192),
                        Some(0),
                        Some(8),
                        Some(256),
                        Some("template"),
                    ),
                    true,
                    true,
                    Some(1_500),
                ),
                true,
                Some("   "),
                Some(8),
                Some(3_000),
            ),
        );
        assert_eq!(result.status, LocalRuntimeSmokeReadinessStatus::Blocked);
        assert_eq!(result.capability_status, LocalRuntimeCapabilityStatus::CapabilityReadyLater);
        assert_eq!(result.normalized_diagnostic_prompt, "");
        assert_eq!(result.diagnostic_prompt_char_count, 0);
        assert!(result.missing_inputs.contains(&"diagnostic_prompt".to_string()));
        assert_llama_runtime_smoke_readiness_boundary_fields(&result);
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn local_runtime_smoke_readiness_preview_is_ready_later_when_capability_ready_and_consent_and_prompt_present() {
        let temp = tempfile::tempdir().unwrap();
        let helper = tempfile::tempdir().unwrap();
        let executable_path = version_probe_helper_executable(&helper, "version_probe_ok.exe");
        let model_path = temp.path().join("ready-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        let result = assert_llama_runtime_smoke_readiness_deterministic_and_path_free(
            &temp,
            smoke_readiness_request(
                capability_request(
                    llama_runtime_validation_request(
                        Some(executable_path.to_string_lossy().as_ref()),
                        Some(model_path.to_string_lossy().as_ref()),
                        Some("llama"),
                        Some("gguf"),
                        Some(8192),
                        Some(0),
                        Some(8),
                        Some(256),
                        Some("template"),
                    ),
                    true,
                    true,
                    Some(1_500),
                ),
                true,
                Some("  smoke prompt for readiness  "),
                Some(8),
                Some(3_000),
            ),
        );
        assert_eq!(result.status, LocalRuntimeSmokeReadinessStatus::SmokeReadyLater);
        assert_eq!(result.capability_status, LocalRuntimeCapabilityStatus::CapabilityReadyLater);
        assert!(result.missing_inputs.is_empty());
        assert_eq!(result.normalized_diagnostic_prompt, "smoke prompt for readiness");
        assert_eq!(result.diagnostic_prompt_char_count, "smoke prompt for readiness".chars().count());
        assert_llama_runtime_smoke_readiness_boundary_fields(&result);
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn local_runtime_smoke_readiness_preview_clamps_prompt_tokens_and_timeout_and_remains_path_free() {
        let temp = tempfile::tempdir().unwrap();
        let helper = tempfile::tempdir().unwrap();
        let executable_path = smoke_readiness_helper_executable(&helper, "version_probe_ready.exe");
        let model_path = temp.path().join("ready-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        let long_prompt = "Smoke readiness prompt ".repeat(20);
        let result = assert_llama_runtime_smoke_readiness_deterministic_and_path_free(
            &temp,
            smoke_readiness_request(
                capability_request(
                    llama_runtime_validation_request(
                        Some(executable_path.to_string_lossy().as_ref()),
                        Some(model_path.to_string_lossy().as_ref()),
                        Some("llama"),
                        Some("gguf"),
                        Some(8192),
                        Some(0),
                        Some(8),
                        Some(256),
                        Some("template"),
                    ),
                    true,
                    true,
                    Some(1_500),
                ),
                true,
                Some(&long_prompt),
                Some(0),
                Some(0),
            ),
        );
        assert_eq!(result.status, LocalRuntimeSmokeReadinessStatus::SmokeReadyLater);
        assert_eq!(result.capability_status, LocalRuntimeCapabilityStatus::CapabilityReadyLater);
        assert_eq!(result.max_output_tokens, LOCAL_RUNTIME_SMOKE_MIN_MAX_OUTPUT_TOKENS);
        assert_eq!(result.timeout_ms, LOCAL_RUNTIME_SMOKE_MIN_TIMEOUT_MS);
        assert_eq!(result.normalized_diagnostic_prompt.chars().count(), LOCAL_RUNTIME_SMOKE_PROMPT_LIMIT);
        assert!(result.warnings.iter().any(|warning| warning.kind == "prompt_truncated"));
        assert!(!helper.path().join("smoke_readiness_unexpected_call.txt").exists());
        let debug = format!("{result:?}");
        let json = serde_json::to_string(&result).unwrap();
        let temp_path = temp.path().to_string_lossy();
        let helper_path = helper.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert!(!debug.contains(helper_path.as_ref()));
        assert!(!json.contains(helper_path.as_ref()));
        assert_llama_runtime_smoke_readiness_boundary_fields(&result);
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn local_runtime_smoke_execution_plan_preview_blocks_when_smoke_readiness_is_blocked() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_llama_runtime_smoke_execution_plan_deterministic_and_path_free(
            &temp,
            smoke_execution_plan_request(
                capability_request(
                    llama_runtime_validation_request(
                        Some(""),
                        Some(""),
                        None,
                        Some("gguf"),
                        Some(8192),
                        Some(0),
                        Some(8),
                        Some(256),
                        Some("template"),
                    ),
                    true,
                    true,
                    Some(1_500),
                ),
                false,
                Some("Smoke execution plan prompt."),
                Some(128),
                Some(1_500),
            ),
        );
        assert_eq!(result.status, LocalRuntimeSmokeExecutionPlanStatus::Blocked);
        assert_eq!(
            result.smoke_readiness_status,
            LocalRuntimeSmokeReadinessStatus::Blocked
        );
        assert_eq!(result.missing_inputs, vec!["capability_ready_later".to_string()]);
        assert_eq!(result.planned_operation, "future_diagnostic_smoke_inference");
        assert_eq!(
            result.planned_inputs,
            vec![
                "llama_runtime_executable".to_string(),
                "gguf_model_file".to_string(),
                "diagnostic_prompt".to_string(),
                "max_output_tokens".to_string(),
                "timeout_ms".to_string(),
            ]
        );
        assert_eq!(
            result.planned_safe_arguments,
            vec![
                "<executable>".to_string(),
                "-m".to_string(),
                "<model-file>".to_string(),
                "-p".to_string(),
                "<diagnostic-prompt>".to_string(),
                "-n".to_string(),
                "<max-output-tokens>".to_string(),
            ]
        );
        assert_eq!(
            result.planned_outputs,
            vec![
                "diagnostic stdout preview".to_string(),
                "diagnostic stderr preview".to_string(),
                "exit status".to_string(),
                "duration_ms".to_string(),
                "diagnostic-only status".to_string(),
                "no artifact output".to_string(),
                "no answer output".to_string(),
            ]
        );
        assert_llama_runtime_smoke_execution_plan_boundary_fields(&result);
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn local_runtime_smoke_execution_plan_preview_needs_review_when_smoke_readiness_needs_review() {
        let temp = tempfile::tempdir().unwrap();
        let helper = tempfile::tempdir().unwrap();
        let source = r#"
use std::{env, thread, time::Duration};

fn main() {
    let args: Vec<String> = env::args().collect();
    let exe_name = env::current_exe()
        .ok()
        .and_then(|path| path.file_name().and_then(|value| value.to_str()).map(|value| value.to_string()))
        .unwrap_or_default();
    println!("stdout marker");
    println!("args={}", args.join(" | "));
    println!("llama.cpp version 1.2.3");
    println!("exe_name={}", exe_name);
    println!("{}", "S".repeat(5000));
    eprintln!("stderr marker");
    eprintln!("args={}", args.join(" | "));
    eprintln!("llama.cpp version 1.2.3");
    eprintln!("exe_name={}", exe_name);
    eprintln!("{}", "E".repeat(5000));
    if exe_name.contains("slow") {
        thread::sleep(Duration::from_millis(750));
    }
    if exe_name.contains("fail") {
        std::process::exit(7);
    }
}
"#;
        let executable_path =
            version_probe_helper_executable_with_source(&helper, "version_probe_fail.exe", &source);
        let model_path = temp.path().join("ready-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        let result = assert_llama_runtime_smoke_execution_plan_deterministic_and_path_free(
            &temp,
            smoke_execution_plan_request(
                capability_request(
                    llama_runtime_validation_request(
                        Some(executable_path.to_string_lossy().as_ref()),
                        Some(model_path.to_string_lossy().as_ref()),
                        Some("llama"),
                        Some("gguf"),
                        Some(8192),
                        Some(0),
                        Some(8),
                        Some(256),
                        Some("template"),
                    ),
                    true,
                    true,
                    Some(1_500),
                ),
                true,
                Some("Smoke execution plan prompt."),
                Some(128),
                Some(1_500),
            ),
        );
        assert_eq!(result.status, LocalRuntimeSmokeExecutionPlanStatus::NeedsReview);
        assert_eq!(
            result.smoke_readiness_status,
            LocalRuntimeSmokeReadinessStatus::NeedsReview
        );
        assert_eq!(result.missing_inputs, vec!["capability_ready_later".to_string()]);
        let debug = format!("{result:?}");
        let json = serde_json::to_string(&result).unwrap();
        let temp_path = temp.path().to_string_lossy();
        let helper_path = helper.path().to_string_lossy();
        let model_path_text = model_path.to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert!(!debug.contains(helper_path.as_ref()));
        assert!(!json.contains(helper_path.as_ref()));
        assert!(!debug.contains(model_path_text.as_ref()));
        assert!(!json.contains(model_path_text.as_ref()));
        assert_llama_runtime_smoke_execution_plan_boundary_fields(&result);
        assert!(result
            .plan_reasons
            .iter()
            .any(|reason| reason.contains("needs review")));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn local_runtime_smoke_execution_plan_preview_is_ready_later_when_smoke_readiness_is_ready_later() {
        let temp = tempfile::tempdir().unwrap();
        let helper = tempfile::tempdir().unwrap();
        let source = r#"
use std::{env, thread, time::Duration};

fn main() {
    let args: Vec<String> = env::args().collect();
    let exe_name = env::current_exe()
        .ok()
        .and_then(|path| path.file_name().and_then(|value| value.to_str()).map(|value| value.to_string()))
        .unwrap_or_default();
    println!("stdout marker");
    println!("args={}", args.join(" | "));
    println!("llama.cpp version 1.2.3");
    println!("exe_name={}", exe_name);
    println!("{}", "S".repeat(5000));
    eprintln!("stderr marker");
    eprintln!("args={}", args.join(" | "));
    eprintln!("llama.cpp version 1.2.3");
    eprintln!("exe_name={}", exe_name);
    eprintln!("{}", "E".repeat(5000));
    if exe_name.contains("slow") {
        thread::sleep(Duration::from_millis(750));
    }
    if exe_name.contains("fail") {
        std::process::exit(7);
    }
}
"#;
        let executable_path =
            version_probe_helper_executable_with_source(&helper, "version_probe_ready.exe", &source);
        let model_path = temp.path().join("ready-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        let result = assert_llama_runtime_smoke_execution_plan_deterministic_and_path_free(
            &temp,
            smoke_execution_plan_request(
                capability_request(
                    llama_runtime_validation_request(
                        Some(executable_path.to_string_lossy().as_ref()),
                        Some(model_path.to_string_lossy().as_ref()),
                        Some("llama"),
                        Some("gguf"),
                        Some(8192),
                        Some(0),
                        Some(8),
                        Some(256),
                        Some("template"),
                    ),
                    true,
                    true,
                    Some(1_500),
                ),
                true,
                Some("Smoke execution plan prompt."),
                Some(128),
                Some(1_500),
            ),
        );
        assert_eq!(result.status, LocalRuntimeSmokeExecutionPlanStatus::PlanReadyLater);
        assert_eq!(
            result.smoke_readiness_status,
            LocalRuntimeSmokeReadinessStatus::SmokeReadyLater
        );
        assert_eq!(result.missing_inputs, Vec::<String>::new());
        assert_eq!(result.planned_operation, "future_diagnostic_smoke_inference");
        let debug = format!("{result:?}");
        let json = serde_json::to_string(&result).unwrap();
        let temp_path = temp.path().to_string_lossy();
        let helper_path = helper.path().to_string_lossy();
        let model_path_text = model_path.to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert!(!debug.contains(helper_path.as_ref()));
        assert!(!json.contains(helper_path.as_ref()));
        assert!(!debug.contains(model_path_text.as_ref()));
        assert!(!json.contains(model_path_text.as_ref()));
        assert!(result
            .planned_safe_arguments
            .iter()
            .all(|value| !value.contains(temp.path().to_string_lossy().as_ref())));
        assert!(result
            .planned_safe_arguments
            .iter()
            .all(|value| !value.contains(helper.path().to_string_lossy().as_ref())));
        assert!(result
            .planned_safe_arguments
            .iter()
            .all(|value| !value.contains(model_path.to_string_lossy().as_ref())));
        assert!(result
            .planned_outputs
            .iter()
            .any(|value| value == "diagnostic stdout preview"));
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 1);
        assert!(!helper.path().join("smoke_readiness_unexpected_call.txt").exists());
        assert_llama_runtime_smoke_execution_plan_boundary_fields(&result);
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn local_runtime_smoke_diagnostic_preview_blocks_when_smoke_execution_plan_is_blocked() {
        let temp = tempfile::tempdir().unwrap();
        let result = assert_llama_runtime_smoke_diagnostic_deterministic_and_path_free(
            &temp,
            smoke_diagnostic_request(
                capability_request(
                    llama_runtime_validation_request(
                        Some(""),
                        Some(""),
                        None,
                        Some("gguf"),
                        Some(8192),
                        Some(0),
                        Some(8),
                        Some(256),
                        Some("template"),
                    ),
                    true,
                    true,
                    Some(1_500),
                ),
                true,
                Some("Diagnostic smoke prompt."),
                Some(8),
                Some(1_500),
                true,
            ),
        );
        assert_eq!(result.status, LocalRuntimeSmokeDiagnosticStatus::Blocked);
        assert_eq!(
            result.smoke_execution_plan_status,
            LocalRuntimeSmokeExecutionPlanStatus::Blocked
        );
        assert!(!result.execution_attempted);
        assert!(result
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "smoke_execution_plan_not_ready_later"));
        assert_llama_runtime_smoke_diagnostic_boundary_fields(&result);
        assert!(!temp.path().join(".aegis").exists());
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 0);
    }

    #[test]
    fn local_runtime_smoke_diagnostic_preview_blocks_when_smoke_execution_plan_needs_review() {
        let temp = tempfile::tempdir().unwrap();
        let helper = tempfile::tempdir().unwrap();
        let executable_path = smoke_readiness_helper_executable(&helper, "smoke_diagnostic_plan.exe");
        let model_path = temp.path().join("ready-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        let result = assert_llama_runtime_smoke_diagnostic_deterministic_and_path_free(
            &temp,
            smoke_diagnostic_request(
                capability_request(
                    llama_runtime_validation_request(
                        Some(executable_path.to_string_lossy().as_ref()),
                        Some(model_path.to_string_lossy().as_ref()),
                        Some("experimental-family"),
                        Some("gguf"),
                        Some(8192),
                        Some(0),
                        Some(8),
                        Some(256),
                        Some("template"),
                    ),
                    true,
                    true,
                    Some(1_500),
                ),
                true,
                Some("Diagnostic smoke prompt."),
                Some(8),
                Some(1_500),
                true,
            ),
        );
        assert_eq!(result.status, LocalRuntimeSmokeDiagnosticStatus::Blocked);
        assert_eq!(
            result.smoke_execution_plan_status,
            LocalRuntimeSmokeExecutionPlanStatus::Blocked
        );
        assert!(!result.execution_attempted);
        assert!(result
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "smoke_execution_plan_not_ready_later"));
        assert_llama_runtime_smoke_diagnostic_boundary_fields(&result);
        assert!(!temp.path().join(".aegis").exists());
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 1);
    }

    #[test]
    fn local_runtime_smoke_diagnostic_preview_blocks_when_smoke_execution_consent_is_missing() {
        let temp = tempfile::tempdir().unwrap();
        let helper = tempfile::tempdir().unwrap();
        let executable_path = smoke_readiness_helper_executable(&helper, "smoke_diagnostic_ready.exe");
        let model_path = temp.path().join("ready-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        let result = assert_llama_runtime_smoke_diagnostic_deterministic_and_path_free(
            &temp,
            smoke_diagnostic_request(
                capability_request(
                    llama_runtime_validation_request(
                        Some(executable_path.to_string_lossy().as_ref()),
                        Some(model_path.to_string_lossy().as_ref()),
                        Some("llama"),
                        Some("gguf"),
                        Some(8192),
                        Some(0),
                        Some(8),
                        Some(256),
                        Some("template"),
                    ),
                    true,
                    true,
                    Some(1_500),
                ),
                true,
                Some("Diagnostic smoke prompt."),
                Some(8),
                Some(1_500),
                false,
            ),
        );
        assert_eq!(result.status, LocalRuntimeSmokeDiagnosticStatus::Blocked);
        assert_eq!(
            result.smoke_execution_plan_status,
            LocalRuntimeSmokeExecutionPlanStatus::PlanReadyLater
        );
        assert!(!result.execution_attempted);
        assert!(result
            .blockers
            .iter()
            .any(|blocker| blocker.kind == "smoke_execution_consent_missing"));
        assert!(result
            .next_required_actions
            .iter()
            .any(|action| action.contains("consent")));
        assert_llama_runtime_smoke_diagnostic_boundary_fields(&result);
        assert!(!temp.path().join(".aegis").exists());
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 1);
    }

    #[test]
    fn local_runtime_smoke_diagnostic_preview_succeeds_with_redacted_and_bounded_output() {
        let temp = tempfile::tempdir().unwrap();
        let helper = tempfile::tempdir().unwrap();
        let source = r#"
use std::{env, thread, time::Duration};

fn main() {
    let args: Vec<String> = env::args().collect();
    let exe_name = env::current_exe()
        .ok()
        .and_then(|path| path.file_name().and_then(|value| value.to_str()).map(|value| value.to_string()))
        .unwrap_or_default();
    println!("stdout marker");
    println!("args={}", args.join(" | "));
    println!("llama.cpp version 1.2.3");
    println!("exe_name={}", exe_name);
    println!("{}", "S".repeat(5000));
    eprintln!("stderr marker");
    eprintln!("args={}", args.join(" | "));
    eprintln!("llama.cpp version 1.2.3");
    eprintln!("exe_name={}", exe_name);
    eprintln!("{}", "E".repeat(5000));
    if exe_name.contains("slow") {
        thread::sleep(Duration::from_millis(750));
    }
}
"#;
        let executable_path =
            version_probe_helper_executable_with_source(&helper, "smoke_diagnostic_ready.exe", &source);
        let model_path = temp.path().join("ready-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        let result = assert_llama_runtime_smoke_diagnostic_deterministic_and_path_free(
            &temp,
            smoke_diagnostic_request(
                capability_request(
                    llama_runtime_validation_request(
                        Some(executable_path.to_string_lossy().as_ref()),
                        Some(model_path.to_string_lossy().as_ref()),
                        Some("llama"),
                        Some("gguf"),
                        Some(8192),
                        Some(0),
                        Some(8),
                        Some(256),
                        Some("template"),
                    ),
                    true,
                    true,
                    Some(1_500),
                ),
                true,
                Some("Say READY in one short sentence."),
                Some(8),
                Some(1_500),
                true,
            ),
        );
        assert_eq!(result.status, LocalRuntimeSmokeDiagnosticStatus::SmokeSucceeded);
        assert!(result.execution_attempted);
        assert!(result.stdout_truncated);
        assert!(result.stderr_truncated);
        assert_eq!(result.exit_code, Some(0));
        assert_eq!(result.safe_model_file_name.as_deref(), Some("ready-model.gguf"));
        assert_eq!(
            result.safe_executable_file_name.as_deref(),
            Some("smoke_diagnostic_ready.exe")
        );
        let debug = format!("{result:?}");
        let json = serde_json::to_string(&result).unwrap();
        let temp_path = temp.path().to_string_lossy();
        let helper_path = helper.path().to_string_lossy();
        let model_path_text = model_path.to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert!(!debug.contains(helper_path.as_ref()));
        assert!(!json.contains(helper_path.as_ref()));
        assert!(!debug.contains(model_path_text.as_ref()));
        assert!(!json.contains(model_path_text.as_ref()));
        assert!(result
            .stdout_preview
            .contains("stdout marker"));
        assert!(!result.stdout_preview.contains(temp_path.as_ref()));
        assert!(!result.stdout_preview.contains(helper_path.as_ref()));
        assert!(!result.stdout_preview.contains(model_path_text.as_ref()));
        assert!(!result.stderr_preview.contains(temp_path.as_ref()));
        assert!(!result.stderr_preview.contains(helper_path.as_ref()));
        assert!(!result.stderr_preview.contains(model_path_text.as_ref()));
        assert_llama_runtime_smoke_diagnostic_boundary_fields(&result);
        assert!(!temp.path().join(".aegis").exists());
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 1);
    }

    #[test]
    fn local_runtime_smoke_diagnostic_preview_fails_with_redacted_and_bounded_output() {
        let temp = tempfile::tempdir().unwrap();
        let helper = tempfile::tempdir().unwrap();
        let source = r#"
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let exe_name = env::current_exe()
        .ok()
        .and_then(|path| path.file_name().and_then(|value| value.to_str()).map(|value| value.to_string()))
        .unwrap_or_default();
    println!("stdout marker");
    println!("args={}", args.join(" | "));
    println!("llama.cpp version 1.2.3");
    println!("exe_name={}", exe_name);
    println!("{}", "S".repeat(5000));
    eprintln!("stderr marker");
    eprintln!("args={}", args.join(" | "));
    eprintln!("llama.cpp version 1.2.3");
    eprintln!("exe_name={}", exe_name);
    eprintln!("{}", "E".repeat(5000));
    if args.iter().any(|arg| matches!(arg.as_str(), "-p" | "--prompt" | "-n" | "--max_output_tokens" | "-m" | "--ctx-size" | "-ngl")) {
        std::process::exit(7);
    }
}
"#;
        let executable_path =
            version_probe_helper_executable_with_source(&helper, "smoke_diagnostic_fail.exe", &source);
        let model_path = temp.path().join("ready-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        let result = assert_llama_runtime_smoke_diagnostic_deterministic_and_path_free(
            &temp,
            smoke_diagnostic_request(
                capability_request(
                    llama_runtime_validation_request(
                        Some(executable_path.to_string_lossy().as_ref()),
                        Some(model_path.to_string_lossy().as_ref()),
                        Some("llama"),
                        Some("gguf"),
                        Some(8192),
                        Some(0),
                        Some(8),
                        Some(256),
                        Some("template"),
                    ),
                    true,
                    true,
                    Some(1_500),
                ),
                true,
                Some("Say FAIL in one short sentence."),
                Some(8),
                Some(1_500),
                true,
            ),
        );
        assert_eq!(result.status, LocalRuntimeSmokeDiagnosticStatus::SmokeFailed);
        assert!(result.execution_attempted);
        assert_eq!(result.exit_code, Some(7));
        assert!(result.stdout_truncated);
        assert!(result.stderr_truncated);
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.kind == "non_zero_exit"));
        let debug = format!("{result:?}");
        let json = serde_json::to_string(&result).unwrap();
        let temp_path = temp.path().to_string_lossy();
        let helper_path = helper.path().to_string_lossy();
        let model_path_text = model_path.to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert!(!debug.contains(helper_path.as_ref()));
        assert!(!json.contains(helper_path.as_ref()));
        assert!(!debug.contains(model_path_text.as_ref()));
        assert!(!json.contains(model_path_text.as_ref()));
        assert!(!result.stdout_preview.contains(temp_path.as_ref()));
        assert!(!result.stdout_preview.contains(helper_path.as_ref()));
        assert!(!result.stdout_preview.contains(model_path_text.as_ref()));
        assert!(!result.stderr_preview.contains(temp_path.as_ref()));
        assert!(!result.stderr_preview.contains(helper_path.as_ref()));
        assert!(!result.stderr_preview.contains(model_path_text.as_ref()));
        assert_llama_runtime_smoke_diagnostic_boundary_fields(&result);
        assert!(!temp.path().join(".aegis").exists());
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 1);
    }

    #[test]
    fn local_runtime_smoke_diagnostic_preview_times_out_with_redacted_and_bounded_output() {
        let temp = tempfile::tempdir().unwrap();
        let helper = tempfile::tempdir().unwrap();
        let source = r#"
use std::{env, thread, time::Duration};

fn main() {
    let args: Vec<String> = env::args().collect();
    let exe_name = env::current_exe()
        .ok()
        .and_then(|path| path.file_name().and_then(|value| value.to_str()).map(|value| value.to_string()))
        .unwrap_or_default();
    println!("stdout marker");
    println!("args={}", args.join(" | "));
    println!("llama.cpp version 1.2.3");
    println!("exe_name={}", exe_name);
    println!("{}", "S".repeat(5000));
    eprintln!("stderr marker");
    eprintln!("args={}", args.join(" | "));
    eprintln!("llama.cpp version 1.2.3");
    eprintln!("exe_name={}", exe_name);
    eprintln!("{}", "E".repeat(5000));
    if exe_name.contains("slow") {
        thread::sleep(Duration::from_millis(750));
    }
}
"#;
        let executable_path =
            version_probe_helper_executable_with_source(&helper, "smoke_diagnostic_slow.exe", &source);
        let model_path = temp.path().join("ready-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        let result = assert_llama_runtime_smoke_diagnostic_deterministic_and_path_free(
            &temp,
            smoke_diagnostic_request(
                capability_request(
                    llama_runtime_validation_request(
                        Some(executable_path.to_string_lossy().as_ref()),
                        Some(model_path.to_string_lossy().as_ref()),
                        Some("llama"),
                        Some("gguf"),
                        Some(8192),
                        Some(0),
                        Some(8),
                        Some(256),
                        Some("template"),
                    ),
                    true,
                    true,
                    Some(1_500),
                ),
                true,
                Some("SLEEP and say READY in one short sentence."),
                Some(8),
                Some(150),
                true,
            ),
        );
        assert_eq!(result.status, LocalRuntimeSmokeDiagnosticStatus::TimedOut);
        assert!(result.execution_attempted);
        assert!(result.stdout_truncated);
        assert!(result.stderr_truncated);
        assert!(result.warnings.iter().any(|warning| warning.kind == "timed_out"));
        let debug = format!("{result:?}");
        let json = serde_json::to_string(&result).unwrap();
        let temp_path = temp.path().to_string_lossy();
        let helper_path = helper.path().to_string_lossy();
        let model_path_text = model_path.to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert!(!debug.contains(helper_path.as_ref()));
        assert!(!json.contains(helper_path.as_ref()));
        assert!(!debug.contains(model_path_text.as_ref()));
        assert!(!json.contains(model_path_text.as_ref()));
        assert!(!result.stdout_preview.contains(temp_path.as_ref()));
        assert!(!result.stdout_preview.contains(helper_path.as_ref()));
        assert!(!result.stdout_preview.contains(model_path_text.as_ref()));
        assert!(!result.stderr_preview.contains(temp_path.as_ref()));
        assert!(!result.stderr_preview.contains(helper_path.as_ref()));
        assert!(!result.stderr_preview.contains(model_path_text.as_ref()));
        assert_llama_runtime_smoke_diagnostic_boundary_fields(&result);
        assert!(!temp.path().join(".aegis").exists());
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 1);
    }

    #[test]
    fn local_runtime_probe_readiness_preview_rejects_traversal_like_paths_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["..", "../llama-server.exe", "nested/../llama-server.exe", "nested\\..\\llama-server.exe"] {
            for request in [
                probe_readiness_request(
                    llama_runtime_validation_request(
                        Some(invalid),
                        Some("model.gguf"),
                        Some("llama"),
                        Some("gguf"),
                        Some(8192),
                        Some(0),
                        Some(8),
                        Some(256),
                        Some("template"),
                    ),
                    true,
                ),
                probe_readiness_request(
                    llama_runtime_validation_request(
                        Some("llama-server.exe"),
                        Some(invalid),
                        Some("llama"),
                        Some("gguf"),
                        Some(8192),
                        Some(0),
                        Some(8),
                        Some(256),
                        Some("template"),
                    ),
                    true,
                ),
            ] {
                let result = preview_llama_runtime_probe_readiness(temp.path(), request);
                assert!(matches!(result, Err(AegisError::LocalModelRuntimeInvalidPath)));
                assert!(!temp.path().join(".aegis").exists());
            }
        }
    }

    #[test]
    fn local_runtime_probe_version_is_blocked_when_execution_is_disabled() {
        let temp = tempfile::tempdir().unwrap();
        let result = probe_local_runtime_version(
            temp.path(),
            probe_request(Some("missing-version-probe.exe"), false, Some(0)),
        )
        .unwrap();
        assert_eq!(result.status, LocalRuntimeProbeStatus::Blocked);
        assert!(!result.execution_attempted);
        assert_eq!(result.probe_argument, "--version");
        assert_eq!(result.timeout_ms, LOCAL_RUNTIME_PROBE_MIN_TIMEOUT_MS);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "execution_disabled"));
        assert!(result.warnings.iter().any(|warning| warning.message.contains("preview-only")));
        assert!(!temp.path().join(".aegis").exists());
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 0);
    }

    #[test]
    fn local_runtime_probe_version_reports_missing_executable_without_paths() {
        let temp = tempfile::tempdir().unwrap();
        let missing_executable_path = temp.path().join("missing-version-probe.exe");
        let result = probe_local_runtime_version(
            temp.path(),
            LocalRuntimeProbeRequest {
                executable_path: Some(missing_executable_path.to_string_lossy().to_string()),
                allow_execution: true,
                timeout_ms: Some(1),
            },
        )
        .unwrap();
        assert_eq!(result.status, LocalRuntimeProbeStatus::Blocked);
        assert!(!result.execution_attempted);
        assert_eq!(result.probe_argument, "--version");
        assert_eq!(result.safe_executable_file_name.as_deref(), Some("missing-version-probe.exe"));
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "executable_missing"));
        let debug = format!("{result:?}");
        let json = serde_json::to_string(&result).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn local_runtime_probe_version_rejects_traversal_like_paths_before_filesystem_access() {
        let temp = tempfile::tempdir().unwrap();
        for invalid in ["..", "../probe.exe", "nested/../probe.exe", "nested\\..\\probe.exe"] {
            let result = probe_local_runtime_version(
                temp.path(),
                probe_request(Some(invalid), true, Some(1000)),
            );
            assert!(matches!(result, Err(AegisError::LocalModelRuntimeInvalidPath)));
            assert!(!temp.path().join(".aegis").exists());
        }
    }

    #[test]
    fn local_runtime_probe_version_is_deterministic_and_path_free() {
        let temp = tempfile::tempdir().unwrap();
        let current_exe = env::current_exe().unwrap();
        let request = LocalRuntimeProbeRequest {
            executable_path: Some(current_exe.to_string_lossy().to_string()),
            allow_execution: true,
            timeout_ms: Some(10_000),
        };
        let first = probe_local_runtime_version(temp.path(), request.clone()).unwrap();
        let second = probe_local_runtime_version(temp.path(), request).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.status, LocalRuntimeProbeStatus::Completed);
        assert!(first.execution_attempted);
        assert_eq!(first.probe_argument, "--version");
        assert_eq!(first.timeout_ms, LOCAL_RUNTIME_PROBE_MAX_TIMEOUT_MS);
        assert_eq!(first.safe_executable_file_name.as_deref(), current_exe.file_name().and_then(|value| value.to_str()));
        let debug = format!("{first:?}");
        let json = serde_json::to_string(&first).unwrap();
        let temp_path = temp.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert!(first.stdout_preview.len() <= LOCAL_RUNTIME_PROBE_PREVIEW_LIMIT + 1);
        assert!(first.stderr_preview.len() <= LOCAL_RUNTIME_PROBE_PREVIEW_LIMIT + 1);
        assert!(!temp.path().join(".aegis").exists());
        assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 0);
    }

    #[test]
    fn local_runtime_probe_version_preview_truncates_long_output() {
        let preview = preview_probe_output(&"x".repeat(LOCAL_RUNTIME_PROBE_PREVIEW_LIMIT + 20), LOCAL_RUNTIME_PROBE_PREVIEW_LIMIT);
        assert!(preview.1);
        assert_eq!(preview.0.chars().count(), LOCAL_RUNTIME_PROBE_PREVIEW_LIMIT + 1);
        assert!(preview.0.ends_with('…'));
    }

    #[test]
    fn local_runtime_probe_timeout_is_clamped() {
        assert_eq!(clamp_probe_timeout_ms(None), LOCAL_RUNTIME_PROBE_DEFAULT_TIMEOUT_MS);
        assert_eq!(clamp_probe_timeout_ms(Some(0)), LOCAL_RUNTIME_PROBE_MIN_TIMEOUT_MS);
        assert_eq!(clamp_probe_timeout_ms(Some(100_000)), LOCAL_RUNTIME_PROBE_MAX_TIMEOUT_MS);
    }

    #[test]
    fn local_runtime_smoke_inference_is_blocked_when_execution_is_disabled() {
        let root = tempfile::tempdir().unwrap();
        let result = smoke_test_local_runtime_inference(
            root.path(),
            smoke_request(None, None, false, Some("Say READY in one short sentence."), Some(0), Some(0)),
        )
        .unwrap();
        assert_eq!(result.status, LocalRuntimeSmokeInferenceStatus::Blocked);
        assert_eq!(result.allow_execution, false);
        assert!(!result.execution_attempted);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "execution_disabled"));
        assert!(result.warnings.iter().any(|warning| warning.kind == "preview_only"));
        assert_smoke_boundary_fields(&result);
        assert_root_clean(&root, 0);
    }

    #[test]
    fn local_runtime_smoke_inference_marks_not_configured_runtime_as_diagnostic_only() {
        let root = tempfile::tempdir().unwrap();
        let result = smoke_test_local_runtime_inference(
            root.path(),
            smoke_request_with_runtime_kind(
                LocalModelRuntimeKind::None,
                None,
                None,
                true,
                Some("Say READY in one short sentence."),
                Some(2_500),
                Some(8),
            ),
        )
        .unwrap();
        assert_eq!(result.status, LocalRuntimeSmokeInferenceStatus::NotConfigured);
        assert!(!result.execution_attempted);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "runtime_not_configured"));
        assert_smoke_boundary_fields(&result);
        let debug = format!("{result:?}");
        let json = serde_json::to_string(&result).unwrap();
        let temp_path = root.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert_root_clean(&root, 0);
    }

    #[test]
    fn local_runtime_smoke_inference_reports_missing_model_without_paths() {
        let root = tempfile::tempdir().unwrap();
        let helper = tempfile::tempdir().unwrap();
        let executable_path = smoke_helper_executable(&helper);
        let result = smoke_test_local_runtime_inference(
            root.path(),
            smoke_request(
                Some("missing-model.gguf"),
                Some(executable_path.to_string_lossy().as_ref()),
                true,
                Some("Say READY in one short sentence."),
                Some(2_500),
                Some(8),
            ),
        )
        .unwrap();
        assert_eq!(result.status, LocalRuntimeSmokeInferenceStatus::ModelMissing);
        assert!(!result.execution_attempted);
        assert_eq!(result.safe_model_file_name.as_deref(), Some("missing-model.gguf"));
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "model_missing"));
        assert_smoke_boundary_fields(&result);
        let debug = format!("{result:?}");
        let json = serde_json::to_string(&result).unwrap();
        let temp_path = root.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert_root_clean(&root, 0);
    }

    #[test]
    fn local_runtime_smoke_inference_reports_missing_executable_without_paths() {
        let root = tempfile::tempdir().unwrap();
        let model_path = root.path().join("ready-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        let result = smoke_test_local_runtime_inference(
            root.path(),
            smoke_request(
                Some(model_path.to_string_lossy().as_ref()),
                Some("missing-smoke-helper.exe"),
                true,
                Some("Say READY in one short sentence."),
                Some(2_500),
                Some(8),
            ),
        )
        .unwrap();
        assert_eq!(result.status, LocalRuntimeSmokeInferenceStatus::ExecutableMissing);
        assert!(!result.execution_attempted);
        assert_eq!(result.safe_executable_file_name.as_deref(), Some("missing-smoke-helper.exe"));
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "executable_missing"));
        assert_smoke_boundary_fields(&result);
        let debug = format!("{result:?}");
        let json = serde_json::to_string(&result).unwrap();
        let temp_path = root.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert_root_clean(&root, 1);
    }

    #[test]
    fn local_runtime_smoke_inference_rejects_traversal_like_paths_before_filesystem_access() {
        let root = tempfile::tempdir().unwrap();
        for invalid in ["..", "../model.gguf", "nested/../model.gguf", "nested\\..\\model.gguf"] {
            let result = smoke_test_local_runtime_inference(
                root.path(),
                smoke_request(Some(invalid), Some("smoke-helper.exe"), true, Some("Say READY in one short sentence."), Some(2_500), Some(8)),
            );
            assert!(matches!(result, Err(AegisError::LocalModelRuntimeInvalidPath)));
            assert_root_clean(&root, 0);
        }
    }

    #[test]
    fn local_runtime_smoke_inference_blocks_non_gguf_models_before_execution() {
        let root = tempfile::tempdir().unwrap();
        let helper = tempfile::tempdir().unwrap();
        let executable_path = smoke_helper_executable(&helper);
        let model_path = root.path().join("smoke-model.txt");
        fs::write(&model_path, "not a gguf model").unwrap();
        let result = smoke_test_local_runtime_inference(
            root.path(),
            smoke_request(
                Some(model_path.to_string_lossy().as_ref()),
                Some(executable_path.to_string_lossy().as_ref()),
                true,
                Some("Say READY in one short sentence."),
                Some(2_500),
                Some(8),
            ),
        )
        .unwrap();
        assert_eq!(result.status, LocalRuntimeSmokeInferenceStatus::Blocked);
        assert!(!result.execution_attempted);
        assert!(result.blockers.iter().any(|blocker| blocker.kind == "model_extension_invalid"));
        assert_smoke_boundary_fields(&result);
        let debug = format!("{result:?}");
        let json = serde_json::to_string(&result).unwrap();
        let temp_path = root.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert_root_clean(&root, 1);
    }

    #[test]
    fn local_runtime_smoke_inference_is_deterministic_and_path_free() {
        let root = tempfile::tempdir().unwrap();
        let helper = tempfile::tempdir().unwrap();
        let executable_path = smoke_helper_executable(&helper);
        let model_path = root.path().join("ready-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        let request = smoke_request(
            Some(model_path.to_string_lossy().as_ref()),
            Some(executable_path.to_string_lossy().as_ref()),
            true,
            Some("Say READY in one short sentence."),
            Some(2_500),
            Some(8),
        );
        let first = smoke_test_local_runtime_inference(root.path(), request.clone()).unwrap();
        let second = smoke_test_local_runtime_inference(root.path(), request).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.status, LocalRuntimeSmokeInferenceStatus::InferenceSucceeded);
        assert!(first.execution_attempted);
        assert_eq!(first.allow_execution, true);
        assert_eq!(first.normalized_prompt, "Say READY in one short sentence.");
        assert_eq!(first.prompt_char_count, "Say READY in one short sentence.".chars().count() as u32);
        assert_eq!(first.max_output_tokens, 8);
        assert_eq!(first.timeout_ms, 2_500);
        assert_eq!(first.safe_model_file_name.as_deref(), Some("ready-model.gguf"));
        assert_eq!(first.safe_executable_file_name.as_deref(), executable_path.file_name().and_then(|value| value.to_str()));
        assert_smoke_boundary_fields(&first);
        assert!(first.stdout_preview.chars().count() <= LOCAL_RUNTIME_SMOKE_PREVIEW_LIMIT + 1);
        assert!(first.stderr_preview.chars().count() <= LOCAL_RUNTIME_SMOKE_PREVIEW_LIMIT + 1);
        assert!(first.stdout_preview.ends_with('…'));
        assert!(first.stderr_preview.ends_with('…'));
        assert!(first.blockers.is_empty());
        assert!(first.warnings.iter().any(|warning| warning.kind == "stdout_truncated"));
        assert!(first.warnings.iter().any(|warning| warning.kind == "stderr_truncated"));
        let debug = format!("{first:?}");
        let json = serde_json::to_string(&first).unwrap();
        let temp_path = root.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert_root_clean(&root, 1);
    }

    #[test]
    fn local_runtime_smoke_inference_clamps_prompt_timeout_and_output_limits() {
        let root = tempfile::tempdir().unwrap();
        let helper = tempfile::tempdir().unwrap();
        let executable_path = smoke_helper_executable(&helper);
        let model_path = root.path().join("ready-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        let long_prompt = format!(
            "SLEEP and say READY in one short sentence. {}",
            "x".repeat(LOCAL_RUNTIME_SMOKE_PROMPT_LIMIT + 64)
        );
        let result = smoke_test_local_runtime_inference(
            root.path(),
            smoke_request(
                Some(model_path.to_string_lossy().as_ref()),
                Some(executable_path.to_string_lossy().as_ref()),
                true,
                Some(&long_prompt),
                Some(0),
                Some(1000),
            ),
        )
        .unwrap();
        assert_eq!(result.status, LocalRuntimeSmokeInferenceStatus::TimedOut);
        assert!(result.execution_attempted);
        assert_eq!(result.timeout_ms, LOCAL_RUNTIME_SMOKE_MIN_TIMEOUT_MS);
        assert_eq!(result.max_output_tokens, LOCAL_RUNTIME_SMOKE_MAX_MAX_OUTPUT_TOKENS);
        assert_eq!(result.normalized_prompt.chars().count(), LOCAL_RUNTIME_SMOKE_PROMPT_LIMIT);
        assert!(result.warnings.iter().any(|warning| warning.kind == "timeout_clamped"));
        assert!(result.warnings.iter().any(|warning| warning.kind == "max_output_tokens_clamped"));
        assert!(result.warnings.iter().any(|warning| warning.kind == "prompt_truncated"));
        assert!(result.warnings.iter().any(|warning| warning.kind == "timed_out"));
        assert_smoke_boundary_fields(&result);
        assert!(result.stdout_preview.chars().count() <= LOCAL_RUNTIME_SMOKE_PREVIEW_LIMIT + 1);
        assert!(result.stderr_preview.chars().count() <= LOCAL_RUNTIME_SMOKE_PREVIEW_LIMIT + 1);
        let debug = format!("{result:?}");
        let json = serde_json::to_string(&result).unwrap();
        let temp_path = root.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert_root_clean(&root, 1);
    }

    #[test]
    fn local_runtime_smoke_inference_marks_failed_execution_as_diagnostic_only() {
        let root = tempfile::tempdir().unwrap();
        let helper = tempfile::tempdir().unwrap();
        let executable_path = smoke_helper_executable(&helper);
        let model_path = root.path().join("ready-model.gguf");
        fs::write(&model_path, "gguf placeholder").unwrap();
        let result = smoke_test_local_runtime_inference(
            root.path(),
            smoke_request(
                Some(model_path.to_string_lossy().as_ref()),
                Some(executable_path.to_string_lossy().as_ref()),
                true,
                Some("Say FAIL in one short sentence."),
                Some(2_500),
                Some(8),
            ),
        )
        .unwrap();
        assert_eq!(result.status, LocalRuntimeSmokeInferenceStatus::InferenceFailed);
        assert!(result.execution_attempted);
        assert_eq!(result.exit_code, Some(7));
        assert_smoke_boundary_fields(&result);
        let debug = format!("{result:?}");
        let json = serde_json::to_string(&result).unwrap();
        let temp_path = root.path().to_string_lossy();
        assert!(!debug.contains(temp_path.as_ref()));
        assert!(!json.contains(temp_path.as_ref()));
        assert_root_clean(&root, 1);
    }
}
