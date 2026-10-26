// src/lib/ipc/secrets.ts — write-only API key storage (OS keychain).
// Keys can be stored, removed, or checked for existence, but never read back
// into the webview. The Rust backend reads them itself when calling MT/AI APIs.
import { invoke } from '@tauri-apps/api/core';

export type SecretName = 'deepl' | 'google_translate' | 'openai';

/** Store a key. Passing an empty string deletes it. */
export async function setSecret(name: SecretName, value: string): Promise<void> {
  return invoke('set_secret', { name, value });
}

export async function hasSecret(name: SecretName): Promise<boolean> {
  return invoke('has_secret', { name });
}
