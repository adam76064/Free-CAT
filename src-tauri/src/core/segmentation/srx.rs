// core/segmentation/srx.rs
// SRX 2.0-inspired sentence segmentation engine.
// Reference: https://ttt.org/oscarstandards/srx/srx20.html
//
// Phase 0: Built-in rules for major languages using regex break/no-break pairs.
// Phase 2: Full SRX 2.0 XML rule file parsing will be added.

use regex::Regex;

/// Segment text into sentences using language-specific break rules.
/// Returns a Vec of sentence strings.
pub fn segment(text: &str, lang: &str) -> Vec<String> {
    let lang_prefix = lang.split('-').next().unwrap_or("en").to_lowercase();
    let rules = get_rules(&lang_prefix);
    apply_rules(text, &rules)
}

struct SegmentRule {
    /// Regex that must match the text BEFORE the potential break point
    before_break: Regex,
    /// Regex that must match the text AFTER the potential break point
    after_break:  Regex,
    /// If true, this is a BREAK rule. If false, it's a NO-BREAK rule.
    is_break:     bool,
}

/// Apply segmentation rules to text. Rules are evaluated in order;
/// the first matching rule determines whether to break or not.
fn apply_rules(text: &str, rules: &[SegmentRule]) -> Vec<String> {
    if text.is_empty() {
        return vec![];
    }

    let chars: Vec<char> = text.chars().collect();
    let mut segments = Vec::new();
    let mut seg_start = 0usize;

    for i in 0..chars.len() {
        // Only evaluate at potential break positions: after punctuation
        let c = chars[i];
        if !is_potential_break(c) {
            continue;
        }

        // Build before/after strings for rule evaluation
        let before: String = chars[..=i].iter().collect();
        let after: String = chars[i+1..].iter().collect();

        // Evaluate rules
        let mut should_break = false;
        for rule in rules {
            let before_matches = rule.before_break.is_match(&before);
            let after_matches = after.is_empty() || rule.after_break.is_match(&after);

            if before_matches && after_matches {
                should_break = rule.is_break;
                break;
            }
        }

        if should_break && i + 1 < chars.len() {
            let seg: String = chars[seg_start..=i].iter().collect();
            let trimmed = seg.trim().to_string();
            if !trimmed.is_empty() {
                segments.push(trimmed);
            }
            seg_start = i + 1;
        }
    }

    // Remaining text
    let remaining: String = chars[seg_start..].iter().collect();
    let trimmed = remaining.trim().to_string();
    if !trimmed.is_empty() {
        segments.push(trimmed);
    }

    segments
}

fn is_potential_break(c: char) -> bool {
    matches!(c, '.' | '?' | '!' | '…' | '。' | '？' | '！' | '।' | '۔')
}

/// Get segmentation rules for a given language code.
fn get_rules(_lang: &str) -> Vec<SegmentRule> {
    let mut rules = Vec::new();

    // ── NO-BREAK rules (evaluated first) ──────────────────────────────────────

    // Abbreviations: Mr. Mrs. Dr. Prof. etc.
    rules.push(SegmentRule {
        before_break: Regex::new(r"(Mr|Mrs|Ms|Dr|Prof|Sr|Jr|St|Ave|Blvd|vs|etc|approx|dept|div|est|fig|govt|Inc|Corp|Ltd)\.$").unwrap(),
        after_break:  Regex::new(r"^\s*\S").unwrap(),
        is_break:     false,
    });

    // Decimal numbers: 3.14, 1.000, etc.
    rules.push(SegmentRule {
        before_break: Regex::new(r"\d\.$").unwrap(),
        after_break:  Regex::new(r"^\d").unwrap(),
        is_break:     false,
    });

    // Single capital letter abbreviation: U.S.A., e.g., i.e.
    rules.push(SegmentRule {
        before_break: Regex::new(r"\b[A-Za-z]\.$").unwrap(),
        after_break:  Regex::new(r"^[A-Za-z\s]*[a-z]").unwrap(),
        is_break:     false,
    });

    // Ellipsis mid-sentence
    rules.push(SegmentRule {
        before_break: Regex::new(r"\.\.\.$").unwrap(),
        after_break:  Regex::new(r"^\s*[a-z]").unwrap(),
        is_break:     false,
    });

    // ── BREAK rules ────────────────────────────────────────────────────────────

    // Period followed by whitespace + uppercase → break
    rules.push(SegmentRule {
        before_break: Regex::new(r"\.$").unwrap(),
        after_break:  Regex::new(r"^\s+[A-Z\u00C0-\u00D6\u00D8-\u00F6]").unwrap(),
        is_break:     true,
    });

    // Question mark / exclamation mark always break
    rules.push(SegmentRule {
        before_break: Regex::new(r"[?!]\s*$").unwrap(),
        after_break:  Regex::new(r".*").unwrap(),
        is_break:     true,
    });

    // CJK sentence-ending punctuation
    rules.push(SegmentRule {
        before_break: Regex::new(r"[。？！]$").unwrap(),
        after_break:  Regex::new(r".*").unwrap(),
        is_break:     true,
    });

    rules
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_english_segmentation() {
        let text = "Hello world. This is Free-CAT. It segments text.";
        let segs = segment(text, "en");
        assert_eq!(segs.len(), 3, "Expected 3 segments, got: {:?}", segs);
    }

    #[test]
    fn abbreviation_not_broken() {
        let text = "Dr. Smith works at Inc. Corp. He is great.";
        let segs = segment(text, "en");
        // Should not split on "Dr." or "Inc." or "Corp."
        println!("Abbreviation test: {:?}", segs);
        assert!(segs.len() <= 2);
    }

    #[test]
    fn question_mark_breaks() {
        let text = "Is this working? Yes it is!";
        let segs = segment(text, "en");
        assert_eq!(segs.len(), 2, "Got: {:?}", segs);
    }
}
