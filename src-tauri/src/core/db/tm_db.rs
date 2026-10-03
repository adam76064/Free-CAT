// core/db/tm_db.rs
// Translation Memory database — SQLite + FTS5 for fast fuzzy candidate retrieval.
// Architecture:
//   1. FTS5 virtual table for BM25 top-50 candidate retrieval (< 1ms)
//   2. strsim Levenshtein + Dice for precise % scoring on candidates (< 5ms)

use std::path::PathBuf;
use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use crate::core::models::{TranslationUnit, TmMatchResult};
use crate::core::tm::fuzzy::{score_match, FuzzyScore};

pub struct TmDb {
    pub id:   String,
    pub name: String,
    conn: Connection,
}

impl TmDb {
    /// Open (or create) a TM database at `path`.
    pub fn open(id: String, name: String, path: PathBuf) -> Result<Self> {
        let conn = Connection::open(&path)
            .with_context(|| format!("Cannot open TM DB at {:?}", path))?;

        conn.execute_batch("
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA cache_size = -64000;
        ")?;

        let mut db = TmDb { id, name, conn };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&mut self) -> Result<()> {
        self.conn.execute_batch("
            CREATE TABLE IF NOT EXISTS translation_units (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                source_lang TEXT    NOT NULL,
                target_lang TEXT    NOT NULL,
                source_text TEXT    NOT NULL,
                target_text TEXT    NOT NULL,
                created_at  TEXT    NOT NULL DEFAULT (datetime('now')),
                modified_at TEXT    NOT NULL DEFAULT (datetime('now')),
                created_by  TEXT,
                client      TEXT,
                domain      TEXT,
                source_file TEXT,
                usage_count INTEGER NOT NULL DEFAULT 0,
                metadata    TEXT
            );

            -- Content-linked FTS5 for fast candidate retrieval
            CREATE VIRTUAL TABLE IF NOT EXISTS tm_fts USING fts5(
                source_text,
                content='translation_units',
                content_rowid='id',
                tokenize='unicode61 remove_diacritics 2'
            );

            -- Keep FTS in sync with translation_units
            CREATE TRIGGER IF NOT EXISTS tm_ai AFTER INSERT ON translation_units BEGIN
                INSERT INTO tm_fts(rowid, source_text) VALUES (new.id, new.source_text);
            END;

            CREATE TRIGGER IF NOT EXISTS tm_au AFTER UPDATE ON translation_units BEGIN
                INSERT INTO tm_fts(tm_fts, rowid, source_text) VALUES ('delete', old.id, old.source_text);
                INSERT INTO tm_fts(rowid, source_text) VALUES (new.id, new.source_text);
            END;

            CREATE TRIGGER IF NOT EXISTS tm_ad AFTER DELETE ON translation_units BEGIN
                INSERT INTO tm_fts(tm_fts, rowid, source_text) VALUES ('delete', old.id, old.source_text);
            END;
        ")?;
        Ok(())
    }

    // ── Write ──────────────────────────────────────────────────────────────────

    /// Insert a new translation unit. If an identical source_text exists for
    /// the same lang pair, update it instead (upsert).
    pub fn upsert(&self, tu: &TranslationUnit) -> Result<()> {
        // Check if identical source exists
        let existing: Option<i64> = self.conn.query_row(
            "SELECT id FROM translation_units WHERE source_text = ?1 AND source_lang = ?2 AND target_lang = ?3 LIMIT 1",
            params![tu.source_text, tu.source_lang, tu.target_lang],
            |r| r.get(0),
        ).ok();

        if let Some(id) = existing {
            self.conn.execute(
                "UPDATE translation_units SET target_text = ?1, modified_at = ?2, usage_count = usage_count + 1
                 WHERE id = ?3",
                params![tu.target_text, tu.modified_at, id],
            )?;
        } else {
            self.conn.execute(
                "INSERT INTO translation_units
                 (source_lang, target_lang, source_text, target_text, created_at, modified_at,
                  created_by, client, domain, source_file, usage_count, metadata)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0, ?11)",
                params![
                    tu.source_lang, tu.target_lang, tu.source_text, tu.target_text,
                    tu.created_at, tu.modified_at, tu.created_by, tu.client,
                    tu.domain, tu.source_file, tu.metadata
                ],
            )?;
        }
        Ok(())
    }

    pub fn delete(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM translation_units WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ── Query ──────────────────────────────────────────────────────────────────

    /// Main TM lookup: FTS5 candidates → precise Levenshtein+Dice scoring.
    /// Returns up to `limit` matches with score >= `threshold` (0.0–1.0).
    pub fn query(
        &self,
        source_text: &str,
        src_lang: &str,
        tgt_lang: &str,
        threshold: f64,  // e.g. 0.75 for 75%
        limit: usize,
        penalty: f64,    // subtract from score (0.0 = no penalty)
    ) -> Result<Vec<TmMatchResult>> {
        // Phase 1: FTS5 BM25 retrieval — get top 50 candidates fast
        let fts_query = build_fts_query(source_text);
        let candidates: Vec<(i64, String, String)> = {
            let mut stmt = self.conn.prepare(
                "SELECT tu.id, tu.source_text, tu.target_text
                 FROM tm_fts
                 JOIN translation_units tu ON tu.id = tm_fts.rowid
                 WHERE tm_fts MATCH ?1
                   AND tu.source_lang = ?2
                   AND tu.target_lang = ?3
                 ORDER BY rank
                 LIMIT 50"
            )?;
            stmt.query_map(params![fts_query, src_lang, tgt_lang], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?
        };

        // Phase 2: Precise scoring with Levenshtein + Dice blend
        let mut results: Vec<TmMatchResult> = candidates
            .into_iter()
            .filter_map(|(id, src, tgt)| {
                let score = score_match(source_text, &src);
                let adjusted = (score.combined - penalty).max(0.0);
                if adjusted >= threshold {
                    Some(TmMatchResult {
                        tu_id:        id,
                        source_text:  src,
                        target_text:  tgt,
                        score_pct:    (adjusted * 100.0).round() as u8,
                        levenshtein:  (score.levenshtein * 100.0).round() as u8,
                        dice:         (score.dice * 100.0).round() as u8,
                        tm_id:        self.id.clone(),
                        tm_name:      self.name.clone(),
                        is_exact:     adjusted >= 1.0,
                        is_context:   false, // set by context check layer
                    })
                } else {
                    None
                }
            })
            .collect();

        // Sort descending by score
        results.sort_by(|a, b| b.score_pct.cmp(&a.score_pct));
        results.truncate(limit);
        Ok(results)
    }

    /// Full-text concordance search across source and target.
    pub fn concordance_search(&self, query: &str, limit: usize) -> Result<Vec<TranslationUnit>> {
        let fts_query = format!("\"{}\"", query.replace('"', "")); // exact phrase
        let mut stmt = self.conn.prepare(
            "SELECT tu.id, tu.source_lang, tu.target_lang, tu.source_text, tu.target_text,
                    tu.created_at, tu.modified_at, tu.created_by, tu.client, tu.domain,
                    tu.source_file, tu.usage_count, tu.metadata
             FROM tm_fts
             JOIN translation_units tu ON tu.id = tm_fts.rowid
             WHERE tm_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2"
        )?;
        let rows = stmt.query_map(params![fts_query, limit as i64], |r| {
            Ok(TranslationUnit {
                id:          Some(r.get(0)?),
                source_lang: r.get(1)?,
                target_lang: r.get(2)?,
                source_text: r.get(3)?,
                target_text: r.get(4)?,
                created_at:  r.get(5)?,
                modified_at: r.get(6)?,
                created_by:  r.get(7)?,
                client:      r.get(8)?,
                domain:      r.get(9)?,
                source_file: r.get(10)?,
                usage_count: r.get(11)?,
                metadata:    r.get(12)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// Fetch all TUs (for export).
    pub fn get_all(&self) -> Result<Vec<TranslationUnit>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, source_lang, target_lang, source_text, target_text,
                    created_at, modified_at, created_by, client, domain,
                    source_file, usage_count, metadata
             FROM translation_units ORDER BY id ASC"
        )?;
        let rows = stmt.query_map([], |r| Ok(TranslationUnit {
            id:          Some(r.get(0)?),
            source_lang: r.get(1)?,
            target_lang: r.get(2)?,
            source_text: r.get(3)?,
            target_text: r.get(4)?,
            created_at:  r.get(5)?,
            modified_at: r.get(6)?,
            created_by:  r.get(7)?,
            client:      r.get(8)?,
            domain:      r.get(9)?,
            source_file: r.get(10)?,
            usage_count: r.get(11)?,
            metadata:    r.get(12)?,
        }))?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn count(&self) -> Result<i64> {
        Ok(self.conn.query_row(
            "SELECT COUNT(*) FROM translation_units", [], |r| r.get(0)
        )?)
    }
}

/// Build a safe FTS5 MATCH query from raw input.
/// Escapes special characters and uses phrase search for multi-word queries.
fn build_fts_query(input: &str) -> String {
    // Tokenise to individual words, strip FTS5 special chars
    let words: Vec<String> = input
        .split_whitespace()
        .map(|w| w.chars().filter(|c| c.is_alphanumeric() || *c == '\'').collect::<String>())
        .filter(|w| !w.is_empty())
        .take(10) // cap at 10 tokens
        .collect();

    if words.is_empty() {
        return String::from("\"\"");
    }
    // Use OR between words for broad candidate recall
    words.join(" OR ")
}

impl std::fmt::Debug for TmDb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TmDb({})", self.name)
    }
}
