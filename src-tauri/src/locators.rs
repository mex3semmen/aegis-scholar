use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LocatorType {
    Page,
    Slide,
    Section,
    Paragraph,
    Theorem,
    Definition,
    Equation,
    Table,
    Figure,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CitationLocator {
    pub locator_type: LocatorType,
    pub label: String,
    pub page: Option<u32>,
    pub slide: Option<u32>,
    pub section_path: Option<Vec<String>>,
    // Byte offsets into the original UTF-8 source text. The schema keeps the legacy
    // char_start/char_end field names for compatibility.
    pub character_start: Option<usize>,
    pub character_end: Option<usize>,
}

impl CitationLocator {
    pub fn paragraph(
        label: impl Into<String>,
        section_path: Option<Vec<String>>,
        character_start: usize,
        character_end: usize,
    ) -> Self {
        Self {
            locator_type: LocatorType::Paragraph,
            label: label.into(),
            page: None,
            slide: None,
            section_path,
            character_start: Some(character_start),
            character_end: Some(character_end),
        }
    }

    #[allow(dead_code)]
    pub fn section(label: impl Into<String>, section_path: Vec<String>, character_start: usize, character_end: usize) -> Self {
        Self {
            locator_type: LocatorType::Section,
            label: label.into(),
            page: None,
            slide: None,
            section_path: Some(section_path),
            character_start: Some(character_start),
            character_end: Some(character_end),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locator_serializes_with_required_fields() {
        let locator = CitationLocator::paragraph("paragraph:1", Some(vec!["Heading".into()]), 0, 4);
        let value = serde_json::to_value(locator).unwrap();
        assert_eq!(value["locator_type"], "paragraph");
        assert_eq!(value["label"], "paragraph:1");
    }
}
