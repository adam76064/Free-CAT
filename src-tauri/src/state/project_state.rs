// state/project_state.rs
// Per-project runtime state: the currently open project's DB handle and metadata.

use std::path::PathBuf;
use crate::core::db::project_db::ProjectDb;

/// Holds the currently open project.
/// This is stored as Option inside AppState or as a separate Tauri-managed state.
#[derive(Debug)]
pub struct ProjectState {
    pub project_id: String,
    pub project_db: ProjectDb,
    pub project_dir: PathBuf,
}
