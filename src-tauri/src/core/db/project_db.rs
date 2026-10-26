// core/db/project_db.rs
// SQLite database for project registry, segments, history, and QA results.
// Uses rusqlite with FTS5 and WAL journal mode.

use std::path::PathBuf;
use anyhow::{Context, Result, anyhow};
use rusqlite::{Connection, params};
use crate::core::models::{Project, ProjectFile, Segment, SegmentHistory, QaResult, SegmentStatus, TqaError, TqaScorecard};

pub struct ProjectDb {
    conn: Connection,
}

/// One change in a batch update. `None` fields are left untouched.
#[derive(Debug, Clone, Default)]
pub struct SegmentPatch {
    pub id: String,
    pub target_text: Option<String>,
    pub status: Option<SegmentStatus>,
    pub locked: Option<bool>,
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

        // Ensure TQA tables exist
        self.conn.execute_batch("
            CREATE TABLE IF NOT EXISTS tqa_errors (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id  TEXT NOT NULL,
                segment_id  TEXT NOT NULL REFERENCES segments(id) ON DELETE CASCADE,
                category    TEXT NOT NULL,
                sub_type    TEXT,
                severity    TEXT NOT NULL,
                penalty     INTEGER NOT NULL,
                comment     TEXT,
                created_at  TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_tqa_project ON tqa_errors(project_id);
            CREATE INDEX IF NOT EXISTS idx_tqa_segment ON tqa_errors(segment_id);
        ")?;

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

    /// Apply many segment changes in ONE transaction (all-or-nothing).
    ///
    /// Unlike `toggle_segment_lock`, lock changes are explicit (`locked: Some(true)`),
    /// so re-running a batch is idempotent. `modified_at` is only bumped when the
    /// target text or status changes, matching the single-segment save path.
    /// Returns the number of segments actually changed (unknown ids are skipped).
    pub fn apply_segment_patches(&self, patches: &[SegmentPatch], modified_at: &str) -> Result<usize> {
        let tx = self.conn.unchecked_transaction()?;
        let mut changed = 0usize;
        {
            let mut stmt = tx.prepare_cached(
                "UPDATE segments SET
                    target_text = COALESCE(?1, target_text),
                    status      = COALESCE(?2, status),
                    locked      = COALESCE(?3, locked),
                    modified_at = CASE WHEN ?1 IS NOT NULL OR ?2 IS NOT NULL THEN ?4 ELSE modified_at END
                 WHERE id = ?5",
            )?;
            for p in patches {
                changed += stmt.execute(params![
                    p.target_text.as_deref(),
                    p.status.as_ref().map(|s| s.as_str()),
                    p.locked.map(|b| b as i32),
                    modified_at,
                    p.id,
                ])?;
            }
        }
        tx.commit()?;
        Ok(changed)
    }

    pub fn update_segment_source(&self, id: &str, new_source: &str, modified_at: &str) -> Result<Segment> {
        self.conn.execute(
            "UPDATE segments SET source_text = ?1, modified_at = ?2 WHERE id = ?3",
            params![new_source, modified_at, id],
        )?;

        // Record history entry for audit trail
        let _ = self.conn.execute(
            "INSERT INTO segment_history (segment_id, target_text, action, actor, timestamp)
             VALUES (?1, ?2, 'source_edit', 'translator', ?3)",
            params![id, new_source, modified_at],
        );

        self.get_segment_by_id(id)?
            .ok_or_else(|| anyhow!("Segment not found: {}", id))
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
               AND locked = 0
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

    pub fn toggle_segment_lock(&self, segment_id: &str) -> Result<bool> {
        let current: i32 = self.conn.query_row(
            "SELECT locked FROM segments WHERE id = ?1",
            params![segment_id],
            |r| r.get(0),
        )?;
        let new_state = if current == 0 { 1 } else { 0 };
        self.conn.execute(
            "UPDATE segments SET locked = ?1 WHERE id = ?2",
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

    // ── TQA (Translation Quality Assessment) ──────────────────────────────────

    pub fn insert_tqa_error(&self, err: &TqaError) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO tqa_errors (project_id, segment_id, category, sub_type, severity, penalty, comment, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                err.project_id,
                err.segment_id,
                err.category,
                err.sub_type,
                err.severity,
                err.penalty,
                err.comment,
                err.created_at
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn delete_tqa_error(&self, id: i64) -> Result<()> {
        self.conn.execute("DELETE FROM tqa_errors WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn get_tqa_errors_for_project(&self, project_id: &str) -> Result<Vec<TqaError>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, segment_id, category, sub_type, severity, penalty, comment, created_at
             FROM tqa_errors WHERE project_id = ?1 ORDER BY id ASC"
        )?;
        let rows = stmt.query_map(params![project_id], |row| Ok(TqaError {
            id:          Some(row.get(0)?),
            project_id:  row.get(1)?,
            segment_id:  row.get(2)?,
            category:    row.get(3)?,
            sub_type:    row.get(4)?,
            severity:    row.get(5)?,
            penalty:     row.get(6)?,
            comment:     row.get(7)?,
            created_at:  row.get(8)?,
        }))?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn get_tqa_errors_for_segment(&self, segment_id: &str) -> Result<Vec<TqaError>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, segment_id, category, sub_type, severity, penalty, comment, created_at
             FROM tqa_errors WHERE segment_id = ?1 ORDER BY id ASC"
        )?;
        let rows = stmt.query_map(params![segment_id], |row| Ok(TqaError {
            id:          Some(row.get(0)?),
            project_id:  row.get(1)?,
            segment_id:  row.get(2)?,
            category:    row.get(3)?,
            sub_type:    row.get(4)?,
            severity:    row.get(5)?,
            penalty:     row.get(6)?,
            comment:     row.get(7)?,
            created_at:  row.get(8)?,
        }))?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn get_project_tqa_scorecard(&self, project_id: &str, total_words: i64) -> Result<TqaScorecard> {
        let errors = self.get_tqa_errors_for_project(project_id)?;
        let mut minor_count = 0;
        let mut major_count = 0;
        let mut critical_count = 0;
        let mut total_penalty = 0;

        for e in &errors {
            match e.severity.to_lowercase().as_str() {
                "minor" => minor_count += 1,
                "major" => major_count += 1,
                "critical" => critical_count += 1,
                _ => {}
            }
            total_penalty += e.penalty;
        }

        let words = if total_words > 0 { total_words } else { 100 };
        // Industry DQF-MQM formula: Score % = max(0, 100 - (penalty / words) * 100)
        let penalty_ratio = (total_penalty as f64 / words as f64) * 100.0;
        let score_pct = (100.0 - penalty_ratio).clamp(0.0, 100.0);
        let passed = score_pct >= 95.0 && critical_count == 0;

        Ok(TqaScorecard {
            project_id: project_id.to_string(),
            total_words: words,
            total_errors: errors.len() as i64,
            minor_count,
            major_count,
            critical_count,
            total_penalty,
            score_pct: (score_pct * 10.0).round() / 10.0,
            passed,
            errors,
        })
    }
}

impl std::fmt::Debug for ProjectDb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ProjectDb")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A throwaway on-disk project DB (removed on drop) with `n` segments in file "f1".
    struct TempDb { db: ProjectDb, path: PathBuf }
    impl Drop for TempDb {
        fn drop(&mut self) {
            for ext in ["", "-wal", "-shm"] {
                let _ = std::fs::remove_file(format!("{}{}", self.path.display(), ext));
            }
        }
    }

    fn temp_db(n: usize) -> TempDb {
        let path = std::env::temp_dir().join(format!("freecat-test-{}.db", uuid::Uuid::new_v4()));
        let db = ProjectDb::open(path.clone()).unwrap();
        // Segments reference project files; this test only exercises segment rows.
        db.conn.execute_batch("PRAGMA foreign_keys = OFF;").unwrap();
        for i in 0..n {
            db.insert_segment(&Segment {
                id: format!("s{i}"),
                file_id: "f1".into(),
                seq_number: i as i64,
                source_text: format!("source {i}"),
                target_text: String::new(),
                status: SegmentStatus::New,
                match_percent: None,
                match_origin: None,
                locked: false,
                bookmarked: false,
                comment: None,
                created_at: "2026-01-01T00:00:00Z".into(),
                modified_at: "2026-01-01T00:00:00Z".into(),
                modified_by: None,
            }).unwrap();
        }
        TempDb { db, path }
    }

    fn patch(id: &str) -> SegmentPatch {
        SegmentPatch { id: id.into(), ..Default::default() }
    }

    #[test]
    fn batch_updates_target_status_and_lock_together() {
        let t = temp_db(3);
        let n = t.db.apply_segment_patches(&[
            SegmentPatch { target_text: Some("مرحبا".into()), status: Some(SegmentStatus::Draft), ..patch("s0") },
            SegmentPatch { locked: Some(true), ..patch("s1") },
        ], "2026-02-02T00:00:00Z").unwrap();
        assert_eq!(n, 2);

        let segs = t.db.get_segments_for_file("f1").unwrap();
        assert_eq!(segs[0].target_text, "مرحبا");
        assert_eq!(segs[0].status, SegmentStatus::Draft);
        assert_eq!(segs[0].modified_at, "2026-02-02T00:00:00Z");
        assert!(!segs[0].locked);
        // lock-only change leaves text/status/modified_at alone
        assert!(segs[1].locked);
        assert_eq!(segs[1].status, SegmentStatus::New);
        assert_eq!(segs[1].modified_at, "2026-01-01T00:00:00Z");
        // untouched segment
        assert_eq!(segs[2].target_text, "");
    }

    #[test]
    fn lock_changes_are_idempotent_unlike_toggle() {
        let t = temp_db(1);
        let lock = [SegmentPatch { locked: Some(true), ..patch("s0") }];
        t.db.apply_segment_patches(&lock, "x").unwrap();
        t.db.apply_segment_patches(&lock, "x").unwrap(); // re-run: still locked
        assert!(t.db.get_segments_for_file("f1").unwrap()[0].locked);
        t.db.apply_segment_patches(&[SegmentPatch { locked: Some(false), ..patch("s0") }], "x").unwrap();
        assert!(!t.db.get_segments_for_file("f1").unwrap()[0].locked);
    }

    #[test]
    fn unknown_ids_are_skipped_and_empty_batch_is_a_noop() {
        let t = temp_db(1);
        assert_eq!(t.db.apply_segment_patches(&[], "x").unwrap(), 0);
        let n = t.db.apply_segment_patches(&[
            SegmentPatch { locked: Some(true), ..patch("missing") },
            SegmentPatch { locked: Some(true), ..patch("s0") },
        ], "x").unwrap();
        assert_eq!(n, 1);
    }

    #[test]
    fn large_batch_runs_in_one_transaction_quickly() {
        let t = temp_db(20_000);
        let patches: Vec<SegmentPatch> = (0..20_000)
            .map(|i| SegmentPatch {
                target_text: Some(format!("t{i}")),
                status: Some(SegmentStatus::Draft),
                ..patch(&format!("s{i}"))
            })
            .collect();
        let start = std::time::Instant::now();
        assert_eq!(t.db.apply_segment_patches(&patches, "2026-03-03T00:00:00Z").unwrap(), 20_000);
        assert!(start.elapsed().as_secs() < 5, "took {:?}", start.elapsed());
        let segs = t.db.get_segments_for_file("f1").unwrap();
        assert_eq!(segs[19_999].target_text, "t19999");
    }
}
