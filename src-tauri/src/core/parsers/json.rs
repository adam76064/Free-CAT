// core/parsers/json.rs — Hierarchical JSON localization parser
// Extracts string values from JSON objects and arrays, preserving key hierarchy and structural context.

use std::path::Path;
use anyhow::{Result, Context};
use serde_json::Value;
use crate::core::parsers::RawSegment;
use crate::core::models::Segment;

pub fn parse(path: &Path) -> Result<Vec<RawSegment>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read JSON file: {:?}", path))?;
    parse_str(&content)
}

pub fn parse_str(content: &str) -> Result<Vec<RawSegment>> {
    let root: Value = serde_json::from_str(content)
        .with_context(|| "Failed to parse JSON content")?;

    let mut segments = Vec::new();
    traverse_json(&root, "", &mut segments);
    Ok(segments)
}

fn traverse_json(val: &Value, path: &str, segments: &mut Vec<RawSegment>) {
    match val {
        Value::String(s) => {
            let trimmed = s.trim();
            if !trimmed.is_empty() {
                segments.push(RawSegment {
                    source_text: s.clone(),
                    target_text: None,
                    notes: None,
                    context: if path.is_empty() { None } else { Some(path.to_string()) },
                });
            }
        }
        Value::Object(map) => {
            for (k, v) in map {
                let subpath = if path.is_empty() { k.clone() } else { format!("{}.{}", path, k) };
                traverse_json(v, &subpath, segments);
            }
        }
        Value::Array(arr) => {
            for (idx, v) in arr.iter().enumerate() {
                let subpath = format!("{}[{}]", path, idx);
                traverse_json(v, &subpath, segments);
            }
        }
        _ => {}
    }
}

pub fn export(original_path: &Path, segments: &[Segment], out_path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(original_path)
        .with_context(|| format!("Failed to read original JSON: {:?}", original_path))?;
    let reconstructed = reconstruct_json(&content, segments)?;
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(out_path, reconstructed)?;
    Ok(())
}

pub fn reconstruct_json(original_content: &str, segments: &[Segment]) -> Result<String> {
    let mut root: Value = serde_json::from_str(original_content)?;
    let mut seg_idx = 0;
    replace_json_values(&mut root, segments, &mut seg_idx);
    Ok(serde_json::to_string_pretty(&root)?)
}

fn replace_json_values(val: &mut Value, segments: &[Segment], seg_idx: &mut usize) {
    match val {
        Value::String(s) => {
            if !s.trim().is_empty() && *seg_idx < segments.len() {
                let seg = &segments[*seg_idx];
                if !seg.target_text.trim().is_empty() {
                    *s = seg.target_text.clone();
                }
                *seg_idx += 1;
            }
        }
        Value::Object(map) => {
            for v in map.values_mut() {
                replace_json_values(v, segments, seg_idx);
            }
        }
        Value::Array(arr) => {
            for v in arr.iter_mut() {
                replace_json_values(v, segments, seg_idx);
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::SegmentStatus;

    #[test]
    fn test_parse_json() {
        let json = r#"{
            "app_name": "Free-CAT",
            "menu": {
                "file": "File",
                "edit": "Edit"
            },
            "items": ["First", "Second"]
        }"#;

        let segs = parse_str(json).unwrap();
        assert_eq!(segs.len(), 5);
        assert!(segs.iter().any(|s| s.source_text == "Free-CAT"));
        assert!(segs.iter().any(|s| s.source_text == "File" && s.context.as_deref() == Some("menu.file")));
        assert!(segs.iter().any(|s| s.source_text == "First" && s.context.as_deref() == Some("items[0]")));
    }

    #[test]
    fn test_reconstruct_json() {
        let original = r#"{"title": "Hello", "count": 42}"#;
        let segments = vec![
            Segment {
                id: "1".into(),
                file_id: "f".into(),
                seq_number: 1,
                source_text: "Hello".into(),
                target_text: "مرحبا".into(),
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

        let out = reconstruct_json(original, &segments).unwrap();
        assert!(out.contains("مرحبا"));
        assert!(out.contains("42"));
    }
}
