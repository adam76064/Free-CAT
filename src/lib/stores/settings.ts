// src/lib/stores/settings.ts
// User preferences store — persisted to localStorage.

import { writable } from 'svelte/store';
import { browser } from '$app/environment';
import { setSecret } from '$lib/ipc/secrets';

export interface UserSettings {
  theme: 'light' | 'dark' | 'system';
  fontSize: number;       // editor font size in px (default 15)
  fontFamily: string;     // editor font (default 'Inter')
  defaultFuzzyThreshold: number; // 0.0–1.0
  autoSaveIntervalMs: number;    // milliseconds (default 2000)
  libretranslate_url: string;
  ollama_url: string;
  layoutMode: 'side-by-side' | 'stacked' | 'zen';
  sourceRatio: number;          // percent (default 50)
  inspectorWidth: number;       // px (default 340)
  targetOpticalBoost: boolean;  // +2px font size for target non-Latin scripts
  spellcheckEnabled: boolean;   // offline spellchecking & linter
  revisionMode: boolean;        // track changes redline revision mode
  // QA settings
  autoRunQa: boolean;
  qaCheckNumbers: boolean;
  qaCheckPunctuation: boolean;
  qaCheckDoubleSpaces: boolean;
  qaCheckTags: boolean;
  // Translation workflow
  autoPropagate: boolean;       // auto-propagate confirmed segments to identical source
}

const DEFAULT_SETTINGS: UserSettings = {
  theme: 'system',
  fontSize: 15,
  fontFamily: 'Inter',
  defaultFuzzyThreshold: 0.75,
  autoSaveIntervalMs: 2000,
  libretranslate_url: 'http://localhost:5000',
  ollama_url: 'http://localhost:11434',
  layoutMode: 'side-by-side',
  sourceRatio: 50,
  inspectorWidth: 340,
  targetOpticalBoost: false,
  spellcheckEnabled: true,
  revisionMode: false,
  autoRunQa: false,
  qaCheckNumbers: true,
  qaCheckPunctuation: true,
  qaCheckDoubleSpaces: true,
  qaCheckTags: true,
  autoPropagate: true,
};

/** Apply a theme to <html>. 'system' follows the OS preference. */
export function applyTheme(theme: UserSettings['theme']) {
  if (typeof document === 'undefined') return;
  const dark =
    theme === 'dark' ||
    (theme === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);
  document.documentElement.classList.toggle('dark', dark);
}

/**
 * One-time migration: earlier versions kept API keys in localStorage.
 * Move them into the OS keychain, then scrub them from localStorage.
 * localStorage is only scrubbed after the keychain write succeeds, so a
 * failure never loses a key.
 */
async function migrateLegacyKeys(raw: Record<string, unknown>) {
  const legacy: [string, 'deepl' | 'google_translate'][] = [
    ['deepl_api_key', 'deepl'],
    ['google_translate_key', 'google_translate'],
  ];
  let changed = false;
  for (const [field, name] of legacy) {
    if (!(field in raw)) continue;
    const value = raw[field];
    try {
      if (typeof value === 'string' && value.trim()) await setSecret(name, value);
      delete raw[field];
      changed = true;
    } catch (e) {
      console.warn(`Could not migrate ${field} to keychain; leaving it in place`, e);
    }
  }
  if (changed) localStorage.setItem('freecat_settings', JSON.stringify(raw));
}

function createSettingsStore() {
  // Load from localStorage if available
  const stored = browser
    ? localStorage.getItem('freecat_settings')
    : null;

  let parsed: Record<string, unknown> = {};
  try {
    parsed = stored ? JSON.parse(stored) : {};
  } catch {
    parsed = {}; // corrupt settings blob — fall back to defaults
  }
  if (browser && ('deepl_api_key' in parsed || 'google_translate_key' in parsed)) {
    void migrateLegacyKeys({ ...parsed });
  }
  // Never carry key fields into the in-memory store
  const { deepl_api_key: _d, google_translate_key: _g, ...safe } = parsed as Record<string, unknown>;

  const initial: UserSettings = { ...DEFAULT_SETTINGS, ...(safe as Partial<UserSettings>) };

  const { subscribe, set, update } = writable<UserSettings>(initial);

  // Apply the saved theme at startup (previously only applied on Save).
  if (browser) applyTheme(initial.theme);

  return {
    subscribe,
    update,
    set: (value: UserSettings) => {
      if (browser) {
        localStorage.setItem('freecat_settings', JSON.stringify(value));
        applyTheme(value.theme);
      }
      set(value);
    },
    reset: () => {
      if (browser) localStorage.removeItem('freecat_settings');
      set(DEFAULT_SETTINGS);
    }
  };
}

export const settings = createSettingsStore();
