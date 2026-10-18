<script lang="ts">
  /**
   * StudioResultsPane.svelte — Trados Upper Split Pane
   * Displays:
   * Left 60%: Translation Results (TM) with match scores, colored source diffs, metadata & 1-click apply
   * Right 40%: Term Recognition (TB) with recognized glossary hits, definitions & 1-click insert
   */
  import { createEventDispatcher } from 'svelte';
  import { tmMatches, tmLoading } from '$lib/stores/tm';
  import { termMatches, tbLoading } from '$lib/stores/termbase';
  import { selectedSegment } from '$lib/stores/segments';
  import { currentProject } from '$lib/stores/project';
  import TmMatchBadge from '$lib/components/tm/TmMatchBadge.svelte';

  export let isOpen = true;
  export let activeAutoPatch: string | null = null;
  export let height = 150;

  const dispatch = createEventDispatcher<{
    applyTm: string;
    applyAutoPatch: string;
    insertTerm: string;
    openConcordance: void;
    openTermManager: void;
    close: void;
  }>();

  let leftTab: 'tm' | 'concordance' = 'tm';
  let isResizing = false;
  let startY = 0;
  let startHeight = height;

  $: topMatch = $tmMatches.length > 0 ? $tmMatches[0] : null;

  function handleStartResize(e: PointerEvent) {
    isResizing = true;
    startY = e.clientY;
    startHeight = height;
    window.addEventListener('pointermove', handlePointerMove);
    window.addEventListener('pointerup', handlePointerUp);
  }

  function handlePointerMove(e: PointerEvent) {
    if (!isResizing) return;
    const delta = e.clientY - startY;
    height = Math.max(90, Math.min(360, startHeight + delta));
  }

  function handlePointerUp() {
    isResizing = false;
    window.removeEventListener('pointermove', handlePointerMove);
    window.removeEventListener('pointerup', handlePointerUp);
  }

  /** Compute simple word-level diff for translation results */
  function computeWordDiff(sourceCurrent: string, sourceMatch: string): { type: 'same' | 'del' | 'add', text: string }[] {
    const curWords = sourceCurrent.trim().split(/\s+/);
    const matchWords = sourceMatch.trim().split(/\s+/);
    const diffs: { type: 'same' | 'del' | 'add', text: string }[] = [];

    // Simple diff display
    let i = 0;
    let j = 0;
    while (i < matchWords.length || j < curWords.length) {
      if (i < matchWords.length && j < curWords.length && matchWords[i].toLowerCase() === curWords[j].toLowerCase()) {
        diffs.push({ type: 'same', text: curWords[j] + ' ' });
        i++;
        j++;
      } else if (i < matchWords.length && (!curWords.includes(matchWords[i]))) {
        diffs.push({ type: 'del', text: matchWords[i] + ' ' });
        i++;
      } else if (j < curWords.length) {
        diffs.push({ type: 'add', text: curWords[j] + ' ' });
        j++;
      } else {
        if (i < matchWords.length) {
          diffs.push({ type: 'del', text: matchWords[i] + ' ' });
          i++;
        }
        if (j < curWords.length) {
          diffs.push({ type: 'add', text: curWords[j] + ' ' });
          j++;
        }
      }
    }
    return diffs;
  }
</script>

{#if isOpen}
  <div
    class="border-b-2 border-gray-200 dark:border-gray-800 bg-gray-50/95 dark:bg-gray-900/95 flex flex-col relative shrink-0 select-none shadow-xs"
    style="height: {height}px;"
  >
    <!-- Split Panes: Translation Results (Left 60%) + Term Recognition (Right 40%) -->
    <div class="flex-1 flex overflow-hidden divide-x divide-gray-200 dark:divide-gray-800 text-xs">
      
      <!-- LEFT 60%: Translation Results (TM) -->
      <div class="w-[60%] flex flex-col min-w-0 bg-white dark:bg-gray-900">
        <!-- Header -->
        <div class="h-6 px-3 border-b border-gray-200 dark:border-gray-800 flex items-center justify-between bg-gray-100/70 dark:bg-gray-950/60 shrink-0">
          <div class="flex items-center gap-2">
            <span class="font-bold text-gray-800 dark:text-gray-200 text-[11px] flex items-center gap-1">
              <span>🧠</span> Translation Results
            </span>
            {#if topMatch}
              <TmMatchBadge pct={topMatch.score_pct} />
              {#if topMatch.score_pct >= 100}
                <span class="text-[10px] font-bold text-emerald-600 dark:text-emerald-400">Exact Match</span>
              {:else if topMatch.score_pct >= 70}
                <span class="text-[10px] font-bold text-amber-600 dark:text-amber-400">Fuzzy Match</span>
              {/if}
            {/if}
          </div>

          <div class="flex items-center gap-2">
            {#if $tmMatches.length > 1}
              <span class="text-[10px] text-gray-400">Match 1 of {$tmMatches.length}</span>
            {/if}
            {#if topMatch}
              <button
                type="button"
                on:click={() => dispatch('applyTm', topMatch.target_text)}
                class="px-2 py-0.5 rounded bg-brand-500 hover:bg-brand-600 text-white text-[10px] font-bold transition-colors shadow-2xs cursor-pointer flex items-center gap-1"
                title="Apply TM Match to Target (Ctrl+T)"
              >
                <span>Apply</span>
                <kbd class="text-[8px] bg-brand-600 px-1 rounded font-mono">Ctrl+T</kbd>
              </button>
            {/if}
          </div>
        </div>

        <!-- Content Area -->
        <div class="flex-1 p-2.5 overflow-y-auto space-y-1.5 font-editor leading-relaxed text-xs">
          {#if $tmLoading}
            <div class="flex items-center justify-center h-full text-gray-400">
              Querying project translation memories…
            </div>
          {:else if !topMatch}
            <div class="flex items-center justify-center h-full text-gray-400 italic">
              No matching translation memory unit found for active segment.
            </div>
          {:else}
            <!-- Source sentence diff comparison -->
            <div class="p-1.5 rounded bg-gray-50 dark:bg-gray-800/60 border border-gray-200 dark:border-gray-700/80">
              <div class="text-[9px] font-mono uppercase text-gray-400 mb-0.5 font-bold">Source Match Diff:</div>
              <div class="text-gray-800 dark:text-gray-200" dir="auto">
                {#if $selectedSegment}
                  {#each computeWordDiff($selectedSegment.source_text, topMatch.source_text) as diff}
                    {#if diff.type === 'del'}
                      <del class="bg-red-100 text-red-700 dark:bg-red-950/70 dark:text-red-300 rounded px-0.5 mx-0.5 font-medium">{diff.text}</del>
                    {:else if diff.type === 'add'}
                      <ins class="bg-emerald-100 text-emerald-800 dark:bg-emerald-950/70 dark:text-emerald-300 rounded px-0.5 mx-0.5 no-underline font-bold">{diff.text}</ins>
                    {:else}
                      <span>{diff.text}</span>
                    {/if}
                  {/each}
                {:else}
                  <span>{topMatch.source_text}</span>
                {/if}
              </div>
            </div>

            <!-- Target proposal -->
            <div class="p-1.5 rounded bg-emerald-50/50 dark:bg-emerald-950/30 border border-emerald-200/80 dark:border-emerald-800/60 flex items-start justify-between gap-2">
              <div class="flex-1 min-w-0">
                <div class="text-[9px] font-mono uppercase text-emerald-700 dark:text-emerald-400 mb-0.5 font-bold">Target Proposal:</div>
                <div class="text-gray-900 dark:text-gray-100 font-semibold" dir="auto">{topMatch.target_text}</div>
              </div>
              <button
                type="button"
                on:click={() => dispatch('applyTm', topMatch.target_text)}
                class="px-2 py-0.5 bg-emerald-600 hover:bg-emerald-700 text-white rounded text-[10px] font-bold shrink-0 transition-colors cursor-pointer"
              >
                Use
              </button>
            </div>

            <!-- FMR Auto-Patch Pill if available -->
            {#if activeAutoPatch}
              <div class="p-1.5 rounded bg-amber-50 dark:bg-amber-950/40 border border-amber-300 dark:border-amber-700 flex items-center justify-between text-xs">
                <div class="flex items-center gap-1.5 truncate">
                  <span class="text-amber-700 dark:text-amber-300 font-bold">⚡ FMR Repaired:</span>
                  <span class="font-semibold text-gray-900 dark:text-gray-100 truncate">{activeAutoPatch}</span>
                </div>
                <button
                  type="button"
                  on:click={() => dispatch('applyAutoPatch', activeAutoPatch)}
                  class="px-2 py-0.5 rounded bg-amber-500 hover:bg-amber-600 text-white font-bold text-[10px] shrink-0 cursor-pointer"
                >
                  Apply Patch (Tab)
                </button>
              </div>
            {/if}
          {/if}
        </div>
      </div>

      <!-- RIGHT 40%: Term Recognition (Termbase) -->
      <div class="w-[40%] flex flex-col min-w-0 bg-white dark:bg-gray-900">
        <!-- Header -->
        <div class="h-6 px-3 border-b border-gray-200 dark:border-gray-800 flex items-center justify-between bg-gray-100/70 dark:bg-gray-950/60 shrink-0">
          <div class="flex items-center gap-1.5 font-bold text-gray-800 dark:text-gray-200 text-[11px]">
            <span>📖</span>
            <span>Term Recognition</span>
            {#if $termMatches.length > 0}
              <span class="px-1.5 py-0.2 rounded-full bg-brand-100 text-brand-700 dark:bg-brand-900/60 dark:text-brand-300 font-mono text-[9px]">
                {$termMatches.length}
              </span>
            {/if}
          </div>

          <button
            type="button"
            on:click={() => dispatch('openTermManager')}
            class="text-[10px] text-brand-600 dark:text-brand-400 hover:underline cursor-pointer"
            title="Open Termbase Workbench"
          >
            Glossary ➔
          </button>
        </div>

        <!-- Term Matches List -->
        <div class="flex-1 p-2 overflow-y-auto space-y-1 divide-y divide-gray-100 dark:divide-gray-800">
          {#if $tbLoading}
            <div class="flex items-center justify-center h-full text-gray-400">
              Recognizing terms…
            </div>
          {:else if $termMatches.length === 0}
            <div class="flex flex-col items-center justify-center h-full text-gray-400 italic text-[11px] text-center p-2">
              <span>No terms recognized in current segment.</span>
              <span class="text-[10px] text-gray-400 mt-1">Press <kbd class="font-mono bg-gray-100 dark:bg-gray-800 px-1 rounded">Ctrl+F2</kbd> to add term.</span>
            </div>
          {:else}
            {#each $termMatches as match, i (match.id || i)}
              <div class="pt-1.5 first:pt-0 flex items-center justify-between gap-2 hover:bg-gray-50 dark:hover:bg-gray-800/60 p-1.5 rounded transition-colors group">
                <div class="min-w-0 flex-1">
                  <div class="flex items-center gap-1.5">
                    <span class="font-bold text-gray-900 dark:text-white font-editor">{match.term}</span>
                    {#if match.status}
                      <span class="text-[8px] font-mono uppercase px-1 rounded font-bold {match.status === 'preferred' ? 'bg-emerald-100 text-emerald-800 dark:bg-emerald-950 dark:text-emerald-300' : 'bg-blue-100 text-blue-800'}">
                        {match.status}
                      </span>
                    {/if}
                  </div>
                  {#if match.definition}
                    <div class="text-[10px] text-gray-500 dark:text-gray-400 truncate">
                      {match.definition}
                    </div>
                  {/if}
                </div>
                <button
                  type="button"
                  on:click={() => dispatch('insertTerm', match.term)}
                  class="px-2 py-0.5 rounded bg-gray-100 dark:bg-gray-800 hover:bg-brand-500 hover:text-white text-gray-700 dark:text-gray-300 font-bold text-[10px] shrink-0 transition-colors cursor-pointer flex items-center gap-1"
                  title="Insert Term at Caret"
                >
                  <span>Insert</span>
                  {#if i < 9}
                    <kbd class="text-[8px] opacity-70 font-mono">^{i+1}</kbd>
                  {/if}
                </button>
              </div>
            {/each}
          {/if}
        </div>
      </div>

    </div>

    <!-- Bottom Draggable Resizer Handle -->
    <div
      class="h-1.5 bg-gray-200 dark:bg-gray-800 hover:bg-brand-500 cursor-row-resize transition-colors flex items-center justify-center opacity-70 hover:opacity-100 z-10"
      role="separator"
      aria-orientation="horizontal"
      aria-label="Resize results pane"
      on:pointerdown={handleStartResize}
      title="Drag to adjust Translation Results pane height"
    >
      <div class="w-8 h-0.5 bg-gray-400 dark:bg-gray-500 rounded"></div>
    </div>
  </div>
{/if}
