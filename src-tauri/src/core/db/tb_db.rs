// core/db/tb_db.rs
// Termbase (glossary) SQLite database.

use std::path::PathBuf;
use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use crate::core::models::TermEntry;

pub struct TbDb {
    pub id:   String,
    pub name: String,
    conn: Connection,
}

impl TbDb {
    pub fn open(id: String, name: String, path: PathBuf) -> Result<Self> {
        let conn = Connection::open(&path)
            .with_context(|| format!("Cannot open TB DB at {:?}", path))?;
        conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;")?;
        let mut db = TbDb { id, name, conn };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&mut self) -> Result<()> {
        self.conn.execute_batch("
            CREATE TABLE IF NOT EXISTS terms (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                lang        TEXT NOT NULL,
                term        TEXT NOT NULL,
                definition  TEXT,
                domain      TEXT,
                note        TEXT,
                status      TEXT NOT NULL DEFAULT 'preferred',
                created_at  TEXT NOT NULL DEFAULT (datetime('now')),
                metadata    TEXT
            );

            -- FTS for term lookup
            CREATE VIRTUAL TABLE IF NOT EXISTS tb_fts USING fts5(
                term,
                content='terms',
                content_rowid='id',
                tokenize='unicode61 remove_diacritics 2'
            );

            CREATE TRIGGER IF NOT EXISTS tb_ai AFTER INSERT ON terms BEGIN
                INSERT INTO tb_fts(rowid, term) VALUES (new.id, new.term);
            END;
            CREATE TRIGGER IF NOT EXISTS tb_au AFTER UPDATE ON terms BEGIN
                INSERT INTO tb_fts(tb_fts, rowid, term) VALUES ('delete', old.id, old.term);
                INSERT INTO tb_fts(rowid, term) VALUES (new.id, new.term);
            END;
            CREATE TRIGGER IF NOT EXISTS tb_ad AFTER DELETE ON terms BEGIN
                INSERT INTO tb_fts(tb_fts, rowid, term) VALUES ('delete', old.id, old.term);
            END;
        ")?;
        Ok(())
    }

    pub fn insert_term(&self, t: &TermEntry) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO terms (lang, term, definition, domain, note, status, created_at, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![t.lang, t.term, t.definition, t.domain, t.note, t.status, t.created_at, t.metadata],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn delete_term(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM terms WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Find terms that appear in the given text for a specific language.
    /// Returns matches sorted by term length descending (longest match first).
    pub fn find_in_text(&self, text: &str, lang: &str) -> Result<Vec<TermEntry>> {
        let text_lower = text.to_lowercase();
        let mut stmt = self.conn.prepare(
            "SELECT id, lang, term, definition, domain, note, status, created_at, metadata
             FROM terms WHERE lang = ?1"
        )?;
        let rows = stmt.query_map(params![lang], |r| Ok(TermEntry {
            id:         Some(r.get(0)?),
            lang:       r.get(1)?,
            term:       r.get(2)?,
            definition: r.get(3)?,
            domain:     r.get(4)?,
            note:       r.get(5)?,
            status:     r.get(6)?,
            created_at: r.get(7)?,
            metadata:   r.get(8)?,
        }))?;

        let mut matches: Vec<TermEntry> = rows
            .collect::<rusqlite::Result<Vec<_>>>()?
            .into_iter()
            .filter(|t| text_lower.contains(&t.term.to_lowercase()))
            .collect();

        // Longest match first to prefer "machine translation" over "translation"
        matches.sort_by(|a, b| b.term.len().cmp(&a.term.len()));
        Ok(matches)
    }

    pub fn get_all(&self) -> Result<Vec<TermEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, lang, term, definition, domain, note, status, created_at, metadata
             FROM terms ORDER BY term ASC"
        )?;
        let rows = stmt.query_map([], |r| Ok(TermEntry {
            id:         Some(r.get(0)?),
            lang:       r.get(1)?,
            term:       r.get(2)?,
            definition: r.get(3)?,
            domain:     r.get(4)?,
            note:       r.get(5)?,
            status:     r.get(6)?,
            created_at: r.get(7)?,
            metadata:   r.get(8)?,
        }))?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn count(&self) -> Result<i64> {
        Ok(self.conn.query_row("SELECT COUNT(*) FROM terms", [], |r| r.get(0))?)
    }
}

impl std::fmt::Debug for TbDb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TbDb({})", self.name)
    }
}
