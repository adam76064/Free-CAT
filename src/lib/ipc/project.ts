// src/lib/ipc/project.ts
import { invoke } from '@tauri-apps/api/core';
import type { Project, ProjectFile } from '$lib/stores/project';

export async function listProjects(): Promise<Project[]> {
  return invoke('list_projects');
}

export async function createProject(args: {
  name: string;
  src_lang: string;
  tgt_lang: string;
}): Promise<Project> {
  return invoke('create_project', { args });
}

export async function openProject(id: string): Promise<[Project, ProjectFile[]]> {
  return invoke('open_project', { id });
}

export async function deleteProject(id: string): Promise<void> {
  return invoke('delete_project', { id });
}

export async function importFile(args: {
  project_id: string;
  file_path: string;
}): Promise<{ file: ProjectFile; segments: any[] }> {
  return invoke('import_file', { args });
}

export async function exportFile(args: {
  file_id: string;
  project_id: string;
  out_path?: string;
}): Promise<string> {
  return invoke('export_file', {
    fileId: args.file_id,
    projectId: args.project_id,
    outPath: args.out_path ?? null,
  });
}

export async function analyzeProject(args: {
  project_id: string;
  file_ids: string[];
}): Promise<any> {
  return invoke('analyze_project', {
    projectId: args.project_id,
    fileIds: args.file_ids,
  });
}

export async function runQa(args: {
  project_id: string;
  file_ids: string[];
  tb_ids: string[];
}): Promise<any[]> {
  return invoke('run_qa', {
    projectId: args.project_id,
    fileIds: args.file_ids,
    tbIds: args.tb_ids,
  });
}

export async function getMtSuggestion(args: {
  source_text: string;
  src_lang: string;
  tgt_lang: string;
  engine: string;
  api_key?: string;
  api_url?: string;
}): Promise<{ translated_text: string; engine: string; score_pct: number | null }> {
  return invoke('get_mt_suggestion', { args });
}

export async function preTranslate(args: {
  project_id: string;
  file_ids: string[];
  min_score?: number;
  overwrite?: boolean;
}): Promise<{ total_segments: number; pretranslated: number; words_filled: number }> {
  return invoke('pre_translate', { args });
}

export async function importSdlppx(args: {
  project_id: string;
  package_path: string;
}): Promise<{ file: ProjectFile; segments: any[] }[]> {
  return invoke('import_sdlppx', {
    projectId: args.project_id,
    packagePath: args.package_path,
  });
}

export async function exportSdlrpx(args: {
  project_id: string;
  out_path?: string;
}): Promise<string> {
  return invoke('export_sdlrpx', {
    projectId: args.project_id,
    outPath: args.out_path ?? null,
  });
}

export interface AnalysisBand {
  id: string;
  name: string;
  match_range: string;
  segments: number;
  words: number;
  default_rate_pct: number;
  weighted_words: number;
}

export interface DetailedAnalysisReport {
  project_id: string;
  total_words: number;
  total_segments: number;
  bands: AnalysisBand[];
  total_weighted_words: number;
}

export async function getDetailedAnalysis(
  projectId: string,
  fileIds: string[]
): Promise<DetailedAnalysisReport> {
  return invoke('get_detailed_analysis', {
    projectId,
    fileIds,
  });
}

export async function exportFcat(
  projectId: string,
  outPath: string
): Promise<string> {
  return invoke('export_fcat', {
    projectId,
    outPath,
  });
}

export async function importFcat(
  fcatPath: string
): Promise<Project> {
  return invoke('import_fcat', {
    fcatPath,
  });
}


