// core/parsers/yaml.rs — Hierarchical YAML localization parser
// Extracts string values from YAML structures while preserving keys and hierarchy.

use std::path::Path;
use anyhow::{Result, Context};
use serde_yaml::Value;
use crate::core::parsers::RawSegment;
use crate::core::models::Segment;

pub fn parse(path: &Path) -> Result<Vec<RawSegment>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read YAML file: {:?}", path))?;
    parse_str(&content)
}

pub fn parse_str(content: &str) -> Result<Vec<RawSegment>> {
    let root: Value = serde_yaml::from_str(content)
        .with_context(|| "Failed to parse YAML content")?;

    let mut segments = Vec::new();
    traverse_yaml(&root, "", &mut segments);
    Ok(segments)
}

fn traverse_yaml(val: &Value, path: &str, segments: &mut Vec<RawSegment>) {
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
        Value::Mapping(map) => {
            for (k, v) in map {
                let key_str = match k {
                    Value::String(s) => s.clone(),
                    _ => format!("{:?}", k),
                };
                let subpath = if path.is_empty() { key_str } else { format!("{}.{}", path, key_str) };
                traverse_yaml(v, &subpath, segments);
            }
        }
        Value::Sequence(seq) => {
            for (idx, v) in seq.iter().enumerate() {
                let subpath = format!("{}[{}]", path, idx);
                traverse_yaml(v, &subpath, segments);
            }
        }
        _ => {}
    }
}

pub fn export(original_path: &Path, segments: &[Segment], out_path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(original_path)
        .with_context(|| format!("Failed to read original YAML: {:?}", original_path))?;
    let reconstructed = reconstruct_yaml(&content, segments)?;
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(out_path, reconstructed)?;
    Ok(())
}

pub fn reconstruct_yaml(original_content: &str, segments: &[Segment]) -> Result<String> {
    let mut root: Value = serde_yaml::from_str(original_content)?;
    let mut seg_idx = 0;
    replace_yaml_values(&mut root, segments, &mut seg_idx);
    Ok(serde_yaml::to_string(&root)?)
}

fn replace_yaml_values(val: &mut Value, segments: &[Segment], seg_idx: &mut usize) {
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
        Value::Mapping(map) => {
            for v in map.values_mut() {
                replace_yaml_values(v, segments, seg_idx);
            }
        }
        Value::Sequence(seq) => {
            for v in seq.iter_mut() {
                replace_yaml_values(v, segments, seg_idx);
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
    fn test_parse_yaml() {
        let yaml = r#"
en:
  welcome: "Welcome to Free-CAT"
  buttons:
    save: "Save Project"
    cancel: "Cancel"
"#;

        let segs = parse_str(yaml).unwrap();
        assert_eq!(segs.len(), 3);
        assert_eq!(segs[0].source_text, "Welcome to Free-CAT");
        assert_eq!(segs[1].source_text, "Save Project");
        assert_eq!(segs[1].context.as_deref(), Some("en.buttons.save"));
    }

    #[test]
    fn test_reconstruct_yaml() {
        let yaml = r#"
app:
  name: "My App"
"#;
        let segments = vec![
            Segment {
                id: "1".into(),
                file_id: "f".into(),
                seq_number: 1,
                source_text: "My App".into(),
                target_text: "تطبيقي".into(),
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

        let out = reconstruct_yaml(yaml, &segments).unwrap();
        assert!(out.contains("تطبيقي"));
    }
}
