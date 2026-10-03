// src/lib/stores/tm.ts
// Store for TM matches for the current segment.

import { writable } from 'svelte/store';

export interface TmMatch {
  tu_id: number;
  source_text: string;
  target_text: string;
  score_pct: number;
  levenshtein: number;
  dice: number;
  tm_id: string;
  tm_name: string;
  is_exact: boolean;
  is_context: boolean;
}

export interface TmInfo {
  id: string;
  name: string;
  src_lang: string;
  tgt_lang: string;
  tu_count: number;
}

// Current TM matches for the selected segment
export const tmMatches = writable<TmMatch[]>([]);
export const tmLoading = writable<boolean>(false);

// Available TMs in the system
export const availableTms = writable<TmInfo[]>([]);

// Concordance search results
export const concordanceResults = writable<any[]>([]);
export const concordanceQuery = writable<string>('');

/** Get color class for a TM score percentage */
export function matchColor(pct: number): string {
  if (pct >= 100) return 'badge-exact';
  if (pct >= 85)  return 'badge-high';
  if (pct >= 75)  return 'badge-medium';
  return 'badge-low';
}

/** Get label for a TM score */
export function matchLabel(pct: number): string {
  if (pct >= 102) return '102%';
  if (pct >= 101) return '101%';
  if (pct >= 100) return '100%';
  return `${pct}%`;
}
