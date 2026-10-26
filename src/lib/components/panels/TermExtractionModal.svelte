<script lang="ts">
  import { dialog } from '$lib/actions/dialog';
  // TermExtractionModal.svelte — Automated Terminology Extraction (ATE) Workbench
  import { extractTerms, addTerm, listTbs, type ExtractedTerm, type TbInfo } from '$lib/ipc/termbase';
  import { currentProject } from '$lib/stores/project';
  import { onMount } from 'svelte';

  interface Props {
    isOpen: boolean;
    onClose: () => void;
    documentText: string;
  }

  let { isOpen, onClose, documentText }: Props = $props();

  let minFreq = $state(2);
  let maxNgram = $state(3);
  let loading = $state(false);
  let terms = $state<ExtractedTerm[]>([]);
  let availableTbs = $state<TbInfo[]>([]);
  let selectedTbId = $state<string>('');
  let targetTranslations = $state<Record<string, string>>({});
  let addedTerms = $state<Set<string>>(new Set());
  let statusMessage = $state<string | null>(null);

  onMount(async () => {
    try {
      availableTbs = await listTbs();
      if (availableTbs.length > 0) {
        selectedTbId = availableTbs[0].id;
      }
    } catch (e) {
      console.error('Failed to load termbases', e);
    }
  });

  $effect(() => {
    if (isOpen && documentText && terms.length === 0) {
      runExtraction();
    }
  });

  async function runExtraction() {
    if (!documentText.trim()) return;
    loading = true;
    statusMessage = null;
    try {
      terms = await extractTerms({
        text: documentText,
        min_freq: minFreq,
        max_ngram: maxNgram,
      });
    } catch (e) {
      statusMessage = `Extraction error: ${e}`;
    } finally {
      loading = false;
    }
  }

  async function handleAddTerm(term: string) {
    if (!selectedTbId) {
      statusMessage = 'Please select a destination termbase first.';
      return;
    }

    const tgt = targetTranslations[term]?.trim() || '';
    try {
      await addTerm({
        tb_id: selectedTbId,
        lang: $currentProject?.tgt_lang || 'en',
        term: tgt || term,
        definition: tgt ? `Source: ${term}` : undefined,
        note: `Extracted via ATE (C-value)`,
      });
      addedTerms.add(term);
      statusMessage = `Added "${term}" to termbase!`;
    } catch (e) {
      statusMessage = `Error adding term: ${e}`;
    }
  }
</script>

{#if isOpen}
  <div use:dialog={{ label: "Automated Terminology Extraction (ATE)" }} class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-xs">
    <div
      class="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-xl shadow-2xl w-full max-w-3xl max-h-[85vh] flex flex-col overflow-hidden"
    >
      <!-- Modal Header -->
      <div class="px-5 py-4 border-b border-gray-200 dark:border-gray-800 flex items-center justify-between bg-gray-50/50 dark:bg-gray-850">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 rounded-lg bg-brand-500/10 text-brand-600 dark:text-brand-400 flex items-center justify-center text-lg font-bold">
            ⚡
          </div>
          <div>
            <h2 class="text-base font-semibold text-gray-900 dark:text-gray-100">
              Automated Terminology Extraction (ATE)
            </h2>
            <p class="text-xs text-gray-500 dark:text-gray-400">
              C-value collocation extraction for technical multi-word terms & domain glossaries
            </p>
          </div>
        </div>
        <button
          onclick={onClose}
          class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 p-1.5 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
          aria-label="Close"
        >
          ✕
        </button>
      </div>

      <!-- Settings toolbar -->
      <div class="p-4 bg-gray-50 dark:bg-gray-800/40 border-b border-gray-200 dark:border-gray-800 flex items-center justify-between gap-4 flex-wrap text-xs">
        <div class="flex items-center gap-3">
          <label class="flex items-center gap-1.5 text-gray-700 dark:text-gray-300">
            <span>Min Freq:</span>
            <input
              type="number"
              bind:value={minFreq}
              min="1"
              max="10"
              class="w-14 px-2 py-1 rounded border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800"
            />
          </label>
          <label class="flex items-center gap-1.5 text-gray-700 dark:text-gray-300">
            <span>Max Words:</span>
            <input
              type="number"
              bind:value={maxNgram}
              min="1"
              max="5"
              class="w-14 px-2 py-1 rounded border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800"
            />
          </label>
          <button
            onclick={runExtraction}
            disabled={loading}
            class="px-3 py-1 bg-brand-500 hover:bg-brand-600 disabled:opacity-50 text-white font-medium rounded shadow-xs transition-colors"
          >
            {loading ? 'Extracting…' : 'Re-Extract'}
          </button>
        </div>

        <div class="flex items-center gap-2">
          <span class="text-gray-600 dark:text-gray-400">Target TB:</span>
          {#if availableTbs.length > 0}
            <select
              bind:value={selectedTbId}
              class="py-1 px-2 rounded border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-800 dark:text-gray-200 text-xs"
            >
              {#each availableTbs as tb}
                <option value={tb.id}>{tb.name} ({tb.count} terms)</option>
              {/each}
            </select>
          {:else}
            <span class="text-amber-600 dark:text-amber-400 italic">No Termbases created yet</span>
          {/if}
        </div>
      </div>

      {#if statusMessage}
        <div class="px-4 py-2 bg-brand-50 dark:bg-brand-950/40 text-brand-700 dark:text-brand-300 text-xs border-b border-brand-200/50 dark:border-brand-800/50 flex justify-between items-center">
          <span>{statusMessage}</span>
          <button onclick={() => statusMessage = null} class="text-xs hover:underline">Dismiss</button>
        </div>
      {/if}

      <!-- Term list table -->
      <div class="flex-1 overflow-y-auto p-4">
        {#if loading}
          <div class="flex flex-col items-center justify-center h-48 text-gray-400 gap-2">
            <div class="w-6 h-6 border-2 border-brand-500 border-t-transparent rounded-full animate-spin"></div>
            <span class="text-sm">Analyzing collocations & scoring C-values…</span>
          </div>
        {:else if terms.length === 0}
          <div class="flex flex-col items-center justify-center h-48 text-gray-400 text-sm">
            <span>No terms found matching the current frequency threshold.</span>
            <span class="text-xs text-gray-500 mt-1">Try lowering the Min Freq or extracting from larger text.</span>
          </div>
        {:else}
          <table class="w-full text-left text-xs">
            <thead>
              <tr class="border-b border-gray-200 dark:border-gray-800 text-gray-500 dark:text-gray-400 uppercase tracking-wider text-[10px]">
                <th class="pb-2 font-semibold">Extracted Term</th>
                <th class="pb-2 font-semibold text-center w-16">Freq</th>
                <th class="pb-2 font-semibold text-center w-20">C-Value</th>
                <th class="pb-2 font-semibold">Translation (Optional)</th>
                <th class="pb-2 font-semibold text-right w-24">Action</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-100 dark:divide-gray-800/60">
              {#each terms as t (t.term)}
                <tr class="hover:bg-gray-50 dark:hover:bg-gray-800/40 transition-colors">
                  <td class="py-2.5 pr-2">
                    <div class="font-medium text-gray-900 dark:text-gray-100 text-sm">
                      {t.term}
                    </div>
                    {#if t.sample_context}
                      <div class="text-[11px] text-gray-400 dark:text-gray-500 line-clamp-1 italic mt-0.5" title={t.sample_context}>
                        "{t.sample_context}"
                      </div>
                    {/if}
                  </td>
                  <td class="py-2.5 text-center font-mono text-gray-600 dark:text-gray-400">
                    {t.frequency}×
                  </td>
                  <td class="py-2.5 text-center">
                    <span class="px-1.5 py-0.5 rounded bg-blue-50 dark:bg-blue-950/40 text-blue-700 dark:text-blue-300 font-mono text-[10px] font-semibold">
                      {t.c_value.toFixed(1)}
                    </span>
                  </td>
                  <td class="py-2.5 pr-2">
                    <input
                      type="text"
                      bind:value={targetTranslations[t.term]}
                      placeholder="Translate to target…"
                      class="w-full px-2 py-1 text-xs rounded border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-800 dark:text-gray-200"
                    />
                  </td>
                  <td class="py-2.5 text-right">
                    {#if addedTerms.has(t.term)}
                      <span class="text-emerald-600 dark:text-emerald-400 font-medium inline-flex items-center gap-1 text-[11px]">
                        ✓ Added
                      </span>
                    {:else}
                      <button
                        onclick={() => handleAddTerm(t.term)}
                        class="px-2.5 py-1 bg-brand-50 hover:bg-brand-100 dark:bg-brand-950/40 dark:hover:bg-brand-900/40 text-brand-600 dark:text-brand-300 font-medium rounded transition-colors text-[11px]"
                      >
                        + Add
                      </button>
                    {/if}
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}
      </div>

      <!-- Modal Footer -->
      <div class="p-3 border-t border-gray-200 dark:border-gray-800 bg-gray-50 dark:bg-gray-850 flex justify-between items-center text-xs text-gray-500">
        <span>Found {terms.length} candidate terms</span>
        <button
          onclick={onClose}
          class="px-4 py-1.5 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 text-gray-800 dark:text-gray-200 rounded font-medium transition-colors"
        >
          Close
        </button>
      </div>
    </div>
  </div>
{/if}
