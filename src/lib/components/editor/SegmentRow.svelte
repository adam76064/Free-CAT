<script lang="ts">
  import { renderSource } from '$lib/utils/html';
  import { extractTags, textStats, cycleCase, getStructureCode } from '$lib/utils/segmentText';
  /**
   * SegmentRow — Enterprise bilingual source|target row with:
   * - Interactive QuickTag HUD (Ctrl+,) & tag balance validation
   * - Multi-layout support: side-by-side, stacked, and zen focus mode
   * - Dynamic optical font scaling (with +2px boost for Arabic/CJK)
   * - Inline AI Refinement HUD on text selection
   * - Track Changes / Revision Mode diff display with Accept/Reject
   * - Native OS spellcheck integration
   */
  import { afterUpdate } from 'svelte';
  import type { Segment } from '$lib/stores/segments';
  import { STATUS_COLORS, STATUS_LABELS } from '$lib/stores/segments';
  import { settings } from '$lib/stores/settings';
  import { currentProject } from '$lib/stores/project';
  import TmMatchBadge from '$lib/components/tm/TmMatchBadge.svelte';
  import AutoSuggestPopup from './AutoSuggestPopup.svelte';
  import QuickPlaceOmnibar from './QuickPlaceOmnibar.svelte';
  import type { SuggestionItem } from '$lib/utils/suggestTrie';
  import { autoTextEntries } from '$lib/stores/autotext';

  export let segment: Segment;
  export let isSelected: boolean;
  /** 1-based position in the full list (rows are windowed, so assistive tech needs it). */
  export let rowIndex: number | undefined = undefined;
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
  export let onToggleLock: (segId: string) => void = () => {};
  export let onQuerySuggestions: (word: string) => void = () => {};
  export let onApplyAutoPatch: (patch: string) => void = () => {};
  export let onAiRefineAction: (action: string, selectedText: string) => void = () => {};
  export let onSourceChange: (id: string, text: string) => void = () => {};

  let targetEl: HTMLDivElement;
  let sourceEditEl: HTMLDivElement;
  let isSourceEditing = false;
  let suggestVisible = false;
  let activeSuggestIndex = 0;

  // QuickPlace Omnibar HUD state
  let quickPlaceVisible = false;
  let activeQuickPlaceIndex = 0;
  let highlightedSourceToken: string | null = null;

  // Track Changes / Revision initial text
  let originalTargetText: string = segment.target_text;

  // Inline AI Refinement HUD state
  let selectedTargetText = '';
  let isAiRefineMenuOpen = false;

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

  $: ({ srcChars, srcWords, tgtChars, tgtWords, expansionRatio } = textStats(segment.source_text, segment.target_text));

  export function focusTarget() {
    targetEl?.focus();
    moveCursorToEnd();
  }

  export function focusSource() {
    if (!isSourceEditing) {
      handleToggleSourceEditing();
    }
    setTimeout(() => {
      sourceEditEl?.focus();
    }, 40);
  }

  function handleAutoTextSpace(e: KeyboardEvent): boolean {
    if (segment.locked || !targetEl) return false;
    const currentWord = getCurrentWordAtCaret();
    if (!currentWord) return false;

    const entry = $autoTextEntries.find(at => at.trigger.toLowerCase() === currentWord.toLowerCase());
    if (entry) {
      e.preventDefault();
      const currentFullText = targetEl.textContent || '';
      const lastIndex = currentFullText.lastIndexOf(currentWord);
      let newText = currentFullText;
      if (lastIndex !== -1) {
        newText = currentFullText.slice(0, lastIndex) + entry.expansion + ' ';
      } else {
        newText = currentFullText + ' ' + entry.expansion + ' ';
      }
      targetEl.textContent = newText;
      onTargetChange(segment.id, newText);
      moveCursorToEnd();
      return true;
    }
    return false;
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

  $: sourceTags = extractTags(segment.source_text);
  $: targetTags = extractTags(segment.target_text);
  $: missingTags = sourceTags.filter(t => !targetTags.includes(t));

  function handleInsertPlaceable(text: string) {
    if (!targetEl || segment.locked) return;
    targetEl.focus();
    const sel = window.getSelection();
    if (sel && sel.rangeCount > 0 && targetEl.contains(sel.anchorNode)) {
      const range = sel.getRangeAt(0);
      range.deleteContents();
      const node = document.createTextNode(text);
      range.insertNode(node);
      range.setStartAfter(node);
      range.collapse(true);
      sel.removeAllRanges();
      sel.addRange(range);
    } else {
      const current = targetEl.textContent || '';
      targetEl.textContent = current ? `${current} ${text}` : text;
      moveCursorToEnd();
    }
    const newText = targetEl.textContent || '';
    onTargetChange(segment.id, newText);
    quickPlaceVisible = false;
    highlightedSourceToken = null;
  }

  function handleInsertAllMissingTags() {
    if (missingTags.length === 0 || segment.locked) return;
    const current = targetEl?.textContent || '';
    const toAppend = (current.endsWith(' ') || !current ? '' : ' ') + missingTags.join(' ');
    const newText = current + toAppend;
    if (targetEl) targetEl.textContent = newText;
    onTargetChange(segment.id, newText);
    quickPlaceVisible = false;
  }

  function handleCopySource() {
    if (segment.locked) return;
    onTargetChange(segment.id, segment.source_text);
    if (targetEl) targetEl.textContent = segment.source_text;
  }

  function handleClearTarget() {
    if (segment.locked) return;
    onTargetChange(segment.id, '');
    if (targetEl) targetEl.textContent = '';
  }

  function handleAcceptRevision() {
    originalTargetText = segment.target_text;
    onConfirm({ ...segment, target_text: segment.target_text });
  }

  function handleRejectRevision() {
    if (targetEl) targetEl.textContent = originalTargetText;
    onTargetChange(segment.id, originalTargetText);
  }

  function handleToggleSourceEditing() {
    isSourceEditing = !isSourceEditing;
    if (isSourceEditing) {
      setTimeout(() => {
        if (sourceEditEl) {
          sourceEditEl.focus();
          const range = document.createRange();
          const sel = window.getSelection();
          range.selectNodeContents(sourceEditEl);
          range.collapse(false);
          sel?.removeAllRanges();
          sel?.addRange(range);
        }
      }, 20);
    }
  }

  function handleSaveSource() {
    if (!sourceEditEl) return;
    const newText = (sourceEditEl.textContent || '').trim();
    isSourceEditing = false;
    if (newText && newText !== segment.source_text) {
      onSourceChange(segment.id, newText);
    }
  }

  function handleCancelSource() {
    if (sourceEditEl) {
      sourceEditEl.textContent = segment.source_text;
    }
    isSourceEditing = false;
  }

  function handleCycleCase() {
    if (segment.locked || !targetEl) return;
    const sel = window.getSelection();
    if (!sel || !sel.rangeCount || !targetEl.contains(sel.anchorNode)) return;

    const selectedText = sel.toString();
    if (selectedText.length > 0) {
      const nextText = cycleCase(selectedText);
      const range = sel.getRangeAt(0);
      range.deleteContents();
      const node = document.createTextNode(nextText);
      range.insertNode(node);
      range.selectNodeContents(node);
      sel.removeAllRanges();
      sel.addRange(range);
      const newFullText = targetEl.textContent || '';
      onTargetChange(segment.id, newFullText);
    } else {
      const full = targetEl.textContent || '';
      const currentWord = getCurrentWordAtCaret();
      if (currentWord) {
        const nextWord = cycleCase(currentWord);
        const lastIdx = full.lastIndexOf(currentWord);
        if (lastIdx !== -1) {
          const replaced = full.slice(0, lastIdx) + nextWord + full.slice(lastIdx + currentWord.length);
          targetEl.textContent = replaced;
          onTargetChange(segment.id, replaced);
        }
      }
    }
  }

  function handleSourceKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSaveSource();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      handleCancelSource();
    }
  }

  function handleSelectionCheck() {
    if (!isSelected) {
      isAiRefineMenuOpen = false;
      return;
    }
    const sel = window.getSelection();
    if (!sel || sel.isCollapsed || !targetEl?.contains(sel.anchorNode)) {
      isAiRefineMenuOpen = false;
      selectedTargetText = '';
      return;
    }
    const txt = sel.toString().trim();
    if (txt.length >= 2) {
      selectedTargetText = txt;
      isAiRefineMenuOpen = true;
    } else {
      isAiRefineMenuOpen = false;
      selectedTargetText = '';
    }
  }

  function handleTriggerAiAction(action: string) {
    if (!selectedTargetText) return;
    onAiRefineAction(action, selectedTargetText);
    isAiRefineMenuOpen = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    // AutoText shorthand expansion on Space
    if (e.key === ' ' && !e.ctrlKey && !e.altKey && !e.metaKey) {
      if (handleAutoTextSpace(e)) {
        return;
      }
    }

    // QuickPlace Omnibar HUD: Ctrl+, or Cmd+,
    if ((e.ctrlKey || e.metaKey) && e.key === ',') {
      e.preventDefault();
      quickPlaceVisible = !quickPlaceVisible;
      activeQuickPlaceIndex = 0;
      return;
    }

    // In-Place Source Segment Editing: Alt+F2
    if (e.altKey && e.key === 'F2') {
      e.preventDefault();
      handleToggleSourceEditing();
      return;
    }

    // Case Cycling: Shift+F3 (Word / Trados standard: lowercase -> UPPERCASE -> Title Case)
    if (e.shiftKey && e.key === 'F3') {
      e.preventDefault();
      handleCycleCase();
      return;
    }

    // Clear Target: Alt+Delete (Trados standard)
    if (e.altKey && e.key === 'Delete') {
      e.preventDefault();
      handleClearTarget();
      return;
    }

    // Copy Source to Target: Ctrl+Shift+S
    if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === 's') {
      e.preventDefault();
      handleCopySource();
      return;
    }

    // Toggle Lock: Ctrl+L
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'l' && !e.shiftKey) {
      e.preventDefault();
      onToggleLock(segment.id);
      return;
    }

    // Delete to end of segment: Ctrl+D
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'd' && !e.shiftKey) {
      e.preventDefault();
      if (!segment.locked && targetEl) {
        const sel = window.getSelection();
        if (sel && sel.rangeCount > 0 && targetEl.contains(sel.anchorNode)) {
          const range = sel.getRangeAt(0);
          range.setEnd(targetEl, targetEl.childNodes.length);
          range.deleteContents();
          const newText = targetEl.textContent || '';
          onTargetChange(segment.id, newText);
        }
      }
      return;
    }

    // Delete to next tag: Ctrl+Shift+D
    if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === 'd') {
      e.preventDefault();
      if (!segment.locked && targetEl) {
        const full = targetEl.textContent || '';
        const sel = window.getSelection();
        const offset = sel?.focusOffset || 0;
        const after = full.slice(offset);
        const tagMatch = after.search(/\{[^}]+\}/);
        let newText = full;
        if (tagMatch !== -1) {
          newText = full.slice(0, offset) + after.slice(tagMatch);
        } else {
          newText = full.slice(0, offset);
        }
        targetEl.textContent = newText;
        onTargetChange(segment.id, newText);
      }
      return;
    }

    // AutoSuggest popup navigation
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
      if (suggestVisible || quickPlaceVisible || isAiRefineMenuOpen) {
        e.preventDefault();
        suggestVisible = false;
        quickPlaceVisible = false;
        highlightedSourceToken = null;
        isAiRefineMenuOpen = false;
        return;
      }
      (e.target as HTMLElement).blur();
      return;
    }

    // Tab when target empty and autoPatchSuggestion available
    if (e.key === 'Tab' && autoPatchSuggestion && (!targetEl?.textContent || !targetEl.textContent.trim())) {
      e.preventDefault();
      onApplyAutoPatch(autoPatchSuggestion);
      return;
    }

    // Revision Mode: Ctrl+Shift+A (Accept) & Ctrl+Shift+R (Reject)
    if ($settings.revisionMode) {
      if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === 'a') {
        e.preventDefault();
        handleAcceptRevision();
        return;
      }
      if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === 'r') {
        e.preventDefault();
        handleRejectRevision();
        return;
      }
    }

    // Ctrl+Enter or Tab (when target has content): Confirm and advance
    if (e.key === 'Tab' || (e.key === 'Enter' && (e.ctrlKey || !e.shiftKey))) {
      e.preventDefault();
      suggestVisible = false;
      quickPlaceVisible = false;
      highlightedSourceToken = null;
      isAiRefineMenuOpen = false;
      onConfirm({ ...segment, target_text: targetEl?.textContent ?? '' });
      return;
    }

    // Ctrl+Ins or Alt+Ins: Copy source
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

  $: structure = getStructureCode(segment.source_text);

  /** Render source text with stylish non-breakable tag pills & live QuickPlace spotlight */
  $: rowBg = isSelected
    ? 'bg-brand-50/70 dark:bg-brand-900/20 ring-1 ring-inset ring-brand-300 dark:ring-brand-700'
    : 'hover:bg-gray-50 dark:hover:bg-gray-800/30';

  $: zenClass = $settings.layoutMode === 'zen' && !isSelected
    ? 'opacity-35 hover:opacity-85 blur-[0.2px] hover:blur-none transition-all duration-200'
    : '';

  $: targetFontSize = $settings.fontSize + ($settings.targetOpticalBoost ? 2 : 0);

  $: isRevised = $settings.revisionMode &&
                 originalTargetText.trim().length > 0 &&
                 segment.target_text !== originalTargetText;
</script>

<!-- svelte-ignore a11y-click-events-have-key-events -->
<!-- svelte-ignore a11y-no-static-element-interactions -->
{#if $settings.layoutMode === 'stacked'}
  <!-- Stacked Layout: Source block over Target block -->
  <div
    class="grid grid-cols-[3px_3.25rem_1fr] min-h-[4.5rem] border-b border-gray-100 dark:border-gray-800/80
           transition-colors cursor-pointer relative group {rowBg} {zenClass}"
    onclick={onSelect}
    role="row"
    aria-rowindex={rowIndex}
    tabindex="-1"
    aria-selected={isSelected}
  >
    <!-- Status edge bar -->
    <div class="self-stretch rounded-l {segment.status === 'translated' ? 'bg-emerald-500' : segment.status === 'draft' ? 'bg-amber-400' : segment.status === 'needs_review' ? 'bg-blue-400' : 'bg-gray-200 dark:bg-gray-700'}"></div>
    <!-- Left: Seq # and status -->
    <div class="flex flex-col items-center justify-between py-2 px-1 border-r border-gray-100 dark:border-gray-800/60 bg-gray-50/40 dark:bg-gray-900/20 select-none">
      <div class="flex flex-col items-center gap-1">
        <span class="text-xs text-gray-400 dark:text-gray-500 tabular-nums font-mono leading-none">{segment.seq_number}</span>
        <span
          class="w-1.5 h-1.5 rounded-full {segment.status === 'translated' ? 'bg-emerald-500' : segment.status === 'draft' ? 'bg-amber-500' : 'bg-gray-300 dark:bg-gray-700'}"
          title={STATUS_LABELS[segment.status]}
        ></span>
      </div>
      <div class="flex flex-col items-center gap-1.5">
        <!-- DSI Structure Code -->
        <span
          class="text-[9px] font-mono font-bold px-1 py-0.2 rounded border {structure.color}"
          title="Document structure: {structure.label}"
        >
          {structure.code}
        </span>
        <!-- Lock Toggle Button -->
        <button
          type="button"
          onclick={() => onToggleLock(segment.id)}
          class="text-[11px] leading-none transition-transform hover:scale-125 {segment.locked ? 'text-rose-500 font-bold' : 'text-gray-300 dark:text-gray-700 hover:text-rose-400'}"
          title={segment.locked ? 'Locked (Ctrl+L to unlock)' : 'Lock segment (Ctrl+L)'}
        >
          {segment.locked ? '🔒' : '🔓'}
        </button>
        <!-- Bookmark Button -->
        <button aria-label={segment.bookmarked ? 'Bookmarked (Ctrl+B)' : 'Add bookmark (Ctrl+B)'}
          onclick={() => onToggleBookmark(segment.id)}
          class="text-xs leading-none transition-transform hover:scale-125 {segment.bookmarked ? 'text-amber-500' : 'text-gray-300 dark:text-gray-700 hover:text-amber-400'}"
          title={segment.bookmarked ? 'Bookmarked (Ctrl+B)' : 'Add bookmark (Ctrl+B)'}
        >★</button>
        {#if segment.match_percent !== null && segment.match_percent !== undefined}
          <TmMatchBadge pct={segment.match_percent} />
        {/if}
        <!-- Segment metrics & expansion ratio gauge -->
        <div class="mt-1 text-[9px] font-mono text-gray-400 text-center select-none" title="Target: {tgtWords} words, {tgtChars} characters">
          <span>{tgtWords}w/{tgtChars}c</span>
          {#if tgtChars > 0 && Math.abs(expansionRatio) >= 20}
            <div
              class="text-[8px] font-bold px-1 rounded mt-0.5 {expansionRatio > 40 ? 'bg-red-100 text-red-700 dark:bg-red-950/60 dark:text-red-300' : 'bg-amber-100 text-amber-700 dark:bg-amber-950/60 dark:text-amber-300'}"
              title="Character expansion vs source ({expansionRatio > 0 ? '+' : ''}{expansionRatio}%)"
            >
              {expansionRatio > 0 ? '+' : ''}{expansionRatio}%
            </div>
          {/if}
        </div>
      </div>
    </div>

    <!-- Right: Stacked Source + Target -->
    <div class="flex flex-col divide-y divide-gray-100 dark:divide-gray-800/70">
      <!-- Source Panel -->
      <div class="p-3 bg-gray-50/40 dark:bg-gray-900/20 text-gray-800 dark:text-gray-200 select-text leading-relaxed font-editor text-sm"
           dir="auto"
           style="font-size: {$settings.fontSize}px;">
        <div class="text-[10px] font-semibold text-gray-400 dark:text-gray-500 uppercase tracking-wider mb-1 flex items-center justify-between select-none">
          <div class="flex items-center gap-2">
            <span>Source ({$currentProject?.src_lang ?? ''})</span>
            {#if isSelected && !isSourceEditing}
              <button
                type="button"
                onclick={handleToggleSourceEditing}
                class="text-[10px] px-1 py-0.2 rounded bg-gray-200 dark:bg-gray-800 text-gray-600 dark:text-gray-300 hover:text-amber-600 hover:bg-amber-50 dark:hover:bg-amber-950/40 transition-colors cursor-pointer"
                title="Edit Source Text (Alt+F2)"
              >
                ✏️ Edit Source (Alt+F2)
              </button>
            {/if}
          </div>
          {#if sourceTags.length > 0}
            <span class="text-[10px] text-amber-600 dark:text-amber-400 font-mono">🏷️ {sourceTags.length} tags</span>
          {/if}
        </div>
        {#if isSourceEditing}
          <div class="mb-1.5 p-1.5 bg-amber-50 dark:bg-amber-950/40 border border-amber-400 dark:border-amber-600 rounded-md">
            <div class="flex items-center justify-between text-[10px] text-amber-800 dark:text-amber-200 mb-1 font-bold">
              <span>✏️ EDITING SOURCE SEGMENT</span>
              <div class="flex items-center gap-1.5">
                <button
                  type="button"
                  onclick={handleSaveSource}
                  class="px-1.5 py-0.5 rounded bg-emerald-600 text-white text-[10px] hover:bg-emerald-700 cursor-pointer"
                >Save (Enter)</button>
                <button
                  type="button"
                  onclick={handleCancelSource}
                  class="px-1.5 py-0.5 rounded bg-gray-300 dark:bg-gray-700 text-gray-800 dark:text-gray-200 text-[10px] cursor-pointer"
                >Cancel (Esc)</button>
              </div>
            </div>
            <div
              bind:this={sourceEditEl}
              contenteditable="true"
              onkeydown={handleSourceKeydown}
              class="outline-none bg-white dark:bg-gray-800 p-2 rounded text-gray-900 dark:text-white leading-relaxed font-editor text-sm ring-1 ring-amber-400"
              dir="auto"
            >{segment.source_text}</div>
          </div>
        {:else}
          <!-- eslint-disable-next-line svelte/no-at-html-tags -->
          {@html renderSource(segment.source_text, highlightedSourceToken)}
        {/if}
      </div>

      <!-- Target Panel -->
      <div class="p-3 relative flex flex-col justify-start">
        <div class="text-[10px] font-semibold text-gray-400 dark:text-gray-500 uppercase tracking-wider mb-1 flex items-center justify-between select-none">
          <span>Target ({$currentProject?.tgt_lang ?? ''})</span>
          {#if missingTags.length > 0}
            <div class="inline-flex items-center gap-1 text-[10px] font-medium text-amber-700 dark:text-amber-300 bg-amber-50 dark:bg-amber-950/40 px-1.5 py-0.5 rounded border border-amber-300/80">
              <span>⚠️ {missingTags.length} missing tag{missingTags.length > 1 ? 's' : ''}</span>
              <button
                type="button"
                onclick={() => { quickPlaceVisible = !quickPlaceVisible; activeQuickPlaceIndex = 0; }}
                class="hover:underline font-bold text-brand-600 dark:text-brand-400 cursor-pointer ml-1"
              >
                QuickPlace (Ctrl+,)
              </button>
            </div>
          {/if}
        </div>

        {#if segment.locked}
          <!-- Locked Segment Banner -->
          <div class="mb-2 px-2.5 py-1 rounded bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-900 text-xs text-rose-700 dark:text-rose-300 flex items-center justify-between select-none">
            <span class="flex items-center gap-1 font-medium"><span>🔒</span> <span>Segment is locked</span></span>
            <button
              type="button"
              onclick={() => onToggleLock(segment.id)}
              class="hover:underline font-semibold cursor-pointer text-[11px]"
            >
              Unlock (Ctrl+L)
            </button>
          </div>
        {/if}

        {#if isRevised}
          <!-- Revision Diff Banner -->
          <div class="mb-2 p-2 rounded bg-amber-50/90 dark:bg-amber-950/50 border border-amber-300 dark:border-amber-700 text-xs flex items-center justify-between">
            <div class="flex items-center gap-2 truncate">
              <span class="font-bold text-amber-800 dark:text-amber-200">📝 Revision:</span>
              <span class="text-red-500 line-through truncate">{originalTargetText}</span>
              <span>➔</span>
              <span class="text-emerald-600 dark:text-emerald-400 font-medium truncate">{segment.target_text}</span>
            </div>
            <div class="flex items-center gap-1 shrink-0 ml-2">
              <button
                onclick={handleAcceptRevision}
                class="px-2 py-0.5 bg-emerald-600 hover:bg-emerald-700 text-white rounded text-[10px] font-bold"
                title="Accept Revision (Ctrl+Shift+A)"
              >Accept</button>
              <button
                onclick={handleRejectRevision}
                class="px-2 py-0.5 bg-red-600 hover:bg-red-700 text-white rounded text-[10px] font-bold"
                title="Reject Revision (Ctrl+Shift+R)"
              >Reject</button>
            </div>
          </div>
        {/if}

        <!-- Auto-Patch Pill Banner -->
        {#if isSelected && autoPatchSuggestion && (!segment.target_text || !segment.target_text.trim())}
          <div class="mb-2 px-2.5 py-1 rounded-md bg-amber-50 dark:bg-amber-950/60 border border-amber-300 dark:border-amber-700 flex items-center justify-between text-xs animate-in fade-in z-20 shadow-xs">
            <div class="flex items-center gap-1.5 min-w-0 pr-2">
              <span class="text-amber-700 dark:text-amber-300 font-bold shrink-0">⚡ Auto-Patch:</span>
              <span class="font-editor text-gray-900 dark:text-gray-100 font-semibold truncate" dir="auto">{autoPatchSuggestion}</span>
            </div>
            <button
              type="button"
              onmousedown={(e) => { e.preventDefault(); onApplyAutoPatch(autoPatchSuggestion); }}
              class="px-2 py-0.5 rounded bg-amber-500 hover:bg-amber-600 text-white font-medium text-[11px] shrink-0 transition-colors flex items-center gap-1 cursor-pointer"
            >
              <span>Apply</span>
              <kbd class="text-[9px] bg-amber-600 px-1 rounded font-mono">Tab</kbd>
            </button>
          </div>
        {/if}

        <QuickPlaceOmnibar
          visible={quickPlaceVisible}
          sourceText={segment.source_text}
          targetText={segment.target_text}
          selectedIndex={activeQuickPlaceIndex}
          onSelect={handleInsertPlaceable}
          onHighlightSource={(tok) => highlightedSourceToken = tok}
          onClose={() => { quickPlaceVisible = false; highlightedSourceToken = null; }}
        />

        {#if isSelected}
          <AutoSuggestPopup
            visible={suggestVisible}
            suggestions={activeSuggestions}
            selectedIndex={activeSuggestIndex}
            onSelect={handleApplySuggestion}
          />
        {/if}

        <!-- Target Contenteditable -->
        <div
          bind:this={targetEl}
          contenteditable={!segment.locked}
          tabindex="0"
          oninput={handleInput}
          onkeydown={handleKeydown}
          onfocus={onSelect}
          onmouseup={handleSelectionCheck}
          onkeyup={handleSelectionCheck}
          class="outline-none flex-1 min-h-[2.5rem] leading-relaxed
                 text-gray-900 dark:text-white caret-brand-500 font-editor
                 {segment.locked ? 'bg-gray-50/60 dark:bg-gray-900/40 text-gray-400 cursor-not-allowed select-none' : ''}
                 {isSelected ? '' : 'text-gray-600 dark:text-gray-300'}"
          dir="auto"
          lang={$currentProject?.tgt_lang || 'en'}
          spellcheck={$settings.spellcheckEnabled && !segment.locked}
          role="textbox"
          aria-multiline="true"
          aria-label="Translation target for segment {segment.seq_number}"
          style="font-size: {targetFontSize}px;"
        ></div>
      </div>
    </div>
  </div>
{:else}
  <!-- Side-by-Side (and Zen Mode) Layout -->
  <div
    class="grid min-h-[3rem] border-b border-gray-100 dark:border-gray-800/80
           transition-colors cursor-pointer relative group {rowBg} {zenClass}"
    style="grid-template-columns: 3px 3.25rem minmax(200px, {$settings.sourceRatio}%) minmax(200px, 1fr);"
    onclick={onSelect}
    role="row"
    aria-rowindex={rowIndex}
    tabindex="-1"
    aria-selected={isSelected}
  >
    <!-- Status edge bar -->
    <div class="self-stretch rounded-l {segment.status === 'translated' ? 'bg-emerald-500' : segment.status === 'draft' ? 'bg-amber-400' : segment.status === 'needs_review' ? 'bg-blue-400' : 'bg-gray-200 dark:bg-gray-700'}"></div>
    <!-- Left: Seq #, status, DSI, and lock -->
    <div class="flex flex-col items-center justify-between py-2 px-1 shrink-0 select-none border-r border-gray-100 dark:border-gray-800/60 bg-gray-50/40 dark:bg-gray-900/20">
      <div class="flex flex-col items-center gap-1">
        <span class="text-xs text-gray-400 dark:text-gray-500 tabular-nums font-mono leading-none">{segment.seq_number}</span>
        <span
          class="w-1.5 h-1.5 rounded-full {segment.status === 'translated' ? 'bg-emerald-500' : segment.status === 'draft' ? 'bg-amber-500' : 'bg-gray-300 dark:bg-gray-700'}"
          title={STATUS_LABELS[segment.status]}
        ></span>
      </div>
      <div class="flex flex-col items-center gap-1.5">
        <!-- DSI Structure Code -->
        <span
          class="text-[9px] font-mono font-bold px-1 py-0.2 rounded border {structure.color}"
          title="Document structure: {structure.label}"
        >
          {structure.code}
        </span>
        <!-- Lock Toggle Button -->
        <button
          type="button"
          onclick={() => onToggleLock(segment.id)}
          class="text-[11px] leading-none transition-transform hover:scale-125 {segment.locked ? 'text-rose-500 font-bold' : 'text-gray-300 dark:text-gray-700 hover:text-rose-400'}"
          title={segment.locked ? 'Locked (Ctrl+L to unlock)' : 'Lock segment (Ctrl+L)'}
        >
          {segment.locked ? '🔒' : '🔓'}
        </button>
        <!-- Bookmark Button -->
        <button aria-label={segment.bookmarked ? 'Bookmarked (Ctrl+B)' : 'Add bookmark (Ctrl+B)'}
          onclick={() => onToggleBookmark(segment.id)}
          class="text-xs leading-none transition-transform hover:scale-125 {segment.bookmarked ? 'text-amber-500' : 'text-gray-300 dark:text-gray-700 hover:text-amber-400'}"
          title={segment.bookmarked ? 'Bookmarked (Ctrl+B)' : 'Add bookmark (Ctrl+B)'}
        >★</button>
        {#if segment.match_percent !== null && segment.match_percent !== undefined}
          <TmMatchBadge pct={segment.match_percent} />
        {/if}
        <!-- Segment metrics & expansion ratio gauge -->
        <div class="mt-1 text-[9px] font-mono text-gray-400 text-center select-none" title="Target: {tgtWords} words, {tgtChars} characters">
          <span>{tgtWords}w/{tgtChars}c</span>
          {#if tgtChars > 0 && Math.abs(expansionRatio) >= 20}
            <div
              class="text-[8px] font-bold px-1 rounded mt-0.5 {expansionRatio > 40 ? 'bg-red-100 text-red-700 dark:bg-red-950/60 dark:text-red-300' : 'bg-amber-100 text-amber-700 dark:bg-amber-950/60 dark:text-amber-300'}"
              title="Character expansion vs source ({expansionRatio > 0 ? '+' : ''}{expansionRatio}%)"
            >
              {expansionRatio > 0 ? '+' : ''}{expansionRatio}%
            </div>
          {/if}
        </div>
      </div>
    </div>

    <!-- Source Column -->
    <div
      class="py-2.5 px-3.5 text-sm text-gray-800 dark:text-gray-200 editor-text
             border-r border-gray-100 dark:border-gray-800 select-text leading-relaxed font-editor relative flex flex-col justify-between"
      dir="auto"
      style="font-size: {$settings.fontSize}px;"
    >
      <div>
        {#if isSourceEditing}
          <div class="mb-1.5 p-1.5 bg-amber-50 dark:bg-amber-950/40 border border-amber-400 dark:border-amber-600 rounded-md">
            <div class="flex items-center justify-between text-[10px] text-amber-800 dark:text-amber-200 mb-1 font-bold">
              <span>✏️ EDITING SOURCE</span>
              <div class="flex items-center gap-1">
                <button
                  type="button"
                  onclick={handleSaveSource}
                  class="px-1.5 py-0.5 rounded bg-emerald-600 text-white text-[10px] hover:bg-emerald-700 cursor-pointer"
                >Save (Enter)</button>
                <button
                  type="button"
                  onclick={handleCancelSource}
                  class="px-1.5 py-0.5 rounded bg-gray-300 dark:bg-gray-700 text-gray-800 dark:text-gray-200 text-[10px] cursor-pointer"
                >Cancel (Esc)</button>
              </div>
            </div>
            <div
              bind:this={sourceEditEl}
              contenteditable="true"
              onkeydown={handleSourceKeydown}
              class="outline-none bg-white dark:bg-gray-800 p-2 rounded text-gray-900 dark:text-white leading-relaxed font-editor text-sm ring-1 ring-amber-400"
              dir="auto"
            >{segment.source_text}</div>
          </div>
        {:else}
          <!-- eslint-disable-next-line svelte/no-at-html-tags -->
          {@html renderSource(segment.source_text, highlightedSourceToken)}
        {/if}
      </div>

      {#if isSelected && !isSourceEditing}
        <div class="mt-1 pt-1 border-t border-gray-100 dark:border-gray-800/60 flex items-center justify-between text-[10px] text-gray-400 select-none">
          <button
            type="button"
            onclick={handleToggleSourceEditing}
            class="hover:text-amber-600 dark:hover:text-amber-400 transition-colors flex items-center gap-1 cursor-pointer font-medium"
            title="Edit Source Text (Alt+F2)"
          >
            <span>✏️ Edit Source</span>
            <kbd class="text-[9px] font-mono px-1 rounded bg-gray-200 dark:bg-gray-700">Alt+F2</kbd>
          </button>
          {#if sourceTags.length > 0}
            <span class="font-mono text-amber-600 dark:text-amber-400">🏷️ {sourceTags.length} tags</span>
          {/if}
        </div>
      {/if}
    </div>

    <!-- Target Column (editable) -->
    <!-- svelte-ignore a11y-interactive-supports-focus -->
    <div class="relative flex flex-col justify-start">
      {#if segment.locked}
        <!-- Locked Segment Banner -->
        <div class="m-1.5 px-2.5 py-1 rounded bg-rose-50 dark:bg-rose-950/40 border border-rose-200 dark:border-rose-900 text-xs text-rose-700 dark:text-rose-300 flex items-center justify-between select-none">
          <span class="flex items-center gap-1 font-medium"><span>🔒</span> <span>Segment is locked</span></span>
          <button
            type="button"
            onclick={() => onToggleLock(segment.id)}
            class="hover:underline font-semibold cursor-pointer text-[11px]"
          >
            Unlock (Ctrl+L)
          </button>
        </div>
      {/if}

      {#if isRevised}
        <!-- Revision Diff Banner -->
        <div class="m-1.5 p-2 rounded bg-amber-50/90 dark:bg-amber-950/50 border border-amber-300 dark:border-amber-700 text-xs flex items-center justify-between">
          <div class="flex items-center gap-2 truncate">
            <span class="font-bold text-amber-800 dark:text-amber-200">📝 Revision:</span>
            <span class="text-red-500 line-through truncate">{originalTargetText}</span>
            <span>➔</span>
            <span class="text-emerald-600 dark:text-emerald-400 font-medium truncate">{segment.target_text}</span>
          </div>
          <div class="flex items-center gap-1 shrink-0 ml-2">
            <button
              onclick={handleAcceptRevision}
              class="px-2 py-0.5 bg-emerald-600 hover:bg-emerald-700 text-white rounded text-[10px] font-bold"
              title="Accept Revision (Ctrl+Shift+A)"
            >Accept</button>
            <button
              onclick={handleRejectRevision}
              class="px-2 py-0.5 bg-red-600 hover:bg-red-700 text-white rounded text-[10px] font-bold"
              title="Reject Revision (Ctrl+Shift+R)"
            >Reject</button>
          </div>
        </div>
      {/if}

      <!-- Missing Tags Pill (when tags present in source but missing from target) -->
      {#if missingTags.length > 0}
        <div class="mx-3 mt-1.5 flex items-center justify-between text-[11px] bg-amber-50/70 dark:bg-amber-950/30 text-amber-800 dark:text-amber-300 px-2 py-0.5 rounded border border-amber-300/60 select-none">
          <span>⚠️ {missingTags.length} missing tag{missingTags.length > 1 ? 's' : ''}: {missingTags.join(' ')}</span>
          <div class="flex items-center gap-2">
            <button
              type="button"
              onclick={() => { quickPlaceVisible = !quickPlaceVisible; activeQuickPlaceIndex = 0; }}
              class="hover:underline font-bold text-brand-600 dark:text-brand-400 cursor-pointer"
            >
              QuickPlace (Ctrl+,)
            </button>
            <span>•</span>
            <button
              type="button"
              onclick={handleInsertAllMissingTags}
              class="hover:underline text-gray-600 dark:text-gray-400 cursor-pointer"
            >
              + All
            </button>
          </div>
        </div>
      {/if}

      <!-- Auto-Patch Pill Banner -->
      {#if isSelected && autoPatchSuggestion && (!segment.target_text || !segment.target_text.trim())}
        <div class="m-1.5 px-2.5 py-1 rounded-md bg-amber-50 dark:bg-amber-950/60 border border-amber-300 dark:border-amber-700 flex items-center justify-between text-xs animate-in fade-in z-20 shadow-xs">
          <div class="flex items-center gap-1.5 min-w-0 pr-2">
            <span class="text-amber-700 dark:text-amber-300 font-bold shrink-0">⚡ Auto-Patch:</span>
            <span class="font-editor text-gray-900 dark:text-gray-100 font-semibold truncate" dir="auto">{autoPatchSuggestion}</span>
          </div>
          <button
            type="button"
            onmousedown={(e) => { e.preventDefault(); onApplyAutoPatch(autoPatchSuggestion); }}
            class="px-2 py-0.5 rounded bg-amber-500 hover:bg-amber-600 text-white font-medium text-[11px] shrink-0 transition-colors flex items-center gap-1 cursor-pointer"
          >
            <span>Apply</span>
            <kbd class="text-[9px] bg-amber-600 px-1 py-0.2 rounded font-mono">Tab</kbd>
          </button>
        </div>
      {/if}

      <QuickPlaceOmnibar
        visible={quickPlaceVisible}
        sourceText={segment.source_text}
        targetText={segment.target_text}
        selectedIndex={activeQuickPlaceIndex}
        onSelect={handleInsertPlaceable}
        onHighlightSource={(tok) => highlightedSourceToken = tok}
        onClose={() => { quickPlaceVisible = false; highlightedSourceToken = null; }}
      />

      {#if isSelected}
        <AutoSuggestPopup
          visible={suggestVisible}
          suggestions={activeSuggestions}
          selectedIndex={activeSuggestIndex}
          onSelect={handleApplySuggestion}
        />
      {/if}

      <!-- Target Contenteditable -->
      <div
        bind:this={targetEl}
        contenteditable={!segment.locked}
        tabindex="0"
        oninput={handleInput}
        onkeydown={handleKeydown}
        onfocus={onSelect}
        onmouseup={handleSelectionCheck}
        onkeyup={handleSelectionCheck}
        class="py-2.5 px-3.5 text-sm editor-text outline-none flex-1 min-h-[3rem] leading-relaxed
               text-gray-900 dark:text-white caret-brand-500 font-editor
               {segment.locked ? 'bg-gray-50/60 dark:bg-gray-900/40 text-gray-400 cursor-not-allowed select-none' : ''}
               {isSelected ? '' : 'text-gray-600 dark:text-gray-300'}"
        dir="auto"
        lang={$currentProject?.tgt_lang || 'en'}
        spellcheck={$settings.spellcheckEnabled && !segment.locked}
        role="textbox"
        aria-multiline="true"
        aria-label="Translation target for segment {segment.seq_number}"
        style="font-size: {targetFontSize}px;"
      ></div>

      <!-- Inline AI Refinement HUD on Text Selection -->
      {#if isAiRefineMenuOpen && selectedTargetText}
        <div class="absolute -top-7 left-3 z-30 flex items-center gap-1 bg-white dark:bg-gray-900 border border-purple-200 dark:border-purple-800 rounded-lg shadow-xl px-2 py-1 text-[11px] animate-in fade-in zoom-in-95">
          <span class="text-purple-600 dark:text-purple-400 font-bold flex items-center gap-1">✨ AI:</span>
          <button
            type="button"
            onclick={() => handleTriggerAiAction('rephrase')}
            class="px-1.5 py-0.5 rounded hover:bg-purple-50 dark:hover:bg-purple-950/50 text-gray-700 dark:text-gray-300"
            title="Rephrase & Polish selected text"
          >Polish</button>
          <button
            type="button"
            onclick={() => handleTriggerAiAction('condense')}
            class="px-1.5 py-0.5 rounded hover:bg-purple-50 dark:hover:bg-purple-950/50 text-gray-700 dark:text-gray-300"
            title="Shorten / Condense selected text"
          >Condense</button>
          <button
            type="button"
            onclick={() => handleTriggerAiAction('tone_formal')}
            class="px-1.5 py-0.5 rounded hover:bg-purple-50 dark:hover:bg-purple-950/50 text-gray-700 dark:text-gray-300"
            title="Make formal"
          >Formal</button>
          <button
            type="button"
            onclick={() => handleTriggerAiAction('tone_casual')}
            class="px-1.5 py-0.5 rounded hover:bg-purple-50 dark:hover:bg-purple-950/50 text-gray-700 dark:text-gray-300"
            title="Make casual"
          >Casual</button>
          <button
            type="button"
            onclick={() => handleTriggerAiAction('fix_grammar')}
            class="px-1.5 py-0.5 rounded hover:bg-purple-50 dark:hover:bg-purple-950/50 text-gray-700 dark:text-gray-300"
            title="Check and fix grammar"
          >Grammar</button>
        </div>
      {/if}

      <!-- Elegant floating quick-action hover buttons on top right -->
      <div class="absolute top-1.5 right-2 flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity bg-white/95 dark:bg-gray-900/95 backdrop-blur-xs px-1 py-0.5 rounded-md border border-gray-200 dark:border-gray-700/80 shadow-xs z-10 {isSelected ? 'opacity-90' : ''}">
        <button aria-label="Copy Source (Alt+Ins)"
          type="button"
          onclick={handleCopySource}
          class="p-1 rounded text-xs text-gray-500 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
          title="Copy Source (Alt+Ins)"
        >📋</button>
        <button aria-label="Clear Target"
          type="button"
          onclick={handleClearTarget}
          class="p-1 rounded text-xs text-gray-500 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
          title="Clear Target"
        >🧹</button>
        <button aria-label="Split Segment (Ctrl+Alt+S)"
          type="button"
          onclick={() => onSplit(segment.id)}
          class="p-1 rounded text-xs text-gray-500 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
          title="Split Segment (Ctrl+Alt+S)"
        >✂</button>
        {#if canMergeNext}
          <button aria-label="Merge Next (Ctrl+J)"
            type="button"
            onclick={() => onMergeNext(segment.id)}
            class="p-1 rounded text-xs text-gray-500 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
            title="Merge Next (Ctrl+J)"
          >⤹</button>
        {/if}
        {#if canMergePrev}
          <button aria-label="Merge Prev (Ctrl+Shift+J)"
            type="button"
            onclick={() => onMergePrev(segment.id)}
            class="p-1 rounded text-xs text-gray-500 hover:text-gray-900 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
            title="Merge Prev (Ctrl+Shift+J)"
          >⤸</button>
        {/if}
      </div>
    </div>
  </div>
{/if}
