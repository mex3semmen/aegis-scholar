use crate::errors::{AegisError, AegisResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Component, Path, PathBuf};

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

fn validate_runtime_path(path: &str) -> AegisResult<()> {
    if Path::new(path)
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(AegisError::LocalModelRuntimeInvalidPath);
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

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
}
