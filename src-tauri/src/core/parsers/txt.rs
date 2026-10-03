// core/parsers/txt.rs — Plain text file parser (one segment per paragraph)

use std::path::Path;
use anyhow::Result;
use crate::core::models::Segment;
use super::RawSegment;

/// Parse a plain text file into segments (one per non-empty paragraph).
pub fn parse(path: &Path) -> Result<Vec<RawSegment>> {
    let content = std::fs::read_to_string(path)?;
    let segments = content
        .split("\n\n")  // double-newline = paragraph boundary
        .map(|s| s.trim().replace('\n', " "))
        .filter(|s| !s.is_empty())
        .map(|s| RawSegment {
            source_text: s,
            target_text: None,
            notes:       None,
            context:     None,
        })
        .collect();
    Ok(segments)
}

/// Export translated segments to a plain text file (one per line, double-newline separated).
pub fn export(segments: &[Segment], out_path: &str) -> Result<()> {
    let content: String = segments
        .iter()
        .map(|s| s.target_text.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");
    std::fs::write(out_path, content)?;
    Ok(())
}
