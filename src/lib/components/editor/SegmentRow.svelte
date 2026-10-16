<script lang="ts">
  /**
   * SegmentRow — A single bilingual source|target row in the editor grid.
   * Source cell: read-only with tag highlighting.
   * Target cell: contenteditable with shortcuts and auto-focus.
   * Row actions: Split, Merge Next/Prev, Copy Source, Clear, Bookmark.
   */
  import { afterUpdate } from 'svelte';
  import type { Segment } from '$lib/stores/segments';
  import { STATUS_COLORS, STATUS_LABELS } from '$lib/stores/segments';
  import TmMatchBadge from '$lib/components/tm/TmMatchBadge.svelte';

  export let segment: Segment;
  export let isSelected: boolean;
  export let index: number = 0;
  export let canMergePrev: boolean = false;
  export let canMergeNext: boolean = false;
  export let onSelect: () => void;
  export let onConfirm: (seg: Segment) => void;
  export let onTargetChange: (id: string, text: string) => void;
  export let onSplit: (segId: string) => void;
  export let onMergeNext: (segId: string) => void;
  export let onMergePrev: (segId: string) => void;
  export let onToggleBookmark: (segId: string) => void;

  let targetEl: HTMLDivElement;

  // Sync contenteditable when target_text changes from outside (e.g. TM apply)
  afterUpdate(() => {
    if (targetEl && targetEl.textContent !== segment.target_text) {
      targetEl.textContent = segment.target_text;
    }
    // Focus when this row becomes selected
    if (isSelected && document.activeElement !== targetEl) {
      targetEl?.focus();
      moveCursorToEnd();
    }
  });

  function moveCursorToEnd() {
    if (!targetEl) return;
    const range = document.createRange();
    const sel = window.getSelection();
    range.selectNodeContents(targetEl);
    range.collapse(false);
    sel?.removeAllRanges();
    sel?.addRange(range);
  }

  function handleInput(e: Event) {
    const text = (e.target as HTMLDivElement).textContent ?? '';
    onTargetChange(segment.id, text);
  }

  function handleCopySource() {
    onTargetChange(segment.id, segment.source_text);
    if (targetEl) targetEl.textContent = segment.source_text;
  }

  function handleClearTarget() {
    onTargetChange(segment.id, '');
    if (targetEl) targetEl.textContent = '';
  }

  function handleKeydown(e: KeyboardEvent) {
    // Ctrl+Enter or Tab: Confirm and advance
    if (e.key === 'Tab' || (e.key === 'Enter' && (e.ctrlKey || !e.shiftKey))) {
      e.preventDefault();
      onConfirm({ ...segment, target_text: targetEl?.textContent ?? '' });
      return;
    }

    // Ctrl+Ins or Alt+Ins: Copy source to target
    if ((e.ctrlKey || e.altKey) && e.key === 'Insert') {
      e.preventDefault();
      handleCopySource();
      return;
    }

    // Ctrl+B: Toggle bookmark
    if (e.ctrlKey && e.key.toLowerCase() === 'b') {
      e.preventDefault();
      onToggleBookmark(segment.id);
      return;
    }

    // Ctrl+J: Merge next
    if (e.ctrlKey && !e.shiftKey && e.key.toLowerCase() === 'j' && canMergeNext) {
      e.preventDefault();
      onMergeNext(segment.id);
      return;
    }

    // Ctrl+Shift+J: Merge prev
    if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === 'j' && canMergePrev) {
      e.preventDefault();
      onMergePrev(segment.id);
      return;
    }

    // Ctrl+Alt+S: Split segment
    if (e.ctrlKey && e.altKey && e.key.toLowerCase() === 's') {
      e.preventDefault();
      onSplit(segment.id);
      return;
    }

    if (e.key === 'Escape') {
      (e.target as HTMLElement).blur();
    }
  }

  /** Render source text with {1}, {2} tag placeholders highlighted */
  function renderSource(text: string): string {
    return text.replace(
      /\{(\d+)\}/g,
      '<span class="tag-placeholder font-mono text-[11px] bg-amber-100 text-amber-800 dark:bg-amber-900/40 dark:text-amber-300 px-1 py-0.5 rounded mx-0.5">{$1}</span>'
    );
  }

  $: rowBg = isSelected
    ? 'bg-brand-50/70 dark:bg-brand-900/20 ring-1 ring-inset ring-brand-300 dark:ring-brand-700'
    : 'hover:bg-gray-50 dark:hover:bg-gray-800/30';
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<!-- svelte-ignore a11y-no-static-element-interactions -->
<div
  class="grid grid-cols-[2.5rem_1fr_1fr] min-h-[3rem] border-b border-gray-100 dark:border-gray-800
         transition-colors cursor-pointer {rowBg}"
  on:click={onSelect}
  role="row"
  tabindex="-1"
  aria-selected={isSelected}
>
  <!-- Row number + badges -->
  <div class="flex flex-col items-center justify-start pt-2 px-1 gap-1 shrink-0">
    <span class="text-xs text-gray-400 dark:text-gray-600 tabular-nums font-mono leading-none select-none">
      {segment.seq_number}
    </span>
    {#if segment.match_percent !== null && segment.match_percent !== undefined}
      <TmMatchBadge pct={segment.match_percent} />
    {/if}
    <button
      on:click|stopPropagation={() => onToggleBookmark(segment.id)}
      class="text-xs leading-none transition-transform hover:scale-125
             {segment.bookmarked ? 'text-amber-500' : 'text-gray-300 dark:text-gray-700 hover:text-amber-400'}"
      title={segment.bookmarked ? 'Bookmarked (Ctrl+B)' : 'Add bookmark (Ctrl+B)'}
    >
      ★
    </button>
  </div>

  <!-- Source (read-only) -->
  <div
    class="py-2 px-3 text-sm text-gray-700 dark:text-gray-300 editor-text
           border-r border-gray-100 dark:border-gray-800 select-text leading-relaxed"
    dir="auto"
  >
    <!-- eslint-disable-next-line svelte/no-at-html-tags -->
    {@html renderSource(segment.source_text)}
  </div>

  <!-- Target (editable) -->
  <!-- svelte-ignore a11y-interactive-supports-focus -->
  <div
    bind:this={targetEl}
    contenteditable="true"
    tabindex="0"
    on:input={handleInput}
    on:keydown={handleKeydown}
    on:focus={onSelect}
    class="py-2 px-3 text-sm editor-text outline-none min-h-[3rem] leading-relaxed
           text-gray-900 dark:text-white caret-brand-500
           {isSelected ? '' : 'text-gray-600 dark:text-gray-300'}"
    dir="auto"
    spellcheck="true"
    data-gramm="false"
    role="textbox"
    aria-multiline="true"
    aria-label="Translation target for segment {segment.seq_number}"
  ></div>
</div>

<!-- Status & Action Bar (when selected) -->
{#if isSelected}
  <div class="flex items-center justify-between px-3 py-1
              bg-brand-50 dark:bg-brand-950/40
              border-b border-brand-200 dark:border-brand-800/60 text-xs">
    <!-- Left: Status badge & comment -->
    <div class="flex items-center gap-2">
      <span class="{STATUS_COLORS[segment.status]} px-1.5 py-0.5 rounded text-[11px] font-medium">
        {STATUS_LABELS[segment.status]}
      </span>
      {#if segment.comment}
        <span class="text-gray-500 dark:text-gray-400 italic truncate max-w-[200px]" title={segment.comment}>
          💬 {segment.comment}
        </span>
      {/if}
    </div>

    <!-- Center/Right: Quick Actions -->
    <div class="flex items-center gap-1">
      <button
        on:click={handleCopySource}
        class="px-2 py-0.5 rounded bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700
               text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
        title="Copy source into target (Ctrl+Ins)"
      >
        📋 Copy Source
      </button>

      <button
        on:click={handleClearTarget}
        class="px-2 py-0.5 rounded bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700
               text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
        title="Clear target translation"
      >
        🧹 Clear
      </button>

      <button
        on:click={() => onSplit(segment.id)}
        class="px-2 py-0.5 rounded bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700
               text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
        title="Split segment (Ctrl+Alt+S)"
      >
        ✂ Split
      </button>

      {#if canMergeNext}
        <button
          on:click={() => onMergeNext(segment.id)}
          class="px-2 py-0.5 rounded bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700
                 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
          title="Merge with next segment (Ctrl+J)"
        >
          ⤹ Merge Next
        </button>
      {/if}

      {#if canMergePrev}
        <button
          on:click={() => onMergePrev(segment.id)}
          class="px-2 py-0.5 rounded bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700
                 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
          title="Merge with previous segment (Ctrl+Shift+J)"
        >
          ⤸ Merge Prev
        </button>
      {/if}

      <span class="text-gray-400 dark:text-gray-600 pl-2 select-none text-[11px]">
        Ctrl+Enter to confirm
      </span>
    </div>
  </div>
{/if}
