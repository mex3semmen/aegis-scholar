use crate::errors::{AegisError, AegisResult};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct CorpusPaths {
    pub root: PathBuf,
}

impl CorpusPaths {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn aegis_dir(&self) -> PathBuf {
        self.root.join(".aegis")
    }

    pub fn corpus_dir(&self) -> PathBuf {
        self.aegis_dir().join("corpus")
    }

    pub fn sources_dir(&self) -> PathBuf {
        self.corpus_dir().join("sources")
    }

    pub fn registry_path(&self) -> PathBuf {
        self.corpus_dir().join("registry.json")
    }

    pub fn audit_dir(&self) -> PathBuf {
        self.aegis_dir().join("audit")
    }

    pub fn audit_events_path(&self) -> PathBuf {
        self.audit_dir().join("events.jsonl")
    }

    pub fn ensure_layout(&self) -> AegisResult<()> {
        fs::create_dir_all(self.sources_dir())?;
        fs::create_dir_all(self.audit_dir())?;
        Ok(())
    }

    pub fn canonicalize_source_path(&self, source_path: &Path) -> AegisResult<PathBuf> {
        if !source_path.exists() {
            return Err(AegisError::SourcePathMissing);
        }
        let canonical = source_path.canonicalize()?;
        Ok(canonical)
    }
}
