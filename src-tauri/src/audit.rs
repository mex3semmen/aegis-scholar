use crate::errors::AegisResult;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    SourceRegistered,
    SourceUpdated,
    SourceRemoved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_id: String,
    pub event_type: AuditEventType,
    pub created_at: DateTime<Utc>,
    pub source_id: Option<String>,
    pub version_id: Option<String>,
    pub summary: String,
}

impl AuditEvent {
    pub fn new(
        event_type: AuditEventType,
        source_id: Option<String>,
        version_id: Option<String>,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            event_id: format!("aud_{}", Uuid::new_v4().simple()),
            event_type,
            created_at: Utc::now(),
            source_id,
            version_id,
            summary: summary.into(),
        }
    }
}

pub fn append_audit_event(path: &Path, event: &AuditEvent) -> AegisResult<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    let line = serde_json::to_string(event)?;
    writeln!(file, "{line}")?;
    Ok(())
}
