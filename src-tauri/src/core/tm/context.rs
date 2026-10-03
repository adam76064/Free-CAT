// core/tm/context.rs
// Context match detection: determines if a 100% exact match qualifies as
// 101% (structural context match) or 102% (full surrounding context match).
//
// Industry definitions (Trados / memoQ standard):
//   102% = source text identical AND both surrounding segment texts match
//   101% = source text identical AND one surrounding segment matches (or same file/doc position)

/// Given a 100% exact TM match, check if surrounding context elevates it.
/// Returns 100, 101, or 102.
pub fn detect_context_match(
    tm_source: &str,
    current_source: &str,
    prev_segment: Option<&str>,
    next_segment: Option<&str>,
) -> u8 {
    if tm_source != current_source {
        return 100;
    }

    let context_count = [prev_segment, next_segment]
        .iter()
        .filter(|s| s.is_some())
        .count();

    if context_count == 0 {
        return 100;
    }

    // For a real 101/102 check we'd compare stored surrounding segments in the TM.
    // Phase 0: simplified — if prev/next segments exist in context, elevate to 101%.
    // Phase 2 will store context hashes in the TM schema for precise 101/102 detection.
    if prev_segment.is_some() || next_segment.is_some() {
        101
    } else {
        100
    }
}
