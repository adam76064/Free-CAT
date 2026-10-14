// commands/file_io.rs — File import and export commands

use tauri::State;
use crate::state::AppState;
use crate::core::models::{ProjectFile, Segment};
use crate::core::parsers;

#[derive(serde::Deserialize)]
pub struct ImportFileArgs {
    pub project_id:  String,
    pub file_path:   String,  // absolute path to source file
}

#[derive(serde::Serialize)]
pub struct ImportResult {
    pub file:     ProjectFile,
    pub segments: Vec<Segment>,
}

/// Import a source file into a project: detect format, parse segments, store in DB.
#[tauri::command]
pub async fn import_file(
    args: ImportFileArgs,
    state: State<'_, AppState>,
) -> Result<ImportResult, String> {
    let path = std::path::Path::new(&args.file_path);
    let format = parsers::detect_format(path)
        .ok_or_else(|| format!("Unsupported file format: {:?}", path.extension()))?;

    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Parse segments from the source file
    let raw_segments = parsers::parse_file(path, &format)
        .map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let file_id = uuid::Uuid::new_v4().to_string();

    // Build ProjectFile record
    let project_file = ProjectFile {
        id:          file_id.clone(),
        project_id:  args.project_id.clone(),
        filename:    filename.clone(),
        format:      format.clone(),
        source_path: args.file_path.clone(),
        status:      "new".into(),
        word_count:  raw_segments.iter().map(|s| s.source_text.split_whitespace().count() as i64).sum(),
        seg_count:   raw_segments.len() as i64,
        added_at:    now.clone(),
    };

    // Build Segment records
    let segments: Vec<Segment> = raw_segments
        .into_iter()
        .enumerate()
        .map(|(i, raw)| Segment {
            id:            uuid::Uuid::new_v4().to_string(),
            file_id:       file_id.clone(),
            seq_number:    i as i64 + 1,
            source_text:   raw.source_text,
            target_text:   raw.target_text.unwrap_or_default(),
            status:        crate::core::models::SegmentStatus::New,
            match_percent: None,
            match_origin:  None,
            locked:        false,
            bookmarked:    false,
            comment:       None,
            created_at:    now.clone(),
            modified_at:   now.clone(),
            modified_by:   None,
        })
        .collect();

    // Persist to DB
    let db = state.project_db.lock().unwrap();
    db.insert_file(&project_file).map_err(|e| e.to_string())?;
    for seg in &segments {
        db.insert_segment(seg).map_err(|e| e.to_string())?;
    }

    // Copy source file to project directory
    let project_dir = state.data_dir.join("projects").join(&args.project_id).join("source");
    let dest = project_dir.join(&filename);
    if !dest.exists() {
        std::fs::copy(&args.file_path, &dest).map_err(|e| e.to_string())?;
    }

    log::info!("Imported '{}' → {} segments", filename, segments.len());
    Ok(ImportResult { file: project_file, segments })
}

/// Export translated segments back to the original file format.
#[tauri::command]
pub async fn export_file(
    file_id:    String,
    project_id: String,
    out_path:   Option<String>,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let db = state.project_db.lock().unwrap();
    let files = db.get_files_for_project(&project_id).map_err(|e| e.to_string())?;
    let file = files.iter().find(|f| f.id == file_id)
        .ok_or_else(|| format!("File {} not found", file_id))?
        .clone();

    let segments = db.get_segments_for_file(&file_id).map_err(|e| e.to_string())?;

    // Determine output path
    let output_path = out_path.unwrap_or_else(|| {
        state.data_dir
            .join("projects").join(&project_id).join("translated")
            .join(&file.filename)
            .to_string_lossy().to_string()
    });

    parsers::export_file(&file, &segments, &output_path)
        .map_err(|e| e.to_string())?;

    log::info!("Exported '{}' → {}", file.filename, output_path);
    Ok(output_path)
}

/// Import a Trados .sdlppx package.
#[tauri::command]
pub async fn import_sdlppx(
    _project_id: String,
    _package_path: String,
    _state: State<'_, AppState>,
) -> Result<Vec<ImportResult>, String> {
    // TODO: Phase 2 - extract .sdlxliff files from ZIP, import each
    log::warn!("SDLPPX import not yet implemented (Phase 2 feature)");
    Err("SDLPPX import will be implemented in v1.0".into())
}
