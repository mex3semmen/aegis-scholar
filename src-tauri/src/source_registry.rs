use crate::errors::{AegisError, AegisResult};
use crate::source_metadata::{CorpusStatus, IngestionStatus, SourceRecord};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SourceRegistry {
    pub sources: Vec<SourceRecord>,
}

impl SourceRegistry {
    pub fn load(path: &Path) -> AegisResult<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }

    pub fn save(&self, path: &Path) -> AegisResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn add_source(&mut self, source: SourceRecord) -> AegisResult<()> {
        if self.sources.iter().any(|item| item.content_hash == source.content_hash) {
            return Err(AegisError::DuplicateSource(source.content_hash));
        }
        self.sources.push(source);
        Ok(())
    }

    pub fn get_source(&self, source_id: &str) -> AegisResult<SourceRecord> {
        self.sources
            .iter()
            .find(|source| source.source_id == source_id)
            .cloned()
            .ok_or_else(|| AegisError::SourceNotFound(source_id.to_string()))
    }

    pub fn list_sources(&self) -> Vec<SourceRecord> {
        self.sources.clone()
    }

    pub fn status(&self) -> CorpusStatus {
        let mut registered_count = 0;
        let mut extracted_count = 0;
        let mut failed_count = 0;

        for source in &self.sources {
            match source.ingestion_status {
                IngestionStatus::Registered => registered_count += 1,
                IngestionStatus::Extracted => extracted_count += 1,
                IngestionStatus::Failed => failed_count += 1,
                _ => {}
            }
        }

        CorpusStatus {
            source_count: self.sources.len(),
            registered_count,
            extracted_count,
            failed_count,
        }
    }
}
