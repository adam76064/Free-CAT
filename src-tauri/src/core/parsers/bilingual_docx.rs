// core/parsers/bilingual_docx.rs — Bilingual Word (.docx) Review & Retrofit Import
// Generates standard 2/4-column Word table for external reviewers and retrofits edits back into CAT segments.

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use anyhow::{Result, Context, anyhow};
use quick_xml::events::Event;
use quick_xml::Reader;
use zip::read::ZipArchive;
use zip::write::{SimpleFileOptions, ZipWriter};

use crate::core::models::Segment;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BilingualRetrofitRow {
    pub seq_number: i64,
    pub source_text: String,
    pub target_text: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BilingualRetrofitReport {
    pub total_rows: usize,
    pub updated_count: usize,
    pub unchanged_count: usize,
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Export project segments to a standalone Bilingual Review Word (.docx) document.
pub fn export_bilingual_docx(segments: &[Segment], document_title: &str, out_path: &Path) -> Result<()> {
    let out_file = File::create(out_path)
        .with_context(|| format!("Failed to create bilingual review DOCX: {:?}", out_path))?;
    let mut zip = ZipWriter::new(out_file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // 1. [Content_Types].xml
    let content_types = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#;
    zip.start_file("[Content_Types].xml", options)?;
    zip.write_all(content_types.as_bytes())?;

    // 2. _rels/.rels
    let root_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#;
    zip.start_file("_rels/.rels", options)?;
    zip.write_all(root_rels.as_bytes())?;

    // 3. word/_rels/document.xml.rels
    let doc_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
</Relationships>"#;
    zip.start_file("word/_rels/document.xml.rels", options)?;
    zip.write_all(doc_rels.as_bytes())?;

    // 4. word/document.xml with 4-column review table
    let mut doc_xml = String::with_capacity(16384);
    doc_xml.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p>
      <w:pPr>
        <w:jc w:val="center"/>
      </w:pPr>
      <w:r>
        <w:rPr>
          <w:b/>
          <w:sz w:val="32"/>
          <w:color w:val="2B579A"/>
        </w:rPr>
        <w:t>Free-CAT Bilingual Review: "#);
    doc_xml.push_str(&escape_xml(document_title));
    doc_xml.push_str(r#"</w:t>
      </w:r>
    </w:p>
    <w:p>
      <w:pPr>
        <w:jc w:val="center"/>
      </w:pPr>
      <w:r>
        <w:rPr>
          <w:i/>
          <w:sz w:val="18"/>
          <w:color w:val="666666"/>
        </w:rPr>
        <w:t>Instruction for Reviewer: Edit the 'Target Translation' column or use Track Changes. Do not modify ID numbers.</w:t>
      </w:r>
    </w:p>
    <w:tbl>
      <w:tblPr>
        <w:tblW w:w="0" w:type="auto"/>
        <w:tblBorders>
          <w:top w:val="single" w:sz="6" w:space="0" w:color="2B579A"/>
          <w:bottom w:val="single" w:sz="6" w:space="0" w:color="2B579A"/>
          <w:insideH w:val="single" w:sz="4" w:space="0" w:color="D9D9D9"/>
          <w:insideV w:val="single" w:sz="4" w:space="0" w:color="D9D9D9"/>
        </w:tblBorders>
      </w:tblPr>
      <w:tr>
        <w:tc>
          <w:tcPr><w:shd w:val="clear" w:color="auto" w:fill="F2F4F7"/><w:tcW w:w="800" w:type="dxa"/></w:tcPr>
          <w:p><w:pPr><w:jc w:val="center"/></w:pPr><w:r><w:rPr><w:b/><w:sz w:val="20"/></w:rPr><w:t>#</w:t></w:r></w:p>
        </w:tc>
        <w:tc>
          <w:tcPr><w:shd w:val="clear" w:color="auto" w:fill="F2F4F7"/><w:tcW w:w="4200" w:type="dxa"/></w:tcPr>
          <w:p><w:r><w:rPr><w:b/><w:sz w:val="20"/></w:rPr><w:t>Source Text</w:t></w:r></w:p>
        </w:tc>
        <w:tc>
          <w:tcPr><w:shd w:val="clear" w:color="auto" w:fill="F2F4F7"/><w:tcW w:w="4200" w:type="dxa"/></w:tcPr>
          <w:p><w:r><w:rPr><w:b/><w:sz w:val="20"/></w:rPr><w:t>Target Translation (Edit Here)</w:t></w:r></w:p>
        </w:tc>
        <w:tc>
          <w:tcPr><w:shd w:val="clear" w:color="auto" w:fill="F2F4F7"/><w:tcW w:w="1200" w:type="dxa"/></w:tcPr>
          <w:p><w:r><w:rPr><w:b/><w:sz w:val="20"/></w:rPr><w:t>Status / Notes</w:t></w:r></w:p>
        </w:tc>
      </w:tr>"#);

    for seg in segments {
        let status_str = match seg.status {
            crate::core::models::SegmentStatus::New => "New",
            crate::core::models::SegmentStatus::Draft => "Draft",
            crate::core::models::SegmentStatus::Translated => "Translated",
            crate::core::models::SegmentStatus::NeedsReview => "Needs Review",
            crate::core::models::SegmentStatus::Reviewed => "Reviewed",
            crate::core::models::SegmentStatus::Final => "Final",
        };

        doc_xml.push_str("<w:tr>");
        
        // Col 1: ID
        doc_xml.push_str("<w:tc><w:p><w:pPr><w:jc w:val=\"center\"/></w:pPr><w:r><w:rPr><w:color w:val=\"888888\"/><w:sz w:val=\"18\"/></w:rPr><w:t>");
        doc_xml.push_str(&seg.seq_number.to_string());
        doc_xml.push_str("</w:t></w:r></w:p></w:tc>");

        // Col 2: Source Text
        doc_xml.push_str("<w:tc><w:p><w:r><w:rPr><w:sz w:val=\"20\"/></w:rPr><w:t xml:space=\"preserve\">");
        doc_xml.push_str(&escape_xml(&seg.source_text));
        doc_xml.push_str("</w:t></w:r></w:p></w:tc>");

        // Col 3: Target Text
        doc_xml.push_str("<w:tc><w:p><w:r><w:rPr><w:sz w:val=\"20\"/></w:rPr><w:t xml:space=\"preserve\">");
        doc_xml.push_str(&escape_xml(&seg.target_text));
        doc_xml.push_str("</w:t></w:r></w:p></w:tc>");

        // Col 4: Status / Notes
        doc_xml.push_str("<w:tc><w:p><w:r><w:rPr><w:sz w:val=\"16\"/><w:color w:val=\"666666\"/></w:rPr><w:t>");
        doc_xml.push_str(status_str);
        if let Some(ref note) = seg.comment {
            doc_xml.push_str(" | ");
            doc_xml.push_str(&escape_xml(note));
        }
        doc_xml.push_str("</w:t></w:r></w:p></w:tc>");

        doc_xml.push_str("</w:tr>");
    }

    doc_xml.push_str(r#"    </w:tbl>
  </w:body>
</w:document>"#);

    zip.start_file("word/document.xml", options)?;
    zip.write_all(doc_xml.as_bytes())?;
    zip.finish()?;

    Ok(())
}

/// Import an edited Bilingual Review Word (.docx) document (Retrofit import).
pub fn import_bilingual_docx_retrofit(docx_path: &Path) -> Result<Vec<BilingualRetrofitRow>> {
    let file = File::open(docx_path)
        .with_context(|| format!("Failed to open bilingual review DOCX: {:?}", docx_path))?;
    let mut archive = ZipArchive::new(file)
        .with_context(|| "Failed to read DOCX ZIP archive")?;

    let mut doc_xml = String::new();
    {
        let mut doc_file = archive.by_name("word/document.xml")
            .with_context(|| "word/document.xml not found in DOCX archive")?;
        doc_file.read_to_string(&mut doc_xml)?;
    }

    let mut reader = Reader::from_str(&doc_xml);
    reader.config_mut().trim_text(false);

    let mut rows: Vec<BilingualRetrofitRow> = Vec::new();
    let mut buf = Vec::new();

    let mut in_tr = false;
    let mut in_tc = false;
    let mut in_t = false;
    let mut current_cell_text = String::new();
    let mut current_row_cells: Vec<String> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name().as_ref() {
                b"w:tr" => {
                    in_tr = true;
                    current_row_cells.clear();
                }
                b"w:tc" => {
                    if in_tr {
                        in_tc = true;
                        current_cell_text.clear();
                    }
                }
                b"w:t" => {
                    if in_tc {
                        in_t = true;
                    }
                }
                _ => {}
            },
            Ok(Event::Text(ref e)) => {
                if in_t {
                    let text = e.unescape().unwrap_or_default();
                    current_cell_text.push_str(&text);
                }
            }
            Ok(Event::End(ref e)) => match e.name().as_ref() {
                b"w:t" => {
                    in_t = false;
                }
                b"w:tc" => {
                    in_tc = false;
                    current_row_cells.push(current_cell_text.trim().to_string());
                    current_cell_text.clear();
                }
                b"w:tr" => {
                    in_tr = false;
                    // We expect at least 3 cells: #, Source, Target
                    if current_row_cells.len() >= 3 {
                        let id_str = &current_row_cells[0];
                        // If cell 0 parses as an integer sequence number, it's a data row!
                        if let Ok(seq) = id_str.parse::<i64>() {
                            let source = current_row_cells[1].clone();
                            let target = current_row_cells[2].clone();
                            let notes = if current_row_cells.len() > 3 && !current_row_cells[3].is_empty() {
                                Some(current_row_cells[3].clone())
                            } else {
                                None
                            };

                            rows.push(BilingualRetrofitRow {
                                seq_number: seq,
                                source_text: source,
                                target_text: target,
                                notes,
                            });
                        }
                    }
                    current_row_cells.clear();
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("XML parse error in bilingual document.xml: {}", e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bilingual_docx_export_and_retrofit_roundtrip() {
        let temp_dir = std::env::temp_dir();
        let docx_path = temp_dir.join(format!("review_test_{}.docx", uuid::Uuid::new_v4()));

        let segments = vec![
            Segment {
                id: "seg-1".into(),
                file_id: "file-1".into(),
                seq_number: 1,
                source_text: "Welcome to Free-CAT.".into(),
                target_text: "Bienvenue sur Free-CAT.".into(),
                status: crate::core::models::SegmentStatus::Translated,
                match_percent: Some(100),
                match_origin: Some("TM".into()),
                locked: false,
                bookmarked: false,
                comment: None,
                created_at: "2026-01-01T00:00:00Z".into(),
                modified_at: "2026-01-01T00:00:00Z".into(),
                modified_by: None,
            },
            Segment {
                id: "seg-2".into(),
                file_id: "file-1".into(),
                seq_number: 2,
                source_text: "Fast & lightweight computer-assisted translation.".into(),
                target_text: "Traduction assistée par ordinateur rapide & légère.".into(),
                status: crate::core::models::SegmentStatus::Draft,
                match_percent: None,
                match_origin: None,
                locked: false,
                bookmarked: true,
                comment: Some("Check technical term".into()),
                created_at: "2026-01-01T00:00:00Z".into(),
                modified_at: "2026-01-01T00:00:00Z".into(),
                modified_by: None,
            },
        ];

        // 1. Export
        export_bilingual_docx(&segments, "Test Document", &docx_path).unwrap();
        assert!(docx_path.exists());

        // 2. Retrofit import
        let parsed = import_bilingual_docx_retrofit(&docx_path).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].seq_number, 1);
        assert_eq!(parsed[0].source_text, "Welcome to Free-CAT.");
        assert_eq!(parsed[0].target_text, "Bienvenue sur Free-CAT.");

        assert_eq!(parsed[1].seq_number, 2);
        assert_eq!(parsed[1].source_text, "Fast & lightweight computer-assisted translation.");
        assert_eq!(parsed[1].target_text, "Traduction assistée par ordinateur rapide & légère.");
        assert!(parsed[1].notes.as_ref().unwrap().contains("Check technical term"));
    }
}
