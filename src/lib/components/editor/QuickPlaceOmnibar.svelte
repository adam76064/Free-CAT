<script context="module" lang="ts">
  export interface PlaceableItem {
    id: string;
    type: 'tag' | 'number' | 'date' | 'url' | 'code';
    label: string;
    typeLabel: string;
    badgeColor: string;
    sourceText: string;
    insertText: string;
    isMissing?: boolean;
  }
</script>

<script lang="ts">
  /**
   * QuickPlaceOmnibar.svelte — Trados-style QuickPlace placeables insertion HUD (Ctrl+,)
   * Recognizes:
   * - Inline tags (formatting, structural)
   * - Numbers, decimals, percentages, currency
   * - Dates & times
   * - URLs and email addresses
   * - Alphanumeric codes, model numbers, variables
   *
   * Ergonomics:
   * - ArrowUp/ArrowDown navigation
   * - Live source token spotlighting
   * - Enter / Tab insertion at caret
   * - Esc to dismiss
   */
  import { onMount, onDestroy } from 'svelte';

  export let visible = false;
  export let sourceText: string = '';
  export let targetText: string = '';
  export let selectedIndex = 0;
  export let onSelect: (insertText: string) => void;
  export let onHighlightSource: (token: string | null) => void = () => {};
  export let onClose: () => void = () => {};

  let items: PlaceableItem[] = [];

  function extractPlaceables(src: string, tgt: string): PlaceableItem[] {
    if (!src) return [];
    const list: PlaceableItem[] = [];
    const seen = new Set<string>();

    // 1. Tags: {1}, {/1}, <b>, </b>, etc.
    const tagMatches = src.match(/\{[^}]+\}|<[^>]+>/g) || [];
    for (const tag of tagMatches) {
      if (!seen.has(tag)) {
        seen.add(tag);
        const isMissing = !tgt.includes(tag);
        list.push({
          id: `tag-${tag}`,
          type: 'tag',
          label: tag,
          typeLabel: isMissing ? '⚠️ Missing Tag' : '🏷️ Tag',
          badgeColor: isMissing
            ? 'bg-amber-100 text-amber-900 dark:bg-amber-900/60 dark:text-amber-200 border-amber-300'
            : 'bg-blue-100 text-blue-900 dark:bg-blue-900/60 dark:text-blue-200 border-blue-300',
          sourceText: tag,
          insertText: tag,
          isMissing,
        });
      }
    }

    // 2. URLs and Emails
    const urlMatches = src.match(/https?:\/\/[^\s<>"{}]+|[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}/g) || [];
    for (const url of urlMatches) {
      if (!seen.has(url)) {
        seen.add(url);
        list.push({
          id: `url-${url}`,
          type: 'url',
          label: url,
          typeLabel: '🔗 Link',
          badgeColor: 'bg-cyan-100 text-cyan-900 dark:bg-cyan-900/60 dark:text-cyan-200 border-cyan-300',
          sourceText: url,
          insertText: url,
        });
      }
    }

    // 3. Dates and Times
    const dateMatches = src.match(/\b\d{1,4}[-/.]\d{1,2}[-/.]\d{1,4}\b|\b\d{1,2}:\d{2}(?::\d{2})?(?:\s*[AaPp][Mm])?\b/g) || [];
    for (const d of dateMatches) {
      if (!seen.has(d)) {
        seen.add(d);
        list.push({
          id: `date-${d}`,
          type: 'date',
          label: d,
          typeLabel: '📅 Date/Time',
          badgeColor: 'bg-emerald-100 text-emerald-900 dark:bg-emerald-900/60 dark:text-emerald-200 border-emerald-300',
          sourceText: d,
          insertText: d,
        });
      }
    }

    // 4. Alphanumeric codes / Model numbers / Variables (e.g. SKU_4901, UTF-8, v2.1)
    const codeMatches = src.match(/\b(?=[A-Za-z0-9_-]*[0-9])(?=[A-Za-z0-9_-]*[A-Za-z])[A-Za-z0-9_-]{3,}\b/g) || [];
    for (const c of codeMatches) {
      if (!seen.has(c)) {
        seen.add(c);
        list.push({
          id: `code-${c}`,
          type: 'code',
          label: c,
          typeLabel: '🔤 Code',
          badgeColor: 'bg-purple-100 text-purple-900 dark:bg-purple-900/60 dark:text-purple-200 border-purple-300',
          sourceText: c,
          insertText: c,
        });
      }
    }

    // 5. Numbers, decimals, percentages, currency
    const numMatches = src.match(/\b\d+(?:[.,]\d+)*(?:%|\$|€|£|¥)?\b/g) || [];
    for (const n of numMatches) {
      if (!seen.has(n)) {
        seen.add(n);
        list.push({
          id: `num-${n}`,
          type: 'number',
          label: n,
          typeLabel: '🔢 Number',
          badgeColor: 'bg-indigo-100 text-indigo-900 dark:bg-indigo-900/60 dark:text-indigo-200 border-indigo-300',
          sourceText: n,
          insertText: n,
        });
      }
    }

    // Sort: Missing tags first, then other items
    list.sort((a, b) => {
      if (a.isMissing && !b.isMissing) return -1;
      if (!a.isMissing && b.isMissing) return 1;
      return 0;
    });

    return list;
  }

  $: {
    if (visible) {
      items = extractPlaceables(sourceText, targetText);
      if (selectedIndex >= items.length) selectedIndex = 0;
      updateSpotlight();
    } else {
      items = [];
      onHighlightSource(null);
    }
  }

  function updateSpotlight() {
    if (items.length > 0 && selectedIndex < items.length) {
      onHighlightSource(items[selectedIndex].sourceText);
    } else {
      onHighlightSource(null);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!visible || items.length === 0) return;

    if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIndex = (selectedIndex + 1) % items.length;
      updateSpotlight();
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIndex = (selectedIndex - 1 + items.length) % items.length;
      updateSpotlight();
    } else if (e.key === 'Enter' || e.key === 'Tab') {
      e.preventDefault();
      const chosen = items[selectedIndex];
      if (chosen) {
        onSelect(chosen.insertText);
        onHighlightSource(null);
      }
    } else if (e.key === 'Escape') {
      e.preventDefault();
      onHighlightSource(null);
      onClose();
    }
  }

  onDestroy(() => {
    onHighlightSource(null);
  });
</script>

<svelte:window on:keydown={handleKeydown} />

{#if visible && items.length > 0}
  <div
    class="absolute left-3 top-1 z-40 w-64 rounded-xl bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700
           shadow-2xl py-1 text-xs animate-in fade-in zoom-in-95 backdrop-blur-md"
  >
    <!-- Header -->
    <div class="px-3 py-1.5 text-[10px] font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider border-b border-gray-100 dark:border-gray-800 flex items-center justify-between">
      <span class="flex items-center gap-1">
        <span>⚡</span>
        <span>QuickPlace Omnibar</span>
      </span>
      <span class="text-[9px] font-mono bg-gray-100 dark:bg-gray-800 px-1.5 py-0.5 rounded text-gray-600 dark:text-gray-300">↵ insert</span>
    </div>

    <!-- Candidate List -->
    <div class="max-h-56 overflow-y-auto py-1">
      {#each items as item, i}
        <button
          type="button"
          on:click|preventDefault={() => {
            onSelect(item.insertText);
            onHighlightSource(null);
          }}
          on:mouseenter={() => {
            selectedIndex = i;
            updateSpotlight();
          }}
          class="w-full text-left px-3 py-1.5 flex items-center justify-between transition-colors cursor-pointer
                 {i === selectedIndex
                   ? 'bg-brand-500 text-white font-medium shadow-2xs'
                   : 'text-gray-800 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-800/80'}"
        >
          <div class="flex items-center gap-2 min-w-0 pr-1 truncate">
            <span class="px-1.5 py-0.5 rounded text-[9px] font-semibold border shrink-0 {i === selectedIndex ? 'bg-brand-600 text-white border-brand-400' : item.badgeColor}">
              {item.typeLabel}
            </span>
            <span class="font-mono text-xs truncate {i === selectedIndex ? 'text-white' : 'text-gray-900 dark:text-gray-100 font-medium'}">
              {item.label}
            </span>
          </div>
          <kbd class="text-[9px] font-mono shrink-0 {i === selectedIndex ? 'text-white/80' : 'text-gray-400'}">#{i + 1}</kbd>
        </button>
      {/each}
    </div>

    <!-- Footer tip -->
    <div class="px-3 py-1 bg-gray-50 dark:bg-gray-950/40 border-t border-gray-100 dark:border-gray-800/80 text-[10px] text-gray-400 flex items-center justify-between">
      <span>↑↓ Navigate</span>
      <span>Esc Close</span>
    </div>
  </div>
{/if}
