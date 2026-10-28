<script lang="ts">
  /**
   * KeyboardShortcutsModal — Cheat-sheet overlay (Ctrl+/ to open/close).
   * Shows all keyboard shortcuts categorized. Accessible, focus-trapped, Escape to close.
   */
  interface Props {
    open?: boolean;
    onClose: () => void;
  }
  let { open = false, onClose }: Props = $props();

  interface Shortcut {
    keys: string[];
    description: string;
  }

  interface ShortcutCategory {
    name: string;
    icon: string;
    shortcuts: Shortcut[];
  }

  const CATEGORIES: ShortcutCategory[] = [
    {
      name: 'Navigation',
      icon: '🧭',
      shortcuts: [
        { keys: ['Alt', '↑ / ↓'], description: 'Move to previous / next segment' },
        { keys: ['Ctrl', 'Home'], description: 'Jump to first segment' },
        { keys: ['Ctrl', 'End'], description: 'Jump to last segment' },
        { keys: ['Ctrl', 'G'], description: 'Go to segment by number' },
        { keys: ['F3'], description: 'Open Concordance search' },
        { keys: ['Ctrl', 'F'], description: 'Open Find & Replace' },
      ],
    },
    {
      name: 'Translation',
      icon: '✍️',
      shortcuts: [
        { keys: ['Ctrl', 'Enter'], description: 'Confirm segment and advance' },
        { keys: ['Ctrl', 'Shift', 'Enter'], description: 'Confirm segment (stay)' },
        { keys: ['Ctrl', 'D'], description: 'Copy source to target' },
        { keys: ['Ctrl', 'Shift', 'U'], description: 'Accept fuzzy match suggestion' },
        { keys: ['Ctrl', 'Alt', '1–9'], description: 'Insert AutoSuggest option 1–9' },
        { keys: ['Tab'], description: 'Accept AutoSuggest top completion' },
      ],
    },
    {
      name: 'Segment Operations',
      icon: '⚡',
      shortcuts: [
        { keys: ['Ctrl', 'L'], description: 'Lock / Unlock segment' },
        { keys: ['Ctrl', 'Shift', 'S'], description: 'Split segment at cursor' },
        { keys: ['Ctrl', 'Shift', 'M'], description: 'Merge with next segment' },
        { keys: ['Ctrl', 'Shift', 'P'], description: 'Open AI Pre-translate dialog' },
        { keys: ['Ctrl', 'Z'], description: 'Undo last change' },
        { keys: ['Ctrl', 'Y'], description: 'Redo last change' },
      ],
    },
    {
      name: 'Quality & Review',
      icon: '🔍',
      shortcuts: [
        { keys: ['F7'], description: 'Run QA checks on current file' },
        { keys: ['Ctrl', 'Shift', 'N'], description: 'Next QA issue' },
        { keys: ['Ctrl', 'Shift', 'B'], description: 'Previous QA issue' },
        { keys: ['Ctrl', 'Shift', 'T'], description: 'Toggle Track Changes (Revision Mode)' },
        { keys: ['Ctrl', 'Shift', 'A'], description: 'Accept all Track Changes' },
        { keys: ['Ctrl', 'Shift', 'R'], description: 'Reject all Track Changes' },
      ],
    },
    {
      name: 'Tags & Formatting',
      icon: '🏷️',
      shortcuts: [
        { keys: ['F8'], description: 'Insert next inline tag' },
        { keys: ['Ctrl', ','], description: 'Open QuickPlace Omnibar' },
        { keys: ['Ctrl', 'Shift', 'X'], description: 'Open Web Lookup (selected text)' },
        { keys: ['Ctrl', 'Shift', 'C'], description: 'Copy source segment text' },
      ],
    },
    {
      name: 'App & View',
      icon: '🖥️',
      shortcuts: [
        { keys: ['Ctrl', '/'], description: 'Show / hide this shortcuts panel' },
        { keys: ['Ctrl', 'Shift', 'D'], description: 'Toggle dark / light mode' },
        { keys: ['Ctrl', ']'], description: 'Increase editor font size' },
        { keys: ['Ctrl', '['], description: 'Decrease editor font size' },
        { keys: ['Ctrl', 'Shift', 'V'], description: 'Toggle TTS proofreader' },
        { keys: ['Ctrl', 'Alt', 'Q'], description: 'Open Quote & Invoice' },
      ],
    },
  ];

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      onClose();
    }
  }
</script>

{#if open}
  <!-- Backdrop -->
  <div
    class="fixed inset-0 z-[200] flex items-center justify-center"
    role="dialog"
    aria-modal="true"
    aria-label="Keyboard Shortcuts"
  >
    <!-- Dark overlay -->
    <button
      class="absolute inset-0 bg-black/50 backdrop-blur-sm cursor-default"
      onclick={onClose}
      aria-label="Close shortcuts modal"
      tabindex="-1"
    ></button>

    <!-- Modal card -->
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
    <div
      class="relative z-10 w-[760px] max-w-[95vw] max-h-[85vh] bg-white dark:bg-gray-900 rounded-2xl shadow-2xl border border-gray-200 dark:border-gray-700 overflow-hidden flex flex-col"
      onkeydown={handleKeydown}
      role="document"
    >
      <!-- Header -->
      <div class="flex items-center justify-between p-4 border-b border-gray-200 dark:border-gray-700 shrink-0">
        <div class="flex items-center gap-2">
          <span class="text-xl">⌨️</span>
          <div>
            <h2 class="font-semibold text-gray-900 dark:text-white text-sm">Keyboard Shortcuts</h2>
            <p class="text-xs text-gray-500 dark:text-gray-400">Press <kbd class="px-1 py-0.5 bg-gray-100 dark:bg-gray-800 rounded text-[10px] font-mono border border-gray-300 dark:border-gray-600">Ctrl+/</kbd> to toggle this panel</p>
          </div>
        </div>
        <button
          type="button"
          onclick={onClose}
          class="text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 p-1.5 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors cursor-pointer"
          aria-label="Close"
        >
          ✕
        </button>
      </div>

      <!-- Shortcut grid -->
      <div class="overflow-y-auto p-4 grid grid-cols-2 gap-4">
        {#each CATEGORIES as cat}
          <div class="space-y-1.5">
            <div class="flex items-center gap-1.5 mb-2">
              <span class="text-base">{cat.icon}</span>
              <h3 class="text-xs font-bold uppercase tracking-wide text-gray-500 dark:text-gray-400">{cat.name}</h3>
            </div>
            {#each cat.shortcuts as sc}
              <div class="flex items-center justify-between gap-2 py-1 border-b border-gray-100 dark:border-gray-800/80">
                <span class="text-xs text-gray-700 dark:text-gray-300">{sc.description}</span>
                <div class="flex items-center gap-0.5 shrink-0">
                  {#each sc.keys as key, i}
                    {#if i > 0}<span class="text-gray-400 text-[10px] px-0.5">+</span>{/if}
                    <kbd class="px-1.5 py-0.5 bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 rounded border border-gray-300 dark:border-gray-600 text-[10px] font-mono font-semibold shadow-sm">{key}</kbd>
                  {/each}
                </div>
              </div>
            {/each}
          </div>
        {/each}
      </div>

      <!-- Footer -->
      <div class="shrink-0 px-4 py-2.5 border-t border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900/50 text-center text-[10px] text-gray-400">
        Free-CAT v0.2 — Enterprise CAT Tool • <a href="https://github.com/adam76064/Free-CAT" target="_blank" rel="noreferrer" class="hover:text-brand-500 underline">GitHub</a>
      </div>
    </div>
  </div>
{/if}
