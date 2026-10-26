// state/app_state.rs
// Global application state shared across all Tauri commands via tauri::State<AppState>.

use std::path::PathBuf;
use parking_lot::Mutex;
use anyhow::Result;

use crate::core::db::project_db::ProjectDb;

/// Top-level application state managed by Tauri.
/// Wrapped in Mutex so it can be safely shared across async command handlers.
pub struct AppState {
    /// Root data directory: ~/.freecat/ (or %APPDATA%\freecat on Windows)
    pub data_dir: PathBuf,
    /// Global project registry database (projects.db)
    pub project_db: Mutex<ProjectDb>,
    /// Index of available TMs: (tm_id, name, src_lang, tgt_lang, path)
    pub tm_index: Mutex<Vec<TmEntry>>,
    /// Index of available termbases: (tb_id, name, path)
    pub tb_index: Mutex<Vec<TbEntry>>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct TmEntry {
    pub id:       String,
    pub name:     String,
    pub src_lang: String,
    pub tgt_lang: String,
    pub path:     PathBuf,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug)]
pub struct TbEntry {
    pub id:   String,
    pub name: String,
    pub path: PathBuf,
}

impl AppState {
    pub fn new(data_dir: PathBuf) -> Result<Self> {
        let project_db = ProjectDb::open(data_dir.join("projects.db"))?;

        // Load TM index from disk (index.toml in tm/ dir)
        let tm_index = Self::load_tm_index(&data_dir)?;
        let tb_index = Self::load_tb_index(&data_dir)?;

        Ok(AppState {
            data_dir,
            project_db: Mutex::new(project_db),
            tm_index: Mutex::new(tm_index),
            tb_index: Mutex::new(tb_index),
        })
    }

    fn load_tm_index(data_dir: &PathBuf) -> Result<Vec<TmEntry>> {
        let index_path = data_dir.join("tm").join("index.toml");
        if !index_path.exists() {
            return Ok(vec![]);
        }
        let content = std::fs::read_to_string(&index_path)?;
        let index: TmIndexFile = toml::from_str(&content)?;
        Ok(index.tms)
    }

    fn load_tb_index(data_dir: &PathBuf) -> Result<Vec<TbEntry>> {
        let index_path = data_dir.join("tb").join("index.toml");
        if !index_path.exists() {
            return Ok(vec![]);
        }
        let content = std::fs::read_to_string(&index_path)?;
        let index: TbIndexFile = toml::from_str(&content)?;
        Ok(index.tbs)
    }

    pub fn save_tm_index(&self) -> Result<()> {
        let tms = self.tm_index.lock().clone();
        let index = TmIndexFile { tms };
        let content = toml::to_string_pretty(&index)?;
        let path = self.data_dir.join("tm").join("index.toml");
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn save_tb_index(&self) -> Result<()> {
        let tbs = self.tb_index.lock().clone();
        let index = TbIndexFile { tbs };
        let content = toml::to_string_pretty(&index)?;
        let path = self.data_dir.join("tb").join("index.toml");
        std::fs::write(path, content)?;
        Ok(())
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct TmIndexFile {
    #[serde(default)]
    tms: Vec<TmEntry>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct TbIndexFile {
    #[serde(default)]
    tbs: Vec<TbEntry>,
}
