// core/analysis/pseudotranslate.rs
// Industry-standard Pseudo-Translation Engine for DTP & Layout Pre-Flight Testing.
// Follows Trados Studio / memoQ / ISO localization simulation standards:
// - Vowel and consonant diacritic accentuation (e.g. a -> ä, e -> é, c -> ç)
// - Configurable text expansion factor (+20%, +30%, +50%) to stress-test UI/document clipping
// - Boundary bracket wrapping ([~ ... ~]) to identify hardcoded / untranslated strings
// - 100% preservation of inline tags ({1}, {/1}, <g>, etc.) and whitespace

use regex::Regex;

#[derive(Debug, Clone)]
pub struct PseudoTranslateOptions {
    /// Percentage to expand text by (e.g. 20, 30, 50). Default is 30%.
    pub expansion_pct: u32,
    /// Wrap text with prefix and suffix delimiters (e.g. "[~" and "~]")
    pub add_brackets: bool,
    /// Substitute standard ASCII characters with accented European/Latin diacritics
    pub use_diacritics: bool,
}

impl Default for PseudoTranslateOptions {
    fn default() -> Self {
        Self {
            expansion_pct: 30,
            add_brackets: true,
            use_diacritics: true,
        }
    }
}

/// Convert an ASCII character to an accented counterpart
fn to_diacritic(c: char) -> char {
    match c {
        'a' => 'ä',
        'A' => 'Ä',
        'b' => 'ƀ',
        'B' => 'Ɓ',
        'c' => 'ç',
        'C' => 'Ç',
        'd' => 'đ',
        'D' => 'Đ',
        'e' => 'é',
        'E' => 'É',
        'f' => 'ƒ',
        'F' => 'Ƒ',
        'g' => 'ğ',
        'G' => 'Ğ',
        'h' => 'ħ',
        'H' => 'Ħ',
        'i' => 'ï',
        'I' => 'Ï',
        'j' => 'ĵ',
        'J' => 'Ĵ',
        'k' => 'ƙ',
        'K' => 'Ƙ',
        'l' => 'ł',
        'L' => 'Ł',
        'm' => 'ɱ',
        'M' => 'Ṁ',
        'n' => 'ñ',
        'N' => 'Ñ',
        'o' => 'ö',
        'O' => 'Ö',
        'p' => 'þ',
        'P' => 'Þ',
        'r' => 'ř',
        'R' => 'Ř',
        's' => 'š',
        'S' => 'Š',
        't' => 'ŧ',
        'T' => 'Ŧ',
        'u' => 'ü',
        'U' => 'Ü',
        'v' => 'ṽ',
        'V' => 'Ṽ',
        'w' => 'ŵ',
        'W' => 'Ŵ',
        'y' => 'ý',
        'Y' => 'Ý',
        'z' => 'ž',
        'Z' => 'Ž',
        other => other,
    }
}

/// Pseudo-translates a single segment text while strictly protecting inline tags.
pub fn pseudotranslate_text(source: &str, options: &PseudoTranslateOptions) -> String {
    if source.trim().is_empty() {
        return source.to_string();
    }

    // Regex matching inline tags like {1}, {/1}, {b}, <g id="1">, </g>
    let tag_re = Regex::new(r"(\{[^}]+\}|<[^>]+>)").unwrap();

    // Split text into tokens: alternating between translatable text and tags
    let mut parts: Vec<(bool, String)> = Vec::new();
    let mut last_idx = 0;

    for m in tag_re.find_iter(source) {
        if m.start() > last_idx {
            parts.push((true, source[last_idx..m.start()].to_string()));
        }
        parts.push((false, m.as_str().to_string()));
        last_idx = m.end();
    }
    if last_idx < source.len() {
        parts.push((true, source[last_idx..].to_string()));
    }

    // Process translatable parts
    let mut transformed_parts: Vec<String> = Vec::new();
    let mut original_translatable_len = 0;

    for (is_text, part) in parts {
        if !is_text {
            transformed_parts.push(part);
        } else {
            original_translatable_len += part.chars().count();
            if options.use_diacritics {
                let transformed: String = part.chars().map(to_diacritic).collect();
                transformed_parts.push(transformed);
            } else {
                transformed_parts.push(part);
            }
        }
    }

    let mut result = transformed_parts.join("");

    // Calculate expansion
    if options.expansion_pct > 0 && original_translatable_len > 0 {
        let extra_chars_needed = (original_translatable_len as f64 * (options.expansion_pct as f64 / 100.0)).round() as usize;
        if extra_chars_needed > 0 {
            // Append expansion pattern: e.g. " ~!@#$..." or repetitive syllables
            let expansion_fill = " ~!@#$%^&*()_+=~";
            let mut padding = String::new();
            while padding.chars().count() < extra_chars_needed {
                padding.push_str(expansion_fill);
            }
            let truncated_padding: String = padding.chars().take(extra_chars_needed).collect();
            result.push_str(&truncated_padding);
        }
    }

    if options.add_brackets {
        format!("[~{}~]", result)
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_diacritics_and_brackets() {
        let opts = PseudoTranslateOptions {
            expansion_pct: 0,
            add_brackets: true,
            use_diacritics: true,
        };
        let src = "Hello World";
        let res = pseudotranslate_text(src, &opts);
        assert!(res.starts_with("[~"));
        assert!(res.ends_with("~]"));
        // 'H' -> 'Ħ', 'e' -> 'é', 'o' -> 'ö'
        assert!(res.contains("Ħéłłö"));
    }

    #[test]
    fn test_expansion_length() {
        let opts = PseudoTranslateOptions {
            expansion_pct: 50,
            add_brackets: false,
            use_diacritics: false,
        };
        let src = "1234567890"; // 10 chars -> 50% = 5 extra chars
        let res = pseudotranslate_text(src, &opts);
        assert_eq!(res.chars().count(), 15);
    }

    #[test]
    fn test_tag_preservation() {
        let opts = PseudoTranslateOptions {
            expansion_pct: 20,
            add_brackets: true,
            use_diacritics: true,
        };
        let src = "Click {1}here{/1} to continue.";
        let res = pseudotranslate_text(src, &opts);
        // Inline tags must remain completely intact
        assert!(res.contains("{1}"));
        assert!(res.contains("{/1}"));
    }
}
