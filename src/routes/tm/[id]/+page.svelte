<script lang="ts">
  /**
   * TM Maintenance Studio — /tm/[id]
   * Trados-style Translation Memory maintenance interface:
   * - Paginated TU browsing
   * - In-place editing of source and target segments
   * - Deletion of obsolete or corrupt units
   * - Batch Find-and-Replace across the entire TM
   */
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import {
    getTmTusPaginated, updateTmTu, deleteTmTu, batchReplaceTm,
    listTms, type TranslationUnit, type TmInfo
  } from '$lib/ipc/tm';
  import { toast } from '$lib/stores/toast';

  const tmId: string = $page.params.id || '';

  let tmInfo = $state<TmInfo | null>(null);
  let tus = $state<TranslationUnit[]>([]);
  let totalCount = $state(0);
  let pageIndex = $state(0);
  const pageSize = 50;
  let searchQuery = $state('');
  let loading = $state(true);
  let savingId = $state<number | null>(null);

  // Batch Replace Modal State
  let showBatchReplaceModal = $state(false);
  let findText = $state('');
  let replaceText = $state('');
  let targetOnly = $state(true);
  let batchReplacing = $state(false);

  // Track edited values: { [tuId: number]: { source: string, target: string, dirty: boolean } }
  let editedTus = $state<Record<number, { source: string; target: string; dirty: boolean }>>({});

  onMount(async () => {
    await loadTmHeader();
    await loadTus();
  });

  async function loadTmHeader() {
    try {
      const allTms = await listTms();
      tmInfo = allTms.find(t => t.id === tmId) ?? null;
    } catch (e) {
      console.error('Failed to load TM header:', e);
    }
  }

  async function loadTus() {
    loading = true;
    try {
      const res = await getTmTusPaginated({
        tm_id: tmId,
        offset: pageIndex * pageSize,
        limit: pageSize,
        search: searchQuery.trim() || undefined,
      });
      tus = res.tus;
      totalCount = res.total_count;

      // Initialize edit state
      const nextEdits: Record<number, { source: string; target: string; dirty: boolean }> = {};
      for (const tu of res.tus) {
        if (tu.id) {
          nextEdits[tu.id] = {
            source: tu.source_text,
            target: tu.target_text,
            dirty: false,
          };
        }
      }
      editedTus = nextEdits;
    } catch (e) {
      console.error('Failed to load TUs:', e);
    } finally {
      loading = false;
    }
  }

  function handleSearchInput() {
    pageIndex = 0;
    loadTus();
  }

  function handleTextChange(tuId: number, field: 'source' | 'target', value: string) {
    if (!editedTus[tuId]) return;
    editedTus[tuId][field] = value;
    editedTus[tuId].dirty = true;
  }

  async function handleSaveTu(tuId: number) {
    const edit = editedTus[tuId];
    if (!edit) return;
    savingId = tuId;
    try {
      await updateTmTu({
        tm_id: tmId,
        tu_id: tuId,
        source_text: edit.source,
        target_text: edit.target,
      });
      edit.dirty = false;
    } catch (e: any) {
      toast.error('Save Failed', String(e));
    } finally {
      savingId = null;
    }
  }

  async function handleDeleteTu(tuId: number) {
    try {
      await deleteTmTu({ tm_id: tmId, tu_id: tuId });
      await loadTus();
      await loadTmHeader();
      toast.success('Unit Deleted', `TU #${tuId} removed.`);
    } catch (e: any) {
      toast.error('Delete Failed', String(e));
    }
  }

  async function handleExecuteBatchReplace() {
    if (!findText.trim()) return;
    batchReplacing = true;
    try {
      const affected = await batchReplaceTm({
        tm_id: tmId,
        find_text: findText,
        replace_text: replaceText,
        target_only: targetOnly,
      });
      toast.success('Batch Replace Complete', `Updated ${affected} translation units.`);
      showBatchReplaceModal = false;
      findText = '';
      replaceText = '';
      await loadTus();
    } catch (e: any) {
      toast.error('Batch Replace Failed', String(e));
    } finally {
      batchReplacing = false;
    }
  }

  let totalPages = $derived(Math.max(1, Math.ceil(totalCount / pageSize)));
</script>

<div class="h-full flex flex-col bg-gray-50 dark:bg-gray-950 text-gray-900 dark:text-gray-100">
  <!-- Studio Top Navigation Header -->
  <div class="px-6 py-4 border-b border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 flex items-center justify-between shrink-0">
    <div class="flex items-center gap-3">
      <button
        onclick={() => goto('/tm')}
        class="px-2.5 py-1 text-xs font-semibold rounded-lg border border-gray-200 dark:border-gray-700 text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors flex items-center gap-1"
      >
        <span>←</span>
        <span>TMs</span>
      </button>

      <div>
        <div class="flex items-center gap-2">
          <h1 class="text-lg font-bold text-gray-900 dark:text-white">
            {tmInfo?.name || 'Translation Memory'}
          </h1>
          <span class="px-2 py-0.5 rounded text-[11px] font-mono font-bold bg-brand-50 text-brand-700 dark:bg-brand-950 dark:text-brand-300 border border-brand-200 dark:border-brand-800">
            {tmInfo?.src_lang || '—'} ➔ {tmInfo?.tgt_lang || '—'}
          </span>
          <span class="text-xs text-gray-500 font-mono">
            ({totalCount.toLocaleString()} units)
          </span>
        </div>
        <div class="text-xs text-gray-400">
          Trados-style Translation Memory Maintenance Studio
        </div>
      </div>
    </div>

    <!-- Actions -->
    <div class="flex items-center gap-2">
      <button
        onclick={() => (showBatchReplaceModal = true)}
        class="px-3 py-1.5 rounded-lg border border-gray-200 dark:border-gray-700 text-xs font-medium text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors flex items-center gap-1.5 shadow-2xs"
      >
        <span>🔍 Batch Replace</span>
      </button>

      <button
        onclick={loadTus}
        class="px-3 py-1.5 rounded-lg bg-brand-500 hover:bg-brand-600 text-white text-xs font-medium shadow-xs transition-colors"
      >
        ↻ Refresh
      </button>
    </div>
  </div>

  <!-- Search & Filter Toolbar -->
  <div class="px-6 py-2.5 bg-gray-100/70 dark:bg-gray-900/40 border-b border-gray-200 dark:border-gray-800 flex items-center justify-between shrink-0 text-xs">
    <div class="flex items-center gap-2 w-96">
      <div class="relative w-full">
        <input
          type="text"
          placeholder="Search in source or target text…"
          bind:value={searchQuery}
          oninput={handleSearchInput}
          class="w-full pl-8 pr-3 py-1.5 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-xs focus:outline-none focus:ring-1 focus:ring-brand-500"
        />
        <span class="absolute left-2.5 top-2 text-gray-400">🔍</span>
      </div>
      {#if searchQuery}
        <button
          onclick={() => { searchQuery = ''; handleSearchInput(); }}
          class="text-xs text-gray-400 hover:text-gray-600"
        >
          Clear
        </button>
      {/if}
    </div>

    <!-- Pagination Summary -->
    <div class="flex items-center gap-3 text-xs text-gray-500 dark:text-gray-400">
      <span>
        Showing {totalCount === 0 ? 0 : pageIndex * pageSize + 1}–{Math.min(totalCount, (pageIndex + 1) * pageSize)} of {totalCount.toLocaleString()}
      </span>
      <div class="flex items-center gap-1">
        <button
          disabled={pageIndex === 0}
          onclick={() => { pageIndex--; loadTus(); }}
          class="px-2 py-1 rounded border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 disabled:opacity-40 disabled:cursor-not-allowed hover:bg-gray-50"
        >
          ◀
        </button>
        <span class="font-mono text-xs px-1">
          {pageIndex + 1} / {totalPages}
        </span>
        <button
          disabled={pageIndex >= totalPages - 1}
          onclick={() => { pageIndex++; loadTus(); }}
          class="px-2 py-1 rounded border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 disabled:opacity-40 disabled:cursor-not-allowed hover:bg-gray-50"
        >
          ▶
        </button>
      </div>
    </div>
  </div>

  <!-- Translation Units Table Canvas -->
  <div class="flex-1 overflow-y-auto p-6">
    {#if loading}
      <div class="flex items-center justify-center h-48 text-gray-400 text-sm gap-2">
        <div class="w-5 h-5 border-2 border-brand-500 border-t-transparent rounded-full animate-spin"></div>
        <span>Loading Translation Units…</span>
      </div>
    {:else if tus.length === 0}
      <div class="flex flex-col items-center justify-center h-48 text-gray-400 text-sm gap-2">
        <span>No Translation Units found matching the search criteria.</span>
      </div>
    {:else}
      <div class="space-y-3">
        {#each tus as tu (tu.id)}
          {@const edit = tu.id ? editedTus[tu.id] : null}
          <div class="p-3 bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 shadow-2xs transition-all hover:border-gray-300 dark:hover:border-gray-700">
            <!-- Row Header: TU ID, Usage, Timestamp, Actions -->
            <div class="flex items-center justify-between pb-2 mb-2 border-b border-gray-100 dark:border-gray-800/80 text-[11px] text-gray-400">
              <div class="flex items-center gap-2">
                <span class="font-bold font-mono px-1.5 py-0.5 rounded bg-blue-50 dark:bg-blue-950 text-blue-700 dark:text-blue-300">
                  TU #{tu.id}
                </span>
                {#if tu.usage_count}
                  <span class="font-mono">
                    Used: {tu.usage_count}x
                  </span>
                {/if}
                {#if tu.modified_at}
                  <span>
                    Modified: {new Date(tu.modified_at).toLocaleDateString()}
                  </span>
                {/if}
              </div>

              <div class="flex items-center gap-2">
                {#if edit && edit.dirty}
                  <span class="text-amber-600 dark:text-amber-400 font-medium text-[10px] animate-pulse">
                    ● Unsaved Changes
                  </span>
                {/if}
                <button
                  type="button"
                  disabled={!edit || !edit.dirty || savingId === tu.id}
                  onclick={() => tu.id && handleSaveTu(tu.id)}
                  class="px-2.5 py-1 rounded bg-emerald-600 hover:bg-emerald-700 disabled:opacity-30 disabled:cursor-not-allowed text-white text-xs font-semibold shadow-2xs transition-colors flex items-center gap-1"
                >
                  {#if savingId === tu.id}
                    <div class="w-3 h-3 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
                  {/if}
                  <span>Save</span>
                </button>
                <button
                  type="button"
                  onclick={() => tu.id && handleDeleteTu(tu.id)}
                  class="px-2.5 py-1 rounded text-red-500 hover:bg-red-50 dark:hover:bg-red-950/40 text-xs transition-colors"
                >
                  Delete
                </button>
              </div>
            </div>

            <!-- Side-by-side or stacked source/target editor -->
            <div class="grid grid-cols-1 md:grid-cols-2 gap-3 text-xs">
              <!-- Source -->
              <div>
                <span class="text-[10px] uppercase font-bold text-gray-400 block mb-1">
                  Source ({tu.source_lang})
                </span>
                <textarea
                  rows="2"
                  value={edit?.source ?? tu.source_text}
                  oninput={(e) => tu.id && handleTextChange(tu.id, 'source', e.currentTarget.value)}
                  class="w-full p-2 rounded-lg border border-gray-200 dark:border-gray-700 bg-gray-50/50 dark:bg-gray-800/50 font-editor leading-relaxed text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-1 focus:ring-brand-500"
                ></textarea>
              </div>

              <!-- Target -->
              <div>
                <span class="text-[10px] uppercase font-bold text-gray-400 block mb-1">
                  Target ({tu.target_lang})
                </span>
                <textarea
                  rows="2"
                  value={edit?.target ?? tu.target_text}
                  oninput={(e) => tu.id && handleTextChange(tu.id, 'target', e.currentTarget.value)}
                  class="w-full p-2 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 font-editor leading-relaxed text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-1 focus:ring-brand-500"
                ></textarea>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<!-- Batch Find and Replace Modal -->
{#if showBatchReplaceModal}
  <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-xs">
    <div class="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-2xl shadow-2xl max-w-md w-full p-6 animate-in fade-in zoom-in-95">
      <h3 class="text-base font-bold text-gray-900 dark:text-white mb-1">
        Batch Find & Replace across TM
      </h3>
      <p class="text-xs text-gray-500 dark:text-gray-400 mb-4">
        Replaces text in all Translation Units of this TM database.
      </p>

      <div class="space-y-3 mb-5 text-xs">
        <div>
          <label for="find-text" class="block font-medium text-gray-700 dark:text-gray-300 mb-1">
            Find Text:
          </label>
          <input
            id="find-text"
            type="text"
            bind:value={findText}
            placeholder="e.g. Acme Corp"
            class="w-full px-3 py-2 rounded-lg border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:outline-none focus:ring-1 focus:ring-brand-500"
          />
        </div>

        <div>
          <label for="replace-text" class="block font-medium text-gray-700 dark:text-gray-300 mb-1">
            Replace With:
          </label>
          <input
            id="replace-text"
            type="text"
            bind:value={replaceText}
            placeholder="e.g. Acme Global Inc."
            class="w-full px-3 py-2 rounded-lg border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:outline-none focus:ring-1 focus:ring-brand-500"
          />
        </div>

        <label class="flex items-center gap-2 cursor-pointer pt-1">
          <input
            type="checkbox"
            bind:checked={targetOnly}
            class="rounded text-brand-500 accent-brand-500"
          />
          <span class="text-gray-700 dark:text-gray-300">
            Replace in Target Translation only (recommended)
          </span>
        </label>
      </div>

      <div class="flex items-center justify-end gap-2">
        <button
          type="button"
          onclick={() => (showBatchReplaceModal = false)}
          class="px-3 py-1.5 text-xs text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg transition-colors"
        >
          Cancel
        </button>
        <button
          type="button"
          disabled={!findText.trim() || batchReplacing}
          onclick={handleExecuteBatchReplace}
          class="px-4 py-1.5 text-xs font-semibold text-white bg-brand-500 hover:bg-brand-600 disabled:opacity-50 rounded-lg shadow-sm transition-colors flex items-center gap-1.5"
        >
          {#if batchReplacing}
            <div class="w-3 h-3 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
          {/if}
          <span>Replace All</span>
        </button>
      </div>
    </div>
  </div>
{/if}
