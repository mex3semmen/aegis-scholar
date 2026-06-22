use crate::errors::{AegisError, AegisResult};
use crate::source_metadata::{CorpusStatus, IngestionStatus, SourceMetadataPatch, SourceRecord};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tempfile::NamedTempFile;

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

        let mut temp_file = NamedTempFile::new_in(path.parent().unwrap_or_else(|| Path::new(".")))?;
        serde_json::to_writer_pretty(temp_file.as_file_mut(), self)?;
        temp_file.as_file_mut().sync_all()?;

        temp_file
            .persist(path)
            .map_err(|error| {
                if error.error.kind() == std::io::ErrorKind::NotFound {
                    AegisError::AtomicRegistryWriteFailed
                } else {
                    error.error.into()
                }
            })?;
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

    pub fn update_metadata(
        &mut self,
        source_id: &str,
        patch: SourceMetadataPatch,
    ) -> AegisResult<SourceRecord> {
        patch.validate().map_err(AegisError::InvalidMetadata)?;
        let source = self
            .sources
            .iter_mut()
            .find(|source| source.source_id == source_id)
            .ok_or_else(|| AegisError::SourceNotFound(source_id.to_string()))?;

        if let Some(title) = patch.title {
            source.title = title;
        }
        if let Some(discipline) = patch.discipline {
            source.discipline = discipline;
        }
        if let Some(subdiscipline) = patch.subdiscipline {
            source.subdiscipline = subdiscipline;
        }
        if let Some(language) = patch.language {
            source.language = language;
        }
        if let Some(tags) = patch.tags {
            source.tags = tags;
        }
        if let Some(reliability_notes) = patch.reliability_notes {
            source.reliability_notes = reliability_notes;
        }

        Ok(source.clone())
    }

    pub fn mark_removed(&mut self, source_id: &str) -> AegisResult<SourceRecord> {
        let source = self
            .sources
            .iter_mut()
            .find(|source| source.source_id == source_id)
            .ok_or_else(|| AegisError::SourceNotFound(source_id.to_string()))?;
        source.ingestion_status = IngestionStatus::Removed;
        Ok(source.clone())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_metadata::SourceType;
    use chrono::Utc;
    use std::fs;

    fn sample_source(path: impl Into<std::path::PathBuf>) -> SourceRecord {
        SourceRecord {
            source_id: "src_test".to_string(),
            version_id: "srcv_test".to_string(),
            title: "Sample".to_string(),
            source_type: SourceType::MarkdownNote,
            discipline: "psychology".to_string(),
            subdiscipline: None,
            language: "en".to_string(),
            path: path.into(),
            content_hash: "sha256:abc".to_string(),
            created_at: Utc::now(),
            ingestion_status: IngestionStatus::Registered,
            tags: vec!["tag".to_string()],
            reliability_notes: None,
        }
    }

    #[test]
    fn missing_registry_loads_as_empty() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("registry.json");
        let registry = SourceRegistry::load(&path).unwrap();
        assert!(registry.sources.is_empty());
    }

    #[test]
    fn malformed_registry_returns_error() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("registry.json");
        fs::write(&path, "{not-json").unwrap();

        let result = SourceRegistry::load(&path);
        assert!(matches!(result, Err(AegisError::Serde(_))));
    }

    #[test]
    fn atomic_write_produces_valid_json() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("registry.json");
        let mut registry = SourceRegistry::default();
        registry.add_source(sample_source(temp.path().join("copy.md"))).unwrap();

        registry.save(&path).unwrap();

        let saved = fs::read_to_string(&path).unwrap();
        let parsed: SourceRegistry = serde_json::from_str(&saved).unwrap();
        assert_eq!(parsed.sources.len(), 1);
    }

    #[test]
    fn source_list_survives_reload_after_registration() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("registry.json");
        let mut registry = SourceRegistry::default();
        registry.add_source(sample_source(temp.path().join("copy.md"))).unwrap();
        registry.save(&path).unwrap();

        let loaded = SourceRegistry::load(&path).unwrap();
        assert_eq!(loaded.list_sources().len(), 1);
        assert_eq!(loaded.get_source("src_test").unwrap().title, "Sample");
    }
}
