// state/mod.rs — Global Tauri application state

use std::path::PathBuf;
use std::sync::Mutex;
use anyhow::Result;

pub mod app_state;
pub mod project_state;

pub use app_state::AppState;
pub use project_state::ProjectState;
