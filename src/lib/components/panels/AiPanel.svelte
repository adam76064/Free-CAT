<script lang="ts">
  /**
   * AiPanel — Document-Aware AI Translation Copilot panel.
   * Supports local Ollama (100% offline) and OpenAI-compatible endpoints.
   * Injects 5-segment discourse sliding window, terminology glossary, and style guide.
   * Actions: Translate in Context, Rephrase, Condense, Formal/Casual Tone, Grammar Fix, Ask.
   */
  import { segments, selectedIndex, selectedSegment } from '$lib/stores/segments';
  import { currentProject } from '$lib/stores/project';
  import { termMatches } from '$lib/stores/termbase';
  import { settings } from '$lib/stores/settings';
  import {
    queryAiAssistant,
    queryDocumentAiCopilot,
    type AiConfig,
    type ContextSegment
  } from '$lib/ipc/ai';

  interface Props {
    onApplyTarget?: (text: string) => void;
  }
  let { onApplyTarget }: Props = $props();

  let provider = $state<'ollama' | 'openai'>('ollama');
  let model = $state('llama3.2');
  let customPrompt = $state('');
  let loading = $state(false);
  let error = $state<string | null>(null);
  let reply = $state<string | null>(null);
  let showConfig = $state(false);

  function getConfig(): AiConfig {
    if (provider === 'openai') {
      return {
        provider: 'openai',
        endpoint: 'https://api.openai.com/v1',
        model: model || 'gpt-4o-mini',
        api_key: $settings.deepl_api_key,
      };
    } else {
      return {
        provider: 'ollama',
        endpoint: $settings.ollama_url || 'http://localhost:11434',
        model: model || 'llama3.2',
      };
    }
  }

  async function handleDocumentAction(action: 'translate' | 'rephrase' | 'condense' | 'tone_formal' | 'tone_casual' | 'fix_grammar' | 'ask') {
    if (!$selectedSegment || !$currentProject) return;
    loading = true;
    error = null;
    reply = null;

    try {
      const curIdx = $selectedIndex;
      const preceding: ContextSegment[] = [];
      if (curIdx > 0 && $segments[curIdx - 1]) {
        if (curIdx > 1 && $segments[curIdx - 2]) {
          preceding.push({
            seq_number: $segments[curIdx - 2].seq_number,
            source_text: $segments[curIdx - 2].source_text,
            target_text: $segments[curIdx - 2].target_text || undefined,
          });
        }
        preceding.push({
          seq_number: $segments[curIdx - 1].seq_number,
          source_text: $segments[curIdx - 1].source_text,
          target_text: $segments[curIdx - 1].target_text || undefined,
        });
      }

      const following: ContextSegment[] = [];
      if (curIdx + 1 < $segments.length) {
        following.push({
          seq_number: $segments[curIdx + 1].seq_number,
          source_text: $segments[curIdx + 1].source_text,
        });
        if (curIdx + 2 < $segments.length) {
          following.push({
            seq_number: $segments[curIdx + 2].seq_number,
            source_text: $segments[curIdx + 2].source_text,
          });
        }
      }

      const tbTerms: [string, string][] = $termMatches
        .filter(m => m.term && m.definition)
        .map(m => [m.term, m.definition!]);

      if (action === 'ask') {
        const res = await queryAiAssistant({
          prompt: customPrompt,
          source_text: $selectedSegment.source_text,
          target_text: $selectedSegment.target_text,
          src_lang: $currentProject.src_lang,
          tgt_lang: $currentProject.tgt_lang,
          action: 'ask',
          config: getConfig(),
        });
        reply = res.reply;
        customPrompt = '';
      } else {
        const res = await queryDocumentAiCopilot({
          current_source: $selectedSegment.source_text,
          current_target: $selectedSegment.target_text || undefined,
          preceding_segments: preceding,
          following_segments: following,
          termbase_matches: tbTerms,
          action,
          src_lang: $currentProject.src_lang,
          tgt_lang: $currentProject.tgt_lang,
          config: getConfig(),
        });
        reply = res.reply;
      }
    } catch (e: any) {
      error = String(e);
    } finally {
      loading = false;
    }
  }
</script>

<div class="flex flex-col h-full bg-white dark:bg-gray-950">
  <!-- Top bar / Settings toggle -->
  <div class="flex items-center justify-between p-2.5 border-b border-gray-200 dark:border-gray-800 bg-gray-50 dark:bg-gray-900/50 text-xs shrink-0">
    <div class="flex items-center gap-1.5 font-medium text-gray-700 dark:text-gray-300">
      <span class="text-base">✨</span>
      <span>AI Document Copilot</span>
      <span class="text-[10px] px-1.5 py-0.2 rounded bg-purple-100 text-purple-700 dark:bg-purple-900/40 dark:text-purple-300 font-mono">
        {provider === 'ollama' ? 'Offline (Ollama)' : 'Cloud (OpenAI)'}
      </span>
    </div>
    <button
      type="button"
      onclick={() => showConfig = !showConfig}
      class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 p-1 cursor-pointer"
      title="Configure AI model"
    >
      ⚙️
    </button>
  </div>

  <!-- Settings Collapsible -->
  {#if showConfig}
    <div class="p-3 bg-gray-50 dark:bg-gray-900 border-b border-gray-200 dark:border-gray-800 text-xs space-y-2 shrink-0">
      <div class="flex items-center justify-between">
        <label for="ai-provider-select" class="text-gray-600 dark:text-gray-400">Provider:</label>
        <select
          id="ai-provider-select"
          bind:value={provider}
          class="px-2 py-1 rounded border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-800 dark:text-gray-200"
        >
          <option value="ollama">Ollama (Local Offline)</option>
          <option value="openai">OpenAI Compatible</option>
        </select>
      </div>

      <div class="flex items-center justify-between">
        <label for="ai-model-input" class="text-gray-600 dark:text-gray-400">Model Name:</label>
        <input
          id="ai-model-input"
          bind:value={model}
          placeholder="llama3.2, qwen2.5:7b, gpt-4o-mini"
          class="px-2 py-1 rounded border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-800 dark:text-gray-200 w-40 font-mono text-[11px]"
        />
      </div>

      <div class="text-[10px] text-gray-400 italic">
        Tip: Runs locally via Ollama with full 5-segment sliding discourse cohesion.
      </div>
    </div>
  {/if}

  <!-- Main Content -->
  <div class="flex-1 p-3 overflow-y-auto space-y-3">
    {#if !$selectedSegment}
      <div class="flex flex-col items-center justify-center h-32 text-gray-400 text-xs text-center gap-1">
        <span class="text-2xl">🤖</span>
        <span>Select a segment in the editor to use AI Copilot.</span>
      </div>
    {:else}
      <!-- Quick Action Grid -->
      <div class="grid grid-cols-2 gap-2">
        <button
          type="button"
          onclick={() => handleDocumentAction('translate')}
          disabled={loading}
          class="p-2 rounded-lg border border-purple-200 dark:border-purple-900/60
                 bg-purple-50/60 dark:bg-purple-950/20 text-purple-700 dark:text-purple-300
                 hover:bg-purple-100 dark:hover:bg-purple-900/40 text-xs font-medium transition-colors
                 disabled:opacity-40 text-left flex flex-col gap-0.5 shadow-xs cursor-pointer"
        >
          <span class="font-bold flex items-center gap-1">🌐 Translate In Context</span>
          <span class="text-[10px] text-purple-600/70 dark:text-purple-400/70">Cohesive translation with glossary</span>
        </button>

        <button
          type="button"
          onclick={() => handleDocumentAction('rephrase')}
          disabled={loading || !$selectedSegment.target_text.trim()}
          class="p-2 rounded-lg border border-blue-200 dark:border-blue-900/60
                 bg-blue-50/60 dark:bg-blue-950/20 text-blue-700 dark:text-blue-300
                 hover:bg-blue-100 dark:hover:bg-blue-900/40 text-xs font-medium transition-colors
                 disabled:opacity-40 text-left flex flex-col gap-0.5 shadow-xs cursor-pointer"
        >
          <span class="font-bold flex items-center gap-1">✨ Rephrase & Polish</span>
          <span class="text-[10px] text-blue-600/70 dark:text-blue-400/70">3 alternative phrasings</span>
        </button>

        <button
          type="button"
          onclick={() => handleDocumentAction('condense')}
          disabled={loading || !$selectedSegment.target_text.trim()}
          class="p-2 rounded-lg border border-emerald-200 dark:border-emerald-900/60
                 bg-emerald-50/60 dark:bg-emerald-950/20 text-emerald-700 dark:text-emerald-300
                 hover:bg-emerald-100 dark:hover:bg-emerald-900/40 text-xs font-medium transition-colors
                 disabled:opacity-40 text-left flex flex-col gap-0.5 shadow-xs cursor-pointer"
        >
          <span class="font-bold flex items-center gap-1">🤏 Condense / Shorten</span>
          <span class="text-[10px] text-emerald-600/70 dark:text-emerald-400/70">Succinct phrasing</span>
        </button>

        <button
          type="button"
          onclick={() => handleDocumentAction('fix_grammar')}
          disabled={loading || !$selectedSegment.target_text.trim()}
          class="p-2 rounded-lg border border-amber-200 dark:border-amber-900/60
                 bg-amber-50/60 dark:bg-amber-950/20 text-amber-700 dark:text-amber-300
                 hover:bg-amber-100 dark:hover:bg-amber-900/40 text-xs font-medium transition-colors
                 disabled:opacity-40 text-left flex flex-col gap-0.5 shadow-xs cursor-pointer"
        >
          <span class="font-bold flex items-center gap-1">🩺 Fix Grammar</span>
          <span class="text-[10px] text-amber-600/70 dark:text-amber-400/70">Syntax, agreement & typos</span>
        </button>

        <button
          type="button"
          onclick={() => handleDocumentAction('tone_formal')}
          disabled={loading || !$selectedSegment.target_text.trim()}
          class="p-2 rounded-lg border border-indigo-200 dark:border-indigo-900/60
                 bg-indigo-50/60 dark:bg-indigo-950/20 text-indigo-700 dark:text-indigo-300
                 hover:bg-indigo-100 dark:hover:bg-indigo-900/40 text-xs font-medium transition-colors
                 disabled:opacity-40 text-left flex flex-col gap-0.5 shadow-xs cursor-pointer"
        >
          <span class="font-bold">👔 Formal Tone</span>
          <span class="text-[10px] text-indigo-600/70 dark:text-indigo-400/70">Elevated professional register</span>
        </button>

        <button
          type="button"
          onclick={() => handleDocumentAction('tone_casual')}
          disabled={loading || !$selectedSegment.target_text.trim()}
          class="p-2 rounded-lg border border-teal-200 dark:border-teal-900/60
                 bg-teal-50/60 dark:bg-teal-950/20 text-teal-700 dark:text-teal-300
                 hover:bg-teal-100 dark:hover:bg-teal-900/40 text-xs font-medium transition-colors
                 disabled:opacity-40 text-left flex flex-col gap-0.5 shadow-xs cursor-pointer"
        >
          <span class="font-bold">☕ Casual Tone</span>
          <span class="text-[10px] text-teal-600/70 dark:text-teal-400/70">Natural, friendly flow</span>
        </button>
      </div>

      <!-- Loading State -->
      {#if loading}
        <div class="p-4 rounded-xl bg-purple-50 dark:bg-purple-950/30 border border-purple-200 dark:border-purple-900 text-center text-xs text-purple-600 dark:text-purple-400 flex items-center justify-center gap-2">
          <span class="animate-spin text-base">⏳</span>
          <span>Generating with {model}…</span>
        </div>
      {/if}

      <!-- Error State -->
      {#if error}
        <div class="p-3 rounded-lg bg-red-50 dark:bg-red-950/30 border border-red-200 dark:border-red-900 text-red-600 dark:text-red-400 text-xs leading-relaxed">
          <strong>AI Error:</strong> {error}
        </div>
      {/if}

      <!-- Reply Box -->
      {#if reply}
        <div class="p-3.5 rounded-xl bg-gray-50 dark:bg-gray-900 border border-gray-200 dark:border-gray-800 text-xs space-y-2">
          <div class="flex items-center justify-between pb-1.5 border-b border-gray-200 dark:border-gray-800">
            <span class="font-semibold text-gray-700 dark:text-gray-300 flex items-center gap-1">
              <span>💡 AI Output</span>
            </span>
            <div class="flex items-center gap-2">
              {#if onApplyTarget}
                <button
                  type="button"
                  onclick={() => onApplyTarget?.(reply!.replace(/^["']|["']$/g, '').trim())}
                  class="px-2 py-0.5 rounded bg-brand-500 hover:bg-brand-600 text-white font-medium text-[11px] transition-colors cursor-pointer"
                  title="Apply translation to active segment target"
                >
                  ✓ Apply to Target
                </button>
              {/if}
              <button
                type="button"
                onclick={() => navigator.clipboard.writeText(reply!)}
                class="text-[11px] text-gray-500 hover:text-gray-800 dark:hover:text-gray-200 cursor-pointer"
              >
                Copy
              </button>
            </div>
          </div>
          <div class="text-gray-800 dark:text-gray-200 whitespace-pre-wrap leading-relaxed font-editor" dir="auto">
            {reply}
          </div>
        </div>
      {/if}
    {/if}
  </div>

  <!-- Custom Prompt Input -->
  <div class="p-2.5 border-t border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-950 shrink-0">
    <div class="flex gap-1.5">
      <input
        bind:value={customPrompt}
        onkeydown={(e) => e.key === 'Enter' && customPrompt.trim() && handleDocumentAction('ask')}
        placeholder="Ask AI or type custom translation instruction…"
        disabled={loading || !$selectedSegment}
        class="flex-1 px-2.5 py-1.5 text-xs rounded-lg border border-gray-300 dark:border-gray-700
               bg-white dark:bg-gray-800 text-gray-900 dark:text-white
               focus:outline-none focus:ring-1 focus:ring-purple-500 disabled:opacity-50"
      />
      <button
        type="button"
        onclick={() => handleDocumentAction('ask')}
        disabled={loading || !customPrompt.trim() || !$selectedSegment}
        class="px-3 py-1.5 text-xs bg-purple-600 hover:bg-purple-700 text-white rounded-lg
               disabled:opacity-40 transition-colors font-medium shadow-xs cursor-pointer"
      >
        Send
      </button>
    </div>
  </div>
</div>
