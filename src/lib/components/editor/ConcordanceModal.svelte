<script lang="ts">
  /**
   * ConcordanceModal.svelte — Trados-style Concordance Search HUD (F3)
   * High-speed bidirectional FTS5 search across all project TMs.
   */
  import { onMount } from 'svelte';
  import { concordanceSearch } from '$lib/ipc/tm';
  import { currentProject, projectSettings } from '$lib/stores/project';

  export let isOpen = false;
  export let initialQuery = '';
  export let onInsert: (text: string) => void = () => {};
  export let onClose: () => void = () => {};

  let query = '';
  let searchTargetOnly = false;
  let loading = false;
  let results: any[] = [];
  let selectedIdx = 0;
  let inputEl: HTMLInputElement;

  $: if (isOpen) {
    query = initialQuery.trim();
    selectedIdx = 0;
    if (query) {
      runSearch();
    } else {
      results = [];
    }
    setTimeout(() => inputEl?.focus(), 50);
  }

  async function runSearch() {
    if (!query.trim()) return;
    loading = true;
    try {
      const res = await concordanceSearch({
        query: query.trim(),
        tm_ids: $projectSettings.tm_ids || [],
        limit: 25,
      });
      results = res || [];
      selectedIdx = 0;
    } catch (e) {
      console.error('Concordance search failed:', e);
      results = [];
    } finally {
      loading = false;
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!isOpen) return;

    if (e.key === 'Escape') {
      e.preventDefault();
      onClose();
      return;
    }

    if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIdx = Math.min(selectedIdx + 1, results.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIdx = Math.max(selectedIdx - 1, 0);
    } else if (e.key === 'Enter' && e.ctrlKey && results[selectedIdx]) {
      e.preventDefault();
      onInsert(results[selectedIdx].target_text);
      onClose();
    }
  }

  function highlightMatches(text: string, q: string): string {
    if (!q || !text) return text;
    try {
      const escaped = q.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
      const re = new RegExp(`(${escaped})`, 'gi');
      return text.replace(re, '<mark class="bg-amber-200 dark:bg-amber-900/80 text-amber-950 dark:text-amber-100 rounded px-0.5 font-bold">$1</mark>');
    } catch {
      return text;
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if isOpen}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4 select-none">
    <div class="bg-white dark:bg-gray-900 rounded-2xl border border-gray-200 dark:border-gray-800 shadow-2xl max-w-3xl w-full flex flex-col max-h-[85vh] animate-in fade-in zoom-in-95">
      <!-- Header -->
      <div class="p-4 border-b border-gray-100 dark:border-gray-800 flex items-center justify-between shrink-0">
        <div class="flex items-center gap-2">
          <span class="text-xl">📖</span>
          <div>
            <h2 class="text-base font-bold text-gray-900 dark:text-white flex items-center gap-2">
              Concordance Search
              <kbd class="text-[10px] font-mono bg-gray-100 dark:bg-gray-800 px-1.5 py-0.5 rounded text-gray-500">F3</kbd>
            </h2>
            <p class="text-xs text-gray-400">Search TM history for terminology in context</p>
          </div>
        </div>
        <button
          type="button"
          on:click={onClose}
          class="w-8 h-8 rounded-lg flex items-center justify-center text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
        >
          ✕
        </button>
      </div>

      <!-- Search Input Bar -->
      <div class="p-4 border-b border-gray-100 dark:border-gray-800 bg-gray-50/50 dark:bg-gray-950/40 shrink-0">
        <form on:submit|preventDefault={runSearch} class="flex items-center gap-2">
          <div class="relative flex-1">
            <input
              bind:this={inputEl}
              type="text"
              bind:value={query}
              placeholder="Search phrase in TM..."
              class="w-full pl-9 pr-4 py-2 rounded-xl border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-sm text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-brand-500 shadow-xs"
            />
            <span class="absolute left-3 top-2.5 text-gray-400 text-sm">🔍</span>
          </div>
          <button
            type="submit"
            disabled={loading || !query.trim()}
            class="px-5 py-2 rounded-xl bg-brand-500 hover:bg-brand-600 disabled:opacity-50 text-white font-medium text-sm transition-colors shadow-xs shrink-0 cursor-pointer"
          >
            {loading ? 'Searching…' : 'Search'}
          </button>
        </form>
      </div>

      <!-- Results Body -->
      <div class="flex-1 overflow-y-auto p-4 space-y-2.5">
        {#if loading}
          <div class="flex flex-col items-center justify-center py-12 text-gray-400 gap-2">
            <div class="w-6 h-6 border-2 border-brand-500 border-t-transparent rounded-full animate-spin"></div>
            <span class="text-xs">Searching Translation Memories...</span>
          </div>
        {:else if results.length === 0}
          <div class="text-center py-12 text-gray-400">
            <span class="text-3xl block mb-2">🔍</span>
            <p class="text-sm font-medium">No concordance matches found</p>
            <p class="text-xs mt-1">Try a different query or shorter sub-phrase</p>
          </div>
        {:else}
          {#each results as tu, i}
            <div
              class="p-3 rounded-xl border transition-all cursor-pointer
                     {i === selectedIdx
                       ? 'bg-brand-50/70 dark:bg-brand-950/40 border-brand-300 dark:border-brand-800 ring-1 ring-brand-400 shadow-xs'
                       : 'bg-white dark:bg-gray-800/60 border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800'}"
              on:click={() => selectedIdx = i}
            >
              <div class="flex items-start justify-between gap-3 mb-2">
                <div class="flex items-center gap-1.5">
                  <span class="text-[10px] font-bold font-mono px-1.5 py-0.5 rounded bg-blue-100 dark:bg-blue-950 text-blue-800 dark:text-blue-300">
                    TU #{tu.id || i + 1}
                  </span>
                  {#if tu.tm_name}
                    <span class="text-[10px] text-gray-400 font-medium">
                      in {tu.tm_name}
                    </span>
                  {/if}
                </div>
                <button
                  type="button"
                  on:click|stopPropagation={() => {
                    onInsert(tu.target_text);
                    onClose();
                  }}
                  class="px-2.5 py-1 rounded-lg bg-emerald-600 hover:bg-emerald-700 text-white text-xs font-semibold shadow-xs flex items-center gap-1 transition-colors cursor-pointer"
                  title="Insert into Target editor (Ctrl+Enter)"
                >
                  <span>Insert Target</span>
                  <kbd class="text-[9px] bg-emerald-800 px-1 rounded font-mono">↵</kbd>
                </button>
              </div>

              <!-- Source segment -->
              <div class="text-xs text-gray-700 dark:text-gray-300 font-editor leading-relaxed mb-1.5 p-2 rounded bg-gray-50/80 dark:bg-gray-900/60" dir="auto">
                <span class="text-[10px] uppercase font-bold text-gray-400 block mb-0.5">Source:</span>
                <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                {@html highlightMatches(tu.source_text, query)}
              </div>

              <!-- Target segment -->
              <div class="text-xs text-gray-900 dark:text-gray-100 font-editor font-medium leading-relaxed p-2 rounded bg-emerald-50/40 dark:bg-emerald-950/20 border border-emerald-200/50 dark:border-emerald-900/50" dir="auto">
                <span class="text-[10px] uppercase font-bold text-emerald-600 dark:text-emerald-400 block mb-0.5">Target:</span>
                <!-- eslint-disable-next-line svelte/no-at-html-tags -->
                {@html highlightMatches(tu.target_text, query)}
              </div>
            </div>
          {/each}
        {/if}
      </div>

      <!-- Footer -->
      <div class="p-3 border-t border-gray-100 dark:border-gray-800 bg-gray-50 dark:bg-gray-950/50 flex items-center justify-between text-xs text-gray-500 shrink-0">
        <span class="tabular-nums">Found {results.length} matches</span>
        <div class="flex items-center gap-3">
          <span>↑↓ Navigate</span>
          <span>Ctrl+Enter Insert</span>
          <span>Esc Close</span>
        </div>
      </div>
    </div>
  </div>
{/if}
