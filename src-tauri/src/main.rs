// Free-CAT — Tauri application entry point
// Calls lib.rs::run() which sets up the Tauri app

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("Free-CAT PANIC: {:?}\n", info);
        eprintln!("{}", msg);
        if let Some(appdata) = std::env::var_os("APPDATA") {
            let mut path = std::path::PathBuf::from(appdata);
            path.push("com.freecat.app");
            let _ = std::fs::create_dir_all(&path);
            let _ = std::fs::write(path.join("panic.log"), &msg);
        }
    }));
    free_cat_lib::run();
}
