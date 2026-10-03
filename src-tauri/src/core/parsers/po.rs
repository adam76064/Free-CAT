// core/parsers/po.rs — GNU PO/POT file parser
// Format: https://www.gnu.org/software/gettext/manual/html_node/PO-Files.html

use std::path::Path;
use anyhow::Result;
use crate::core::models::Segment;
use super::RawSegment;

/// Parse a .po or .pot file into segments.
/// Each msgid/msgstr pair = one segment.
pub fn parse(path: &Path) -> Result<Vec<RawSegment>> {
    let content = std::fs::read_to_string(path)?;
    let mut segments = Vec::new();

    let mut current_msgid  = String::new();
    let mut current_msgstr = String::new();
    let mut current_comment = String::new();
    let mut in_msgid  = false;
    let mut in_msgstr = false;

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with('#') {
            // Comment line
            if !in_msgid && !in_msgstr {
                current_comment.push_str(line);
                current_comment.push('\n');
            }
        } else if line.starts_with("msgid ") {
            // Save previous entry
            if !current_msgid.is_empty() {
                segments.push(RawSegment {
                    source_text: unescape_po(&current_msgid),
                    target_text: if current_msgstr.is_empty() {
                        None
                    } else {
                        Some(unescape_po(&current_msgstr))
                    },
                    notes:   if current_comment.is_empty() { None } else { Some(current_comment.clone()) },
                    context: None,
                });
            }
            current_msgid  = extract_po_string(line, "msgid ");
            current_msgstr = String::new();
            current_comment = String::new();
            in_msgid  = true;
            in_msgstr = false;
        } else if line.starts_with("msgstr ") {
            current_msgstr = extract_po_string(line, "msgstr ");
            in_msgid  = false;
            in_msgstr = true;
        } else if line.starts_with('"') {
            // Continuation string
            let s = line.trim_matches('"');
            if in_msgid  { current_msgid.push_str(s); }
            if in_msgstr { current_msgstr.push_str(s); }
        } else if line.is_empty() {
            in_msgid  = false;
            in_msgstr = false;
        }
    }

    // Last entry
    if !current_msgid.is_empty() {
        segments.push(RawSegment {
            source_text: unescape_po(&current_msgid),
            target_text: if current_msgstr.is_empty() { None } else { Some(unescape_po(&current_msgstr)) },
            notes:       None,
            context:     None,
        });
    }

    // Filter out empty msgid (PO header)
    Ok(segments.into_iter().filter(|s| !s.source_text.is_empty()).collect())
}

/// Export to .po format.
pub fn export(segments: &[Segment], out_path: &str) -> Result<()> {
    let mut output = String::new();
    for seg in segments {
        output.push_str(&format!(
            "msgid \"{}\"\nmsgstr \"{}\"\n\n",
            escape_po(&seg.source_text),
            escape_po(&seg.target_text)
        ));
    }
    std::fs::write(out_path, output)?;
    Ok(())
}

fn extract_po_string(line: &str, prefix: &str) -> String {
    let after = &line[prefix.len()..];
    after.trim().trim_matches('"').to_string()
}

fn unescape_po(s: &str) -> String {
    s.replace("\\n", "\n")
     .replace("\\t", "\t")
     .replace("\\\"", "\"")
     .replace("\\\\", "\\")
}

fn escape_po(s: &str) -> String {
    s.replace('\\', "\\\\")
     .replace('"', "\\\"")
     .replace('\n', "\\n")
     .replace('\t', "\\t")
}
