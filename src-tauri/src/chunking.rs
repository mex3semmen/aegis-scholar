use crate::audit::{append_audit_event, AuditEvent, AuditEventType};
use crate::corpus_paths::CorpusPaths;
use crate::errors::{AegisError, AegisResult};
use crate::extraction::{ExtractionReport, ExtractedUnit};
use crate::locators::CitationLocator;
use crate::source_metadata::{IngestionStatus, SourceRecord};
use crate::source_registry::SourceRegistry;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;

const TARGET_CHUNK_CHARS: usize = 1200;
const MAX_CHUNK_CHARS: usize = 1800;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkRecord {
    pub chunk_id: String,
    pub source_id: String,
    pub version_id: String,
    pub locator: CitationLocator,
    pub text: String,
    pub content_hash: String,
    pub extraction_unit_hash: String,
    pub chunk_index: usize,
    pub discipline: Option<String>,
    pub subdiscipline: Option<String>,
    pub method_tags: Vec<String>,
    pub topic_tags: Vec<String>,
    pub extraction_confidence: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkingReport {
    pub source_id: String,
    pub version_id: String,
    pub chunked_at: DateTime<Utc>,
    pub chunk_count: usize,
    pub extraction_report_hash: String,
    pub warnings: Vec<String>,
    pub chunks: Vec<ChunkRecord>,
}

pub struct ChunkingService {
    paths: CorpusPaths,
}

impl ChunkingService {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { paths: CorpusPaths::new(root) }
    }

    pub fn chunk_source(&self, source_id: &str) -> AegisResult<ChunkingReport> {
        self.paths.ensure_layout()?;
        let registry_path = self.paths.registry_path();
        let mut registry = SourceRegistry::load(&registry_path)?;
        let record = registry.get_source(source_id)?;
        let result = (|| -> AegisResult<ChunkingReport> {
            let extraction_report = self.read_extraction_report(&record)?;
            let chunks = chunk_from_report(&extraction_report)?;
            let report = ChunkingReport {
                source_id: record.source_id.clone(),
                version_id: record.version_id.clone(),
                chunked_at: Utc::now(),
                chunk_count: chunks.len(),
                extraction_report_hash: hash_report(&extraction_report)?,
                warnings: Vec::new(),
                chunks,
            };

            self.write_report(&report)?;
            let mut updated = record.clone();
            updated.ingestion_status = IngestionStatus::Chunked;
            registry.replace(updated)?;
            registry.save(&registry_path)?;
            self.append_audit(AuditEventType::SourceChunked, &report.source_id, &report.version_id, "source chunked")?;
            Ok(report)
        })();

        if result.is_err() {
            self.append_audit(AuditEventType::SourceChunkingFailed, &record.source_id, &record.version_id, "source chunking failed")?;
        }

        result
    }

    pub fn read_chunking_report(&self, source_id: &str) -> AegisResult<ChunkingReport> {
        self.paths.ensure_layout()?;
        let registry = SourceRegistry::load(&self.paths.registry_path())?;
        let record = registry.get_source(source_id)?;
        let path = self.report_path(&record.source_id, &record.version_id);
        if !path.exists() {
            return Err(AegisError::ChunkingReportMissing);
        }
        let content = fs::read_to_string(&path).map_err(|_| AegisError::ChunkingReportReadFailed)?;
        serde_json::from_str(&content).map_err(|_| AegisError::ChunkingReportReadFailed)
    }

    fn read_extraction_report(&self, record: &SourceRecord) -> AegisResult<ExtractionReport> {
        let path = self
            .paths
            .source_version_dir(&record.source_id, &record.version_id)
            .join("extraction")
            .join("report.json");
        if !path.exists() {
            return Err(AegisError::ChunkingInputMissing);
        }
        let content = fs::read_to_string(&path).map_err(|_| AegisError::ChunkingReportReadFailed)?;
        serde_json::from_str(&content).map_err(|_| AegisError::ChunkingReportReadFailed)
    }

    fn write_report(&self, report: &ChunkingReport) -> AegisResult<()> {
        let path = self.report_path(&report.source_id, &report.version_id);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|_| AegisError::ChunkingReportWriteFailed)?;
        }
        let body = serde_json::to_string_pretty(report)?;
        fs::write(&path, body).map_err(|_| AegisError::ChunkingReportWriteFailed)?;
        Ok(())
    }

    fn report_path(&self, source_id: &str, version_id: &str) -> PathBuf {
        self.paths
            .source_version_dir(source_id, version_id)
            .join("chunks")
            .join("chunks.json")
    }

    fn append_audit(&self, event_type: AuditEventType, source_id: &str, version_id: &str, summary: &str) -> AegisResult<()> {
        let event = AuditEvent::new(event_type, Some(source_id.to_string()), Some(version_id.to_string()), summary);
        append_audit_event(&self.paths.audit_events_path(), &event)
    }
}

impl SourceRegistry {
    fn replace(&mut self, source: SourceRecord) -> AegisResult<()> {
        self.sources.retain(|item| item.source_id != source.source_id);
        self.sources.push(source);
        Ok(())
    }
}

pub fn chunk_from_report(report: &ExtractionReport) -> AegisResult<Vec<ChunkRecord>> {
    if report.units.is_empty() {
        return Err(AegisError::ChunkingInputEmpty);
    }
    let mut chunks = Vec::new();
    let mut chunk_index = 0usize;
    for unit in &report.units {
        if unit.text.is_empty() {
            continue;
        }
        for piece in split_unit(unit) {
            let text_hash = sha256_text(&piece);
            let chunk_id = deterministic_chunk_id(&report.source_id, &report.version_id, &unit.text_hash, chunk_index, &text_hash);
            chunks.push(ChunkRecord {
                chunk_id,
                source_id: report.source_id.clone(),
                version_id: report.version_id.clone(),
                locator: unit.locator.clone(),
                text: piece,
                content_hash: text_hash.clone(),
                extraction_unit_hash: unit.text_hash.clone(),
                chunk_index,
                discipline: None,
                subdiscipline: None,
                method_tags: Vec::new(),
                topic_tags: Vec::new(),
                extraction_confidence: None,
            });
            chunk_index += 1;
        }
    }
    if chunks.is_empty() {
        return Err(AegisError::ChunkingInputEmpty);
    }
    Ok(chunks)
}

fn split_unit(unit: &ExtractedUnit) -> Vec<String> {
    if unit.text.len() <= MAX_CHUNK_CHARS {
        return vec![unit.text.clone()];
    }
    let mut pieces = Vec::new();
    let mut start = 0usize;
    while start < unit.text.len() {
        let mut end = (start + TARGET_CHUNK_CHARS).min(unit.text.len());
        if end < unit.text.len() {
            if let Some(split) = unit.text[start..end].rfind(char::is_whitespace) {
                let candidate = start + split;
                if candidate > start {
                    end = candidate;
                }
            }
        }
        if end <= start {
            end = next_char_boundary(&unit.text, start + 1);
        }
        let slice = unit.text[start..end].trim().to_string();
        if !slice.is_empty() {
            pieces.push(slice);
        }
        start = end;
        while start < unit.text.len() && unit.text[start..].chars().next().map(|c| c.is_whitespace()).unwrap_or(false) {
            start += unit.text[start..].chars().next().unwrap().len_utf8();
        }
    }
    pieces
}

fn next_char_boundary(text: &str, from: usize) -> usize {
    let mut idx = from.min(text.len());
    while idx < text.len() && !text.is_char_boundary(idx) {
        idx += 1;
    }
    idx.max(from.min(text.len()))
}

fn deterministic_chunk_id(source_id: &str, version_id: &str, extraction_unit_hash: &str, chunk_index: usize, chunk_text_hash: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source_id.as_bytes());
    hasher.update(version_id.as_bytes());
    hasher.update(extraction_unit_hash.as_bytes());
    hasher.update(chunk_index.to_string().as_bytes());
    hasher.update(chunk_text_hash.as_bytes());
    let digest = format!("{:x}", hasher.finalize());
    format!("chk_{}", &digest[..16])
}

fn hash_report(report: &ExtractionReport) -> AegisResult<String> {
    let serialized = serde_json::to_string(report)?;
    Ok(sha256_text(&serialized))
}

fn sha256_text(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("sha256:{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus_authority::CorpusAuthority;
    use crate::extraction::{ExtractedUnit, ExtractionReport};
    use crate::locators::{CitationLocator, LocatorType};
    use crate::source_metadata::{IngestionStatus, SourceMetadataInput, SourceRecord, SourceType};
    use std::fs;

    fn valid_metadata() -> SourceMetadataInput {
        SourceMetadataInput {
            title: "Notes".to_string(),
            source_type: SourceType::MarkdownNote,
            discipline: "psychology".to_string(),
            subdiscipline: Some("statistics".to_string()),
            language: "en".to_string(),
            tags: vec!["study".to_string()],
            reliability_notes: None,
        }
    }

    fn write_extraction(temp: &tempfile::TempDir, text: &str) -> (SourceRecord, ExtractionReport) {
        let source_path = temp.path().join("note.md");
        fs::write(&source_path, text).unwrap();
        let authority = CorpusAuthority::new(temp.path());
        let source = authority.register_source(&source_path, valid_metadata()).unwrap();
        let extraction = crate::extraction::ExtractionService::new(temp.path())
            .extract_source(&source.source_id)
            .unwrap();
        (source, extraction)
    }

    #[test]
    fn chunk_serializes_with_required_fields() {
        let locator = CitationLocator::paragraph("paragraph:1", None, 0, 5);
        let chunk = ChunkRecord {
            chunk_id: "chk_123".to_string(),
            source_id: "src_1".to_string(),
            version_id: "srcv_1".to_string(),
            locator,
            text: "hello".to_string(),
            content_hash: "sha256:abc".to_string(),
            extraction_unit_hash: "sha256:def".to_string(),
            chunk_index: 0,
            discipline: None,
            subdiscipline: None,
            method_tags: vec![],
            topic_tags: vec![],
            extraction_confidence: None,
        };
        let value = serde_json::to_value(chunk).unwrap();
        assert_eq!(value["chunk_id"], "chk_123");
        assert_eq!(value["content_hash"], "sha256:abc");
    }

    #[test]
    fn chunk_ids_are_deterministic() {
        let report = ExtractionReport {
            source_id: "src_1".into(),
            version_id: "srcv_1".into(),
            source_type: SourceType::MarkdownNote,
            extracted_at: Utc::now(),
            unit_count: 1,
            warnings: vec![],
            units: vec![ExtractedUnit {
                source_id: "src_1".into(),
                version_id: "srcv_1".into(),
                locator: CitationLocator::paragraph("paragraph:1", None, 0, 5),
                text: "hello".into(),
                text_hash: "sha256:111".into(),
                char_start: 0,
                char_end: 5,
            }],
        };
        let a = chunk_from_report(&report).unwrap();
        let b = chunk_from_report(&report).unwrap();
        assert_eq!(a[0].chunk_id, b[0].chunk_id);
        assert_eq!(a[0].content_hash, b[0].content_hash);
    }

    #[test]
    fn chunk_report_is_serializable_json() {
        let report = ChunkingReport {
            source_id: "src_1".into(),
            version_id: "srcv_1".into(),
            chunked_at: Utc::now(),
            chunk_count: 0,
            extraction_report_hash: "sha256:abc".into(),
            warnings: vec![],
            chunks: vec![],
        };
        serde_json::to_string(&report).unwrap();
    }

    #[test]
    fn source_with_extraction_report_can_be_chunked() {
        let temp = tempfile::tempdir().unwrap();
        let (source, _extraction) = write_extraction(&temp, "# A\n\nfirst paragraph here\n");
        let report = ChunkingService::new(temp.path()).chunk_source(&source.source_id).unwrap();
        assert_eq!(report.source_id, source.source_id);
        assert_eq!(report.version_id, source.version_id);
        assert_eq!(report.chunk_count, 1);
        let report_path = ChunkingService::new(temp.path()).report_path(&source.source_id, &source.version_id);
        assert!(report_path.exists());
        let reread = ChunkingService::new(temp.path()).read_chunking_report(&source.source_id).unwrap();
        assert_eq!(reread.chunk_count, 1);
        let updated = CorpusAuthority::new(temp.path()).get_source(&source.source_id).unwrap();
        assert_eq!(updated.ingestion_status, IngestionStatus::Chunked);
        let audit = fs::read_to_string(temp.path().join(".aegis").join("audit").join("events.jsonl")).unwrap();
        assert!(audit.contains("source_chunked"));
    }

    #[test]
    fn read_back_returns_same_chunk_report() {
        let temp = tempfile::tempdir().unwrap();
        let (source, _extraction) = write_extraction(&temp, "alpha\n\nbeta\n");
        let report = ChunkingService::new(temp.path()).chunk_source(&source.source_id).unwrap();
        let reread = ChunkingService::new(temp.path()).read_chunking_report(&source.source_id).unwrap();
        assert_eq!(report.source_id, reread.source_id);
        assert_eq!(report.version_id, reread.version_id);
        assert_eq!(report.chunk_count, reread.chunk_count);
        assert_eq!(report.chunks.len(), reread.chunks.len());
    }

    #[test]
    fn missing_chunk_report_returns_typed_error() {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("note.md");
        fs::write(&source_path, "hello").unwrap();
        let authority = CorpusAuthority::new(temp.path());
        let source = authority.register_source(&source_path, valid_metadata()).unwrap();
        let result = ChunkingService::new(temp.path()).read_chunking_report(&source.source_id);
        assert!(matches!(result, Err(AegisError::ChunkingReportMissing)));
    }

    #[test]
    fn malformed_chunk_report_returns_typed_error() {
        let temp = tempfile::tempdir().unwrap();
        let (source, _extraction) = write_extraction(&temp, "alpha\n");
        let path = ChunkingService::new(temp.path()).report_path(&source.source_id, &source.version_id);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "{not json").unwrap();
        let result = ChunkingService::new(temp.path()).read_chunking_report(&source.source_id);
        assert!(matches!(result, Err(AegisError::ChunkingReportReadFailed)));
    }

    #[test]
    fn long_extraction_unit_is_split_deterministically() {
        let long = "a".repeat(2001);
        let report = ExtractionReport {
            source_id: "src_1".into(),
            version_id: "srcv_1".into(),
            source_type: SourceType::MarkdownNote,
            extracted_at: Utc::now(),
            unit_count: 1,
            warnings: vec![],
            units: vec![ExtractedUnit {
                source_id: "src_1".into(),
                version_id: "srcv_1".into(),
                locator: CitationLocator::paragraph("paragraph:1", None, 0, long.len()),
                text: long.clone(),
                text_hash: sha256_text(&long),
                char_start: 0,
                char_end: long.len(),
            }],
        };
        let chunks = chunk_from_report(&report).unwrap();
        assert!(chunks.len() >= 2);
        assert!(chunks.iter().all(|c| !c.text.is_empty()));
        assert_eq!(chunks[0].locator.locator_type, LocatorType::Paragraph);
    }

    #[test]
    fn repeated_chunking_produces_identical_ids_and_hashes() {
        let report = ExtractionReport {
            source_id: "src_1".into(),
            version_id: "srcv_1".into(),
            source_type: SourceType::MarkdownNote,
            extracted_at: Utc::now(),
            unit_count: 2,
            warnings: vec![],
            units: vec![
                ExtractedUnit {
                    source_id: "src_1".into(),
                    version_id: "srcv_1".into(),
                    locator: CitationLocator::paragraph("paragraph:1", None, 0, 5),
                    text: "alpha".into(),
                    text_hash: sha256_text("alpha"),
                    char_start: 0,
                    char_end: 5,
                },
                ExtractedUnit {
                    source_id: "src_1".into(),
                    version_id: "srcv_1".into(),
                    locator: CitationLocator::paragraph("paragraph:2", None, 6, 11),
                    text: "bravo".into(),
                    text_hash: sha256_text("bravo"),
                    char_start: 6,
                    char_end: 11,
                },
            ],
        };
        let a = chunk_from_report(&report).unwrap();
        let b = chunk_from_report(&report).unwrap();
        assert_eq!(a.iter().map(|c| &c.chunk_id).collect::<Vec<_>>(), b.iter().map(|c| &c.chunk_id).collect::<Vec<_>>());
        assert_eq!(a.iter().map(|c| &c.content_hash).collect::<Vec<_>>(), b.iter().map(|c| &c.content_hash).collect::<Vec<_>>());
    }

    #[test]
    fn chunking_never_merges_units_with_different_locators() {
        let report = ExtractionReport {
            source_id: "src_1".into(),
            version_id: "srcv_1".into(),
            source_type: SourceType::MarkdownNote,
            extracted_at: Utc::now(),
            unit_count: 2,
            warnings: vec![],
            units: vec![
                ExtractedUnit {
                    source_id: "src_1".into(),
                    version_id: "srcv_1".into(),
                    locator: CitationLocator::paragraph("paragraph:1", Some(vec!["A".into()]), 0, 5),
                    text: "alpha".into(),
                    text_hash: sha256_text("alpha"),
                    char_start: 0,
                    char_end: 5,
                },
                ExtractedUnit {
                    source_id: "src_1".into(),
                    version_id: "srcv_1".into(),
                    locator: CitationLocator::paragraph("paragraph:2", Some(vec!["B".into()]), 6, 11),
                    text: "bravo".into(),
                    text_hash: sha256_text("bravo"),
                    char_start: 6,
                    char_end: 11,
                },
            ],
        };
        let chunks = chunk_from_report(&report).unwrap();
        assert_eq!(chunks.len(), 2);
        assert_ne!(chunks[0].locator.section_path, chunks[1].locator.section_path);
    }

    #[test]
    fn missing_extraction_report_returns_typed_error_and_writes_failure_audit() {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("note.md");
        fs::write(&source_path, "hello").unwrap();
        let authority = CorpusAuthority::new(temp.path());
        let source = authority.register_source(&source_path, valid_metadata()).unwrap();
        let result = ChunkingService::new(temp.path()).chunk_source(&source.source_id);
        assert!(matches!(result, Err(AegisError::ChunkingInputMissing)));
        let updated = CorpusAuthority::new(temp.path()).get_source(&source.source_id).unwrap();
        assert_ne!(updated.ingestion_status, IngestionStatus::Chunked);
        let audit = fs::read_to_string(temp.path().join(".aegis").join("audit").join("events.jsonl")).unwrap();
        assert!(audit.contains("source_chunking_failed"));
    }

    #[test]
    fn empty_extraction_report_returns_typed_error() {
        let report = ExtractionReport {
            source_id: "src_1".into(),
            version_id: "srcv_1".into(),
            source_type: SourceType::MarkdownNote,
            extracted_at: Utc::now(),
            unit_count: 0,
            warnings: vec![],
            units: vec![],
        };
        assert!(matches!(chunk_from_report(&report), Err(AegisError::ChunkingInputEmpty)));
    }
}
