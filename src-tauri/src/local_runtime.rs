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

struct PathInspection {
    state: LocalModelRuntimePathState,
    extension_valid: bool,
    file_name: Option<String>,
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

struct LocalRuntimeProbeExecution {
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
}
