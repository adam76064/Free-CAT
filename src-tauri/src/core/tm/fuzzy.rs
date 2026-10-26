// core/tm/fuzzy.rs
// Fuzzy matching score engine.
//
// Formula (from plan research):
//   score = 0.7 × normalized_levenshtein + 0.3 × dice_coefficient
//
// References:
//   - strsim crate: https://crates.io/crates/strsim
//   - "Fuzzy Match Repair" research — ACL Anthology 2024
//   - Industry standard: SDL Trados uses Levenshtein; Déjà Vu uses Dice blend

use strsim::normalized_damerau_levenshtein;

/// Raw fuzzy scores before blending.
#[derive(Debug, Clone)]
pub struct FuzzyScore {
    pub levenshtein: f64,  // normalized edit distance similarity (0.0–1.0)
    pub dice:        f64,  // word-level bigram Dice coefficient (0.0–1.0)
    pub combined:    f64,  // 0.7 × levenshtein + 0.3 × dice
}

/// Compute the blended fuzzy match score between `query` and `candidate`.
///
/// Both strings are normalized (trimmed, collapsed whitespace) before scoring.
/// Uses Damerau-Levenshtein (handles transpositions like "teh" → "the") instead
/// of plain Levenshtein for better human-edit simulation.
pub fn score_match(query: &str, candidate: &str) -> FuzzyScore {
    let q = normalize_text(query);
    let c = normalize_text(candidate);

    // Shortcut: identical strings
    if q == c {
        return FuzzyScore { levenshtein: 1.0, dice: 1.0, combined: 1.0 };
    }

    // Empty string edge case
    if q.is_empty() || c.is_empty() {
        return FuzzyScore { levenshtein: 0.0, dice: 0.0, combined: 0.0 };
    }

    // Arabic-script orthographic variants (diacritics, tatweel, alef/yaa forms,
    // numeral systems) are scored as the same text. Strings that differ ONLY in
    // such variants are capped at 99%, so a non-identical match never reads 100%.
    let fq = fold_arabic(&q);
    let fc = fold_arabic(&c);
    if fq == fc {
        return FuzzyScore { levenshtein: 0.99, dice: 0.99, combined: 0.99 };
    }

    let lev  = normalized_damerau_levenshtein(&fq, &fc);
    let dice = word_dice(&fq, &fc);
    let combined = 0.7 * lev + 0.3 * dice;

    FuzzyScore { levenshtein: lev, dice, combined }
}

/// Convert a score (0.0–1.0) to a display percentage (0–102).
/// 102% = context match (set externally); 101% = structure context match.
pub fn score_to_pct(score: f64) -> u8 {
    (score * 100.0).round().min(100.0) as u8
}

/// Word-level Dice coefficient: bigram overlap of word tokens.
/// Better than character Dice for sentences — word order and vocabulary matter more.
fn word_dice(a: &str, b: &str) -> f64 {
    let a_words: Vec<&str> = a.split_whitespace().collect();
    let b_words: Vec<&str> = b.split_whitespace().collect();

    if a_words.is_empty() || b_words.is_empty() {
        return 0.0;
    }

    // Build bigram sets from word tokens
    let a_bigrams: std::collections::HashSet<(&str, &str)> = a_words.windows(2)
        .map(|w| (w[0], w[1]))
        .collect();
    let b_bigrams: std::collections::HashSet<(&str, &str)> = b_words.windows(2)
        .map(|w| (w[0], w[1]))
        .collect();

    // For single-word inputs, fall back to word-level intersection
    if a_words.len() == 1 || b_words.len() == 1 {
        let a_set: std::collections::HashSet<&str> = a_words.iter().copied().collect();
        let b_set: std::collections::HashSet<&str> = b_words.iter().copied().collect();
        let intersection = a_set.intersection(&b_set).count() as f64;
        return (2.0 * intersection) / (a_set.len() + b_set.len()) as f64;
    }

    let intersection = a_bigrams.intersection(&b_bigrams).count() as f64;
    (2.0 * intersection) / (a_bigrams.len() + b_bigrams.len()) as f64
}

fn is_arabic_script(c: char) -> bool {
    matches!(c, '\u{0600}'..='\u{06FF}' | '\u{0750}'..='\u{077F}' | '\u{FB50}'..='\u{FDFF}' | '\u{FE70}'..='\u{FEFF}')
}

/// Fold Arabic-script orthographic variants so they don't count as edits:
/// - removes harakat/tashkeel (U+064B–U+065F), superscript alef (U+0670) and tatweel (U+0640)
/// - unifies alef forms أ إ آ ٱ → ا, alef maqsura ى → ي, taa marbuta ة → ه
/// - maps Arabic-Indic (٠-٩) and Persian (۰-۹) digits to ASCII
/// Text with no Arabic-script characters is returned unchanged.
fn fold_arabic(s: &str) -> String {
    if !s.chars().any(is_arabic_script) {
        return s.to_string();
    }
    s.chars()
        .filter_map(|c| match c {
            '\u{064B}'..='\u{065F}' | '\u{0670}' | '\u{0640}' => None,
            '\u{0623}' | '\u{0625}' | '\u{0622}' | '\u{0671}' => Some('\u{0627}'),
            '\u{0649}' => Some('\u{064A}'),
            '\u{0629}' => Some('\u{0647}'),
            '\u{0660}'..='\u{0669}' => char::from_digit(c as u32 - 0x0660, 10),
            '\u{06F0}'..='\u{06F9}' => char::from_digit(c as u32 - 0x06F0, 10),
            _ => Some(c),
        })
        .collect()
}

/// Normalize text for comparison: trim, collapse whitespace, lowercase.
fn normalize_text(s: &str) -> String {
    s.split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_strings_score_100() {
        let s = score_match("The quick brown fox", "The quick brown fox");
        assert_eq!(s.combined, 1.0);
    }

    #[test]
    fn empty_strings_score_zero() {
        let s = score_match("", "hello");
        assert_eq!(s.combined, 0.0);
    }

    #[test]
    fn high_similarity_above_threshold() {
        // One word changed
        let s = score_match(
            "The document must be translated by Friday.",
            "The document must be translated by Monday."
        );
        println!("One word diff: {:.2}%", s.combined * 100.0);
        assert!(s.combined >= 0.75, "Expected >= 75%, got {:.2}%", s.combined * 100.0);
    }

    #[test]
    fn low_similarity_below_threshold() {
        let s = score_match("Hello world", "Completely different sentence here.");
        println!("Low sim: {:.2}%", s.combined * 100.0);
        assert!(s.combined < 0.5);
    }

    #[test]
    fn transposition_handled() {
        // Damerau-Levenshtein handles "teh" → "the"
        let s = score_match("Please submit teh report", "Please submit the report");
        println!("Transposition: {:.2}%", s.combined * 100.0);
        assert!(s.levenshtein >= 0.95, "Damerau-Levenshtein should catch transposition: {:.2}%", s.levenshtein * 100.0);
        assert!(s.combined >= 0.75, "Combined score should be >= 75%: {:.2}%", s.combined * 100.0);
    }

    #[test]
    fn arabic_diacritics_are_not_edits() {
        let r = score_match("كَتَبَ الوَلَدُ الدرس", "كتب الولد الدرس");
        assert!((r.combined - 0.99).abs() < 1e-9, "got {}", r.combined);
        assert_eq!(score_to_pct(r.combined), 99);
    }

    #[test]
    fn arabic_letter_variants_and_tatweel_fold() {
        assert!((score_match("أحمد ذهب إلى المدرسة", "احمد ذهب الى المدرسه").combined - 0.99).abs() < 1e-9);
        assert!((score_match("الــعــربية", "العربية").combined - 0.99).abs() < 1e-9);
        assert!((score_match("على", "علي").combined - 0.99).abs() < 1e-9);
    }

    #[test]
    fn arabic_indic_digits_fold_to_ascii() {
        assert!((score_match("اشترى ٣ كتب", "اشترى 3 كتب").combined - 0.99).abs() < 1e-9);
        assert!((score_match("۱۲۳", "123").combined - 0.99).abs() < 1e-9);
    }

    #[test]
    fn arabic_identical_text_is_still_100() {
        assert_eq!(score_to_pct(score_match("ذهب الولد", "ذهب الولد").combined), 100);
    }

    #[test]
    fn arabic_real_differences_still_penalised() {
        let r = score_match("ذهب الولد إلى المدرسة", "قرأت البنت كتابا جديدا");
        assert!(r.combined < 0.5, "got {}", r.combined);
        // a genuine one-word change must score below the variant-only cap
        let near = score_match("ذهب الولد إلى المدرسة", "ذهب الولد إلى السوق");
        assert!(near.combined < 0.99 && near.combined > 0.5, "got {}", near.combined);
    }

    #[test]
    fn latin_text_unaffected_by_folding() {
        assert_eq!(fold_arabic("Hello World 123"), "Hello World 123");
        assert_eq!(score_to_pct(score_match("Hello world", "hello  WORLD").combined), 100);
    }
}
