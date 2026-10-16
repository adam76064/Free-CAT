// core/parsers/docx.rs — Microsoft Word (.docx) OpenXML parser and exporter
// Standard: ECMA-376 (Office Open XML File Formats)

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use anyhow::{Result, Context, anyhow};
use quick_xml::events::{Event, BytesStart, BytesText};
use quick_xml::Reader;
use quick_xml::Writer;
use zip::read::ZipArchive;
use zip::write::{SimpleFileOptions, ZipWriter};

use crate::core::models::Segment;
use crate::core::segmentation::srx;
use super::RawSegment;

/// Parse a .docx file and extract translatable text segments.
pub fn parse(path: &Path) -> Result<Vec<RawSegment>> {
    let file = File::open(path)
        .with_context(|| format!("Failed to open DOCX file: {:?}", path))?;
    let mut archive = ZipArchive::new(file)
        .with_context(|| "Failed to read DOCX ZIP archive")?;

    let mut doc_xml = String::new();
    {
        let mut doc_file = archive.by_name("word/document.xml")
            .with_context(|| "word/document.xml not found in DOCX archive")?;
        doc_file.read_to_string(&mut doc_xml)?;
    }

    parse_docx_xml(&doc_xml)
}

/// Parse extracted word/document.xml content into segments.
pub fn parse_docx_xml(xml: &str) -> Result<Vec<RawSegment>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut raw_segments = Vec::new();
    let mut buf = Vec::new();

    let mut in_p = false;
    let mut in_t = false;
    let mut current_p_text = String::new();
    let mut p_index = 0usize;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name().as_ref() {
                b"w:p" => {
                    in_p = true;
                    current_p_text.clear();
                }
                b"w:t" => {
                    if in_p {
                        in_t = true;
                    }
                }
                _ => {}
            },
            Ok(Event::Text(ref e)) => {
                if in_t {
                    let text = e.unescape().unwrap_or_default();
                    current_p_text.push_str(&text);
                }
            }
            Ok(Event::End(ref e)) => match e.name().as_ref() {
                b"w:t" => {
                    in_t = false;
                }
                b"w:p" => {
                    in_p = false;
                    let trimmed = current_p_text.trim();
                    if !trimmed.is_empty() {
                        p_index += 1;
                        // Segment paragraph using SRX rule engine
                        let sentences = srx::segment(trimmed, "en");
                        for (s_idx, sentence) in sentences.into_iter().enumerate() {
                            let s_trimmed = sentence.trim();
                            if !s_trimmed.is_empty() {
                                raw_segments.push(RawSegment {
                                    source_text: s_trimmed.to_string(),
                                    target_text: None,
                                    notes: None,
                                    context: Some(format!("p{}_s{}", p_index, s_idx + 1)),
                                });
                            }
                        }
                    }
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("XML parse error in word/document.xml: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(raw_segments)
}

/// Export translated segments back into a .docx document.
pub fn export(segments: &[Segment], source_path: &str, out_path: &str) -> Result<()> {
    let src_file = File::open(source_path)
        .with_context(|| format!("Failed to open source DOCX: {}", source_path))?;
    let mut src_archive = ZipArchive::new(src_file)?;

    // Read original document.xml
    let mut original_xml = String::new();
    {
        let mut doc_file = src_archive.by_name("word/document.xml")
            .with_context(|| "word/document.xml not found in source DOCX")?;
        doc_file.read_to_string(&mut original_xml)?;
    }

    // Reconstruct translated document.xml
    let translated_xml = reconstruct_docx_xml(&original_xml, segments)?;

    // Create target DOCX zip archive
    let out_file = File::create(out_path)
        .with_context(|| format!("Failed to create output DOCX: {}", out_path))?;
    let mut out_archive = ZipWriter::new(out_file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // Copy all original files, substituting translated word/document.xml
    for i in 0..src_archive.len() {
        let mut entry = src_archive.by_index(i)?;
        let name = entry.name().to_string();

        if name.ends_with('/') {
            out_archive.add_directory(&name, SimpleFileOptions::default())?;
            continue;
        }

        out_archive.start_file(&name, options)?;
        if name == "word/document.xml" {
            out_archive.write_all(translated_xml.as_bytes())?;
        } else {
            let mut data = Vec::new();
            entry.read_to_end(&mut data)?;
            out_archive.write_all(&data)?;
        }
    }

    out_archive.finish()?;
    Ok(())
}

/// Reconstruct word/document.xml with translated text in each paragraph.
pub fn reconstruct_docx_xml(original_xml: &str, segments: &[Segment]) -> Result<String> {
    let mut reader = Reader::from_str(original_xml);
    reader.config_mut().trim_text(false);

    let mut writer = Writer::new(Vec::new());
    let mut buf = Vec::new();

    // Map segments to paragraph indices or sequential order
    let mut seg_iter = segments.iter();

    let mut in_p = false;
    let mut p_has_text = false;
    let mut p_buffer_events: Vec<Event<'static>> = Vec::new();
    let mut p_text = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"w:p" => {
                in_p = true;
                p_has_text = false;
                p_text.clear();
                p_buffer_events.clear();
                p_buffer_events.push(Event::Start(e.clone().into_owned()));
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"w:p" => {
                in_p = false;
                p_buffer_events.push(Event::End(e.clone().into_owned()));

                if p_has_text && !p_text.trim().is_empty() {
                    // Substitute paragraph text with translated segment
                    let translated_text = if let Some(seg) = seg_iter.next() {
                        if !seg.target_text.trim().is_empty() {
                            seg.target_text.clone()
                        } else {
                            seg.source_text.clone()
                        }
                    } else {
                        p_text.clone()
                    };

                    write_translated_paragraph(&mut writer, &p_buffer_events, &translated_text)?;
                } else {
                    // Empty paragraph or formatting spacer — write as-is
                    for evt in &p_buffer_events {
                        writer.write_event(evt.clone())?;
                    }
                }
                p_buffer_events.clear();
            }
            Ok(Event::Eof) => break,
            Ok(evt) => {
                if in_p {
                    if let Event::Text(ref t) = evt {
                        let text = t.unescape().unwrap_or_default();
                        p_text.push_str(&text);
                        p_has_text = true;
                    }
                    p_buffer_events.push(evt.into_owned());
                } else {
                    writer.write_event(evt)?;
                }
            }
            Err(e) => return Err(anyhow!("XML reconstruction error: {}", e)),
        }
        buf.clear();
    }

    let result = String::from_utf8(writer.into_inner())?;
    Ok(result)
}

/// Write a translated paragraph preserving all properties (<w:pPr>) and styling runs.
fn write_translated_paragraph(
    writer: &mut Writer<Vec<u8>>,
    events: &[Event<'static>],
    translated_text: &str,
) -> Result<()> {
    let mut wrote_translation = false;
    let mut in_p_pr = false;

    for evt in events {
        match evt {
            Event::Start(ref e) if e.name().as_ref() == b"w:pPr" => {
                in_p_pr = true;
                writer.write_event(Event::Start(e.clone()))?;
            }
            Event::End(ref e) if e.name().as_ref() == b"w:pPr" => {
                in_p_pr = false;
                writer.write_event(Event::End(e.clone()))?;
            }
            _ if in_p_pr => {
                // Preserve all paragraph properties intact
                writer.write_event(evt.clone())?;
            }
            Event::Start(ref e) if e.name().as_ref() == b"w:p" => {
                writer.write_event(Event::Start(e.clone()))?;
            }
            Event::End(ref e) if e.name().as_ref() == b"w:p" => {
                if !wrote_translation {
                    // Write single run containing translated text
                    write_run_with_text(writer, translated_text)?;
                    wrote_translation = true;
                }
                writer.write_event(Event::End(e.clone()))?;
            }
            _ => {
                // Once pPr is written, insert the translated run and omit original <w:r> runs
                if !in_p_pr && !wrote_translation {
                    write_run_with_text(writer, translated_text)?;
                    wrote_translation = true;
                }
            }
        }
    }

    Ok(())
}

/// Helper to write `<w:r><w:t xml:space="preserve">text</w:t></w:r>`
fn write_run_with_text(writer: &mut Writer<Vec<u8>>, text: &str) -> Result<()> {
    writer.write_event(Event::Start(BytesStart::new("w:r")))?;
    let mut t_start = BytesStart::new("w:t");
    t_start.push_attribute(("xml:space", "preserve"));
    writer.write_event(Event::Start(t_start))?;
    writer.write_event(Event::Text(BytesText::new(text)))?;
    writer.write_event(Event::End(quick_xml::events::BytesEnd::new("w:t")))?;
    writer.write_event(Event::End(quick_xml::events::BytesEnd::new("w:r")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_DOCX_XML: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>
      <w:pPr><w:pStyle w:val="Heading1"/></w:pPr>
      <w:r><w:t>Welcome to Free-CAT.</w:t></w:r>
    </w:p>
    <w:p>
      <w:r><w:t>This is a professional Computer-Assisted Translation tool.</w:t></w:r>
      <w:r><w:t> It supports multiple file formats.</w:t></w:r>
    </w:p>
  </w:body>
</w:document>"#;

    #[test]
    fn test_parse_docx_xml() {
        let segments = parse_docx_xml(SAMPLE_DOCX_XML).unwrap();
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0].source_text, "Welcome to Free-CAT.");
        assert_eq!(segments[1].source_text, "This is a professional Computer-Assisted Translation tool.");
        assert_eq!(segments[2].source_text, "It supports multiple file formats.");
    }

    use crate::core::models::SegmentStatus;

    #[test]
    fn test_reconstruct_docx_xml() {
        let segments = vec![
            Segment {
                id: "1".into(),
                file_id: "file1".into(),
                seq_number: 1,
                source_text: "Welcome to Free-CAT.".into(),
                target_text: "مرحباً بكم في Free-CAT.".into(),
                status: SegmentStatus::Final,
                match_percent: None,
                match_origin: None,
                locked: false,
                bookmarked: false,
                comment: None,
                created_at: "".into(),
                modified_at: "".into(),
                modified_by: None,
            },
            Segment {
                id: "2".into(),
                file_id: "file1".into(),
                seq_number: 2,
                source_text: "This is a professional Computer-Assisted Translation tool.".into(),
                target_text: "هذه أداة ترجمة احترافية بمساعدة الحاسوب.".into(),
                status: SegmentStatus::Final,
                match_percent: None,
                match_origin: None,
                locked: false,
                bookmarked: false,
                comment: None,
                created_at: "".into(),
                modified_at: "".into(),
                modified_by: None,
            },
        ];

        let result = reconstruct_docx_xml(SAMPLE_DOCX_XML, &segments).unwrap();
        assert!(result.contains("مرحباً بكم في Free-CAT."));
        assert!(result.contains("هذه أداة ترجمة احترافية بمساعدة الحاسوب."));
        assert!(result.contains(r#"<w:pStyle w:val="Heading1"/>"#));
    }
}
