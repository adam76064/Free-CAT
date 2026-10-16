// commands/aligner.rs — IPC command handlers for the Bilingual File Aligner

use std::path::Path;
use tauri::State;
use serde::Deserialize;

use crate::state::AppState;
use crate::core::aligner::{AlignedPair, align_raw_texts, align_sentences, export_pairs_to_tmx};
use crate::core::parsers::{detect_format, parse_file};
use crate::core::models::TranslationUnit;
use crate::core::db::tm_db::TmDb;

#[derive(Deserialize)]
pub struct AlignTextsArgs {
    pub source_text: String,
    pub target_text: String,
    pub src_lang: String,
    pub tgt_lang: String,
}

#[tauri::command]
pub async fn align_texts(
    args: AlignTextsArgs,
) -> Result<Vec<AlignedPair>, String> {
    Ok(align_raw_texts(
        &args.source_text,
        &args.target_text,
        &args.src_lang,
        &args.tgt_lang,
    ))
}

#[derive(Deserialize)]
pub struct AlignFilesArgs {
    pub source_path: String,
    pub target_path: String,
    pub src_lang: String,
    pub tgt_lang: String,
}

#[tauri::command]
pub async fn align_files(
    args: AlignFilesArgs,
) -> Result<Vec<AlignedPair>, String> {
    let src_p = Path::new(&args.source_path);
    let tgt_p = Path::new(&args.target_path);

    if !src_p.exists() {
        return Err(format!("Source file does not exist: {}", args.source_path));
    }
    if !tgt_p.exists() {
        return Err(format!("Target file does not exist: {}", args.target_path));
    }

    let src_fmt = detect_format(src_p).unwrap_or_else(|| "txt".into());
    let tgt_fmt = detect_format(tgt_p).unwrap_or_else(|| "txt".into());

    let src_raw = parse_file(src_p, &src_fmt).map_err(|e| format!("Failed to parse source file: {}", e))?;
    let tgt_raw = parse_file(tgt_p, &tgt_fmt).map_err(|e| format!("Failed to parse target file: {}", e))?;

    let src_sentences: Vec<String> = src_raw.into_iter().map(|s| s.source_text).collect();
    let tgt_sentences: Vec<String> = tgt_raw.into_iter().map(|s| s.source_text).collect();

    Ok(align_sentences(&src_sentences, &tgt_sentences))
}

#[derive(Deserialize)]
pub struct ExportAlignmentTmxArgs {
    pub pairs: Vec<AlignedPair>,
    pub src_lang: String,
    pub tgt_lang: String,
    pub output_path: String,
}

#[tauri::command]
pub async fn export_alignment_tmx(
    args: ExportAlignmentTmxArgs,
) -> Result<String, String> {
    let xml = export_pairs_to_tmx(&args.pairs, &args.src_lang, &args.tgt_lang);
    std::fs::write(&args.output_path, xml)
        .map_err(|e| format!("Failed to write TMX file: {}", e))?;
    Ok(args.output_path)
}

#[derive(Deserialize)]
pub struct ImportAlignmentTmArgs {
    pub tm_id: String,
    pub pairs: Vec<AlignedPair>,
    pub src_lang: String,
    pub tgt_lang: String,
}

#[tauri::command]
pub async fn import_alignment_to_tm(
    args: ImportAlignmentTmArgs,
    state: State<'_, AppState>,
) -> Result<usize, String> {
    let tm_entry = state.tm_index.lock().unwrap()
        .iter()
        .find(|e| e.id == args.tm_id)
        .cloned()
        .ok_or_else(|| format!("TM {} not found", args.tm_id))?;

    let tm = TmDb::open(tm_entry.id, tm_entry.name, tm_entry.path)
        .map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let mut count = 0;

    for p in args.pairs {
        if p.confirmed && !p.source_text.trim().is_empty() && !p.target_text.trim().is_empty() {
            let tu = TranslationUnit {
                id: None,
                source_lang: args.src_lang.clone(),
                target_lang: args.tgt_lang.clone(),
                source_text: p.source_text,
                target_text: p.target_text,
                created_at: now.clone(),
                modified_at: now.clone(),
                created_by: Some("Free-CAT Aligner".into()),
                client: None,
                domain: None,
                source_file: None,
                usage_count: None,
                metadata: None,
            };
            tm.upsert(&tu).map_err(|e| e.to_string())?;
            count += 1;
        }
    }

    Ok(count)
}
