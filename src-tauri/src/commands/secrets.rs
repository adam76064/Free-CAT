// commands/secrets.rs — write-only IPC surface for API keys.

use crate::core::secrets;

#[tauri::command]
pub fn set_secret(name: String, value: String) -> Result<(), String> {
    secrets::set(&name, &value)
}

#[tauri::command]
pub fn has_secret(name: String) -> Result<bool, String> {
    if !secrets::ALLOWED.contains(&name.as_str()) {
        return Err(format!("Unknown secret name: {name}"));
    }
    Ok(secrets::exists(&name))
}
