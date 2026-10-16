// src/lib/ipc/tqa.ts
// IPC wrappers for TAUS DQF-MQM Translation Quality Assessment
import { invoke } from '@tauri-apps/api/core';

export interface TqaError {
  id?: number;
  project_id: string;
  segment_id: string;
  category: string; // 'accuracy' | 'fluency' | 'terminology' | 'style' | 'design'
  sub_type?: string | null;
  severity: string; // 'minor' | 'major' | 'critical'
  penalty: number;
  comment?: string | null;
  created_at: string;
}

export interface TqaScorecard {
  project_id: string;
  total_words: number;
  total_errors: number;
  minor_count: number;
  major_count: number;
  critical_count: number;
  total_penalty: number;
  score_pct: number;
  passed: boolean;
  errors: TqaError[];
}

export async function addTqaError(args: {
  project_id: string;
  segment_id: string;
  category: string;
  sub_type?: string;
  severity: string;
  comment?: string;
}): Promise<TqaError> {
  return invoke('add_tqa_error', { args });
}

export async function deleteTqaError(id: number): Promise<void> {
  return invoke('delete_tqa_error', { id });
}

export async function getSegmentTqaErrors(segment_id: string): Promise<TqaError[]> {
  return invoke('get_segment_tqa_errors', { segmentId: segment_id });
}

export async function getTqaScorecard(
  project_id: string,
  total_words?: number
): Promise<TqaScorecard> {
  return invoke('get_tqa_scorecard', { projectId: project_id, totalWords: total_words ?? null });
}
