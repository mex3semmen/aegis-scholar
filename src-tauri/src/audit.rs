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
    SourceExtracted,
    SourceExtractionFailed,
    SourceChunked,
    SourceChunkingFailed,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn append_audit_event_writes_json_line() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("events.jsonl");
        let event = AuditEvent::new(
            AuditEventType::SourceRegistered,
            Some("src_1".to_string()),
            Some("srcv_1".to_string()),
            "registered source",
        );

        append_audit_event(&path, &event).unwrap();

        let content = fs::read_to_string(path).unwrap();
        assert_eq!(content.lines().count(), 1);
        assert!(content.contains("src_1"));
    }

    #[test]
    fn append_audit_event_appends_multiple_lines() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("events.jsonl");

        let first = AuditEvent::new(AuditEventType::SourceRegistered, None, None, "one");
        let second = AuditEvent::new(AuditEventType::SourceRemoved, None, None, "two");

        append_audit_event(&path, &first).unwrap();
        append_audit_event(&path, &second).unwrap();

        let content = fs::read_to_string(path).unwrap();
        assert_eq!(content.lines().count(), 2);
    }
}
