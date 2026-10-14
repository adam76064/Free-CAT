// core/db/project_db.rs
// SQLite database for project registry, segments, history, and QA results.
// Uses rusqlite with FTS5 and WAL journal mode.

use std::path::PathBuf;
use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use crate::core::models::{Project, ProjectFile, Segment, SegmentHistory, QaResult, SegmentStatus};

pub struct ProjectDb {
    conn: Connection,
}

impl ProjectDb {
    /// Open (or create) the project database at `path`.
    pub fn open(path: PathBuf) -> Result<Self> {
        let conn = Connection::open(&path)
            .with_context(|| format!("Cannot open project DB at {:?}", path))?;

        // Performance pragmas
        conn.execute_batch("
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA foreign_keys = ON;
            PRAGMA cache_size = -32000;
        ")?;

        let mut db = ProjectDb { conn };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&mut self) -> Result<()> {
        self.conn.execute_batch("
            CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY
            );
        ")?;

        let version: i64 = self.conn
            .query_row("SELECT COALESCE(MAX(version), 0) FROM schema_version", [], |r| r.get(0))
            .unwrap_or(0);

        if version < 1 {
            self.migrate_v1()?;
        }
        Ok(())
    }

    fn migrate_v1(&mut self) -> Result<()> {
        self.conn.execute_batch("
            -- Projects registry
            CREATE TABLE IF NOT EXISTS projects (
                id          TEXT PRIMARY KEY,
                name        TEXT NOT NULL,
                src_lang    TEXT NOT NULL,
                tgt_lang    TEXT NOT NULL,
                created_at  TEXT NOT NULL,
                modified_at TEXT NOT NULL,
                settings    TEXT NOT NULL DEFAULT '{}'
            );

            -- Files within a project
            CREATE TABLE IF NOT EXISTS project_files (
                id          TEXT PRIMARY KEY,
                project_id  TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
                filename    TEXT NOT NULL,
                format      TEXT NOT NULL,
                source_path TEXT NOT NULL,
                status      TEXT NOT NULL DEFAULT 'new',
                word_count  INTEGER DEFAULT 0,
                seg_count   INTEGER DEFAULT 0,
                added_at    TEXT NOT NULL
            );

            -- Translation segments
            CREATE TABLE IF NOT EXISTS segments (
                id              TEXT PRIMARY KEY,
                file_id         TEXT NOT NULL REFERENCES project_files(id) ON DELETE CASCADE,
                seq_number      INTEGER NOT NULL,
                source_text     TEXT NOT NULL,
                target_text     TEXT NOT NULL DEFAULT '',
                status          TEXT NOT NULL DEFAULT 'new',
                match_percent   INTEGER,
                match_origin    TEXT,
                locked          INTEGER NOT NULL DEFAULT 0,
                bookmarked      INTEGER NOT NULL DEFAULT 0,
                comment         TEXT,
                created_at      TEXT NOT NULL,
                modified_at     TEXT NOT NULL,
                modified_by     TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_segments_file_id ON segments(file_id);
            CREATE INDEX IF NOT EXISTS idx_segments_status  ON segments(status);

            -- Segment edit history (audit trail)
            CREATE TABLE IF NOT EXISTS segment_history (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                segment_id  TEXT NOT NULL REFERENCES segments(id) ON DELETE CASCADE,
                target_text TEXT NOT NULL,
                action      TEXT NOT NULL,
                actor       TEXT,
                timestamp   TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_history_segment ON segment_history(segment_id);

            -- QA check results
            CREATE TABLE IF NOT EXISTS qa_results (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id  TEXT NOT NULL,
                segment_id  TEXT REFERENCES segments(id) ON DELETE CASCADE,
                check_name  TEXT NOT NULL,
                severity    TEXT NOT NULL,
                message     TEXT NOT NULL,
                run_at      TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_qa_project ON qa_results(project_id);

            INSERT INTO schema_version VALUES (1);
        ")?;

        log::info!("ProjectDB: migrated to schema v1");
        Ok(())
    }

    // ── Project CRUD ──────────────────────────────────────────────────────────

    pub fn insert_project(&self, p: &Project) -> Result<()> {
        self.conn.execute(
            "INSERT INTO projects (id, name, src_lang, tgt_lang, created_at, modified_at, settings)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![p.id, p.name, p.src_lang, p.tgt_lang,
                    p.created_at, p.modified_at, p.settings],
        )?;
        Ok(())
    }

    pub fn get_project(&self, id: &str) -> Result<Option<Project>> {
        let result = self.conn.query_row(
            "SELECT id, name, src_lang, tgt_lang, created_at, modified_at, settings
             FROM projects WHERE id = ?1",
            params![id],
            |row| Ok(Project {
                id:          row.get(0)?,
                name:        row.get(1)?,
                src_lang:    row.get(2)?,
                tgt_lang:    row.get(3)?,
                created_at:  row.get(4)?,
                modified_at: row.get(5)?,
                settings:    row.get(6)?,
            }),
        );
        match result {
            Ok(p) => Ok(Some(p)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn list_projects(&self) -> Result<Vec<Project>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, src_lang, tgt_lang, created_at, modified_at, settings
             FROM projects ORDER BY modified_at DESC"
        )?;
        let rows = stmt.query_map([], |row| Ok(Project {
            id:          row.get(0)?,
            name:        row.get(1)?,
            src_lang:    row.get(2)?,
            tgt_lang:    row.get(3)?,
            created_at:  row.get(4)?,
            modified_at: row.get(5)?,
            settings:    row.get(6)?,
        }))?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn delete_project(&self, id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ── Project File CRUD ─────────────────────────────────────────────────────

    pub fn insert_file(&self, f: &ProjectFile) -> Result<()> {
        self.conn.execute(
            "INSERT INTO project_files (id, project_id, filename, format, source_path, status, word_count, seg_count, added_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![f.id, f.project_id, f.filename, f.format,
                    f.source_path, f.status, f.word_count, f.seg_count, f.added_at],
        )?;
        Ok(())
    }

    pub fn get_files_for_project(&self, project_id: &str) -> Result<Vec<ProjectFile>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, filename, format, source_path, status, word_count, seg_count, added_at
             FROM project_files WHERE project_id = ?1 ORDER BY added_at ASC"
        )?;
        let rows = stmt.query_map(params![project_id], |row| Ok(ProjectFile {
            id:          row.get(0)?,
            project_id:  row.get(1)?,
            filename:    row.get(2)?,
            format:      row.get(3)?,
            source_path: row.get(4)?,
            status:      row.get(5)?,
            word_count:  row.get(6)?,
            seg_count:   row.get(7)?,
            added_at:    row.get(8)?,
        }))?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    // ── Segment CRUD ──────────────────────────────────────────────────────────

    pub fn insert_segment(&self, s: &Segment) -> Result<()> {
        self.conn.execute(
            "INSERT INTO segments (id, file_id, seq_number, source_text, target_text, status,
             match_percent, match_origin, locked, bookmarked, comment, created_at, modified_at, modified_by)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                s.id, s.file_id, s.seq_number, s.source_text, s.target_text,
                s.status.as_str(), s.match_percent, s.match_origin,
                s.locked as i32, s.bookmarked as i32,
                s.comment, s.created_at, s.modified_at, s.modified_by
            ],
        )?;
        Ok(())
    }

    pub fn get_segments_for_file(&self, file_id: &str) -> Result<Vec<Segment>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_id, seq_number, source_text, target_text, status,
                    match_percent, match_origin, locked, bookmarked, comment,
                    created_at, modified_at, modified_by
             FROM segments WHERE file_id = ?1 ORDER BY seq_number ASC"
        )?;
        let rows = stmt.query_map(params![file_id], |row| {
            let status_str: String = row.get(5)?;
            Ok(Segment {
                id:            row.get(0)?,
                file_id:       row.get(1)?,
                seq_number:    row.get(2)?,
                source_text:   row.get(3)?,
                target_text:   row.get(4)?,
                status:        SegmentStatus::from_str(&status_str),
                match_percent: row.get(6)?,
                match_origin:  row.get(7)?,
                locked:        row.get::<_, i32>(8)? != 0,
                bookmarked:    row.get::<_, i32>(9)? != 0,
                comment:       row.get(10)?,
                created_at:    row.get(11)?,
                modified_at:   row.get(12)?,
                modified_by:   row.get(13)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn update_segment_target(&self, id: &str, target: &str, status: &SegmentStatus, modified_at: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE segments SET target_text = ?1, status = ?2, modified_at = ?3 WHERE id = ?4",
            params![target, status.as_str(), modified_at, id],
        )?;
        Ok(())
    }

    pub fn update_segment_full(
        &self,
        id: &str,
        target: &str,
        status: &SegmentStatus,
        match_percent: Option<i64>,
        match_origin: Option<&str>,
        modified_at: &str,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE segments
             SET target_text = ?1, status = ?2, match_percent = ?3, match_origin = ?4, modified_at = ?5
             WHERE id = ?6",
            params![target, status.as_str(), match_percent, match_origin, modified_at, id],
        )?;
        Ok(())
    }

    pub fn propagate_translation(
        &self,
        project_id: &str,
        source_text: &str,
        target_text: &str,
        modified_at: &str,
    ) -> Result<usize> {
        let count = self.conn.execute(
            "UPDATE segments
             SET target_text = ?1, status = 'draft', modified_at = ?2, match_origin = 'auto_propagate'
             WHERE source_text = ?3
               AND status IN ('new', 'draft')
               AND file_id IN (SELECT id FROM project_files WHERE project_id = ?4)",
            params![target_text, modified_at, source_text, project_id],
        )?;
        Ok(count)
    }

    pub fn get_segments_for_project(&self, project_id: &str) -> Result<Vec<Segment>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_id, seq_number, source_text, target_text, status,
                    match_percent, match_origin, locked, bookmarked, comment,
                    created_at, modified_at, modified_by
             FROM segments
             WHERE file_id IN (SELECT id FROM project_files WHERE project_id = ?1)
             ORDER BY seq_number ASC"
        )?;
        let rows = stmt.query_map(params![project_id], |row| {
            let status_str: String = row.get(5)?;
            Ok(Segment {
                id:            row.get(0)?,
                file_id:       row.get(1)?,
                seq_number:    row.get(2)?,
                source_text:   row.get(3)?,
                target_text:   row.get(4)?,
                status:        SegmentStatus::from_str(&status_str),
                match_percent: row.get(6)?,
                match_origin:  row.get(7)?,
                locked:        row.get::<_, i32>(8)? != 0,
                bookmarked:    row.get::<_, i32>(9)? != 0,
                comment:       row.get(10)?,
                created_at:    row.get(11)?,
                modified_at:   row.get(12)?,
                modified_by:   row.get(13)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn insert_history(&self, h: &SegmentHistory) -> Result<()> {
        self.conn.execute(
            "INSERT INTO segment_history (segment_id, target_text, action, actor, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![h.segment_id, h.target_text, h.action, h.actor, h.timestamp],
        )?;
        Ok(())
    }

    // ── QA Results ────────────────────────────────────────────────────────────

    pub fn insert_qa_result(&self, r: &QaResult) -> Result<()> {
        self.conn.execute(
            "INSERT INTO qa_results (project_id, segment_id, check_name, severity, message, run_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![r.project_id, r.segment_id, r.check_name, r.severity, r.message, r.run_at],
        )?;
        Ok(())
    }

    pub fn get_qa_results(&self, project_id: &str) -> Result<Vec<QaResult>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, segment_id, check_name, severity, message, run_at
             FROM qa_results WHERE project_id = ?1 ORDER BY id ASC"
        )?;
        let rows = stmt.query_map(params![project_id], |row| Ok(QaResult {
            id:          row.get(0)?,
            project_id:  row.get(1)?,
            segment_id:  row.get(2)?,
            check_name:  row.get(3)?,
            severity:    row.get(4)?,
            message:     row.get(5)?,
            run_at:      row.get(6)?,
        }))?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn clear_qa_results(&self, project_id: &str) -> Result<()> {
        self.conn.execute("DELETE FROM qa_results WHERE project_id = ?1", params![project_id])?;
        Ok(())
    }
}

impl std::fmt::Debug for ProjectDb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProjectDb")
    }
}
