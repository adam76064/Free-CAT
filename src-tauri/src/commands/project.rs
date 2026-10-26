// commands/project.rs — IPC handlers for project management

use tauri::State;
use anyhow::Result;
use uuid::Uuid;
use chrono::Utc;

use crate::state::{AppState, TmEntry};
use crate::core::models::{Project, ProjectFile, ProjectSettings, TranslationUnit};
use crate::core::db::tm_db::TmDb;

#[derive(serde::Deserialize)]
pub struct CreateProjectArgs {
    pub name:     String,
    pub src_lang: String,
    pub tgt_lang: String,
}

/// Create a new project and persist it.
#[tauri::command]
pub async fn create_project(
    args: CreateProjectArgs,
    state: State<'_, AppState>,
) -> Result<Project, String> {
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let id = Uuid::new_v4().to_string();

    // Auto-create dedicated Project TM
    let tm_id = format!("proj-tm-{}", id);
    let tm_name = format!("{} Project TM", args.name);
    let tm_path = state.data_dir.join("tm").join(format!("{}.db", tm_id));
    if let Ok(_tm) = TmDb::open(tm_id.clone(), tm_name.clone(), tm_path.clone()) {
        let entry = TmEntry {
            id: tm_id.clone(),
            name: tm_name,
            src_lang: args.src_lang.clone(),
            tgt_lang: args.tgt_lang.clone(),
            path: tm_path,
        };
        state.tm_index.lock().push(entry);
        let _ = state.save_tm_index();
    }

    let mut proj_settings = ProjectSettings::default();
    proj_settings.tm_ids.push(tm_id);
    let settings = serde_json::to_string(&proj_settings)
        .map_err(|e| e.to_string())?;

    let project = Project {
        id:          id.clone(),
        name:        args.name,
        src_lang:    args.src_lang,
        tgt_lang:    args.tgt_lang,
        created_at:  now.clone(),
        modified_at: now,
        settings,
    };

    // Create project directory
    let project_dir = state.data_dir.join("projects").join(&id);
    std::fs::create_dir_all(&project_dir)
        .map_err(|e| e.to_string())?;
    std::fs::create_dir_all(project_dir.join("source"))
        .map_err(|e| e.to_string())?;
    std::fs::create_dir_all(project_dir.join("translated"))
        .map_err(|e| e.to_string())?;

    // Persist to DB
    state.project_db
        .lock()
        .insert_project(&project)
        .map_err(|e| e.to_string())?;

    log::info!("Created project: {} ({}) with dedicated TM", project.name, project.id);
    Ok(project)
}

/// List all projects ordered by last modified.
#[tauri::command]
pub async fn list_projects(
    state: State<'_, AppState>,
) -> Result<Vec<Project>, String> {
    state.project_db
        .lock()
        .list_projects()
        .map_err(|e| e.to_string())
}

/// Get a single project by ID.
#[tauri::command]
pub async fn get_project(
    id: String,
    state: State<'_, AppState>,
) -> Result<Option<Project>, String> {
    state.project_db
        .lock()
        .get_project(&id)
        .map_err(|e| e.to_string())
}

/// Open a project — returns the project and its files.
/// Guarantees that a Project TM is linked and backfills any existing confirmed segments.
#[tauri::command]
pub async fn open_project(
    id: String,
    state: State<'_, AppState>,
) -> Result<(Project, Vec<ProjectFile>), String> {
    let mut project = {
        let db = state.project_db.lock();
        db.get_project(&id)
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("Project {} not found", id))?
    };

    let mut settings: ProjectSettings = serde_json::from_str(&project.settings).unwrap_or_default();
    let tm_id = format!("proj-tm-{}", project.id);
    let tm_name = format!("{} Project TM", project.name);
    let tm_path = state.data_dir.join("tm").join(format!("{}.db", tm_id));

    // Ensure TM exists in index
    {
        let mut tm_index = state.tm_index.lock();
        if !tm_index.iter().any(|e| e.id == tm_id) {
            if let Ok(_tm) = TmDb::open(tm_id.clone(), tm_name.clone(), tm_path.clone()) {
                tm_index.push(TmEntry {
                    id: tm_id.clone(),
                    name: tm_name.clone(),
                    src_lang: project.src_lang.clone(),
                    tgt_lang: project.tgt_lang.clone(),
                    path: tm_path.clone(),
                });
                let _ = state.save_tm_index();
            }
        }
    }

    // Ensure TM is attached to settings
    let mut modified = false;
    if !settings.tm_ids.contains(&tm_id) {
        settings.tm_ids.push(tm_id.clone());
        modified = true;
    }

    if modified {
        if let Ok(new_settings_json) = serde_json::to_string(&settings) {
            project.settings = new_settings_json;
            let db = state.project_db.lock();
            let _ = db.update_project_settings(&project.id, &project.settings);
        }
    }

    // Backfill any confirmed segments into the project TM
    if let Ok(tm) = TmDb::open(tm_id, tm_name, tm_path) {
        let db = state.project_db.lock();
        if let Ok(files) = db.get_files_for_project(&project.id) {
            for f in files {
                if let Ok(segs) = db.get_segments_for_file(&f.id) {
                    for s in segs {
                        if !s.target_text.trim().is_empty() {
                            let tu = TranslationUnit {
                                id: None,
                                source_lang: project.src_lang.clone(),
                                target_lang: project.tgt_lang.clone(),
                                source_text: s.source_text,
                                target_text: s.target_text,
                                created_at: s.created_at.clone(),
                                modified_at: s.modified_at,
                                created_by: None,
                                client: None,
                                domain: None,
                                source_file: Some(f.filename.clone()),
                                usage_count: Some(1),
                                metadata: None,
                            };
                            let _ = tm.upsert(&tu);
                        }
                    }
                }
            }
        }
    }

    let db = state.project_db.lock();
    let files = db.get_files_for_project(&id)
        .map_err(|e| e.to_string())?;
    Ok((project, files))
}

/// Delete a project and remove its directory.
#[tauri::command]
pub async fn delete_project(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.project_db
        .lock()
        .delete_project(&id)
        .map_err(|e| e.to_string())?;

    let project_dir = state.data_dir.join("projects").join(&id);
    if project_dir.exists() {
        std::fs::remove_dir_all(&project_dir).map_err(|e| e.to_string())?;
    }

    log::info!("Deleted project: {}", id);
    Ok(())
}

/// Export an entire project as a single-file portable .fcat package archive.
#[tauri::command]
pub async fn export_fcat(
    project_id: String,
    out_path: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let db = state.project_db.lock();
    let out = std::path::Path::new(&out_path);
    crate::core::packages::fcat::export_fcat(&project_id, &db, out)
        .map_err(|e| e.to_string())?;
    log::info!("Exported project {} to .fcat at {}", project_id, out_path);
    Ok(out_path)
}

/// Import a .fcat package archive into Free-CAT.
#[tauri::command]
pub async fn import_fcat(
    fcat_path: String,
    state: State<'_, AppState>,
) -> Result<Project, String> {
    let db = state.project_db.lock();
    let path = std::path::Path::new(&fcat_path);
    let proj = crate::core::packages::fcat::import_fcat(path, &db, &state.data_dir)
        .map_err(|e| e.to_string())?;
    log::info!("Imported .fcat package: {} (id: {})", proj.name, proj.id);
    Ok(proj)
}
