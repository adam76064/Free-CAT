<script lang="ts">
  /**
   * SegmentRow — A single bilingual source|target row in the editor grid.
   * Source cell: read-only with tag highlighting.
   * Target cell: contenteditable with predictive AutoSuggest and shortcuts.
   * Row actions: Split, Merge Next/Prev, Copy Source, Clear, Bookmark.
   */
  import { afterUpdate } from 'svelte';
  import type { Segment } from '$lib/stores/segments';
  import { STATUS_COLORS, STATUS_LABELS } from '$lib/stores/segments';
  import TmMatchBadge from '$lib/components/tm/TmMatchBadge.svelte';
  import AutoSuggestPopup from './AutoSuggestPopup.svelte';
  import type { SuggestionItem } from '$lib/utils/suggestTrie';

  export let segment: Segment;
  export let isSelected: boolean;
  export let canMergePrev: boolean = false;
  export let canMergeNext: boolean = false;
  export let activeSuggestions: SuggestionItem[] = [];
  export let autoPatchSuggestion: string | null = null;
  export let onSelect: () => void;
  export let onConfirm: (seg: Segment) => void;
  export let onTargetChange: (id: string, text: string) => void;
  export let onSplit: (segId: string) => void;
  export let onMergeNext: (segId: string) => void;
  export let onMergePrev: (segId: string) => void;
  export let onToggleBookmark: (segId: string) => void;
  export let onQuerySuggestions: (word: string) => void = () => {};
  export let onApplyAutoPatch: (patch: string) => void = () => {};

  let targetEl: HTMLDivElement;
  let suggestVisible = false;
  let activeSuggestIndex = 0;

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

  function getCurrentWordAtCaret(): string {
    const sel = window.getSelection();
    if (!sel || !sel.focusNode) return '';
    const textBeforeCaret = sel.focusNode.textContent?.slice(0, sel.focusOffset) || '';
    const match = textBeforeCaret.match(/[\p{L}\p{N}_-]+$/u);
    return match ? match[0] : '';
  }

  function handleInput(e: Event) {
    const text = (e.target as HTMLDivElement).textContent ?? '';
    onTargetChange(segment.id, text);

    const currentWord = getCurrentWordAtCaret();
    if (currentWord.length >= 2) {
      onQuerySuggestions(currentWord);
      suggestVisible = true;
      activeSuggestIndex = 0;
    } else {
      suggestVisible = false;
    }
  }

  function handleApplySuggestion(item: SuggestionItem) {
    const currentWord = getCurrentWordAtCaret();
    const currentFullText = targetEl?.textContent ?? '';
    let newText = currentFullText;

    if (currentWord) {
      // Replace the trailing word prefix with the full suggestion
      const lastIndex = currentFullText.lastIndexOf(currentWord);
      if (lastIndex !== -1) {
        newText = currentFullText.slice(0, lastIndex) + item.text + ' ';
      } else {
        newText = currentFullText + ' ' + item.text;
      }
    } else {
      newText = currentFullText ? `${currentFullText} ${item.text}` : item.text;
    }

    onTargetChange(segment.id, newText);
    if (targetEl) targetEl.textContent = newText;
    suggestVisible = false;
    setTimeout(() => moveCursorToEnd(), 20);
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
    // If suggestions are visible and Tab is pressed: accept top suggestion
    if (suggestVisible && activeSuggestions.length > 0 && e.key === 'Tab') {
      e.preventDefault();
      handleApplySuggestion(activeSuggestions[activeSuggestIndex]);
      return;
    }

    if (suggestVisible && activeSuggestions.length > 0 && (e.key === 'ArrowDown' || e.key === 'ArrowUp')) {
      e.preventDefault();
      if (e.key === 'ArrowDown') {
        activeSuggestIndex = (activeSuggestIndex + 1) % activeSuggestions.length;
      } else {
        activeSuggestIndex = (activeSuggestIndex - 1 + activeSuggestions.length) % activeSuggestions.length;
      }
      return;
    }

    if (e.key === 'Escape') {
      if (suggestVisible) {
        e.preventDefault();
        suggestVisible = false;
        return;
      }
      (e.target as HTMLElement).blur();
      return;
    }

    // Tab when target is empty and autoPatchSuggestion is available: apply auto patch!
    if (e.key === 'Tab' && autoPatchSuggestion && (!targetEl?.textContent || !targetEl.textContent.trim())) {
      e.preventDefault();
      onApplyAutoPatch(autoPatchSuggestion);
      return;
    }

    // Ctrl+Enter or Tab (when no suggestions popup and target has content): Confirm and advance
    if (e.key === 'Tab' || (e.key === 'Enter' && (e.ctrlKey || !e.shiftKey))) {
      e.preventDefault();
      suggestVisible = false;
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
  class="grid grid-cols-[2.75rem_1fr_1fr] min-h-[3rem] border-b border-gray-100 dark:border-gray-800/80
         transition-colors cursor-pointer relative group {rowBg}"
  on:click={onSelect}
  role="row"
  tabindex="-1"
  aria-selected={isSelected}
>
  <!-- Row number + Status indicator + bookmark -->
  <div class="flex items-center justify-between px-2 py-2 shrink-0 select-none border-r border-gray-100 dark:border-gray-800/60 bg-gray-50/40 dark:bg-gray-900/20">
    <div class="flex flex-col items-center gap-1">
      <span class="text-xs text-gray-400 dark:text-gray-500 tabular-nums font-mono leading-none">
        {segment.seq_number}
      </span>
      <span
        class="w-1.5 h-1.5 rounded-full {segment.status === 'translated' ? 'bg-emerald-500' : segment.status === 'draft' ? 'bg-amber-500' : 'bg-gray-300 dark:bg-gray-700'}"
        title={STATUS_LABELS[segment.status]}
      ></span>
    </div>
    <div class="flex flex-col items-center gap-1">
      <button
        on:click|stopPropagation={() => onToggleBookmark(segment.id)}
        class="text-xs leading-none transition-transform hover:scale-125
               {segment.bookmarked ? 'text-amber-500' : 'text-gray-300 dark:text-gray-700 hover:text-amber-400'}"
        title={segment.bookmarked ? 'Bookmarked (Ctrl+B)' : 'Add bookmark (Ctrl+B)'}
      >
        ★
      </button>
      {#if segment.match_percent !== null && segment.match_percent !== undefined}
        <TmMatchBadge pct={segment.match_percent} />
      {/if}
    </div>
  </div>

  <!-- Source (read-only) -->
  <div
    class="py-2.5 px-3.5 text-sm text-gray-800 dark:text-gray-200 editor-text
           border-r border-gray-100 dark:border-gray-800 select-text leading-relaxed font-editor"
    dir="auto"
  >
    <!-- eslint-disable-next-line svelte/no-at-html-tags -->
    {@html renderSource(segment.source_text)}
  </div>

  <!-- Target (editable) -->
  <!-- svelte-ignore a11y-interactive-supports-focus -->
  <div class="relative flex flex-col justify-start">
    <!-- Auto-Patch Pill Banner (when available and target empty) -->
    {#if isSelected && autoPatchSuggestion && (!segment.target_text || !segment.target_text.trim())}
      <div class="m-1.5 px-2.5 py-1 rounded-md bg-amber-50 dark:bg-amber-950/60 border border-amber-300 dark:border-amber-700 flex items-center justify-between text-xs animate-in fade-in z-20 shadow-xs">
        <div class="flex items-center gap-1.5 min-w-0 pr-2">
          <span class="text-amber-700 dark:text-amber-300 font-bold shrink-0">⚡ Auto-Patch:</span>
          <span class="font-editor text-gray-900 dark:text-gray-100 font-semibold truncate" dir="auto">{autoPatchSuggestion}</span>
        </div>
        <button
          type="button"
          on:mousedown|preventDefault={() => onApplyAutoPatch(autoPatchSuggestion)}
          class="px-2 py-0.5 rounded bg-amber-500 hover:bg-amber-600 text-white font-medium text-[11px] shrink-0 transition-colors flex items-center gap-1 cursor-pointer shadow-xs"
        >
          <span>Apply</span>
          <kbd class="text-[9px] bg-amber-600 px-1 py-0.2 rounded font-mono">Tab</kbd>
        </button>
      </div>
    {/if}

    {#if isSelected}
      <AutoSuggestPopup
        visible={suggestVisible}
        suggestions={activeSuggestions}
        selectedIndex={activeSuggestIndex}
        onSelect={handleApplySuggestion}
      />
    {/if}

    <div
      bind:this={targetEl}
      contenteditable="true"
      tabindex="0"
      on:input={handleInput}
      on:keydown={handleKeydown}
      on:focus={onSelect}
      class="py-2.5 px-3.5 text-sm editor-text outline-none flex-1 min-h-[3rem] leading-relaxed
             text-gray-900 dark:text-white caret-brand-500 font-editor
             {isSelected ? '' : 'text-gray-600 dark:text-gray-300'}"
      dir="auto"
      spellcheck="true"
      data-gramm="false"
      role="textbox"
      aria-multiline="true"
      aria-label="Translation target for segment {segment.seq_number}"
    ></div>

    <!-- Elegant floating quick-action hover buttons on top right -->
    <div class="absolute top-1.5 right-2 flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity bg-white/95 dark:bg-gray-900/95 backdrop-blur-xs px-1 py-0.5 rounded-md border border-gray-200 dark:border-gray-700/80 shadow-xs z-10 {isSelected ? 'opacity-90' : ''}">
      <button
        type="button"
        on:click|stopPropagation={handleCopySource}
        class="p-1 rounded text-xs text-gray-500 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
        title="Copy Source (Alt+Ins)"
      >📋</button>
      <button
        type="button"
        on:click|stopPropagation={handleClearTarget}
        class="p-1 rounded text-xs text-gray-500 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
        title="Clear Target"
      >🧹</button>
      <button
        type="button"
        on:click|stopPropagation={() => onSplit(segment.id)}
        class="p-1 rounded text-xs text-gray-500 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
        title="Split Segment (Ctrl+Alt+S)"
      >✂</button>
      {#if canMergeNext}
        <button
          type="button"
          on:click|stopPropagation={() => onMergeNext(segment.id)}
          class="p-1 rounded text-xs text-gray-500 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
          title="Merge Next (Ctrl+J)"
        >⤹</button>
      {/if}
      {#if canMergePrev}
        <button
          type="button"
          on:click|stopPropagation={() => onMergePrev(segment.id)}
          class="p-1 rounded text-xs text-gray-500 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
          title="Merge Prev (Ctrl+Shift+J)"
        >⤸</button>
      {/if}
    </div>
  </div>
</div>
