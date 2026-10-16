// core/db/project_db.rs
// SQLite database for project registry, segments, history, and QA results.
// Uses rusqlite with FTS5 and WAL journal mode.

use std::path::PathBuf;
use anyhow::{Context, Result, anyhow};
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

    pub fn update_project_settings(&self, id: &str, settings: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE projects SET settings = ?1 WHERE id = ?2",
            params![settings, id],
        )?;
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

    pub fn get_segments_for_files(&self, file_ids: &[String]) -> Result<Vec<Segment>> {
        if file_ids.is_empty() {
            return Ok(Vec::new());
        }
        let placeholders: Vec<String> = (1..=file_ids.len()).map(|i| format!("?{}", i)).collect();
        let sql = format!(
            "SELECT id, file_id, seq_number, source_text, target_text, status,
                    match_percent, match_origin, locked, bookmarked, comment,
                    created_at, modified_at, modified_by
             FROM segments
             WHERE file_id IN ({})
             ORDER BY file_id, seq_number ASC",
            placeholders.join(",")
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let params = rusqlite::params_from_iter(file_ids);
        let rows = stmt.query_map(params, |row| {
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

    pub fn get_segment_by_id(&self, id: &str) -> Result<Option<Segment>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, file_id, seq_number, source_text, target_text, status,
                    match_percent, match_origin, locked, bookmarked, comment,
                    created_at, modified_at, modified_by
             FROM segments WHERE id = ?1"
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
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
        match rows.next() {
            Some(res) => Ok(Some(res?)),
            None => Ok(None),
        }
    }

    pub fn split_segment(&self, segment_id: &str, split_index: usize) -> Result<(Segment, Segment)> {
        let seg = self.get_segment_by_id(segment_id)?
            .ok_or_else(|| anyhow!("Segment not found: {}", segment_id))?;

        if split_index == 0 || split_index >= seg.source_text.len() {
            return Err(anyhow!("Invalid split index {} for source length {}", split_index, seg.source_text.len()));
        }

        // Ensure char boundary
        let char_idx = seg.source_text.char_indices()
            .map(|(i, _)| i)
            .find(|&i| i >= split_index)
            .unwrap_or(split_index);

        let part1_source = seg.source_text[..char_idx].trim().to_string();
        let part2_source = seg.source_text[char_idx..].trim().to_string();

        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let new_id = uuid::Uuid::new_v4().to_string();

        // Increment subsequent seq_numbers for this file
        self.conn.execute(
            "UPDATE segments SET seq_number = seq_number + 1 WHERE file_id = ?1 AND seq_number > ?2",
            params![seg.file_id, seg.seq_number],
        )?;

        // Update original segment to part 1
        self.conn.execute(
            "UPDATE segments SET source_text = ?1, modified_at = ?2 WHERE id = ?3",
            params![part1_source, now, seg.id],
        )?;

        // Insert new segment for part 2
        let new_seg = Segment {
            id: new_id.clone(),
            file_id: seg.file_id.clone(),
            seq_number: seg.seq_number + 1,
            source_text: part2_source.clone(),
            target_text: String::new(),
            status: SegmentStatus::New,
            match_percent: None,
            match_origin: None,
            locked: false,
            bookmarked: false,
            comment: None,
            created_at: now.clone(),
            modified_at: now.clone(),
            modified_by: None,
        };
        self.insert_segment(&new_seg)?;

        let mut updated_seg1 = seg;
        updated_seg1.source_text = part1_source;
        updated_seg1.modified_at = now;

        Ok((updated_seg1, new_seg))
    }

    pub fn merge_segments(&self, segment_id_1: &str, segment_id_2: &str) -> Result<Segment> {
        let seg1 = self.get_segment_by_id(segment_id_1)?
            .ok_or_else(|| anyhow!("Segment 1 not found: {}", segment_id_1))?;
        let seg2 = self.get_segment_by_id(segment_id_2)?
            .ok_or_else(|| anyhow!("Segment 2 not found: {}", segment_id_2))?;

        if seg1.file_id != seg2.file_id {
            return Err(anyhow!("Cannot merge segments from different files"));
        }

        let combined_source = format!("{} {}", seg1.source_text.trim(), seg2.source_text.trim());
        let combined_target = match (!seg1.target_text.trim().is_empty(), !seg2.target_text.trim().is_empty()) {
            (true, true) => format!("{} {}", seg1.target_text.trim(), seg2.target_text.trim()),
            (true, false) => seg1.target_text.clone(),
            (false, true) => seg2.target_text.clone(),
            (false, false) => String::new(),
        };

        let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

        // Update seg1 with combined text
        self.conn.execute(
            "UPDATE segments SET source_text = ?1, target_text = ?2, modified_at = ?3 WHERE id = ?4",
            params![combined_source, combined_target, now, seg1.id],
        )?;

        // Delete seg2
        self.conn.execute("DELETE FROM segments WHERE id = ?1", params![seg2.id])?;

        // Decrement seq_numbers of subsequent segments
        self.conn.execute(
            "UPDATE segments SET seq_number = seq_number - 1 WHERE file_id = ?1 AND seq_number > ?2",
            params![seg1.file_id, seg2.seq_number],
        )?;

        let mut updated_seg = seg1;
        updated_seg.source_text = combined_source;
        updated_seg.target_text = combined_target;
        updated_seg.modified_at = now;

        Ok(updated_seg)
    }

    pub fn toggle_bookmark(&self, segment_id: &str) -> Result<bool> {
        let current: i32 = self.conn.query_row(
            "SELECT bookmarked FROM segments WHERE id = ?1",
            params![segment_id],
            |r| r.get(0),
        )?;
        let new_state = if current == 0 { 1 } else { 0 };
        self.conn.execute(
            "UPDATE segments SET bookmarked = ?1 WHERE id = ?2",
            params![new_state, segment_id],
        )?;
        Ok(new_state != 0)
    }

    pub fn update_segment_comment(&self, segment_id: &str, comment: Option<&str>) -> Result<()> {
        self.conn.execute(
            "UPDATE segments SET comment = ?1 WHERE id = ?2",
            params![comment, segment_id],
        )?;
        Ok(())
    }

    pub fn get_history_for_segment(&self, segment_id: &str) -> Result<Vec<SegmentHistory>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, segment_id, target_text, action, actor, timestamp
             FROM segment_history WHERE segment_id = ?1 ORDER BY timestamp DESC"
        )?;
        let rows = stmt.query_map(params![segment_id], |row| Ok(SegmentHistory {
            id:          row.get(0)?,
            segment_id:  row.get(1)?,
            target_text: row.get(2)?,
            action:      row.get(3)?,
            actor:       row.get(4)?,
            timestamp:   row.get(5)?,
        }))?;
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
