use crate::audit::{append_audit_event, AuditEvent, AuditEventType};
use crate::corpus_paths::CorpusPaths;
use crate::errors::{AegisError, AegisResult};
use crate::source_metadata::{IngestionStatus, SourceMetadataInput, SourceMetadataPatch, SourceRecord};
use crate::source_registry::SourceRegistry;
use chrono::Utc;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub struct CorpusAuthority {
    paths: CorpusPaths,
}

impl CorpusAuthority {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            paths: CorpusPaths::new(root),
        }
    }

    pub fn register_source(
        &self,
        source_path: impl AsRef<Path>,
        metadata: SourceMetadataInput,
    ) -> AegisResult<SourceRecord> {
        metadata.validate().map_err(AegisError::InvalidMetadata)?;
        self.paths.ensure_layout()?;

        let canonical_path = self.paths.canonicalize_source_path(source_path.as_ref())?;
        let content_hash = calculate_sha256(&canonical_path)?;

        let mut registry = SourceRegistry::load(&self.paths.registry_path())?;
        let record = SourceRecord {
            source_id: format!("src_{}", Uuid::new_v4().simple()),
            version_id: format!("srcv_{}", Uuid::new_v4().simple()),
            title: metadata.title,
            source_type: metadata.source_type,
            discipline: metadata.discipline,
            subdiscipline: metadata.subdiscipline,
            language: metadata.language,
            path: canonical_path,
            content_hash,
            created_at: Utc::now(),
            ingestion_status: IngestionStatus::Registered,
            tags: metadata.tags,
            reliability_notes: metadata.reliability_notes,
        };

        registry.add_source(record.clone())?;
        registry.save(&self.paths.registry_path())?;

        let event = AuditEvent::new(
            AuditEventType::SourceRegistered,
            Some(record.source_id.clone()),
            Some(record.version_id.clone()),
            format!("registered source {}", record.title),
        );
        append_audit_event(&self.paths.audit_events_path(), &event)?;

        Ok(record)
    }

    pub fn get_source(&self, source_id: &str) -> AegisResult<SourceRecord> {
        let registry = SourceRegistry::load(&self.paths.registry_path())?;
        registry.get_source(source_id)
    }

    pub fn list_sources(&self) -> AegisResult<Vec<SourceRecord>> {
        let registry = SourceRegistry::load(&self.paths.registry_path())?;
        Ok(registry.list_sources())
    }

    pub fn update_source_metadata(
        &self,
        source_id: &str,
        patch: SourceMetadataPatch,
    ) -> AegisResult<SourceRecord> {
        self.paths.ensure_layout()?;
        let mut registry = SourceRegistry::load(&self.paths.registry_path())?;
        let updated = registry.update_metadata(source_id, patch)?;
        registry.save(&self.paths.registry_path())?;

        let event = AuditEvent::new(
            AuditEventType::SourceUpdated,
            Some(updated.source_id.clone()),
            Some(updated.version_id.clone()),
            format!("updated source {}", updated.title),
        );
        append_audit_event(&self.paths.audit_events_path(), &event)?;

        Ok(updated)
    }

    pub fn remove_source(&self, source_id: &str) -> AegisResult<SourceRecord> {
        self.paths.ensure_layout()?;
        let mut registry = SourceRegistry::load(&self.paths.registry_path())?;
        let removed = registry.mark_removed(source_id)?;
        registry.save(&self.paths.registry_path())?;

        let event = AuditEvent::new(
            AuditEventType::SourceRemoved,
            Some(removed.source_id.clone()),
            Some(removed.version_id.clone()),
            format!("removed source {}", removed.title),
        );
        append_audit_event(&self.paths.audit_events_path(), &event)?;

        Ok(removed)
    }

    pub fn get_corpus_status(&self) -> AegisResult<crate::source_metadata::CorpusStatus> {
        self.paths.ensure_layout()?;
        let registry = SourceRegistry::load(&self.paths.registry_path())?;
        Ok(registry.status())
    }
}

fn calculate_sha256(path: &Path) -> AegisResult<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];

    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(format!("sha256:{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_metadata::SourceType;
    use std::fs;

    fn valid_metadata() -> SourceMetadataInput {
        SourceMetadataInput {
            title: "Lecture 01".to_string(),
            source_type: SourceType::MarkdownNote,
            discipline: "psychology".to_string(),
            subdiscipline: Some("statistics".to_string()),
            language: "de".to_string(),
            tags: vec!["intro".to_string()],
            reliability_notes: None,
        }
    }

    #[test]
    fn empty_corpus_is_valid() {
        let temp = tempfile::tempdir().unwrap();
        let authority = CorpusAuthority::new(temp.path());
        let status = authority.get_corpus_status().unwrap();
        assert_eq!(status.source_count, 0);
    }

    #[test]
    fn register_source_with_valid_metadata() {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("note.md");
        fs::write(&source_path, "hello").unwrap();

        let authority = CorpusAuthority::new(temp.path());
        let record = authority
            .register_source(&source_path, valid_metadata())
            .unwrap();

        assert!(record.source_id.starts_with("src_"));
        assert!(record.version_id.starts_with("srcv_"));
        assert_eq!(authority.list_sources().unwrap().len(), 1);
    }

    #[test]
    fn missing_title_is_denied() {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("note.md");
        fs::write(&source_path, "hello").unwrap();

        let mut metadata = valid_metadata();
        metadata.title = "".to_string();

        let authority = CorpusAuthority::new(temp.path());
        let result = authority.register_source(&source_path, metadata);
        assert!(result.is_err());
    }

    #[test]
    fn duplicate_content_hash_is_detected() {
        let temp = tempfile::tempdir().unwrap();
        let source_a = temp.path().join("a.md");
        let source_b = temp.path().join("b.md");
        fs::write(&source_a, "same").unwrap();
        fs::write(&source_b, "same").unwrap();

        let authority = CorpusAuthority::new(temp.path());
        authority.register_source(&source_a, valid_metadata()).unwrap();
        let result = authority.register_source(&source_b, valid_metadata());

        assert!(matches!(result, Err(AegisError::DuplicateSource(_))));
    }

    #[test]
    fn update_metadata_preserves_source_id_and_hash() {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("note.md");
        fs::write(&source_path, "hello").unwrap();

        let authority = CorpusAuthority::new(temp.path());
        let record = authority.register_source(&source_path, valid_metadata()).unwrap();
        let updated = authority
            .update_source_metadata(
                &record.source_id,
                SourceMetadataPatch {
                    title: Some("Lecture 01 updated".to_string()),
                    ..Default::default()
                },
            )
            .unwrap();

        assert_eq!(record.source_id, updated.source_id);
        assert_eq!(record.content_hash, updated.content_hash);
        assert_eq!(updated.title, "Lecture 01 updated");
    }

    #[test]
    fn remove_source_marks_removed() {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("note.md");
        fs::write(&source_path, "hello").unwrap();

        let authority = CorpusAuthority::new(temp.path());
        let record = authority.register_source(&source_path, valid_metadata()).unwrap();
        let removed = authority.remove_source(&record.source_id).unwrap();

        assert_eq!(removed.ingestion_status, IngestionStatus::Removed);
    }
}
