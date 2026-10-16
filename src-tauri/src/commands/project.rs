// commands/project.rs — IPC handlers for project management

use tauri::State;
use anyhow::Result;
use uuid::Uuid;
use chrono::Utc;

use crate::state::AppState;
use crate::core::models::{Project, ProjectFile, ProjectSettings};

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

    let settings = serde_json::to_string(&ProjectSettings::default())
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
        .lock().unwrap()
        .insert_project(&project)
        .map_err(|e| e.to_string())?;

    log::info!("Created project: {} ({})", project.name, project.id);
    Ok(project)
}

/// List all projects ordered by last modified.
#[tauri::command]
pub async fn list_projects(
    state: State<'_, AppState>,
) -> Result<Vec<Project>, String> {
    state.project_db
        .lock().unwrap()
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
        .lock().unwrap()
        .get_project(&id)
        .map_err(|e| e.to_string())
}

/// Open a project — returns the project and its files.
#[tauri::command]
pub async fn open_project(
    id: String,
    state: State<'_, AppState>,
) -> Result<(Project, Vec<ProjectFile>), String> {
    let db = state.project_db.lock().unwrap();
    let project = db.get_project(&id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Project {} not found", id))?;
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
        .lock().unwrap()
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
    let db = state.project_db.lock().unwrap();
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
    let db = state.project_db.lock().unwrap();
    let path = std::path::Path::new(&fcat_path);
    let proj = crate::core::packages::fcat::import_fcat(path, &db, &state.data_dir)
        .map_err(|e| e.to_string())?;
    log::info!("Imported .fcat package: {} (id: {})", proj.name, proj.id);
    Ok(proj)
}
