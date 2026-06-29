use crate::errors::AegisResult;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 48_921;
const DEFAULT_ALIAS: &str = "aegis-local-gemma";
const DEFAULT_CONTEXT_WINDOW: u32 = 4_096;
const DEFAULT_GPU_LAYERS: i32 = 0;
const HEALTH_TIMEOUT_MS: u64 = 1_500;
const HEALTH_PREVIEW_LIMIT: usize = 256;
const MONITOR_SLEEP_MS: u64 = 250;

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

#[derive(Clone, Default)]
pub struct ManagedLlamaServerState {
    runtime: Arc<Mutex<ManagedLlamaServerRuntime>>,
}

struct ManagedLlamaServerRuntime {
    active: Option<ManagedLlamaServerProcess>,
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
            return Ok(status_preview_from_plan(
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
            ));
        }
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
    }
    spawn_monitor_thread(state.runtime.clone());

    Ok(status_preview_from_plan(
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

    Ok(status_preview_from_plan(
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
    }

    let plan = plan_from_config(&process.config);
    Ok(status_preview_from_plan(
        ManagedLlamaServerLifecycleStatus::Stopped,
        ManagedLlamaServerHealthStatus::NotStarted,
        &plan,
        Some(process.pid),
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
    ))
}

pub fn inspect_managed_llama_server_status(state: &ManagedLlamaServerState) -> AegisResult<ManagedLlamaServerStatusPreview> {
    let runtime = state.lock_runtime();
    if let Some(active) = runtime.active.as_ref() {
        let plan = plan_from_config(&active.config);
        Ok(status_preview_from_plan(
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
        ))
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
        let source = r#"
use std::env;
use std::io::{Read, Write};
use std::net::TcpListener;

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
        } else {
            ("HTTP/1.1 404 Not Found", "not found")
        };
        let response = format!("{status_line}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len());
        let _ = stream.write_all(response.as_bytes());
    }
}
"#;
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
}
