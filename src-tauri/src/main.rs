// Free-CAT — Tauri application entry point
// Calls lib.rs::run() which sets up the Tauri app

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    free_cat_lib::run();
}
