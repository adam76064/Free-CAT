<script lang="ts">
  /**
   * AdvancedDisplayFilterPanel.svelte — Trados Advanced Display Filter 2.0 Studio
   * Provides multi-criteria compound filtering:
   * - Statuses (New, Draft, Translated, Needs Review)
   * - Attributes (Locked, Bookmarked, QA Issues, Comments)
   * - Origin / Match Tiers (100%, Fuzzy, Fragment, MT, Auto-propagated)
   * - Document Structure (Headings, Paragraphs, Tables, Lists, Footnotes)
   * - Regular Expressions (Source / Target) with case sensitivity
   * - Invert Filter query ("Show non-matching")
   * - ISO 17100 QA Audit Sampling slider (1% to 100%)
   */
  import {
    advancedFilter, defaultAdvancedFilter, segments,
    filteredSegments, type AdvancedFilterCriteria, type SegmentStatus
  } from '$lib/stores/segments';

  export let isOpen = false;
  export let onClose: () => void = () => {};

  const availableStatuses: SegmentStatus[] = ['new', 'draft', 'translated', 'needs_review', 'reviewed'];

  let draft = { ...$advancedFilter };

  $: if (isOpen) {
    draft = { ...$advancedFilter };
  }

  function toggleStatus(st: SegmentStatus) {
    if (draft.statuses.includes(st)) {
      draft.statuses = draft.statuses.filter(s => s !== st);
    } else {
      draft.statuses = [...draft.statuses, st];
    }
  }

  function toggleMatchTier(tier: string) {
    if (draft.matchTiers.includes(tier)) {
      draft.matchTiers = draft.matchTiers.filter(t => t !== tier);
    } else {
      draft.matchTiers = [...draft.matchTiers, tier];
    }
  }

  function toggleStructure(struct: string) {
    if (draft.structureTypes.includes(struct)) {
      draft.structureTypes = draft.structureTypes.filter(s => s !== struct);
    } else {
      draft.structureTypes = [...draft.structureTypes, struct];
    }
  }

  function applyFilter() {
    draft.isActive = true;
    advancedFilter.set({ ...draft });
  }

  function resetFilter() {
    draft = { ...defaultAdvancedFilter, isActive: false };
    advancedFilter.set({ ...defaultAdvancedFilter });
  }

  function clearActive() {
    advancedFilter.update(f => ({ ...f, isActive: false }));
  }
</script>

{#if isOpen}
  <div class="p-3 bg-gray-50/95 dark:bg-gray-900/95 border-b border-gray-200 dark:border-gray-800 shadow-md text-xs animate-in slide-in-from-top-2 select-none">
    <div class="flex items-center justify-between pb-2 mb-2 border-b border-gray-200/80 dark:border-gray-800">
      <div class="flex items-center gap-2">
        <span class="text-sm">🎯</span>
        <span class="font-bold text-gray-900 dark:text-gray-100 text-xs">Advanced Display Filter 2.0</span>
        {#if $advancedFilter.isActive}
          <span class="px-2 py-0.5 rounded-full text-[10px] font-bold bg-brand-100 text-brand-800 dark:bg-brand-900/50 dark:text-brand-300">
            Active ({$filteredSegments.length}/{$segments.length} segments)
          </span>
        {/if}
      </div>
      <div class="flex items-center gap-2">
        {#if $advancedFilter.isActive}
          <button
            type="button"
            on:click={clearActive}
            class="px-2 py-1 text-[11px] rounded bg-gray-200 hover:bg-gray-300 dark:bg-gray-800 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-300"
          >
            Pause Filter
          </button>
        {/if}
        <button
          type="button"
          on:click={resetFilter}
          class="px-2 py-1 text-[11px] rounded text-red-600 hover:bg-red-50 dark:hover:bg-red-950/40"
        >
          Reset All
        </button>
        <button
          type="button"
          on:click={applyFilter}
          class="px-3 py-1 text-[11px] font-bold rounded bg-brand-500 hover:bg-brand-600 text-white shadow-xs"
        >
          Apply Filter
        </button>
        <button
          type="button"
          on:click={onClose}
          class="w-6 h-6 flex items-center justify-center rounded hover:bg-gray-200 dark:hover:bg-gray-800 text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 font-bold"
        >
          ✕
        </button>
      </div>
    </div>

    <!-- Filter Columns Grid -->
    <div class="grid grid-cols-1 md:grid-cols-4 gap-3">
      <!-- Col 1: Status & Attributes -->
      <div class="space-y-2 p-2 bg-white dark:bg-gray-950/70 rounded-lg border border-gray-200 dark:border-gray-800">
        <div class="text-[10px] font-bold uppercase tracking-wider text-gray-400">1. Segment Status</div>
        <div class="flex flex-wrap gap-1">
          {#each availableStatuses as st}
            <button
              type="button"
              on:click={() => toggleStatus(st)}
              class="px-2 py-0.5 rounded text-[11px] font-medium border transition-colors cursor-pointer
                     {draft.statuses.includes(st)
                       ? 'bg-brand-500 text-white border-brand-600 font-bold'
                       : 'bg-gray-50 text-gray-600 dark:bg-gray-900 dark:text-gray-400 border-gray-200 dark:border-gray-800 hover:bg-gray-100'}"
            >
              {st === 'new' ? 'Untranslated' : st === 'draft' ? 'Draft' : st === 'translated' ? 'Translated' : 'Needs Review'}
            </button>
          {/each}
        </div>

        <div class="text-[10px] font-bold uppercase tracking-wider text-gray-400 pt-1">Attributes</div>
        <div class="flex flex-col gap-1 text-[11px]">
          <label class="flex items-center gap-1.5 cursor-pointer">
            <input type="checkbox" bind:checked={draft.bookmarkedOnly} class="rounded text-brand-500 accent-brand-500" />
            <span>★ Bookmarked only</span>
          </label>
          <label class="flex items-center gap-1.5 cursor-pointer">
            <input type="checkbox" bind:checked={draft.qaIssuesOnly} class="rounded text-brand-500 accent-brand-500" />
            <span>⚠️ With QA warnings</span>
          </label>
          <label class="flex items-center gap-1.5 cursor-pointer">
            <input type="checkbox" bind:checked={draft.commentsOnly} class="rounded text-brand-500 accent-brand-500" />
            <span>💬 With comments</span>
          </label>
          <div class="flex items-center gap-1 pt-0.5">
            <span class="text-gray-500">Lock:</span>
            <button
              type="button"
              on:click={() => draft.lockedOnly = draft.lockedOnly === true ? null : true}
              class="px-1.5 py-0.5 rounded text-[10px] border {draft.lockedOnly === true ? 'bg-rose-500 text-white border-rose-600 font-bold' : 'bg-gray-100 dark:bg-gray-900 text-gray-600 border-gray-200 dark:border-gray-800'}"
            >🔒 Only Locked</button>
            <button
              type="button"
              on:click={() => draft.lockedOnly = draft.lockedOnly === false ? null : false}
              class="px-1.5 py-0.5 rounded text-[10px] border {draft.lockedOnly === false ? 'bg-emerald-500 text-white border-emerald-600 font-bold' : 'bg-gray-100 dark:bg-gray-900 text-gray-600 border-gray-200 dark:border-gray-800'}"
            >🔓 Only Unlocked</button>
          </div>
        </div>
      </div>

      <!-- Col 2: Match Origin Tiers -->
      <div class="space-y-2 p-2 bg-white dark:bg-gray-950/70 rounded-lg border border-gray-200 dark:border-gray-800">
        <div class="text-[10px] font-bold uppercase tracking-wider text-gray-400">2. Match Origin</div>
        <div class="flex flex-col gap-1">
          {#each [
            { id: '100', label: '100% Exact Match', badge: '100%' },
            { id: 'fuzzy', label: 'Fuzzy Match (<100%)', badge: 'Fuzzy' },
            { id: 'frag', label: 'Fragment Match', badge: 'Frag' },
            { id: 'auto_propagate', label: 'Auto-propagated', badge: 'Auto' },
            { id: 'mt', label: 'Machine Translation', badge: 'MT' }
          ] as tier}
            <button
              type="button"
              on:click={() => toggleMatchTier(tier.id)}
              class="text-left px-2 py-1 rounded text-[11px] border flex items-center justify-between transition-colors cursor-pointer
                     {draft.matchTiers.includes(tier.id)
                       ? 'bg-brand-500 text-white border-brand-600 font-bold'
                       : 'bg-gray-50 text-gray-700 dark:bg-gray-900 dark:text-gray-300 border-gray-200 dark:border-gray-800 hover:bg-gray-100'}"
            >
              <span>{tier.label}</span>
              <span class="text-[9px] font-mono px-1 rounded bg-black/10">{tier.badge}</span>
            </button>
          {/each}
        </div>
      </div>

      <!-- Col 3: Structure Codes -->
      <div class="space-y-2 p-2 bg-white dark:bg-gray-950/70 rounded-lg border border-gray-200 dark:border-gray-800">
        <div class="text-[10px] font-bold uppercase tracking-wider text-gray-400">3. Document Structure</div>
        <div class="flex flex-col gap-1">
          {#each [
            { id: 'H1', label: 'H1 Headings', badge: 'H1' },
            { id: 'H2', label: 'H2 Headings', badge: 'H2' },
            { id: 'H3', label: 'H3 Headings', badge: 'H3' },
            { id: 'P', label: 'Paragraphs', badge: 'P' },
            { id: 'TC', label: 'Table Cells', badge: 'TC' },
            { id: 'LI', label: 'List Items', badge: 'LI' },
            { id: 'FN', label: 'Footnotes', badge: 'FN' }
          ] as s}
            <button
              type="button"
              on:click={() => toggleStructure(s.id)}
              class="text-left px-2 py-0.5 rounded text-[11px] border flex items-center justify-between transition-colors cursor-pointer
                     {draft.structureTypes.includes(s.id)
                       ? 'bg-brand-500 text-white border-brand-600 font-bold'
                       : 'bg-gray-50 text-gray-700 dark:bg-gray-900 dark:text-gray-300 border-gray-200 dark:border-gray-800 hover:bg-gray-100'}"
            >
              <span>{s.label}</span>
              <span class="text-[9px] font-mono px-1 rounded bg-black/10">{s.badge}</span>
            </button>
          {/each}
        </div>
      </div>

      <!-- Col 4: Regex & ISO 17100 QA Sampling -->
      <div class="space-y-2 p-2 bg-white dark:bg-gray-950/70 rounded-lg border border-gray-200 dark:border-gray-800 flex flex-col justify-between">
        <div>
          <div class="text-[10px] font-bold uppercase tracking-wider text-gray-400 mb-1">4. Regex Search</div>
          <input
            type="text"
            placeholder="Source Regex..."
            bind:value={draft.sourceRegex}
            class="w-full px-2 py-1 mb-1 rounded border border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900 font-mono text-[11px]"
          />
          <input
            type="text"
            placeholder="Target Regex..."
            bind:value={draft.targetRegex}
            class="w-full px-2 py-1 mb-1 rounded border border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900 font-mono text-[11px]"
          />
          <div class="flex items-center justify-between text-[11px] text-gray-500">
            <label class="flex items-center gap-1 cursor-pointer">
              <input type="checkbox" bind:checked={draft.caseSensitive} class="rounded accent-brand-500" />
              <span>Case Sensitive</span>
            </label>
            <label class="flex items-center gap-1 cursor-pointer font-bold text-amber-700 dark:text-amber-400">
              <input type="checkbox" bind:checked={draft.invertMatch} class="rounded accent-amber-500" />
              <span>Invert Match (NOT)</span>
            </label>
          </div>
        </div>

        <!-- ISO 17100 Audit Sampling -->
        <div class="pt-2 border-t border-gray-200/80 dark:border-gray-800">
          <div class="flex items-center justify-between text-[10px] font-bold uppercase text-gray-400 mb-0.5">
            <span>ISO 17100 QA Sample</span>
            <span class="text-brand-600 dark:text-brand-400 font-mono">{draft.samplePercent}%</span>
          </div>
          <input
            type="range"
            min="5"
            max="100"
            step="5"
            bind:value={draft.samplePercent}
            class="w-full accent-brand-500 cursor-pointer"
          />
          <div class="flex items-center justify-between text-[9px] text-gray-400">
            <span>5% (Quick)</span>
            <span>20% (Standard)</span>
            <span>100% (All)</span>
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}
