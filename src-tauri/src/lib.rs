// Free-CAT — lib.rs
// Tauri application setup: registers all IPC commands and plugins.

use tauri::Manager;

pub mod commands;
pub mod core;
pub mod state;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // ── Plugins ─────────────────────────────────────────────────────────
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Debug)
                .build(),
        )
        // ── App state setup ─────────────────────────────────────────────────
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to resolve app data dir");

            // Ensure ~/.freecat/ exists
            std::fs::create_dir_all(&app_data_dir)?;
            std::fs::create_dir_all(app_data_dir.join("projects"))?;
            std::fs::create_dir_all(app_data_dir.join("tm"))?;
            std::fs::create_dir_all(app_data_dir.join("tb"))?;

            log::info!("Free-CAT data dir: {:?}", app_data_dir);

            // Initialise shared application state
            let app_state = AppState::new(app_data_dir)
                .expect("Failed to initialise AppState");

            app.manage(app_state);
            Ok(())
        })
        // ── IPC command handlers ─────────────────────────────────────────────
        .invoke_handler(tauri::generate_handler![
            // Project commands
            commands::project::create_project,
            commands::project::open_project,
            commands::project::list_projects,
            commands::project::delete_project,
            commands::project::get_project,
            commands::project::export_fcat,
            commands::project::import_fcat,
            // Segment commands
            commands::segment::get_segments,
            commands::segment::update_segment,
            commands::segment::confirm_segment,
            commands::segment::get_segment,
            commands::segment::split_segment,
            commands::segment::merge_segments,
            commands::segment::toggle_bookmark,
            commands::segment::update_segment_comment,
            commands::segment::get_segment_history,
            // TM commands
            commands::tm::query_tm,
            commands::tm::add_to_tm,
            commands::tm::import_tmx,
            commands::tm::export_tmx,
            commands::tm::list_tms,
            commands::tm::create_tm,
            commands::tm::delete_tm,
            commands::tm::get_tm_stats,
            commands::tm::concordance_search,
            // Termbase commands
            commands::termbase::query_tb,
            commands::termbase::add_term,
            commands::termbase::import_tbx,
            commands::termbase::export_tbx,
            commands::termbase::list_tbs,
            commands::termbase::create_tb,
            commands::termbase::delete_tb,
            commands::termbase::extract_terms,
            commands::termbase::get_terms,
            commands::termbase::delete_term,
            // QA commands
            commands::qa::run_qa,
            commands::qa::get_qa_results,
            commands::qa::clear_qa_results,
            // Analysis commands
            commands::analysis::analyze_project,
            commands::analysis::get_analysis_report,
            commands::analysis::get_detailed_analysis,
            commands::analysis::pre_translate,
            // MT commands
            commands::mt::get_mt_suggestion,
            // File import/export commands
            commands::file_io::import_file,
            commands::file_io::export_file,
            commands::file_io::import_sdlppx,
            commands::file_io::export_sdlrpx,
            // AI assistant commands
            commands::ai::query_ai_assistant,
            // Aligner commands
            commands::aligner::align_texts,
            commands::aligner::align_files,
            commands::aligner::export_alignment_tmx,
            commands::aligner::import_alignment_to_tm,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Free-CAT");
}
