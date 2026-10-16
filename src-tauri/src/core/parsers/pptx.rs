// core/parsers/pptx.rs — PowerPoint Presentation OpenXML parser
// Extracts translatable text from ppt/slides/slide*.xml and speaker notes, preserving shapes and slide layouts.

use std::path::Path;
use std::io::{Read, Write, Cursor};
use anyhow::{Result, Context, anyhow};
use zip::{ZipArchive, ZipWriter, write::FileOptions};
use quick_xml::reader::Reader;
use quick_xml::events::Event;
use crate::core::parsers::RawSegment;
use crate::core::models::Segment;

pub fn parse(path: &Path) -> Result<Vec<RawSegment>> {
    let file = std::fs::File::open(path)
        .with_context(|| format!("Failed to open PPTX file: {:?}", path))?;
    let mut zip = ZipArchive::new(file)?;

    let mut segments = Vec::new();

    // Iterate over all entries in the ZIP
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let name = entry.name().to_string();

        if (name.starts_with("ppt/slides/slide") || name.starts_with("ppt/notesSlides/notesSlide")) && name.ends_with(".xml") {
            let mut content = String::new();
            entry.read_to_string(&mut content)?;
            extract_pptx_texts(&content, &name, &mut segments)?;
        }
    }

    Ok(segments)
}

fn extract_pptx_texts(xml: &str, file_name: &str, segments: &mut Vec<RawSegment>) -> Result<()> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut in_t_tag = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let local_name = e.local_name();
                if local_name.as_ref() == b"t" {
                    in_t_tag = true;
                }
            }
            Ok(Event::End(ref e)) => {
                let local_name = e.local_name();
                if local_name.as_ref() == b"t" {
                    in_t_tag = false;
                }
            }
            Ok(Event::Text(ref e)) if in_t_tag => {
                let text = e.unescape()?.to_string();
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    segments.push(RawSegment {
                        source_text: text,
                        target_text: None,
                        notes: None,
                        context: Some(file_name.to_string()),
                    });
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("Error reading XML in PPTX: {}", e)),
            _ => {}
        }
    }

    Ok(())
}

pub fn export(original_path: &Path, segments: &[Segment], out_path: &Path) -> Result<()> {
    let file = std::fs::File::open(original_path)
        .with_context(|| format!("Failed to open original PPTX: {:?}", original_path))?;
    let mut zip = ZipArchive::new(file)?;

    let out_file = std::fs::File::create(out_path)
        .with_context(|| format!("Failed to create output PPTX: {:?}", out_path))?;
    let mut out_zip = ZipWriter::new(out_file);

    let mut seg_idx = 0;

    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let name = entry.name().to_string();

        let options = FileOptions::<()>::default()
            .compression_method(entry.compression())
            .unix_permissions(entry.unix_mode().unwrap_or(0o644));

        out_zip.start_file(&name, options)?;

        if (name.starts_with("ppt/slides/slide") || name.starts_with("ppt/notesSlides/notesSlide")) && name.ends_with(".xml") {
            let mut xml_content = String::new();
            entry.read_to_string(&mut xml_content)?;
            let reconstructed = replace_pptx_texts(&xml_content, segments, &mut seg_idx)?;
            out_zip.write_all(reconstructed.as_bytes())?;
        } else {
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf)?;
            out_zip.write_all(&buf)?;
        }
    }

    out_zip.finish()?;
    Ok(())
}

pub fn replace_pptx_texts(xml: &str, segments: &[Segment], seg_idx: &mut usize) -> Result<String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut writer = quick_xml::Writer::new(Cursor::new(Vec::new()));
    let mut in_t_tag = false;

    loop {
        match reader.read_event()? {
            Event::Start(ref e) => {
                let local_name = e.local_name();
                if local_name.as_ref() == b"t" {
                    in_t_tag = true;
                }
                writer.write_event(Event::Start(e.clone()))?;
            }
            Event::End(ref e) => {
                let local_name = e.local_name();
                if local_name.as_ref() == b"t" {
                    in_t_tag = false;
                }
                writer.write_event(Event::End(e.clone()))?;
            }
            Event::Text(ref e) if in_t_tag => {
                let original_text = e.unescape()?.to_string();
                if !original_text.trim().is_empty() && *seg_idx < segments.len() {
                    let seg = &segments[*seg_idx];
                    let replacement = if !seg.target_text.trim().is_empty() {
                        &seg.target_text
                    } else {
                        &original_text
                    };
                    writer.write_event(Event::Text(quick_xml::events::BytesText::new(replacement)))?;
                    *seg_idx += 1;
                } else {
                    writer.write_event(Event::Text(e.clone()))?;
                }
            }
            Event::Eof => break,
            event => writer.write_event(event)?,
        }
    }

    let result = String::from_utf8(writer.into_inner().into_inner())?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::SegmentStatus;

    #[test]
    fn test_extract_and_replace_slide_text() {
        let xml = r#"<p:sp xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main">
  <p:txBody>
    <a:p>
      <a:r><a:t>Quarterly Financial Results</a:t></a:r>
    </a:p>
  </p:txBody>
</p:sp>"#;

        let mut segs = Vec::new();
        extract_pptx_texts(xml, "ppt/slides/slide1.xml", &mut segs).unwrap();
        assert_eq!(segs.len(), 1);
        assert_eq!(segs[0].source_text, "Quarterly Financial Results");

        let segments = vec![
            Segment {
                id: "1".into(),
                file_id: "f".into(),
                seq_number: 1,
                source_text: "Quarterly Financial Results".into(),
                target_text: "النتائج المالية الربع سنوية".into(),
                status: SegmentStatus::Translated,
                match_percent: None,
                match_origin: None,
                locked: false,
                bookmarked: false,
                comment: None,
                created_at: "".into(),
                modified_at: "".into(),
                modified_by: None,
            }
        ];

        let mut idx = 0;
        let replaced = replace_pptx_texts(xml, &segments, &mut idx).unwrap();
        assert!(replaced.contains("النتائج المالية الربع سنوية"));
    }
}
