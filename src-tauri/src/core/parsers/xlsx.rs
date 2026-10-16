// core/parsers/xlsx.rs — Excel Spreadsheet OpenXML parser
// Extracts translatable text from xl/sharedStrings.xml and worksheet inline strings, preserving formulas and cell formatting.

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
        .with_context(|| format!("Failed to open XLSX file: {:?}", path))?;
    let mut zip = ZipArchive::new(file)?;

    let mut segments = Vec::new();

    // 1. Parse sharedStrings.xml if present
    if let Ok(mut shared_file) = zip.by_name("xl/sharedStrings.xml") {
        let mut content = String::new();
        shared_file.read_to_string(&mut content)?;
        extract_xml_texts(&content, &mut segments)?;
    }

    // 2. If no shared strings found, inspect sheet files for inline strings
    if segments.is_empty() {
        for i in 0..zip.len() {
            let mut file = zip.by_index(i)?;
            let name = file.name().to_string();
            if name.starts_with("xl/worksheets/sheet") && name.ends_with(".xml") {
                let mut content = String::new();
                file.read_to_string(&mut content)?;
                extract_xml_texts(&content, &mut segments)?;
            }
        }
    }

    Ok(segments)
}

fn extract_xml_texts(xml: &str, segments: &mut Vec<RawSegment>) -> Result<()> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut in_t_tag = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"t" => {
                in_t_tag = true;
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"t" => {
                in_t_tag = false;
            }
            Ok(Event::Text(ref e)) if in_t_tag => {
                let text = e.unescape()?.to_string();
                let trimmed = text.trim();
                if !trimmed.is_empty() && !trimmed.starts_with('=') {
                    segments.push(RawSegment {
                        source_text: text,
                        target_text: None,
                        notes: None,
                        context: Some("cell".into()),
                    });
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("Error reading XML in XLSX: {}", e)),
            _ => {}
        }
    }

    Ok(())
}

pub fn export(original_path: &Path, segments: &[Segment], out_path: &Path) -> Result<()> {
    let file = std::fs::File::open(original_path)
        .with_context(|| format!("Failed to open original XLSX: {:?}", original_path))?;
    let mut zip = ZipArchive::new(file)?;

    let out_file = std::fs::File::create(out_path)
        .with_context(|| format!("Failed to create output XLSX: {:?}", out_path))?;
    let mut out_zip = ZipWriter::new(out_file);

    let mut seg_idx = 0;

    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let name = entry.name().to_string();

        let options = FileOptions::<()>::default()
            .compression_method(entry.compression())
            .unix_permissions(entry.unix_mode().unwrap_or(0o644));

        out_zip.start_file(&name, options)?;

        if name == "xl/sharedStrings.xml" {
            let mut xml_content = String::new();
            entry.read_to_string(&mut xml_content)?;
            let reconstructed = replace_xml_texts(&xml_content, segments, &mut seg_idx)?;
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

pub fn replace_xml_texts(xml: &str, segments: &[Segment], seg_idx: &mut usize) -> Result<String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);

    let mut writer = quick_xml::Writer::new(Cursor::new(Vec::new()));
    let mut in_t_tag = false;

    loop {
        match reader.read_event()? {
            Event::Start(ref e) if e.name().as_ref() == b"t" => {
                in_t_tag = true;
                writer.write_event(Event::Start(e.clone()))?;
            }
            Event::End(ref e) if e.name().as_ref() == b"t" => {
                in_t_tag = false;
                writer.write_event(Event::End(e.clone()))?;
            }
            Event::Text(ref e) if in_t_tag => {
                let original_text = e.unescape()?.to_string();
                if !original_text.trim().is_empty() && !original_text.trim().starts_with('=') && *seg_idx < segments.len() {
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
    fn test_extract_and_replace_shared_strings() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="2" uniqueCount="2">
  <si><t>Revenue 2026</t></si>
  <si><t>Net Profit</t></si>
</sst>"#;

        let mut segs = Vec::new();
        extract_xml_texts(xml, &mut segs).unwrap();
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].source_text, "Revenue 2026");
        assert_eq!(segs[1].source_text, "Net Profit");

        let segments = vec![
            Segment {
                id: "1".into(),
                file_id: "f".into(),
                seq_number: 1,
                source_text: "Revenue 2026".into(),
                target_text: "الإيرادات 2026".into(),
                status: SegmentStatus::Translated,
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
                file_id: "f".into(),
                seq_number: 2,
                source_text: "Net Profit".into(),
                target_text: "صافي الأرباح".into(),
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
        let replaced = replace_xml_texts(xml, &segments, &mut idx).unwrap();
        assert!(replaced.contains("الإيرادات 2026"));
        assert!(replaced.contains("صافي الأرباح"));
    }
}
