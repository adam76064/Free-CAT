// commands/termbase.rs — IPC handlers for termbase operations

use tauri::State;
use uuid::Uuid;
use chrono::Utc;

use crate::state::{AppState, TbEntry};
use crate::core::models::TermEntry;
use crate::core::db::tb_db::TbDb;

/// Find termbase matches in a given text.
#[tauri::command]
pub async fn query_tb(
    text: String,
    lang: String,
    tb_ids: Vec<String>,
    state: State<'_, AppState>,
) -> Result<Vec<TermEntry>, String> {
    let index = state.tb_index.lock().unwrap().clone();
    let mut all: Vec<TermEntry> = Vec::new();

    for entry in index.iter().filter(|e| tb_ids.contains(&e.id)) {
        let tb = TbDb::open(entry.id.clone(), entry.name.clone(), entry.path.clone())
            .map_err(|e| e.to_string())?;
        let mut matches = tb.find_in_text(&text, &lang)
            .map_err(|e| e.to_string())?;
        all.append(&mut matches);
    }

    // Deduplicate by term
    let mut seen = std::collections::HashSet::new();
    let unique: Vec<TermEntry> = all.into_iter()
        .filter(|t| seen.insert(t.term.to_lowercase()))
        .collect();

    Ok(unique)
}

#[derive(serde::Deserialize)]
pub struct AddTermArgs {
    pub tb_id:      String,
    pub lang:       String,
    pub term:       String,
    pub definition: Option<String>,
    pub domain:     Option<String>,
    pub note:       Option<String>,
    pub status:     Option<String>,
}

/// Add a term to a termbase.
#[tauri::command]
pub async fn add_term(
    args: AddTermArgs,
    state: State<'_, AppState>,
) -> Result<i64, String> {
    let entry = state.tb_index.lock().unwrap()
        .iter().find(|e| e.id == args.tb_id).cloned()
        .ok_or_else(|| format!("TB {} not found", args.tb_id))?;

    let tb = TbDb::open(entry.id, entry.name, entry.path)
        .map_err(|e| e.to_string())?;

    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let term = TermEntry {
        id:         None,
        lang:       args.lang,
        term:       args.term,
        definition: args.definition,
        domain:     args.domain,
        note:       args.note,
        status:     args.status.unwrap_or_else(|| "preferred".to_string()),
        created_at: now,
        metadata:   None,
    };

    tb.insert_term(&term).map_err(|e| e.to_string())
}

/// Import a TBX file into an existing termbase.
#[tauri::command]
pub async fn import_tbx(
    tb_id:    String,
    tbx_path: String,
    state: State<'_, AppState>,
) -> Result<u64, String> {
    let entry = state.tb_index.lock().unwrap()
        .iter().find(|e| e.id == tb_id).cloned()
        .ok_or_else(|| format!("TB {} not found", tb_id))?;

    let tb = TbDb::open(entry.id, entry.name, entry.path)
        .map_err(|e| e.to_string())?;

    let terms = crate::core::parsers::tbx::parse_tbx(std::path::Path::new(&tbx_path))
        .map_err(|e| e.to_string())?;

    let mut count = 0u64;
    for t in &terms {
        if tb.insert_term(t).is_ok() {
            count += 1;
        }
    }

    log::info!("Imported {} terms into TB {}", count, tb_id);
    Ok(count)
}

/// Export a termbase to TBX format.
#[tauri::command]
pub async fn export_tbx(
    tb_id:    String,
    out_path: String,
    state: State<'_, AppState>,
) -> Result<u64, String> {
    let entry = state.tb_index.lock().unwrap()
        .iter().find(|e| e.id == tb_id).cloned()
        .ok_or_else(|| format!("TB {} not found", tb_id))?;

    let tb = TbDb::open(entry.id, entry.name, entry.path)
        .map_err(|e| e.to_string())?;

    let terms = tb.get_all().map_err(|e| e.to_string())?;
    crate::core::parsers::tbx::export_tbx(&terms, &out_path)
        .map_err(|e| e.to_string())?;

    log::info!("Exported {} terms from TB {} to {}", terms.len(), tb_id, out_path);
    Ok(terms.len() as u64)
}

/// List all available termbases.
#[tauri::command]
pub async fn list_tbs(
    state: State<'_, AppState>,
) -> Result<Vec<serde_json::Value>, String> {
    let index = state.tb_index.lock().unwrap().clone();
    let mut infos = Vec::new();
    for entry in index {
        if let Ok(tb) = TbDb::open(entry.id.clone(), entry.name.clone(), entry.path) {
            let count = tb.count().unwrap_or(0);
            infos.push(serde_json::json!({
                "id":    entry.id,
                "name":  entry.name,
                "count": count
            }));
        }
    }
    Ok(infos)
}

#[derive(serde::Deserialize)]
pub struct CreateTbArgs {
    pub name: String,
}

/// Create a new empty termbase.
#[tauri::command]
pub async fn create_tb(
    args: CreateTbArgs,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let id = Uuid::new_v4().to_string();
    let path = state.data_dir.join("tb").join(format!("{}.db", id));

    TbDb::open(id.clone(), args.name.clone(), path.clone())
        .map_err(|e| e.to_string())?;

    let entry = TbEntry {
        id:   id.clone(),
        name: args.name.clone(),
        path,
    };

    state.tb_index.lock().unwrap().push(entry);
    state.save_tb_index().map_err(|e| e.to_string())?;

    Ok(serde_json::json!({ "id": id, "name": args.name, "count": 0 }))
}

/// Delete a termbase.
#[tauri::command]
pub async fn delete_tb(
    tb_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let path = {
        let mut index = state.tb_index.lock().unwrap();
        let pos = index.iter().position(|e| e.id == tb_id)
            .ok_or_else(|| format!("TB {} not found", tb_id))?;
        index.remove(pos).path
    };

    state.save_tb_index().map_err(|e| e.to_string())?;
    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}
