// core/db/tm_db.rs
// Translation Memory database — SQLite + FTS5 for fast fuzzy candidate retrieval.
// Architecture:
//   1. FTS5 virtual table for BM25 top-50 candidate retrieval (< 1ms)
//   2. strsim Levenshtein + Dice for precise % scoring on candidates (< 5ms)

use std::path::PathBuf;
use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use crate::core::models::{TranslationUnit, TmMatchResult};
use crate::core::tm::fuzzy::score_match;

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

    /// Paginated TU retrieval with optional search query for TM maintenance.
    pub fn list_tus_paginated(
        &self,
        offset: usize,
        limit: usize,
        search: Option<&str>,
    ) -> Result<(Vec<TranslationUnit>, usize)> {
        let (tus, total_count) = if let Some(q) = search.map(|s| s.trim()).filter(|s| !s.is_empty()) {
            let pattern = format!("%{}%", q);
            let total: usize = self.conn.query_row(
                "SELECT COUNT(*) FROM translation_units WHERE source_text LIKE ?1 OR target_text LIKE ?1",
                params![pattern],
                |r| r.get(0),
            )?;
            let mut stmt = self.conn.prepare(
                "SELECT id, source_lang, target_lang, source_text, target_text,
                        created_at, modified_at, created_by, client, domain,
                        source_file, usage_count, metadata
                 FROM translation_units
                 WHERE source_text LIKE ?1 OR target_text LIKE ?1
                 ORDER BY id DESC
                 LIMIT ?2 OFFSET ?3"
            )?;
            let rows = stmt.query_map(params![pattern, limit as i64, offset as i64], |r| {
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
            let items: Vec<TranslationUnit> = rows.collect::<rusqlite::Result<Vec<_>>>()?;
            (items, total)
        } else {
            let total: usize = self.conn.query_row(
                "SELECT COUNT(*) FROM translation_units",
                [],
                |r| r.get(0),
            )?;
            let mut stmt = self.conn.prepare(
                "SELECT id, source_lang, target_lang, source_text, target_text,
                        created_at, modified_at, created_by, client, domain,
                        source_file, usage_count, metadata
                 FROM translation_units
                 ORDER BY id DESC
                 LIMIT ?1 OFFSET ?2"
            )?;
            let rows = stmt.query_map(params![limit as i64, offset as i64], |r| {
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
            let items: Vec<TranslationUnit> = rows.collect::<rusqlite::Result<Vec<_>>>()?;
            (items, total)
        };
        Ok((tus, total_count))
    }

    /// Update an existing TU in place (triggers automatically sync FTS5).
    pub fn update_tu(&self, id: i64, source_text: &str, target_text: &str) -> Result<()> {
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        self.conn.execute(
            "UPDATE translation_units SET source_text = ?1, target_text = ?2, modified_at = ?3 WHERE id = ?4",
            params![source_text, target_text, now, id],
        )?;
        Ok(())
    }

    /// Batch find-and-replace across all translation units.
    pub fn batch_replace(&self, find: &str, replace_with: &str, target_only: bool) -> Result<usize> {
        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let affected = if target_only {
            self.conn.execute(
                "UPDATE translation_units
                 SET target_text = REPLACE(target_text, ?1, ?2), modified_at = ?3
                 WHERE target_text LIKE '%' || ?1 || '%'",
                params![find, replace_with, now],
            )?
        } else {
            self.conn.execute(
                "UPDATE translation_units
                 SET source_text = REPLACE(source_text, ?1, ?2),
                     target_text = REPLACE(target_text, ?1, ?2),
                     modified_at = ?3
                 WHERE source_text LIKE '%' || ?1 || '%' OR target_text LIKE '%' || ?1 || '%'",
                params![find, replace_with, now],
            )?
        };
        Ok(affected)
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
        let rows = stmt.query_map(params![fts_query, src_lang, tgt_lang], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get(2)?))
        })?;
        let candidates: Vec<(i64, String, String)> = rows.collect::<rusqlite::Result<Vec<_>>>()?;

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
                        repaired_target: None,
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

    /// Search for sub-segment fragment matches against n-grams of the source text.
    pub fn search_fragments(
        &self,
        source_text: &str,
        src_lang: &str,
        tgt_lang: &str,
        limit: usize,
    ) -> Result<Vec<crate::core::tm::fragment::FragmentMatch>> {
        use crate::core::tm::fragment::{extract_source_ngrams, FragmentMatch};

        let ngrams = extract_source_ngrams(source_text, 3, 6);
        let mut results = Vec::new();
        let mut seen_targets = std::collections::HashSet::new();

        let mut stmt = self.conn.prepare(
            "SELECT tu.id, tu.source_text, tu.target_text
             FROM tm_fts
             JOIN translation_units tu ON tu.id = tm_fts.rowid
             WHERE tm_fts MATCH ?1
               AND tu.source_lang = ?2
               AND tu.target_lang = ?3
             ORDER BY rank
             LIMIT 3"
        )?;

        for phrase in ngrams {
            let query = format!("\"{}\"", phrase.replace('"', ""));
            let rows = stmt.query_map(params![query, src_lang, tgt_lang], |r| {
                Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))
            });

            if let Ok(mapped) = rows {
                for item in mapped.flatten() {
                    let (tu_id, src_sentence, tgt_sentence) = item;
                    if src_sentence.trim() != source_text.trim() && seen_targets.insert(tgt_sentence.clone()) {
                        results.push(FragmentMatch {
                            source_fragment: phrase.clone(),
                            target_fragment: tgt_sentence.clone(),
                            source_sentence: src_sentence,
                            target_sentence: tgt_sentence,
                            tu_id,
                            tm_id: self.id.clone(),
                            tm_name: self.name.clone(),
                        });
                        if results.len() >= limit {
                            return Ok(results);
                        }
                    }
                }
            }
        }

        Ok(results)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tm_maintenance_operations() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("test_tm_maint_{}.db", uuid::Uuid::new_v4()));
        let tm = TmDb::open("test-tm".into(), "Test TM".into(), db_path).unwrap();

        let tu1 = TranslationUnit {
            id: None,
            source_lang: "en".into(),
            target_lang: "fr".into(),
            source_text: "Welcome to Acme Corp.".into(),
            target_text: "Bienvenue chez Acme Corp.".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
            modified_at: "2026-01-01T00:00:00Z".into(),
            created_by: None,
            client: None,
            domain: None,
            source_file: None,
            usage_count: Some(1),
            metadata: None,
        };
        let tu2 = TranslationUnit {
            id: None,
            source_lang: "en".into(),
            target_lang: "fr".into(),
            source_text: "Contact Acme Corp support.".into(),
            target_text: "Contactez le support Acme Corp.".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
            modified_at: "2026-01-01T00:00:00Z".into(),
            created_by: None,
            client: None,
            domain: None,
            source_file: None,
            usage_count: Some(2),
            metadata: None,
        };
        tm.upsert(&tu1).unwrap();
        tm.upsert(&tu2).unwrap();

        // 1. Paginated retrieval
        let (tus, total) = tm.list_tus_paginated(0, 10, None).unwrap();
        assert_eq!(total, 2);
        assert_eq!(tus.len(), 2);

        // 2. Search
        let (filtered, count) = tm.list_tus_paginated(0, 10, Some("support")).unwrap();
        assert_eq!(count, 1);
        assert_eq!(filtered[0].source_text, "Contact Acme Corp support.");

        // 3. In-place update
        let tu_id = filtered[0].id.unwrap();
        tm.update_tu(tu_id, "Contact Acme Global support.", "Contactez le support Acme Global.").unwrap();
        let (updated, _) = tm.list_tus_paginated(0, 10, Some("Global")).unwrap();
        assert_eq!(updated.len(), 1);
        assert_eq!(updated[0].target_text, "Contactez le support Acme Global.");

        // 4. Batch replace
        let replaced = tm.batch_replace("Acme Corp", "Acme International", true).unwrap();
        assert_eq!(replaced, 1);
        let (all_tus, _) = tm.list_tus_paginated(0, 10, None).unwrap();
        let found = all_tus.iter().any(|t| t.target_text.contains("Acme International"));
        assert!(found);
    }
}

