// src/lib/ipc/aligner.ts
import { invoke } from '@tauri-apps/api/core';

export interface AlignedPair {
  id: string;
  source_text: string;
  target_text: string;
  score: number;
  alignment_type: string;
  confirmed: boolean;
}

export async function alignTexts(args: {
  source_text: string;
  target_text: string;
  src_lang: string;
  tgt_lang: string;
}): Promise<AlignedPair[]> {
  return invoke('align_texts', { args });
}

export async function alignFiles(args: {
  source_path: string;
  target_path: string;
  src_lang: string;
  tgt_lang: string;
}): Promise<AlignedPair[]> {
  return invoke('align_files', { args });
}

export async function exportAlignmentTmx(args: {
  pairs: AlignedPair[];
  src_lang: string;
  tgt_lang: string;
  output_path: string;
}): Promise<string> {
  return invoke('export_alignment_tmx', { args });
}

export async function importAlignmentToTm(args: {
  tm_id: string;
  pairs: AlignedPair[];
  src_lang: string;
  tgt_lang: string;
}): Promise<number> {
  return invoke('import_alignment_to_tm', { args });
}
