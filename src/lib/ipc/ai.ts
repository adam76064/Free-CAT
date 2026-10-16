// src/lib/ipc/ai.ts — IPC wrapper for AI Translation Assistant
import { invoke } from '@tauri-apps/api/core';

export interface AiConfig {
  provider: 'ollama' | 'openai';
  endpoint?: string;
  model?: string;
  api_key?: string;
}

export interface AiRequest {
  prompt: string;
  source_text?: string;
  target_text?: string;
  src_lang?: string;
  tgt_lang?: string;
  action: 'ask' | 'rephrase' | 'critique';
  config: AiConfig;
}

export interface AiResponse {
  reply: string;
  model: string;
}

export async function queryAiAssistant(request: AiRequest): Promise<AiResponse> {
  return invoke('query_ai_assistant', { request });
}

export interface ContextSegment {
  seq_number: number;
  source_text: string;
  target_text?: string;
}

export interface DocumentAiRequest {
  current_source: string;
  current_target?: string;
  preceding_segments: ContextSegment[];
  following_segments: ContextSegment[];
  termbase_matches: [string, string][];
  style_guide?: string;
  action: 'translate' | 'rephrase' | 'condense' | 'tone_formal' | 'tone_casual' | 'fix_grammar';
  selected_text?: string;
  src_lang: string;
  tgt_lang: string;
  config: AiConfig;
}

export async function queryDocumentAiCopilot(request: DocumentAiRequest): Promise<AiResponse> {
  return invoke('query_document_ai_copilot', { request });
}
