<script lang="ts">
  /**
   * SegmentRow — A single bilingual source|target row in the editor grid.
   *
   * Source cell: read-only, displays plain text with tag placeholders highlighted.
   * Target cell: contenteditable div with tag protection, auto-save, and keyboard shortcuts.
   */
  import { createEventDispatcher, onMount } from 'svelte';
  import type { Segment } from '$lib/stores/segments';
  import { STATUS_COLORS, STATUS_LABELS } from '$lib/stores/segments';
  import TmMatchBadge from '$lib/components/tm/TmMatchBadge.svelte';

  interface Props {
    segment: Segment;
    isSelected: boolean;
    index: number;
    onSelect: () => void;
    onConfirm: (seg: Segment) => void;
    onTargetChange: (id: string, text: string) => void;
  }

  let { segment, isSelected, index, onSelect, onConfirm, onTargetChange }: Props = $props();

  let targetEl: HTMLDivElement | undefined = $state();

  // Sync target editor when segment changes externally (e.g. TM apply)
  $effect(() => {
    if (targetEl && targetEl.textContent !== segment.target_text) {
      targetEl.textContent = segment.target_text;
    }
  });

  // Focus target editor when this row is selected
  $effect(() => {
    if (isSelected && targetEl) {
      targetEl.focus();
      // Move cursor to end
      const range = document.createRange();
      const sel = window.getSelection();
      range.selectNodeContents(targetEl);
      range.collapse(false);
      sel?.removeAllRanges();
      sel?.addRange(range);
    }
  });

  function handleInput(e: Event) {
    const text = (e.target as HTMLDivElement).textContent ?? '';
    onTargetChange(segment.id, text);
  }

  function handleKeydown(e: KeyboardEvent) {
    // Tab or Enter → confirm and move to next segment
    if (e.key === 'Tab' || (e.key === 'Enter' && !e.shiftKey)) {
      e.preventDefault();
      onConfirm({ ...segment, target_text: targetEl?.textContent ?? '' });
    }
    // Escape → deselect / revert
    if (e.key === 'Escape') {
      targetEl?.blur();
    }
  }

  /** Render source text with tag placeholders ({1}, {2} etc.) highlighted */
  function renderSource(text: string): string {
    return text.replace(
      /\{(\d+)\}/g,
      '<span class="tag-placeholder">{$1}</span>'
    );
  }

  const rowBg = $derived(
    isSelected
      ? 'bg-brand-50 dark:bg-brand-900/20 ring-1 ring-inset ring-brand-300 dark:ring-brand-700'
      : 'hover:bg-gray-50 dark:hover:bg-gray-800/30'
  );
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="grid grid-cols-[2.5rem_1fr_1fr] min-h-[3rem] border-b border-gray-100 dark:border-gray-800
         transition-colors cursor-pointer {rowBg}"
  onclick={onSelect}
  role="row"
  aria-selected={isSelected}
>
  <!-- Row number + status indicator -->
  <div class="flex flex-col items-center justify-start pt-2 px-1 gap-1">
    <span class="text-xs text-gray-400 dark:text-gray-600 tabular-nums font-mono leading-none">
      {segment.seq_number}
    </span>
    {#if segment.match_percent !== null}
      <TmMatchBadge pct={segment.match_percent} />
    {/if}
    {#if segment.bookmarked}
      <span class="text-amber-500 text-xs" title="Bookmarked">★</span>
    {/if}
  </div>

  <!-- Source text (read-only) -->
  <div
    class="py-2 px-3 text-sm text-gray-700 dark:text-gray-300 editor-text border-r border-gray-100 dark:border-gray-800 select-text"
    dir="auto"
  >
    <!-- eslint-disable-next-line svelte/no-at-html-tags -->
    {@html renderSource(segment.source_text)}
  </div>

  <!-- Target editor (contenteditable) -->
  <div
    bind:this={targetEl}
    contenteditable="true"
    oninput={handleInput}
    onkeydown={handleKeydown}
    onfocus={onSelect}
    class="py-2 px-3 text-sm editor-text outline-none min-h-[3rem]
           text-gray-900 dark:text-white caret-brand-500
           {isSelected ? '' : 'text-gray-600 dark:text-gray-300'}"
    dir="auto"
    spellcheck="true"
    data-gramm="false"
    role="textbox"
    aria-multiline="true"
    aria-label="Translation target"
  ></div>
</div>

<!-- Status bar for selected row -->
{#if isSelected}
  <div class="flex items-center justify-between px-2 py-0.5 bg-brand-50 dark:bg-brand-900/10
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
    <span class="text-gray-400 dark:text-gray-600">
      Tab / Enter to confirm
    </span>
  </div>
{/if}
