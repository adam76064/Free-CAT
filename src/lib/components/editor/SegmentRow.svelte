<script lang="ts">
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
  import QuickTagPopup from './QuickTagPopup.svelte';
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
  export let onAiRefineAction: (action: string, selectedText: string) => void = () => {};

  let targetEl: HTMLDivElement;
  let suggestVisible = false;
  let activeSuggestIndex = 0;

  // QuickTag HUD state
  let tagPopupVisible = false;
  let activeTagIndex = 0;

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

  // Extract tags: e.g. {1}, {2}, {/1}, etc.
  function extractTags(text: string): string[] {
    if (!text) return [];
    const matches = text.match(/\{[^}]+\}/g);
    return matches ? Array.from(new Set(matches)) : [];
  }

  $: sourceTags = extractTags(segment.source_text);
  $: targetTags = extractTags(segment.target_text);
  $: missingTags = sourceTags.filter(t => !targetTags.includes(t));

  function handleInsertTag(tag: string) {
    if (!targetEl) return;
    targetEl.focus();
    const sel = window.getSelection();
    if (sel && sel.rangeCount > 0 && targetEl.contains(sel.anchorNode)) {
      const range = sel.getRangeAt(0);
      range.deleteContents();
      const node = document.createTextNode(tag);
      range.insertNode(node);
      range.setStartAfter(node);
      range.collapse(true);
      sel.removeAllRanges();
      sel.addRange(range);
    } else {
      const current = targetEl.textContent || '';
      targetEl.textContent = current ? `${current} ${tag}` : tag;
      moveCursorToEnd();
    }
    const newText = targetEl.textContent || '';
    onTargetChange(segment.id, newText);
    tagPopupVisible = false;
  }

  function handleInsertAllMissingTags() {
    if (missingTags.length === 0) return;
    const current = targetEl?.textContent || '';
    const toAppend = (current.endsWith(' ') || !current ? '' : ' ') + missingTags.join(' ');
    const newText = current + toAppend;
    if (targetEl) targetEl.textContent = newText;
    onTargetChange(segment.id, newText);
    tagPopupVisible = false;
  }

  function handleCopySource() {
    onTargetChange(segment.id, segment.source_text);
    if (targetEl) targetEl.textContent = segment.source_text;
  }

  function handleClearTarget() {
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
    // QuickTag HUD: Ctrl+, or Cmd+,
    if ((e.ctrlKey || e.metaKey) && e.key === ',') {
      e.preventDefault();
      if (missingTags.length > 0) {
        tagPopupVisible = !tagPopupVisible;
        activeTagIndex = 0;
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
      if (suggestVisible || tagPopupVisible || isAiRefineMenuOpen) {
        e.preventDefault();
        suggestVisible = false;
        tagPopupVisible = false;
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
      tagPopupVisible = false;
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

  /** Render source text with stylish non-breakable tag pills */
  function renderSource(text: string): string {
    return text.replace(
      /\{([^}]+)\}/g,
      '<span class="inline-flex items-center gap-0.5 font-mono text-[11px] font-semibold bg-amber-100 text-amber-900 dark:bg-amber-950/70 dark:text-amber-300 border border-amber-300/80 dark:border-amber-700/80 px-1.5 py-0.2 rounded select-none mx-0.5 tracking-tight cursor-default" title="Tag placeholder {$1}">🏷️$1</span>'
    );
  }

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
    class="grid grid-cols-[2.75rem_1fr] min-h-[4.5rem] border-b border-gray-100 dark:border-gray-800/80
           transition-colors cursor-pointer relative group {rowBg} {zenClass}"
    on:click={onSelect}
    role="row"
    tabindex="-1"
    aria-selected={isSelected}
  >
    <!-- Left: Seq # and status -->
    <div class="flex flex-col items-center justify-between py-2 px-1 border-r border-gray-100 dark:border-gray-800/60 bg-gray-50/40 dark:bg-gray-900/20 select-none">
      <div class="flex flex-col items-center gap-1">
        <span class="text-xs text-gray-400 dark:text-gray-500 tabular-nums font-mono leading-none">{segment.seq_number}</span>
        <span
          class="w-1.5 h-1.5 rounded-full {segment.status === 'translated' ? 'bg-emerald-500' : segment.status === 'draft' ? 'bg-amber-500' : 'bg-gray-300 dark:bg-gray-700'}"
          title={STATUS_LABELS[segment.status]}
        ></span>
      </div>
      <div class="flex flex-col items-center gap-1">
        <button
          on:click|stopPropagation={() => onToggleBookmark(segment.id)}
          class="text-xs leading-none transition-transform hover:scale-125 {segment.bookmarked ? 'text-amber-500' : 'text-gray-300 dark:text-gray-700 hover:text-amber-400'}"
          title={segment.bookmarked ? 'Bookmarked (Ctrl+B)' : 'Add bookmark (Ctrl+B)'}
        >★</button>
        {#if segment.match_percent !== null && segment.match_percent !== undefined}
          <TmMatchBadge pct={segment.match_percent} />
        {/if}
      </div>
    </div>

    <!-- Right: Stacked Source + Target -->
    <div class="flex flex-col divide-y divide-gray-100 dark:divide-gray-800/70">
      <!-- Source Panel -->
      <div class="p-3 bg-gray-50/40 dark:bg-gray-900/20 text-gray-800 dark:text-gray-200 select-text leading-relaxed font-editor text-sm"
           dir="auto"
           style="font-size: {$settings.fontSize}px;">
        <div class="text-[10px] font-semibold text-gray-400 dark:text-gray-500 uppercase tracking-wider mb-1 flex items-center justify-between select-none">
          <span>Source ({$currentProject?.src_lang ?? ''})</span>
          {#if sourceTags.length > 0}
            <span class="text-[10px] text-amber-600 dark:text-amber-400 font-mono">🏷️ {sourceTags.length} tags</span>
          {/if}
        </div>
        <!-- eslint-disable-next-line svelte/no-at-html-tags -->
        {@html renderSource(segment.source_text)}
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
                on:click|stopPropagation={() => { tagPopupVisible = !tagPopupVisible; activeTagIndex = 0; }}
                class="hover:underline font-bold text-brand-600 dark:text-brand-400 cursor-pointer ml-1"
              >
                Insert (Ctrl+,)
              </button>
            </div>
          {/if}
        </div>

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
                on:click|stopPropagation={handleAcceptRevision}
                class="px-2 py-0.5 bg-emerald-600 hover:bg-emerald-700 text-white rounded text-[10px] font-bold"
                title="Accept Revision (Ctrl+Shift+A)"
              >Accept</button>
              <button
                on:click|stopPropagation={handleRejectRevision}
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
              on:mousedown|preventDefault={() => onApplyAutoPatch(autoPatchSuggestion)}
              class="px-2 py-0.5 rounded bg-amber-500 hover:bg-amber-600 text-white font-medium text-[11px] shrink-0 transition-colors flex items-center gap-1 cursor-pointer"
            >
              <span>Apply</span>
              <kbd class="text-[9px] bg-amber-600 px-1 rounded font-mono">Tab</kbd>
            </button>
          </div>
        {/if}

        <QuickTagPopup
          visible={tagPopupVisible}
          tags={missingTags}
          selectedIndex={activeTagIndex}
          onSelect={handleInsertTag}
          onClose={() => tagPopupVisible = false}
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
          contenteditable="true"
          tabindex="0"
          on:input={handleInput}
          on:keydown={handleKeydown}
          on:focus={onSelect}
          on:mouseup={handleSelectionCheck}
          on:keyup={handleSelectionCheck}
          class="outline-none flex-1 min-h-[2.5rem] leading-relaxed
                 text-gray-900 dark:text-white caret-brand-500 font-editor
                 {isSelected ? '' : 'text-gray-600 dark:text-gray-300'}"
          dir="auto"
          lang={$currentProject?.tgt_lang || 'en'}
          spellcheck={$settings.spellcheckEnabled}
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
    style="grid-template-columns: 2.75rem minmax(200px, {$settings.sourceRatio}%) minmax(200px, 1fr);"
    on:click={onSelect}
    role="row"
    tabindex="-1"
    aria-selected={isSelected}
  >
    <!-- Left: Seq # and status -->
    <div class="flex items-center justify-between px-2 py-2 shrink-0 select-none border-r border-gray-100 dark:border-gray-800/60 bg-gray-50/40 dark:bg-gray-900/20">
      <div class="flex flex-col items-center gap-1">
        <span class="text-xs text-gray-400 dark:text-gray-500 tabular-nums font-mono leading-none">{segment.seq_number}</span>
        <span
          class="w-1.5 h-1.5 rounded-full {segment.status === 'translated' ? 'bg-emerald-500' : segment.status === 'draft' ? 'bg-amber-500' : 'bg-gray-300 dark:bg-gray-700'}"
          title={STATUS_LABELS[segment.status]}
        ></span>
      </div>
      <div class="flex flex-col items-center gap-1">
        <button
          on:click|stopPropagation={() => onToggleBookmark(segment.id)}
          class="text-xs leading-none transition-transform hover:scale-125 {segment.bookmarked ? 'text-amber-500' : 'text-gray-300 dark:text-gray-700 hover:text-amber-400'}"
          title={segment.bookmarked ? 'Bookmarked (Ctrl+B)' : 'Add bookmark (Ctrl+B)'}
        >★</button>
        {#if segment.match_percent !== null && segment.match_percent !== undefined}
          <TmMatchBadge pct={segment.match_percent} />
        {/if}
      </div>
    </div>

    <!-- Source Column (read-only) -->
    <div
      class="py-2.5 px-3.5 text-sm text-gray-800 dark:text-gray-200 editor-text
             border-r border-gray-100 dark:border-gray-800 select-text leading-relaxed font-editor"
      dir="auto"
      style="font-size: {$settings.fontSize}px;"
    >
      <!-- eslint-disable-next-line svelte/no-at-html-tags -->
      {@html renderSource(segment.source_text)}
    </div>

    <!-- Target Column (editable) -->
    <!-- svelte-ignore a11y-interactive-supports-focus -->
    <div class="relative flex flex-col justify-start">
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
              on:click|stopPropagation={handleAcceptRevision}
              class="px-2 py-0.5 bg-emerald-600 hover:bg-emerald-700 text-white rounded text-[10px] font-bold"
              title="Accept Revision (Ctrl+Shift+A)"
            >Accept</button>
            <button
              on:click|stopPropagation={handleRejectRevision}
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
              on:click|stopPropagation={() => { tagPopupVisible = !tagPopupVisible; activeTagIndex = 0; }}
              class="hover:underline font-bold text-brand-600 dark:text-brand-400 cursor-pointer"
            >
              Insert (Ctrl+,)
            </button>
            <span>•</span>
            <button
              type="button"
              on:click|stopPropagation={handleInsertAllMissingTags}
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
            on:mousedown|preventDefault={() => onApplyAutoPatch(autoPatchSuggestion)}
            class="px-2 py-0.5 rounded bg-amber-500 hover:bg-amber-600 text-white font-medium text-[11px] shrink-0 transition-colors flex items-center gap-1 cursor-pointer"
          >
            <span>Apply</span>
            <kbd class="text-[9px] bg-amber-600 px-1 py-0.2 rounded font-mono">Tab</kbd>
          </button>
        </div>
      {/if}

      <QuickTagPopup
        visible={tagPopupVisible}
        tags={missingTags}
        selectedIndex={activeTagIndex}
        onSelect={handleInsertTag}
        onClose={() => tagPopupVisible = false}
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
        contenteditable="true"
        tabindex="0"
        on:input={handleInput}
        on:keydown={handleKeydown}
        on:focus={onSelect}
        on:mouseup={handleSelectionCheck}
        on:keyup={handleSelectionCheck}
        class="py-2.5 px-3.5 text-sm editor-text outline-none flex-1 min-h-[3rem] leading-relaxed
               text-gray-900 dark:text-white caret-brand-500 font-editor
               {isSelected ? '' : 'text-gray-600 dark:text-gray-300'}"
        dir="auto"
        lang={$currentProject?.tgt_lang || 'en'}
        spellcheck={$settings.spellcheckEnabled}
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
            on:click|stopPropagation={() => handleTriggerAiAction('rephrase')}
            class="px-1.5 py-0.5 rounded hover:bg-purple-50 dark:hover:bg-purple-950/50 text-gray-700 dark:text-gray-300"
            title="Rephrase & Polish selected text"
          >Polish</button>
          <button
            type="button"
            on:click|stopPropagation={() => handleTriggerAiAction('condense')}
            class="px-1.5 py-0.5 rounded hover:bg-purple-50 dark:hover:bg-purple-950/50 text-gray-700 dark:text-gray-300"
            title="Shorten / Condense selected text"
          >Condense</button>
          <button
            type="button"
            on:click|stopPropagation={() => handleTriggerAiAction('tone_formal')}
            class="px-1.5 py-0.5 rounded hover:bg-purple-50 dark:hover:bg-purple-950/50 text-gray-700 dark:text-gray-300"
            title="Make formal"
          >Formal</button>
          <button
            type="button"
            on:click|stopPropagation={() => handleTriggerAiAction('tone_casual')}
            class="px-1.5 py-0.5 rounded hover:bg-purple-50 dark:hover:bg-purple-950/50 text-gray-700 dark:text-gray-300"
            title="Make casual"
          >Casual</button>
          <button
            type="button"
            on:click|stopPropagation={() => handleTriggerAiAction('fix_grammar')}
            class="px-1.5 py-0.5 rounded hover:bg-purple-50 dark:hover:bg-purple-950/50 text-gray-700 dark:text-gray-300"
            title="Check and fix grammar"
          >Grammar</button>
        </div>
      {/if}

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
{/if}
