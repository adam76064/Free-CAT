// state/mod.rs — Global Tauri application state

pub mod app_state;
pub mod project_state;

pub use app_state::{AppState, TmEntry, TbEntry};
pub use project_state::ProjectState;
