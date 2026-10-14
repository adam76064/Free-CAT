<script lang="ts">
  /**
   * SegmentRow — A single bilingual source|target row in the editor grid.
   * Source cell: read-only with tag highlighting.
   * Target cell: contenteditable with Tab-to-confirm and auto-focus.
   */
  import { afterUpdate } from 'svelte';
  import type { Segment } from '$lib/stores/segments';
  import { STATUS_COLORS, STATUS_LABELS } from '$lib/stores/segments';
  import TmMatchBadge from '$lib/components/tm/TmMatchBadge.svelte';

  export let segment: Segment;
  export let isSelected: boolean;
  export let index: number = 0;
  export let onSelect: () => void;
  export let onConfirm: (seg: Segment) => void;
  export let onTargetChange: (id: string, text: string) => void;

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

  function handleKeydown(e: KeyboardEvent) {
    // Tab or Enter → confirm and move to next
    if (e.key === 'Tab' || (e.key === 'Enter' && !e.shiftKey)) {
      e.preventDefault();
      onConfirm({ ...segment, target_text: targetEl?.textContent ?? '' });
    }
    if (e.key === 'Escape') {
      (e.target as HTMLElement).blur();
    }
  }

  /** Render source text with {1}, {2} tag placeholders highlighted */
  function renderSource(text: string): string {
    return text.replace(
      /\{(\d+)\}/g,
      '<span class="tag-placeholder">{$1}</span>'
    );
  }

  $: rowBg = isSelected
    ? 'bg-brand-50 dark:bg-brand-900/20 ring-1 ring-inset ring-brand-300 dark:ring-brand-700'
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
    {#if segment.bookmarked}
      <span class="text-amber-500 text-xs leading-none" title="Bookmarked">★</span>
    {/if}
  </div>

  <!-- Source (read-only) -->
  <div
    class="py-2 px-3 text-sm text-gray-700 dark:text-gray-300 editor-text
           border-r border-gray-100 dark:border-gray-800 select-text"
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
    class="py-2 px-3 text-sm editor-text outline-none min-h-[3rem]
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

<!-- Status bar (only when selected) -->
{#if isSelected}
  <div class="flex items-center justify-between px-3 py-0.5
              bg-brand-50 dark:bg-brand-900/10
              border-b border-brand-100 dark:border-brand-900 text-xs">
    <div class="flex items-center gap-2">
      <span class="{STATUS_COLORS[segment.status]} px-1.5 py-0.5 rounded text-xs font-medium">
        {STATUS_LABELS[segment.status]}
      </span>
      {#if segment.comment}
        <span class="text-gray-500 dark:text-gray-400 italic truncate max-w-[200px]" title={segment.comment}>
          💬 {segment.comment}
        </span>
      {/if}
    </div>
    <span class="text-gray-400 dark:text-gray-600 select-none">Tab / Enter to confirm</span>
  </div>
{/if}
