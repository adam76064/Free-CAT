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
  import { queryTm, addToTm } from '$lib/ipc/tm';
  import { queryTb } from '$lib/ipc/termbase';
  import { updateSegment, confirmSegment, getSegments, splitSegment, mergeSegments, toggleBookmark } from '$lib/ipc/segment';
  import { settings } from '$lib/stores/settings';
  import { runQa, preTranslate } from '$lib/ipc/project';

  // Auto-save debounce handle
  let autoSaveTimer: ReturnType<typeof setTimeout>;
  let gridContainer: HTMLDivElement;
  let activeTab: 'tm' | 'tb' | 'qa' | 'ai' = 'tm';
  let pretranslating = false;

  // Find & Replace state
  let isFindOpen = false;
  let isReplaceOpen = false;

  // Split Segment modal state
  let segmentToSplit: Segment | null = null;
  let splitSliderPos = 1;

  // Global keyboard shortcuts (Ctrl+F, Ctrl+H)
  function handleWindowKeydown(e: KeyboardEvent) {
    if (e.ctrlKey && e.key.toLowerCase() === 'f') {
      e.preventDefault();
      isFindOpen = true;
      isReplaceOpen = false;
    } else if (e.ctrlKey && e.key.toLowerCase() === 'h') {
      e.preventDefault();
      isFindOpen = true;
      isReplaceOpen = true;
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
    if ($projectSettings.tm_ids.length > 0) {
      lookupTm($selectedSegment.source_text);
    }
    if ($projectSettings.tb_ids.length > 0) {
      lookupTb($selectedSegment.source_text);
    }
  }

  async function lookupTm(sourceText: string) {
    if (!$currentProject) return;
    tmLoading.set(true);
    try {
      const matches = await queryTm({
        source_text: sourceText,
        src_lang: $currentProject.src_lang,
        tgt_lang: $currentProject.tgt_lang,
        tm_ids: $projectSettings.tm_ids,
        threshold: $projectSettings.fuzzy_threshold,
      });
      tmMatches.set(matches);
    } catch (e) {
      console.error('TM lookup failed:', e);
      tmMatches.set([]);
    } finally {
      tmLoading.set(false);
    }
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

      if ($projectSettings.writeback_mode === 'always' && $projectSettings.tm_ids.length > 0) {
        for (const tmId of $projectSettings.tm_ids) {
          await addToTm({
            tm_id: tmId,
            source_text: seg.source_text,
            target_text: seg.target_text,
            src_lang: $currentProject.src_lang,
            tgt_lang: $currentProject.tgt_lang,
          }).catch(() => {/* silent */});
        }
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
  <!-- Toolbar -->
  <div class="flex items-center gap-2 px-3 py-2 border-b border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-950 shrink-0">
    <!-- Filter tabs -->
    <div class="flex items-center gap-0.5 bg-gray-100 dark:bg-gray-800 rounded-lg p-0.5">
      {#each FILTERS as f}
        <button
          on:click={() => filterMode.set(f.value)}
          class="px-2.5 py-1 text-xs rounded-md font-medium transition-colors flex items-center gap-1.5
                 {$filterMode === f.value
                   ? 'bg-white dark:bg-gray-700 text-gray-900 dark:text-white shadow-sm'
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

    <!-- Find button / Search filter -->
    <input
      bind:value={$segmentSearch}
      placeholder="Filter rows…"
      class="px-2.5 py-1 text-xs rounded-lg border border-gray-200 dark:border-gray-700
             bg-white dark:bg-gray-800 text-gray-900 dark:text-white w-36
             focus:outline-none focus:ring-1 focus:ring-brand-500"
    />

    <button
      on:click={() => { isFindOpen = !isFindOpen; if (!isFindOpen) isReplaceOpen = false; }}
      class="px-2.5 py-1 text-xs font-medium rounded-lg border border-gray-200 dark:border-gray-700
             {isFindOpen ? 'bg-brand-50 dark:bg-brand-900/30 text-brand-600 border-brand-300' : 'text-gray-700 dark:text-gray-300'}
             hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors"
      title="Find & Replace (Ctrl+F / Ctrl+H)"
    >
      🔍 Find / Replace
    </button>

    <!-- Pre-translate button -->
    <button
      on:click={handlePretranslate}
      disabled={pretranslating}
      class="px-2.5 py-1 text-xs font-medium rounded-lg border border-gray-200 dark:border-gray-700
             text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors disabled:opacity-50"
      title="Fill 100% exact matches from TM automatically"
    >
      {pretranslating ? '⚡ Pretranslating…' : '⚡ Pre-translate'}
    </button>

    <div class="ml-auto flex items-center gap-3">
      <!-- Progress bar -->
      <div class="flex items-center gap-1.5">
        <div class="w-24 h-1.5 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
          <div class="h-full bg-green-500 rounded-full transition-all" style="width: {$progressStats.pct}%"></div>
        </div>
        <span class="text-xs text-gray-500 dark:text-gray-400 tabular-nums">
          {$progressStats.pct}% ({$progressStats.translated}/{$progressStats.total})
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
            index={i}
            isSelected={i === $selectedIndex}
            canMergePrev={i > 0}
            canMergeNext={i < $filteredSegments.length - 1}
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
    <div class="w-80 shrink-0 border-l border-gray-200 dark:border-gray-800 flex flex-col bg-white dark:bg-gray-950">
      <!-- Tab headers -->
      <div class="flex items-center border-b border-gray-200 dark:border-gray-800 bg-gray-50 dark:bg-gray-900 px-1 py-1 gap-1 shrink-0">
        <button
          on:click={() => (activeTab = 'tm')}
          class="flex-1 py-1 px-2 text-xs font-medium rounded-md text-center transition-colors
                 {activeTab === 'tm'
                   ? 'bg-white dark:bg-gray-800 text-brand-600 dark:text-brand-400 shadow-sm'
                   : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'}"
        >
          🧠 TM / MT
        </button>
        <button
          on:click={() => (activeTab = 'tb')}
          class="flex-1 py-1 px-2 text-xs font-medium rounded-md text-center transition-colors flex items-center justify-center gap-1
                 {activeTab === 'tb'
                   ? 'bg-white dark:bg-gray-800 text-brand-600 dark:text-brand-400 shadow-sm'
                   : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'}"
        >
          <span>📖 Glossary</span>
          {#if $termMatches.length > 0}
            <span class="w-4 h-4 rounded-full bg-brand-500 text-white text-[10px] flex items-center justify-center">
              {$termMatches.length}
            </span>
          {/if}
        </button>
        <button
          on:click={() => (activeTab = 'qa')}
          class="flex-1 py-1 px-2 text-xs font-medium rounded-md text-center transition-colors flex items-center justify-center gap-1
                 {activeTab === 'qa'
                   ? 'bg-white dark:bg-gray-800 text-brand-600 dark:text-brand-400 shadow-sm'
                   : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'}"
        >
          <span>✓ QA</span>
          {#if $qaErrors.length > 0}
            <span class="w-4 h-4 rounded-full bg-red-500 text-white text-[10px] flex items-center justify-center">
              {$qaErrors.length}
            </span>
          {/if}
        </button>
        <button
          on:click={() => (activeTab = 'ai')}
          class="flex-1 py-1 px-2 text-xs font-medium rounded-md text-center transition-colors flex items-center justify-center gap-1
                 {activeTab === 'ai'
                   ? 'bg-white dark:bg-gray-800 text-purple-600 dark:text-purple-400 shadow-sm'
                   : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'}"
        >
          <span>✨ AI</span>
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
        {/if}
      </div>
    </div>

  </div>
</div>

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
