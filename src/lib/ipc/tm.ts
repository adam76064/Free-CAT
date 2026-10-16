// src/lib/ipc/tm.ts
// Typed Tauri IPC wrappers for TM commands.

import { invoke } from '@tauri-apps/api/core';
import type { TmMatch, TmInfo } from '$lib/stores/tm';
export type { TmMatch, TmInfo };

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

export interface FragmentMatch {
  source_fragment: string;
  target_fragment: string;
  source_sentence: string;
  target_sentence: string;
  tu_id: number;
  tm_id: string;
  tm_name: string;
}

export async function queryFragments(args: {
  source_text: string;
  src_lang: string;
  tgt_lang: string;
  tm_ids: string[];
}): Promise<FragmentMatch[]> {
  return invoke('query_fragments', { args });
}

export interface TranslationUnit {
  id?: number;
  source_lang: string;
  target_lang: string;
  source_text: string;
  target_text: string;
  created_at?: string;
  modified_at?: string;
  created_by?: string | null;
  client?: string | null;
  domain?: string | null;
  source_file?: string | null;
  usage_count?: number;
  metadata?: string | null;
}

export interface PaginatedTusResult {
  tus: TranslationUnit[];
  total_count: number;
  offset: number;
  limit: number;
}

export async function getTmTusPaginated(args: {
  tm_id: string;
  offset: number;
  limit: number;
  search?: string;
}): Promise<PaginatedTusResult> {
  return invoke('get_tm_tus_paginated', {
    tmId: args.tm_id,
    offset: args.offset,
    limit: args.limit,
    search: args.search ?? null,
  });
}

export async function updateTmTu(args: {
  tm_id: string;
  tu_id: number;
  source_text: string;
  target_text: string;
}): Promise<void> {
  return invoke('update_tm_tu', {
    tmId: args.tm_id,
    tuId: args.tu_id,
    sourceText: args.source_text,
    targetText: args.target_text,
  });
}

export async function deleteTmTu(args: {
  tm_id: string;
  tu_id: number;
}): Promise<void> {
  return invoke('delete_tm_tu', {
    tmId: args.tm_id,
    tuId: args.tu_id,
  });
}

export async function batchReplaceTm(args: {
  tm_id: string;
  find_text: string;
  replace_text: string;
  target_only: boolean;
}): Promise<number> {
  return invoke('batch_replace_tm', {
    tmId: args.tm_id,
    findText: args.find_text,
    replaceText: args.replace_text,
    targetOnly: args.target_only,
  });
}

