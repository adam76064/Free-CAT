<script lang="ts">
  /**
   * SegmentContextMenu.svelte — Floating right-click context menu for Segment rows.
   * Provides rapid access to segment actions: Copy, Lock, Bookmark, Split, Merge, Status change.
   * Closes on click outside or Escape.
   */
  import type { Segment } from '$lib/stores/segments';

  interface Props {
    segment: Segment;
    x: number;
    y: number;
    canMergeNext?: boolean;
    canMergePrev?: boolean;
    onClose: () => void;
    onCopySource: () => void;
    onCopyTarget: () => void;
    onCopySourceToTarget: () => void;
    onClearTarget: () => void;
    onToggleLock: () => void;
    onToggleBookmark: () => void;
    onSplit?: () => void;
    onMergeNext?: () => void;
    onConfirm: () => void;
  }

  let {
    segment,
    x,
    y,
    canMergeNext = false,
    canMergePrev = false,
    onClose,
    onCopySource,
    onCopyTarget,
    onCopySourceToTarget,
    onClearTarget,
    onToggleLock,
    onToggleBookmark,
    onSplit,
    onMergeNext,
    onConfirm,
  }: Props = $props();

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      onClose();
    }
  }

  // Ensure menu stays within viewport
  let menuEl = $state<HTMLDivElement | null>(null);
  let adjustedX = $derived(
    typeof window !== 'undefined' && menuEl
      ? Math.min(x, window.innerWidth - 220)
      : x
  );
  let adjustedY = $derived(
    typeof window !== 'undefined' && menuEl
      ? Math.min(y, window.innerHeight - 340)
      : y
  );
</script>

<svelte:window onkeydown={handleKeydown} onclick={onClose} />

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<div
  bind:this={menuEl}
  class="fixed z-[150] w-56 rounded-xl bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700 shadow-2xl py-1.5 text-xs text-gray-700 dark:text-gray-200 divide-y divide-gray-100 dark:divide-gray-800 select-none animate-in fade-in zoom-in-95 duration-100"
  style="top: {adjustedY}px; left: {adjustedX}px;"
  role="menu"
  tabindex="-1"
  onclick={(e) => e.stopPropagation()}
>
  <!-- Segment info header -->
  <div class="px-3 py-1 text-[10px] text-gray-400 font-mono flex items-center justify-between">
    <span>Segment #{segment.seq_number}</span>
    <span class="capitalize">{segment.status}</span>
  </div>

  <!-- Primary actions -->
  <div class="py-1">
    <button
      type="button"
      onclick={() => { onConfirm(); onClose(); }}
      class="w-full text-left px-3 py-1.5 flex items-center justify-between hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors cursor-pointer"
      role="menuitem"
    >
      <span class="flex items-center gap-2">
        <span class="text-emerald-500">✓</span> Confirm Segment
      </span>
      <kbd class="text-[9px] text-gray-400 font-mono">Ctrl+↵</kbd>
    </button>

    <button
      type="button"
      onclick={() => { onCopySourceToTarget(); onClose(); }}
      class="w-full text-left px-3 py-1.5 flex items-center justify-between hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors cursor-pointer"
      role="menuitem"
    >
      <span class="flex items-center gap-2">
        <span>📋</span> Copy Source to Target
      </span>
      <kbd class="text-[9px] text-gray-400 font-mono">Ctrl+D</kbd>
    </button>
  </div>

  <!-- Clipboard & Text -->
  <div class="py-1">
    <button
      type="button"
      onclick={() => { onCopySource(); onClose(); }}
      class="w-full text-left px-3 py-1.5 flex items-center gap-2 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors cursor-pointer"
      role="menuitem"
    >
      <span>📄</span> Copy Source Text
    </button>

    <button
      type="button"
      onclick={() => { onCopyTarget(); onClose(); }}
      class="w-full text-left px-3 py-1.5 flex items-center gap-2 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors cursor-pointer"
      role="menuitem"
    >
      <span>📑</span> Copy Target Text
    </button>

    <button
      type="button"
      onclick={() => { onClearTarget(); onClose(); }}
      class="w-full text-left px-3 py-1.5 flex items-center gap-2 hover:bg-gray-100 dark:hover:bg-gray-800 text-red-600 dark:text-red-400 transition-colors cursor-pointer"
      role="menuitem"
    >
      <span>🗑️</span> Clear Target
    </button>
  </div>

  <!-- Segment status & structure -->
  <div class="py-1">
    <button
      type="button"
      onclick={() => { onToggleLock(); onClose(); }}
      class="w-full text-left px-3 py-1.5 flex items-center justify-between hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors cursor-pointer"
      role="menuitem"
    >
      <span class="flex items-center gap-2">
        <span>{segment.locked ? '🔓' : '🔒'}</span>
        {segment.locked ? 'Unlock Segment' : 'Lock Segment'}
      </span>
      <kbd class="text-[9px] text-gray-400 font-mono">Ctrl+L</kbd>
    </button>

    <button
      type="button"
      onclick={() => { onToggleBookmark(); onClose(); }}
      class="w-full text-left px-3 py-1.5 flex items-center gap-2 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors cursor-pointer"
      role="menuitem"
    >
      <span>{segment.bookmarked ? '⭐' : '☆'}</span>
      {segment.bookmarked ? 'Remove Bookmark' : 'Bookmark Segment'}
    </button>
  </div>

  <!-- Split & Merge -->
  <div class="py-1">
    {#if onSplit}
      <button
        type="button"
        onclick={() => { onSplit(); onClose(); }}
        class="w-full text-left px-3 py-1.5 flex items-center justify-between hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors cursor-pointer"
        role="menuitem"
      >
        <span class="flex items-center gap-2">
          <span>✂️</span> Split Segment
        </span>
        <kbd class="text-[9px] text-gray-400 font-mono">Ctrl+⇧+S</kbd>
      </button>
    {/if}

    {#if onMergeNext && canMergeNext}
      <button
        type="button"
        onclick={() => { onMergeNext(); onClose(); }}
        class="w-full text-left px-3 py-1.5 flex items-center justify-between hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors cursor-pointer"
        role="menuitem"
      >
        <span class="flex items-center gap-2">
          <span>🔗</span> Merge with Next
        </span>
        <kbd class="text-[9px] text-gray-400 font-mono">Ctrl+⇧+M</kbd>
      </button>
    {/if}
  </div>
</div>
