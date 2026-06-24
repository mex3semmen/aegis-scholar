use crate::audit::{append_audit_event, AuditEvent, AuditEventType};
use crate::corpus_paths::CorpusPaths;
use crate::errors::{AegisError, AegisResult};
use crate::locators::CitationLocator;
use crate::source_metadata::IngestionStatus;
use crate::source_registry::SourceRegistry;
use crate::chunking::ChunkingService;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

const INDEX_VERSION: &str = "retrieval-index-v1";
const PREVIEW_LIMIT: usize = 240;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalIndex {
    pub source_id: String,
    pub version_id: String,
    pub indexed_at: DateTime<Utc>,
    pub chunk_count: usize,
    pub index_version: String,
    pub chunk_report_hash: String,
    pub entries: Vec<RetrievalIndexEntry>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalIndexEntry {
    pub chunk_id: String,
    pub source_id: String,
    pub version_id: String,
    pub locator: CitationLocator,
    pub text_hash: String,
    pub normalized_terms: Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalQuery {
    pub query: String,
    pub max_results: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalResult {
    pub chunk_id: String,
    pub source_id: String,
    pub version_id: String,
    pub locator: CitationLocator,
    pub score: f32,
    pub matched_terms: Vec<String>,
    pub text_hash: String,
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalResponse {
    pub query: String,
    pub normalized_query_terms: Vec<String>,
    pub result_count: usize,
    pub results: Vec<RetrievalResult>,
}

pub struct RetrievalService {
    paths: CorpusPaths,
}

impl RetrievalService {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { paths: CorpusPaths::new(root) }
    }

    pub fn build_index(&self, source_id: &str) -> AegisResult<RetrievalIndex> {
        self.paths.ensure_layout()?;
        let registry_path = self.paths.registry_path();
        let mut registry = SourceRegistry::load(&registry_path)?;
        let record = registry.get_source(source_id)?;
        let result = (|| -> AegisResult<RetrievalIndex> {
            let chunking_report = ChunkingService::new(self.paths.root.clone()).read_chunking_report(source_id)?;
            if chunking_report.chunks.is_empty() || count_chunking_report_units(&chunking_report) == 0 {
                return Err(AegisError::RetrievalInputMissing);
            }
            if chunking_report.chunk_count != chunking_report.chunks.len() {
                return Err(AegisError::RetrievalIndexFailed);
            }

            let entries = chunking_report
                .chunks
                .iter()
                .map(|chunk| RetrievalIndexEntry {
                    chunk_id: chunk.chunk_id.clone(),
                    source_id: chunk.source_id.clone(),
                    version_id: chunk.version_id.clone(),
                    locator: chunk.locator.clone(),
                    text_hash: chunk.content_hash.clone(),
                    normalized_terms: normalize_terms(&chunk.text),
                })
                .collect::<Vec<_>>();

            let index = RetrievalIndex {
                source_id: record.source_id.clone(),
                version_id: record.version_id.clone(),
                indexed_at: Utc::now(),
                chunk_count: entries.len(),
                index_version: INDEX_VERSION.to_string(),
                chunk_report_hash: hash_report(&chunking_report)?,
                entries,
                warnings: Vec::new(),
            };

            self.write_index(&index)?;
            let mut updated = record.clone();
            updated.ingestion_status = IngestionStatus::Indexed;
            registry.replace(updated)?;
            registry.save(&registry_path)?;
            self.append_audit(AuditEventType::SourceIndexed, &index.source_id, &index.version_id, "source indexed")?;
            Ok(index)
        })();

        if result.is_err() {
            self.append_audit(AuditEventType::SourceIndexingFailed, &record.source_id, &record.version_id, "source indexing failed")?;
        }

        result
    }

    pub fn read_index(&self, source_id: &str) -> AegisResult<RetrievalIndex> {
        self.paths.ensure_layout()?;
        self.read_existing_index(source_id)
    }

    pub fn search_source(&self, source_id: &str, query: &str, max_results: usize) -> AegisResult<RetrievalResponse> {
        self.paths.ensure_layout()?;
        if max_results == 0 {
            return Err(AegisError::RetrievalInvalidLimit);
        }
        let query_terms = normalize_terms(query);
        if query.trim().is_empty() || query_terms.is_empty() {
            return Err(AegisError::RetrievalQueryEmpty);
        }
        let registry = SourceRegistry::load(&self.paths.registry_path())?;
        let record = registry.get_source(source_id)?;
        let index = match self.read_existing_index(source_id) {
            Ok(index) => index,
            Err(AegisError::RetrievalIndexMissing) => self.build_index(source_id)?,
            Err(error) => return Err(error),
        };
        let chunk_report = self.read_existing_chunking_report(&record.source_id, &record.version_id)?;
        let results = self.search_with_index(index, chunk_report, &query_terms, max_results)?;

        let _ = record;
        Ok(RetrievalResponse {
            query: query.to_string(),
            normalized_query_terms: query_terms,
            result_count: results.len(),
            results,
        })
    }

    pub fn preview_search_source(&self, source_id: &str, query: &str, max_results: usize) -> AegisResult<RetrievalResponse> {
        if max_results == 0 {
            return Err(AegisError::RetrievalInvalidLimit);
        }
        let query_terms = normalize_terms(query);
        if query.trim().is_empty() || query_terms.is_empty() {
            return Err(AegisError::RetrievalQueryEmpty);
        }
        let registry = SourceRegistry::load(&self.paths.registry_path())?;
        let record = registry.get_source(source_id)?;
        let index = self.read_existing_index(source_id)?;
        let chunk_report = self.read_existing_chunking_report(&record.source_id, &record.version_id)?;
        let results = self.search_with_index(index, chunk_report, &query_terms, max_results)?;
        Ok(RetrievalResponse {
            query: query.to_string(),
            normalized_query_terms: query_terms,
            result_count: results.len(),
            results,
        })
    }

    fn search_with_index(
        &self,
        index: RetrievalIndex,
        chunk_report: crate::chunking::ChunkingReport,
        query_terms: &[String],
        max_results: usize,
    ) -> AegisResult<Vec<RetrievalResult>> {
        let chunk_map = chunk_report
            .chunks
            .iter()
            .map(|chunk| (chunk.chunk_id.clone(), chunk.text.clone()))
            .collect::<BTreeMap<_, _>>();
        let query_set: BTreeSet<_> = query_terms.iter().cloned().collect();
        let mut results = index
            .entries
            .iter()
            .filter_map(|entry| {
                let matched: Vec<String> = entry
                    .normalized_terms
                    .iter()
                    .filter(|term| query_set.contains(*term))
                    .cloned()
                    .collect();
                if matched.is_empty() {
                    return None;
                }
                let score = matched.len() as f32 / query_terms.len() as f32;
                let preview = chunk_map
                    .get(&entry.chunk_id)
                    .map(|text| preview_text(text, PREVIEW_LIMIT))
                    .unwrap_or_default();
                Some(RetrievalResult {
                    chunk_id: entry.chunk_id.clone(),
                    source_id: entry.source_id.clone(),
                    version_id: entry.version_id.clone(),
                    locator: entry.locator.clone(),
                    score,
                    matched_terms: matched,
                    text_hash: entry.text_hash.clone(),
                    preview,
                })
            })
            .collect::<Vec<_>>();

        results.sort_by(|a, b| match b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal) {
            Ordering::Equal => a.chunk_id.cmp(&b.chunk_id),
            other => other,
        });
        results.truncate(max_results);
        Ok(results)
    }

    fn read_existing_index(&self, source_id: &str) -> AegisResult<RetrievalIndex> {
        let registry = SourceRegistry::load(&self.paths.registry_path())?;
        let record = registry.get_source(source_id)?;
        let path = self.index_path(&record.source_id, &record.version_id);
        if !path.exists() {
            return Err(AegisError::RetrievalIndexMissing);
        }
        let content = fs::read_to_string(&path).map_err(|_| AegisError::RetrievalIndexReadFailed)?;
        serde_json::from_str(&content).map_err(|_| AegisError::RetrievalIndexReadFailed)
    }

    fn read_existing_chunking_report(&self, source_id: &str, version_id: &str) -> AegisResult<crate::chunking::ChunkingReport> {
        let path = self
            .paths
            .source_version_dir(source_id, version_id)
            .join("chunks")
            .join("chunks.json");
        if !path.exists() {
            return Err(AegisError::ChunkingReportMissing);
        }
        let content = fs::read_to_string(&path).map_err(|_| AegisError::ChunkingReportReadFailed)?;
        serde_json::from_str(&content).map_err(|_| AegisError::ChunkingReportReadFailed)
    }

    fn write_index(&self, index: &RetrievalIndex) -> AegisResult<()> {
        let path = self.index_path(&index.source_id, &index.version_id);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|_| AegisError::RetrievalIndexWriteFailed)?;
        }
        let body = serde_json::to_string_pretty(index)?;
        fs::write(&path, body).map_err(|_| AegisError::RetrievalIndexWriteFailed)?;
        Ok(())
    }

    fn index_path(&self, source_id: &str, version_id: &str) -> PathBuf {
        self.paths
            .source_version_dir(source_id, version_id)
            .join("retrieval")
            .join("index.json")
    }

    fn append_audit(&self, event_type: AuditEventType, source_id: &str, version_id: &str, summary: &str) -> AegisResult<()> {
        let event = AuditEvent::new(event_type, Some(source_id.to_string()), Some(version_id.to_string()), summary);
        append_audit_event(&self.paths.audit_events_path(), &event)
    }
}

pub fn normalize_terms(text: &str) -> Vec<String> {
    let mut terms = text
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|term| term.chars().count() > 1)
        .map(|term| term.to_string())
        .collect::<Vec<_>>();
    terms.sort();
    terms.dedup();
    terms
}

fn hash_report<T: Serialize>(report: &T) -> AegisResult<String> {
    let serialized = serde_json::to_string(report)?;
    Ok(sha256_text(&serialized))
}

fn sha256_text(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("sha256:{:x}", hasher.finalize())
}

fn preview_text(text: &str, limit: usize) -> String {
    if text.len() <= limit {
        return text.to_string();
    }
    let mut end = limit;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    text[..end].to_string()
}

fn count_chunking_report_units(report: &crate::chunking::ChunkingReport) -> usize {
    report
        .chunks
        .iter()
        .map(|chunk| chunk.chunk_index)
        .max()
        .map(|max| max + 1)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::corpus_authority::CorpusAuthority;
    use crate::chunking::ChunkingService;
    use crate::extraction::ExtractionService;
    use crate::source_metadata::{SourceMetadataInput, SourceType};
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

    fn build_source(temp: &tempfile::TempDir, text: &str) -> String {
        let source_path = temp.path().join("note.md");
        fs::write(&source_path, text).unwrap();
        let authority = CorpusAuthority::new(temp.path());
        let source = authority.register_source(&source_path, valid_metadata()).unwrap();
        ExtractionService::new(temp.path()).extract_source(&source.source_id).unwrap();
        ChunkingService::new(temp.path()).chunk_source(&source.source_id).unwrap();
        source.source_id
    }

    #[test]
    fn lowercase_normalization_works() {
        assert_eq!(normalize_terms("Alpha Beta"), vec!["alpha", "beta"]);
    }

    #[test]
    fn punctuation_splitting_works() {
        assert_eq!(normalize_terms("alpha, beta; gamma!"), vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn unicode_normalization_is_deterministic() {
        let first = normalize_terms("Grüß Gott");
        let second = normalize_terms("Grüß Gott");
        assert_eq!(first, second);
    }

    #[test]
    fn empty_terms_are_dropped() {
        assert_eq!(normalize_terms("a ! b"), Vec::<String>::new());
    }

    #[test]
    fn repeated_terms_are_deduplicated_deterministically() {
        assert_eq!(normalize_terms("beta beta alpha alpha"), vec!["alpha", "beta"]);
    }

    #[test]
    fn normalization_is_deterministic_for_crlf_text() {
        let first = normalize_terms("Alpha\r\nBeta\r\n");
        let second = normalize_terms("Alpha\r\nBeta\r\n");
        assert_eq!(first, second);
    }

    #[test]
    fn normalization_handles_unicode_without_panicking() {
        let terms = normalize_terms("Grüß, мир, 世界");
        assert!(terms.contains(&"grüß".to_string()));
        assert!(terms.contains(&"мир".to_string()));
        assert!(terms.contains(&"世界".to_string()));
    }

    #[test]
    fn build_and_read_index_round_trips() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source(&temp, "alpha beta\n\nbravo charlie\n");
        let service = RetrievalService::new(temp.path());
        let index = service.build_index(&source_id).unwrap();
        assert_eq!(index.source_id, source_id);
        let reread = service.read_index(&source_id).unwrap();
        assert_eq!(index.entries.len(), reread.entries.len());
        assert_eq!(index.version_id, reread.version_id);
        assert_eq!(index.chunk_count, reread.chunk_count);
    }

    #[test]
    fn read_index_uses_current_source_version_from_registry() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source(&temp, "alpha beta\n");
        let service = RetrievalService::new(temp.path());
        let index = service.build_index(&source_id).unwrap();
        let record = CorpusAuthority::new(temp.path()).get_source(&source_id).unwrap();
        assert_eq!(index.version_id, record.version_id);
    }

    #[test]
    fn read_index_uses_managed_corpus_path_only() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source(&temp, "alpha beta\n");
        let service = RetrievalService::new(temp.path());
        let index = service.build_index(&source_id).unwrap();
        let path = temp
            .path()
            .join(".aegis")
            .join("corpus")
            .join("sources")
            .join(&source_id)
            .join("versions")
            .join(&index.version_id)
            .join("retrieval")
            .join("index.json");
        assert!(path.exists());
        let reread = service.read_index(&source_id).unwrap();
        assert_eq!(reread.source_id, source_id);
    }

    #[test]
    fn search_returns_matches_and_respects_max_results() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source(&temp, "alpha beta\n\nbeta gamma\n");
        let service = RetrievalService::new(temp.path());
        let response = service.search_source(&source_id, "beta", 1).unwrap();
        assert_eq!(response.result_count, 1);
        assert_eq!(response.results.len(), 1);
        assert_eq!(response.results[0].source_id, source_id);
        assert!(!response.results[0].preview.is_empty());
        assert_eq!(response.results[0].matched_terms, vec!["beta".to_string()]);
    }

    #[test]
    fn search_returns_empty_results_for_no_match() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source(&temp, "alpha beta\n\nbravo gamma\n");
        let service = RetrievalService::new(temp.path());
        let response = service.search_source(&source_id, "delta", 10).unwrap();
        assert_eq!(response.result_count, 0);
        assert!(response.results.is_empty());
    }

    #[test]
    fn repeated_search_returns_same_ordering() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source(&temp, "alpha beta\n\nbeta alpha\n");
        let service = RetrievalService::new(temp.path());
        let first = service.search_source(&source_id, "alpha beta", 10).unwrap();
        let second = service.search_source(&source_id, "alpha beta", 10).unwrap();
        assert_eq!(first.results.iter().map(|r| &r.chunk_id).collect::<Vec<_>>(), second.results.iter().map(|r| &r.chunk_id).collect::<Vec<_>>());
    }

    #[test]
    fn score_ordering_is_deterministic() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source(&temp, "alpha beta\n\nalpha gamma\n");
        let service = RetrievalService::new(temp.path());
        let response = service.search_source(&source_id, "alpha", 10).unwrap();
        assert!(response.results.iter().all(|r| (0.0..=1.0).contains(&r.score)));
        assert!(response.results.windows(2).all(|pair| pair[0].score >= pair[1].score));
    }

    #[test]
    fn score_ties_are_sorted_by_chunk_id() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source(&temp, "alpha\n\nalpha\n");
        let service = RetrievalService::new(temp.path());
        let response = service.search_source(&source_id, "alpha", 10).unwrap();
        assert!(response.results.len() >= 2);
        assert!(response.results[0].score == response.results[1].score);
        assert!(response.results[0].chunk_id < response.results[1].chunk_id);
    }

    #[test]
    fn duplicate_query_terms_do_not_inflate_score() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source(&temp, "alpha beta\n");
        let service = RetrievalService::new(temp.path());
        let deduped = service.search_source(&source_id, "alpha beta", 10).unwrap();
        let repeated = service.search_source(&source_id, "alpha alpha beta beta", 10).unwrap();
        assert_eq!(deduped.results[0].score, repeated.results[0].score);
    }

    #[test]
    fn max_results_zero_returns_typed_error() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source(&temp, "alpha beta\n");
        let service = RetrievalService::new(temp.path());
        let result = service.search_source(&source_id, "alpha", 0);
        assert!(matches!(result, Err(AegisError::RetrievalInvalidLimit)));
    }

    #[test]
    fn empty_query_returns_typed_error() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source(&temp, "alpha beta\n");
        let service = RetrievalService::new(temp.path());
        let result = service.search_source(&source_id, "", 10);
        assert!(matches!(result, Err(AegisError::RetrievalQueryEmpty)));
    }

    #[test]
    fn one_character_query_returns_typed_error() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source(&temp, "alpha beta\n");
        let service = RetrievalService::new(temp.path());
        let result = service.search_source(&source_id, "a b c", 10);
        assert!(matches!(result, Err(AegisError::RetrievalQueryEmpty)));
    }

    #[test]
    fn search_uses_preview_from_chunk_text() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source(&temp, "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu\n");
        let service = RetrievalService::new(temp.path());
        let response = service.search_source(&source_id, "alpha", 10).unwrap();
        assert!(response.results[0].preview.contains("alpha"));
    }

    #[test]
    fn long_preview_is_truncated_deterministically() {
        let text = format!("{} alpha", "a".repeat(500));
        let preview = preview_text(&text, 240);
        assert!(preview.len() <= 240);
        assert!(text.starts_with(&preview));
    }

    #[test]
    fn build_index_marks_source_indexed_and_writes_audit() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source(&temp, "alpha beta\n\ncharlie delta\n");
        let index = RetrievalService::new(temp.path()).build_index(&source_id).unwrap();
        assert_eq!(index.index_version, INDEX_VERSION);
        let updated = CorpusAuthority::new(temp.path()).get_source(&source_id).unwrap();
        assert_eq!(updated.ingestion_status, IngestionStatus::Indexed);
        let audit = fs::read_to_string(temp.path().join(".aegis").join("audit").join("events.jsonl")).unwrap();
        assert!(audit.contains("source_indexed"));
    }

    #[test]
    fn missing_index_returns_typed_error() {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("note.md");
        fs::write(&source_path, "alpha").unwrap();
        let source = CorpusAuthority::new(temp.path()).register_source(&source_path, valid_metadata()).unwrap();
        let result = RetrievalService::new(temp.path()).read_index(&source.source_id);
        assert!(matches!(result, Err(AegisError::RetrievalIndexMissing)));
    }

    #[test]
    fn preview_search_does_not_build_index_on_missing_index() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source(&temp, "alpha beta\n");
        let service = RetrievalService::new(temp.path());
        let version_id = CorpusAuthority::new(temp.path()).get_source(&source_id).unwrap().version_id;
        let index_path = temp
            .path()
            .join(".aegis")
            .join("corpus")
            .join("sources")
            .join(&source_id)
            .join("versions")
            .join(&version_id)
            .join("retrieval")
            .join("index.json");
        assert!(!index_path.exists());
        let result = service.preview_search_source(&source_id, "alpha", 10);
        assert!(matches!(result, Err(AegisError::RetrievalIndexMissing)));
        assert!(!index_path.exists());
    }

    #[test]
    fn malformed_index_returns_typed_error() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source(&temp, "alpha beta\n");
        let path = RetrievalService::new(temp.path())
            .index_path(&source_id, "srcv_0");
        // Intentionally point at the actual managed path.
        let actual_path = temp
            .path()
            .join(".aegis")
            .join("corpus")
            .join("sources")
            .join(&source_id)
            .join("versions")
            .join(CorpusAuthority::new(temp.path()).get_source(&source_id).unwrap().version_id)
            .join("retrieval")
            .join("index.json");
        fs::create_dir_all(actual_path.parent().unwrap()).unwrap();
        fs::write(&actual_path, "{not json").unwrap();
        let result = RetrievalService::new(temp.path()).read_index(&source_id);
        assert!(matches!(result, Err(AegisError::RetrievalIndexReadFailed)));
        let _ = path;
    }

    #[test]
    fn missing_chunking_report_returns_typed_error() {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("note.md");
        fs::write(&source_path, "alpha").unwrap();
        let source = CorpusAuthority::new(temp.path()).register_source(&source_path, valid_metadata()).unwrap();
        let result = RetrievalService::new(temp.path()).build_index(&source.source_id);
        assert!(matches!(result, Err(AegisError::ChunkingReportMissing)));
    }

    #[test]
    fn empty_chunking_report_returns_typed_error() {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("note.md");
        fs::write(&source_path, "alpha").unwrap();
        let authority = CorpusAuthority::new(temp.path());
        let source = authority.register_source(&source_path, valid_metadata()).unwrap();
        ExtractionService::new(temp.path()).extract_source(&source.source_id).unwrap();
        let path = temp
            .path()
            .join(".aegis")
            .join("corpus")
            .join("sources")
            .join(&source.source_id)
            .join("versions")
            .join(&source.version_id)
            .join("chunks")
            .join("chunks.json");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let empty_report = crate::chunking::ChunkingReport {
            source_id: source.source_id.clone(),
            version_id: source.version_id.clone(),
            chunked_at: Utc::now(),
            chunk_count: 0,
            extraction_report_hash: "sha256:abc".into(),
            warnings: vec![],
            chunks: vec![],
        };
        fs::write(&path, serde_json::to_string_pretty(&empty_report).unwrap()).unwrap();
        let result = RetrievalService::new(temp.path()).build_index(&source.source_id);
        assert!(matches!(result, Err(AegisError::RetrievalInputMissing)));
    }

    #[test]
    fn search_builds_index_on_demand() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source(&temp, "alpha beta\n");
        let index_path = temp
            .path()
            .join(".aegis")
            .join("corpus")
            .join("sources")
            .join(&source_id)
            .join("versions")
            .join(CorpusAuthority::new(temp.path()).get_source(&source_id).unwrap().version_id)
            .join("retrieval")
            .join("index.json");
        assert!(!index_path.exists());
        let response = RetrievalService::new(temp.path()).search_source(&source_id, "alpha", 10).unwrap();
        assert_eq!(response.result_count, 1);
        assert!(index_path.exists());
    }

    #[test]
    fn search_does_not_require_embeddings_or_generate_answers() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source(&temp, "alpha beta\n");
        let response = RetrievalService::new(temp.path()).search_source(&source_id, "alpha", 10).unwrap();
        assert_eq!(response.results[0].preview, "alpha beta");
    }

    #[test]
    fn search_does_not_search_across_sources() {
        let temp = tempfile::tempdir().unwrap();
        let source_a = build_source(&temp, "alpha beta\n");
        let source_b_path = temp.path().join("other.md");
        fs::write(&source_b_path, "alpha beta gamma\n").unwrap();
        let authority = CorpusAuthority::new(temp.path());
        let source_b = authority.register_source(&source_b_path, valid_metadata()).unwrap();
        ExtractionService::new(temp.path()).extract_source(&source_b.source_id).unwrap();
        ChunkingService::new(temp.path()).chunk_source(&source_b.source_id).unwrap();
        let service = RetrievalService::new(temp.path());
        let response = service.search_source(&source_a, "alpha", 10).unwrap();
        assert!(response.results.iter().all(|r| r.source_id == source_a));
    }

    #[test]
    fn chunk_count_mismatch_is_detected() {
        let temp = tempfile::tempdir().unwrap();
        let source_id = build_source(&temp, "alpha beta\n");
        let source = CorpusAuthority::new(temp.path()).get_source(&source_id).unwrap();
        let path = temp
            .path()
            .join(".aegis")
            .join("corpus")
            .join("sources")
            .join(&source.source_id)
            .join("versions")
            .join(&source.version_id)
            .join("chunks")
            .join("chunks.json");
        let mut report = ChunkingService::new(temp.path()).read_chunking_report(&source.source_id).unwrap();
        report.chunk_count = report.chunk_count + 1;
        fs::write(&path, serde_json::to_string_pretty(&report).unwrap()).unwrap();
        let result = RetrievalService::new(temp.path()).build_index(&source.source_id);
        assert!(matches!(result, Err(AegisError::RetrievalIndexFailed)));
        let updated = CorpusAuthority::new(temp.path()).get_source(&source.source_id).unwrap();
        assert_ne!(updated.ingestion_status, IngestionStatus::Indexed);
    }

    #[test]
    fn failed_index_build_does_not_write_success_index() {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("note.md");
        fs::write(&source_path, "alpha").unwrap();
        let source = CorpusAuthority::new(temp.path()).register_source(&source_path, valid_metadata()).unwrap();
        let result = RetrievalService::new(temp.path()).build_index(&source.source_id);
        assert!(matches!(result, Err(AegisError::ChunkingReportMissing)));
        let path = temp
            .path()
            .join(".aegis")
            .join("corpus")
            .join("sources")
            .join(&source.source_id)
            .join("versions")
            .join(&source.version_id)
            .join("retrieval")
            .join("index.json");
        assert!(!path.exists());
    }
}
