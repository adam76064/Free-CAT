// core/parsers/markdown.rs — Markdown (.md) parser and reconstructor

use std::path::Path;
use anyhow::{Result, Context};
use crate::core::models::Segment;
use crate::core::segmentation::srx;
use super::RawSegment;

/// Parse a Markdown file into translatable segments, skipping code blocks.
pub fn parse(path: &Path) -> Result<Vec<RawSegment>> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read Markdown file: {:?}", path))?;
    parse_markdown_str(&content)
}

pub fn parse_markdown_str(content: &str) -> Result<Vec<RawSegment>> {
    let mut segments = Vec::new();
    let mut in_code_block = false;

    let blocks = content.split("\n\n");
    let mut block_idx = 0usize;

    for block in blocks {
        let trimmed = block.trim();
        if trimmed.starts_with("```") {
            in_code_block = true;
            if trimmed.len() > 3 && trimmed[3..].contains("```") {
                in_code_block = false;
            }
            continue;
        }
        if in_code_block {
            if trimmed.ends_with("```") {
                in_code_block = false;
            }
            continue;
        }
        if trimmed.is_empty() {
            continue;
        }

        if trimmed.lines().count() > 1 && trimmed.lines().all(|l| l.trim().starts_with("- ") || l.trim().starts_with("* ")) {
            for line in trimmed.lines() {
                let l_trimmed = line.trim();
                let (prefix, text) = strip_md_prefix(l_trimmed);
                if text.is_empty() {
                    continue;
                }
                block_idx += 1;
                let sentences = srx::segment(text, "en");
                for (s_idx, s) in sentences.into_iter().enumerate() {
                    let s_trimmed = s.trim();
                    if !s_trimmed.is_empty() {
                        segments.push(RawSegment {
                            source_text: s_trimmed.to_string(),
                            target_text: None,
                            notes: if !prefix.is_empty() { Some(format!("prefix:{}", prefix)) } else { None },
                            context: Some(format!("b{}_s{}", block_idx, s_idx + 1)),
                        });
                    }
                }
            }
            continue;
        }

        // Strip markdown prefix for translation (e.g. #, ##, -, *, 1.)
        let (prefix, text) = strip_md_prefix(trimmed);
        if text.is_empty() {
            continue;
        }

        block_idx += 1;
        let sentences = srx::segment(text, "en");
        for (s_idx, s) in sentences.into_iter().enumerate() {
            let s_trimmed = s.trim();
            if !s_trimmed.is_empty() {
                segments.push(RawSegment {
                    source_text: s_trimmed.to_string(),
                    target_text: None,
                    notes: if !prefix.is_empty() { Some(format!("prefix:{}", prefix)) } else { None },
                    context: Some(format!("b{}_s{}", block_idx, s_idx + 1)),
                });
            }
        }
    }

    Ok(segments)
}

/// Export translated segments back into a Markdown document.
pub fn export(segments: &[Segment], source_path: &str, out_path: &str) -> Result<()> {
    let source_md = std::fs::read_to_string(source_path)
        .with_context(|| format!("Failed to read source Markdown: {}", source_path))?;
    let translated_md = reconstruct_markdown(&source_md, segments)?;
    std::fs::write(out_path, translated_md)
        .with_context(|| format!("Failed to write translated Markdown: {}", out_path))?;
    Ok(())
}

pub fn reconstruct_markdown(source_md: &str, segments: &[Segment]) -> Result<String> {
    let mut output_blocks = Vec::new();
    let mut seg_iter = segments.iter();
    let mut in_code_block = false;

    let blocks = source_md.split("\n\n");

    for block in blocks {
        let trimmed = block.trim();
        if trimmed.starts_with("```") {
            in_code_block = true;
            output_blocks.push(block.to_string());
            if trimmed.len() > 3 && trimmed[3..].contains("```") {
                in_code_block = false;
            }
            continue;
        }
        if in_code_block {
            output_blocks.push(block.to_string());
            if trimmed.ends_with("```") {
                in_code_block = false;
            }
            continue;
        }
        if trimmed.is_empty() {
            output_blocks.push(block.to_string());
            continue;
        }

        if trimmed.lines().count() > 1 && trimmed.lines().all(|l| l.trim().starts_with("- ") || l.trim().starts_with("* ")) {
            let mut list_out = Vec::new();
            for line in trimmed.lines() {
                let l_trimmed = line.trim();
                let (prefix, text) = strip_md_prefix(l_trimmed);
                if text.is_empty() {
                    continue;
                }
                let sentences = srx::segment(text, "en");
                let mut trans_line = Vec::new();
                for s in sentences {
                    if s.trim().is_empty() {
                        continue;
                    }
                    if let Some(seg) = seg_iter.next() {
                        if !seg.target_text.trim().is_empty() {
                            trans_line.push(seg.target_text.clone());
                        } else {
                            trans_line.push(seg.source_text.clone());
                        }
                    } else {
                        trans_line.push(s.to_string());
                    }
                }
                list_out.push(format!("{}{}", prefix, trans_line.join(" ")));
            }
            output_blocks.push(list_out.join("\n"));
            continue;
        }

        let (prefix, text) = strip_md_prefix(trimmed);
        if text.is_empty() {
            output_blocks.push(block.to_string());
            continue;
        }

        let sentences = srx::segment(text, "en");
        let mut translated_sentences = Vec::new();

        for s in sentences {
            if s.trim().is_empty() {
                continue;
            }
            if let Some(seg) = seg_iter.next() {
                if !seg.target_text.trim().is_empty() {
                    translated_sentences.push(seg.target_text.clone());
                } else {
                    translated_sentences.push(seg.source_text.clone());
                }
            } else {
                translated_sentences.push(s.to_string());
            }
        }

        let joined_text = translated_sentences.join(" ");
        if !prefix.is_empty() {
            output_blocks.push(format!("{}{}", prefix, joined_text));
        } else {
            output_blocks.push(joined_text);
        }
    }

    Ok(output_blocks.join("\n\n"))
}

fn strip_md_prefix(s: &str) -> (&str, &str) {
    if let Some(rest) = s.strip_prefix("### ") {
        ("### ", rest)
    } else if let Some(rest) = s.strip_prefix("## ") {
        ("## ", rest)
    } else if let Some(rest) = s.strip_prefix("# ") {
        ("# ", rest)
    } else if let Some(rest) = s.strip_prefix("- ") {
        ("- ", rest)
    } else if let Some(rest) = s.strip_prefix("* ") {
        ("* ", rest)
    } else if let Some(rest) = s.strip_prefix("> ") {
        ("> ", rest)
    } else {
        ("", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{Segment, SegmentStatus};

    const SAMPLE_MD: &str = r#"# Free-CAT Documentation

Free-CAT is a free, open-source CAT tool.

```rust
fn main() {
    println!("Hello, World!");
}
```

- Feature one.
- Feature two."#;

    #[test]
    fn test_parse_markdown() {
        let segments = parse_markdown_str(SAMPLE_MD).unwrap();
        assert_eq!(segments.len(), 4);
        assert_eq!(segments[0].source_text, "Free-CAT Documentation");
        assert_eq!(segments[1].source_text, "Free-CAT is a free, open-source CAT tool.");
        assert_eq!(segments[2].source_text, "Feature one.");
        assert_eq!(segments[3].source_text, "Feature two.");
    }

    #[test]
    fn test_reconstruct_markdown() {
        let segments = vec![
            Segment {
                id: "1".into(), file_id: "f".into(), seq_number: 1,
                source_text: "Free-CAT Documentation".into(), target_text: "توثيق Free-CAT".into(),
                status: SegmentStatus::Final, match_percent: None, match_origin: None,
                locked: false, bookmarked: false, comment: None, created_at: "".into(),
                modified_at: "".into(), modified_by: None,
            },
            Segment {
                id: "2".into(), file_id: "f".into(), seq_number: 2,
                source_text: "Free-CAT is a free, open-source CAT tool.".into(),
                target_text: "Free-CAT هي أداة ترجمة مجانية ومفتوحة المصدر.".into(),
                status: SegmentStatus::Final, match_percent: None, match_origin: None,
                locked: false, bookmarked: false, comment: None, created_at: "".into(),
                modified_at: "".into(), modified_by: None,
            },
            Segment {
                id: "3".into(), file_id: "f".into(), seq_number: 3,
                source_text: "Feature one.".into(), target_text: "الميزة الأولى.".into(),
                status: SegmentStatus::Final, match_percent: None, match_origin: None,
                locked: false, bookmarked: false, comment: None, created_at: "".into(),
                modified_at: "".into(), modified_by: None,
            },
            Segment {
                id: "4".into(), file_id: "f".into(), seq_number: 4,
                source_text: "Feature two.".into(), target_text: "الميزة الثانية.".into(),
                status: SegmentStatus::Final, match_percent: None, match_origin: None,
                locked: false, bookmarked: false, comment: None, created_at: "".into(),
                modified_at: "".into(), modified_by: None,
            },
        ];

        let result = reconstruct_markdown(SAMPLE_MD, &segments).unwrap();
        assert!(result.contains("# توثيق Free-CAT"));
        assert!(result.contains("Free-CAT هي أداة ترجمة مجانية ومفتوحة المصدر."));
        assert!(result.contains("- الميزة الأولى."));
        assert!(result.contains(r#"println!("Hello, World!");"#));
    }
}
