// core/packages/fcat.rs — Free-CAT Portable Project Package (.fcat)
// Single-file archive encapsulating project metadata, files, segments, history, and QA results.

use std::path::Path;
use std::io::{Read, Write};
use anyhow::{Result, Context, anyhow};
use zip::{ZipArchive, ZipWriter, write::FileOptions};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

use crate::core::models::{Project, ProjectFile, Segment, SegmentHistory, QaResult};
use crate::core::db::project_db::ProjectDb;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FcatManifest {
    pub format_version: String,
    pub exported_at: String,
    pub project: Project,
    pub files: Vec<ProjectFile>,
    pub segments: Vec<Segment>,
    pub history: Vec<SegmentHistory>,
    pub qa_results: Vec<QaResult>,
}

pub fn export_fcat(
    project_id: &str,
    db: &ProjectDb,
    out_path: &Path,
) -> Result<()> {
    let project = db.get_project(project_id)?
        .ok_or_else(|| anyhow!("Project {} not found", project_id))?;

    let files = db.get_files_for_project(project_id)?;

    let mut all_segments = Vec::new();
    let mut all_history = Vec::new();
    let mut all_qa = Vec::new();

    for file in &files {
        let segs = db.get_segments_for_file(&file.id)?;
        for seg in &segs {
            if let Ok(hist) = db.get_history_for_segment(&seg.id) {
                all_history.extend(hist);
            }
        }
        all_segments.extend(segs);

        if let Ok(qa) = db.get_qa_results(&file.id) {
            all_qa.extend(qa);
        }
    }

    let manifest = FcatManifest {
        format_version: "1.0".to_string(),
        exported_at: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        project,
        files: files.clone(),
        segments: all_segments,
        history: all_history,
        qa_results: all_qa,
    };

    let manifest_json = serde_json::to_string_pretty(&manifest)?;

    let out_file = std::fs::File::create(out_path)
        .with_context(|| format!("Failed to create .fcat package at {:?}", out_path))?;
    let mut zip = ZipWriter::new(out_file);
    let options = FileOptions::<()>::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o644);

    // Write manifest.json
    zip.start_file("manifest.json", options)?;
    zip.write_all(manifest_json.as_bytes())?;

    // Write original source files if they exist
    for file in &files {
        let src_path = Path::new(&file.source_path);
        if src_path.exists() && src_path.is_file() {
            if let Ok(mut sf) = std::fs::File::open(src_path) {
                let entry_name = format!("source_files/{}", file.filename);
                if zip.start_file(entry_name, options).is_ok() {
                    let mut buf = Vec::new();
                    if sf.read_to_end(&mut buf).is_ok() {
                        let _ = zip.write_all(&buf);
                    }
                }
            }
        }
    }

    zip.finish()?;
    Ok(())
}

pub fn import_fcat(
    fcat_path: &Path,
    db: &ProjectDb,
    data_dir: &Path,
) -> Result<Project> {
    let file = std::fs::File::open(fcat_path)
        .with_context(|| format!("Failed to open .fcat package at {:?}", fcat_path))?;
    let mut zip = ZipArchive::new(file)
        .with_context(|| "Failed to read .fcat file as ZIP archive")?;

    let mut manifest_entry = zip.by_name("manifest.json")
        .with_context(|| "manifest.json missing in .fcat package")?;
    let mut manifest_str = String::new();
    manifest_entry.read_to_string(&mut manifest_str)?;
    drop(manifest_entry);

    let manifest: FcatManifest = serde_json::from_str(&manifest_str)
        .with_context(|| "Invalid manifest.json in .fcat package")?;

    // Determine target project ID and name
    let existing = db.get_project(&manifest.project.id)?;
    let (target_proj_id, target_proj_name) = if existing.is_some() {
        (
            Uuid::new_v4().to_string(),
            format!("{} (Imported)", manifest.project.name),
        )
    } else {
        (manifest.project.id.clone(), manifest.project.name.clone())
    };

    let proj_dir = data_dir.join("projects").join(&target_proj_id);
    std::fs::create_dir_all(&proj_dir)?;

    let id_remapped = target_proj_id != manifest.project.id;
    let mut file_id_map = std::collections::HashMap::new();

    // Import files and extract stored source files
    let mut imported_files = Vec::new();
    for orig_file in &manifest.files {
        let new_file_id = if id_remapped {
            Uuid::new_v4().to_string()
        } else {
            orig_file.id.clone()
        };
        file_id_map.insert(orig_file.id.clone(), new_file_id.clone());

        let target_src_path = proj_dir.join(&orig_file.filename);
        let entry_name = format!("source_files/{}", orig_file.filename);

        if let Ok(mut entry) = zip.by_name(&entry_name) {
            let mut buf = Vec::new();
            if entry.read_to_end(&mut buf).is_ok() {
                let _ = std::fs::write(&target_src_path, &buf);
            }
        }

        let new_file = ProjectFile {
            id: new_file_id,
            project_id: target_proj_id.clone(),
            filename: orig_file.filename.clone(),
            format: orig_file.format.clone(),
            source_path: target_src_path.to_string_lossy().to_string(),
            status: orig_file.status.clone(),
            word_count: orig_file.word_count,
            seg_count: orig_file.seg_count,
            added_at: orig_file.added_at.clone(),
        };
        imported_files.push(new_file);
    }

    // Create the project in DB
    let new_project = Project {
        id: target_proj_id.clone(),
        name: target_proj_name,
        src_lang: manifest.project.src_lang.clone(),
        tgt_lang: manifest.project.tgt_lang.clone(),
        created_at: manifest.project.created_at.clone(),
        modified_at: Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        settings: manifest.project.settings.clone(),
    };
    db.insert_project(&new_project)?;

    // Insert files
    for file in &imported_files {
        db.insert_file(file)?;
    }

    // Insert segments
    for seg in &manifest.segments {
        let target_file_id = if let Some(mapped) = file_id_map.get(&seg.file_id) {
            mapped.clone()
        } else {
            seg.file_id.clone()
        };

        let new_seg_id = if id_remapped {
            Uuid::new_v4().to_string()
        } else {
            seg.id.clone()
        };

        let s = Segment {
            id: new_seg_id,
            file_id: target_file_id,
            seq_number: seg.seq_number,
            source_text: seg.source_text.clone(),
            target_text: seg.target_text.clone(),
            status: seg.status.clone(),
            match_percent: seg.match_percent,
            match_origin: seg.match_origin.clone(),
            locked: seg.locked,
            bookmarked: seg.bookmarked,
            comment: seg.comment.clone(),
            created_at: seg.created_at.clone(),
            modified_at: seg.modified_at.clone(),
            modified_by: seg.modified_by.clone(),
        };
        db.insert_segment(&s)?;
    }

    // Insert history
    for h in &manifest.history {
        let target_seg_id = if id_remapped {
            h.segment_id.clone()
        } else {
            h.segment_id.clone()
        };
        let record = SegmentHistory {
            id: None,
            segment_id: target_seg_id,
            target_text: h.target_text.clone(),
            action: h.action.clone(),
            actor: h.actor.clone(),
            timestamp: h.timestamp.clone(),
        };
        let _ = db.insert_history(&record);
    }

    // Insert QA results
    for q in &manifest.qa_results {
        let q_rec = QaResult {
            id: None,
            project_id: target_proj_id.clone(),
            segment_id: q.segment_id.clone(),
            check_name: q.check_name.clone(),
            severity: q.severity.clone(),
            message: q.message.clone(),
            run_at: q.run_at.clone(),
        };
        let _ = db.insert_qa_result(&q_rec);
    }

    Ok(new_project)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fcat_export_and_import() {
        let dir = std::env::temp_dir().join(format!("freecat_test_{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("test_projects.db");
        let db = ProjectDb::open(db_path).unwrap();

        let proj = Project {
            id: "proj-1".into(),
            name: "Test Package Project".into(),
            src_lang: "en".into(),
            tgt_lang: "de".into(),
            created_at: "2026-10-16T12:00:00Z".into(),
            modified_at: "2026-10-16T12:00:00Z".into(),
            settings: "{}".into(),
        };
        db.insert_project(&proj).unwrap();

        let dummy_src = dir.join("sample.txt");
        std::fs::write(&dummy_src, "Hello world").unwrap();

        let file = ProjectFile {
            id: "file-1".into(),
            project_id: "proj-1".into(),
            filename: "sample.txt".into(),
            format: "txt".into(),
            source_path: dummy_src.to_string_lossy().to_string(),
            status: "new".into(),
            word_count: 2,
            seg_count: 1,
            added_at: "2026-10-16T12:00:00Z".into(),
        };
        db.insert_file(&file).unwrap();

        let seg = Segment {
            id: "seg-1".into(),
            file_id: "file-1".into(),
            seq_number: 1,
            source_text: "Hello world".into(),
            target_text: "Hallo Welt".into(),
            status: crate::core::models::SegmentStatus::Translated,
            match_percent: Some(100),
            match_origin: Some("User".into()),
            locked: false,
            bookmarked: false,
            comment: Some("Test note".into()),
            created_at: "2026-10-16T12:00:00Z".into(),
            modified_at: "2026-10-16T12:00:00Z".into(),
            modified_by: Some("tester".into()),
        };
        db.insert_segment(&seg).unwrap();

        let fcat_file = dir.join("export.fcat");
        export_fcat("proj-1", &db, &fcat_file).unwrap();
        assert!(fcat_file.exists());

        // Import into new DB
        let db_import_path = dir.join("imported_projects.db");
        let db_import = ProjectDb::open(db_import_path).unwrap();
        let imported = import_fcat(&fcat_file, &db_import, &dir).unwrap();

        assert_eq!(imported.name, "Test Package Project");
        let imported_files = db_import.get_files_for_project(&imported.id).unwrap();
        assert_eq!(imported_files.len(), 1);
        assert_eq!(imported_files[0].filename, "sample.txt");

        let imported_segs = db_import.get_segments_for_file(&imported_files[0].id).unwrap();
        assert_eq!(imported_segs.len(), 1);
        assert_eq!(imported_segs[0].target_text, "Hallo Welt");
        assert_eq!(imported_segs[0].comment.as_deref(), Some("Test note"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
