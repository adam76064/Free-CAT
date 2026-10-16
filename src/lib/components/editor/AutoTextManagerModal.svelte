<script lang="ts">
  /**
   * AutoTextManagerModal.svelte — Studio-grade AutoText & Shorthand Dictionary Manager
   * Allows translators to manage rapid typing triggers and boilerplate expansions.
   */
  import { createEventDispatcher } from 'svelte';
  import { autoTextEntries, addAutoTextEntry, deleteAutoTextEntry, resetAutoTextEntries, type AutoTextEntry } from '$lib/stores/autotext';

  export let isOpen = false;

  const dispatch = createEventDispatcher();

  let searchQuery = '';
  let newTrigger = '';
  let newExpansion = '';
  let newDescription = '';

  $: filteredEntries = $autoTextEntries.filter(e =>
    e.trigger.toLowerCase().includes(searchQuery.toLowerCase()) ||
    e.expansion.toLowerCase().includes(searchQuery.toLowerCase()) ||
    (e.description && e.description.toLowerCase().includes(searchQuery.toLowerCase()))
  );

  function handleAdd() {
    if (!newTrigger.trim() || !newExpansion.trim()) return;
    addAutoTextEntry(newTrigger, newExpansion, newDescription);
    newTrigger = '';
    newExpansion = '';
    newDescription = '';
  }

  function handleClose() {
    dispatch('close');
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      handleClose();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if isOpen}
  <!-- Backdrop -->
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4 animate-in fade-in duration-150">
    <div class="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-xl shadow-2xl max-w-2xl w-full flex flex-col max-h-[85vh] overflow-hidden">
      <!-- Header -->
      <div class="px-5 py-4 border-b border-gray-200 dark:border-gray-800 flex items-center justify-between bg-gray-50/80 dark:bg-gray-900/80">
        <div class="flex items-center gap-2.5">
          <span class="text-xl">⚡</span>
          <div>
            <h3 class="text-sm font-bold text-gray-900 dark:text-gray-100">
              AutoText & Shorthand Dictionary Manager
            </h3>
            <p class="text-[11px] text-gray-500 dark:text-gray-400">
              Expands abbreviations automatically via AutoSuggest or typing triggers.
            </p>
          </div>
        </div>
        <button
          type="button"
          on:click={handleClose}
          class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 text-lg leading-none p-1 cursor-pointer"
        >
          ✕
        </button>
      </div>

      <!-- Search & Add Controls -->
      <div class="p-4 border-b border-gray-100 dark:border-gray-800 space-y-3 bg-white dark:bg-gray-950">
        <!-- Search bar -->
        <div class="relative">
          <input
            type="text"
            bind:value={searchQuery}
            placeholder="Search triggers, expansions, or descriptions..."
            class="w-full text-xs px-3 py-2 pl-8 rounded-lg bg-gray-50 dark:bg-gray-900 border border-gray-200 dark:border-gray-700 text-gray-800 dark:text-gray-200 focus:outline-none focus:ring-1 focus:ring-brand-500"
          />
          <span class="absolute left-2.5 top-2.5 text-xs text-gray-400">🔍</span>
          {#if searchQuery}
            <button
              on:click={() => (searchQuery = '')}
              class="absolute right-2.5 top-2.5 text-xs text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 cursor-pointer"
            >
              ✕
            </button>
          {/if}
        </div>

        <!-- Add new shorthand row -->
        <form on:submit|preventDefault={handleAdd} class="grid grid-cols-12 gap-2 items-center bg-brand-50/40 dark:bg-brand-950/20 p-2.5 rounded-lg border border-brand-200/60 dark:border-brand-900/40">
          <div class="col-span-3">
            <input
              type="text"
              bind:value={newTrigger}
              placeholder="Trigger (e.g. b/c)"
              class="w-full text-xs px-2.5 py-1.5 rounded bg-white dark:bg-gray-900 border border-gray-300 dark:border-gray-700 font-mono text-brand-700 dark:text-brand-300 focus:outline-none focus:ring-1 focus:ring-brand-500"
            />
          </div>
          <div class="col-span-5">
            <input
              type="text"
              bind:value={newExpansion}
              placeholder="Full text (e.g. because)"
              class="w-full text-xs px-2.5 py-1.5 rounded bg-white dark:bg-gray-900 border border-gray-300 dark:border-gray-700 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-1 focus:ring-brand-500"
            />
          </div>
          <div class="col-span-3">
            <input
              type="text"
              bind:value={newDescription}
              placeholder="Context / Note (optional)"
              class="w-full text-xs px-2.5 py-1.5 rounded bg-white dark:bg-gray-900 border border-gray-300 dark:border-gray-700 text-gray-500 dark:text-gray-400 focus:outline-none focus:ring-1 focus:ring-brand-500"
            />
          </div>
          <div class="col-span-1 flex justify-end">
            <button
              type="submit"
              disabled={!newTrigger.trim() || !newExpansion.trim()}
              class="w-full py-1.5 bg-brand-500 hover:bg-brand-600 disabled:opacity-40 text-white text-xs font-bold rounded transition-colors shadow-2xs cursor-pointer"
              title="Add Shorthand"
            >
              +
            </button>
          </div>
        </form>
      </div>

      <!-- Shorthand Entries Table -->
      <div class="flex-1 overflow-y-auto p-4 space-y-1.5 divide-y divide-gray-100 dark:divide-gray-800">
        {#if filteredEntries.length === 0}
          <div class="py-12 text-center text-xs text-gray-400">
            No AutoText triggers found matching "{searchQuery}".
          </div>
        {:else}
          {#each filteredEntries as item (item.id)}
            <div class="pt-2 first:pt-0 flex items-center justify-between gap-3 text-xs hover:bg-gray-50 dark:hover:bg-gray-800/50 p-2 rounded-lg transition-colors">
              <div class="flex items-center gap-3 min-w-0 flex-1">
                <span class="font-mono font-bold text-brand-600 dark:text-brand-400 bg-brand-50 dark:bg-brand-900/40 px-2 py-0.5 rounded text-[11px] shrink-0 border border-brand-200 dark:border-brand-800">
                  {item.trigger}
                </span>
                <span class="text-gray-400">➔</span>
                <span class="font-medium text-gray-900 dark:text-gray-100 truncate">
                  {item.expansion}
                </span>
                {#if item.description}
                  <span class="text-[10px] text-gray-400 italic truncate shrink-0">
                    ({item.description})
                  </span>
                {/if}
              </div>
              <button
                type="button"
                on:click={() => deleteAutoTextEntry(item.id)}
                class="text-gray-400 hover:text-red-500 text-xs p-1 rounded hover:bg-red-50 dark:hover:bg-red-950/40 transition-colors cursor-pointer shrink-0"
                title="Delete shorthand"
              >
                🗑
              </button>
            </div>
          {/each}
        {/if}
      </div>

      <!-- Footer -->
      <div class="px-5 py-3 border-t border-gray-200 dark:border-gray-800 bg-gray-50/80 dark:bg-gray-900/80 flex items-center justify-between text-xs">
        <div class="flex items-center gap-2">
          <button
            type="button"
            on:click={resetAutoTextEntries}
            class="text-[11px] text-gray-500 hover:text-red-600 transition-colors underline cursor-pointer"
          >
            Reset to Standard Defaults
          </button>
          <span class="text-gray-300 dark:text-gray-700">|</span>
          <span class="text-gray-500 text-[11px]">
            {$autoTextEntries.length} total entries
          </span>
        </div>
        <button
          type="button"
          on:click={handleClose}
          class="px-4 py-1.5 bg-gray-200 dark:bg-gray-800 hover:bg-gray-300 dark:hover:bg-gray-700 text-gray-800 dark:text-gray-200 font-semibold rounded-lg text-xs transition-colors cursor-pointer"
        >
          Done
        </button>
      </div>
    </div>
  </div>
{/if}
