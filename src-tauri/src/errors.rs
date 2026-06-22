#[derive(Debug, thiserror::Error)]
pub enum AegisError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Invalid source metadata: {0}")]
    InvalidMetadata(String),

    #[error("Source not found: {0}")]
    SourceNotFound(String),

    #[error("Duplicate source content hash: {0}")]
    DuplicateSource(String),

    #[error("Source path does not exist")]
    SourcePathMissing,

    #[error("Source path must point to a file")]
    SourcePathNotAFile,

    #[error("Source path cannot be inside the corpus workspace")]
    SourcePathInsideCorpus,

    #[error("Registry save failed atomically")]
    AtomicRegistryWriteFailed,
}

pub type AegisResult<T> = Result<T, AegisError>;

pub fn to_user_error(error: AegisError) -> String {
    error.to_string()
}
