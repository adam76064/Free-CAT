<script lang="ts">
  /**
   * SegmentGrid — the main bilingual editor grid.
   * Handles TM lookup on selection change, auto-save, segment confirmation,
   * keyboard navigation, filter/search, and the split right panel.
   */
  import { tick } from 'svelte';
  import {
    segments, selectedIndex, filteredSegments, selectedSegment,
    filterMode, segmentSearch, updateSegmentInStore,
    progressStats, type FilterMode
  } from '$lib/stores/segments';
  import { tmMatches, tmLoading } from '$lib/stores/tm';
  import { currentProject, currentFile, projectSettings } from '$lib/stores/project';
  import { qaResults } from '$lib/stores/qa';
  import SegmentRow from './SegmentRow.svelte';
  import TmPanel from '$lib/components/panels/TmPanel.svelte';
  import QaPanel from '$lib/components/panels/QaPanel.svelte';
  import { queryTm, addToTm } from '$lib/ipc/tm';
  import { updateSegment, confirmSegment } from '$lib/ipc/segment';
  import { settings } from '$lib/stores/settings';
  import { runQa } from '$lib/ipc/project';

  // Auto-save debounce handle
  let autoSaveTimer: ReturnType<typeof setTimeout>;
  let gridContainer: HTMLDivElement;

  // TM lookup whenever selected segment changes
  $: if ($selectedSegment && $currentProject && $projectSettings.tm_ids.length > 0) {
    lookupTm($selectedSegment.source_text);
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

  function applyTmMatch(targetText: string) {
    const seg = $selectedSegment;
    if (!seg) return;
    updateSegmentInStore(seg.id, { target_text: targetText, status: 'draft' });
    scheduleAutoSave(seg.id, targetText, 'draft');
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
      await confirmSegment({
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

    <!-- Right panel -->
    <div class="w-80 shrink-0 border-l border-gray-200 dark:border-gray-800 flex flex-col bg-white dark:bg-gray-950">
      <div class="flex-[3] overflow-hidden border-b border-gray-200 dark:border-gray-800">
        <TmPanel onApply={applyTmMatch} />
      </div>
      <div class="flex-[2] overflow-hidden">
        <QaPanel onJumpToSegment={jumpToSegment} onRunQa={handleRunQa} />
      </div>
    </div>

  </div>
</div>
