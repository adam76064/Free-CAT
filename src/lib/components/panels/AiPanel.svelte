<script lang="ts">
  /**
   * AiPanel — Document-Aware AI Translation Copilot panel.
   * Supports local Ollama (100% offline), LM Studio, OpenAI, DeepSeek, Gemini, Claude, and Custom OpenAI endpoints.
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

  interface AiPreset {
    id: string;
    name: string;
    provider: 'ollama' | 'openai';
    endpoint: string;
    model: string;
    badge: string;
    badgeClass: string;
    requiresKey: boolean;
    hint: string;
  }

  const PRESETS: AiPreset[] = [
    {
      id: 'ollama',
      name: 'Ollama (Local Offline)',
      provider: 'ollama',
      endpoint: 'http://localhost:11434',
      model: 'llama3.2',
      badge: 'Offline',
      badgeClass: 'bg-emerald-100 text-emerald-700 dark:bg-emerald-950/60 dark:text-emerald-300',
      requiresKey: false,
      hint: '100% private and offline. Requires Ollama running locally.',
    },
    {
      id: 'lmstudio',
      name: 'LM Studio (Local)',
      provider: 'openai',
      endpoint: 'http://localhost:1234/v1',
      model: 'local-model',
      badge: 'Offline',
      badgeClass: 'bg-emerald-100 text-emerald-700 dark:bg-emerald-950/60 dark:text-emerald-300',
      requiresKey: false,
      hint: "Runs locally via LM Studio's OpenAI-compatible server.",
    },
    {
      id: 'openai',
      name: 'OpenAI',
      provider: 'openai',
      endpoint: 'https://api.openai.com/v1',
      model: 'gpt-4o-mini',
      badge: 'Cloud',
      badgeClass: 'bg-blue-100 text-blue-700 dark:bg-blue-950/60 dark:text-blue-300',
      requiresKey: true,
      hint: 'Requires OpenAI API key configured in Settings or OS Keychain.',
    },
    {
      id: 'deepseek',
      name: 'DeepSeek',
      provider: 'openai',
      endpoint: 'https://api.deepseek.com/v1',
      model: 'deepseek-chat',
      badge: 'Cloud',
      badgeClass: 'bg-cyan-100 text-cyan-700 dark:bg-cyan-950/60 dark:text-cyan-300',
      requiresKey: true,
      hint: 'High translation capability, OpenAI-compatible API.',
    },
    {
      id: 'gemini',
      name: 'Google Gemini',
      provider: 'openai',
      endpoint: 'https://generativelanguage.googleapis.com/v1beta/openai/',
      model: 'gemini-2.0-flash',
      badge: 'Cloud',
      badgeClass: 'bg-purple-100 text-purple-700 dark:bg-purple-950/60 dark:text-purple-300',
      requiresKey: true,
      hint: 'Gemini via OpenAI-compatible endpoint. API key required.',
    },
    {
      id: 'claude',
      name: 'Anthropic Claude',
      provider: 'openai',
      endpoint: 'https://api.anthropic.com/v1',
      model: 'claude-sonnet-4-5',
      badge: 'Cloud',
      badgeClass: 'bg-orange-100 text-orange-700 dark:bg-orange-950/60 dark:text-orange-300',
      requiresKey: true,
      hint: 'Requires OpenAI-compatible Claude proxy or endpoint.',
    },
    {
      id: 'custom',
      name: 'Custom Endpoint',
      provider: 'openai',
      endpoint: '',
      model: '',
      badge: 'Custom',
      badgeClass: 'bg-gray-100 text-gray-700 dark:bg-gray-800 dark:text-gray-300',
      requiresKey: false,
      hint: 'Connect to any OpenAI-compatible server (vLLM, LocalAI, text-generation-webui).',
    },
  ];

  let selectedPresetId = $state<string>('ollama');
  let customEndpoint = $state('');
  let model = $state('llama3.2');
  let customPrompt = $state('');
  let loading = $state(false);
  let error = $state<string | null>(null);
  let reply = $state<string | null>(null);
  let showConfig = $state(false);

  // History stack for Back button
  let historyStack = $state<string[]>([]);
  let historyIndex = $state<number>(-1);

  let selectedPreset = $derived(
    PRESETS.find((p) => p.id === selectedPresetId) ?? PRESETS[0]
  );
  let provider = $derived<'ollama' | 'openai'>(selectedPreset.provider);

  function selectPreset(id: string) {
    selectedPresetId = id;
    const p = PRESETS.find((x) => x.id === id);
    if (p) {
      if (p.id !== 'custom') {
        model = p.model;
        customEndpoint = p.endpoint;
      }
    }
  }

  function getConfig(): AiConfig {
    if (selectedPreset.provider === 'ollama') {
      return {
        provider: 'ollama',
        endpoint: $settings.ollama_url || selectedPreset.endpoint || 'http://localhost:11434',
        model: model || 'llama3.2',
      };
    } else {
      const endpoint = selectedPreset.id === 'custom'
        ? (customEndpoint || 'http://localhost:1234/v1')
        : selectedPreset.endpoint;
      const cfg: AiConfig = {
        provider: 'openai',
        endpoint,
        model: model || selectedPreset.model || 'gpt-4o-mini',
      };
      if (!selectedPreset.requiresKey) {
        cfg.api_key = 'not-needed';
      }
      return cfg;
    }
  }

  function pushReply(newReply: string) {
    reply = newReply;
    if (historyIndex < historyStack.length - 1) {
      historyStack = historyStack.slice(0, historyIndex + 1);
    }
    historyStack.push(newReply);
    if (historyStack.length > 10) historyStack.shift();
    historyIndex = historyStack.length - 1;
  }

  function goBack() {
    if (historyIndex > 0) {
      historyIndex -= 1;
      reply = historyStack[historyIndex];
    }
  }

  function goForward() {
    if (historyIndex < historyStack.length - 1) {
      historyIndex += 1;
      reply = historyStack[historyIndex];
    }
  }

  // Parse numbered alternatives if action was 'rephrase'
  let parsedAlternatives = $derived.by(() => {
    if (!reply) return [];
    const lines = reply.split('\n');
    const items: string[] = [];
    for (const rawLine of lines) {
      const line = rawLine.trim();
      const match = line.match(/^(?:[1-9]\.|\([1-9]\)|[1-9]\))\s+(.+)$/);
      if (match && match[1]) {
        items.push(match[1].replace(/^["']|["']$/g, '').trim());
      }
    }
    return items;
  });

  // Basic Quality Estimation (QE) confidence score
  let qeScore = $derived.by(() => {
    if (!reply || !$selectedSegment) return null;
    const src = $selectedSegment.source_text.trim();
    const tgt = reply.trim();
    if (!src || !tgt) return null;

    let score = 30; // base score

    // Check non-identity
    if (src.toLowerCase() !== tgt.toLowerCase()) {
      score += 20;
    }

    // Length sanity ratio (0.4 - 2.5)
    const ratio = tgt.length / (src.length || 1);
    if (ratio >= 0.4 && ratio <= 2.5) {
      score += 20;
    }

    // Number preservation check
    const srcNums: string[] = src.match(/\b\d+(?:[.,]\d+)?\b/g) || [];
    const tgtNums: string[] = tgt.match(/\b\d+(?:[.,]\d+)?\b/g) || [];
    if (srcNums.length === 0 || srcNums.every((n) => tgtNums.includes(n))) {
      score += 20;
    }

    // Tag / placeholder preservation check (e.g. {1}, <g1>)
    const srcTags: string[] = src.match(/<[^>]+>|\{\d+\}/g) || [];
    const tgtTags: string[] = tgt.match(/<[^>]+>|\{\d+\}/g) || [];
    if (srcTags.length === 0 || srcTags.every((t) => tgtTags.includes(t))) {
      score += 10;
    }

    return Math.min(100, Math.max(10, score));
  });

  async function handleDocumentAction(action: 'translate' | 'rephrase' | 'condense' | 'tone_formal' | 'tone_casual' | 'fix_grammar' | 'ask') {
    if (!$selectedSegment || !$currentProject) return;
    loading = true;
    error = null;

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
        pushReply(res.reply);
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
        pushReply(res.reply);
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
      <span>AI Copilot</span>
      <span class="text-[10px] px-1.5 py-0.5 rounded font-mono font-semibold {selectedPreset.badgeClass}">
        {selectedPreset.name}
      </span>
    </div>
    <div class="flex items-center gap-1">
      {#if historyIndex > 0}
        <button
          type="button"
          onclick={goBack}
          class="p-1 text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 text-xs rounded hover:bg-gray-200 dark:hover:bg-gray-800 cursor-pointer"
          title="Previous AI response"
        >
          ◀
        </button>
      {/if}
      {#if historyIndex < historyStack.length - 1}
        <button
          type="button"
          onclick={goForward}
          class="p-1 text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 text-xs rounded hover:bg-gray-200 dark:hover:bg-gray-800 cursor-pointer"
          title="Next AI response"
        >
          ▶
        </button>
      {/if}
      <button aria-label="Configure AI model"
        type="button"
        onclick={() => showConfig = !showConfig}
        class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 p-1 cursor-pointer rounded hover:bg-gray-200 dark:hover:bg-gray-800"
        title="Configure AI model & preset"
      >
        ⚙️
      </button>
    </div>
  </div>

  <!-- Settings Collapsible with Preset Dropdown -->
  {#if showConfig}
    <div class="p-3 bg-gray-50 dark:bg-gray-900 border-b border-gray-200 dark:border-gray-800 text-xs space-y-2.5 shrink-0 shadow-inner">
      <div class="flex items-center justify-between">
        <label for="ai-preset-select" class="text-gray-600 dark:text-gray-400 font-medium">Provider Preset:</label>
        <select
          id="ai-preset-select"
          value={selectedPresetId}
          onchange={(e) => selectPreset(e.currentTarget.value)}
          class="px-2 py-1 rounded border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-800 dark:text-gray-200 text-xs font-medium cursor-pointer"
        >
          {#each PRESETS as p}
            <option value={p.id}>{p.name} [{p.badge}]</option>
          {/each}
        </select>
      </div>

      {#if selectedPreset.id === 'custom'}
        <div class="flex items-center justify-between">
          <label for="ai-endpoint-input" class="text-gray-600 dark:text-gray-400">Endpoint URL:</label>
          <input
            id="ai-endpoint-input"
            bind:value={customEndpoint}
            placeholder="http://localhost:1234/v1"
            class="px-2 py-1 rounded border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-800 dark:text-gray-200 w-48 font-mono text-[11px]"
          />
        </div>
      {/if}

      <div class="flex items-center justify-between">
        <label for="ai-model-input" class="text-gray-600 dark:text-gray-400">Model Name:</label>
        <input
          id="ai-model-input"
          bind:value={model}
          placeholder="llama3.2, gpt-4o-mini, etc."
          class="px-2 py-1 rounded border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-800 dark:text-gray-200 w-48 font-mono text-[11px]"
        />
      </div>

      <div class="text-[10px] text-gray-500 dark:text-gray-400 italic bg-gray-100 dark:bg-gray-800/60 p-1.5 rounded border border-gray-200 dark:border-gray-700/60">
        💡 {selectedPreset.hint}
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
        <div class="p-3.5 rounded-xl bg-gray-50 dark:bg-gray-900 border border-gray-200 dark:border-gray-800 text-xs space-y-2.5">
          <div class="flex items-center justify-between pb-1.5 border-b border-gray-200 dark:border-gray-800">
            <div class="flex items-center gap-2">
              <span class="font-semibold text-gray-700 dark:text-gray-300 flex items-center gap-1">
                <span>💡 AI Output</span>
              </span>
              {#if qeScore !== null}
                <span
                  class="text-[10px] font-mono font-bold px-1.5 py-0.2 rounded {qeScore >= 80 ? 'bg-emerald-100 text-emerald-800 dark:bg-emerald-950 dark:text-emerald-300' : qeScore >= 60 ? 'bg-amber-100 text-amber-800 dark:bg-amber-950 dark:text-amber-300' : 'bg-red-100 text-red-800 dark:bg-red-950 dark:text-red-300'}"
                  title="Heuristic Quality Estimation score based on length, tags and numbers preservation"
                >
                  QE: {qeScore}%
                </span>
              {/if}
            </div>
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

          <!-- Clickable numbered alternative tiles (if rephrase returned list) -->
          {#if parsedAlternatives.length > 1}
            <div class="space-y-1.5">
              <div class="text-[10px] font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wide">
                Alternative Phrasings:
              </div>
              {#each parsedAlternatives as alt, i}
                <div class="group flex items-center justify-between p-2 rounded-lg border border-gray-200 dark:border-gray-700/70 bg-white dark:bg-gray-800/80 hover:border-purple-300 dark:hover:border-purple-600 transition-colors">
                  <div class="flex items-start gap-1.5 text-gray-800 dark:text-gray-200 font-editor text-xs leading-relaxed" dir="auto">
                    <span class="font-bold text-purple-600 dark:text-purple-400 shrink-0">{i + 1}.</span>
                    <span>{alt}</span>
                  </div>
                  {#if onApplyTarget}
                    <button
                      type="button"
                      onclick={() => onApplyTarget?.(alt)}
                      class="opacity-0 group-hover:opacity-100 transition-opacity ml-2 px-1.5 py-0.5 text-[10px] rounded bg-purple-500 hover:bg-purple-600 text-white font-medium shrink-0 cursor-pointer"
                      title="Apply this alternative"
                    >
                      Apply
                    </button>
                  {/if}
                </div>
              {/each}
            </div>
          {:else}
            <div class="text-gray-800 dark:text-gray-200 whitespace-pre-wrap leading-relaxed font-editor" dir="auto">
              {reply}
            </div>
          {/if}
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
