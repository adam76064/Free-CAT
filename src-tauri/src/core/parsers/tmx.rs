// core/parsers/tmx.rs
// TMX 1.4b parser and writer.
// Standard: https://ttt.org/oscarstandards/tmx/tmx14b.html

use anyhow::{Result, Context};
use quick_xml::events::Event;
use quick_xml::Reader;
use chrono::Utc;

use crate::core::models::TranslationUnit;

/// Parse a TMX 1.4b file and return all translation units.
pub fn parse_tmx_file(path: &str) -> Result<Vec<TranslationUnit>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Cannot read TMX file: {}", path))?;

    parse_tmx_str(&content)
}

pub fn parse_tmx_str(content: &str) -> Result<Vec<TranslationUnit>> {
    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);

    let mut units: Vec<TranslationUnit> = Vec::new();
    let mut buf = Vec::new();

    // Parser state
    let mut in_tu = false;
    let mut in_tuv = false;
    let mut in_seg = false;
    let mut current_lang = String::new();
    let mut current_src = String::new();
    let mut current_tgt = String::new();
    let mut src_lang = String::new();
    let mut tgt_lang = String::new();
    let mut tu_created_at = String::new();
    let mut tu_changed_at = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"tu" => {
                    in_tu = true;
                    current_src.clear();
                    current_tgt.clear();
                    // Read attributes: creationdate, changedate
                    for attr in e.attributes().flatten() {
                        let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("").to_string();
                        let val = attr.unescape_value().unwrap_or_default().to_string();
                        match key.as_str() {
                            "creationdate" => tu_created_at = val,
                            "changedate"   => tu_changed_at = val,
                            _ => {}
                        }
                    }
                }
                b"tuv" => {
                    in_tuv = true;
                    // Read xml:lang attribute
                    for attr in e.attributes().flatten() {
                        let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("").to_string();
                        if key == "xml:lang" || key.ends_with(":lang") {
                            current_lang = attr.unescape_value().unwrap_or_default().to_string();
                            if !src_lang.is_empty() && !current_lang.eq_ignore_ascii_case(&src_lang) && tgt_lang.is_empty() {
                                tgt_lang = current_lang.clone();
                            }
                        }
                    }
                }
                b"seg" => {
                    if in_tuv { in_seg = true; }
                }
                b"header" => {
                    // Extract srclang from header
                    for attr in e.attributes().flatten() {
                        let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("").to_string();
                        if key == "srclang" {
                            src_lang = attr.unescape_value().unwrap_or_default().to_string();
                        }
                    }
                }
                _ => {}
            },
            Ok(Event::Text(e)) => {
                if in_seg {
                    let text = e.unescape().unwrap_or_default().to_string();
                    // Assign to src or tgt based on lang
                    if src_lang.is_empty() || current_lang.to_lowercase().starts_with(&src_lang.to_lowercase()[..2]) {
                        if current_src.is_empty() {
                            current_src = text;
                        } else {
                            current_tgt = text;
                        }
                    } else {
                        if current_src.is_empty() {
                            current_src = text;
                        } else {
                            current_tgt = text;
                        }
                    }
                }
            }
            Ok(Event::End(e)) => match e.name().as_ref() {
                b"seg" => { in_seg = false; }
                b"tuv" => {
                    // When closing a tuv, determine if this was src or tgt
                    in_tuv = false;
                }
                b"tu" => {
                    if in_tu && !current_src.is_empty() && !current_tgt.is_empty() {
                        let now = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
                        units.push(TranslationUnit {
                            id:          None,
                            source_lang: src_lang.clone(),
                            target_lang: tgt_lang.clone(),
                            source_text: current_src.clone(),
                            target_text: current_tgt.clone(),
                            created_at:  if tu_created_at.is_empty() { now.clone() } else { tu_created_at.clone() },
                            modified_at: if tu_changed_at.is_empty() { now } else { tu_changed_at.clone() },
                            created_by:  None,
                            client:      None,
                            domain:      None,
                            source_file: None,
                            usage_count: Some(0),
                            metadata:    None,
                        });
                    }
                    in_tu = false;
                    current_src.clear();
                    current_tgt.clear();
                    tu_created_at.clear();
                    tu_changed_at.clear();
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(e) => {
                log::warn!("TMX parse error: {}", e);
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    log::info!("Parsed {} TUs from TMX", units.len());
    Ok(units)
}

/// Write translation units to a TMX 1.4b file.
pub fn write_tmx_file(
    units: &[TranslationUnit],
    src_lang: &str,
    tgt_lang: &str,
    path: &str,
) -> Result<()> {
    let content = write_tmx_str(units, src_lang, tgt_lang)?;
    std::fs::write(path, content)
        .with_context(|| format!("Cannot write TMX to: {}", path))?;
    Ok(())
}

pub fn write_tmx_str(
    units: &[TranslationUnit],
    src_lang: &str,
    tgt_lang: &str,
) -> Result<String> {
    let now = Utc::now().format("%Y%m%dT%H%M%SZ").to_string();

    let mut output = String::new();
    output.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    output.push('\n');
    output.push_str(r#"<!DOCTYPE tmx SYSTEM "tmx14.dtd">"#);
    output.push('\n');
    output.push_str(r#"<tmx version="1.4">"#);
    output.push('\n');

    // Header
    output.push_str(&format!(
        r#"  <header creationtool="Free-CAT" creationtoolversion="0.1.0" datatype="PlainText" segtype="sentence" adminlang="EN-US" srclang="{}" creationdate="{}" />"#,
        src_lang, now
    ));
    output.push('\n');

    output.push_str("  <body>\n");

    for tu in units {
        let created = &tu.created_at;
        let changed = &tu.modified_at;
        output.push_str(&format!(
            r#"    <tu creationdate="{}" changedate="{}">"#,
            created, changed
        ));
        output.push('\n');

        // Source TUV
        output.push_str(&format!(
            r#"      <tuv xml:lang="{}"><seg>{}</seg></tuv>"#,
            src_lang,
            xml_escape(&tu.source_text)
        ));
        output.push('\n');

        // Target TUV
        output.push_str(&format!(
            r#"      <tuv xml:lang="{}"><seg>{}</seg></tuv>"#,
            tgt_lang,
            xml_escape(&tu.target_text)
        ));
        output.push('\n');

        output.push_str("    </tu>\n");
    }

    output.push_str("  </body>\n");
    output.push_str("</tmx>\n");

    Ok(output)
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_TMX: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<tmx version="1.4">
  <header srclang="EN" />
  <body>
    <tu>
      <tuv xml:lang="EN"><seg>Hello world</seg></tuv>
      <tuv xml:lang="FR"><seg>Bonjour monde</seg></tuv>
    </tu>
    <tu>
      <tuv xml:lang="EN"><seg>The document has been translated.</seg></tuv>
      <tuv xml:lang="FR"><seg>Le document a été traduit.</seg></tuv>
    </tu>
  </body>
</tmx>"#;

    #[test]
    fn parse_sample_tmx() {
        let units = parse_tmx_str(SAMPLE_TMX).unwrap();
        assert_eq!(units.len(), 2);
        assert!(units[0].source_text.contains("Hello") || units[0].target_text.contains("Hello"));
    }
}
