// core/parsers/srt.rs — SubRip (.srt) and WebVTT (.vtt) subtitle parser and exporter

use std::path::Path;
use anyhow::{Result, Context};
use crate::core::models::Segment;
use super::RawSegment;

#[derive(Debug, Clone)]
struct SubtitleBlock {
    index: String,
    timestamp: String,
    text: String,
}

/// Parse an SRT or VTT file into subtitle segments.
pub fn parse(path: &Path) -> Result<Vec<RawSegment>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read subtitle file: {:?}", path))?;
    parse_subtitle_str(&content)
}

pub fn parse_subtitle_str(content: &str) -> Result<Vec<RawSegment>> {
    let blocks = parse_blocks(content);
    let mut segments = Vec::new();

    for block in blocks {
        if block.text.trim().is_empty() {
            continue;
        }
        segments.push(RawSegment {
            source_text: block.text.trim().to_string(),
            target_text: None,
            notes: Some(format!("Time: {}", block.timestamp)),
            context: Some(format!("cue_{}", block.index)),
        });
    }

    Ok(segments)
}

/// Export translated segments back into an SRT or VTT file.
pub fn export(segments: &[Segment], source_path: &str, out_path: &str) -> Result<()> {
    let content = std::fs::read_to_string(source_path)
        .with_context(|| format!("Failed to read source subtitle file: {}", source_path))?;
    let mut blocks = parse_blocks(&content);
    let is_vtt = source_path.ends_with(".vtt") || out_path.ends_with(".vtt");

    let mut output = String::new();
    if is_vtt {
        output.push_str("WEBVTT\n\n");
    }

    let mut seg_iter = segments.iter();

    for block in &mut blocks {
        let translated = if let Some(seg) = seg_iter.next() {
            if !seg.target_text.trim().is_empty() {
                seg.target_text.as_str()
            } else {
                seg.source_text.as_str()
            }
        } else {
            block.text.as_str()
        };

        if !block.index.is_empty() {
            output.push_str(&block.index);
            output.push('\n');
        }
        output.push_str(&block.timestamp);
        output.push('\n');
        output.push_str(translated);
        output.push_str("\n\n");
    }

    std::fs::write(out_path, output.trim_end().to_string() + "\n")
        .with_context(|| format!("Failed to write translated subtitle file: {}", out_path))?;
    Ok(())
}

fn parse_blocks(content: &str) -> Vec<SubtitleBlock> {
    let normalized = content.replace("\r\n", "\n").replace('\r', "\n");
    let raw_blocks = normalized.split("\n\n");
    let mut blocks = Vec::new();

    for raw in raw_blocks {
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed == "WEBVTT" {
            continue;
        }

        let lines: Vec<&str> = trimmed.lines().collect();
        if lines.is_empty() {
            continue;
        }

        // Determine if first line is cue index or timestamp line
        if lines[0].contains("-->") {
            let timestamp = lines[0].to_string();
            let text = lines[1..].join("\n");
            blocks.push(SubtitleBlock {
                index: String::new(),
                timestamp,
                text,
            });
        } else if lines.len() >= 2 && lines[1].contains("-->") {
            let index = lines[0].to_string();
            let timestamp = lines[1].to_string();
            let text = lines[2..].join("\n");
            blocks.push(SubtitleBlock {
                index,
                timestamp,
                text,
            });
        }
    }

    blocks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{Segment, SegmentStatus};

    const SAMPLE_SRT: &str = r#"1
00:00:01,000 --> 00:00:04,000
Welcome to Free-CAT.

2
00:00:05,000 --> 00:00:09,000
The professional open source CAT tool."#;

    #[test]
    fn test_parse_srt() {
        let segments = parse_subtitle_str(SAMPLE_SRT).unwrap();
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].source_text, "Welcome to Free-CAT.");
        assert_eq!(segments[1].source_text, "The professional open source CAT tool.");
    }

    #[test]
    fn test_export_srt() {
        let _segments = vec![
            Segment {
                id: "1".into(), file_id: "f".into(), seq_number: 1,
                source_text: "Welcome to Free-CAT.".into(),
                target_text: "مرحباً بكم في Free-CAT.".into(),
                status: SegmentStatus::Final, match_percent: None, match_origin: None,
                locked: false, bookmarked: false, comment: None, created_at: "".into(),
                modified_at: "".into(), modified_by: None,
            },
            Segment {
                id: "2".into(), file_id: "f".into(), seq_number: 2,
                source_text: "The professional open source CAT tool.".into(),
                target_text: "أداة الترجمة الاحترافية مفتوحة المصدر.".into(),
                status: SegmentStatus::Final, match_percent: None, match_origin: None,
                locked: false, bookmarked: false, comment: None, created_at: "".into(),
                modified_at: "".into(), modified_by: None,
            },
        ];

        let out = parse_blocks(SAMPLE_SRT);
        assert_eq!(out.len(), 2);
    }
}
