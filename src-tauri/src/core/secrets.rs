// core/secrets.rs — API key storage in the OS credential store.
//
// Keys live in Windows Credential Manager / macOS Keychain / Secret Service
// and are only ever read here, inside the Rust process. The webview can
// write or delete a key and ask whether one exists, but can never read it
// back, so a compromised frontend cannot exfiltrate stored credentials.

use keyring::Entry;

const SERVICE: &str = "com.freecat.app";

/// The only credential names the app understands.
pub const ALLOWED: &[&str] = &["deepl", "google_translate", "openai"];

fn entry(name: &str) -> Result<Entry, String> {
    if !ALLOWED.contains(&name) {
        return Err(format!("Unknown secret name: {name}"));
    }
    Entry::new(SERVICE, name).map_err(|e| format!("Keychain unavailable: {e}"))
}

/// Read a secret. Returns `None` if it is unset or the keychain is unavailable.
pub fn get(name: &str) -> Option<String> {
    entry(name).ok()?.get_password().ok().filter(|v| !v.is_empty())
}

/// Store a secret. An empty value deletes it.
pub fn set(name: &str, value: &str) -> Result<(), String> {
    let e = entry(name)?;
    if value.trim().is_empty() {
        return match e.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(err) => Err(format!("Failed to delete secret: {err}")),
        };
    }
    e.set_password(value.trim())
        .map_err(|err| format!("Failed to store secret: {err}"))
}

pub fn exists(name: &str) -> bool {
    get(name).is_some()
}

/// Use `provided` if non-empty, otherwise fall back to the stored secret.
pub fn resolve(provided: Option<String>, name: &str) -> Option<String> {
    match provided {
        Some(k) if !k.trim().is_empty() => Some(k),
        _ => get(name),
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_names() {
        assert!(set("../etc/passwd", "x").is_err());
        assert!(get("not-a-real-key").is_none());
        assert!(!exists("nope"));
    }

    #[test]
    fn resolve_prefers_provided_non_empty_value() {
        assert_eq!(resolve(Some("abc".into()), "deepl").as_deref(), Some("abc"));
    }

    #[test]
    fn resolve_ignores_blank_and_unknown_fallback() {
        // blank provided value falls through to the keychain; an unknown
        // name never resolves, so this is deterministic on any machine.
        assert_eq!(resolve(Some("   ".into()), "bogus"), None);
        assert_eq!(resolve(None, "bogus"), None);
    }
}
