<script lang="ts">
  /**
   * FindReplaceBar — Floating/docked in-editor search and replace.
   * Supports find-next, find-prev, match case, whole word, search target/source,
   * single replace, and replace all.
   */
  import { onMount, tick } from 'svelte';
  import type { Segment } from '$lib/stores/segments';

  export let segments: Segment[] = [];
  export let activeSegmentIndex: number = 0;
  export let isReplaceOpen: boolean = false;
  export let onSelectSegment: (index: number) => void;
  export let onUpdateSegment: (segId: string, newTarget: string) => void;
  export let onClose: () => void;

  let findInput: HTMLInputElement;
  let findText = '';
  let replaceText = '';
  let matchCase = false;
  let wholeWord = false;
  let targetField: 'both' | 'source' | 'target' = 'both';

  interface MatchLocation {
    segmentIndex: number;
    segmentId: string;
    field: 'source' | 'target';
    start: number;
    length: number;
  }

  let matches: MatchLocation[] = [];
  let currentMatchIndex = -1;

  onMount(() => {
    findInput?.focus();
    findInput?.select();
  });

  $: if (isReplaceOpen) {
    tick().then(() => {
      // Keep focus when toggled to replace
    });
  }

  // Recalculate matches whenever findText, options, or segments change
  $: computeMatches(findText, matchCase, wholeWord, targetField, segments);

  function computeMatches(
    query: string,
    caseSens: boolean,
    wordOnly: boolean,
    field: 'both' | 'source' | 'target',
    segs: Segment[]
  ) {
    if (!query) {
      matches = [];
      currentMatchIndex = -1;
      return;
    }

    const flags = caseSens ? 'g' : 'gi';
    const escaped = query.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    const patternStr = wordOnly ? `\\b${escaped}\\b` : escaped;
    let regex: RegExp;
    try {
      regex = new RegExp(patternStr, flags);
    } catch {
      matches = [];
      currentMatchIndex = -1;
      return;
    }

    const found: MatchLocation[] = [];

    segs.forEach((seg, sIdx) => {
      if (field === 'both' || field === 'source') {
        regex.lastIndex = 0;
        let m: RegExpExecArray | null;
        while ((m = regex.exec(seg.source_text)) !== null) {
          found.push({
            segmentIndex: sIdx,
            segmentId: seg.id,
            field: 'source',
            start: m.index,
            length: m[0].length,
          });
          if (m[0].length === 0) regex.lastIndex++;
        }
      }

      if (field === 'both' || field === 'target') {
        regex.lastIndex = 0;
        let m: RegExpExecArray | null;
        while ((m = regex.exec(seg.target_text)) !== null) {
          found.push({
            segmentIndex: sIdx,
            segmentId: seg.id,
            field: 'target',
            start: m.index,
            length: m[0].length,
          });
          if (m[0].length === 0) regex.lastIndex++;
        }
      }
    });

    matches = found;
    if (matches.length > 0) {
      // Find the first match at or after activeSegmentIndex
      const nextMatch = matches.findIndex(m => m.segmentIndex >= activeSegmentIndex);
      currentMatchIndex = nextMatch !== -1 ? nextMatch : 0;
    } else {
      currentMatchIndex = -1;
    }
  }

  function handleNext() {
    if (matches.length === 0) return;
    currentMatchIndex = (currentMatchIndex + 1) % matches.length;
    goToCurrentMatch();
  }

  function handlePrev() {
    if (matches.length === 0) return;
    currentMatchIndex = (currentMatchIndex - 1 + matches.length) % matches.length;
    goToCurrentMatch();
  }

  function goToCurrentMatch() {
    if (currentMatchIndex < 0 || currentMatchIndex >= matches.length) return;
    const match = matches[currentMatchIndex];
    onSelectSegment(match.segmentIndex);
  }

  function handleReplace() {
    if (matches.length === 0 || currentMatchIndex < 0) return;
    const match = matches[currentMatchIndex];
    const seg = segments[match.segmentIndex];
    if (!seg) return;

    // Replacement only applies to target text
    if (match.field === 'target') {
      const before = seg.target_text.slice(0, match.start);
      const after = seg.target_text.slice(match.start + match.length);
      const newTarget = before + replaceText + after;
      onUpdateSegment(seg.id, newTarget);
    } else {
      // If match was in source, replace in target if matching string exists in target
      const flags = matchCase ? 'g' : 'gi';
      const escaped = findText.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
      const pattern = wholeWord ? new RegExp(`\\b${escaped}\\b`, flags) : new RegExp(escaped, flags);
      if (pattern.test(seg.target_text)) {
        const newTarget = seg.target_text.replace(pattern, replaceText);
        onUpdateSegment(seg.id, newTarget);
      }
    }

    // Move to next match
    handleNext();
  }

  function handleReplaceAll() {
    if (!findText) return;
    const flags = matchCase ? 'g' : 'gi';
    const escaped = findText.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    const pattern = wholeWord ? new RegExp(`\\b${escaped}\\b`, flags) : new RegExp(escaped, flags);

    let replacedCount = 0;
    segments.forEach(seg => {
      if (pattern.test(seg.target_text)) {
        const newTarget = seg.target_text.replace(pattern, replaceText);
        onUpdateSegment(seg.id, newTarget);
        replacedCount++;
      }
    });

    alert(`Replaced occurrences in ${replacedCount} segments.`);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      onClose();
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (e.shiftKey) {
        handlePrev();
      } else {
        handleNext();
      }
    }
  }
</script>

<div class="px-3 py-2 bg-gray-50 dark:bg-gray-900 border-b border-gray-200 dark:border-gray-800 flex flex-col gap-1.5 shadow-sm">
  <div class="flex items-center gap-2">
    <!-- Search Icon & Input -->
    <div class="relative flex items-center">
      <span class="absolute left-2.5 text-xs text-gray-400">🔍</span>
      <input
        bind:this={findInput}
        bind:value={findText}
        on:keydown={handleKeydown}
        placeholder="Find in segments…"
        class="pl-7 pr-3 py-1 text-xs rounded-lg border border-gray-300 dark:border-gray-700
               bg-white dark:bg-gray-800 text-gray-900 dark:text-white w-52
               focus:outline-none focus:ring-1 focus:ring-brand-500"
      />
    </div>

    <!-- Match counter -->
    <span class="text-xs text-gray-500 dark:text-gray-400 min-w-[4.5rem]">
      {#if findText}
        {matches.length > 0 ? `${currentMatchIndex + 1} of ${matches.length}` : '0 results'}
      {:else}
        No query
      {/if}
    </span>

    <!-- Previous / Next buttons -->
    <div class="flex items-center gap-0.5">
      <button
        on:click={handlePrev}
        disabled={matches.length === 0}
        title="Previous match (Shift+Enter)"
        class="p-1 text-xs rounded hover:bg-gray-200 dark:hover:bg-gray-800 disabled:opacity-30 text-gray-700 dark:text-gray-300"
      >
        ▲
      </button>
      <button
        on:click={handleNext}
        disabled={matches.length === 0}
        title="Next match (Enter)"
        class="p-1 text-xs rounded hover:bg-gray-200 dark:hover:bg-gray-800 disabled:opacity-30 text-gray-700 dark:text-gray-300"
      >
        ▼
      </button>
    </div>

    <!-- Options: Match Case, Whole Word, Field -->
    <div class="flex items-center gap-1 border-l border-gray-200 dark:border-gray-700 pl-2">
      <button
        on:click={() => matchCase = !matchCase}
        class="px-1.5 py-0.5 text-[10px] font-mono rounded border transition-colors
               {matchCase
                 ? 'bg-brand-100 dark:bg-brand-900/40 border-brand-500 text-brand-600 dark:text-brand-300 font-bold'
                 : 'border-gray-300 dark:border-gray-700 text-gray-500 hover:bg-gray-200 dark:hover:bg-gray-800'}"
        title="Match case"
      >
        Aa
      </button>

      <button
        on:click={() => wholeWord = !wholeWord}
        class="px-1.5 py-0.5 text-[10px] font-mono rounded border transition-colors
               {wholeWord
                 ? 'bg-brand-100 dark:bg-brand-900/40 border-brand-500 text-brand-600 dark:text-brand-300 font-bold'
                 : 'border-gray-300 dark:border-gray-700 text-gray-500 hover:bg-gray-200 dark:hover:bg-gray-800'}"
        title="Whole word"
      >
        \b
      </button>

      <select
        bind:value={targetField}
        class="text-xs py-0.5 px-1 rounded border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300 focus:outline-none"
      >
        <option value="both">Both</option>
        <option value="source">Source</option>
        <option value="target">Target</option>
      </select>
    </div>

    <!-- Toggle Replace button -->
    <button
      on:click={() => isReplaceOpen = !isReplaceOpen}
      class="text-xs px-2 py-0.5 rounded border border-gray-300 dark:border-gray-700
             {isReplaceOpen ? 'bg-brand-50 dark:bg-brand-900/20 text-brand-600' : 'text-gray-600 dark:text-gray-400'}
             hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
    >
      Replace
    </button>

    <!-- Close button -->
    <button
      on:click={onClose}
      class="ml-auto text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 text-xs p-1"
      title="Close (Esc)"
    >
      ✕
    </button>
  </div>

  <!-- Replace row (when active) -->
  {#if isReplaceOpen}
    <div class="flex items-center gap-2 pt-1 border-t border-gray-200 dark:border-gray-800">
      <div class="relative flex items-center">
        <span class="absolute left-2.5 text-xs text-gray-400">✏️</span>
        <input
          bind:value={replaceText}
          on:keydown={(e) => e.key === 'Enter' && handleReplace()}
          placeholder="Replace target with…"
          class="pl-7 pr-3 py-1 text-xs rounded-lg border border-gray-300 dark:border-gray-700
                 bg-white dark:bg-gray-800 text-gray-900 dark:text-white w-52
                 focus:outline-none focus:ring-1 focus:ring-brand-500"
        />
      </div>

      <button
        on:click={handleReplace}
        disabled={matches.length === 0}
        class="px-2.5 py-1 text-xs font-medium rounded-lg bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-700
               text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-40 transition-colors"
      >
        Replace
      </button>

      <button
        on:click={handleReplaceAll}
        disabled={!findText}
        class="px-2.5 py-1 text-xs font-medium rounded-lg bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-700
               text-gray-700 dark:text-gray-200 hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-40 transition-colors"
      >
        Replace All
      </button>
    </div>
  {/if}
</div>
