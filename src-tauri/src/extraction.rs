use crate::audit::{append_audit_event, AuditEvent, AuditEventType};
use crate::corpus_paths::CorpusPaths;
use crate::errors::{AegisError, AegisResult};
use crate::locators::CitationLocator;
use crate::source_metadata::{IngestionStatus, SourceRecord, SourceType};
use crate::source_registry::SourceRegistry;
use chrono::{DateTime, Utc};
use lopdf::Document;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedUnit {
    pub source_id: String,
    pub version_id: String,
    pub locator: CitationLocator,
    pub text: String,
    pub text_hash: String,
    pub char_start: usize,
    pub char_end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionReport {
    pub source_id: String,
    pub version_id: String,
    pub source_type: SourceType,
    pub extracted_at: DateTime<Utc>,
    pub unit_count: usize,
    pub warnings: Vec<String>,
    pub units: Vec<ExtractedUnit>,
}

pub struct ExtractionService {
    paths: CorpusPaths,
}

impl ExtractionService {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { paths: CorpusPaths::new(root) }
    }

    pub fn extract_source(&self, source_id: &str) -> AegisResult<ExtractionReport> {
        let registry_path = self.paths.registry_path();
        let mut registry = SourceRegistry::load(&registry_path)?;
        let record = registry.get_source(source_id)?;
        let report = self.extract_record(&record);
        match report {
            Ok(report) => {
                self.paths.ensure_layout()?;
                let mut updated = record.clone();
                updated.ingestion_status = IngestionStatus::Extracted;
                registry.remove_and_replace(updated)?;
                registry.save(&registry_path)?;
                self.write_report(&report)?;
                self.append_audit(AuditEventType::SourceExtracted, &report.source_id, &report.version_id, "source extracted")?;
                Ok(report)
            }
            Err(error) => {
                self.paths.ensure_layout()?;
                let mut failed = record.clone();
                failed.ingestion_status = IngestionStatus::Failed;
                registry.remove_and_replace(failed)?;
                registry.save(&registry_path)?;
                self.append_audit(AuditEventType::SourceExtractionFailed, &record.source_id, &record.version_id, "source extraction failed")?;
                Err(error)
            }
        }
    }

    pub fn read_extraction_report(&self, source_id: &str) -> AegisResult<ExtractionReport> {
        let registry = SourceRegistry::load(&self.paths.registry_path())?;
        let record = registry.get_source(source_id)?;
        self.paths.ensure_layout()?;
        let path = self.report_path(&record.source_id, &record.version_id);
        if !path.exists() {
            return Err(AegisError::ExtractionReportMissing);
        }
        let mut file = fs::File::open(&path).map_err(|_| AegisError::ExtractionReportReadFailed)?;
        let mut content = String::new();
        file.read_to_string(&mut content).map_err(|_| AegisError::ExtractionReportReadFailed)?;
        serde_json::from_str(&content).map_err(|_| AegisError::ExtractionReportReadFailed)
    }

    fn extract_record(&self, record: &SourceRecord) -> AegisResult<ExtractionReport> {
        match &record.source_type {
            SourceType::Pdf => self.extract_pdf_text(record),
            SourceType::MarkdownNote | SourceType::DatasetNote | SourceType::WebSnapshot => {
                let mut bytes = Vec::new();
                fs::File::open(&record.path)
                    .map_err(|_| AegisError::ExtractionInputMissing)?
                    .read_to_end(&mut bytes)
                    .map_err(|_| AegisError::ExtractionInputMissing)?;
                let text = String::from_utf8(bytes).map_err(|_| AegisError::ExtractionInputNotUtf8)?;
                self.extract_text(record, text)
            }
            other => Err(AegisError::UnsupportedExtractionType(format!("{other:?}"))),
        }
    }

    fn extract_pdf_text(&self, record: &SourceRecord) -> AegisResult<ExtractionReport> {
        let bytes = fs::read(&record.path).map_err(|_| AegisError::ExtractionInputMissing)?;
        let document = Document::load_mem(&bytes).map_err(|_| AegisError::PdfTextExtractionFailed)?;
        let mut units = Vec::new();
        let mut warnings = Vec::new();

        for page_number in document.get_pages().keys().copied() {
            let raw_text = document.extract_text(&[page_number]).map_err(|_| AegisError::PdfTextExtractionFailed)?;
            let text = raw_text.trim().to_string();
            if text.is_empty() {
                warnings.push(format!("page {} has no extractable text layer", page_number));
                continue;
            }

            let char_end = text.len();
            units.push(ExtractedUnit {
                source_id: record.source_id.clone(),
                version_id: record.version_id.clone(),
                locator: CitationLocator {
                    locator_type: crate::locators::LocatorType::Page,
                    label: format!("page:{}", page_number),
                    page: Some(page_number),
                    slide: None,
                    section_path: None,
                    character_start: Some(0),
                    character_end: Some(char_end),
                },
                text: text.clone(),
                text_hash: sha256_text(&text),
                char_start: 0,
                char_end,
            });
        }

        if units.is_empty() {
            return Err(AegisError::PdfTextLayerMissing);
        }

        Ok(ExtractionReport {
            source_id: record.source_id.clone(),
            version_id: record.version_id.clone(),
            source_type: record.source_type.clone(),
            extracted_at: Utc::now(),
            unit_count: units.len(),
            warnings,
            units,
        })
    }

    fn extract_text(&self, record: &SourceRecord, text: String) -> AegisResult<ExtractionReport> {
        let units = parse_markdown_text(&text, &record.source_id, &record.version_id)?;
        Ok(ExtractionReport {
            source_id: record.source_id.clone(),
            version_id: record.version_id.clone(),
            source_type: record.source_type.clone(),
            extracted_at: Utc::now(),
            unit_count: units.len(),
            warnings: Vec::new(),
            units,
        })
    }

    fn write_report(&self, report: &ExtractionReport) -> AegisResult<()> {
        let path = self.report_path(&report.source_id, &report.version_id);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|_| AegisError::ExtractionReportWriteFailed)?;
        }
        let body = serde_json::to_string_pretty(report)?;
        fs::write(&path, body).map_err(|_| AegisError::ExtractionReportWriteFailed)?;
        Ok(())
    }

    fn append_audit(
        &self,
        event_type: AuditEventType,
        source_id: &str,
        version_id: &str,
        summary: &str,
    ) -> AegisResult<()> {
        let event = AuditEvent::new(event_type, Some(source_id.to_string()), Some(version_id.to_string()), summary);
        append_audit_event(&self.paths.audit_events_path(), &event)
    }

    pub fn report_path(&self, source_id: &str, version_id: &str) -> PathBuf {
        self.paths
            .source_version_dir(source_id, version_id)
            .join("extraction")
            .join("report.json")
    }
}

impl SourceRegistry {
    fn remove_and_replace(&mut self, source: SourceRecord) -> AegisResult<()> {
        self.sources.retain(|item| item.source_id != source.source_id);
        self.sources.push(source);
        Ok(())
    }
}

pub fn parse_markdown_text(
    text: &str,
    source_id: &str,
    version_id: &str,
) -> AegisResult<Vec<ExtractedUnit>> {
    let mut units = Vec::new();
    let mut section_path: Vec<String> = Vec::new();
    let mut paragraph_index = 0_usize;
    let mut current_para_start: Option<usize> = None;
    let mut current_para_end: usize = 0;

    for (line_start, raw_line) in split_lines_with_offsets(text) {
        let line = raw_line.trim_end_matches(['\r', '\n']);
        let trimmed = line.trim();

        if trimmed.is_empty() {
            flush_paragraph(
                source_id,
                version_id,
                &section_path,
                &mut paragraph_index,
                &mut current_para_start,
                current_para_end,
                text,
                &mut units,
            )?;
            continue;
        }

        if let Some((level, title)) = parse_heading(trimmed) {
            flush_paragraph(
                source_id,
                version_id,
                &section_path,
                &mut paragraph_index,
                &mut current_para_start,
                current_para_end,
                text,
                &mut units,
            )?;
            section_path.truncate(level.saturating_sub(1));
            section_path.push(title);
            continue;
        }

        if current_para_start.is_none() {
            current_para_start = Some(line_start + line.len() - line.trim_start().len());
        }
        current_para_end = line_start + line.trim_end().len();
    }

    flush_paragraph(
        source_id,
        version_id,
        &section_path,
        &mut paragraph_index,
        &mut current_para_start,
        current_para_end,
        text,
        &mut units,
    )?;

    Ok(units)
}

fn flush_paragraph(
    source_id: &str,
    version_id: &str,
    section_path: &[String],
    paragraph_index: &mut usize,
    start: &mut Option<usize>,
    end: usize,
    text: &str,
    units: &mut Vec<ExtractedUnit>,
) -> AegisResult<()> {
    let Some(start) = start.take() else { return Ok(()); };
    if end <= start {
        return Ok(());
    }
    let body = &text[start..end];
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Ok(());
    }
    *paragraph_index += 1;
    let char_start = start + body.find(trimmed).unwrap_or(0);
    let char_end = char_start + trimmed.len();
    let locator = CitationLocator::paragraph(
        format!("paragraph:{}", *paragraph_index),
        if section_path.is_empty() { None } else { Some(section_path.to_vec()) },
        char_start,
        char_end,
    );
    units.push(ExtractedUnit {
        source_id: source_id.to_string(),
        version_id: version_id.to_string(),
        locator,
        text: trimmed.to_string(),
        text_hash: sha256_text(trimmed),
        char_start,
        char_end,
    });
    Ok(())
}

fn parse_heading(line: &str) -> Option<(usize, String)> {
    let level = line.chars().take_while(|c| *c == '#').count();
    if level == 0 {
        return None;
    }
    let title = line[level..].trim();
    if title.is_empty() {
        return None;
    }
    Some((level, title.to_string()))
}

fn split_lines_with_offsets(text: &str) -> Vec<(usize, &str)> {
    let mut result = Vec::new();
    let mut start = 0;
    for line in text.split_inclusive('\n') {
        result.push((start, line));
        start += line.len();
    }
    if !text.ends_with('\n') && start < text.len() {
        result.push((start, &text[start..]));
    }
    result
}

fn sha256_text(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    format!("sha256:{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lopdf::{content::{Content, Operation}, dictionary, Document, Object, Stream};
    use crate::source_metadata::{IngestionStatus, SourceMetadataInput};
    use crate::corpus_authority::CorpusAuthority;
    use crate::locators::LocatorType;
    use crate::source_metadata::SourceType;
    use std::fs;
    use std::path::Path;

    fn valid_metadata(source_type: SourceType) -> SourceMetadataInput {
        SourceMetadataInput {
            title: "Notes".to_string(),
            source_type,
            discipline: "psychology".to_string(),
            subdiscipline: Some("statistics".to_string()),
            language: "en".to_string(),
            tags: vec!["study".to_string()],
            reliability_notes: None,
        }
    }

    fn write_pdf_with_pages(path: &Path, pages: &[&str]) {
        let mut doc = Document::with_version("1.5");
        let pages_id = doc.new_object_id();
        let font_id = doc.new_object_id();
        let resources_id = doc.new_object_id();

        doc.objects.insert(
            font_id,
            Object::Dictionary(dictionary! {
                "Type" => "Font",
                "Subtype" => "Type1",
                "BaseFont" => "Helvetica",
            }),
        );
        doc.objects.insert(
            resources_id,
            Object::Dictionary(dictionary! {
                "Font" => dictionary! {
                    "F1" => Object::Reference(font_id),
                },
            }),
        );

        let mut page_ids = Vec::new();
        for page_text in pages {
            let content_id = doc.new_object_id();
            let mut operations = vec![
                Operation::new("BT", vec![]),
                Operation::new("Tf", vec![Object::Name(b"F1".to_vec()), Object::Integer(18)]),
                Operation::new("Td", vec![Object::Integer(50), Object::Integer(700)]),
            ];
            if !page_text.is_empty() {
                operations.push(Operation::new("Tj", vec![Object::string_literal(*page_text)]));
            }
            operations.push(Operation::new("ET", vec![]));
            let content = Content { operations };
            doc.objects.insert(
                content_id,
                Object::Stream(Stream::new(dictionary! {}, content.encode().unwrap())),
            );

            let page_id = doc.new_object_id();
            doc.objects.insert(
                page_id,
                Object::Dictionary(dictionary! {
                    "Type" => "Page",
                    "Parent" => Object::Reference(pages_id),
                    "Contents" => Object::Reference(content_id),
                    "Resources" => Object::Reference(resources_id),
                    "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
                }),
            );
            page_ids.push(page_id);
        }

        doc.objects.insert(
            pages_id,
            Object::Dictionary(dictionary! {
                "Type" => "Pages",
                "Kids" => page_ids.iter().copied().map(Object::Reference).collect::<Vec<_>>(),
                "Count" => page_ids.len() as i64,
            }),
        );

        let catalog_id = doc.new_object_id();
        doc.objects.insert(
            catalog_id,
            Object::Dictionary(dictionary! {
                "Type" => "Catalog",
                "Pages" => Object::Reference(pages_id),
            }),
        );
        doc.trailer.set("Root", Object::Reference(catalog_id));
        doc.compress();
        doc.save(path).unwrap();
    }

    #[test]
    fn paragraphs_receive_stable_labels() {
        let units = parse_markdown_text("a\n\nb\n", "src_1", "srcv_1").unwrap();
        assert_eq!(units[0].locator.label, "paragraph:1");
        assert_eq!(units[1].locator.label, "paragraph:2");
    }

    #[test]
    fn empty_paragraphs_are_ignored_and_offsets_slice_text() {
        let units = parse_markdown_text("  hello  \n\nworld\n", "src_1", "srcv_1").unwrap();
        assert_eq!(units.len(), 2);
        assert_eq!(units[0].text, "hello");
        assert_eq!(&"  hello  \n\nworld\n"[units[0].char_start..units[0].char_end], "hello");
        assert_eq!(units[1].text, "world");
    }

    #[test]
    fn markdown_headings_produce_section_paths() {
        let units = parse_markdown_text("# Intro\n\nalpha\n", "src_1", "srcv_1").unwrap();
        assert_eq!(units[0].locator.section_path.as_ref().unwrap(), &vec!["Intro".to_string()]);
    }

    #[test]
    fn nested_markdown_headings_produce_nested_section_paths() {
        let units = parse_markdown_text("# Intro\n## Stats\nvalue\n", "src_1", "srcv_1").unwrap();
        assert_eq!(
            units[0].locator.section_path.as_ref().unwrap(),
            &vec!["Intro".to_string(), "Stats".to_string()]
        );
    }

    #[test]
    fn extractor_updates_status_and_writes_report() {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("note.md");
        fs::write(&source_path, "# Heading\n\nfirst paragraph\n").unwrap();

        let authority = CorpusAuthority::new(temp.path());
        let record = authority.register_source(&source_path, valid_metadata(SourceType::MarkdownNote)).unwrap();
        let report = ExtractionService::new(temp.path()).extract_source(&record.source_id).unwrap();

        assert_eq!(report.source_id, record.source_id);
        assert_eq!(report.version_id, record.version_id);
        assert_eq!(report.unit_count, 1);

        let reread = CorpusAuthority::new(temp.path()).get_source(&record.source_id).unwrap();
        assert_eq!(reread.ingestion_status, IngestionStatus::Extracted);

        let report_path = ExtractionService::new(temp.path()).report_path(&record.source_id, &record.version_id);
        assert!(report_path.exists());

        let audit_path = temp.path().join(".aegis").join("audit").join("events.jsonl");
        let audit = fs::read_to_string(audit_path).unwrap();
        assert!(audit.contains("source_extracted"));

        let reread = ExtractionService::new(temp.path())
            .read_extraction_report(&record.source_id)
            .unwrap();
        assert_eq!(reread.source_id, record.source_id);
        assert_eq!(reread.version_id, record.version_id);
    }

    #[test]
    fn unsupported_source_type_returns_typed_error() {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("paper.txt");
        fs::write(&source_path, "text").unwrap();

        let authority = CorpusAuthority::new(temp.path());
        let record = authority.register_source(&source_path, valid_metadata(SourceType::Paper)).unwrap();
        let result = ExtractionService::new(temp.path()).extract_source(&record.source_id);

        assert!(matches!(result, Err(AegisError::UnsupportedExtractionType(_))));
    }

    #[test]
    fn pdf_text_layer_extracts_page_locators_and_preserves_order() {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("paper.pdf");
        write_pdf_with_pages(&source_path, &["First page text", "Second page text"]);

        let authority = CorpusAuthority::new(temp.path());
        let record = authority.register_source(&source_path, valid_metadata(SourceType::Pdf)).unwrap();
        let report = ExtractionService::new(temp.path()).extract_source(&record.source_id).unwrap();

        assert_eq!(report.source_type, SourceType::Pdf);
        assert_eq!(report.unit_count, 2);
        assert_eq!(report.units[0].locator.locator_type, LocatorType::Page);
        assert_eq!(report.units[0].locator.page, Some(1));
        assert_eq!(report.units[0].locator.label, "page:1");
        assert_eq!(report.units[0].text, "First page text");
        assert_eq!(report.units[1].locator.locator_type, LocatorType::Page);
        assert_eq!(report.units[1].locator.page, Some(2));
        assert_eq!(report.units[1].locator.label, "page:2");
        assert_eq!(report.units[1].text, "Second page text");
        assert!(report.warnings.is_empty());
    }

    #[test]
    fn pdf_without_text_layer_returns_typed_error_and_writes_no_report() {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("blank.pdf");
        write_pdf_with_pages(&source_path, &[""]);

        let authority = CorpusAuthority::new(temp.path());
        let record = authority.register_source(&source_path, valid_metadata(SourceType::Pdf)).unwrap();
        let service = ExtractionService::new(temp.path());
        let result = service.extract_source(&record.source_id);
        assert!(matches!(result, Err(AegisError::PdfTextLayerMissing)));

        let report_path = service.report_path(&record.source_id, &record.version_id);
        assert!(!report_path.exists());
        let reread = CorpusAuthority::new(temp.path()).get_source(&record.source_id).unwrap();
        assert_eq!(reread.ingestion_status, IngestionStatus::Failed);
    }

    #[test]
    fn invalid_utf8_source_returns_typed_error_and_marks_failed() {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("note.md");
        fs::write(&source_path, vec![0xff, 0xfe, 0xfd]).unwrap();

        let authority = CorpusAuthority::new(temp.path());
        let record = authority.register_source(&source_path, valid_metadata(SourceType::MarkdownNote)).unwrap();
        let result = ExtractionService::new(temp.path()).extract_source(&record.source_id);

        assert!(matches!(result, Err(AegisError::ExtractionInputNotUtf8)));
        let updated = CorpusAuthority::new(temp.path()).get_source(&record.source_id).unwrap();
        assert_eq!(updated.ingestion_status, IngestionStatus::Failed);
        let report_path = ExtractionService::new(temp.path()).report_path(&record.source_id, &record.version_id);
        assert!(!report_path.exists());
    }

    #[test]
    fn missing_source_id_returns_typed_error_without_creating_layout() {
        let temp = tempfile::tempdir().unwrap();
        let result = ExtractionService::new(temp.path()).extract_source("src_missing");
        assert!(matches!(result, Err(AegisError::SourceNotFound(_))));
        assert!(!temp.path().join(".aegis").exists());
    }

    #[test]
    fn missing_pdf_file_returns_typed_error() {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("paper.pdf");
        write_pdf_with_pages(&source_path, &["hello"]);

        let authority = CorpusAuthority::new(temp.path());
        let record = authority.register_source(&source_path, valid_metadata(SourceType::Pdf)).unwrap();
        fs::remove_file(&record.path).unwrap();

        let result = ExtractionService::new(temp.path()).extract_source(&record.source_id);
        assert!(matches!(result, Err(AegisError::ExtractionInputMissing)));
    }

    #[test]
    fn missing_report_returns_typed_error() {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("note.md");
        fs::write(&source_path, "hello").unwrap();

        let authority = CorpusAuthority::new(temp.path());
        let record = authority.register_source(&source_path, valid_metadata(SourceType::MarkdownNote)).unwrap();
        let result = ExtractionService::new(temp.path()).read_extraction_report(&record.source_id);
        assert!(matches!(result, Err(AegisError::ExtractionReportMissing)));
    }

    #[test]
    fn malformed_report_returns_typed_error() {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("note.md");
        fs::write(&source_path, "hello").unwrap();

        let authority = CorpusAuthority::new(temp.path());
        let record = authority.register_source(&source_path, valid_metadata(SourceType::MarkdownNote)).unwrap();
        let path = ExtractionService::new(temp.path()).report_path(&record.source_id, &record.version_id);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "{not json").unwrap();
        let result = ExtractionService::new(temp.path()).read_extraction_report(&record.source_id);
        assert!(matches!(result, Err(AegisError::ExtractionReportReadFailed)));
    }

    #[test]
    fn crlf_unicode_and_whitespace_offsets_are_slice_valid() {
        let text = "# One\r\n\r\n  Grüß Gott  \r\n";
        let units = parse_markdown_text(text, "src_1", "srcv_1").unwrap();
        assert_eq!(units.len(), 1);
        assert_eq!(units[0].locator.section_path.as_ref().unwrap(), &vec!["One".to_string()]);
        assert_eq!(units[0].text, "Grüß Gott");
        assert_eq!(&text[units[0].char_start..units[0].char_end], "Grüß Gott");
    }

    #[test]
    fn final_paragraph_without_trailing_newline_is_extracted() {
        let units = parse_markdown_text("alpha\n\nomega", "src_1", "srcv_1").unwrap();
        assert_eq!(units.len(), 2);
        assert_eq!(units[1].text, "omega");
    }

    #[test]
    fn heading_reset_behavior_is_correct() {
        let units = parse_markdown_text("# A\n## B\npara\n# C\npara2\n", "src_1", "srcv_1").unwrap();
        assert_eq!(units[0].locator.section_path.as_ref().unwrap(), &vec!["A".to_string(), "B".to_string()]);
        assert_eq!(units[1].locator.section_path.as_ref().unwrap(), &vec!["C".to_string()]);
    }

    #[test]
    fn paragraphs_before_first_heading_have_empty_section_path() {
        let units = parse_markdown_text("intro\n\n# H\nbody\n", "src_1", "srcv_1").unwrap();
        assert!(units[0].locator.section_path.is_none());
        assert_eq!(units[1].locator.section_path.as_ref().unwrap(), &vec!["H".to_string()]);
    }
}
