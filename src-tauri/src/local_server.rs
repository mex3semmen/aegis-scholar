use crate::errors::AegisResult;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::net::{SocketAddr, TcpListener};
use std::path::{Component, Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 48_921;
const DEFAULT_ALIAS: &str = "aegis-local-gemma";
const DEFAULT_CONTEXT_WINDOW: u32 = 4_096;
const DEFAULT_GPU_LAYERS: i32 = 0;
const HEALTH_TIMEOUT_MS: u64 = 1_500;
const HEALTH_PREVIEW_LIMIT: usize = 256;
const MONITOR_SLEEP_MS: u64 = 250;
const CHAT_DIAGNOSTIC_DEFAULT_PROMPT: &str = "Say READY in one short sentence.";
const CHAT_DIAGNOSTIC_DEFAULT_MAX_TOKENS: u32 = 16;
const CHAT_DIAGNOSTIC_DEFAULT_TEMPERATURE: f32 = 0.2;
const CHAT_DIAGNOSTIC_DEFAULT_TIMEOUT_MS: u64 = 5_000;
const CHAT_DIAGNOSTIC_PROMPT_PREVIEW_LIMIT: usize = 256;
const CHAT_DIAGNOSTIC_RESPONSE_PREVIEW_LIMIT: usize = 512;
const CHAT_DIAGNOSTIC_MESSAGE_PREVIEW_LIMIT: usize = 256;
const CHAT_DIAGNOSTIC_MAX_TOKENS_LIMIT: u32 = 64;
const CHAT_DIAGNOSTIC_MIN_TIMEOUT_MS: u64 = 250;
const CHAT_DIAGNOSTIC_MAX_TIMEOUT_MS: u64 = 15_000;
const CHAT_DIAGNOSTIC_MIN_TEMPERATURE: f32 = 0.0;
const CHAT_DIAGNOSTIC_MAX_TEMPERATURE: f32 = 2.0;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ManagedLlamaServerLaunchPlanStatus {
    Blocked,
    LaunchReadyLater,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ManagedLlamaServerLifecycleStatus {
    NotStarted,
    Starting,
    Running,
    Stopped,
    Failed,
    Blocked,
    AlreadyRunning,
    PortOccupied,
    ExternalServerDetected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ManagedLlamaServerHealthStatus {
    NotStarted,
    Loading,
    Ready,
    Unreachable,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ManagedLlamaServerPortOccupancyStatus {
    Free,
    ManagedOwned,
    ExternalServerDetected,
    PortOccupied,
    UnknownOwner,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ManagedLlamaServerChatDiagnosticStatus {
    Blocked,
    ServerNotReady,
    DiagnosticSucceeded,
    DiagnosticFailed,
    TimedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ManagedLlamaServerSmokeDiagnosticStatus {
    Blocked,
    ServerNotRunning,
    SmokeSucceeded,
    SmokeFailed,
    TimedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManagedLlamaServerNotice {
    pub kind: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ManagedLlamaServerLaunchPlanRequest {
    pub executable_path: Option<String>,
    pub model_path: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub alias: Option<String>,
    pub context_window: Option<u32>,
    pub gpu_layers: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ManagedLlamaServerStartRequest {
    pub allow_server_start: bool,
    pub launch_plan_request: ManagedLlamaServerLaunchPlanRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ManagedLlamaServerLaunchPlanPreview {
    pub status: ManagedLlamaServerLaunchPlanStatus,
    pub executable_path_present: bool,
    pub model_path_present: bool,
    pub executable_is_file: bool,
    pub model_is_file: bool,
    pub model_extension_valid: bool,
    pub safe_executable_file_name: Option<String>,
    pub safe_model_file_name: Option<String>,
    pub host: String,
    pub port: u16,
    pub alias: String,
    pub context_window: u32,
    pub gpu_layers: i32,
    pub blockers: Vec<ManagedLlamaServerNotice>,
    pub warnings: Vec<ManagedLlamaServerNotice>,
    pub next_required_actions: Vec<String>,
    pub summary: String,
    pub preview_only: bool,
    pub no_process_spawn: bool,
    pub no_model_output_used: bool,
    pub no_answer_generation: bool,
    pub no_persistence: bool,
    pub no_artifact_write: bool,
    pub no_lan_binding_by_default: bool,
    pub no_auto_start_on_launch: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ManagedLlamaServerStatusPreview {
    pub lifecycle_status: ManagedLlamaServerLifecycleStatus,
    pub health_status: ManagedLlamaServerHealthStatus,
    pub owns_active_server: bool,
    pub port_occupied: bool,
    pub port_occupied_by_unmanaged_process: bool,
    pub port_occupancy_status: ManagedLlamaServerPortOccupancyStatus,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub alias: Option<String>,
    pub process_id: Option<u32>,
    pub exit_code: Option<i32>,
    pub safe_executable_file_name: Option<String>,
    pub safe_model_file_name: Option<String>,
    pub health_url: Option<String>,
    pub response_body_preview: String,
    pub response_body_truncated: bool,
    pub blockers: Vec<ManagedLlamaServerNotice>,
    pub warnings: Vec<ManagedLlamaServerNotice>,
    pub next_required_actions: Vec<String>,
    pub summary: String,
    pub preview_only: bool,
    pub no_process_spawn: bool,
    pub no_model_output_used: bool,
    pub no_answer_generation: bool,
    pub no_persistence: bool,
    pub no_artifact_write: bool,
    pub no_lan_binding_by_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ManagedLlamaServerChatDiagnosticRequest {
    pub allow_chat_diagnostic: bool,
    pub prompt: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ManagedLlamaServerChatDiagnosticPreview {
    pub status: ManagedLlamaServerChatDiagnosticStatus,
    pub request_attempted: bool,
    pub lifecycle_status: ManagedLlamaServerLifecycleStatus,
    pub health_status: ManagedLlamaServerHealthStatus,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub alias: Option<String>,
    pub safe_model_file_name: Option<String>,
    pub prompt_char_count: usize,
    pub max_tokens: u32,
    pub temperature: f32,
    pub timeout_ms: u64,
    pub http_status: Option<u16>,
    pub response_preview: String,
    pub response_preview_truncated: bool,
    pub extracted_message_preview: Option<String>,
    pub duration_ms: u64,
    pub blockers: Vec<ManagedLlamaServerNotice>,
    pub warnings: Vec<ManagedLlamaServerNotice>,
    pub next_required_actions: Vec<String>,
    pub summary: String,
    pub diagnostic_only: bool,
    pub not_scholar_chat_answer: bool,
    pub no_final_answer_created: bool,
    pub no_grounding_applied: bool,
    pub no_artifact_write: bool,
    pub no_persistence: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ManagedLlamaServerSmokeDiagnosticRequest {
    pub allow_smoke_execution: bool,
    pub prompt: Option<String>,
    pub max_output_tokens: Option<u32>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ManagedLlamaServerSmokeDiagnosticPreview {
    pub status: ManagedLlamaServerSmokeDiagnosticStatus,
    pub execution_attempted: bool,
    pub lifecycle_status: ManagedLlamaServerLifecycleStatus,
    pub health_status: ManagedLlamaServerHealthStatus,
    pub owns_active_server: bool,
    pub port_occupied: bool,
    pub port_occupied_by_unmanaged_process: bool,
    pub port_occupancy_status: ManagedLlamaServerPortOccupancyStatus,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub alias: Option<String>,
    pub safe_model_file_name: Option<String>,
    pub prompt_char_count: usize,
    pub max_output_tokens: u32,
    pub timeout_ms: u64,
    pub http_status: Option<u16>,
    pub response_preview: String,
    pub response_preview_truncated: bool,
    pub extracted_output_preview: Option<String>,
    pub error_preview: String,
    pub error_preview_truncated: bool,
    pub duration_ms: u64,
    pub blockers: Vec<ManagedLlamaServerNotice>,
    pub warnings: Vec<ManagedLlamaServerNotice>,
    pub next_required_actions: Vec<String>,
    pub summary: String,
    pub diagnostic_only: bool,
    pub not_scholar_chat_answer: bool,
    pub no_grounding_applied: bool,
    pub no_evidence_pack_used: bool,
    pub no_artifact_write: bool,
    pub no_audit_write: bool,
    pub no_persistence: bool,
    pub no_final_answer_created: bool,
}

#[derive(Clone, Default)]
pub struct ManagedLlamaServerState {
    runtime: Arc<Mutex<ManagedLlamaServerRuntime>>,
}

struct ManagedLlamaServerRuntime {
    active: Option<ManagedLlamaServerProcess>,
    last_config: Option<ManagedLlamaServerConfig>,
    last_status: ManagedLlamaServerLifecycleStatus,
    last_health_status: ManagedLlamaServerHealthStatus,
    last_health_preview: String,
    last_health_truncated: bool,
    last_exit_code: Option<i32>,
}

struct ManagedLlamaServerProcess {
    child: Child,
    pid: u32,
    config: ManagedLlamaServerConfig,
}

#[derive(Clone)]
struct ManagedLlamaServerConfig {
    executable_path: PathBuf,
    model_path: PathBuf,
    host: String,
    port: u16,
    alias: String,
    context_window: u32,
    gpu_layers: i32,
}

#[derive(Clone)]
struct LaunchPlan {
    config: ManagedLlamaServerConfig,
    safe_executable_file_name: Option<String>,
    safe_model_file_name: Option<String>,
    executable_path_present: bool,
    model_path_present: bool,
    executable_is_file: bool,
    model_is_file: bool,
    model_extension_valid: bool,
    blockers: Vec<ManagedLlamaServerNotice>,
    warnings: Vec<ManagedLlamaServerNotice>,
    next_required_actions: Vec<String>,
    summary: String,
    status: ManagedLlamaServerLaunchPlanStatus,
}

impl Default for ManagedLlamaServerRuntime {
    fn default() -> Self {
        Self {
            active: None,
            last_config: None,
            last_status: ManagedLlamaServerLifecycleStatus::NotStarted,
            last_health_status: ManagedLlamaServerHealthStatus::NotStarted,
            last_health_preview: String::new(),
            last_health_truncated: false,
            last_exit_code: None,
        }
    }
}

impl ManagedLlamaServerState {
    fn lock_runtime(&self) -> std::sync::MutexGuard<'_, ManagedLlamaServerRuntime> {
        self.runtime.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Drop for ManagedLlamaServerState {
    fn drop(&mut self) {
        if let Ok(mut runtime) = self.runtime.lock() {
            if let Some(mut active) = runtime.active.take() {
                let _ = active.child.kill();
                runtime.last_exit_code = active.child.wait().ok().and_then(|status| status.code());
                runtime.last_status = ManagedLlamaServerLifecycleStatus::Stopped;
                runtime.last_health_status = ManagedLlamaServerHealthStatus::NotStarted;
                runtime.last_health_preview.clear();
                runtime.last_health_truncated = false;
            }
        }
    }
}

pub fn preview_managed_llama_server_launch_plan(
    root: impl Into<PathBuf>,
    request: ManagedLlamaServerLaunchPlanRequest,
) -> AegisResult<ManagedLlamaServerLaunchPlanPreview> {
    Ok(build_launch_plan(root.into().as_path(), request).preview())
}

pub fn start_managed_llama_server(
    root: impl Into<PathBuf>,
    state: &ManagedLlamaServerState,
    request: ManagedLlamaServerStartRequest,
) -> AegisResult<ManagedLlamaServerStatusPreview> {
    let plan = build_launch_plan(root.into().as_path(), request.launch_plan_request);

    if plan.status == ManagedLlamaServerLaunchPlanStatus::Blocked {
        return Ok(status_preview_from_plan(
            ManagedLlamaServerLifecycleStatus::Blocked,
            ManagedLlamaServerHealthStatus::NotStarted,
            &plan,
            None,
            None,
            None,
            String::new(),
            false,
            plan.summary.clone(),
            plan.blockers.clone(),
            plan.warnings.clone(),
            plan.next_required_actions.clone(),
        ));
    }

    if !request.allow_server_start {
        return Ok(status_preview_from_plan(
            ManagedLlamaServerLifecycleStatus::Blocked,
            ManagedLlamaServerHealthStatus::NotStarted,
            &plan,
            None,
            None,
            None,
            String::new(),
            false,
            "Starting the managed llama-server requires explicit consent.".to_string(),
            vec![notice(
                "consent_missing",
                "Start is blocked until explicit server-start consent is provided.",
            )],
            vec![notice(
                "preview_only",
                "This action is preview-only until the server-start consent checkbox is selected.",
            )],
            vec![
                "Confirm consent before starting the managed server.".to_string(),
                "Re-run the start action after reviewing the launch plan.".to_string(),
            ],
        ));
    }

    {
        let mut runtime = state.lock_runtime();
        sweep_runtime(&mut runtime);
        if runtime.active.is_some() || runtime.last_status == ManagedLlamaServerLifecycleStatus::Starting {
            return Ok(apply_port_occupancy(
                status_preview_from_plan(
                ManagedLlamaServerLifecycleStatus::AlreadyRunning,
                ManagedLlamaServerHealthStatus::NotStarted,
                &plan,
                runtime.active.as_ref().map(|active| active.pid),
                runtime.last_exit_code,
                Some(format!("http://{}:{}/health", plan.config.host, plan.config.port)),
                String::new(),
                false,
                "Only one AEGIS-managed llama-server process can be active at a time.".to_string(),
                vec![notice(
                    "already_running",
                    "Only one AEGIS-managed llama-server process can be active at a time.",
                )],
                vec![],
                vec!["Stop the active managed server before starting another one.".to_string()],
                ),
                ManagedLlamaServerPortOccupancyStatus::ManagedOwned,
                true,
                false,
                runtime.active.is_some(),
            ));
        }
        runtime.last_config = Some(plan.config.clone());
    }

    let port_probe = probe_managed_llama_server_port(&plan.config);
    if !matches!(port_probe.occupancy_status, ManagedLlamaServerPortOccupancyStatus::Free) {
        let lifecycle_status = match port_probe.occupancy_status {
            ManagedLlamaServerPortOccupancyStatus::ExternalServerDetected => {
                ManagedLlamaServerLifecycleStatus::ExternalServerDetected
            }
            ManagedLlamaServerPortOccupancyStatus::PortOccupied
            | ManagedLlamaServerPortOccupancyStatus::UnknownOwner => ManagedLlamaServerLifecycleStatus::PortOccupied,
            ManagedLlamaServerPortOccupancyStatus::ManagedOwned => ManagedLlamaServerLifecycleStatus::AlreadyRunning,
            ManagedLlamaServerPortOccupancyStatus::Free => ManagedLlamaServerLifecycleStatus::Blocked,
        };
        {
            let mut runtime = state.lock_runtime();
            runtime.active = None;
            runtime.last_status = lifecycle_status.clone();
            runtime.last_health_status = port_probe.health_status.clone();
            runtime.last_health_preview = port_probe.response_body_preview.clone();
            runtime.last_health_truncated = port_probe.response_body_truncated;
            runtime.last_exit_code = None;
        }
        let preview = status_preview_from_plan(
            lifecycle_status,
            port_probe.health_status.clone(),
            &plan,
            None,
            None,
            Some(format!("http://{}:{}/health", plan.config.host, plan.config.port)),
            port_probe.response_body_preview.clone(),
            port_probe.response_body_truncated,
            format!("Managed server start blocked: {}", port_probe.summary),
            port_probe.blockers.clone(),
            port_probe.warnings.clone(),
            port_probe.next_required_actions.clone(),
        );
        return Ok(apply_port_probe_to_status_preview(preview, &port_probe, false));
    }

    {
        let mut runtime = state.lock_runtime();
        runtime.last_status = ManagedLlamaServerLifecycleStatus::Starting;
        runtime.last_health_status = ManagedLlamaServerHealthStatus::NotStarted;
        runtime.last_health_preview.clear();
        runtime.last_health_truncated = false;
        runtime.last_exit_code = None;
    }

    let mut command = Command::new(&plan.config.executable_path);
    command
        .arg("-m")
        .arg(&plan.config.model_path)
        .arg("-c")
        .arg(plan.config.context_window.to_string())
        .arg("-ngl")
        .arg(plan.config.gpu_layers.to_string())
        .arg("--host")
        .arg(&plan.config.host)
        .arg("--port")
        .arg(plan.config.port.to_string())
        .arg("--alias")
        .arg(&plan.config.alias)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            let mut runtime = state.lock_runtime();
            runtime.active = None;
            runtime.last_status = ManagedLlamaServerLifecycleStatus::Failed;
            runtime.last_health_status = ManagedLlamaServerHealthStatus::Failed;
            runtime.last_exit_code = None;
            return Ok(status_preview_from_plan(
                ManagedLlamaServerLifecycleStatus::Failed,
                ManagedLlamaServerHealthStatus::Failed,
                &plan,
                None,
                None,
                None,
                String::new(),
                false,
                format!("Failed to start the managed llama-server: {error}"),
                vec![notice(
                    "spawn_failed",
                    "The managed llama-server process could not be started.",
                )],
                vec![],
                vec!["Review the executable path, model path, and local permissions before retrying.".to_string()],
            ));
        }
    };

    let pid = child.id();
    {
        let mut runtime = state.lock_runtime();
        runtime.active = Some(ManagedLlamaServerProcess {
            child,
            pid,
            config: plan.config.clone(),
        });
        runtime.last_status = ManagedLlamaServerLifecycleStatus::Starting;
        runtime.last_health_status = ManagedLlamaServerHealthStatus::NotStarted;
        runtime.last_health_preview.clear();
        runtime.last_health_truncated = false;
        runtime.last_exit_code = None;
        runtime.last_config = Some(plan.config.clone());
    }
    spawn_monitor_thread(state.runtime.clone());

    Ok(apply_port_occupancy(
        status_preview_from_plan(
        ManagedLlamaServerLifecycleStatus::Starting,
        ManagedLlamaServerHealthStatus::NotStarted,
        &plan,
        Some(pid),
        None,
        Some(format!("http://{}:{}/health", plan.config.host, plan.config.port)),
        String::new(),
        false,
        "The managed llama-server process has been started on localhost.".to_string(),
        Vec::new(),
        vec![notice(
            "health_check_pending",
            "Check health after the server has had time to bind its localhost port.",
        )],
        vec![
            "Run the health check after the process has started listening.".to_string(),
            "Stop only the AEGIS-managed server if you need to shut it down.".to_string(),
        ],
        ),
        ManagedLlamaServerPortOccupancyStatus::ManagedOwned,
        true,
        false,
        true,
    ))
}

pub fn check_managed_llama_server_health(state: &ManagedLlamaServerState) -> AegisResult<ManagedLlamaServerStatusPreview> {
    let (plan, pid) = {
        let mut runtime = state.lock_runtime();
        sweep_runtime(&mut runtime);
        let Some(active) = runtime.active.as_ref() else {
            return Ok(status_preview_from_empty(
                ManagedLlamaServerLifecycleStatus::NotStarted,
                ManagedLlamaServerHealthStatus::NotStarted,
                "No AEGIS-managed llama-server is running.".to_string(),
            )
            .with_warnings(vec![notice(
                "not_started",
                "Start the managed server before checking localhost health.",
            )])
            .with_actions(vec!["Start the managed server with explicit consent first.".to_string()]));
        };
        (plan_from_config(&active.config), active.pid)
    };

    let health_url = format!("http://{}:{}/health", plan.config.host, plan.config.port);
    let client = match Client::builder().timeout(Duration::from_millis(HEALTH_TIMEOUT_MS)).build() {
        Ok(client) => client,
        Err(error) => {
            return Ok(status_preview_from_plan(
                ManagedLlamaServerLifecycleStatus::Failed,
                ManagedLlamaServerHealthStatus::Failed,
                &plan,
                Some(pid),
                None,
                Some(health_url),
                String::new(),
                false,
                format!("The managed llama-server health check could not be created: {error}"),
                vec![notice(
                    "health_client_failed",
                    "The localhost health check could not be created.",
                )],
                Vec::new(),
                Vec::new(),
            ));
        }
    };

    let response = client.get(&health_url).send();
    let (
        lifecycle_status,
        health_status,
        response_body_preview,
        response_body_truncated,
        blockers,
        warnings,
        next_required_actions,
        summary,
    ) = match response {
        Ok(response) => {
            let status_code = response.status().as_u16();
            let body = response.text().unwrap_or_default();
            let (preview, truncated) = preview_text(&body, HEALTH_PREVIEW_LIMIT);
            let normalized = body.trim().to_ascii_lowercase();
            if status_code == 200 && normalized == "ok" {
                (
                    ManagedLlamaServerLifecycleStatus::Running,
                    ManagedLlamaServerHealthStatus::Ready,
                    preview,
                    truncated,
                    Vec::new(),
                    vec![notice(
                        "health_ready",
                        "The managed llama-server health endpoint returned 200/ok.",
                    )],
                    vec!["Stop the AEGIS-managed server when you no longer need it.".to_string()],
                    "The managed llama-server is ready on localhost.".to_string(),
                )
            } else if status_code == 503 || normalized.contains("loading") {
                (
                    ManagedLlamaServerLifecycleStatus::Starting,
                    ManagedLlamaServerHealthStatus::Loading,
                    preview,
                    truncated,
                    Vec::new(),
                    vec![notice(
                        "health_loading",
                        "The managed llama-server is still loading or warming up.",
                    )],
                    vec!["Check health again after the server finishes loading.".to_string()],
                    "The managed llama-server is still loading on localhost.".to_string(),
                )
            } else {
                (
                    ManagedLlamaServerLifecycleStatus::Failed,
                    ManagedLlamaServerHealthStatus::Failed,
                    preview,
                    truncated,
                    vec![notice(
                        "unexpected_health_response",
                        "The managed llama-server returned an unexpected health response.",
                    )],
                    vec![notice(
                        "health_unexpected",
                        "Inspect the server logs or retry the health check.",
                    )],
                    vec!["Review the managed server launch configuration.".to_string()],
                    format!("The managed llama-server health endpoint returned HTTP {status_code}."),
                )
            }
        }
        Err(error) if error.is_connect() || error.is_timeout() => (
            ManagedLlamaServerLifecycleStatus::Starting,
            ManagedLlamaServerHealthStatus::Unreachable,
            String::new(),
            false,
            vec![notice(
                "health_unreachable",
                "The managed llama-server health endpoint is not reachable yet.",
            )],
            vec![notice(
                "connection_refused_or_timeout",
                "The localhost health check could not connect within the bounded timeout.",
            )],
            vec!["Try the health check again after the server has had time to bind.".to_string()],
            format!("The managed llama-server health endpoint is unreachable: {error}"),
        ),
        Err(error) => (
            ManagedLlamaServerLifecycleStatus::Failed,
            ManagedLlamaServerHealthStatus::Failed,
            String::new(),
            false,
            vec![notice(
                "health_request_failed",
                "The managed llama-server health check failed unexpectedly.",
            )],
            vec![notice(
                "health_check_failed",
                "Review the local server configuration and retry the health check.",
            )],
            vec!["Inspect the launch plan and local permissions before retrying.".to_string()],
            format!("The managed llama-server health check failed: {error}"),
        ),
    };

    {
        let mut runtime = state.lock_runtime();
        runtime.last_status = lifecycle_status.clone();
        runtime.last_health_status = health_status.clone();
        runtime.last_health_preview = response_body_preview.clone();
        runtime.last_health_truncated = response_body_truncated;
    }

    Ok(apply_port_occupancy(
        status_preview_from_plan(
        lifecycle_status,
        health_status,
        &plan,
        Some(pid),
        None,
        Some(health_url),
        response_body_preview,
        response_body_truncated,
        summary,
        blockers,
        warnings,
        next_required_actions,
        ),
        ManagedLlamaServerPortOccupancyStatus::ManagedOwned,
        true,
        false,
        true,
    ))
}

pub fn stop_managed_llama_server(state: &ManagedLlamaServerState) -> AegisResult<ManagedLlamaServerStatusPreview> {
    let maybe_process = {
        let mut runtime = state.lock_runtime();
        sweep_runtime(&mut runtime);
        runtime.active.take()
    };

    let Some(mut process) = maybe_process else {
        return Ok(status_preview_from_empty(
            ManagedLlamaServerLifecycleStatus::NotStarted,
            ManagedLlamaServerHealthStatus::NotStarted,
            "No AEGIS-managed llama-server is running.".to_string(),
        )
        .with_warnings(vec![notice(
            "not_running",
            "There is no AEGIS-managed llama-server to stop.",
        )])
        .with_actions(vec!["Start the managed server first if you need it again.".to_string()]));
    };

    let exit_code = match process.child.kill() {
        Ok(_) => process.child.wait().ok().and_then(|status| status.code()),
        Err(_) => process.child.wait().ok().and_then(|status| status.code()),
    };

    {
        let mut runtime = state.lock_runtime();
        runtime.last_status = ManagedLlamaServerLifecycleStatus::Stopped;
        runtime.last_health_status = ManagedLlamaServerHealthStatus::NotStarted;
        runtime.last_health_preview.clear();
        runtime.last_health_truncated = false;
        runtime.last_exit_code = exit_code;
        runtime.last_config = Some(process.config.clone());
    }

    let plan = plan_from_config(&process.config);
    let port_probe = probe_managed_llama_server_port(&process.config);
    if matches!(port_probe.occupancy_status, ManagedLlamaServerPortOccupancyStatus::Free) {
        Ok(apply_port_occupancy(
            status_preview_from_plan(
                ManagedLlamaServerLifecycleStatus::Stopped,
                ManagedLlamaServerHealthStatus::NotStarted,
                &plan,
                None,
                exit_code,
                None,
                String::new(),
                false,
                "The AEGIS-managed llama-server process was stopped.".to_string(),
                Vec::new(),
                vec![notice(
                    "stopped",
                    "AEGIS only stops the managed server process it started.",
                )],
                vec!["Restart the managed server with explicit consent if you need it again.".to_string()],
            ),
            ManagedLlamaServerPortOccupancyStatus::Free,
            false,
            false,
            false,
        ))
    } else {
        let lifecycle_status = match port_probe.occupancy_status {
            ManagedLlamaServerPortOccupancyStatus::ExternalServerDetected => {
                ManagedLlamaServerLifecycleStatus::ExternalServerDetected
            }
            ManagedLlamaServerPortOccupancyStatus::PortOccupied
            | ManagedLlamaServerPortOccupancyStatus::UnknownOwner => ManagedLlamaServerLifecycleStatus::PortOccupied,
            ManagedLlamaServerPortOccupancyStatus::ManagedOwned => ManagedLlamaServerLifecycleStatus::Stopped,
            ManagedLlamaServerPortOccupancyStatus::Free => ManagedLlamaServerLifecycleStatus::Stopped,
        };
        Ok(apply_port_probe_to_status_preview(
            status_preview_from_plan(
                lifecycle_status,
                port_probe.health_status.clone(),
                &plan,
                None,
                exit_code,
                Some(format!("http://{}:{}/health", plan.config.host, plan.config.port)),
                String::new(),
                false,
                "The AEGIS-managed llama-server process was stopped, but the configured port still appears occupied.".to_string(),
                Vec::new(),
                port_probe.warnings.clone(),
                port_probe.next_required_actions.clone(),
            ),
            &port_probe,
            false,
        ))
    }
}

pub fn inspect_managed_llama_server_status(state: &ManagedLlamaServerState) -> AegisResult<ManagedLlamaServerStatusPreview> {
    let runtime = state.lock_runtime();
    if let Some(active) = runtime.active.as_ref() {
        let plan = plan_from_config(&active.config);
        Ok(apply_port_occupancy(
            status_preview_from_plan(
            runtime.last_status.clone(),
            runtime.last_health_status.clone(),
            &plan,
            Some(active.pid),
            runtime.last_exit_code,
            Some(format!("http://{}:{}/health", active.config.host, active.config.port)),
            runtime.last_health_preview.clone(),
            runtime.last_health_truncated,
            "The managed llama-server state is available.".to_string(),
            Vec::new(),
            vec![notice(
                "managed_server_active",
                "The AEGIS-managed llama-server process is tracked in backend state.",
            )],
            vec!["Check health if you need the latest ready/loading status.".to_string()],
            ),
            ManagedLlamaServerPortOccupancyStatus::ManagedOwned,
            true,
            false,
            true,
        ))
    } else if let Some(config) = runtime.last_config.as_ref() {
        let plan = plan_from_config(config);
        let port_probe = probe_managed_llama_server_port(config);
        if matches!(port_probe.occupancy_status, ManagedLlamaServerPortOccupancyStatus::Free) {
            let lifecycle_status = match runtime.last_status {
                ManagedLlamaServerLifecycleStatus::PortOccupied
                | ManagedLlamaServerLifecycleStatus::ExternalServerDetected => {
                    ManagedLlamaServerLifecycleStatus::NotStarted
                }
                _ => runtime.last_status.clone(),
            };
            let health_status = if matches!(lifecycle_status, ManagedLlamaServerLifecycleStatus::NotStarted) {
                ManagedLlamaServerHealthStatus::NotStarted
            } else {
                runtime.last_health_status.clone()
            };
            let response_body_preview = if matches!(lifecycle_status, ManagedLlamaServerLifecycleStatus::NotStarted) {
                String::new()
            } else {
                runtime.last_health_preview.clone()
            };
            let response_body_truncated = if matches!(lifecycle_status, ManagedLlamaServerLifecycleStatus::NotStarted) {
                false
            } else {
                runtime.last_health_truncated
            };
            Ok(apply_port_occupancy(
                status_preview_from_plan(
                    lifecycle_status,
                    health_status,
                    &plan,
                    None,
                    runtime.last_exit_code,
                    None,
                    response_body_preview,
                    response_body_truncated,
                    "No AEGIS-managed llama-server is currently running.".to_string(),
                    Vec::new(),
                    vec![notice("not_running", "No managed server process is active.")],
                    vec!["Start the managed server with explicit consent if you need it again.".to_string()],
                ),
                ManagedLlamaServerPortOccupancyStatus::Free,
                false,
                false,
                false,
            ))
        } else {
            let lifecycle_status = match port_probe.occupancy_status {
                ManagedLlamaServerPortOccupancyStatus::ExternalServerDetected => {
                    ManagedLlamaServerLifecycleStatus::ExternalServerDetected
                }
                ManagedLlamaServerPortOccupancyStatus::PortOccupied
                | ManagedLlamaServerPortOccupancyStatus::UnknownOwner => ManagedLlamaServerLifecycleStatus::PortOccupied,
                ManagedLlamaServerPortOccupancyStatus::ManagedOwned => ManagedLlamaServerLifecycleStatus::AlreadyRunning,
                ManagedLlamaServerPortOccupancyStatus::Free => ManagedLlamaServerLifecycleStatus::Stopped,
            };
            Ok(apply_port_probe_to_status_preview(
                status_preview_from_plan(
                    lifecycle_status,
                    port_probe.health_status.clone(),
                    &plan,
                    None,
                    runtime.last_exit_code,
                    Some(format!("http://{}:{}/health", plan.config.host, plan.config.port)),
                    port_probe.response_body_preview.clone(),
                    port_probe.response_body_truncated,
                    "The configured managed server port is occupied by a server that AEGIS does not own.".to_string(),
                    port_probe.blockers.clone(),
                    port_probe.warnings.clone(),
                    port_probe.next_required_actions.clone(),
                ),
                &port_probe,
                false,
            ))
        }
    } else {
        Ok(status_preview_from_empty(
            runtime.last_status.clone(),
            runtime.last_health_status.clone(),
            "No AEGIS-managed llama-server is currently running.".to_string(),
        )
        .with_warnings(vec![notice("not_running", "No managed server process is active.")])
        .with_actions(vec!["Start the managed server with explicit consent if you need it again.".to_string()]))
    }
}

pub fn run_managed_llama_server_chat_diagnostic(
    state: &ManagedLlamaServerState,
    request: ManagedLlamaServerChatDiagnosticRequest,
) -> AegisResult<ManagedLlamaServerChatDiagnosticPreview> {
    let status = inspect_managed_llama_server_status(state)?;
    Ok(run_managed_llama_server_chat_diagnostic_from_status(&status, request))
}

fn run_managed_llama_server_chat_diagnostic_from_status(
    status: &ManagedLlamaServerStatusPreview,
    request: ManagedLlamaServerChatDiagnosticRequest,
) -> ManagedLlamaServerChatDiagnosticPreview {
    let normalized = normalize_chat_diagnostic_request(request);
    let mut blockers = status.blockers.clone();
    let mut warnings = status.warnings.clone();
    warnings.push(notice(
        "diagnostic_only",
        "This is a diagnostic-only local request; it does not create a Scholar Chat answer.",
    ));
    if normalized.prompt_truncated {
        warnings.push(notice(
            "prompt_truncated",
            "The diagnostic prompt was truncated to keep the request bounded.",
        ));
    }
    if normalized.max_tokens_was_clamped {
        warnings.push(notice(
            "max_tokens_clamped",
            "The diagnostic max_tokens value was clamped to the bounded request limit.",
        ));
    }
    if normalized.temperature_was_clamped {
        warnings.push(notice(
            "temperature_clamped",
            "The diagnostic temperature was clamped to the supported local range.",
        ));
    }
    if normalized.timeout_ms_was_clamped {
        warnings.push(notice(
            "timeout_clamped",
            "The diagnostic timeout was clamped to the bounded request range.",
        ));
    }

    if !normalized.allow_chat_diagnostic {
        blockers.push(notice(
            "chat_diagnostic_consent_missing",
            "Chat diagnostic consent is required before a local request is sent.",
        ));
        return chat_diagnostic_preview(
            ManagedLlamaServerChatDiagnosticStatus::Blocked,
            false,
            &status,
            normalized,
            None,
            String::new(),
            false,
            None,
            0,
            blockers,
            warnings,
            vec!["Enable chat diagnostic consent, then run the diagnostic again.".to_string()],
            "The managed chat diagnostic is blocked until explicit consent is granted.".to_string(),
        );
    }

    if !matches!(status.lifecycle_status, ManagedLlamaServerLifecycleStatus::Running)
        || !matches!(status.health_status, ManagedLlamaServerHealthStatus::Ready)
    {
        blockers.push(notice(
            "server_not_ready",
            "The managed llama-server must be running and health-ready before the chat diagnostic can run.",
        ));
        return chat_diagnostic_preview(
            ManagedLlamaServerChatDiagnosticStatus::ServerNotReady,
            false,
            &status,
            normalized,
            None,
            String::new(),
            false,
            None,
            0,
            blockers,
            warnings,
            vec![
                "Start the managed server with explicit consent.".to_string(),
                "Wait until health reports ready, then run the chat diagnostic again.".to_string(),
            ],
            "The managed llama-server is not ready for a diagnostic chat request yet.".to_string(),
        );
    }

    let Some(host) = status.host.clone() else {
        blockers.push(notice(
            "host_missing",
            "The managed llama-server host is missing from the tracked status.",
        ));
        return chat_diagnostic_preview(
            ManagedLlamaServerChatDiagnosticStatus::ServerNotReady,
            false,
            &status,
            normalized,
            None,
            String::new(),
            false,
            None,
            0,
            blockers,
            warnings,
            vec!["Review the managed launch state and retry once the host is available.".to_string()],
            "The managed llama-server host is not available for the chat diagnostic.".to_string(),
        );
    };
    if !is_local_host(&host) {
        blockers.push(notice(
            "host_not_local",
            "The managed chat diagnostic only targets localhost or 127.0.0.1.",
        ));
        return chat_diagnostic_preview(
            ManagedLlamaServerChatDiagnosticStatus::Blocked,
            false,
            &status,
            normalized,
            None,
            String::new(),
            false,
            None,
            0,
            blockers,
            warnings,
            vec!["Restore the managed server to localhost and retry the diagnostic.".to_string()],
            "The managed chat diagnostic is blocked because the managed server is not localhost-bound.".to_string(),
        );
    }
    let Some(port) = status.port else {
        blockers.push(notice(
            "port_missing",
            "The managed llama-server port is missing from the tracked status.",
        ));
        return chat_diagnostic_preview(
            ManagedLlamaServerChatDiagnosticStatus::ServerNotReady,
            false,
            &status,
            normalized,
            None,
            String::new(),
            false,
            None,
            0,
            blockers,
            warnings,
            vec!["Review the managed launch state and retry once the port is available.".to_string()],
            "The managed llama-server port is not available for the chat diagnostic.".to_string(),
        );
    };

    let timeout_ms = normalized.timeout_ms;
    let client = match Client::builder()
        .timeout(Duration::from_millis(timeout_ms))
        .build()
    {
        Ok(client) => client,
        Err(error) => {
            blockers.push(notice(
                "client_build_failed",
                "The managed chat diagnostic client could not be created.",
            ));
            return chat_diagnostic_preview(
                ManagedLlamaServerChatDiagnosticStatus::DiagnosticFailed,
                false,
                &status,
                normalized,
                None,
                String::new(),
                false,
                None,
                0,
                blockers,
                warnings,
                vec!["Review the local runtime environment and retry the diagnostic.".to_string()],
                format!("The managed chat diagnostic client could not be created: {error}"),
            );
        }
    };

    let url = format!("http://{host}:{port}/v1/chat/completions");
    let model = status
        .alias
        .clone()
        .or_else(|| status.safe_model_file_name.clone())
        .unwrap_or_else(|| DEFAULT_ALIAS.to_string());
    let payload = json!({
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": normalized.prompt
            }
        ],
        "max_tokens": normalized.max_tokens,
        "temperature": normalized.temperature,
        "stream": false,
    });

    let start = Instant::now();
    let response = client.post(&url).json(&payload).send();
    let duration_ms = start
        .elapsed()
        .as_millis()
        .min(u128::from(u64::MAX)) as u64;

    match response {
        Ok(response) => {
            let http_status = Some(response.status().as_u16());
            let body = response.text().unwrap_or_default();
            let (response_preview, response_preview_truncated) =
                preview_text(&body, CHAT_DIAGNOSTIC_RESPONSE_PREVIEW_LIMIT);
            let parsed: Result<Value, _> = serde_json::from_str(&body);
            match parsed {
                Ok(parsed_json) => {
                    if let Some(extracted_message) = extract_completion_text(&parsed_json) {
                        let (extracted_message_preview, _) =
                            compact_text_preview(&extracted_message, CHAT_DIAGNOSTIC_MESSAGE_PREVIEW_LIMIT);
                        let mut success_warnings = warnings;
                        success_warnings.push(notice(
                            "diagnostic_success",
                            "The managed chat diagnostic request succeeded and remained diagnostic-only.",
                        ));
                        success_warnings.push(notice(
                            "not_scholar_chat_answer",
                            "The diagnostic message preview is not a Scholar Chat answer.",
                        ));
                        return chat_diagnostic_preview(
                            ManagedLlamaServerChatDiagnosticStatus::DiagnosticSucceeded,
                            true,
                            &status,
                            normalized,
                            http_status,
                            response_preview,
                            response_preview_truncated,
                            Some(extracted_message_preview),
                            duration_ms,
                            blockers,
                            success_warnings,
                            vec!["Stop the managed server when you no longer need it.".to_string()],
                            "The managed chat diagnostic succeeded and remains diagnostic-only.".to_string(),
                        );
                    }

                    blockers.push(notice(
                        "message_missing",
                        "The managed chat diagnostic response did not include an assistant message content field.",
                    ));
                    warnings.push(notice(
                        "response_parse_failed",
                        "The response parsed as JSON, but no assistant message content could be extracted.",
                    ));
                    return chat_diagnostic_preview(
                        ManagedLlamaServerChatDiagnosticStatus::DiagnosticFailed,
                        true,
                        &status,
                        normalized,
                        http_status,
                        response_preview,
                        response_preview_truncated,
                        None,
                        duration_ms,
                        blockers,
                        warnings,
                        vec!["Inspect the raw response preview and retry the diagnostic.".to_string()],
                        "The managed chat diagnostic response could not be parsed into an assistant message.".to_string(),
                    );
                }
                Err(error) => {
                    blockers.push(notice(
                        "response_parse_failed",
                        "The managed chat diagnostic response body was not valid JSON.",
                    ));
                    warnings.push(notice(
                        "response_not_parseable",
                        "Inspect the bounded raw response preview and retry the diagnostic.",
                    ));
                    return chat_diagnostic_preview(
                        ManagedLlamaServerChatDiagnosticStatus::DiagnosticFailed,
                        true,
                        &status,
                        normalized,
                        http_status,
                        response_preview,
                        response_preview_truncated,
                        None,
                        duration_ms,
                        blockers,
                        warnings,
                        vec!["Inspect the raw response preview and retry the diagnostic.".to_string()],
                        format!("The managed chat diagnostic response was not valid JSON: {error}"),
                    );
                }
            }
        }
        Err(error) if error.is_timeout() => {
            blockers.push(notice(
                "request_timeout",
                "The managed chat diagnostic request timed out.",
            ));
            warnings.push(notice(
                "timeout",
                "The bounded chat diagnostic timeout elapsed before a response was received.",
            ));
            return chat_diagnostic_preview(
                ManagedLlamaServerChatDiagnosticStatus::TimedOut,
                true,
                &status,
                normalized,
                None,
                String::new(),
                false,
                None,
                duration_ms,
                blockers,
                warnings,
                vec!["Retry the diagnostic or increase the bounded timeout.".to_string()],
                format!("The managed chat diagnostic timed out after {duration_ms} ms."),
            );
        }
        Err(error) => {
            blockers.push(notice(
                "request_failed",
                "The managed chat diagnostic request failed before a usable response arrived.",
            ));
            warnings.push(notice(
                "request_error",
                "Inspect the bounded request preview, raw response preview, and local server status.",
            ));
            return chat_diagnostic_preview(
                ManagedLlamaServerChatDiagnosticStatus::DiagnosticFailed,
                true,
                &status,
                normalized,
                None,
                String::new(),
                false,
                None,
                duration_ms,
                blockers,
                warnings,
                vec!["Review the managed server status and retry the diagnostic.".to_string()],
                format!("The managed chat diagnostic request failed: {error}"),
            );
        }
    }
}

pub fn run_managed_llama_server_smoke_diagnostic(
    state: &ManagedLlamaServerState,
    request: ManagedLlamaServerSmokeDiagnosticRequest,
) -> AegisResult<ManagedLlamaServerSmokeDiagnosticPreview> {
    let status = inspect_managed_llama_server_status(state)?;
    Ok(run_managed_llama_server_smoke_diagnostic_from_status(&status, request))
}

fn run_managed_llama_server_smoke_diagnostic_from_status(
    status: &ManagedLlamaServerStatusPreview,
    request: ManagedLlamaServerSmokeDiagnosticRequest,
) -> ManagedLlamaServerSmokeDiagnosticPreview {
    let normalized = normalize_smoke_diagnostic_request(request);
    let mut blockers = status.blockers.clone();
    let mut warnings = status.warnings.clone();
    warnings.push(notice(
        "diagnostic_only",
        "This is a diagnostic-only local model smoke test; it does not create a Scholar Chat answer.",
    ));
    if normalized.prompt_truncated {
        warnings.push(notice(
            "prompt_truncated",
            "The smoke diagnostic prompt was truncated to keep the request bounded.",
        ));
    }
    if normalized.max_output_tokens_was_clamped {
        warnings.push(notice(
            "max_output_tokens_clamped",
            "The smoke diagnostic max_output_tokens value was clamped to the bounded request limit.",
        ));
    }
    if normalized.timeout_ms_was_clamped {
        warnings.push(notice(
            "timeout_clamped",
            "The smoke diagnostic timeout was clamped to the bounded request range.",
        ));
    }

    if !normalized.allow_smoke_execution {
        blockers.push(notice(
            "smoke_execution_consent_missing",
            "Smoke diagnostic consent is required before a local request is sent.",
        ));
        return smoke_diagnostic_preview(
            ManagedLlamaServerSmokeDiagnosticStatus::Blocked,
            false,
            status,
            normalized,
            None,
            String::new(),
            false,
            None,
            String::new(),
            false,
            0,
            blockers,
            warnings,
            vec!["Enable smoke diagnostic consent, then run the smoke test again.".to_string()],
            "The managed smoke diagnostic is blocked until explicit consent is granted.".to_string(),
        );
    }

    if !status.owns_active_server {
        if status.port_occupied_by_unmanaged_process {
            blockers.push(notice(
                "external_server_detected",
                "The managed llama-server port is already occupied by an unmanaged process.",
            ));
            return smoke_diagnostic_preview(
                ManagedLlamaServerSmokeDiagnosticStatus::Blocked,
                false,
                status,
                normalized,
                None,
                String::new(),
                false,
                None,
                String::new(),
                false,
                0,
                blockers,
                warnings,
                vec!["Stop the external process or choose another port before running the smoke test.".to_string()],
                "The managed smoke diagnostic is blocked because the occupied port is not AEGIS-owned.".to_string(),
            );
        }
        blockers.push(notice(
            "server_not_running",
            "The managed llama-server is not running yet.",
        ));
        return smoke_diagnostic_preview(
            ManagedLlamaServerSmokeDiagnosticStatus::ServerNotRunning,
            false,
            status,
            normalized,
            None,
            String::new(),
            false,
            None,
            String::new(),
            false,
            0,
            blockers,
            warnings,
            vec!["Start the managed server with explicit consent before running the smoke test.".to_string()],
            "The managed smoke diagnostic cannot run until an AEGIS-owned server is running.".to_string(),
        );
    }

    if !matches!(status.lifecycle_status, ManagedLlamaServerLifecycleStatus::Running)
        || !matches!(status.health_status, ManagedLlamaServerHealthStatus::Ready)
    {
        blockers.push(notice(
            "server_not_ready",
            "The managed llama-server must be running and health-ready before the smoke diagnostic can run.",
        ));
        return smoke_diagnostic_preview(
            ManagedLlamaServerSmokeDiagnosticStatus::ServerNotRunning,
            false,
            status,
            normalized,
            None,
            String::new(),
            false,
            None,
            String::new(),
            false,
            0,
            blockers,
            warnings,
            vec![
                "Start the managed server with explicit consent.".to_string(),
                "Wait until health reports ready, then run the smoke test again.".to_string(),
            ],
            "The managed smoke diagnostic is not ready yet because the server is not health-ready.".to_string(),
        );
    }

    let Some(host) = status.host.clone() else {
        blockers.push(notice(
            "host_missing",
            "The managed llama-server host is missing from the tracked status.",
        ));
        return smoke_diagnostic_preview(
            ManagedLlamaServerSmokeDiagnosticStatus::ServerNotRunning,
            false,
            status,
            normalized,
            None,
            String::new(),
            false,
            None,
            String::new(),
            false,
            0,
            blockers,
            warnings,
            vec!["Review the managed launch state and retry once the host is available.".to_string()],
            "The managed llama-server host is not available for the smoke diagnostic.".to_string(),
        );
    };
    if !is_local_host(&host) {
        blockers.push(notice(
            "host_not_local",
            "The managed smoke diagnostic only targets localhost or 127.0.0.1.",
        ));
        return smoke_diagnostic_preview(
            ManagedLlamaServerSmokeDiagnosticStatus::Blocked,
            false,
            status,
            normalized,
            None,
            String::new(),
            false,
            None,
            String::new(),
            false,
            0,
            blockers,
            warnings,
            vec!["Restore the managed server to localhost and retry the smoke diagnostic.".to_string()],
            "The managed smoke diagnostic is blocked because the managed server is not localhost-bound.".to_string(),
        );
    }
    let Some(port) = status.port else {
        blockers.push(notice(
            "port_missing",
            "The managed llama-server port is missing from the tracked status.",
        ));
        return smoke_diagnostic_preview(
            ManagedLlamaServerSmokeDiagnosticStatus::ServerNotRunning,
            false,
            status,
            normalized,
            None,
            String::new(),
            false,
            None,
            String::new(),
            false,
            0,
            blockers,
            warnings,
            vec!["Review the managed launch state and retry once the port is available.".to_string()],
            "The managed llama-server port is not available for the smoke diagnostic.".to_string(),
        );
    };

    let timeout_ms = normalized.timeout_ms;
    let client = match Client::builder()
        .timeout(Duration::from_millis(timeout_ms))
        .build()
    {
        Ok(client) => client,
        Err(error) => {
            blockers.push(notice(
                "client_build_failed",
                "The managed smoke diagnostic client could not be created.",
            ));
            let error_text = format!("The managed smoke diagnostic client could not be created: {error}");
            let (error_preview, error_preview_truncated) =
                compact_text_preview(&error_text, CHAT_DIAGNOSTIC_RESPONSE_PREVIEW_LIMIT);
            return smoke_diagnostic_preview(
                ManagedLlamaServerSmokeDiagnosticStatus::SmokeFailed,
                false,
                status,
                normalized,
                None,
                String::new(),
                false,
                None,
                error_preview,
                error_preview_truncated,
                0,
                blockers,
                warnings,
                vec!["Review the local runtime environment and retry the smoke test.".to_string()],
                error_text,
            );
        }
    };

    let url = format!("http://{host}:{port}/v1/completions");
    let model = status
        .alias
        .clone()
        .or_else(|| status.safe_model_file_name.clone())
        .unwrap_or_else(|| DEFAULT_ALIAS.to_string());
    let payload = json!({
        "model": model,
        "prompt": normalized.prompt,
        "max_tokens": normalized.max_output_tokens,
        "temperature": 0.2_f32,
        "stream": false,
    });

    let start = Instant::now();
    let response = client.post(&url).json(&payload).send();
    let duration_ms = start
        .elapsed()
        .as_millis()
        .min(u128::from(u64::MAX)) as u64;

    match response {
        Ok(response) => {
            let http_status_code = response.status();
            let http_status = Some(http_status_code.as_u16());
            let success_status = http_status_code.is_success();
            let body = response.text().unwrap_or_default();
            let (response_preview, response_preview_truncated) =
                preview_text(&body, CHAT_DIAGNOSTIC_RESPONSE_PREVIEW_LIMIT);
            let parsed: Result<Value, _> = serde_json::from_str(&body);
            match parsed {
                Ok(parsed_json) => {
                    if success_status {
                        if let Some(extracted_output) = extract_completion_text(&parsed_json) {
                            let (extracted_output_preview, _) =
                                compact_text_preview(&extracted_output, CHAT_DIAGNOSTIC_MESSAGE_PREVIEW_LIMIT);
                            let mut success_warnings = warnings;
                            success_warnings.push(notice(
                                "diagnostic_success",
                                "The managed smoke diagnostic request succeeded and remained diagnostic-only.",
                            ));
                            success_warnings.push(notice(
                                "not_scholar_chat_answer",
                                "The smoke output preview is not a Scholar Chat answer.",
                            ));
                            return smoke_diagnostic_preview(
                                ManagedLlamaServerSmokeDiagnosticStatus::SmokeSucceeded,
                                true,
                                status,
                                normalized,
                                http_status,
                                response_preview,
                                response_preview_truncated,
                                Some(extracted_output_preview),
                                String::new(),
                                false,
                                duration_ms,
                                blockers,
                                success_warnings,
                                vec!["Stop the managed server when you no longer need it.".to_string()],
                                "The managed smoke diagnostic succeeded and remains diagnostic-only.".to_string(),
                            );
                        }

                        blockers.push(notice(
                            "output_missing",
                            "The managed smoke diagnostic response did not include completion text.",
                        ));
                        warnings.push(notice(
                            "response_parse_failed",
                            "The response parsed as JSON, but no completion text could be extracted.",
                        ));
                        let error_text = "The managed smoke diagnostic response could not be parsed into completion text.".to_string();
                        let (error_preview, error_preview_truncated) =
                            compact_text_preview(&error_text, CHAT_DIAGNOSTIC_RESPONSE_PREVIEW_LIMIT);
                        return smoke_diagnostic_preview(
                            ManagedLlamaServerSmokeDiagnosticStatus::SmokeFailed,
                            true,
                            status,
                            normalized,
                            http_status,
                            response_preview,
                            response_preview_truncated,
                            None,
                            error_preview,
                            error_preview_truncated,
                            duration_ms,
                            blockers,
                            warnings,
                            vec!["Inspect the raw response preview and retry the smoke test.".to_string()],
                            error_text,
                        );
                    }

                    blockers.push(notice(
                        "unexpected_http_status",
                        "The managed smoke diagnostic response returned a non-success HTTP status.",
                    ));
                    warnings.push(notice(
                        "response_not_success",
                        "Inspect the bounded raw response preview and retry the smoke test.",
                    ));
                    let error_text = format!(
                        "The managed smoke diagnostic response returned HTTP {}.",
                        http_status_code.as_u16()
                    );
                    let (error_preview, error_preview_truncated) =
                        compact_text_preview(&error_text, CHAT_DIAGNOSTIC_RESPONSE_PREVIEW_LIMIT);
                    return smoke_diagnostic_preview(
                        ManagedLlamaServerSmokeDiagnosticStatus::SmokeFailed,
                        true,
                        status,
                        normalized,
                        http_status,
                        response_preview,
                        response_preview_truncated,
                        None,
                        error_preview,
                        error_preview_truncated,
                        duration_ms,
                        blockers,
                        warnings,
                        vec!["Inspect the raw response preview and retry the smoke test.".to_string()],
                        error_text,
                    );
                }
                Err(error) => {
                    blockers.push(notice(
                        "response_parse_failed",
                        "The managed smoke diagnostic response body was not valid JSON.",
                    ));
                    warnings.push(notice(
                        "response_not_parseable",
                        "Inspect the bounded raw response preview and retry the smoke test.",
                    ));
                    let error_text = format!("The managed smoke diagnostic response was not valid JSON: {error}");
                    let (error_preview, error_preview_truncated) =
                        compact_text_preview(&error_text, CHAT_DIAGNOSTIC_RESPONSE_PREVIEW_LIMIT);
                    return smoke_diagnostic_preview(
                        ManagedLlamaServerSmokeDiagnosticStatus::SmokeFailed,
                        true,
                        status,
                        normalized,
                        http_status,
                        response_preview,
                        response_preview_truncated,
                        None,
                        error_preview,
                        error_preview_truncated,
                        duration_ms,
                        blockers,
                        warnings,
                        vec!["Inspect the raw response preview and retry the smoke test.".to_string()],
                        error_text,
                    );
                }
            }
        }
        Err(error) if error.is_timeout() => {
            blockers.push(notice(
                "request_timeout",
                "The managed smoke diagnostic request timed out.",
            ));
            warnings.push(notice(
                "timeout",
                "The bounded smoke diagnostic timeout elapsed before a response was received.",
            ));
            let error_text = format!("The managed smoke diagnostic timed out after {duration_ms} ms.");
            let (error_preview, error_preview_truncated) =
                compact_text_preview(&error_text, CHAT_DIAGNOSTIC_RESPONSE_PREVIEW_LIMIT);
            return smoke_diagnostic_preview(
                ManagedLlamaServerSmokeDiagnosticStatus::TimedOut,
                true,
                status,
                normalized,
                None,
                String::new(),
                false,
                None,
                error_preview,
                error_preview_truncated,
                duration_ms,
                blockers,
                warnings,
                vec!["Retry the smoke test or increase the bounded timeout.".to_string()],
                error_text,
            );
        }
        Err(error) => {
            blockers.push(notice(
                "request_failed",
                "The managed smoke diagnostic request failed before a usable response arrived.",
            ));
            warnings.push(notice(
                "request_error",
                "Inspect the bounded response preview and local server status.",
            ));
            let error_text = format!("The managed smoke diagnostic request failed: {error}");
            let (error_preview, error_preview_truncated) =
                compact_text_preview(&error_text, CHAT_DIAGNOSTIC_RESPONSE_PREVIEW_LIMIT);
            return smoke_diagnostic_preview(
                ManagedLlamaServerSmokeDiagnosticStatus::SmokeFailed,
                true,
                status,
                normalized,
                None,
                String::new(),
                false,
                None,
                error_preview,
                error_preview_truncated,
                duration_ms,
                blockers,
                warnings,
                vec!["Review the managed server status and retry the smoke test.".to_string()],
                error_text,
            );
        }
    }
}

fn build_launch_plan(root: &Path, request: ManagedLlamaServerLaunchPlanRequest) -> LaunchPlan {
    let executable_path = normalize_optional_text(request.executable_path);
    let model_path = normalize_optional_text(request.model_path);
    let host = request.host.as_deref().map(normalize_host).unwrap_or_else(|| DEFAULT_HOST.to_string());
    let port = request.port.filter(|value| *value != 0).unwrap_or(DEFAULT_PORT);
    let alias = normalize_optional_text(request.alias).unwrap_or_else(|| DEFAULT_ALIAS.to_string());
    let context_window = request.context_window.filter(|value| *value > 0).unwrap_or(DEFAULT_CONTEXT_WINDOW);
    let gpu_layers = request.gpu_layers.filter(|value| *value >= 0).unwrap_or(DEFAULT_GPU_LAYERS);

    let executable_path_present = executable_path.is_some();
    let model_path_present = model_path.is_some();
    let executable_path_buf = executable_path.as_deref().and_then(|value| {
        if has_parent_dir_component(value) {
            None
        } else {
            Some(resolve_runtime_path(root, value))
        }
    });
    let model_path_buf = model_path.as_deref().and_then(|value| {
        if has_parent_dir_component(value) {
            None
        } else {
            Some(resolve_runtime_path(root, value))
        }
    });
    let executable_is_file = executable_path_buf
        .as_ref()
        .map(|path| matches!(fs::metadata(path), Ok(metadata) if metadata.is_file()))
        .unwrap_or(false);
    let model_is_file = model_path_buf
        .as_ref()
        .map(|path| matches!(fs::metadata(path), Ok(metadata) if metadata.is_file()))
        .unwrap_or(false);
    let model_extension_valid = model_path_buf
        .as_ref()
        .map(|path| has_gguf_extension(path.as_path()))
        .unwrap_or(false);

    let mut blockers = Vec::new();
    if !is_local_host(&host) {
        blockers.push(notice(
            "host_not_local",
            "The managed llama-server must bind to 127.0.0.1 or localhost in this phase.",
        ));
    }
    if port == 0 {
        blockers.push(notice("port_invalid", "The managed llama-server port must be greater than zero."));
    }
    if !executable_path_present {
        blockers.push(notice("executable_missing", "The configured llama-server executable must be provided."));
    } else if executable_path_buf.is_none() {
        blockers.push(notice("executable_invalid_path", "The configured llama-server executable path is invalid."));
    } else if !executable_is_file {
        blockers.push(notice(
            "executable_missing",
            "The configured llama-server executable must point to an existing file.",
        ));
    }
    if !model_path_present {
        blockers.push(notice("model_missing", "The configured model path must be provided."));
    } else if model_path_buf.is_none() {
        blockers.push(notice("model_invalid_path", "The configured model path is invalid."));
    } else if !model_is_file {
        blockers.push(notice(
            "model_missing",
            "The configured model path must point to an existing .gguf file.",
        ));
    } else if !model_extension_valid {
        blockers.push(notice("model_extension_invalid", "The managed llama-server requires a .gguf model file."));
    }

    let warnings = vec![
        notice("preview_only", "This launch plan is preview only; no process is started."),
        notice("no_persistence", "Managed server configuration is not persisted."),
        notice("no_answer_generation", "This phase does not route llama-server output into Scholar Chat answers yet."),
        notice("localhost_only", "The managed server stays bound to localhost in this phase."),
        notice("managed_stop_only", "AEGIS only stops the managed server process it started."),
        notice("consent_required", "Starting the managed server later still requires explicit consent."),
    ];

    let next_required_actions = if blockers.is_empty() {
        vec![
            "Review the launch plan, then start the managed server with explicit consent.".to_string(),
            "Run the health check after the server begins listening on localhost.".to_string(),
            "Stop only the AEGIS-managed process if you need to shut it down.".to_string(),
        ]
    } else {
        vec!["Fix the blocked inputs before trying to start the managed server.".to_string()]
    };

    let status = if blockers.is_empty() {
        ManagedLlamaServerLaunchPlanStatus::LaunchReadyLater
    } else {
        ManagedLlamaServerLaunchPlanStatus::Blocked
    };

    let summary = format!(
        "The managed llama-server launch plan is {} Safe executable: {}. Safe model: {}. Host: {}. Port: {}. Alias: {}.",
        match status {
            ManagedLlamaServerLaunchPlanStatus::Blocked => "blocked until the required inputs are fixed.",
            ManagedLlamaServerLaunchPlanStatus::LaunchReadyLater => "ready later for explicit local start.",
        },
        safe_file_name(executable_path.as_deref()).unwrap_or_else(|| "missing".to_string()),
        safe_file_name(model_path.as_deref()).unwrap_or_else(|| "missing".to_string()),
        host,
        port,
        alias
    );

    LaunchPlan {
        config: ManagedLlamaServerConfig {
            executable_path: executable_path_buf.unwrap_or_default(),
            model_path: model_path_buf.unwrap_or_default(),
            host,
            port,
            alias,
            context_window,
            gpu_layers,
        },
        safe_executable_file_name: safe_file_name(executable_path.as_deref()),
        safe_model_file_name: safe_file_name(model_path.as_deref()),
        executable_path_present,
        model_path_present,
        executable_is_file,
        model_is_file,
        model_extension_valid,
        blockers,
        warnings,
        next_required_actions,
        summary,
        status,
    }
}

impl LaunchPlan {
    fn preview(self) -> ManagedLlamaServerLaunchPlanPreview {
        ManagedLlamaServerLaunchPlanPreview {
            status: self.status,
            executable_path_present: self.executable_path_present,
            model_path_present: self.model_path_present,
            executable_is_file: self.executable_is_file,
            model_is_file: self.model_is_file,
            model_extension_valid: self.model_extension_valid,
            safe_executable_file_name: self.safe_executable_file_name,
            safe_model_file_name: self.safe_model_file_name,
            host: self.config.host,
            port: self.config.port,
            alias: self.config.alias,
            context_window: self.config.context_window,
            gpu_layers: self.config.gpu_layers,
            blockers: self.blockers,
            warnings: self.warnings,
            next_required_actions: self.next_required_actions,
            summary: self.summary,
            preview_only: true,
            no_process_spawn: true,
            no_model_output_used: true,
            no_answer_generation: true,
            no_persistence: true,
            no_artifact_write: true,
            no_lan_binding_by_default: true,
            no_auto_start_on_launch: true,
        }
    }
}

impl ManagedLlamaServerStatusPreview {
    fn with_warnings(mut self, warnings: Vec<ManagedLlamaServerNotice>) -> Self {
        self.warnings = warnings;
        self
    }

    fn with_actions(mut self, next_required_actions: Vec<String>) -> Self {
        self.next_required_actions = next_required_actions;
        self
    }
}

struct NormalizedChatDiagnosticRequest {
    prompt: String,
    prompt_char_count: usize,
    prompt_truncated: bool,
    allow_chat_diagnostic: bool,
    max_tokens: u32,
    max_tokens_was_clamped: bool,
    temperature: f32,
    temperature_was_clamped: bool,
    timeout_ms: u64,
    timeout_ms_was_clamped: bool,
}

fn normalize_chat_diagnostic_request(
    request: ManagedLlamaServerChatDiagnosticRequest,
) -> NormalizedChatDiagnosticRequest {
    let prompt = normalize_optional_text(request.prompt).unwrap_or_else(|| CHAT_DIAGNOSTIC_DEFAULT_PROMPT.to_string());
    let mut prompt_chars = prompt.chars();
    let prompt = prompt_chars
        .by_ref()
        .take(CHAT_DIAGNOSTIC_PROMPT_PREVIEW_LIMIT)
        .collect::<String>();
    let prompt_truncated = prompt_chars.next().is_some();

    let raw_max_tokens = request.max_tokens.filter(|value| *value > 0).unwrap_or(CHAT_DIAGNOSTIC_DEFAULT_MAX_TOKENS);
    let max_tokens = raw_max_tokens.min(CHAT_DIAGNOSTIC_MAX_TOKENS_LIMIT);
    let max_tokens_was_clamped = max_tokens != raw_max_tokens;

    let raw_temperature = request
        .temperature
        .filter(|value| value.is_finite())
        .unwrap_or(CHAT_DIAGNOSTIC_DEFAULT_TEMPERATURE);
    let temperature = raw_temperature.clamp(CHAT_DIAGNOSTIC_MIN_TEMPERATURE, CHAT_DIAGNOSTIC_MAX_TEMPERATURE);
    let temperature_was_clamped = (temperature - raw_temperature).abs() > f32::EPSILON;

    let raw_timeout_ms = request
        .timeout_ms
        .filter(|value| *value > 0)
        .unwrap_or(CHAT_DIAGNOSTIC_DEFAULT_TIMEOUT_MS);
    let timeout_ms = raw_timeout_ms.clamp(CHAT_DIAGNOSTIC_MIN_TIMEOUT_MS, CHAT_DIAGNOSTIC_MAX_TIMEOUT_MS);
    let timeout_ms_was_clamped = timeout_ms != raw_timeout_ms;

    NormalizedChatDiagnosticRequest {
        prompt_char_count: prompt.chars().count(),
        prompt,
        prompt_truncated,
        allow_chat_diagnostic: request.allow_chat_diagnostic,
        max_tokens,
        max_tokens_was_clamped,
        temperature,
        temperature_was_clamped,
        timeout_ms,
        timeout_ms_was_clamped,
    }
}

fn extract_completion_text(parsed: &Value) -> Option<String> {
    let choices = parsed.get("choices")?.as_array()?;
    let first_choice = choices.first()?;
    if let Some(text) = first_choice.get("text").and_then(|value| value.as_str()) {
        let content = text.trim().to_string();
        if !content.is_empty() {
            return Some(content);
        }
    }
    let message = first_choice.get("message")?;
    let content = message.get("content")?.as_str()?.trim().to_string();
    if content.is_empty() {
        None
    } else {
        Some(content)
    }
}

fn compact_text_preview(value: &str, limit: usize) -> (String, bool) {
    let compacted = value.split_whitespace().collect::<Vec<_>>().join(" ");
    preview_text(&compacted, limit)
}

struct NormalizedSmokeDiagnosticRequest {
    prompt: String,
    prompt_char_count: usize,
    prompt_truncated: bool,
    allow_smoke_execution: bool,
    max_output_tokens: u32,
    max_output_tokens_was_clamped: bool,
    timeout_ms: u64,
    timeout_ms_was_clamped: bool,
}

fn normalize_smoke_diagnostic_request(
    request: ManagedLlamaServerSmokeDiagnosticRequest,
) -> NormalizedSmokeDiagnosticRequest {
    let prompt = normalize_optional_text(request.prompt)
        .unwrap_or_else(|| "Say READY in one short sentence.".to_string());
    let mut prompt_chars = prompt.chars();
    let prompt = prompt_chars
        .by_ref()
        .take(CHAT_DIAGNOSTIC_PROMPT_PREVIEW_LIMIT)
        .collect::<String>();
    let prompt_truncated = prompt_chars.next().is_some();

    let raw_max_output_tokens = request
        .max_output_tokens
        .filter(|value| *value > 0)
        .unwrap_or(16);
    let max_output_tokens = raw_max_output_tokens.min(32);
    let max_output_tokens_was_clamped = max_output_tokens != raw_max_output_tokens;

    let raw_timeout_ms = request
        .timeout_ms
        .filter(|value| *value > 0)
        .unwrap_or(30_000);
    let timeout_ms = raw_timeout_ms.clamp(500, 30_000);
    let timeout_ms_was_clamped = timeout_ms != raw_timeout_ms;

    NormalizedSmokeDiagnosticRequest {
        prompt_char_count: prompt.chars().count(),
        prompt,
        prompt_truncated,
        allow_smoke_execution: request.allow_smoke_execution,
        max_output_tokens,
        max_output_tokens_was_clamped,
        timeout_ms,
        timeout_ms_was_clamped,
    }
}

fn smoke_diagnostic_preview(
    status: ManagedLlamaServerSmokeDiagnosticStatus,
    execution_attempted: bool,
    managed_status: &ManagedLlamaServerStatusPreview,
    normalized: NormalizedSmokeDiagnosticRequest,
    http_status: Option<u16>,
    response_preview: String,
    response_preview_truncated: bool,
    extracted_output_preview: Option<String>,
    error_preview: String,
    error_preview_truncated: bool,
    duration_ms: u64,
    blockers: Vec<ManagedLlamaServerNotice>,
    warnings: Vec<ManagedLlamaServerNotice>,
    next_required_actions: Vec<String>,
    summary: String,
) -> ManagedLlamaServerSmokeDiagnosticPreview {
    ManagedLlamaServerSmokeDiagnosticPreview {
        status,
        execution_attempted,
        lifecycle_status: managed_status.lifecycle_status.clone(),
        health_status: managed_status.health_status.clone(),
        owns_active_server: managed_status.owns_active_server,
        port_occupied: managed_status.port_occupied,
        port_occupied_by_unmanaged_process: managed_status.port_occupied_by_unmanaged_process,
        port_occupancy_status: managed_status.port_occupancy_status.clone(),
        host: managed_status.host.clone(),
        port: managed_status.port,
        alias: managed_status.alias.clone(),
        safe_model_file_name: managed_status.safe_model_file_name.clone(),
        prompt_char_count: normalized.prompt_char_count,
        max_output_tokens: normalized.max_output_tokens,
        timeout_ms: normalized.timeout_ms,
        http_status,
        response_preview,
        response_preview_truncated,
        extracted_output_preview,
        error_preview,
        error_preview_truncated,
        duration_ms,
        blockers,
        warnings,
        next_required_actions,
        summary,
        diagnostic_only: true,
        not_scholar_chat_answer: true,
        no_grounding_applied: true,
        no_evidence_pack_used: true,
        no_artifact_write: true,
        no_audit_write: true,
        no_persistence: true,
        no_final_answer_created: true,
    }
}

fn chat_diagnostic_preview(
    status: ManagedLlamaServerChatDiagnosticStatus,
    request_attempted: bool,
    managed_status: &ManagedLlamaServerStatusPreview,
    normalized: NormalizedChatDiagnosticRequest,
    http_status: Option<u16>,
    response_preview: String,
    response_preview_truncated: bool,
    extracted_message_preview: Option<String>,
    duration_ms: u64,
    blockers: Vec<ManagedLlamaServerNotice>,
    warnings: Vec<ManagedLlamaServerNotice>,
    next_required_actions: Vec<String>,
    summary: String,
) -> ManagedLlamaServerChatDiagnosticPreview {
    ManagedLlamaServerChatDiagnosticPreview {
        status,
        request_attempted,
        lifecycle_status: managed_status.lifecycle_status.clone(),
        health_status: managed_status.health_status.clone(),
        host: managed_status.host.clone(),
        port: managed_status.port,
        alias: managed_status.alias.clone(),
        safe_model_file_name: managed_status.safe_model_file_name.clone(),
        prompt_char_count: normalized.prompt_char_count,
        max_tokens: normalized.max_tokens,
        temperature: normalized.temperature,
        timeout_ms: normalized.timeout_ms,
        http_status,
        response_preview,
        response_preview_truncated,
        extracted_message_preview,
        duration_ms,
        blockers,
        warnings,
        next_required_actions,
        summary,
        diagnostic_only: true,
        not_scholar_chat_answer: true,
        no_final_answer_created: true,
        no_grounding_applied: true,
        no_artifact_write: true,
        no_persistence: true,
    }
}

fn plan_from_config(config: &ManagedLlamaServerConfig) -> LaunchPlan {
    let executable_path = config.executable_path.to_string_lossy().to_string();
    let model_path = config.model_path.to_string_lossy().to_string();

    LaunchPlan {
        config: config.clone(),
        safe_executable_file_name: safe_file_name(Some(executable_path.as_str())),
        safe_model_file_name: safe_file_name(Some(model_path.as_str())),
        executable_path_present: !executable_path.trim().is_empty(),
        model_path_present: !model_path.trim().is_empty(),
        executable_is_file: matches!(fs::metadata(&config.executable_path), Ok(metadata) if metadata.is_file()),
        model_is_file: matches!(fs::metadata(&config.model_path), Ok(metadata) if metadata.is_file()),
        model_extension_valid: has_gguf_extension(&config.model_path),
        blockers: Vec::new(),
        warnings: Vec::new(),
        next_required_actions: Vec::new(),
        summary: "The managed llama-server state is available.".to_string(),
        status: ManagedLlamaServerLaunchPlanStatus::LaunchReadyLater,
    }
}

fn status_preview_from_plan(
    lifecycle_status: ManagedLlamaServerLifecycleStatus,
    health_status: ManagedLlamaServerHealthStatus,
    plan: &LaunchPlan,
    process_id: Option<u32>,
    exit_code: Option<i32>,
    health_url: Option<String>,
    response_body_preview: String,
    response_body_truncated: bool,
    summary: String,
    blockers: Vec<ManagedLlamaServerNotice>,
    warnings: Vec<ManagedLlamaServerNotice>,
    next_required_actions: Vec<String>,
) -> ManagedLlamaServerStatusPreview {
    ManagedLlamaServerStatusPreview {
        lifecycle_status,
        health_status,
        owns_active_server: false,
        port_occupied: false,
        port_occupied_by_unmanaged_process: false,
        port_occupancy_status: ManagedLlamaServerPortOccupancyStatus::Free,
        host: Some(plan.config.host.clone()),
        port: Some(plan.config.port),
        alias: Some(plan.config.alias.clone()),
        process_id,
        exit_code,
        safe_executable_file_name: plan.safe_executable_file_name.clone(),
        safe_model_file_name: plan.safe_model_file_name.clone(),
        health_url,
        response_body_preview,
        response_body_truncated,
        blockers,
        warnings,
        next_required_actions,
        summary,
        preview_only: true,
        no_process_spawn: true,
        no_model_output_used: true,
        no_answer_generation: true,
        no_persistence: true,
        no_artifact_write: true,
        no_lan_binding_by_default: true,
    }
}

fn status_preview_from_empty(
    lifecycle_status: ManagedLlamaServerLifecycleStatus,
    health_status: ManagedLlamaServerHealthStatus,
    summary: String,
) -> ManagedLlamaServerStatusPreview {
    ManagedLlamaServerStatusPreview {
        lifecycle_status,
        health_status,
        owns_active_server: false,
        port_occupied: false,
        port_occupied_by_unmanaged_process: false,
        port_occupancy_status: ManagedLlamaServerPortOccupancyStatus::Free,
        host: None,
        port: None,
        alias: None,
        process_id: None,
        exit_code: None,
        safe_executable_file_name: None,
        safe_model_file_name: None,
        health_url: None,
        response_body_preview: String::new(),
        response_body_truncated: false,
        blockers: Vec::new(),
        warnings: Vec::new(),
        next_required_actions: Vec::new(),
        summary,
        preview_only: true,
        no_process_spawn: true,
        no_model_output_used: true,
        no_answer_generation: true,
        no_persistence: true,
        no_artifact_write: true,
        no_lan_binding_by_default: true,
    }
}

fn apply_port_occupancy(
    mut preview: ManagedLlamaServerStatusPreview,
    status: ManagedLlamaServerPortOccupancyStatus,
    port_occupied: bool,
    port_occupied_by_unmanaged_process: bool,
    owns_active_server: bool,
) -> ManagedLlamaServerStatusPreview {
    preview.port_occupancy_status = status;
    preview.port_occupied = port_occupied;
    preview.port_occupied_by_unmanaged_process = port_occupied_by_unmanaged_process;
    preview.owns_active_server = owns_active_server;
    preview
}

fn apply_port_probe_to_status_preview(
    mut preview: ManagedLlamaServerStatusPreview,
    probe: &ManagedLlamaServerPortProbe,
    owns_active_server: bool,
) -> ManagedLlamaServerStatusPreview {
    preview.health_status = probe.health_status.clone();
    preview.response_body_preview = probe.response_body_preview.clone();
    preview.response_body_truncated = probe.response_body_truncated;
    preview.blockers = probe.blockers.clone();
    preview.warnings = probe.warnings.clone();
    preview.next_required_actions = probe.next_required_actions.clone();
    preview.summary = probe.summary.clone();
    apply_port_occupancy(
        preview,
        probe.occupancy_status.clone(),
        !matches!(probe.occupancy_status, ManagedLlamaServerPortOccupancyStatus::Free),
        matches!(
            probe.occupancy_status,
            ManagedLlamaServerPortOccupancyStatus::ExternalServerDetected
                | ManagedLlamaServerPortOccupancyStatus::PortOccupied
                | ManagedLlamaServerPortOccupancyStatus::UnknownOwner
        ),
        owns_active_server,
    )
}

struct ManagedLlamaServerPortProbe {
    occupancy_status: ManagedLlamaServerPortOccupancyStatus,
    health_status: ManagedLlamaServerHealthStatus,
    response_body_preview: String,
    response_body_truncated: bool,
    summary: String,
    blockers: Vec<ManagedLlamaServerNotice>,
    warnings: Vec<ManagedLlamaServerNotice>,
    next_required_actions: Vec<String>,
}

fn probe_managed_llama_server_port(config: &ManagedLlamaServerConfig) -> ManagedLlamaServerPortProbe {
    let Some(socket_addr) = localhost_socket_addr(&config.host, config.port) else {
        return ManagedLlamaServerPortProbe {
            occupancy_status: ManagedLlamaServerPortOccupancyStatus::UnknownOwner,
            health_status: ManagedLlamaServerHealthStatus::Failed,
            response_body_preview: String::new(),
            response_body_truncated: false,
            summary: "The configured localhost address could not be parsed for port occupancy inspection.".to_string(),
            blockers: vec![notice(
                "port_probe_invalid_host",
                "The configured managed server host could not be resolved for local occupancy inspection.",
            )],
            warnings: vec![notice(
                "port_probe_unknown_owner",
                "The managed server port could not be inspected, so ownership remains unknown.",
            )],
            next_required_actions: vec!["Choose a valid localhost host name or IP address and retry.".to_string()],
        };
    };

    match TcpListener::bind(socket_addr) {
        Ok(listener) => {
            drop(listener);
            ManagedLlamaServerPortProbe {
                occupancy_status: ManagedLlamaServerPortOccupancyStatus::Free,
                health_status: ManagedLlamaServerHealthStatus::NotStarted,
                response_body_preview: String::new(),
                response_body_truncated: false,
                summary: format!("localhost:{} is available for an AEGIS-managed llama-server start.", config.port),
                blockers: Vec::new(),
                warnings: Vec::new(),
                next_required_actions: vec!["Start the managed server with explicit consent when ready.".to_string()],
            }
        }
        Err(_) => {
            let health_url = format!("http://{}:{}/health", config.host, config.port);
            let client = match Client::builder().timeout(Duration::from_millis(500)).build() {
                Ok(client) => client,
                Err(error) => {
                    return ManagedLlamaServerPortProbe {
                        occupancy_status: ManagedLlamaServerPortOccupancyStatus::UnknownOwner,
                        health_status: ManagedLlamaServerHealthStatus::Failed,
                        response_body_preview: String::new(),
                        response_body_truncated: false,
                        summary: format!("localhost:{} is occupied, but the health client could not be created: {error}", config.port),
                        blockers: vec![notice(
                            "port_occupied",
                            "The configured localhost port is already in use.",
                        )],
                        warnings: vec![notice(
                            "unknown_owner",
                            "The occupied port could not be identified as managed or external.",
                        )],
                        next_required_actions: vec!["Stop the external process or choose another port.".to_string()],
                    };
                }
            };

            match client.get(&health_url).send() {
                Ok(response) => {
                    let status_code = response.status().as_u16();
                    let body = response.text().unwrap_or_default();
                    let (preview, truncated) = preview_text(&body, HEALTH_PREVIEW_LIMIT);
                    let normalized = body.trim().to_ascii_lowercase();
                    if status_code == 200 && normalized == "ok" {
                        ManagedLlamaServerPortProbe {
                            occupancy_status: ManagedLlamaServerPortOccupancyStatus::ExternalServerDetected,
                            health_status: ManagedLlamaServerHealthStatus::Ready,
                            response_body_preview: preview,
                            response_body_truncated: truncated,
                            summary: format!(
                                "localhost:{} is already serving a ready server that AEGIS did not start.",
                                config.port
                            ),
                            blockers: vec![notice(
                                "external_process_detected",
                                "A ready server is already listening on the managed localhost port and is not AEGIS-owned.",
                            )],
                            warnings: vec![notice(
                                "external_server_detected",
                                "The configured localhost port appears to be owned by an external server.",
                            )],
                            next_required_actions: vec!["Stop the external process or choose another port.".to_string()],
                        }
                    } else if status_code == 503 || normalized.contains("loading") {
                        ManagedLlamaServerPortProbe {
                            occupancy_status: ManagedLlamaServerPortOccupancyStatus::ExternalServerDetected,
                            health_status: ManagedLlamaServerHealthStatus::Loading,
                            response_body_preview: preview,
                            response_body_truncated: truncated,
                            summary: format!(
                                "localhost:{} is already serving a loading server that AEGIS did not start.",
                                config.port
                            ),
                            blockers: vec![notice(
                                "external_process_detected",
                                "A loading server is already listening on the managed localhost port and is not AEGIS-owned.",
                            )],
                            warnings: vec![notice(
                                "external_server_detected",
                                "The configured localhost port appears to be owned by an external server.",
                            )],
                            next_required_actions: vec!["Stop the external process or choose another port.".to_string()],
                        }
                    } else {
                        ManagedLlamaServerPortProbe {
                            occupancy_status: ManagedLlamaServerPortOccupancyStatus::PortOccupied,
                            health_status: ManagedLlamaServerHealthStatus::Failed,
                            response_body_preview: preview,
                            response_body_truncated: truncated,
                            summary: format!(
                                "localhost:{} is occupied, but AEGIS could not verify ownership from the health response.",
                                config.port
                            ),
                            blockers: vec![notice(
                                "port_occupied",
                                "The configured localhost port is already in use.",
                            )],
                            warnings: vec![notice(
                                "unknown_owner",
                                "The occupied port could not be verified as managed or external from the health response.",
                            )],
                            next_required_actions: vec!["Stop the external process or choose another port.".to_string()],
                        }
                    }
                }
                Err(error) if error.is_timeout() => ManagedLlamaServerPortProbe {
                    occupancy_status: ManagedLlamaServerPortOccupancyStatus::PortOccupied,
                    health_status: ManagedLlamaServerHealthStatus::Unreachable,
                    response_body_preview: String::new(),
                    response_body_truncated: false,
                    summary: format!("localhost:{} is occupied, but health could not be reached within the bounded probe.", config.port),
                    blockers: vec![notice(
                        "port_occupied",
                        "The configured localhost port is already in use.",
                    )],
                    warnings: vec![notice(
                        "unknown_owner",
                        "The occupied port could not be verified as managed or external from the bounded probe.",
                    )],
                    next_required_actions: vec!["Stop the external process or choose another port.".to_string()],
                },
                Err(error) => ManagedLlamaServerPortProbe {
                    occupancy_status: ManagedLlamaServerPortOccupancyStatus::UnknownOwner,
                    health_status: ManagedLlamaServerHealthStatus::Failed,
                    response_body_preview: String::new(),
                    response_body_truncated: false,
                    summary: format!("localhost:{} is occupied, but the health probe failed: {error}", config.port),
                    blockers: vec![notice(
                        "port_occupied",
                        "The configured localhost port is already in use.",
                    )],
                    warnings: vec![notice(
                        "unknown_owner",
                        "The occupied port could not be verified as managed or external.",
                    )],
                    next_required_actions: vec!["Stop the external process or choose another port.".to_string()],
                },
            }
        }
    }
}

fn localhost_socket_addr(host: &str, port: u16) -> Option<SocketAddr> {
    let normalized_host = if host.eq_ignore_ascii_case("localhost") {
        "127.0.0.1"
    } else {
        host
    };
    format!("{normalized_host}:{port}").parse().ok()
}

fn notice(kind: &str, message: &str) -> ManagedLlamaServerNotice {
    ManagedLlamaServerNotice {
        kind: kind.to_string(),
        message: message.to_string(),
    }
}

fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn normalize_host(value: &str) -> String {
    let normalized = value.trim();
    if normalized.is_empty() {
        DEFAULT_HOST.to_string()
    } else {
        normalized.to_string()
    }
}

fn is_local_host(host: &str) -> bool {
    matches!(host, "127.0.0.1" | "localhost")
}

fn has_parent_dir_component(path: &str) -> bool {
    Path::new(path)
        .components()
        .any(|component| matches!(component, Component::ParentDir))
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

fn safe_file_name(path: Option<&str>) -> Option<String> {
    path.and_then(|value| {
        Path::new(value)
            .file_name()
            .and_then(|component| component.to_str())
            .map(|component| component.to_string())
    })
}

fn preview_text(value: &str, limit: usize) -> (String, bool) {
    let mut chars = value.trim().chars();
    let preview = chars.by_ref().take(limit).collect::<String>();
    let truncated = chars.next().is_some();
    if truncated {
        (format!("{preview}…"), true)
    } else {
        (preview, false)
    }
}

fn sweep_runtime(runtime: &mut ManagedLlamaServerRuntime) {
    let Some(active) = runtime.active.as_mut() else {
        return;
    };
    match active.child.try_wait() {
        Ok(Some(status)) => {
            runtime.last_exit_code = status.code();
            runtime.last_status = if status.success() {
                ManagedLlamaServerLifecycleStatus::Stopped
            } else {
                ManagedLlamaServerLifecycleStatus::Failed
            };
            runtime.last_health_status = ManagedLlamaServerHealthStatus::NotStarted;
            runtime.last_health_preview.clear();
            runtime.last_health_truncated = false;
            runtime.active = None;
        }
        Ok(None) => {}
        Err(_) => {
            runtime.last_exit_code = None;
            runtime.last_status = ManagedLlamaServerLifecycleStatus::Failed;
            runtime.last_health_status = ManagedLlamaServerHealthStatus::Failed;
            runtime.last_health_preview.clear();
            runtime.last_health_truncated = false;
            runtime.active = None;
        }
    }
}

fn spawn_monitor_thread(runtime: Arc<Mutex<ManagedLlamaServerRuntime>>) {
    thread::spawn(move || loop {
        let mut should_break = false;
        {
            let mut guard = runtime.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            let Some(active) = guard.active.as_mut() else {
                break;
            };
            match active.child.try_wait() {
                Ok(Some(status)) => {
                    guard.last_exit_code = status.code();
                    guard.last_status = if status.success() {
                        ManagedLlamaServerLifecycleStatus::Stopped
                    } else {
                        ManagedLlamaServerLifecycleStatus::Failed
                    };
                    guard.last_health_status = ManagedLlamaServerHealthStatus::NotStarted;
                    guard.last_health_preview.clear();
                    guard.last_health_truncated = false;
                    guard.active = None;
                    should_break = true;
                }
                Ok(None) => {}
                Err(_) => {
                    guard.last_exit_code = None;
                    guard.last_status = ManagedLlamaServerLifecycleStatus::Failed;
                    guard.last_health_status = ManagedLlamaServerHealthStatus::Failed;
                    guard.last_health_preview.clear();
                    guard.last_health_truncated = false;
                    guard.active = None;
                    should_break = true;
                }
            }
        }
        if should_break {
            break;
        }
        thread::sleep(Duration::from_millis(MONITOR_SLEEP_MS));
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::net::TcpListener;
    use reqwest::blocking::Client;
    use tempfile::tempdir;

    fn launch_request(
        executable_path: Option<&str>,
        model_path: Option<&str>,
        host: Option<&str>,
        port: Option<u16>,
        alias: Option<&str>,
        context_window: Option<u32>,
        gpu_layers: Option<i32>,
    ) -> ManagedLlamaServerLaunchPlanRequest {
        ManagedLlamaServerLaunchPlanRequest {
            executable_path: executable_path.map(|value| value.to_string()),
            model_path: model_path.map(|value| value.to_string()),
            host: host.map(|value| value.to_string()),
            port,
            alias: alias.map(|value| value.to_string()),
            context_window,
            gpu_layers,
        }
    }

    fn helper_server_executable(temp: &tempfile::TempDir) -> PathBuf {
        let source_path = temp.path().join("managed_server_helper.rs");
        let executable_path = temp
            .path()
            .join(if cfg!(windows) { "managed_server_helper.exe" } else { "managed_server_helper" });
        let source = r##"
use std::env;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

fn main() {
    let mut host = String::new();
    let mut port = String::new();
    let mut model = String::new();
    let mut context = String::new();
    let mut gpu = String::new();
    let mut alias = String::new();
    let args = env::args().skip(1).collect::<Vec<_>>();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-m" => { model = args.get(i + 1).cloned().unwrap_or_default(); i += 2; }
            "-c" => { context = args.get(i + 1).cloned().unwrap_or_default(); i += 2; }
            "-ngl" => { gpu = args.get(i + 1).cloned().unwrap_or_default(); i += 2; }
            "--host" => { host = args.get(i + 1).cloned().unwrap_or_default(); i += 2; }
            "--port" => { port = args.get(i + 1).cloned().unwrap_or_default(); i += 2; }
            "--alias" => { alias = args.get(i + 1).cloned().unwrap_or_default(); i += 2; }
            _ => { i += 1; }
        }
    }
    if model.is_empty() || context.is_empty() || gpu.is_empty() || host.is_empty() || port.is_empty() || alias.is_empty() {
        eprintln!("missing managed llama-server arguments");
        std::process::exit(2);
    }
    let listener = TcpListener::bind(format!("{}:{}", host, port)).unwrap();
    for incoming in listener.incoming() {
        let mut stream = incoming.unwrap();
        let mut buffer = [0_u8; 1024];
        let read = stream.read(&mut buffer).unwrap_or(0);
        let request = String::from_utf8_lossy(&buffer[..read]);
        let (status_line, body) = if request.starts_with("GET /health") {
            ("HTTP/1.1 200 OK", "ok")
        } else if request.starts_with("POST /v1/chat/completions") {
            if request.contains("MALFORMED_CHAT_DIAGNOSTIC") {
                ("HTTP/1.1 200 OK", "{not valid json")
            } else {
                if request.contains("SLOW_CHAT_DIAGNOSTIC") {
                    thread::sleep(Duration::from_millis(650));
                }
                (
                    "HTTP/1.1 200 OK",
                    r#"{"id":"chatcmpl-test","object":"chat.completion","created":1,"model":"aegis-local-gemma","choices":[{"index":0,"message":{"role":"assistant","content":"READY"},"finish_reason":"stop"}],"usage":{"prompt_tokens":5,"completion_tokens":1,"total_tokens":6}}"#,
                )
            }
        } else if request.starts_with("POST /v1/completions") {
            if request.contains("MALFORMED_SMOKE_DIAGNOSTIC") {
                ("HTTP/1.1 200 OK", "{not valid json")
            } else {
                if request.contains("SLOW_SMOKE_DIAGNOSTIC") {
                    thread::sleep(Duration::from_millis(650));
                }
                let smoke_text = if request.contains("LONG_SMOKE_DIAGNOSTIC") {
                    "READY ".repeat(128)
                } else {
                    "READY".to_string()
                };
                let response = format!(
                    r#"{{"id":"cmpl-test","object":"text_completion","created":1,"model":"aegis-local-gemma","choices":[{{"index":0,"text":"{}","finish_reason":"stop"}}],"usage":{{"prompt_tokens":5,"completion_tokens":1,"total_tokens":6}}}}"#,
                    smoke_text
                );
                ("HTTP/1.1 200 OK", Box::leak(response.into_boxed_str()) as &str)
            }
        } else {
            ("HTTP/1.1 404 Not Found", "not found")
        };
        let response = format!("{status_line}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
        let _ = stream.write_all(response.as_bytes());
    }
}
"##;
        fs::write(&source_path, source).unwrap();
        let rustc = env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
        let status = Command::new(rustc)
            .arg(&source_path)
            .arg("-O")
            .arg("-o")
            .arg(&executable_path)
            .status()
            .unwrap();
        assert!(status.success());
        executable_path
    }

    fn free_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        port
    }

    fn managed_chat_status_preview(
        lifecycle_status: ManagedLlamaServerLifecycleStatus,
        health_status: ManagedLlamaServerHealthStatus,
        host: Option<&str>,
        port: Option<u16>,
        alias: Option<&str>,
        safe_model_file_name: Option<&str>,
    ) -> ManagedLlamaServerStatusPreview {
        let owns_active_server = matches!(
            lifecycle_status,
            ManagedLlamaServerLifecycleStatus::Running
                | ManagedLlamaServerLifecycleStatus::Starting
                | ManagedLlamaServerLifecycleStatus::AlreadyRunning
        );
        let port_occupied = !matches!(
            lifecycle_status,
            ManagedLlamaServerLifecycleStatus::NotStarted
                | ManagedLlamaServerLifecycleStatus::Stopped
                | ManagedLlamaServerLifecycleStatus::Failed
                | ManagedLlamaServerLifecycleStatus::Blocked
        );
        ManagedLlamaServerStatusPreview {
            lifecycle_status,
            health_status,
            owns_active_server,
            port_occupied,
            port_occupied_by_unmanaged_process: false,
            port_occupancy_status: ManagedLlamaServerPortOccupancyStatus::ManagedOwned,
            host: host.map(|value| value.to_string()),
            port,
            alias: alias.map(|value| value.to_string()),
            process_id: Some(4242),
            exit_code: None,
            safe_executable_file_name: Some("llama-server.exe".to_string()),
            safe_model_file_name: safe_model_file_name.map(|value| value.to_string()),
            health_url: host.zip(port).map(|(host, port)| format!("http://{}:{}/health", host, port)),
            response_body_preview: "ok".to_string(),
            response_body_truncated: false,
            blockers: Vec::new(),
            warnings: Vec::new(),
            next_required_actions: Vec::new(),
            summary: "managed server ready".to_string(),
            preview_only: true,
            no_process_spawn: true,
            no_model_output_used: true,
            no_answer_generation: true,
            no_persistence: true,
            no_artifact_write: true,
            no_lan_binding_by_default: true,
        }
    }

    fn smoke_diagnostic_request(
        allow_smoke_execution: bool,
        prompt: Option<&str>,
        max_output_tokens: Option<u32>,
        timeout_ms: Option<u64>,
    ) -> ManagedLlamaServerSmokeDiagnosticRequest {
        ManagedLlamaServerSmokeDiagnosticRequest {
            allow_smoke_execution,
            prompt: prompt.map(|value| value.to_string()),
            max_output_tokens,
            timeout_ms,
        }
    }

    fn assert_managed_llama_server_smoke_boundary_fields(
        preview: &ManagedLlamaServerSmokeDiagnosticPreview,
    ) {
        assert!(preview.diagnostic_only);
        assert!(preview.not_scholar_chat_answer);
        assert!(preview.no_grounding_applied);
        assert!(preview.no_evidence_pack_used);
        assert!(preview.no_artifact_write);
        assert!(preview.no_audit_write);
        assert!(preview.no_persistence);
        assert!(preview.no_final_answer_created);
    }

    fn spawn_chat_helper_server(
        executable: &Path,
        model: &Path,
        host: &str,
        port: u16,
        alias: &str,
    ) -> std::process::Child {
        std::process::Command::new(executable)
            .arg("-m")
            .arg(model)
            .arg("-c")
            .arg(DEFAULT_CONTEXT_WINDOW.to_string())
            .arg("-ngl")
            .arg(DEFAULT_GPU_LAYERS.to_string())
            .arg("--host")
            .arg(host)
            .arg("--port")
            .arg(port.to_string())
            .arg("--alias")
            .arg(alias)
            .spawn()
            .unwrap()
    }

    struct ChildGuard(Option<std::process::Child>);

    impl Drop for ChildGuard {
        fn drop(&mut self) {
            if let Some(mut child) = self.0.take() {
                let _ = child.kill();
                let _ = child.wait();
            }
        }
    }

    fn wait_for_chat_helper_server(host: &str, port: u16) {
        let client = Client::builder()
            .timeout(Duration::from_millis(250))
            .build()
            .unwrap();
        let url = format!("http://{host}:{port}/health");
        for _ in 0..40 {
            if let Ok(response) = client.get(&url).send() {
                if response.status().as_u16() == 200 {
                    let body = response.text().unwrap_or_default();
                    if body.trim() == "ok" {
                        return;
                    }
                }
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        panic!("managed chat helper server did not become healthy");
    }

    fn wait_for_port_available(port: u16) {
        for _ in 0..40 {
            if TcpListener::bind(("127.0.0.1", port)).is_ok() {
                return;
            }
            thread::sleep(Duration::from_millis(50));
        }
        panic!("port {port} did not become available");
    }

    #[test]
    fn launch_plan_blocks_remote_host() {
        let temp = tempdir().unwrap();
        let executable = temp.path().join("llama-server.exe");
        let model = temp.path().join("gemma.gguf");
        fs::write(&executable, "exe").unwrap();
        fs::write(&model, "model").unwrap();
        let result = preview_managed_llama_server_launch_plan(
            temp.path(),
            launch_request(
                Some(executable.to_string_lossy().as_ref()),
                Some(model.to_string_lossy().as_ref()),
                Some("0.0.0.0"),
                Some(DEFAULT_PORT),
                Some(DEFAULT_ALIAS),
                Some(DEFAULT_CONTEXT_WINDOW),
                Some(DEFAULT_GPU_LAYERS),
            ),
        )
        .unwrap();
        assert_eq!(result.status, ManagedLlamaServerLaunchPlanStatus::Blocked);
        assert!(result.blockers.iter().any(|notice| notice.kind == "host_not_local"));
    }

    #[test]
    fn launch_plan_is_ready_later_for_local_paths() {
        let temp = tempdir().unwrap();
        let executable = temp.path().join("llama-server.exe");
        let model = temp.path().join("gemma.gguf");
        fs::write(&executable, "exe").unwrap();
        fs::write(&model, "model").unwrap();
        let result = preview_managed_llama_server_launch_plan(
            temp.path(),
            launch_request(
                Some(executable.to_string_lossy().as_ref()),
                Some(model.to_string_lossy().as_ref()),
                Some("127.0.0.1"),
                Some(DEFAULT_PORT),
                Some(DEFAULT_ALIAS),
                Some(DEFAULT_CONTEXT_WINDOW),
                Some(DEFAULT_GPU_LAYERS),
            ),
        )
        .unwrap();
        assert_eq!(result.status, ManagedLlamaServerLaunchPlanStatus::LaunchReadyLater);
        assert_eq!(result.safe_executable_file_name.as_deref(), Some("llama-server.exe"));
        assert_eq!(result.safe_model_file_name.as_deref(), Some("gemma.gguf"));
    }

    #[test]
    fn lifecycle_start_health_and_stop_round_trip() {
        let temp = tempdir().unwrap();
        let helper_temp = tempdir().unwrap();
        let executable = helper_server_executable(&helper_temp);
        let model = temp.path().join("gemma.gguf");
        fs::write(&model, "model").unwrap();
        let port = free_port();
        let state = ManagedLlamaServerState::default();

        let start = start_managed_llama_server(
            temp.path(),
            &state,
            ManagedLlamaServerStartRequest {
                allow_server_start: true,
                launch_plan_request: launch_request(
                    Some(executable.to_string_lossy().as_ref()),
                    Some(model.to_string_lossy().as_ref()),
                    Some("127.0.0.1"),
                    Some(port),
                    Some(DEFAULT_ALIAS),
                    Some(DEFAULT_CONTEXT_WINDOW),
                    Some(DEFAULT_GPU_LAYERS),
                ),
            },
        )
        .unwrap();
        assert_eq!(start.lifecycle_status, ManagedLlamaServerLifecycleStatus::Starting);

        let mut healthy = false;
        for _ in 0..40 {
            let health = check_managed_llama_server_health(&state).unwrap();
            if health.health_status == ManagedLlamaServerHealthStatus::Ready {
                healthy = true;
                assert_eq!(health.lifecycle_status, ManagedLlamaServerLifecycleStatus::Running);
                assert_eq!(health.port, Some(port));
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }
        assert!(healthy);

        let stop = stop_managed_llama_server(&state).unwrap();
        assert_eq!(stop.lifecycle_status, ManagedLlamaServerLifecycleStatus::Stopped);

        let status = inspect_managed_llama_server_status(&state).unwrap();
        assert_eq!(status.lifecycle_status, ManagedLlamaServerLifecycleStatus::Stopped);
        assert_eq!(status.health_status, ManagedLlamaServerHealthStatus::NotStarted);
    }

    #[test]
    fn start_blocks_when_port_is_occupied_before_spawn() {
        let temp = tempdir().unwrap();
        let helper_temp = tempdir().unwrap();
        let executable = helper_server_executable(&helper_temp);
        let model = temp.path().join("gemma.gguf");
        fs::write(&model, "model").unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let state = ManagedLlamaServerState::default();

        let result = start_managed_llama_server(
            temp.path(),
            &state,
            ManagedLlamaServerStartRequest {
                allow_server_start: true,
                launch_plan_request: launch_request(
                    Some(executable.to_string_lossy().as_ref()),
                    Some(model.to_string_lossy().as_ref()),
                    Some("127.0.0.1"),
                    Some(port),
                    Some(DEFAULT_ALIAS),
                    Some(DEFAULT_CONTEXT_WINDOW),
                    Some(DEFAULT_GPU_LAYERS),
                ),
            },
        )
        .unwrap();

        assert_eq!(result.lifecycle_status, ManagedLlamaServerLifecycleStatus::PortOccupied);
        assert!(!result.owns_active_server);
        assert!(result.port_occupied);
        assert!(result.port_occupied_by_unmanaged_process);
        assert!(result.next_required_actions.iter().any(|action| action.contains("Stop the external process or choose another port.")));
        drop(listener);
    }

    #[test]
    fn start_blocks_external_server_and_status_reports_unmanaged_occupancy() {
        let temp = tempdir().unwrap();
        let helper_temp = tempdir().unwrap();
        let executable = helper_server_executable(&helper_temp);
        let model = temp.path().join("gemma.gguf");
        fs::write(&model, "model").unwrap();
        let port = free_port();
        let external_child = spawn_chat_helper_server(
            &executable,
            &model,
            "127.0.0.1",
            port,
            DEFAULT_ALIAS,
        );
        let _guard = ChildGuard(Some(external_child));
        wait_for_chat_helper_server("127.0.0.1", port);

        let state = ManagedLlamaServerState::default();
        let start_result = start_managed_llama_server(
            temp.path(),
            &state,
            ManagedLlamaServerStartRequest {
                allow_server_start: true,
                launch_plan_request: launch_request(
                    Some(executable.to_string_lossy().as_ref()),
                    Some(model.to_string_lossy().as_ref()),
                    Some("127.0.0.1"),
                    Some(port),
                    Some(DEFAULT_ALIAS),
                    Some(DEFAULT_CONTEXT_WINDOW),
                    Some(DEFAULT_GPU_LAYERS),
                ),
            },
        )
        .unwrap();

        assert_eq!(start_result.lifecycle_status, ManagedLlamaServerLifecycleStatus::ExternalServerDetected);
        assert!(!start_result.owns_active_server);
        assert!(start_result.port_occupied);
        assert!(start_result.port_occupied_by_unmanaged_process);
        assert!(start_result.blockers.iter().any(|notice| notice.kind == "external_process_detected"));

        let status = inspect_managed_llama_server_status(&state).unwrap();
        assert_eq!(status.lifecycle_status, ManagedLlamaServerLifecycleStatus::ExternalServerDetected);
        assert!(!status.owns_active_server);
        assert!(status.port_occupied);
        assert!(status.port_occupied_by_unmanaged_process);
    }

    #[test]
    fn stop_without_active_server_is_safe() {
        let state = ManagedLlamaServerState::default();
        let stop = stop_managed_llama_server(&state).unwrap();
        assert_eq!(stop.lifecycle_status, ManagedLlamaServerLifecycleStatus::NotStarted);
        assert!(!stop.owns_active_server);
        assert!(!stop.port_occupied);
    }

    #[test]
    fn drop_cleanup_stops_managed_server() {
        let temp = tempdir().unwrap();
        let helper_temp = tempdir().unwrap();
        let executable = helper_server_executable(&helper_temp);
        let model = temp.path().join("gemma.gguf");
        fs::write(&model, "model").unwrap();
        let port = free_port();

        {
            let state = ManagedLlamaServerState::default();
            let start = start_managed_llama_server(
                temp.path(),
                &state,
                ManagedLlamaServerStartRequest {
                    allow_server_start: true,
                    launch_plan_request: launch_request(
                        Some(executable.to_string_lossy().as_ref()),
                        Some(model.to_string_lossy().as_ref()),
                        Some("127.0.0.1"),
                        Some(port),
                        Some(DEFAULT_ALIAS),
                        Some(DEFAULT_CONTEXT_WINDOW),
                        Some(DEFAULT_GPU_LAYERS),
                    ),
                },
            )
            .unwrap();
            assert_eq!(start.lifecycle_status, ManagedLlamaServerLifecycleStatus::Starting);
            wait_for_chat_helper_server("127.0.0.1", port);
        }

        wait_for_port_available(port);
    }

    #[test]
    fn chat_diagnostic_blocks_without_consent() {
        let status = managed_chat_status_preview(
            ManagedLlamaServerLifecycleStatus::Running,
            ManagedLlamaServerHealthStatus::Ready,
            Some("127.0.0.1"),
            Some(48921),
            Some(DEFAULT_ALIAS),
            Some("gemma.gguf"),
        );
        let result = run_managed_llama_server_chat_diagnostic_from_status(
            &status,
            ManagedLlamaServerChatDiagnosticRequest {
                allow_chat_diagnostic: false,
                prompt: Some("Say READY in one short sentence.".to_string()),
                max_tokens: Some(16),
                temperature: Some(0.2),
                timeout_ms: Some(5_000),
            },
        );
        assert_eq!(result.status, ManagedLlamaServerChatDiagnosticStatus::Blocked);
        assert!(!result.request_attempted);
        assert!(result.blockers.iter().any(|notice| notice.kind == "chat_diagnostic_consent_missing"));
    }

    #[test]
    fn chat_diagnostic_requires_server_ready() {
        let status = status_preview_from_empty(
            ManagedLlamaServerLifecycleStatus::NotStarted,
            ManagedLlamaServerHealthStatus::NotStarted,
            "No AEGIS-managed llama-server is running.".to_string(),
        );
        let result = run_managed_llama_server_chat_diagnostic_from_status(
            &status,
            ManagedLlamaServerChatDiagnosticRequest {
                allow_chat_diagnostic: true,
                prompt: Some("Say READY in one short sentence.".to_string()),
                max_tokens: Some(16),
                temperature: Some(0.2),
                timeout_ms: Some(5_000),
            },
        );
        assert_eq!(result.status, ManagedLlamaServerChatDiagnosticStatus::ServerNotReady);
        assert!(!result.request_attempted);
        assert!(result.blockers.iter().any(|notice| notice.kind == "server_not_ready"));
    }

    #[test]
    fn chat_diagnostic_rejects_remote_host() {
        let status = managed_chat_status_preview(
            ManagedLlamaServerLifecycleStatus::Running,
            ManagedLlamaServerHealthStatus::Ready,
            Some("0.0.0.0"),
            Some(48921),
            Some(DEFAULT_ALIAS),
            Some("gemma.gguf"),
        );
        let result = run_managed_llama_server_chat_diagnostic_from_status(
            &status,
            ManagedLlamaServerChatDiagnosticRequest {
                allow_chat_diagnostic: true,
                prompt: Some("Say READY in one short sentence.".to_string()),
                max_tokens: Some(16),
                temperature: Some(0.2),
                timeout_ms: Some(5_000),
            },
        );
        assert_eq!(result.status, ManagedLlamaServerChatDiagnosticStatus::Blocked);
        assert!(!result.request_attempted);
        assert!(result.blockers.iter().any(|notice| notice.kind == "host_not_local"));
    }

    #[test]
    fn chat_diagnostic_bounds_inputs_and_parses_minimal_response() {
        let temp = tempdir().unwrap();
        let helper_temp = tempdir().unwrap();
        let executable = helper_server_executable(&helper_temp);
        let model = temp.path().join("gemma.gguf");
        fs::write(&model, "model").unwrap();
        let port = free_port();
        let _guard = ChildGuard(Some(spawn_chat_helper_server(
            &executable,
            &model,
            "127.0.0.1",
            port,
            DEFAULT_ALIAS,
        )));
        wait_for_chat_helper_server("127.0.0.1", port);

        let result = run_managed_llama_server_chat_diagnostic_from_status(
            &managed_chat_status_preview(
                ManagedLlamaServerLifecycleStatus::Running,
                ManagedLlamaServerHealthStatus::Ready,
                Some("127.0.0.1"),
                Some(port),
                Some(DEFAULT_ALIAS),
                Some("gemma.gguf"),
            ),
            ManagedLlamaServerChatDiagnosticRequest {
                allow_chat_diagnostic: true,
                prompt: Some("A".repeat(300)),
                max_tokens: Some(999),
                temperature: Some(5.0),
                timeout_ms: Some(1),
            },
        );

        assert_eq!(result.status, ManagedLlamaServerChatDiagnosticStatus::DiagnosticSucceeded);
        assert!(result.request_attempted);
        assert_eq!(result.prompt_char_count, 256);
        assert_eq!(result.max_tokens, 64);
        assert!((result.temperature - 2.0).abs() < f32::EPSILON);
        assert_eq!(result.timeout_ms, 250);
        assert_eq!(result.http_status, Some(200));
        assert_eq!(result.extracted_message_preview.as_deref(), Some("READY"));
        assert!(result.response_preview.contains("chat.completion"));
        assert!(result.warnings.iter().any(|notice| notice.kind == "prompt_truncated"));
        assert!(result.warnings.iter().any(|notice| notice.kind == "max_tokens_clamped"));
        assert!(result.warnings.iter().any(|notice| notice.kind == "temperature_clamped"));
        assert!(result.warnings.iter().any(|notice| notice.kind == "timeout_clamped"));
    }

    #[test]
    fn chat_diagnostic_malformed_response_is_failed() {
        let temp = tempdir().unwrap();
        let helper_temp = tempdir().unwrap();
        let executable = helper_server_executable(&helper_temp);
        let model = temp.path().join("gemma.gguf");
        fs::write(&model, "model").unwrap();
        let port = free_port();
        let _guard = ChildGuard(Some(spawn_chat_helper_server(
            &executable,
            &model,
            "127.0.0.1",
            port,
            DEFAULT_ALIAS,
        )));
        wait_for_chat_helper_server("127.0.0.1", port);

        let result = run_managed_llama_server_chat_diagnostic_from_status(
            &managed_chat_status_preview(
                ManagedLlamaServerLifecycleStatus::Running,
                ManagedLlamaServerHealthStatus::Ready,
                Some("127.0.0.1"),
                Some(port),
                Some(DEFAULT_ALIAS),
                Some("gemma.gguf"),
            ),
            ManagedLlamaServerChatDiagnosticRequest {
                allow_chat_diagnostic: true,
                prompt: Some("MALFORMED_CHAT_DIAGNOSTIC".to_string()),
                max_tokens: Some(16),
                temperature: Some(0.2),
                timeout_ms: Some(5_000),
            },
        );

        assert_eq!(result.status, ManagedLlamaServerChatDiagnosticStatus::DiagnosticFailed);
        assert!(result.request_attempted);
        assert_eq!(result.extracted_message_preview, None);
        assert!(result.blockers.iter().any(|notice| notice.kind == "response_parse_failed"));
    }

    #[test]
    fn chat_diagnostic_times_out_when_response_is_too_slow() {
        let temp = tempdir().unwrap();
        let helper_temp = tempdir().unwrap();
        let executable = helper_server_executable(&helper_temp);
        let model = temp.path().join("gemma.gguf");
        fs::write(&model, "model").unwrap();
        let port = free_port();
        let _guard = ChildGuard(Some(spawn_chat_helper_server(
            &executable,
            &model,
            "127.0.0.1",
            port,
            DEFAULT_ALIAS,
        )));
        wait_for_chat_helper_server("127.0.0.1", port);

        let result = run_managed_llama_server_chat_diagnostic_from_status(
            &managed_chat_status_preview(
                ManagedLlamaServerLifecycleStatus::Running,
                ManagedLlamaServerHealthStatus::Ready,
                Some("127.0.0.1"),
                Some(port),
                Some(DEFAULT_ALIAS),
                Some("gemma.gguf"),
            ),
            ManagedLlamaServerChatDiagnosticRequest {
                allow_chat_diagnostic: true,
                prompt: Some("SLOW_CHAT_DIAGNOSTIC".to_string()),
                max_tokens: Some(16),
                temperature: Some(0.2),
                timeout_ms: Some(1),
            },
        );

        assert_eq!(result.status, ManagedLlamaServerChatDiagnosticStatus::TimedOut);
        assert!(result.request_attempted);
        assert!(result.blockers.iter().any(|notice| notice.kind == "request_timeout"));
    }

    #[test]
    fn smoke_diagnostic_blocks_without_consent() {
        let result = run_managed_llama_server_smoke_diagnostic_from_status(
            &managed_chat_status_preview(
                ManagedLlamaServerLifecycleStatus::Running,
                ManagedLlamaServerHealthStatus::Ready,
                Some("127.0.0.1"),
                Some(48921),
                Some(DEFAULT_ALIAS),
                Some("gemma.gguf"),
            ),
            smoke_diagnostic_request(false, Some("Say READY in one short sentence."), Some(16), Some(5_000)),
        );

        assert_eq!(result.status, ManagedLlamaServerSmokeDiagnosticStatus::Blocked);
        assert!(!result.execution_attempted);
        assert!(result.blockers.iter().any(|notice| notice.kind == "smoke_execution_consent_missing"));
        assert_managed_llama_server_smoke_boundary_fields(&result);
    }

    #[test]
    fn smoke_diagnostic_reports_server_not_running_when_managed_server_is_not_active() {
        let result = run_managed_llama_server_smoke_diagnostic_from_status(
            &status_preview_from_empty(
                ManagedLlamaServerLifecycleStatus::NotStarted,
                ManagedLlamaServerHealthStatus::NotStarted,
                "No AEGIS-managed llama-server is running.".to_string(),
            ),
            smoke_diagnostic_request(true, Some("Say READY in one short sentence."), Some(16), Some(5_000)),
        );

        assert_eq!(result.status, ManagedLlamaServerSmokeDiagnosticStatus::ServerNotRunning);
        assert!(!result.execution_attempted);
        assert!(result.blockers.iter().any(|notice| notice.kind == "server_not_running"));
        assert_managed_llama_server_smoke_boundary_fields(&result);
    }

    #[test]
    fn smoke_diagnostic_rejects_remote_host() {
        let result = run_managed_llama_server_smoke_diagnostic_from_status(
            &managed_chat_status_preview(
                ManagedLlamaServerLifecycleStatus::Running,
                ManagedLlamaServerHealthStatus::Ready,
                Some("0.0.0.0"),
                Some(48921),
                Some(DEFAULT_ALIAS),
                Some("gemma.gguf"),
            ),
            smoke_diagnostic_request(true, Some("Say READY in one short sentence."), Some(16), Some(5_000)),
        );

        assert_eq!(result.status, ManagedLlamaServerSmokeDiagnosticStatus::Blocked);
        assert!(!result.execution_attempted);
        assert!(result.blockers.iter().any(|notice| notice.kind == "host_not_local"));
        assert_managed_llama_server_smoke_boundary_fields(&result);
    }

    #[test]
    fn smoke_diagnostic_succeeds_with_bounded_and_truncated_output() {
        let temp = tempdir().unwrap();
        let helper_temp = tempdir().unwrap();
        let executable = helper_server_executable(&helper_temp);
        let model = temp.path().join("gemma.gguf");
        fs::write(&model, "model").unwrap();
        let port = free_port();
        let _guard = ChildGuard(Some(spawn_chat_helper_server(
            &executable,
            &model,
            "127.0.0.1",
            port,
            DEFAULT_ALIAS,
        )));
        wait_for_chat_helper_server("127.0.0.1", port);

        let result = run_managed_llama_server_smoke_diagnostic_from_status(
            &managed_chat_status_preview(
                ManagedLlamaServerLifecycleStatus::Running,
                ManagedLlamaServerHealthStatus::Ready,
                Some("127.0.0.1"),
                Some(port),
                Some(DEFAULT_ALIAS),
                Some("gemma.gguf"),
            ),
            smoke_diagnostic_request(true, Some("LONG_SMOKE_DIAGNOSTIC"), Some(999), Some(5_000)),
        );

        assert_eq!(result.status, ManagedLlamaServerSmokeDiagnosticStatus::SmokeSucceeded);
        assert!(result.execution_attempted);
        assert_eq!(result.http_status, Some(200));
        assert!(result.response_preview.contains("text_completion"));
        assert!(result.response_preview_truncated);
        assert!(result
            .extracted_output_preview
            .as_deref()
            .map_or(false, |value| value.chars().count() <= CHAT_DIAGNOSTIC_MESSAGE_PREVIEW_LIMIT + 1));
        assert!(result.warnings.iter().any(|notice| notice.kind == "diagnostic_only"));
        assert!(result.warnings.iter().any(|notice| notice.kind == "diagnostic_success"));
        assert!(result.warnings.iter().any(|notice| notice.kind == "not_scholar_chat_answer"));
        assert_managed_llama_server_smoke_boundary_fields(&result);
    }

    #[test]
    fn smoke_diagnostic_times_out_when_response_is_too_slow() {
        let temp = tempdir().unwrap();
        let helper_temp = tempdir().unwrap();
        let executable = helper_server_executable(&helper_temp);
        let model = temp.path().join("gemma.gguf");
        fs::write(&model, "model").unwrap();
        let port = free_port();
        let _guard = ChildGuard(Some(spawn_chat_helper_server(
            &executable,
            &model,
            "127.0.0.1",
            port,
            DEFAULT_ALIAS,
        )));
        wait_for_chat_helper_server("127.0.0.1", port);

        let result = run_managed_llama_server_smoke_diagnostic_from_status(
            &managed_chat_status_preview(
                ManagedLlamaServerLifecycleStatus::Running,
                ManagedLlamaServerHealthStatus::Ready,
                Some("127.0.0.1"),
                Some(port),
                Some(DEFAULT_ALIAS),
                Some("gemma.gguf"),
            ),
            smoke_diagnostic_request(true, Some("SLOW_SMOKE_DIAGNOSTIC"), Some(16), Some(1)),
        );

        assert_eq!(result.status, ManagedLlamaServerSmokeDiagnosticStatus::TimedOut);
        assert!(result.execution_attempted);
        assert!(result.blockers.iter().any(|notice| notice.kind == "request_timeout"));
        assert_managed_llama_server_smoke_boundary_fields(&result);
    }

    #[test]
    fn smoke_diagnostic_refuses_unmanaged_server_and_leaves_it_running() {
        let temp = tempdir().unwrap();
        let helper_temp = tempdir().unwrap();
        let executable = helper_server_executable(&helper_temp);
        let model = temp.path().join("gemma.gguf");
        fs::write(&model, "model").unwrap();
        let port = free_port();
        let external_child = spawn_chat_helper_server(
            &executable,
            &model,
            "127.0.0.1",
            port,
            DEFAULT_ALIAS,
        );
        let _guard = ChildGuard(Some(external_child));
        wait_for_chat_helper_server("127.0.0.1", port);

        let state = ManagedLlamaServerState::default();
        let start_result = start_managed_llama_server(
            temp.path(),
            &state,
            ManagedLlamaServerStartRequest {
                allow_server_start: true,
                launch_plan_request: launch_request(
                    Some(executable.to_string_lossy().as_ref()),
                    Some(model.to_string_lossy().as_ref()),
                    Some("127.0.0.1"),
                    Some(port),
                    Some(DEFAULT_ALIAS),
                    Some(DEFAULT_CONTEXT_WINDOW),
                    Some(DEFAULT_GPU_LAYERS),
                ),
            },
        )
        .unwrap();

        assert_eq!(start_result.lifecycle_status, ManagedLlamaServerLifecycleStatus::ExternalServerDetected);
        assert!(start_result.port_occupied_by_unmanaged_process);

        let result = run_managed_llama_server_smoke_diagnostic(
            &state,
            smoke_diagnostic_request(true, Some("Say READY in one short sentence."), Some(16), Some(5_000)),
        )
        .unwrap();

        assert_eq!(result.status, ManagedLlamaServerSmokeDiagnosticStatus::Blocked);
        assert!(!result.execution_attempted);
        assert!(result.blockers.iter().any(|notice| notice.kind == "external_server_detected"));
        assert_managed_llama_server_smoke_boundary_fields(&result);

        wait_for_chat_helper_server("127.0.0.1", port);
    }
}
