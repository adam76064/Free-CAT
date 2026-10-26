// core/models.rs
// All shared data models used by the Rust backend and serialized to the frontend via Tauri IPC.

use serde::{Deserialize, Serialize};

// ── Project ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id:          String,
    pub name:        String,
    pub src_lang:    String,
    pub tgt_lang:    String,
    pub created_at:  String,
    pub modified_at: String,
    pub settings:    String, // JSON
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    pub fuzzy_threshold:   f64,         // 0.0–1.0, default 0.75
    pub tm_ids:            Vec<String>,  // ordered list of TM IDs
    pub tb_ids:            Vec<String>,  // ordered list of TB IDs
    pub mt_engine:         Option<String>,
    pub mt_penalty:        f64,         // default 0.03 (3%)
    pub auto_propagate:    bool,
    pub writeback_mode:    WritebackMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WritebackMode {
    Always,
    Ask,
    Never,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        ProjectSettings {
            fuzzy_threshold: 0.75,
            tm_ids: vec![],
            tb_ids: vec![],
            mt_engine: None,
            mt_penalty: 0.03,
            auto_propagate: true,
            writeback_mode: WritebackMode::Always,
        }
    }
}

// ── Project File ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFile {
    pub id:          String,
    pub project_id:  String,
    pub filename:    String,
    pub format:      String, // "docx" | "xliff12" | "xliff20" | "po" | "txt" | "html" | ...
    pub source_path: String,
    pub status:      String, // "new" | "in_progress" | "translated" | "reviewed"
    pub word_count:  i64,
    pub seg_count:   i64,
    pub added_at:    String,
}

// ── Segment ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub id:            String,
    pub file_id:       String,
    pub seq_number:    i64,
    pub source_text:   String,
    pub target_text:   String,
    pub status:        SegmentStatus,
    pub match_percent: Option<i64>,
    pub match_origin:  Option<String>,
    pub locked:        bool,
    pub bookmarked:    bool,
    pub comment:       Option<String>,
    pub created_at:    String,
    pub modified_at:   String,
    pub modified_by:   Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SegmentStatus {
    New,
    Draft,
    Translated,
    NeedsReview,
    Reviewed,
    Final,
}

impl SegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SegmentStatus::New         => "new",
            SegmentStatus::Draft       => "draft",
            SegmentStatus::Translated  => "translated",
            SegmentStatus::NeedsReview => "needs_review",
            SegmentStatus::Reviewed    => "reviewed",
            SegmentStatus::Final       => "final",
        }
    }

    /// Strict parse: `None` for anything that isn't a known status name.
    pub fn parse(s: &str) -> Option<SegmentStatus> {
        match s {
            "new"          => Some(SegmentStatus::New),
            "draft"        => Some(SegmentStatus::Draft),
            "translated"   => Some(SegmentStatus::Translated),
            "needs_review" => Some(SegmentStatus::NeedsReview),
            "reviewed"     => Some(SegmentStatus::Reviewed),
            "final"        => Some(SegmentStatus::Final),
            _              => None,
        }
    }

    /// Lenient parse used when reading stored data: unknown values become `New`.
    pub fn from_str(s: &str) -> SegmentStatus {
        Self::parse(s).unwrap_or(SegmentStatus::New)
    }
}

// ── Segment History ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentHistory {
    pub id:          Option<i64>,
    pub segment_id:  String,
    pub target_text: String,
    pub action:      String, // "edit" | "confirm" | "pre_translate" | "review_accept"
    pub actor:       Option<String>,
    pub timestamp:   String,
}

// ── Translation Memory ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationUnit {
    pub id:          Option<i64>,
    pub source_lang: String,
    pub target_lang: String,
    pub source_text: String,
    pub target_text: String,
    pub created_at:  String,
    pub modified_at: String,
    pub created_by:  Option<String>,
    pub client:      Option<String>,
    pub domain:      Option<String>,
    pub source_file: Option<String>,
    pub usage_count: Option<i64>,
    pub metadata:    Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmMatchResult {
    pub tu_id:       i64,
    pub source_text: String,
    pub target_text: String,
    pub score_pct:   u8,      // 0–102
    pub levenshtein: u8,      // raw Levenshtein component %
    pub dice:        u8,      // raw Dice component %
    pub tm_id:       String,
    pub tm_name:     String,
    pub is_exact:    bool,    // score_pct == 100
    pub is_context:  bool,    // 101% or 102% context match
    pub repaired_target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmInfo {
    pub id:       String,
    pub name:     String,
    pub src_lang: String,
    pub tgt_lang: String,
    pub tu_count: i64,
}

// ── Termbase ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermEntry {
    pub id:         Option<i64>,
    pub lang:       String,
    pub term:       String,
    pub definition: Option<String>,
    pub domain:     Option<String>,
    pub note:       Option<String>,
    pub status:     String, // "preferred" | "allowed" | "forbidden" | "deprecated"
    pub created_at: String,
    pub metadata:   Option<String>,
}

// ── QA ────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QaResult {
    pub id:          Option<i64>,
    pub project_id:  String,
    pub segment_id:  Option<String>,
    pub check_name:  String,
    pub severity:    String, // "warning" | "error"
    pub message:     String,
    pub run_at:      String,
}

// ── Analysis ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub project_id:     String,
    pub total_words:    i64,
    pub total_segments: i64,
    pub exact_words:    i64,   // 100%+ matches
    pub fuzzy_words:    i64,   // 75–99%
    pub new_words:      i64,   // no match
    pub exact_segments: i64,
    pub fuzzy_segments: i64,
    pub new_segments:   i64,
    pub tm_savings_pct: f64,
}

// ── TQA (Translation Quality Assessment / DQF-MQM) ───────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TqaError {
    pub id:          Option<i64>,
    pub project_id:  String,
    pub segment_id:  String,
    pub category:    String, // "accuracy" | "fluency" | "terminology" | "style" | "design"
    pub sub_type:    Option<String>,
    pub severity:    String, // "minor" | "major" | "critical"
    pub penalty:     i64,    // 1, 5, 25
    pub comment:     Option<String>,
    pub created_at:  String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TqaScorecard {
    pub project_id:     String,
    pub total_words:    i64,
    pub total_errors:   i64,
    pub minor_count:    i64,
    pub major_count:    i64,
    pub critical_count: i64,
    pub total_penalty:  i64,
    pub score_pct:      f64,
    pub passed:         bool,
    pub errors:         Vec<TqaError>,
}
