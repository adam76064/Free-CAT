<script lang="ts">
  import { dialog } from '$lib/actions/dialog';
  /**
   * CommandPalette — Global Omni-Action modal for keyboard-first navigation (Ctrl+K).
   * Provides lightning-fast access to any action without taking hands off the keyboard.
   */
  import { onMount, onDestroy } from 'svelte';
  import { goto } from '$app/navigation';
  import { selectedSegment, selectedIndex, filteredSegments } from '$lib/stores/segments';
  import { currentProject, currentFile } from '$lib/stores/project';

  export let isOpen = false;
  export let onToggleInspector: () => void = () => {};
  export let onTogglePreview: () => void = () => {};
  export let onOpenFind: () => void = () => {};
  export let onOpenReplace: () => void = () => {};
  export let onRunQa: () => void = () => {};
  export let onExport: () => void = () => {};
  export let onSplit: () => void = () => {};
  export let onCopySource: () => void = () => {};
  export let onClearTarget: () => void = () => {};
  export let onOpenInvoice: () => void = () => {};
  export let onExtractTerms: () => void = () => {};

  let searchQuery = '';
  let activeIndex = 0;
  let searchInput: HTMLInputElement;

  interface CommandItem {
    id: string;
    title: string;
    category: 'Editor' | 'Tools' | 'View' | 'Project' | 'Linguistics';
    shortcut?: string;
    icon?: string;
    action: () => void;
  }

  $: commands = [
    {
      id: 'copy-source',
      title: 'Copy Source Text to Target',
      category: 'Editor',
      shortcut: 'Alt+Ins',
      icon: '📋',
      action: () => { onCopySource(); close(); }
    },
    {
      id: 'clear-target',
      title: 'Clear Target Text',
      category: 'Editor',
      shortcut: 'Ctrl+Del',
      icon: '🧹',
      action: () => { onClearTarget(); close(); }
    },
    {
      id: 'split-segment',
      title: 'Split Current Segment',
      category: 'Editor',
      shortcut: 'Ctrl+Alt+S',
      icon: '✂',
      action: () => { onSplit(); close(); }
    },
    {
      id: 'find-text',
      title: 'Find in Document',
      category: 'Editor',
      shortcut: 'Ctrl+F',
      icon: '🔍',
      action: () => { onOpenFind(); close(); }
    },
    {
      id: 'replace-text',
      title: 'Find & Replace',
      category: 'Editor',
      shortcut: 'Ctrl+H',
      icon: '🔄',
      action: () => { onOpenReplace(); close(); }
    },
    {
      id: 'toggle-inspector',
      title: 'Toggle Inspector Sidebar',
      category: 'View',
      shortcut: 'Ctrl+\\',
      icon: '⮌',
      action: () => { onToggleInspector(); close(); }
    },
    {
      id: 'toggle-preview',
      title: 'Toggle Live In-Context Preview',
      category: 'View',
      shortcut: 'Ctrl+P',
      icon: '👁',
      action: () => { onTogglePreview(); close(); }
    },
    {
      id: 'run-qa',
      title: 'Run Quality Assurance (QA) Checks',
      category: 'Linguistics',
      shortcut: 'F7',
      icon: '✓',
      action: () => { onRunQa(); close(); }
    },
    {
      id: 'extract-terms',
      title: 'Automated Terminology Extraction (ATE)',
      category: 'Linguistics',
      shortcut: 'Ctrl+Shift+T',
      icon: '⚡',
      action: () => { onExtractTerms(); close(); }
    },
    {
      id: 'open-aligner',
      title: 'Open Visual Bilingual File Aligner',
      category: 'Tools',
      icon: '📐',
      action: () => { goto('/aligner'); close(); }
    },
    {
      id: 'open-invoice',
      title: 'Generate Client Quote / Invoice',
      category: 'Project',
      icon: '📑',
      action: () => { onOpenInvoice(); close(); }
    },
    {
      id: 'export-file',
      title: 'Export Translated Target File',
      category: 'Project',
      shortcut: 'Ctrl+Shift+E',
      icon: '⬇',
      action: () => { onExport(); close(); }
    },
    {
      id: 'switch-theme',
      title: 'Toggle Dark / Light Theme',
      category: 'View',
      icon: '🌓',
      action: () => {
        document.documentElement.classList.toggle('dark');
        close();
      }
    }
  ] as CommandItem[];

  $: filteredCommands = commands.filter(cmd => {
    if (!searchQuery.trim()) return true;
    const q = searchQuery.toLowerCase();
    return cmd.title.toLowerCase().includes(q) || cmd.category.toLowerCase().includes(q);
  });

  $: if (filteredCommands.length > 0 && activeIndex >= filteredCommands.length) {
    activeIndex = 0;
  }

  function handleKeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
      e.preventDefault();
      isOpen = !isOpen;
      if (isOpen) {
        searchQuery = '';
        activeIndex = 0;
        setTimeout(() => searchInput?.focus(), 50);
      }
    } else if (isOpen) {
      if (e.key === 'Escape') {
        e.preventDefault();
        close();
      } else if (e.key === 'ArrowDown') {
        e.preventDefault();
        activeIndex = (activeIndex + 1) % filteredCommands.length;
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        activeIndex = (activeIndex - 1 + filteredCommands.length) % filteredCommands.length;
      } else if (e.key === 'Enter') {
        e.preventDefault();
        if (filteredCommands[activeIndex]) {
          filteredCommands[activeIndex].action();
        }
      }
    }
  }

  function close() {
    isOpen = false;
    searchQuery = '';
  }

  onMount(() => {
    window.addEventListener('keydown', handleKeydown);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeydown);
  });
</script>

{#if isOpen}
  <!-- Backdrop -->
  <!-- svelte-ignore a11y-click-events-have-key-events -->
  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div use:dialog={{ label: "Command palette" }}
    class="fixed inset-0 z-50 flex items-start justify-center pt-24 bg-black/50 backdrop-blur-xs p-4 animate-in fade-in duration-100"
    on:click|self={close}
  >
    <div
      class="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 shadow-2xl max-w-xl w-full overflow-hidden flex flex-col animate-in zoom-in-95 duration-150"
    >
      <!-- Search Input -->
      <div class="flex items-center px-4 py-3 border-b border-gray-100 dark:border-gray-800 gap-3">
        <span class="text-gray-400 text-lg">⚡</span>
        <input
          bind:this={searchInput}
          bind:value={searchQuery}
          type="text"
          placeholder="Type a command or search action (e.g. Export, QA, Aligner)..."
          class="flex-1 bg-transparent text-sm text-gray-900 dark:text-white placeholder-gray-400 outline-none"
        />
        <kbd class="text-[10px] font-mono bg-gray-100 dark:bg-gray-800 text-gray-500 px-1.5 py-0.5 rounded border border-gray-200 dark:border-gray-700">
          ESC
        </kbd>
      </div>

      <!-- Commands list -->
      <div class="max-h-80 overflow-y-auto p-2 space-y-1">
        {#if filteredCommands.length === 0}
          <div class="px-4 py-8 text-center text-xs text-gray-400">
            No matching commands found.
          </div>
        {:else}
          {#each filteredCommands as cmd, i}
            <!-- svelte-ignore a11y-click-events-have-key-events -->
            <!-- svelte-ignore a11y-no-static-element-interactions -->
            <div
              on:click={cmd.action}
              on:mouseenter={() => (activeIndex = i)}
              class="flex items-center justify-between px-3 py-2 rounded-lg cursor-pointer text-xs transition-colors
                     {i === activeIndex
                       ? 'bg-brand-500 text-white dark:bg-brand-600'
                       : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800'}"
            >
              <div class="flex items-center gap-2.5">
                <span class="text-base">{cmd.icon || '⚡'}</span>
                <div>
                  <span class="font-medium">{cmd.title}</span>
                  <span class="ml-2 text-[10px] opacity-70">({cmd.category})</span>
                </div>
              </div>
              {#if cmd.shortcut}
                <kbd class="font-mono text-[10px] px-1.5 py-0.5 rounded
                            {i === activeIndex
                              ? 'bg-brand-700/50 text-white'
                              : 'bg-gray-100 dark:bg-gray-800 text-gray-400 border border-gray-200 dark:border-gray-700'}">
                  {cmd.shortcut}
                </kbd>
              {/if}
            </div>
          {/each}
        {/if}
      </div>

      <!-- Footer tips -->
      <div class="px-4 py-2 border-t border-gray-100 dark:border-gray-800 bg-gray-50 dark:bg-gray-900/50 flex items-center justify-between text-[11px] text-gray-400">
        <span>Use <kbd class="font-mono font-semibold">↑</kbd> <kbd class="font-mono font-semibold">↓</kbd> to navigate</span>
        <span>Press <kbd class="font-mono font-semibold">↵ Enter</kbd> to run</span>
      </div>
    </div>
  </div>
{/if}
