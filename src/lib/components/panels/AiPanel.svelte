<script lang="ts">
  /**
   * AiPanel — AI Translation Assistant panel in the editor.
   * Supports local Ollama (100% offline) and OpenAI-compatible endpoints.
   * Actions: Rephrase, Critique, Freeform Ask.
   */
  import { selectedSegment } from '$lib/stores/segments';
  import { currentProject } from '$lib/stores/project';
  import { settings } from '$lib/stores/settings';
  import { queryAiAssistant, type AiConfig } from '$lib/ipc/ai';

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
        api_key: $settings.deepl_api_key, // or custom key
      };
    } else {
      return {
        provider: 'ollama',
        endpoint: $settings.ollama_url || 'http://localhost:11434',
        model: model || 'llama3.2',
      };
    }
  }

  async function handleAction(action: 'rephrase' | 'critique' | 'ask') {
    if (!$selectedSegment || !$currentProject) return;
    loading = true;
    error = null;
    reply = null;

    try {
      const res = await queryAiAssistant({
        prompt: customPrompt,
        source_text: $selectedSegment.source_text,
        target_text: $selectedSegment.target_text,
        src_lang: $currentProject.src_lang,
        tgt_lang: $currentProject.tgt_lang,
        action,
        config: getConfig(),
      });
      reply = res.reply;
      customPrompt = '';
    } catch (e: any) {
      error = String(e);
    } finally {
      loading = false;
    }
  }
</script>

<div class="flex flex-col h-full bg-white dark:bg-gray-950">
  <!-- Top bar / Settings toggle -->
  <div class="flex items-center justify-between p-2.5 border-b border-gray-200 dark:border-gray-800 bg-gray-50 dark:bg-gray-900/50 text-xs">
    <div class="flex items-center gap-1.5 font-medium text-gray-700 dark:text-gray-300">
      <span class="text-base">✨</span>
      <span>AI Assistant</span>
      <span class="text-[10px] px-1.5 py-0.2 rounded bg-purple-100 text-purple-700 dark:bg-purple-900/40 dark:text-purple-300 font-mono">
        {provider === 'ollama' ? 'Offline (Ollama)' : 'Cloud (OpenAI)'}
      </span>
    </div>
    <button
      onclick={() => showConfig = !showConfig}
      class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 p-1"
      title="Configure AI model"
    >
      ⚙️
    </button>
  </div>

  <!-- Settings Collapsible -->
  {#if showConfig}
    <div class="p-3 bg-gray-50 dark:bg-gray-900 border-b border-gray-200 dark:border-gray-800 text-xs space-y-2">
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
        <label for="ai-model-input" class="text-gray-600 dark:text-gray-400">Model:</label>
        <input
          id="ai-model-input"
          bind:value={model}
          placeholder="llama3.2, qwen2.5:7b, gpt-4o-mini"
          class="px-2 py-1 rounded border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-800 dark:text-gray-200 w-36 text-right"
        />
      </div>
      {#if provider === 'ollama'}
        <div class="text-[11px] text-gray-500">
          Make sure Ollama is running (`ollama run {model}`).
        </div>
      {/if}
    </div>
  {/if}

  <!-- Active Context -->
  <div class="flex-1 p-3 overflow-y-auto space-y-3">
    {#if !$selectedSegment}
      <div class="flex flex-col items-center justify-center h-32 text-gray-400 text-xs text-center gap-1">
        <span class="text-2xl">🤖</span>
        <span>Select a segment in the editor to use AI tools.</span>
      </div>
    {:else}
      <!-- Quick Action Buttons -->
      <div class="grid grid-cols-2 gap-2">
        <button
          onclick={() => handleAction('rephrase')}
          disabled={loading || !$selectedSegment.target_text.trim()}
          class="p-2 rounded-lg border border-purple-200 dark:border-purple-900/60
                 bg-purple-50/60 dark:bg-purple-950/20 text-purple-700 dark:text-purple-300
                 hover:bg-purple-100 dark:hover:bg-purple-900/40 text-xs font-medium transition-colors
                 disabled:opacity-40 text-left flex flex-col gap-0.5 shadow-xs"
        >
          <span class="font-bold">✨ Rephrase</span>
          <span class="text-[10px] text-purple-600/70 dark:text-purple-400/70">3 alternative phrasings</span>
        </button>

        <button
          onclick={() => handleAction('critique')}
          disabled={loading || !$selectedSegment.target_text.trim()}
          class="p-2 rounded-lg border border-blue-200 dark:border-blue-900/60
                 bg-blue-50/60 dark:bg-blue-950/20 text-blue-700 dark:text-blue-300
                 hover:bg-blue-100 dark:hover:bg-blue-900/40 text-xs font-medium transition-colors
                 disabled:opacity-40 text-left flex flex-col gap-0.5 shadow-xs"
        >
          <span class="font-bold">🔍 Critique</span>
          <span class="text-[10px] text-blue-600/70 dark:text-blue-400/70">Grammar, tone & flow</span>
        </button>
      </div>

      <!-- Loading State -->
      {#if loading}
        <div class="p-4 rounded-xl bg-purple-50 dark:bg-purple-950/30 border border-purple-200 dark:border-purple-900 text-center text-xs text-purple-600 dark:text-purple-400 flex items-center justify-center gap-2">
          <span class="animate-spin text-base">⏳</span>
          <span>Thinking with {model}…</span>
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
              <span>💡 Suggestions</span>
            </span>
            <button
              onclick={() => navigator.clipboard.writeText(reply!)}
              class="text-[11px] text-gray-500 hover:text-gray-800 dark:hover:text-gray-200"
            >
              Copy
            </button>
          </div>
          <div class="text-gray-800 dark:text-gray-200 whitespace-pre-wrap leading-relaxed font-editor">
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
        onkeydown={(e) => e.key === 'Enter' && customPrompt.trim() && handleAction('ask')}
        placeholder="Ask AI about this segment…"
        disabled={loading || !$selectedSegment}
        class="flex-1 px-2.5 py-1.5 text-xs rounded-lg border border-gray-300 dark:border-gray-700
               bg-white dark:bg-gray-800 text-gray-900 dark:text-white
               focus:outline-none focus:ring-1 focus:ring-purple-500 disabled:opacity-50"
      />
      <button
        onclick={() => handleAction('ask')}
        disabled={loading || !customPrompt.trim() || !$selectedSegment}
        class="px-3 py-1.5 text-xs bg-purple-600 hover:bg-purple-700 text-white rounded-lg
               disabled:opacity-40 transition-colors font-medium shadow-xs"
      >
        Send
      </button>
    </div>
  </div>
</div>
