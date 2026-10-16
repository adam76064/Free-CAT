// src/lib/stores/settings.ts
// User preferences store — persisted to localStorage.

import { writable } from 'svelte/store';
import { browser } from '$app/environment';

export interface UserSettings {
  theme: 'light' | 'dark' | 'system';
  fontSize: number;       // editor font size in px (default 15)
  fontFamily: string;     // editor font (default 'Inter')
  defaultFuzzyThreshold: number; // 0.0–1.0
  autoSaveIntervalMs: number;    // milliseconds (default 2000)
  deepl_api_key: string;
  libretranslate_url: string;
  google_translate_key: string;
  ollama_url: string;
  layoutMode: 'side-by-side' | 'stacked' | 'zen';
  sourceRatio: number;          // percent (default 50)
  inspectorWidth: number;       // px (default 340)
  targetOpticalBoost: boolean;  // +2px font size for target non-Latin scripts
  spellcheckEnabled: boolean;   // offline spellchecking & linter
  revisionMode: boolean;        // track changes redline revision mode
}

const DEFAULT_SETTINGS: UserSettings = {
  theme: 'system',
  fontSize: 15,
  fontFamily: 'Inter',
  defaultFuzzyThreshold: 0.75,
  autoSaveIntervalMs: 2000,
  deepl_api_key: '',
  libretranslate_url: 'http://localhost:5000',
  google_translate_key: '',
  ollama_url: 'http://localhost:11434',
  layoutMode: 'side-by-side',
  sourceRatio: 50,
  inspectorWidth: 340,
  targetOpticalBoost: false,
  spellcheckEnabled: true,
  revisionMode: false,
};

function createSettingsStore() {
  // Load from localStorage if available
  const stored = browser
    ? localStorage.getItem('freecat_settings')
    : null;

  const initial: UserSettings = stored
    ? { ...DEFAULT_SETTINGS, ...JSON.parse(stored) }
    : DEFAULT_SETTINGS;

  const { subscribe, set, update } = writable<UserSettings>(initial);

  return {
    subscribe,
    update,
    set: (value: UserSettings) => {
      if (browser) {
        localStorage.setItem('freecat_settings', JSON.stringify(value));
        // Apply theme
        if (value.theme === 'dark') {
          document.documentElement.classList.add('dark');
        } else if (value.theme === 'light') {
          document.documentElement.classList.remove('dark');
        } else {
          const systemDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
          document.documentElement.classList.toggle('dark', systemDark);
        }
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
