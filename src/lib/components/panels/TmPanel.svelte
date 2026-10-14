<script lang="ts">
  // TmPanel — shows TM matches for the selected segment and concordance search.
  import { tmMatches, tmLoading, concordanceResults, concordanceQuery } from '$lib/stores/tm';
  import { selectedSegment } from '$lib/stores/segments';
  import TmMatchBadge from '$lib/components/tm/TmMatchBadge.svelte';
  import { concordanceSearch } from '$lib/ipc/tm';

  interface Props {
    onApply: (target: string) => void;
  }
  let { onApply }: Props = $props();

  let showConcordance = $state(false);
  let searchQuery = $state('');
  let searching = $state(false);

  async function doSearch() {
    if (!searchQuery.trim()) return;
    searching = true;
    try {
      // TODO: get tm_ids from project settings store
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
    // Simple word-level highlighting
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
  <!-- Header with tabs -->
  <div class="flex border-b border-gray-200 dark:border-gray-800 shrink-0">
    <button
      onclick={() => showConcordance = false}
      class="flex-1 py-2 text-xs font-medium transition-colors
             {!showConcordance
               ? 'text-brand-600 dark:text-brand-400 border-b-2 border-brand-500'
               : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'}"
    >
      TM Matches
    </button>
    <button
      onclick={() => showConcordance = true}
      class="flex-1 py-2 text-xs font-medium transition-colors
             {showConcordance
               ? 'text-brand-600 dark:text-brand-400 border-b-2 border-brand-500'
               : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'}"
    >
      Concordance
    </button>
  </div>

  {#if !showConcordance}
    <!-- TM Matches pane -->
    <div class="flex-1 overflow-y-auto">
      {#if $tmLoading}
        <div class="flex items-center justify-center h-24 text-gray-400 text-sm">
          Searching TM…
        </div>
      {:else if $tmMatches.length === 0}
        <div class="flex flex-col items-center justify-center h-24 text-gray-400 text-sm gap-1">
          <span>No TM matches</span>
          {#if !$selectedSegment}
            <span class="text-xs">Select a segment to search</span>
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
