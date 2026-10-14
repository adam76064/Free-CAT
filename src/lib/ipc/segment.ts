// src/lib/ipc/segment.ts
import { invoke } from '@tauri-apps/api/core';
import type { Segment } from '$lib/stores/segments';

export async function getSegments(file_id: string, project_id: string): Promise<Segment[]> {
  return invoke('get_segments', { fileId: file_id, projectId: project_id });
}

export async function updateSegment(args: {
  segment_id: string;
  target_text: string;
  status: string;
}): Promise<void> {
  return invoke('update_segment', { args });
}

export async function confirmSegment(args: {
  segment_id: string;
  target_text: string;
  project_id: string;
  file_name?: string;
  source_text: string;
  src_lang: string;
  tgt_lang: string;
}): Promise<number> {
  return invoke('confirm_segment', { args });
}
