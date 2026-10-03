// src/lib/stores/project.ts
// Svelte stores for project, files, and navigation state.

import { writable, derived } from 'svelte/store';

export interface Project {
  id: string;
  name: string;
  src_lang: string;
  tgt_lang: string;
  created_at: string;
  modified_at: string;
  settings: string;
}

export interface ProjectFile {
  id: string;
  project_id: string;
  filename: string;
  format: string;
  source_path: string;
  status: string;
  word_count: number;
  seg_count: number;
  added_at: string;
}

export interface ProjectSettings {
  fuzzy_threshold: number;
  tm_ids: string[];
  tb_ids: string[];
  mt_engine: string | null;
  mt_penalty: number;
  auto_propagate: boolean;
  writeback_mode: 'always' | 'ask' | 'never';
}

// Currently open project
export const currentProject = writable<Project | null>(null);
export const currentFiles = writable<ProjectFile[]>([]);
export const currentFileId = writable<string | null>(null);
export const projectSettings = writable<ProjectSettings>({
  fuzzy_threshold: 0.75,
  tm_ids: [],
  tb_ids: [],
  mt_engine: null,
  mt_penalty: 0.03,
  auto_propagate: true,
  writeback_mode: 'always',
});

// Derived: current file info
export const currentFile = derived(
  [currentFiles, currentFileId],
  ([$files, $fileId]) => $files.find(f => f.id === $fileId) ?? null
);

// Parse settings from JSON string
export function parseSettings(jsonStr: string): ProjectSettings {
  try {
    return JSON.parse(jsonStr);
  } catch {
    return {
      fuzzy_threshold: 0.75,
      tm_ids: [],
      tb_ids: [],
      mt_engine: null,
      mt_penalty: 0.03,
      auto_propagate: true,
      writeback_mode: 'always',
    };
  }
}
