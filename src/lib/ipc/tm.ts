// src/lib/ipc/tm.ts
// Typed Tauri IPC wrappers for TM commands.

import { invoke } from '@tauri-apps/api/core';
import type { TmMatch, TmInfo } from '$lib/stores/tm';

export async function queryTm(args: {
  source_text: string;
  src_lang: string;
  tgt_lang: string;
  tm_ids: string[];
  threshold: number;
  prev_seg?: string;
  next_seg?: string;
  penalties?: Record<string, number>;
}): Promise<TmMatch[]> {
  return invoke('query_tm', {
    args: {
      ...args,
      penalties: args.penalties ?? {},
    }
  });
}

export async function addToTm(args: {
  tm_id: string;
  source_text: string;
  target_text: string;
  src_lang: string;
  tgt_lang: string;
  source_file?: string;
}): Promise<void> {
  return invoke('add_to_tm', { args });
}

export async function listTms(): Promise<TmInfo[]> {
  return invoke('list_tms');
}

export async function createTm(args: {
  name: string;
  src_lang: string;
  tgt_lang: string;
}): Promise<TmInfo> {
  return invoke('create_tm', { args });
}

export async function deleteTm(tm_id: string): Promise<void> {
  return invoke('delete_tm', { tmId: tm_id });
}

export async function importTmx(tm_id: string, tmx_path: string): Promise<number> {
  return invoke('import_tmx', { tmId: tm_id, tmxPath: tmx_path });
}

export async function exportTmx(tm_id: string, out_path: string): Promise<number> {
  return invoke('export_tmx', { tmId: tm_id, outPath: out_path });
}

export async function concordanceSearch(args: {
  query: string;
  tm_ids: string[];
  limit: number;
}): Promise<any[]> {
  return invoke('concordance_search', {
    query: args.query,
    tmIds: args.tm_ids,
    limit: args.limit,
  });
}
