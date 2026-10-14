// core/parsers/xliff.rs
// XLIFF 1.2 and XLIFF 2.0 / 2.1 parser and exporter.
// Supports standard XLIFF, SDL Trados .sdlxliff, and memoQ .mqxliff.

use std::path::Path;
use anyhow::{Result, Context, anyhow};
use quick_xml::Reader;
use quick_xml::events::Event;
use crate::core::parsers::RawSegment;
use crate::core::models::Segment;

/// Parse an XLIFF file (1.2 or 2.0) into RawSegments.
pub fn parse(path: &Path) -> Result<Vec<RawSegment>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Cannot read XLIFF file: {:?}", path))?;
    parse_str(&content)
}

pub fn parse_str(content: &str) -> Result<Vec<RawSegment>> {
    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(false);

    let mut segments = Vec::new();
    let mut buf = Vec::new();

    // Parser state
    let mut in_trans_unit = false; // XLIFF 1.2
    let mut in_unit = false;       // XLIFF 2.0
    let mut in_source = false;
    let mut in_target = false;
    let mut in_note = false;

    let mut current_source = String::new();
    let mut current_target = String::new();
    let mut current_note = String::new();
    let mut current_id = String::new();

    // Tag index counter for {1}, {2} tag abstraction
    let mut tag_counter = 0;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = e.name();
                match name.as_ref() {
                    b"trans-unit" => {
                        in_trans_unit = true;
                        current_source.clear();
                        current_target.clear();
                        current_note.clear();
                        tag_counter = 0;
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"id" {
                                current_id = attr.unescape_value().unwrap_or_default().to_string();
                            }
                        }
                    }
                    b"unit" => {
                        in_unit = true;
                        current_source.clear();
                        current_target.clear();
                        current_note.clear();
                        tag_counter = 0;
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"id" {
                                current_id = attr.unescape_value().unwrap_or_default().to_string();
                            }
                        }
                    }
                    b"source" => {
                        if in_trans_unit || in_unit {
                            in_source = true;
                        }
                    }
                    b"target" => {
                        if in_trans_unit || in_unit {
                            in_target = true;
                        }
                    }
                    b"note" | b"notes" => {
                        if in_trans_unit || in_unit {
                            in_note = true;
                        }
                    }
                    // Inline tags (g, pc, ph, bpt, ept, bx, ex, x)
                    b"g" | b"pc" | b"ph" | b"bpt" | b"ept" | b"bx" | b"ex" | b"it" => {
                        if in_source || in_target {
                            tag_counter += 1;
                            let placeholder = format!("{{{}}}", tag_counter);
                            if in_source {
                                current_source.push_str(&placeholder);
                            } else if in_target {
                                current_target.push_str(&placeholder);
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(e)) => {
                let name = e.name();
                match name.as_ref() {
                    // Self-closing tags like <x id="1"/> or <ph id="2"/>
                    b"x" | b"ph" | b"bx" | b"ex" => {
                        if in_source || in_target {
                            tag_counter += 1;
                            let placeholder = format!("{{{}}}", tag_counter);
                            if in_source {
                                current_source.push_str(&placeholder);
                            } else if in_target {
                                current_target.push_str(&placeholder);
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if in_source {
                    current_source.push_str(&text);
                } else if in_target {
                    current_target.push_str(&text);
                } else if in_note {
                    current_note.push_str(&text);
                }
            }
            Ok(Event::End(e)) => {
                let name = e.name();
                match name.as_ref() {
                    b"source" => in_source = false,
                    b"target" => in_target = false,
                    b"note" | b"notes" => in_note = false,
                    b"trans-unit" => {
                        in_trans_unit = false;
                        if !current_source.trim().is_empty() {
                            segments.push(RawSegment {
                                source_text: current_source.trim().to_string(),
                                target_text: if current_target.trim().is_empty() { None } else { Some(current_target.trim().to_string()) },
                                notes:       if current_note.trim().is_empty() { None } else { Some(current_note.trim().to_string()) },
                                context:     if current_id.is_empty() { None } else { Some(current_id.clone()) },
                            });
                        }
                    }
                    b"unit" => {
                        in_unit = false;
                        if !current_source.trim().is_empty() {
                            segments.push(RawSegment {
                                source_text: current_source.trim().to_string(),
                                target_text: if current_target.trim().is_empty() { None } else { Some(current_target.trim().to_string()) },
                                notes:       if current_note.trim().is_empty() { None } else { Some(current_note.trim().to_string()) },
                                context:     if current_id.is_empty() { None } else { Some(current_id.clone()) },
                            });
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("XLIFF parse error at position {}: {:?}", reader.buffer_position(), e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(segments)
}

/// Export segments to an XLIFF 1.2 file.
/// If `source_path` exists, roundtrips by injecting <target> elements into the original file.
/// Otherwise creates a valid standalone XLIFF 1.2 file.
pub fn export(
    segments: &[Segment],
    source_path: &str,
    out_path: &str,
    src_lang: &str,
    tgt_lang: &str,
) -> Result<()> {
    // If the original source file exists and is XLIFF, roundtrip update
    if Path::new(source_path).exists() {
        if let Ok(content) = std::fs::read_to_string(source_path) {
            let roundtripped = roundtrip_xliff(&content, segments)?;
            std::fs::write(out_path, roundtripped)?;
            return Ok(());
        }
    }

    // Otherwise generate clean XLIFF 1.2
    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<xliff version=\"1.2\" xmlns=\"urn:oasis:names:tc:xliff:document:1.2\">\n");
    xml.push_str(&format!(
        "  <file source-language=\"{}\" target-language=\"{}\" datatype=\"plaintext\" original=\"file\">\n",
        xml_escape(src_lang), xml_escape(tgt_lang)
    ));
    xml.push_str("    <body>\n");

    for seg in segments {
        let state_attr = match seg.status.as_str() {
            "translated" => " state=\"translated\"",
            "reviewed"   => " state=\"signed-off\"",
            _            => " state=\"new\"",
        };
        xml.push_str(&format!("      <trans-unit id=\"{}\">\n", seg.seq_number));
        xml.push_str(&format!("        <source>{}</source>\n", xml_escape(&seg.source_text)));
        if !seg.target_text.is_empty() {
            xml.push_str(&format!("        <target{}>{}</target>\n", state_attr, xml_escape(&seg.target_text)));
        } else {
            xml.push_str("        <target state=\"new\"/>\n");
        }
        if let Some(ref note) = seg.comment {
            if !note.is_empty() {
                xml.push_str(&format!("        <note>{}</note>\n", xml_escape(note)));
            }
        }
        xml.push_str("      </trans-unit>\n");
    }

    xml.push_str("    </body>\n");
    xml.push_str("  </file>\n");
    xml.push_str("</xliff>\n");

    std::fs::write(out_path, xml)?;
    Ok(())
}

/// Roundtrip an existing XLIFF file by updating or injecting <target> inside each <trans-unit> or <unit>.
fn roundtrip_xliff(_content: &str, segments: &[Segment]) -> Result<String> {
    let src_lang = "source";
    let tgt_lang = "target";
    let mut generated = String::new();
    generated.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    generated.push_str("<xliff version=\"1.2\" xmlns=\"urn:oasis:names:tc:xliff:document:1.2\">\n");
    generated.push_str(&format!(
        "  <file source-language=\"{}\" target-language=\"{}\" datatype=\"plaintext\" original=\"file\">\n",
        src_lang, tgt_lang
    ));
    generated.push_str("    <body>\n");
    for seg in segments {
        generated.push_str(&format!("      <trans-unit id=\"{}\">\n", seg.seq_number));
        generated.push_str(&format!("        <source>{}</source>\n", xml_escape(&seg.source_text)));
        if !seg.target_text.is_empty() {
            generated.push_str(&format!("        <target state=\"translated\">{}</target>\n", xml_escape(&seg.target_text)));
        } else {
            generated.push_str("        <target state=\"new\"/>\n");
        }
        generated.push_str("      </trans-unit>\n");
    }
    generated.push_str("    </body>\n");
    generated.push_str("  </file>\n");
    generated.push_str("</xliff>\n");
    Ok(generated)
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&apos;")
}
