// core/parsers/tbx.rs
// TBX-Basic (ISO 30042) parser and exporter for termbases.

use std::path::Path;
use anyhow::{Result, Context, anyhow};
use quick_xml::Reader;
use quick_xml::events::Event;
use crate::core::models::TermEntry;

/// Parse a TBX-Basic file and return all extracted term entries.
pub fn parse_tbx(path: &Path) -> Result<Vec<TermEntry>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Cannot read TBX file: {:?}", path))?;
    parse_tbx_str(&content)
}

pub fn parse_tbx_str(content: &str) -> Result<Vec<TermEntry>> {
    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);

    let mut entries = Vec::new();
    let mut buf = Vec::new();

    let mut in_term_entry = false;
    let mut in_lang_set = false;
    let mut in_term = false;
    let mut in_descrip = false;
    let mut in_note = false;

    let mut current_lang = String::new();
    let mut current_term = String::new();
    let mut current_def = String::new();
    let mut current_note = String::new();
    let mut current_status = "preferred".to_string();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name().as_ref() {
                    b"termEntry" => {
                        in_term_entry = true;
                        current_def.clear();
                    }
                    b"langSet" => {
                        if in_term_entry {
                            in_lang_set = true;
                            current_lang.clear();
                            for attr in e.attributes().flatten() {
                                if attr.key.as_ref() == b"xml:lang" || attr.key.as_ref() == b"lang" {
                                    current_lang = attr.unescape_value().unwrap_or_default().to_string();
                                }
                            }
                        }
                    }
                    b"term" => {
                        if in_lang_set {
                            in_term = true;
                            current_term.clear();
                            current_status = "preferred".to_string();
                            current_note.clear();
                        }
                    }
                    b"descrip" => {
                        in_descrip = true;
                    }
                    b"termNote" | b"note" => {
                        in_note = true;
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if in_term {
                    current_term.push_str(&text);
                } else if in_descrip {
                    current_def.push_str(&text);
                } else if in_note {
                    current_note.push_str(&text);
                    if text.eq_ignore_ascii_case("forbidden") || text.eq_ignore_ascii_case("admitted") {
                        current_status = text.to_lowercase();
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                match e.name().as_ref() {
                    b"term" => {
                        in_term = false;
                        if !current_term.is_empty() && !current_lang.is_empty() {
                            entries.push(TermEntry {
                                id:         None,
                                lang:       current_lang.clone(),
                                term:       current_term.clone(),
                                definition: if current_def.is_empty() { None } else { Some(current_def.clone()) },
                                domain:     None,
                                note:       if current_note.is_empty() { None } else { Some(current_note.clone()) },
                                status:     current_status.clone(),
                                created_at: chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                                metadata:   None,
                            });
                        }
                    }
                    b"descrip" => in_descrip = false,
                    b"termNote" | b"note" => in_note = false,
                    b"langSet" => in_lang_set = false,
                    b"termEntry" => in_term_entry = false,
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(anyhow!("TBX parse error: {:?}", e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(entries)
}

/// Export term entries to a TBX-Basic XML file.
pub fn export_tbx(entries: &[TermEntry], out_path: &str) -> Result<()> {
    let mut xml = String::new();
    xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<martif type=\"TBX-Basic\" xml:lang=\"en\">\n");
    xml.push_str("  <martifHeader>\n");
    xml.push_str("    <fileDesc>\n");
    xml.push_str("      <sourceDesc><p>Exported from Free-CAT</p></sourceDesc>\n");
    xml.push_str("    </fileDesc>\n");
    xml.push_str("  </martifHeader>\n");
    xml.push_str("  <text>\n");
    xml.push_str("    <body>\n");

    for (i, entry) in entries.iter().enumerate() {
        xml.push_str(&format!("      <termEntry id=\"eid_{}\">\n", i + 1));
        if let Some(ref def) = entry.definition {
            xml.push_str(&format!("        <descrip type=\"definition\">{}</descrip>\n", xml_escape(def)));
        }
        xml.push_str(&format!("        <langSet xml:lang=\"{}\">\n", xml_escape(&entry.lang)));
        xml.push_str("          <tig>\n");
        xml.push_str(&format!("            <term>{}</term>\n", xml_escape(&entry.term)));
        xml.push_str(&format!("            <termNote type=\"termType\">{}</termNote>\n", xml_escape(&entry.status)));
        if let Some(ref note) = entry.note {
            xml.push_str(&format!("            <note>{}</note>\n", xml_escape(note)));
        }
        xml.push_str("          </tig>\n");
        xml.push_str("        </langSet>\n");
        xml.push_str("      </termEntry>\n");
    }

    xml.push_str("    </body>\n");
    xml.push_str("  </text>\n");
    xml.push_str("</martif>\n");

    std::fs::write(out_path, xml)?;
    Ok(())
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&apos;")
}
