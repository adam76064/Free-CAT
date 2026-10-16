// src/lib/ipc/termbase.ts
import { invoke } from '@tauri-apps/api/core';

export interface TermEntry {
  id?: number;
  lang: string;
  term: string;
  definition?: string;
  domain?: string;
  note?: string;
  status: string;
  created_at: string;
  metadata?: string;
}

export interface TbInfo {
  id: string;
  name: string;
  count: number;
}

export async function queryTb(args: {
  text: string;
  lang: string;
  tb_ids: string[];
}): Promise<TermEntry[]> {
  return invoke('query_tb', {
    text: args.text,
    lang: args.lang,
    tbIds: args.tb_ids,
  });
}

export async function addTerm(args: {
  tb_id: string;
  lang: string;
  term: string;
  definition?: string;
  domain?: string;
  note?: string;
  status?: string;
}): Promise<number> {
  return invoke('add_term', { args });
}

export async function listTbs(): Promise<TbInfo[]> {
  return invoke('list_tbs');
}

export async function createTb(name: string): Promise<string> {
  return invoke('create_tb', { name });
}

export async function deleteTb(id: string): Promise<void> {
  return invoke('delete_tb', { id });
}

export async function importTbx(tb_id: string, tbx_path: string): Promise<number> {
  return invoke('import_tbx', { tbId: tb_id, tbxPath: tbx_path });
}

export async function exportTbx(tb_id: string, out_path: string): Promise<number> {
  return invoke('export_tbx', { tbId: tb_id, outPath: out_path });
}

export interface ExtractedTerm {
  term: string;
  frequency: number;
  c_value: number;
  sample_context: string;
}

export async function extractTerms(args: {
  text: string;
  min_freq?: number;
  max_ngram?: number;
}): Promise<ExtractedTerm[]> {
  return invoke('extract_terms', {
    text: args.text,
    minFreq: args.min_freq,
    maxNgram: args.max_ngram,
  });
}
