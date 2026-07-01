use crate::corpus_paths::CorpusPaths;
use crate::errors::{AegisError, AegisResult};
use crate::scholar_chat::{
    ScholarChatAgenticWorkflowExecutionGatePreview,
    ScholarChatAgenticWorkflowPlanPreview,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use tempfile::NamedTempFile;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatTranscriptRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScholarChatTranscriptKind {
    Prompt,
    WorkflowPreview,
    ExecutionGate,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScholarChatTranscriptMessage {
    pub id: usize,
    pub role: ScholarChatTranscriptRole,
    pub kind: ScholarChatTranscriptKind,
    pub prompt: String,
    pub title: String,
    pub content: String,
    pub created_at: i64,
    pub workflow_preview: Option<ScholarChatAgenticWorkflowPlanPreview>,
    pub execution_gate_preview: Option<ScholarChatAgenticWorkflowExecutionGatePreview>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScholarChatSessionSummary {
    pub session_id: String,
    pub title: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub message_count: usize,
    pub last_message_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScholarChatSessionIndex {
    schema_version: usize,
    sessions: Vec<ScholarChatSessionSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScholarChatTranscriptEnvelope {
    session_id: String,
    recorded_at: i64,
    message: ScholarChatTranscriptMessage,
}

pub struct ScholarChatSessionStore {
    paths: CorpusPaths,
}

impl ScholarChatSessionStore {
    pub fn new(root: impl Into<std::path::PathBuf>) -> Self {
        Self {
            paths: CorpusPaths::new(root),
        }
    }

    fn index_path(&self) -> PathBuf {
        self.paths.chat_index_path()
    }

    fn session_path(&self, session_id: &str) -> AegisResult<std::path::PathBuf> {
        validate_session_id(session_id)?;
        Ok(self.paths.chat_session_path(session_id))
    }

    fn load_index(&self) -> AegisResult<ScholarChatSessionIndex> {
        let path = self.index_path();
        if !path.exists() {
            return Ok(ScholarChatSessionIndex {
                schema_version: 1,
                sessions: Vec::new(),
            });
        }

        let content = fs::read_to_string(path).map_err(|_| AegisError::ScholarChatSessionIndexReadFailed)?;
        serde_json::from_str(&content).map_err(|_| AegisError::ScholarChatSessionIndexReadFailed)
    }

    fn save_index(&self, index: &ScholarChatSessionIndex) -> AegisResult<()> {
        let path = self.index_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|_| AegisError::ScholarChatSessionIndexWriteFailed)?;
        }

        let parent = path.parent().ok_or(AegisError::ScholarChatSessionIndexWriteFailed)?;
        let mut temp_file = NamedTempFile::new_in(parent).map_err(|_| AegisError::ScholarChatSessionIndexWriteFailed)?;
        serde_json::to_writer_pretty(temp_file.as_file_mut(), index)
            .map_err(|_| AegisError::ScholarChatSessionIndexWriteFailed)?;
        temp_file
            .as_file_mut()
            .sync_all()
            .map_err(|_| AegisError::ScholarChatSessionIndexWriteFailed)?;
        temp_file
            .persist(path)
            .map_err(|_| AegisError::ScholarChatSessionIndexWriteFailed)?;
        Ok(())
    }

    fn update_session_summary<F>(&self, session_id: &str, mut update: F) -> AegisResult<ScholarChatSessionSummary>
    where
        F: FnMut(&mut ScholarChatSessionSummary),
    {
        let mut index = self.load_index()?;
        let summary = index
            .sessions
            .iter_mut()
            .find(|item| item.session_id == session_id)
            .ok_or_else(|| AegisError::ScholarChatSessionNotFound(session_id.to_string()))?;
        update(summary);
        let updated = summary.clone();
        index.sessions.sort_by(|left, right| right.updated_at.cmp(&left.updated_at).then_with(|| right.created_at.cmp(&left.created_at)).then_with(|| left.session_id.cmp(&right.session_id)));
        self.save_index(&index)?;
        Ok(updated)
    }

    fn replay_message(transcript: &mut Vec<ScholarChatTranscriptMessage>, message: ScholarChatTranscriptMessage) {
        match message.role {
            ScholarChatTranscriptRole::User => {
                if transcript
                    .iter()
                    .rev()
                    .find(|item| item.role == ScholarChatTranscriptRole::User)
                    .is_some_and(|last_user| last_user.prompt == message.prompt)
                {
                    return;
                }
                transcript.push(message);
            }
            ScholarChatTranscriptRole::Assistant | ScholarChatTranscriptRole::System => {
                if let Some(index) = transcript
                    .iter()
                    .position(|item| item.kind == message.kind && item.prompt == message.prompt)
                {
                    transcript[index] = message;
                } else {
                    transcript.push(message);
                }
            }
        }
    }

    pub fn list_sessions(&self) -> AegisResult<Vec<ScholarChatSessionSummary>> {
        let mut index = self.load_index()?;
        index.sessions.sort_by(|left, right| {
            right
                .updated_at
                .cmp(&left.updated_at)
                .then_with(|| right.created_at.cmp(&left.created_at))
                .then_with(|| left.session_id.cmp(&right.session_id))
        });
        Ok(index.sessions)
    }

    pub fn create_session(&self, title: Option<String>) -> AegisResult<ScholarChatSessionSummary> {
        self.paths.ensure_layout().map_err(|_| AegisError::ScholarChatSessionIndexWriteFailed)?;
        let mut index = self.load_index()?;
        let now = now_millis();
        let summary = ScholarChatSessionSummary {
            session_id: format!("schat_{}", Uuid::new_v4().simple()),
            title: normalize_title(title.unwrap_or_else(|| "Scholar Chat session".to_string()))?,
            created_at: now,
            updated_at: now,
            message_count: 0,
            last_message_at: None,
        };
        index.sessions.push(summary.clone());
        index.sessions.sort_by(|left, right| {
            right
                .updated_at
                .cmp(&left.updated_at)
                .then_with(|| right.created_at.cmp(&left.created_at))
                .then_with(|| left.session_id.cmp(&right.session_id))
        });
        self.save_index(&index)?;
        Ok(summary)
    }

    pub fn rename_session(&self, session_id: &str, title: String) -> AegisResult<ScholarChatSessionSummary> {
        let title = normalize_title(title)?;
        self.update_session_summary(session_id, |summary| {
            summary.title = title.clone();
            summary.updated_at = now_millis();
        })
    }

    pub fn delete_session(&self, session_id: &str) -> AegisResult<ScholarChatSessionSummary> {
        let mut index = self.load_index()?;
        let position = index
            .sessions
            .iter()
            .position(|item| item.session_id == session_id)
            .ok_or_else(|| AegisError::ScholarChatSessionNotFound(session_id.to_string()))?;
        let summary = index.sessions.remove(position);
        let session_path = self.session_path(session_id)?;
        if session_path.exists() {
            fs::remove_file(session_path).map_err(|_| AegisError::ScholarChatSessionTranscriptWriteFailed)?;
        }
        self.save_index(&index)?;
        Ok(summary)
    }

    pub fn load_session_transcript(&self, session_id: &str) -> AegisResult<Vec<ScholarChatTranscriptMessage>> {
        let session_path = self.session_path(session_id)?;
        if !session_path.exists() {
            return Ok(Vec::new());
        }

        let file = fs::File::open(&session_path).map_err(|_| AegisError::ScholarChatSessionTranscriptReadFailed)?;
        let reader = BufReader::new(file);
        let mut transcript = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|_| AegisError::ScholarChatSessionTranscriptReadFailed)?;
            if line.trim().is_empty() {
                continue;
            }
            let entry: ScholarChatTranscriptEnvelope = serde_json::from_str(&line)
                .map_err(|_| AegisError::ScholarChatSessionTranscriptReadFailed)?;
            if entry.session_id != session_id {
                return Err(AegisError::ScholarChatSessionTranscriptReadFailed);
            }
            Self::replay_message(&mut transcript, entry.message);
        }

        Ok(transcript)
    }

    pub fn append_transcript_entry(
        &self,
        session_id: &str,
        message: ScholarChatTranscriptMessage,
    ) -> AegisResult<ScholarChatSessionSummary> {
        let session_path = self.session_path(session_id)?;
        self.paths.ensure_layout().map_err(|_| AegisError::ScholarChatSessionTranscriptWriteFailed)?;
        let mut index = self.load_index()?;
        let summary = index
            .sessions
            .iter_mut()
            .find(|item| item.session_id == session_id)
            .ok_or_else(|| AegisError::ScholarChatSessionNotFound(session_id.to_string()))?;

        if let Some(parent) = session_path.parent() {
            fs::create_dir_all(parent).map_err(|_| AegisError::ScholarChatSessionTranscriptWriteFailed)?;
        }

        let envelope = ScholarChatTranscriptEnvelope {
            session_id: session_id.to_string(),
            recorded_at: now_millis(),
            message,
        };

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&session_path)
            .map_err(|_| AegisError::ScholarChatSessionTranscriptWriteFailed)?;
        let line = serde_json::to_string(&envelope).map_err(|_| AegisError::ScholarChatSessionTranscriptWriteFailed)?;
        writeln!(file, "{line}").map_err(|_| AegisError::ScholarChatSessionTranscriptWriteFailed)?;

        summary.message_count += 1;
        summary.updated_at = envelope.recorded_at;
        summary.last_message_at = Some(envelope.recorded_at);
        let updated = summary.clone();
        index.sessions.sort_by(|left, right| {
            right
                .updated_at
                .cmp(&left.updated_at)
                .then_with(|| right.created_at.cmp(&left.created_at))
                .then_with(|| left.session_id.cmp(&right.session_id))
        });
        self.save_index(&index)?;
        Ok(updated)
    }
}

fn normalize_title(title: String) -> AegisResult<String> {
    let normalized = title.replace(['\r', '\n'], " ").trim().to_string();
    if normalized.is_empty() {
        return Err(AegisError::ScholarChatSessionInvalidTitle);
    }
    Ok(normalized)
}

fn validate_session_id(session_id: &str) -> AegisResult<()> {
    if session_id.trim().is_empty()
        || session_id.contains('/')
        || session_id.contains('\\')
        || session_id.contains("..")
    {
        return Err(AegisError::ScholarChatSessionInvalidId);
    }
    Ok(())
}

fn now_millis() -> i64 {
    Utc::now().timestamp_millis()
}

impl Default for ScholarChatSessionIndex {
    fn default() -> Self {
        Self {
            schema_version: 1,
            sessions: Vec::new(),
        }
    }
}

pub fn list_scholar_chat_sessions(root: String) -> Result<Vec<ScholarChatSessionSummary>, String> {
    ScholarChatSessionStore::new(root)
        .list_sessions()
        .map_err(crate::errors::to_user_error)
}

pub fn create_scholar_chat_session(
    root: String,
    title: Option<String>,
) -> Result<ScholarChatSessionSummary, String> {
    ScholarChatSessionStore::new(root)
        .create_session(title)
        .map_err(crate::errors::to_user_error)
}

pub fn rename_scholar_chat_session(
    root: String,
    session_id: String,
    title: String,
) -> Result<ScholarChatSessionSummary, String> {
    ScholarChatSessionStore::new(root)
        .rename_session(&session_id, title)
        .map_err(crate::errors::to_user_error)
}

pub fn delete_scholar_chat_session(
    root: String,
    session_id: String,
) -> Result<ScholarChatSessionSummary, String> {
    ScholarChatSessionStore::new(root)
        .delete_session(&session_id)
        .map_err(crate::errors::to_user_error)
}

pub fn load_scholar_chat_session_transcript(
    root: String,
    session_id: String,
) -> Result<Vec<ScholarChatTranscriptMessage>, String> {
    ScholarChatSessionStore::new(root)
        .load_session_transcript(&session_id)
        .map_err(crate::errors::to_user_error)
}

pub fn append_scholar_chat_transcript_entry(
    root: String,
    session_id: String,
    message: ScholarChatTranscriptMessage,
) -> Result<ScholarChatSessionSummary, String> {
    ScholarChatSessionStore::new(root)
        .append_transcript_entry(&session_id, message)
        .map_err(crate::errors::to_user_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scholar_chat::{ScholarChatAgenticWorkflowExecutionGateDecision, ScholarChatAgenticWorkflowExecutionGateStatus, ScholarChatAgenticWorkflowFutureAction, ScholarChatAgenticWorkflowIntent, ScholarChatAgenticWorkflowPlanStatus, ScholarChatAgenticWorkflowExecutionGatePreview, ScholarChatAgenticWorkflowPlanPreview, GroundingPolicy, ScholarChatMode};

    fn sample_prompt_message(id: usize, prompt: &str) -> ScholarChatTranscriptMessage {
        ScholarChatTranscriptMessage {
            id,
            role: ScholarChatTranscriptRole::User,
            kind: ScholarChatTranscriptKind::Prompt,
            prompt: prompt.to_string(),
            title: "You".to_string(),
            content: prompt.to_string(),
            created_at: now_millis(),
            workflow_preview: None,
            execution_gate_preview: None,
        }
    }

    fn sample_workflow_preview_message(id: usize, prompt: &str, summary: &str) -> ScholarChatTranscriptMessage {
        ScholarChatTranscriptMessage {
            id,
            role: ScholarChatTranscriptRole::Assistant,
            kind: ScholarChatTranscriptKind::WorkflowPreview,
            prompt: prompt.to_string(),
            title: "AEGIS".to_string(),
            content: summary.to_string(),
            created_at: now_millis(),
            workflow_preview: Some(ScholarChatAgenticWorkflowPlanPreview {
                status: ScholarChatAgenticWorkflowPlanStatus::NeedsReview,
                recognized_intent: ScholarChatAgenticWorkflowIntent::AskLocalSources,
                normalized_prompt: prompt.to_string(),
                mode: ScholarChatMode::GeneralScholar,
                grounding_policy: GroundingPolicy::LocalFirst,
                selected_source_ids: vec![],
                selected_source_count: 0,
                required_local_context: vec![],
                planned_steps: vec!["Preview".to_string()],
                blockers: vec![],
                warnings: vec![],
                next_required_actions: vec![],
                summary: summary.to_string(),
                execution_allowed: false,
                preview_only: true,
                no_runtime_execution: true,
                no_llm_call: true,
                no_answer_generated: true,
                no_evidence_pack_built: true,
                no_persistence: true,
                no_artifact_write: true,
                no_registry_status_change: true,
                no_audit_write: true,
            }),
            execution_gate_preview: None,
        }
    }

    fn sample_execution_gate_message(id: usize, prompt: &str, blocked_reason: &str) -> ScholarChatTranscriptMessage {
        ScholarChatTranscriptMessage {
            id,
            role: ScholarChatTranscriptRole::Assistant,
            kind: ScholarChatTranscriptKind::ExecutionGate,
            prompt: prompt.to_string(),
            title: "AEGIS".to_string(),
            content: blocked_reason.to_string(),
            created_at: now_millis(),
            workflow_preview: None,
            execution_gate_preview: Some(ScholarChatAgenticWorkflowExecutionGatePreview {
                status: ScholarChatAgenticWorkflowExecutionGateStatus::NeedsReview,
                planned_intent: ScholarChatAgenticWorkflowIntent::AskLocalSources,
                gate_decision: ScholarChatAgenticWorkflowExecutionGateDecision::NeedsContext,
                consent_required: false,
                user_consent_present: false,
                allowed_future_action: ScholarChatAgenticWorkflowFutureAction::AskLocalSourcesLater,
                blocked_reason: blocked_reason.to_string(),
                blockers: vec![],
                warnings: vec![],
                required_local_context: vec![],
                planned_steps: vec!["Step".to_string()],
                next_required_actions: vec![],
                safety_invariants: vec!["preview_only".to_string()],
                selected_source_ids: vec![],
                selected_source_count: 0,
                execution_allowed_now: false,
                preview_only: true,
                no_filesystem_write: true,
                no_backend_mutation: true,
                no_runtime_execution: true,
                no_llm_call: true,
                no_network_call: true,
            }),
        }
    }

    #[test]
    fn session_store_scopes_chat_under_project_root() {
        let temp = tempfile::tempdir().unwrap();
        let paths = CorpusPaths::new(temp.path());

        assert_eq!(paths.chat_index_path(), temp.path().join(".aegis/chat/index.json"));
        assert_eq!(
            paths.chat_session_path("schat_123"),
            temp.path().join(".aegis/chat/sessions/schat_123.jsonl")
        );
    }

    #[test]
    fn session_store_appends_and_replays_transcript_messages() {
        let temp = tempfile::tempdir().unwrap();
        let store = ScholarChatSessionStore::new(temp.path());

        let session = store.create_session(Some("First session".to_string())).unwrap();
        store
            .append_transcript_entry(&session.session_id, sample_prompt_message(1, "What is a p-value?"))
            .unwrap();
        store
            .append_transcript_entry(
                &session.session_id,
                sample_workflow_preview_message(2, "What is a p-value?", "Preview one"),
            )
            .unwrap();
        store
            .append_transcript_entry(
                &session.session_id,
                sample_workflow_preview_message(3, "What is a p-value?", "Preview two"),
            )
            .unwrap();
        store
            .append_transcript_entry(
                &session.session_id,
                sample_execution_gate_message(4, "What is a p-value?", "Blocked for context"),
            )
            .unwrap();

        let transcript = store.load_session_transcript(&session.session_id).unwrap();
        assert_eq!(transcript.len(), 3);
        assert_eq!(transcript[0].role, ScholarChatTranscriptRole::User);
        assert_eq!(transcript[1].kind, ScholarChatTranscriptKind::WorkflowPreview);
        assert_eq!(transcript[1].content, "Preview two");
        assert_eq!(transcript[2].kind, ScholarChatTranscriptKind::ExecutionGate);
        assert_eq!(transcript[2].content, "Blocked for context");
    }

    #[test]
    fn session_store_starts_empty_and_creates_layout_lazily_on_first_write() {
        let temp = tempfile::tempdir().unwrap();
        let store = ScholarChatSessionStore::new(temp.path());

        assert!(!temp.path().join(".aegis/chat/index.json").exists());
        assert!(store.list_sessions().unwrap().is_empty());
        assert!(store
            .load_session_transcript("schat_missing")
            .unwrap()
            .is_empty());

        let created = store.create_session(None).unwrap();
        assert!(temp.path().join(".aegis/chat/index.json").exists());
        assert!(temp.path().join(".aegis/chat/sessions").exists());

        store
            .append_transcript_entry(&created.session_id, sample_prompt_message(1, "What should I do next?"))
            .unwrap();

        let sessions = store.list_sessions().unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].session_id, created.session_id);
        assert_eq!(sessions[0].message_count, 1);

        let transcript = store.load_session_transcript(&created.session_id).unwrap();
        assert_eq!(transcript.len(), 1);
        assert_eq!(transcript[0].content, "What should I do next?");
    }
}
