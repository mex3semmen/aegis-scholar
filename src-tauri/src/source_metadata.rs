use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    Pdf,
    LectureSlides,
    Paper,
    Textbook,
    MarkdownNote,
    WebSnapshot,
    DatasetNote,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IngestionStatus {
    Registered,
    ExtractionPending,
    Extracted,
    Chunked,
    Indexed,
    AnswerDrafted,
    EvidenceReady,
    Failed,
    Removed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMetadataInput {
    pub title: String,
    pub source_type: SourceType,
    pub discipline: String,
    pub subdiscipline: Option<String>,
    pub language: String,
    pub tags: Vec<String>,
    pub reliability_notes: Option<String>,
}

impl SourceMetadataInput {
    pub fn validate(&self) -> Result<(), String> {
        if self.title.trim().is_empty() {
            return Err("title is required".to_string());
        }
        if self.discipline.trim().is_empty() {
            return Err("discipline is required".to_string());
        }
        if self.language.trim().is_empty() {
            return Err("language is required".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SourceMetadataPatch {
    pub title: Option<String>,
    pub discipline: Option<String>,
    pub subdiscipline: Option<Option<String>>,
    pub language: Option<String>,
    pub tags: Option<Vec<String>>,
    pub reliability_notes: Option<Option<String>>,
}

impl SourceMetadataPatch {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(title) = &self.title {
            if title.trim().is_empty() {
                return Err("title cannot be empty".to_string());
            }
        }
        if let Some(discipline) = &self.discipline {
            if discipline.trim().is_empty() {
                return Err("discipline cannot be empty".to_string());
            }
        }
        if let Some(language) = &self.language {
            if language.trim().is_empty() {
                return Err("language cannot be empty".to_string());
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceRecord {
    pub source_id: String,
    pub version_id: String,
    pub title: String,
    pub source_type: SourceType,
    pub discipline: String,
    pub subdiscipline: Option<String>,
    pub language: String,
    pub path: PathBuf,
    pub content_hash: String,
    pub created_at: DateTime<Utc>,
    pub ingestion_status: IngestionStatus,
    pub tags: Vec<String>,
    pub reliability_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusStatus {
    pub source_count: usize,
    pub registered_count: usize,
    pub extracted_count: usize,
    pub failed_count: usize,
}
