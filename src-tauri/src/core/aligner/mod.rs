pub mod dtw;
pub use dtw::{AlignedPair, align_sentences};

use crate::core::segmentation::srx::segment;

/// Segment source and target raw texts using SRX rules and run DTW alignment.
pub fn align_raw_texts(
    source_text: &str,
    target_text: &str,
    src_lang: &str,
    tgt_lang: &str,
) -> Vec<AlignedPair> {
    let src_segs: Vec<String> = segment(source_text, src_lang)
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let tgt_segs: Vec<String> = segment(target_text, tgt_lang)
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    align_sentences(&src_segs, &tgt_segs)
}

/// Generate a valid TMX 1.4b document from aligned sentence pairs.
pub fn export_pairs_to_tmx(
    pairs: &[AlignedPair],
    src_lang: &str,
    tgt_lang: &str,
) -> String {
    let now = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();
    let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE tmx SYSTEM "tmx14.dtd">
<tmx version="1.4">
  <header
    creationtool="Free-CAT"
    creationtoolversion="{VERSION}"
    segtype="sentence"
    o-tmf="Free-CAT Aligner"
    adminlang="en"
    srclang=""#).replace("{VERSION}", env!("CARGO_PKG_VERSION"));
    xml.push_str(src_lang);
    xml.push_str(r#""
    datatype="PlainText"
    creationdate=""#);
    xml.push_str(&now);
    xml.push_str(r#"">
  </header>
  <body>
"#);

    for pair in pairs {
        if pair.source_text.trim().is_empty() || pair.target_text.trim().is_empty() {
            continue;
        }

        xml.push_str("    <tu>\n");
        xml.push_str(&format!(
            "      <tuv xml:lang=\"{}\"><seg>{}</seg></tuv>\n",
            src_lang,
            quick_xml::escape::escape(&pair.source_text)
        ));
        xml.push_str(&format!(
            "      <tuv xml:lang=\"{}\"><seg>{}</seg></tuv>\n",
            tgt_lang,
            quick_xml::escape::escape(&pair.target_text)
        ));
        xml.push_str("    </tu>\n");
    }

    xml.push_str("  </body>\n</tmx>\n");
    xml
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align_raw_texts_and_tmx() {
        let src = "First sentence. Second sentence with 2026. Third sentence.";
        let tgt = "الجملة الأولى. الجملة الثانية مع 2026. الجملة الثالثة.";

        let pairs = align_raw_texts(src, tgt, "en", "ar");
        assert_eq!(pairs.len(), 3);
        assert!(pairs[1].source_text.contains("2026"));
        assert!(pairs[1].target_text.contains("2026"));

        let tmx = export_pairs_to_tmx(&pairs, "en", "ar");
        assert!(tmx.contains(r#"<tmx version="1.4">"#));
        assert!(tmx.contains("<tu>"));
        assert!(tmx.contains("2026"));
    }
}
