// core/parsers/html.rs — HTML document parser and reconstructor

use std::path::Path;
use anyhow::{Result, Context};
use quick_xml::events::{Event, BytesText};
use quick_xml::Reader;
use quick_xml::Writer;

use crate::core::models::Segment;
use crate::core::segmentation::srx;
use super::RawSegment;

/// Tags whose contents should NEVER be translated
const NON_TRANSLATABLE_TAGS: &[&[u8]] = &[
    b"script", b"style", b"code", b"pre", b"svg", b"noscript",
];

/// Tags that indicate structural blocks (paragraph, header, list item, table cell)
const BLOCK_TAGS: &[&[u8]] = &[
    b"p", b"h1", b"h2", b"h3", b"h4", b"h5", b"h6",
    b"li", b"td", b"th", b"blockquote", b"caption",
    b"figcaption", b"title",
];

/// Parse an HTML document and extract translatable text segments.
pub fn parse(path: &Path) -> Result<Vec<RawSegment>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read HTML file: {:?}", path))?;
    parse_html_str(&content)
}

pub fn parse_html_str(html: &str) -> Result<Vec<RawSegment>> {
    let mut reader = Reader::from_str(html);
    reader.config_mut().trim_text(false);

    let mut segments = Vec::new();
    let mut buf = Vec::new();

    let mut in_skip = false;
    let mut current_block_tag: Option<Vec<u8>> = None;
    let mut current_block_text = String::new();
    let mut block_idx = 0usize;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = e.name().as_ref().to_ascii_lowercase();
                if NON_TRANSLATABLE_TAGS.contains(&name.as_slice()) {
                    in_skip = true;
                } else if BLOCK_TAGS.contains(&name.as_slice()) {
                    current_block_tag = Some(name);
                    current_block_text.clear();
                }
            }
            Ok(Event::End(ref e)) => {
                let name = e.name().as_ref().to_ascii_lowercase();
                if NON_TRANSLATABLE_TAGS.contains(&name.as_slice()) {
                    in_skip = false;
                } else if current_block_tag.as_ref().map(|t| t.as_slice()) == Some(&name) {
                    current_block_tag = None;
                    let trimmed = current_block_text.trim();
                    if !trimmed.is_empty() {
                        block_idx += 1;
                        let sentences = srx::segment(trimmed, "en");
                        for (s_idx, s) in sentences.into_iter().enumerate() {
                            let s_clean = s.trim();
                            if !s_clean.is_empty() {
                                segments.push(RawSegment {
                                    source_text: s_clean.to_string(),
                                    target_text: None,
                                    notes: None,
                                    context: Some(format!("b{}_s{}", block_idx, s_idx + 1)),
                                });
                            }
                        }
                    }
                    current_block_text.clear();
                }
            }
            Ok(Event::Text(ref e)) => {
                if !in_skip && current_block_tag.is_some() {
                    let text = e.unescape().unwrap_or_default();
                    current_block_text.push_str(&text);
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => {
                // If quick_xml encounters non-strict HTML entities, continue gracefully
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(segments)
}

/// Export translated segments back into an HTML document.
pub fn export(segments: &[Segment], source_path: &str, out_path: &str) -> Result<()> {
    let source_html = std::fs::read_to_string(source_path)
        .with_context(|| format!("Failed to read source HTML: {}", source_path))?;
    let translated_html = reconstruct_html(&source_html, segments)?;
    std::fs::write(out_path, translated_html)
        .with_context(|| format!("Failed to write translated HTML: {}", out_path))?;
    Ok(())
}

pub fn reconstruct_html(source_html: &str, segments: &[Segment]) -> Result<String> {
    let mut reader = Reader::from_str(source_html);
    reader.config_mut().trim_text(false);

    let mut writer = Writer::new(Vec::new());
    let mut buf = Vec::new();

    let mut seg_iter = segments.iter();
    let mut in_skip = false;
    let mut in_block = false;
    let mut block_has_text = false;
    let mut block_tag_name = Vec::new();
    let mut block_text = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = e.name().as_ref().to_ascii_lowercase();
                if NON_TRANSLATABLE_TAGS.contains(&name.as_slice()) {
                    in_skip = true;
                    writer.write_event(Event::Start(e.clone()))?;
                } else if BLOCK_TAGS.contains(&name.as_slice()) {
                    in_block = true;
                    block_has_text = false;
                    block_tag_name = name;
                    block_text.clear();
                    writer.write_event(Event::Start(e.clone()))?;
                } else {
                    writer.write_event(Event::Start(e.clone()))?;
                }
            }
            Ok(Event::End(ref e)) => {
                let name = e.name().as_ref().to_ascii_lowercase();
                if NON_TRANSLATABLE_TAGS.contains(&name.as_slice()) {
                    in_skip = false;
                    writer.write_event(Event::End(e.clone()))?;
                } else if in_block && block_tag_name == name {
                    in_block = false;
                    if block_has_text && !block_text.trim().is_empty() {
                        let translated = if let Some(seg) = seg_iter.next() {
                            if !seg.target_text.trim().is_empty() {
                                seg.target_text.as_str()
                            } else {
                                seg.source_text.as_str()
                            }
                        } else {
                            block_text.as_str()
                        };
                        writer.write_event(Event::Text(BytesText::new(translated)))?;
                    }
                    writer.write_event(Event::End(e.clone()))?;
                } else {
                    writer.write_event(Event::End(e.clone()))?;
                }
            }
            Ok(Event::Text(ref e)) => {
                if in_block && !in_skip {
                    let text = e.unescape().unwrap_or_default();
                    block_text.push_str(&text);
                    block_has_text = true;
                    // Do not write original text; it will be substituted on block End
                } else {
                    writer.write_event(Event::Text(e.clone()))?;
                }
            }
            Ok(Event::Eof) => break,
            Ok(evt) => {
                writer.write_event(evt)?;
            }
            Err(_) => {}
        }
        buf.clear();
    }

    let out = String::from_utf8(writer.into_inner())?;
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{Segment, SegmentStatus};

    const SAMPLE_HTML: &str = r#"<!DOCTYPE html>
<html>
<head><title>Test Page</title></head>
<body>
  <h1>Welcome to Free-CAT</h1>
  <p>Free-CAT is an open-source CAT tool.</p>
  <script>console.log("ignore me");</script>
</body>
</html>"#;

    #[test]
    fn test_parse_html() {
        let segments = parse_html_str(SAMPLE_HTML).unwrap();
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].source_text, "Test Page");
        assert_eq!(segments[1].source_text, "Welcome to Free-CAT");
        assert_eq!(segments[2].source_text, "Free-CAT is an open-source CAT tool.");
    }

    #[test]
    fn test_reconstruct_html() {
        let segments = vec![
            Segment {
                id: "1".into(), file_id: "f".into(), seq_number: 1,
                source_text: "Test Page".into(), target_text: "صفحة الاختبار".into(),
                status: SegmentStatus::Final, match_percent: None, match_origin: None,
                locked: false, bookmarked: false, comment: None, created_at: "".into(),
                modified_at: "".into(), modified_by: None,
            },
            Segment {
                id: "2".into(), file_id: "f".into(), seq_number: 2,
                source_text: "Welcome to Free-CAT".into(), target_text: "مرحباً بكم في Free-CAT".into(),
                status: SegmentStatus::Final, match_percent: None, match_origin: None,
                locked: false, bookmarked: false, comment: None, created_at: "".into(),
                modified_at: "".into(), modified_by: None,
            },
            Segment {
                id: "3".into(), file_id: "f".into(), seq_number: 3,
                source_text: "Free-CAT is an open-source CAT tool.".into(),
                target_text: "Free-CAT هي أداة ترجمة مفتوحة المصدر.".into(),
                status: SegmentStatus::Final, match_percent: None, match_origin: None,
                locked: false, bookmarked: false, comment: None, created_at: "".into(),
                modified_at: "".into(), modified_by: None,
            },
        ];

        let result = reconstruct_html(SAMPLE_HTML, &segments).unwrap();
        assert!(result.contains("صفحة الاختبار"));
        assert!(result.contains("مرحباً بكم في Free-CAT"));
        assert!(result.contains("Free-CAT هي أداة ترجمة مفتوحة المصدر."));
        assert!(result.contains(r#"console.log("ignore me");"#));
    }
}
