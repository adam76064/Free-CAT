<script lang="ts">
  /**
   * SegmentGrid — the main bilingual editor grid.
   * Handles TM lookup & Termbase lookup on selection change, auto-save,
   * segment confirmation with auto-propagation, keyboard navigation,
   * filter/search, pre-translation, find & replace, split/merge, and tabbed right panel.
   */
  import { tick, onMount, onDestroy } from 'svelte';
  import {
    segments, selectedIndex, filteredSegments, selectedSegment,
    filterMode, segmentSearch, updateSegmentInStore,
    progressStats, type FilterMode, type Segment
  } from '$lib/stores/segments';
  import { tmMatches, tmLoading } from '$lib/stores/tm';
  import { termMatches, tbLoading } from '$lib/stores/termbase';
  import { currentProject, currentFile, projectSettings } from '$lib/stores/project';
  import { qaResults, qaErrors } from '$lib/stores/qa';
  import SegmentRow from './SegmentRow.svelte';
  import FindReplaceBar from './FindReplaceBar.svelte';
  import TmPanel from '$lib/components/panels/TmPanel.svelte';
  import TbPanel from '$lib/components/panels/TbPanel.svelte';
  import QaPanel from '$lib/components/panels/QaPanel.svelte';
  import AiPanel from '$lib/components/panels/AiPanel.svelte';
  import CommentsPanel from '$lib/components/panels/CommentsPanel.svelte';
  import LivePreviewPanel from '$lib/components/panels/LivePreviewPanel.svelte';
  import TermExtractionModal from '$lib/components/panels/TermExtractionModal.svelte';
  import QuoteInvoiceModal from '$lib/components/analysis/QuoteInvoiceModal.svelte';
  import CommandPalette from './CommandPalette.svelte';
  import { SuggestionTrie, type SuggestionItem } from '$lib/utils/suggestTrie';
  import { queryTm, addToTm } from '$lib/ipc/tm';
  import { queryTb } from '$lib/ipc/termbase';
  import { updateSegment, confirmSegment, getSegments, splitSegment, mergeSegments, toggleBookmark } from '$lib/ipc/segment';
  import { settings } from '$lib/stores/settings';
  import { runQa, preTranslate } from '$lib/ipc/project';

  // Auto-save debounce handle
  let autoSaveTimer: ReturnType<typeof setTimeout>;
  let gridContainer: HTMLDivElement;
  let activeTab: 'tm' | 'tb' | 'qa' | 'ai' | 'comments' | 'preview' = 'tm';
  let isInspectorOpen = true;
  let isCommandPaletteOpen = false;
  let isTermExtractionOpen = false;
  let isQuoteInvoiceOpen = false;
  let pretranslating = false;
  let isToolsMenuOpen = false;
  let activeAutoPatch: string | null = null;

  $: documentText = $segments.map(s => s.source_text).join('. ');

  // In-memory Trie for AutoSuggest
  const suggestTrie = new SuggestionTrie();
  let activeSuggestions: SuggestionItem[] = [];

  $: {
    suggestTrie.clear();
    for (const m of $termMatches) {
      suggestTrie.insert(m.target_term, {
        text: m.target_term,
        source: 'termbase',
        matchScore: 100,
        displayText: `${m.source_term} → ${m.target_term}`
      });
    }
    for (const tm of $tmMatches) {
      if (tm.target_text) {
        const words = tm.target_text.split(/\s+/).filter(w => w.length > 2);
        for (const w of words) {
          suggestTrie.insert(w, {
            text: w,
            source: 'tm',
            matchScore: tm.similarity,
          });
        }
      }
    }
  }

  function handleQuerySuggestions(word: string) {
    activeSuggestions = suggestTrie.searchPrefix(word, 5);
  }

  // Find & Replace state
  let isFindOpen = false;
  let isReplaceOpen = false;

  // Split Segment modal state
  let segmentToSplit: Segment | null = null;
  let splitSliderPos = 1;

  // Global keyboard shortcuts (Ctrl+F, Ctrl+H, Ctrl+\, Ctrl+K)
  function handleWindowKeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'f') {
      e.preventDefault();
      isFindOpen = true;
      isReplaceOpen = false;
    } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'h') {
      e.preventDefault();
      isFindOpen = true;
      isReplaceOpen = true;
    } else if ((e.ctrlKey || e.metaKey) && e.key === '\\') {
      e.preventDefault();
      isInspectorOpen = !isInspectorOpen;
    } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
      e.preventDefault();
      isCommandPaletteOpen = !isCommandPaletteOpen;
    }
  }

  onMount(() => {
    window.addEventListener('keydown', handleWindowKeydown);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleWindowKeydown);
  });

  // TM and TB lookup whenever selected segment changes
  $: if ($selectedSegment && $currentProject) {
    lookupTm($selectedSegment.source_text);
    if ($projectSettings.tb_ids && $projectSettings.tb_ids.length > 0) {
      lookupTb($selectedSegment.source_text);
    }
  }

  async function lookupTm(sourceText: string) {
    if (!$currentProject) return;
    activeAutoPatch = null;
    tmLoading.set(true);
    try {
      const matches = await queryTm({
        source_text: sourceText,
        src_lang: $currentProject.src_lang,
        tgt_lang: $currentProject.tgt_lang,
        tm_ids: $projectSettings.tm_ids || [],
        threshold: $projectSettings.fuzzy_threshold || 0.7,
      });
      tmMatches.set(matches);

      // Auto-detect FMR repaired suggestion from top match
      if (matches.length > 0 && matches[0].repaired_target && matches[0].repaired_target.trim()) {
        activeAutoPatch = matches[0].repaired_target;
      }
    } catch (e) {
      console.error('TM lookup failed:', e);
      tmMatches.set([]);
    } finally {
      tmLoading.set(false);
    }
  }

  function handleApplyAutoPatch(patchText: string) {
    const seg = $selectedSegment;
    if (!seg) return;
    updateSegmentInStore(seg.id, { target_text: patchText, status: 'draft' });
    scheduleAutoSave(seg.id, patchText, 'draft');
    activeAutoPatch = null;
  }

  async function handleAutoPatchAll() {
    if (!$currentProject || !$currentFile) return;
    let patchedCount = 0;
    for (const seg of $segments) {
      if (!seg.target_text.trim() && seg.status === 'new') {
        try {
          const matches = await queryTm({
            source_text: seg.source_text,
            src_lang: $currentProject.src_lang,
            tgt_lang: $currentProject.tgt_lang,
            tm_ids: $projectSettings.tm_ids || [],
            threshold: 0.7,
          });
          if (matches.length > 0 && matches[0].repaired_target) {
            updateSegmentInStore(seg.id, {
              target_text: matches[0].repaired_target,
              status: 'draft',
              match_percent: matches[0].score_pct,
              match_origin: 'tm_repaired',
            });
            await updateSegment({
              segment_id: seg.id,
              target_text: matches[0].repaired_target,
              status: 'draft',
            });
            patchedCount++;
          }
        } catch (e) {
          /* ignore */
        }
      }
    }
    alert(`Auto-patch complete: ${patchedCount} fuzzy segments repaired and filled!`);
  }

  async function lookupTb(sourceText: string) {
    if (!$currentProject) return;
    tbLoading.set(true);
    try {
      const matches = await queryTb({
        text: sourceText,
        lang: $currentProject.src_lang,
        tb_ids: $projectSettings.tb_ids,
      });
      termMatches.set(matches);
    } catch (e) {
      console.error('TB lookup failed:', e);
      termMatches.set([]);
    } finally {
      tbLoading.set(false);
    }
  }

  function applyTmMatch(targetText: string) {
    const seg = $selectedSegment;
    if (!seg) return;
    updateSegmentInStore(seg.id, { target_text: targetText, status: 'draft' });
    scheduleAutoSave(seg.id, targetText, 'draft');
  }

  function handleInsertTerm(termText: string) {
    const seg = $selectedSegment;
    if (!seg) return;
    const newText = seg.target_text ? `${seg.target_text} ${termText}` : termText;
    updateSegmentInStore(seg.id, { target_text: newText, status: 'draft' });
    scheduleAutoSave(seg.id, newText, 'draft');
  }

  function handleTargetChange(segId: string, text: string) {
    const status = text.trim() ? 'draft' : 'new';
    updateSegmentInStore(segId, { target_text: text, status });
    scheduleAutoSave(segId, text, status);
  }

  function scheduleAutoSave(segId: string, text: string, status: string) {
    clearTimeout(autoSaveTimer);
    autoSaveTimer = setTimeout(async () => {
      try {
        await updateSegment({ segment_id: segId, target_text: text, status });
      } catch (e) {
        console.error('Auto-save failed:', e);
      }
    }, $settings.autoSaveIntervalMs);
  }

  async function handleConfirm(seg: { id: string; source_text: string; target_text: string }) {
    if (!seg || !$currentProject) return;
    if (!seg.target_text.trim()) return;

    clearTimeout(autoSaveTimer);
    updateSegmentInStore(seg.id, { status: 'translated', target_text: seg.target_text });

    try {
      const propagated = await confirmSegment({
        segment_id: seg.id,
        target_text: seg.target_text,
        project_id: $currentProject.id,
        source_text: seg.source_text,
        src_lang: $currentProject.src_lang,
        tgt_lang: $currentProject.tgt_lang,
      });

      const targetTmIds = ($projectSettings.tm_ids && $projectSettings.tm_ids.length > 0)
        ? $projectSettings.tm_ids
        : [`proj-tm-${$currentProject.id}`];

      for (const tmId of targetTmIds) {
        await addToTm({
          tm_id: tmId,
          source_text: seg.source_text,
          target_text: seg.target_text,
          src_lang: $currentProject.src_lang,
          tgt_lang: $currentProject.tgt_lang,
        }).catch(() => {/* silent */});
      }

      // If auto-propagated to repeated segments, refresh file segments
      if (propagated > 0 && $currentFile) {
        const refreshed = await getSegments($currentFile.id, $currentProject.id);
        segments.set(refreshed);
      }
    } catch (e) {
      console.error('Confirm failed:', e);
    }

    selectedIndex.update(i => Math.min(i + 1, $filteredSegments.length - 1));
    await tick();
    scrollToSelected();
  }

  function handleGridKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIndex.update(i => Math.min(i + 1, $filteredSegments.length - 1));
      scrollToSelected();
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIndex.update(i => Math.max(i - 1, 0));
      scrollToSelected();
    }
  }

  function scrollToSelected() {
    if (!gridContainer) return;
    const rowHeight = 52;
    const top = $selectedIndex * rowHeight;
    const containerHeight = gridContainer.clientHeight;
    if (top < gridContainer.scrollTop || top + rowHeight > gridContainer.scrollTop + containerHeight) {
      gridContainer.scrollTop = top - containerHeight / 2;
    }
  }

  function jumpToSegment(segId: string) {
    const idx = $filteredSegments.findIndex(s => s.id === segId);
    if (idx !== -1) {
      selectedIndex.set(idx);
      scrollToSelected();
    }
  }

  // Split Segment Handlers
  function openSplitModal(segId: string) {
    const seg = $segments.find(s => s.id === segId);
    if (!seg || seg.source_text.length <= 1) return;
    segmentToSplit = seg;
    splitSliderPos = Math.max(1, Math.floor(seg.source_text.length / 2));
  }

  async function confirmSplit() {
    if (!segmentToSplit || !$currentFile || !$currentProject) return;
    try {
      await splitSegment({
        segment_id: segmentToSplit.id,
        split_index: splitSliderPos,
      });
      const reloaded = await getSegments($currentFile.id, $currentProject.id);
      segments.set(reloaded);
      segmentToSplit = null;
    } catch (e: any) {
      alert(`Split segment failed: ${e}`);
    }
  }

  // Merge Handlers
  async function handleMergeNext(segId: string) {
    if (!$currentFile || !$currentProject) return;
    const idx = $segments.findIndex(s => s.id === segId);
    if (idx === -1 || idx >= $segments.length - 1) return;
    const nextSeg = $segments[idx + 1];
    try {
      await mergeSegments({
        segment_id_1: segId,
        segment_id_2: nextSeg.id,
      });
      const reloaded = await getSegments($currentFile.id, $currentProject.id);
      segments.set(reloaded);
    } catch (e: any) {
      alert(`Merge failed: ${e}`);
    }
  }

  async function handleMergePrev(segId: string) {
    if (!$currentFile || !$currentProject) return;
    const idx = $segments.findIndex(s => s.id === segId);
    if (idx <= 0) return;
    const prevSeg = $segments[idx - 1];
    try {
      await mergeSegments({
        segment_id_1: prevSeg.id,
        segment_id_2: segId,
      });
      const reloaded = await getSegments($currentFile.id, $currentProject.id);
      segments.set(reloaded);
    } catch (e: any) {
      alert(`Merge failed: ${e}`);
    }
  }

  // Bookmark Handler
  async function handleToggleBookmark(segId: string) {
    try {
      const newState = await toggleBookmark(segId);
      updateSegmentInStore(segId, { bookmarked: newState });
    } catch (e) {
      console.error('Bookmark toggle failed:', e);
    }
  }

  async function handleRunQa() {
    if (!$currentProject || !$currentFile) return;
    try {
      const results = await runQa({
        project_id: $currentProject.id,
        file_ids: [$currentFile.id],
        tb_ids: $projectSettings.tb_ids,
      });
      qaResults.set(results);
    } catch (e) {
      console.error('QA failed:', e);
    }
  }

  async function handlePretranslate() {
    if (!$currentProject || !$currentFile) return;
    pretranslating = true;
    try {
      const res = await preTranslate({
        project_id: $currentProject.id,
        file_ids: [$currentFile.id],
        min_score: 1.0,
      });
      const reloaded = await getSegments($currentFile.id, $currentProject.id);
      segments.set(reloaded);
      alert(`Pre-translation complete: ${res.pretranslated} segments filled!`);
    } catch (e) {
      console.error('Pre-translate failed:', e);
    } finally {
      pretranslating = false;
    }
  }

  const FILTERS: { label: string; value: FilterMode }[] = [
    { label: 'All', value: 'all' },
    { label: 'New', value: 'new' },
    { label: 'Draft', value: 'draft' },
    { label: 'Translated', value: 'translated' },
    { label: 'Review', value: 'needs_review' },
    { label: '★', value: 'bookmarked' },
    { label: 'QA Issues', value: 'qa_warning' },
  ];
</script>

<div class="flex flex-col h-full relative">
  <!-- Studio Toolbar -->
  <div class="flex items-center justify-between px-3.5 py-2 border-b border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-950 shrink-0 gap-3">
    <!-- Left: Filter tabs + Search -->
    <div class="flex items-center gap-2 min-w-0">
      <!-- Filter tabs -->
      <div class="flex items-center gap-0.5 bg-gray-100 dark:bg-gray-800 rounded-lg p-0.5 shrink-0">
        {#each FILTERS as f}
          <button
            on:click={() => filterMode.set(f.value)}
            class="px-2.5 py-1 text-xs rounded-md font-medium transition-colors flex items-center gap-1.5
                   {$filterMode === f.value
                     ? 'bg-white dark:bg-gray-700 text-gray-900 dark:text-white shadow-xs'
                     : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'}"
          >
            <span>{f.label}</span>
            {#if f.value === 'qa_warning' && $qaResults.length > 0}
              <span class="px-1 py-0.2 rounded-full bg-red-500 text-white text-[10px] font-bold leading-none">
                {$qaResults.length}
              </span>
            {/if}
          </button>
        {/each}
      </div>

      <!-- Search input -->
      <div class="relative">
        <input
          bind:value={$segmentSearch}
          placeholder="Filter rows… (Ctrl+F)"
          class="pl-7 pr-2.5 py-1 text-xs rounded-lg border border-gray-200 dark:border-gray-700
                 bg-white dark:bg-gray-800 text-gray-900 dark:text-white w-40
                 focus:outline-none focus:ring-1 focus:ring-brand-500"
        />
        <span class="absolute left-2.5 top-1.5 text-xs text-gray-400">🔍</span>
      </div>
    </div>

    <!-- Right: Tools Dropdown + Actions + Inspector + Preview + Progress -->
    <div class="flex items-center gap-2 shrink-0">
      <!-- Tools Dropdown Menu -->
      <div class="relative">
        <button
          on:click={() => (isToolsMenuOpen = !isToolsMenuOpen)}
          class="px-2.5 py-1 text-xs font-semibold rounded-lg border border-gray-200 dark:border-gray-700
                 {isToolsMenuOpen ? 'bg-brand-50 dark:bg-brand-900/30 text-brand-600 border-brand-300' : 'text-gray-700 dark:text-gray-300'}
                 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors flex items-center gap-1.5 shadow-xs"
          title="Project Automation & Analysis Tools"
        >
          <span>⚡ Tools</span>
          <span class="text-[10px] text-gray-400">▾</span>
        </button>

        {#if isToolsMenuOpen}
          <!-- Backdrop to close -->
          <!-- svelte-ignore a11y-click-events-have-key-events -->
          <!-- svelte-ignore a11y-no-static-element-interactions -->
          <div class="fixed inset-0 z-40" on:click={() => (isToolsMenuOpen = false)}></div>

          <div class="absolute right-0 mt-1.5 w-60 rounded-xl border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 shadow-xl py-1 z-50 text-xs">
            <button
              on:click={() => { isToolsMenuOpen = false; handlePretranslate(); }}
              class="w-full text-left px-3 py-2 hover:bg-gray-50 dark:hover:bg-gray-800 text-gray-700 dark:text-gray-200 flex items-center gap-2.5"
            >
              <span class="text-amber-500 text-base">⚡</span>
              <div>
                <div class="font-medium">Pre-translate 100%</div>
                <div class="text-[10px] text-gray-400">Auto-fill exact TM matches</div>
              </div>
            </button>

            <button
              on:click={() => { isToolsMenuOpen = false; handleAutoPatchAll(); }}
              class="w-full text-left px-3 py-2 hover:bg-gray-50 dark:hover:bg-gray-800 text-gray-700 dark:text-gray-200 flex items-center gap-2.5"
            >
              <span class="text-blue-500 text-base">🔧</span>
              <div>
                <div class="font-medium">Auto-Patch Fuzzy Matches</div>
                <div class="text-[10px] text-gray-400">Batch repair numbers & terms (FMR)</div>
              </div>
            </button>

            <div class="my-1 border-t border-gray-100 dark:border-gray-800"></div>

            <button
              on:click={() => { isToolsMenuOpen = false; isFindOpen = true; isReplaceOpen = false; }}
              class="w-full text-left px-3 py-2 hover:bg-gray-50 dark:hover:bg-gray-800 text-gray-700 dark:text-gray-200 flex items-center justify-between"
            >
              <span class="flex items-center gap-2.5"><span>🔍</span> Find & Replace</span>
              <kbd class="text-[10px] font-mono text-gray-400">Ctrl+H</kbd>
            </button>

            <button
              on:click={() => { isToolsMenuOpen = false; isTermExtractionOpen = true; }}
              class="w-full text-left px-3 py-2 hover:bg-gray-50 dark:hover:bg-gray-800 text-gray-700 dark:text-gray-200 flex items-center gap-2.5"
            >
              <span class="text-purple-500 text-base">🏷️</span>
              <div>
                <div class="font-medium">Extract Terminology (ATE)</div>
                <div class="text-[10px] text-gray-400">C-value candidate extraction</div>
              </div>
            </button>

            <button
              on:click={() => { isToolsMenuOpen = false; isQuoteInvoiceOpen = true; }}
              class="w-full text-left px-3 py-2 hover:bg-gray-50 dark:hover:bg-gray-800 text-gray-700 dark:text-gray-200 flex items-center gap-2.5"
            >
              <span class="text-emerald-500 text-base">💼</span>
              <div>
                <div class="font-medium">Quoting & Invoicing</div>
                <div class="text-[10px] text-gray-400">Weighted word count & billing</div>
              </div>
            </button>
          </div>
        {/if}
      </div>

      <!-- Actions / Command Palette button -->
      <button
        on:click={() => (isCommandPaletteOpen = true)}
        class="px-2.5 py-1 text-xs font-medium rounded-lg border border-gray-200 dark:border-gray-700
               text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors flex items-center gap-1.5"
        title="Omni Command Palette (Ctrl+K)"
      >
        <span>⚡ Actions</span>
        <kbd class="text-[9px] font-mono bg-gray-100 dark:bg-gray-800 text-gray-500 px-1 rounded border border-gray-200 dark:border-gray-700">Ctrl+K</kbd>
      </button>

      <!-- Toggle Inspector button -->
      <button
        on:click={() => (isInspectorOpen = !isInspectorOpen)}
        class="px-2.5 py-1 text-xs font-medium rounded-lg border border-gray-200 dark:border-gray-700
               {isInspectorOpen ? 'bg-gray-100 dark:bg-gray-800 text-brand-600' : 'text-gray-700 dark:text-gray-300'}
               hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors flex items-center gap-1"
        title="Toggle Right Inspector (Ctrl+\)"
      >
        <span>⮌ Inspector</span>
      </button>

      <!-- Live Preview button -->
      <button
        on:click={() => {
          activeTab = 'preview';
          isInspectorOpen = true;
        }}
        class="px-2.5 py-1 text-xs font-medium rounded-lg border border-gray-200 dark:border-gray-700
               {isInspectorOpen && activeTab === 'preview' ? 'bg-brand-50 dark:bg-brand-900/30 text-brand-600 border-brand-300' : 'text-gray-700 dark:text-gray-300'}
               hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors flex items-center gap-1"
        title="Live In-Context Document Flow Preview"
      >
        <span>👁 Preview</span>
      </button>

      <!-- Progress bar -->
      <div class="flex items-center gap-1.5 pl-2 border-l border-gray-200 dark:border-gray-800">
        <div class="w-16 h-1.5 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
          <div class="h-full bg-emerald-500 rounded-full transition-all" style="width: {$progressStats.pct}%"></div>
        </div>
        <span class="text-[11px] text-gray-500 dark:text-gray-400 tabular-nums font-mono">
          {$progressStats.pct}%
        </span>
      </div>
    </div>
  </div>

  <!-- Find & Replace Floating Toolbar -->
  {#if isFindOpen}
    <FindReplaceBar
      segments={$filteredSegments}
      activeSegmentIndex={$selectedIndex}
      {isReplaceOpen}
      onSelectSegment={(idx) => {
        selectedIndex.set(idx);
        scrollToSelected();
      }}
      onUpdateSegment={(segId, newTarget) => {
        handleTargetChange(segId, newTarget);
      }}
      onClose={() => {
        isFindOpen = false;
        isReplaceOpen = false;
      }}
    />
  {/if}

  <!-- Grid + right panel -->
  <div class="flex flex-1 overflow-hidden">

    <!-- Segment grid -->
    <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
    <div
      bind:this={gridContainer}
      class="flex-1 overflow-y-auto"
      on:keydown={handleGridKeydown}
      tabindex="-1"
      role="grid"
      aria-label="Translation editor"
    >
      <!-- Column headers -->
      <div class="grid grid-cols-[2.5rem_1fr_1fr] sticky top-0 z-10
                  bg-gray-50 dark:bg-gray-900 border-b-2 border-gray-200 dark:border-gray-800">
        <div class="py-1.5 px-1 text-xs text-gray-400 text-center">#</div>
        <div class="py-1.5 px-3 text-xs font-semibold text-gray-600 dark:text-gray-400 border-r border-gray-200 dark:border-gray-700">
          Source ({$currentProject?.src_lang ?? ''})
        </div>
        <div class="py-1.5 px-3 text-xs font-semibold text-gray-600 dark:text-gray-400">
          Target ({$currentProject?.tgt_lang ?? ''})
        </div>
      </div>

      <!-- Segment rows -->
      {#if $filteredSegments.length === 0}
        <div class="flex items-center justify-center h-48 text-gray-400 text-sm">
          No segments match the filter
        </div>
      {:else}
        {#each $filteredSegments as segment, i (segment.id)}
          <SegmentRow
            {segment}
            isSelected={i === $selectedIndex}
            canMergePrev={i > 0}
            canMergeNext={i < $filteredSegments.length - 1}
            activeSuggestions={activeSuggestions}
            autoPatchSuggestion={i === $selectedIndex ? activeAutoPatch : null}
            onApplyAutoPatch={handleApplyAutoPatch}
            onQuerySuggestions={handleQuerySuggestions}
            onSelect={() => selectedIndex.set(i)}
            onConfirm={handleConfirm}
            onTargetChange={handleTargetChange}
            onSplit={openSplitModal}
            onMergeNext={handleMergeNext}
            onMergePrev={handleMergePrev}
            onToggleBookmark={handleToggleBookmark}
          />
        {/each}
      {/if}
    </div>

    <!-- Right panel with tabs -->
    {#if isInspectorOpen}
      <div class="w-80 shrink-0 border-l border-gray-200 dark:border-gray-800 flex flex-col bg-white dark:bg-gray-950">
        <!-- Tab headers -->
        <div class="flex items-center border-b border-gray-200 dark:border-gray-800 bg-gray-50 dark:bg-gray-900 px-1 py-1 gap-1 shrink-0 overflow-x-auto">
          <button
            on:click={() => (activeTab = 'tm')}
            class="flex-1 py-1 px-1.5 text-[11px] font-medium rounded-md text-center transition-colors
                   {activeTab === 'tm'
                     ? 'bg-white dark:bg-gray-800 text-brand-600 dark:text-brand-400 shadow-xs'
                     : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'}"
          >
            🧠 TM
          </button>
          <button
            on:click={() => (activeTab = 'tb')}
            class="flex-1 py-1 px-1.5 text-[11px] font-medium rounded-md text-center transition-colors flex items-center justify-center gap-1
                   {activeTab === 'tb'
                     ? 'bg-white dark:bg-gray-800 text-brand-600 dark:text-brand-400 shadow-xs'
                     : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'}"
          >
            <span>📖 Term</span>
            {#if $termMatches.length > 0}
              <span class="w-3.5 h-3.5 rounded-full bg-brand-500 text-white text-[9px] flex items-center justify-center">
                {$termMatches.length}
              </span>
            {/if}
          </button>
          <button
            on:click={() => (activeTab = 'qa')}
            class="flex-1 py-1 px-1.5 text-[11px] font-medium rounded-md text-center transition-colors flex items-center justify-center gap-1
                   {activeTab === 'qa'
                     ? 'bg-white dark:bg-gray-800 text-brand-600 dark:text-brand-400 shadow-xs'
                     : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'}"
          >
            <span>✓ QA</span>
            {#if $qaErrors.length > 0}
              <span class="w-3.5 h-3.5 rounded-full bg-red-500 text-white text-[9px] flex items-center justify-center">
                {$qaErrors.length}
              </span>
            {/if}
          </button>
          <button
            on:click={() => (activeTab = 'ai')}
            class="flex-1 py-1 px-1.5 text-[11px] font-medium rounded-md text-center transition-colors flex items-center justify-center gap-1
                   {activeTab === 'ai'
                     ? 'bg-white dark:bg-gray-800 text-purple-600 dark:text-purple-400 shadow-xs'
                     : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'}"
          >
            <span>✨ AI</span>
          </button>
          <button
            on:click={() => (activeTab = 'comments')}
            class="flex-1 py-1 px-1.5 text-[11px] font-medium rounded-md text-center transition-colors flex items-center justify-center gap-1
                   {activeTab === 'comments'
                     ? 'bg-white dark:bg-gray-800 text-blue-600 dark:text-blue-400 shadow-xs'
                     : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'}"
          >
            <span>💬 Notes</span>
            {#if $selectedSegment?.comment}
              <span class="w-2 h-2 rounded-full bg-blue-500"></span>
            {/if}
          </button>
          <button
            on:click={() => (activeTab = 'preview')}
            class="flex-1 py-1 px-1.5 text-[11px] font-medium rounded-md text-center transition-colors flex items-center justify-center gap-1
                   {activeTab === 'preview'
                     ? 'bg-white dark:bg-gray-800 text-emerald-600 dark:text-emerald-400 shadow-xs'
                     : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'}"
          >
            <span>👁 Flow</span>
          </button>
        </div>

        <!-- Tab content -->
        <div class="flex-1 overflow-hidden">
          {#if activeTab === 'tm'}
            <TmPanel onApply={applyTmMatch} />
          {:else if activeTab === 'tb'}
            <TbPanel onInsertTerm={handleInsertTerm} />
          {:else if activeTab === 'qa'}
            <QaPanel onJumpToSegment={jumpToSegment} onRunQa={handleRunQa} />
          {:else if activeTab === 'ai'}
            <AiPanel onApplyTarget={(txt) => {
              const seg = $selectedSegment;
              if (seg) handleTargetChange(seg.id, txt);
            }} />
          {:else if activeTab === 'comments'}
            <CommentsPanel />
          {:else if activeTab === 'preview'}
            <LivePreviewPanel onSelectSegment={(idx) => jumpToSegment(idx)} />
          {/if}
        </div>
      </div>
    {/if}

  </div>
</div>

<CommandPalette
  isOpen={isCommandPaletteOpen}
  onToggleInspector={() => (isInspectorOpen = !isInspectorOpen)}
  onTogglePreview={() => {
    activeTab = 'preview';
    isInspectorOpen = true;
  }}
  onOpenFind={() => { isFindOpen = true; isReplaceOpen = false; }}
  onOpenReplace={() => { isFindOpen = true; isReplaceOpen = true; }}
  onRunQa={handleRunQa}
  onExtractTerms={() => (isTermExtractionOpen = true)}
  onOpenInvoice={() => (isQuoteInvoiceOpen = true)}
  onSplit={() => { if ($selectedSegment) openSplitModal($selectedSegment.id); }}
  onCopySource={() => { if ($selectedSegment) handleTargetChange($selectedSegment.id, $selectedSegment.source_text); }}
  onClearTarget={() => { if ($selectedSegment) handleTargetChange($selectedSegment.id, ''); }}
/>

<TermExtractionModal
  isOpen={isTermExtractionOpen}
  onClose={() => (isTermExtractionOpen = false)}
  documentText={documentText}
/>

<QuoteInvoiceModal
  isOpen={isQuoteInvoiceOpen}
  onClose={() => (isQuoteInvoiceOpen = false)}
  projectId={$currentProject?.id || ''}
  fileIds={$currentFile?.id ? [$currentFile.id] : []}
/>

<!-- Split Segment Modal -->
{#if segmentToSplit}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4">
    <div class="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 shadow-2xl max-w-lg w-full p-6 animate-in fade-in zoom-in-95">
      <h3 class="text-base font-semibold text-gray-900 dark:text-white mb-1">
        Split Segment #{segmentToSplit.seq_number}
      </h3>
      <p class="text-xs text-gray-500 dark:text-gray-400 mb-4">
        Move the slider to choose the split boundary between Part 1 and Part 2.
      </p>

      <!-- Live split preview text -->
      <div class="p-3 bg-gray-50 dark:bg-gray-800/80 rounded-lg text-sm mb-4 font-editor leading-relaxed border border-gray-200 dark:border-gray-700">
        <span class="text-blue-700 dark:text-blue-300 bg-blue-50 dark:bg-blue-900/40 rounded px-1">
          {segmentToSplit.source_text.slice(0, splitSliderPos)}
        </span>
        <span class="text-red-500 font-bold px-0.5">|</span>
        <span class="text-emerald-700 dark:text-emerald-300 bg-emerald-50 dark:bg-emerald-900/40 rounded px-1">
          {segmentToSplit.source_text.slice(splitSliderPos)}
        </span>
      </div>

      <input
        type="range"
        min="1"
        max={Math.max(1, segmentToSplit.source_text.length - 1)}
        bind:value={splitSliderPos}
        class="w-full accent-brand-500 mb-4 cursor-pointer"
      />

      <div class="grid grid-cols-2 gap-3 mb-6 text-xs">
        <div class="p-2.5 rounded bg-blue-50/50 dark:bg-blue-950/20 border border-blue-200 dark:border-blue-900">
          <div class="font-semibold text-blue-700 dark:text-blue-400 mb-1">Segment {segmentToSplit.seq_number} (Part 1)</div>
          <div class="text-gray-800 dark:text-gray-200 italic line-clamp-3">{segmentToSplit.source_text.slice(0, splitSliderPos)}</div>
        </div>
        <div class="p-2.5 rounded bg-emerald-50/50 dark:bg-emerald-950/20 border border-emerald-200 dark:border-emerald-900">
          <div class="font-semibold text-emerald-700 dark:text-emerald-400 mb-1">Segment {segmentToSplit.seq_number + 1} (Part 2)</div>
          <div class="text-gray-800 dark:text-gray-200 italic line-clamp-3">{segmentToSplit.source_text.slice(splitSliderPos)}</div>
        </div>
      </div>

      <div class="flex items-center justify-end gap-2">
        <button
          on:click={() => segmentToSplit = null}
          class="px-3 py-1.5 text-xs text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg transition-colors"
        >
          Cancel
        </button>
        <button
          on:click={confirmSplit}
          class="px-4 py-1.5 text-xs font-medium text-white bg-brand-500 hover:bg-brand-600 rounded-lg shadow-sm transition-colors"
        >
          Split Segment
        </button>
      </div>
    </div>
  </div>
{/if}
