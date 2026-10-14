<script lang="ts">
  /**
   * SegmentGrid — the main bilingual editor grid.
   * Handles TM lookup & Termbase lookup on selection change, auto-save,
   * segment confirmation with auto-propagation, keyboard navigation,
   * filter/search, pre-translation, and the tabbed right panel.
   */
  import { tick } from 'svelte';
  import {
    segments, selectedIndex, filteredSegments, selectedSegment,
    filterMode, segmentSearch, updateSegmentInStore,
    progressStats, type FilterMode
  } from '$lib/stores/segments';
  import { tmMatches, tmLoading } from '$lib/stores/tm';
  import { termMatches, tbLoading } from '$lib/stores/termbase';
  import { currentProject, currentFile, projectSettings } from '$lib/stores/project';
  import { qaResults, qaErrors } from '$lib/stores/qa';
  import SegmentRow from './SegmentRow.svelte';
  import TmPanel from '$lib/components/panels/TmPanel.svelte';
  import TbPanel from '$lib/components/panels/TbPanel.svelte';
  import QaPanel from '$lib/components/panels/QaPanel.svelte';
  import { queryTm, addToTm } from '$lib/ipc/tm';
  import { queryTb } from '$lib/ipc/termbase';
  import { updateSegment, confirmSegment, getSegments } from '$lib/ipc/segment';
  import { settings } from '$lib/stores/settings';
  import { runQa, preTranslate } from '$lib/ipc/project';

  // Auto-save debounce handle
  let autoSaveTimer: ReturnType<typeof setTimeout>;
  let gridContainer: HTMLDivElement;
  let activeTab: 'tm' | 'tb' | 'qa' = 'tm';
  let pretranslating = false;

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
  ];
</script>

<div class="flex flex-col h-full">
  <!-- Toolbar -->
  <div class="flex items-center gap-2 px-3 py-2 border-b border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-950 shrink-0">
    <!-- Filter tabs -->
    <div class="flex items-center gap-0.5 bg-gray-100 dark:bg-gray-800 rounded-lg p-0.5">
      {#each FILTERS as f}
        <button
          on:click={() => filterMode.set(f.value)}
          class="px-2.5 py-1 text-xs rounded-md font-medium transition-colors
                 {$filterMode === f.value
                   ? 'bg-white dark:bg-gray-700 text-gray-900 dark:text-white shadow-sm'
                   : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'}"
        >
          {f.label}
        </button>
      {/each}
    </div>

    <!-- Search -->
    <input
      bind:value={$segmentSearch}
      placeholder="Search segments…"
      class="px-2.5 py-1 text-xs rounded-lg border border-gray-200 dark:border-gray-700
             bg-white dark:bg-gray-800 text-gray-900 dark:text-white w-48
             focus:outline-none focus:ring-1 focus:ring-brand-500"
    />

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

    <div class="ml-auto flex items-center gap-2">
      <!-- Progress bar -->
      <div class="flex items-center gap-1.5">
        <div class="w-24 h-1.5 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
          <div class="h-full bg-green-500 rounded-full transition-all" style="width: {$progressStats.pct}%"></div>
        </div>
        <span class="text-xs text-gray-500 dark:text-gray-400">
          {$progressStats.translated}/{$progressStats.total}
        </span>
      </div>
    </div>
  </div>

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
            onSelect={() => selectedIndex.set(i)}
            onConfirm={handleConfirm}
            onTargetChange={handleTargetChange}
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
          🧠 TM
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
      </div>

      <!-- Tab content -->
      <div class="flex-1 overflow-hidden">
        {#if activeTab === 'tm'}
          <TmPanel onApply={applyTmMatch} />
        {:else if activeTab === 'tb'}
          <TbPanel onInsertTerm={handleInsertTerm} />
        {:else if activeTab === 'qa'}
          <QaPanel onJumpToSegment={jumpToSegment} onRunQa={handleRunQa} />
        {/if}
      </div>
    </div>

  </div>
</div>
