// core/parsers/mod.rs
// Format detection and parser dispatch.

use std::path::Path;
use anyhow::{Result, anyhow};
use crate::core::models::{ProjectFile, Segment};

pub mod tmx;
pub mod txt;
pub mod po;

/// Raw segment data returned by any parser.
#[derive(Debug, Clone)]
pub struct RawSegment {
    pub source_text: String,
    pub target_text: Option<String>, // Some for bilingual formats, None for source-only
    pub notes:       Option<String>,
    pub context:     Option<String>,
}

/// Detect file format from extension.
pub fn detect_format(path: &Path) -> Option<String> {
    match path.extension()?.to_str()?.to_lowercase().as_str() {
        "xliff" | "xlf"  => Some("xliff".into()),
        "sdlxliff"        => Some("sdlxliff".into()),
        "mqxliff"         => Some("mqxliff".into()),
        "tmx"             => Some("tmx".into()),
        "docx"            => Some("docx".into()),
        "xlsx"            => Some("xlsx".into()),
        "pptx"            => Some("pptx".into()),
        "html" | "htm"    => Some("html".into()),
        "po" | "pot"      => Some("po".into()),
        "txt"             => Some("txt".into()),
        "srt"             => Some("srt".into()),
        "vtt"             => Some("vtt".into()),
        "json"            => Some("json".into()),
        "yaml" | "yml"    => Some("yaml".into()),
        "md"              => Some("md".into()),
        _                 => None,
    }
}

/// Dispatch file parsing to the correct parser based on format string.
pub fn parse_file(path: &Path, format: &str) -> Result<Vec<RawSegment>> {
    match format {
        "txt"             => txt::parse(path),
        "po"              => po::parse(path),
        "xliff" | "sdlxliff" | "mqxliff" => {
            // TODO: XLIFF parser (Phase 1 completion)
            log::warn!("XLIFF parser stub — returning empty segments");
            Ok(vec![])
        }
        "docx"            => {
            // TODO: DOCX parser (Phase 1 completion)
            log::warn!("DOCX parser stub — returning placeholder");
            Ok(vec![RawSegment {
                source_text: "DOCX parser not yet implemented".into(),
                target_text: None, notes: None, context: None,
            }])
        }
        _ => Err(anyhow!("No parser for format: {}", format)),
    }
}

/// Export translated segments back to a file.
pub fn export_file(file: &ProjectFile, segments: &[Segment], out_path: &str) -> Result<()> {
    match file.format.as_str() {
        "txt" => txt::export(segments, out_path),
        "po"  => po::export(segments, out_path),
        _ => {
            log::warn!("Export not yet implemented for format: {}", file.format);
            Err(anyhow!("Export for '{}' not yet implemented", file.format))
        }
    }
}
