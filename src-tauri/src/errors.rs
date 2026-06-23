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

    #[error("Unsupported extraction type: {0}")]
    UnsupportedExtractionType(String),

    #[error("Extraction input missing")]
    ExtractionInputMissing,

    #[error("Extraction input is not valid UTF-8")]
    #[allow(dead_code)]
    ExtractionInputNotUtf8,

    #[error("Extraction report write failed")]
    ExtractionReportWriteFailed,

    #[error("Extraction report missing")]
    ExtractionReportMissing,

    #[error("Extraction report read failed")]
    ExtractionReportReadFailed,

    #[error("Chunking report missing")]
    ChunkingReportMissing,

    #[error("Chunking report read failed")]
    ChunkingReportReadFailed,

    #[error("Chunking report write failed")]
    ChunkingReportWriteFailed,

    #[error("Chunking input missing")]
    ChunkingInputMissing,

    #[error("Chunking input empty")]
    ChunkingInputEmpty,

    #[error("Retrieval index missing")]
    RetrievalIndexMissing,

    #[error("Retrieval index read failed")]
    RetrievalIndexReadFailed,

    #[error("Retrieval index write failed")]
    RetrievalIndexWriteFailed,

    #[error("Retrieval input missing")]
    RetrievalInputMissing,

    #[error("Retrieval query empty")]
    RetrievalQueryEmpty,

    #[error("Retrieval invalid limit")]
    RetrievalInvalidLimit,

    #[error("Retrieval failed")]
    RetrievalIndexFailed,

    #[error("Evidence pack missing")]
    EvidencePackMissing,

    #[error("Evidence pack read failed")]
    EvidencePackReadFailed,

    #[error("Evidence pack write failed")]
    EvidencePackWriteFailed,

    #[error("Evidence pack query empty")]
    EvidencePackQueryEmpty,

    #[error("Evidence pack input missing")]
    EvidencePackInputMissing,

    #[error("Evidence pack invalid id")]
    EvidencePackInvalidId,

    #[error("Evidence pack empty")]
    EvidencePackEmpty,

    #[error("Evidence pack invalid limit")]
    EvidencePackInvalidLimit,

    #[error("Answer draft missing")]
    AnswerDraftMissing,

    #[error("Answer draft read failed")]
    AnswerDraftReadFailed,

    #[error("Answer draft write failed")]
    AnswerDraftWriteFailed,

    #[error("Answer draft invalid id")]
    AnswerDraftInvalidId,

    #[error("Answer draft input missing")]
    AnswerDraftInputMissing,

    #[error("Answer draft empty evidence")]
    AnswerDraftEmptyEvidence,

}

pub type AegisResult<T> = Result<T, AegisError>;

pub fn to_user_error(error: AegisError) -> String {
    error.to_string()
}
