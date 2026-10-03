// commands/tm.rs — IPC handlers for Translation Memory operations

use tauri::State;
use uuid::Uuid;
use chrono::Utc;
use std::path::PathBuf;

use crate::state::{AppState, TmEntry};
use crate::core::models::{TmMatchResult, TranslationUnit, TmInfo};
use crate::core::db::tm_db::TmDb;
use crate::core::parsers::tmx;

#[derive(serde::Deserialize)]
pub struct QueryTmArgs {
    pub source_text: String,
    pub src_lang:    String,
    pub tgt_lang:    String,
    pub tm_ids:      Vec<String>,
    pub threshold:   f64,
    pub prev_seg:    Option<String>,
    pub next_seg:    Option<String>,
    pub penalties:   std::collections::HashMap<String, f64>,
}

/// Query one or more TMs for matches against a source segment.
/// Returns merged, ranked results (up to 5).
#[tauri::command]
pub async fn query_tm(
    args: QueryTmArgs,
    state: State<'_, AppState>,
) -> Result<Vec<TmMatchResult>, String> {
    let tm_index = state.tm_index.lock().unwrap().clone();

    let mut all_results: Vec<TmMatchResult> = Vec::new();

    for tm_entry in tm_index.iter().filter(|e| args.tm_ids.contains(&e.id)) {
        let penalty = args.penalties.get(&tm_entry.id).copied().unwrap_or(0.0);
        let tm = TmDb::open(
            tm_entry.id.clone(),
            tm_entry.name.clone(),
            tm_entry.path.clone(),
        ).map_err(|e| e.to_string())?;

        let mut matches = tm.query(
            &args.source_text,
            &args.src_lang,
            &args.tgt_lang,
            args.threshold,
            5,
            penalty,
        ).map_err(|e| e.to_string())?;

        all_results.append(&mut matches);
    }

    // Sort descending by score, deduplicate by target
    all_results.sort_by(|a, b| b.score_pct.cmp(&a.score_pct));
    let mut seen = std::collections::HashSet::new();
    let results: Vec<TmMatchResult> = all_results
        .into_iter()
        .filter(|m| seen.insert(m.target_text.clone()))
        .take(5)
        .collect();

    Ok(results)
}

#[derive(serde::Deserialize)]
pub struct AddToTmArgs {
    pub tm_id:       String,
    pub source_text: String,
    pub target_text: String,
    pub src_lang:    String,
    pub tgt_lang:    String,
    pub source_file: Option<String>,
}

/// Add or update a translation unit in a TM (write-back on segment confirm).
#[tauri::command]
pub async fn add_to_tm(
    args: AddToTmArgs,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let tm_entry = state.tm_index.lock().unwrap()
        .iter()
        .find(|e| e.id == args.tm_id)
        .cloned()
        .ok_or_else(|| format!("TM {} not found", args.tm_id))?;

    let tm = TmDb::open(tm_entry.id, tm_entry.name, tm_entry.path)
        .map_err(|e| e.to_string())?;

    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let tu = TranslationUnit {
        id:          None,
        source_lang: args.src_lang,
        target_lang: args.tgt_lang,
        source_text: args.source_text,
        target_text: args.target_text,
        created_at:  now.clone(),
        modified_at: now,
        created_by:  None,
        client:      None,
        domain:      None,
        source_file: args.source_file,
        usage_count: None,
        metadata:    None,
    };

    tm.upsert(&tu).map_err(|e| e.to_string())?;
    Ok(())
}

/// Import a TMX file into an existing TM.
#[tauri::command]
pub async fn import_tmx(
    tm_id: String,
    tmx_path: String,
    state: State<'_, AppState>,
) -> Result<u64, String> {
    let tm_entry = state.tm_index.lock().unwrap()
        .iter()
        .find(|e| e.id == tm_id)
        .cloned()
        .ok_or_else(|| format!("TM {} not found", tm_id))?;

    let tm = TmDb::open(tm_entry.id, tm_entry.name, tm_entry.path)
        .map_err(|e| e.to_string())?;

    let units = tmx::parse_tmx_file(&tmx_path)
        .map_err(|e| e.to_string())?;

    let count = units.len() as u64;
    for tu in units {
        tm.upsert(&tu).map_err(|e| e.to_string())?;
    }

    log::info!("Imported {} TUs from TMX into TM {}", count, tm_id);
    Ok(count)
}

/// Export a TM to TMX format.
#[tauri::command]
pub async fn export_tmx(
    tm_id:    String,
    out_path: String,
    state: State<'_, AppState>,
) -> Result<u64, String> {
    let tm_entry = state.tm_index.lock().unwrap()
        .iter()
        .find(|e| e.id == tm_id)
        .cloned()
        .ok_or_else(|| format!("TM {} not found", tm_id))?;

    let tm = TmDb::open(tm_entry.id.clone(), tm_entry.name.clone(), tm_entry.path)
        .map_err(|e| e.to_string())?;

    let units = tm.get_all().map_err(|e| e.to_string())?;
    let count = units.len() as u64;

    tmx::write_tmx_file(&units, &tm_entry.src_lang, &tm_entry.tgt_lang, &out_path)
        .map_err(|e| e.to_string())?;

    log::info!("Exported {} TUs to TMX: {}", count, out_path);
    Ok(count)
}

/// List all available TMs.
#[tauri::command]
pub async fn list_tms(
    state: State<'_, AppState>,
) -> Result<Vec<TmInfo>, String> {
    let index = state.tm_index.lock().unwrap().clone();
    let mut infos = Vec::new();
    for entry in index {
        if let Ok(tm) = TmDb::open(entry.id.clone(), entry.name.clone(), entry.path) {
            let count = tm.count().unwrap_or(0);
            infos.push(TmInfo {
                id:       entry.id,
                name:     entry.name,
                src_lang: entry.src_lang,
                tgt_lang: entry.tgt_lang,
                tu_count: count,
            });
        }
    }
    Ok(infos)
}

#[derive(serde::Deserialize)]
pub struct CreateTmArgs {
    pub name:     String,
    pub src_lang: String,
    pub tgt_lang: String,
}

/// Create a new empty TM.
#[tauri::command]
pub async fn create_tm(
    args: CreateTmArgs,
    state: State<'_, AppState>,
) -> Result<TmInfo, String> {
    let id = Uuid::new_v4().to_string();
    let path = state.data_dir.join("tm").join(format!("{}.db", id));

    // Create the DB (initializes schema)
    let tm = TmDb::open(id.clone(), args.name.clone(), path.clone())
        .map_err(|e| e.to_string())?;

    let entry = TmEntry {
        id:       id.clone(),
        name:     args.name.clone(),
        src_lang: args.src_lang.clone(),
        tgt_lang: args.tgt_lang.clone(),
        path,
    };

    state.tm_index.lock().unwrap().push(entry);
    state.save_tm_index().map_err(|e| e.to_string())?;

    Ok(TmInfo {
        id,
        name:     args.name,
        src_lang: args.src_lang,
        tgt_lang: args.tgt_lang,
        tu_count: 0,
    })
}

/// Delete a TM and its database file.
#[tauri::command]
pub async fn delete_tm(
    tm_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let path = {
        let mut index = state.tm_index.lock().unwrap();
        let pos = index.iter().position(|e| e.id == tm_id)
            .ok_or_else(|| format!("TM {} not found", tm_id))?;
        let entry = index.remove(pos);
        entry.path
    };

    state.save_tm_index().map_err(|e| e.to_string())?;

    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    }

    log::info!("Deleted TM: {}", tm_id);
    Ok(())
}

/// Get stats for a TM (count, etc.)
#[tauri::command]
pub async fn get_tm_stats(
    tm_id: String,
    state: State<'_, AppState>,
) -> Result<TmInfo, String> {
    let entry = state.tm_index.lock().unwrap()
        .iter().find(|e| e.id == tm_id).cloned()
        .ok_or_else(|| format!("TM {} not found", tm_id))?;

    let tm = TmDb::open(entry.id.clone(), entry.name.clone(), entry.path)
        .map_err(|e| e.to_string())?;

    Ok(TmInfo {
        id:       entry.id,
        name:     entry.name,
        src_lang: entry.src_lang,
        tgt_lang: entry.tgt_lang,
        tu_count: tm.count().map_err(|e| e.to_string())?,
    })
}

/// Concordance search: full-text search across TM source and target.
#[tauri::command]
pub async fn concordance_search(
    query:   String,
    tm_ids:  Vec<String>,
    limit:   usize,
    state: State<'_, AppState>,
) -> Result<Vec<TranslationUnit>, String> {
    let index = state.tm_index.lock().unwrap().clone();
    let mut all: Vec<TranslationUnit> = Vec::new();

    for entry in index.iter().filter(|e| tm_ids.contains(&e.id)) {
        let tm = TmDb::open(entry.id.clone(), entry.name.clone(), entry.path.clone())
            .map_err(|e| e.to_string())?;
        let mut results = tm.concordance_search(&query, limit)
            .map_err(|e| e.to_string())?;
        all.append(&mut results);
    }

    all.truncate(limit);
    Ok(all)
}
