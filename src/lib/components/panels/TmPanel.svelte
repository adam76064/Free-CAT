<script lang="ts">
  // TmPanel — shows TM matches, MT suggestions, and concordance search.
  import { tmMatches, tmLoading, concordanceResults, concordanceQuery } from '$lib/stores/tm';
  import { selectedSegment } from '$lib/stores/segments';
  import { currentProject } from '$lib/stores/project';
  import { settings } from '$lib/stores/settings';
  import TmMatchBadge from '$lib/components/tm/TmMatchBadge.svelte';
  import { concordanceSearch } from '$lib/ipc/tm';
  import { getMtSuggestion } from '$lib/ipc/project';

  interface Props {
    onApply: (target: string) => void;
  }
  let { onApply }: Props = $props();

  let activeTab = $state<'tm' | 'mt' | 'concordance'>('tm');

  // Concordance state
  let searchQuery = $state('');
  let searching = $state(false);

  // MT state
  let mtEngine = $state<'deepl' | 'libretranslate' | 'google'>('libretranslate');
  let mtLoading = $state(false);
  let mtResult = $state<string | null>(null);
  let mtError = $state<string | null>(null);
  let lastFetchedSource = $state<string | null>(null);

  // Watch selectedSegment and reset MT result if source changed
  $effect(() => {
    if ($selectedSegment && $selectedSegment.source_text !== lastFetchedSource) {
      mtResult = null;
      mtError = null;
    }
  });

  async function fetchMt() {
    if (!$selectedSegment || !$currentProject) return;
    mtLoading = true;
    mtError = null;
    lastFetchedSource = $selectedSegment.source_text;

    try {
      const apiKey = mtEngine === 'deepl'
        ? $settings.deepl_api_key
        : mtEngine === 'google'
        ? $settings.google_translate_key
        : undefined;

      const apiUrl = mtEngine === 'libretranslate'
        ? $settings.libretranslate_url
        : undefined;

      const res = await getMtSuggestion({
        source_text: $selectedSegment.source_text,
        src_lang: $currentProject.src_lang,
        tgt_lang: $currentProject.tgt_lang,
        engine: mtEngine,
        api_key: apiKey,
        api_url: apiUrl,
      });

      mtResult = res.translated_text;
    } catch (e: any) {
      mtError = String(e);
      mtResult = null;
    } finally {
      mtLoading = false;
    }
  }

  async function doSearch() {
    if (!searchQuery.trim()) return;
    searching = true;
    try {
      concordanceResults.set(await concordanceSearch({
        query: searchQuery,
        tm_ids: [],
        limit: 20,
      }));
      concordanceQuery.set(searchQuery);
    } finally {
      searching = false;
    }
  }

  // Highlight diff between TM source and current source
  function highlightDiff(tmSrc: string, curSrc: string): string {
    if (tmSrc === curSrc) return tmSrc;
    const tmWords = tmSrc.split(' ');
    const curWords = curSrc.split(' ');
    return tmWords.map(w =>
      curWords.includes(w)
        ? w
        : `<mark class="bg-red-100 dark:bg-red-900/30 rounded px-0.5">${w}</mark>`
    ).join(' ');
  }
</script>

<div class="flex flex-col h-full">
  <!-- Header with sub-tabs -->
  <div class="flex border-b border-gray-200 dark:border-gray-800 shrink-0 bg-gray-50 dark:bg-gray-900/50">
    <button
      onclick={() => activeTab = 'tm'}
      class="flex-1 py-1.5 text-xs font-medium transition-colors text-center
             {activeTab === 'tm'
               ? 'text-brand-600 dark:text-brand-400 border-b-2 border-brand-500 bg-white dark:bg-gray-900 font-semibold'
               : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'}"
    >
      TM Matches
    </button>
    <button
      onclick={() => activeTab = 'mt'}
      class="flex-1 py-1.5 text-xs font-medium transition-colors text-center
             {activeTab === 'mt'
               ? 'text-brand-600 dark:text-brand-400 border-b-2 border-brand-500 bg-white dark:bg-gray-900 font-semibold'
               : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'}"
    >
      🤖 Machine Trans
    </button>
    <button
      onclick={() => activeTab = 'concordance'}
      class="flex-1 py-1.5 text-xs font-medium transition-colors text-center
             {activeTab === 'concordance'
               ? 'text-brand-600 dark:text-brand-400 border-b-2 border-brand-500 bg-white dark:bg-gray-900 font-semibold'
               : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'}"
    >
      Concordance
    </button>
  </div>

  {#if activeTab === 'tm'}
    <!-- TM Matches pane -->
    <div class="flex-1 overflow-y-auto">
      {#if $tmLoading}
        <div class="flex items-center justify-center h-24 text-gray-400 text-sm">
          Searching TM…
        </div>
      {:else if $tmMatches.length === 0}
        <div class="flex flex-col items-center justify-center h-28 text-gray-400 text-sm gap-1 px-4 text-center">
          <span>No TM matches found</span>
          {#if !$selectedSegment}
            <span class="text-xs">Select a segment to search</span>
          {:else}
            <span class="text-xs text-gray-400">Score &lt; threshold. Try Concordance or MT.</span>
          {/if}
        </div>
      {:else}
        <div class="divide-y divide-gray-100 dark:divide-gray-800">
          {#each $tmMatches as match, i (match.tu_id)}
            <button
              onclick={() => onApply(match.target_text)}
              class="w-full text-left p-3 hover:bg-gray-50 dark:hover:bg-gray-800/50 transition-colors group"
            >
              <div class="flex items-center justify-between mb-1.5">
                <div class="flex items-center gap-2">
                  <TmMatchBadge pct={match.score_pct} />
                  <span class="text-xs text-gray-500 dark:text-gray-400 truncate max-w-[120px]">
                    {match.tm_name}
                  </span>
                </div>
                <kbd class="text-xs text-gray-400 dark:text-gray-600 font-mono hidden group-hover:inline">
                  Alt+{i + 1}
                </kbd>
              </div>
              <!-- Source with diff highlighting -->
              <div
                class="text-xs text-gray-500 dark:text-gray-400 mb-1 line-clamp-2 font-editor"
                title={match.source_text}
              >
                <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                {@html highlightDiff(match.source_text, $selectedSegment?.source_text ?? '')}
              </div>
              <!-- Target text -->
              <div class="text-sm text-gray-800 dark:text-gray-200 line-clamp-2 font-editor">
                {match.target_text}
              </div>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  {:else if activeTab === 'mt'}
    <!-- MT Pane -->
    <div class="flex-1 flex flex-col p-3 overflow-y-auto gap-3">
      <div class="flex items-center justify-between">
        <label for="mt-engine-select" class="text-xs font-semibold text-gray-700 dark:text-gray-300">Engine:</label>
        <select
          id="mt-engine-select"
          bind:value={mtEngine}
          class="text-xs py-1 px-2 rounded-lg border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-800 dark:text-gray-200"
        >
          <option value="libretranslate">LibreTranslate (Local/Self-hosted)</option>
          <option value="deepl">DeepL API</option>
          <option value="google">Google Translate API</option>
        </select>
      </div>

      {#if !$selectedSegment}
        <div class="flex items-center justify-center h-24 text-gray-400 text-xs text-center">
          Select a segment in the editor to fetch MT suggestions.
        </div>
      {:else}
        <div class="p-2.5 rounded-lg bg-gray-50 dark:bg-gray-800/60 border border-gray-200 dark:border-gray-700 text-xs">
          <div class="text-[11px] text-gray-400 dark:text-gray-500 mb-1 font-semibold">Source:</div>
          <div class="text-gray-700 dark:text-gray-300 line-clamp-3 italic">
            {$selectedSegment.source_text}
          </div>
        </div>

        <button
          onclick={fetchMt}
          disabled={mtLoading}
          class="w-full py-2 bg-brand-500 hover:bg-brand-600 disabled:opacity-50 text-white rounded-lg text-xs font-medium transition-colors shadow-sm flex items-center justify-center gap-1.5"
        >
          <span>{mtLoading ? 'Translating…' : '⚡ Translate with ' + mtEngine.toUpperCase()}</span>
        </button>

        {#if mtError}
          <div class="p-2.5 rounded-lg bg-red-50 dark:bg-red-950/30 border border-red-200 dark:border-red-900 text-red-600 dark:text-red-400 text-xs">
            {mtError}
            <div class="mt-1 text-[11px] text-gray-500">
              Configure your API keys in the Settings modal.
            </div>
          </div>
        {/if}

        {#if mtResult}
          <div class="p-3 rounded-lg bg-emerald-50 dark:bg-emerald-950/20 border border-emerald-200 dark:border-emerald-800 flex flex-col gap-2">
            <div class="flex items-center justify-between">
              <span class="text-[11px] font-semibold text-emerald-700 dark:text-emerald-400">
                MT Suggestion ({mtEngine})
              </span>
              <button
                onclick={() => onApply(mtResult!)}
                class="px-2 py-0.5 bg-emerald-600 hover:bg-emerald-700 text-white text-[11px] font-medium rounded transition-colors shadow-xs"
              >
                Apply to Target ↵
              </button>
            </div>
            <div class="text-sm text-gray-900 dark:text-gray-100 font-editor leading-relaxed">
              {mtResult}
            </div>
          </div>
        {/if}
      {/if}
    </div>
  {:else}
    <!-- Concordance search pane -->
    <div class="p-2 border-b border-gray-100 dark:border-gray-800 shrink-0">
      <div class="flex gap-1.5">
        <input
          bind:value={searchQuery}
          onkeydown={(e) => e.key === 'Enter' && doSearch()}
          placeholder="Search TM…"
          class="flex-1 px-2 py-1.5 text-xs rounded-lg border border-gray-300 dark:border-gray-700
                 bg-white dark:bg-gray-800 text-gray-900 dark:text-white
                 focus:outline-none focus:ring-1 focus:ring-brand-500"
        />
        <button
          onclick={doSearch}
          disabled={searching}
          class="px-3 py-1.5 text-xs bg-brand-500 hover:bg-brand-600 text-white rounded-lg
                 disabled:opacity-50 transition-colors"
        >
          {searching ? '…' : 'Search'}
        </button>
      </div>
    </div>
    <div class="flex-1 overflow-y-auto">
      {#if $concordanceResults.length === 0}
        <div class="flex items-center justify-center h-24 text-gray-400 text-sm">
          {$concordanceQuery ? 'No results' : 'Enter a search term'}
        </div>
      {:else}
        <div class="divide-y divide-gray-100 dark:divide-gray-800">
          {#each $concordanceResults as tu}
            <button
              onclick={() => onApply(tu.target_text)}
              class="w-full text-left p-3 hover:bg-gray-50 dark:hover:bg-gray-800/50 transition-colors"
            >
              <div class="text-xs text-gray-500 dark:text-gray-400 mb-0.5 font-editor line-clamp-2">
                {tu.source_text}
              </div>
              <div class="text-sm text-gray-800 dark:text-gray-200 font-editor line-clamp-2">
                {tu.target_text}
              </div>
            </button>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>
