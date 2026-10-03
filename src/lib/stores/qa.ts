// src/lib/stores/qa.ts
import { writable, derived } from 'svelte/store';

export interface QaResult {
  id: number | null;
  project_id: string;
  segment_id: string | null;
  check_name: string;
  severity: 'warning' | 'error';
  message: string;
  run_at: string;
}

export const qaResults = writable<QaResult[]>([]);
export const qaRunning = writable<boolean>(false);

export const qaErrors   = derived(qaResults, $r => $r.filter(r => r.severity === 'error'));
export const qaWarnings = derived(qaResults, $r => $r.filter(r => r.severity === 'warning'));

export const QA_CHECK_LABELS: Record<string, string> = {
  empty_target:           'Empty Target',
  tag_mismatch:           'Tag Mismatch',
  leading_trailing_space: 'Whitespace',
  double_space:           'Double Space',
  number_inconsistency:   'Number Mismatch',
  repeated_word:          'Repeated Word',
  untranslated:           'Untranslated',
  segment_too_long:       'Segment Too Long',
  forbidden_term:         'Forbidden Term',
};
