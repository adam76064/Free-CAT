<script lang="ts">
  /**
   * AutoSuggestPopup — Floating predictive typing overlay.
   * Renders instant terminology and TM sub-phrase completions as the user types.
   */
  import type { SuggestionItem } from '$lib/utils/suggestTrie';

  export let suggestions: SuggestionItem[] = [];
  export let selectedIndex = 0;
  export let onSelect: (item: SuggestionItem) => void;
  export let visible = false;
</script>

{#if visible && suggestions.length > 0}
  <div
    class="absolute z-40 bg-white dark:bg-gray-900 border border-brand-200 dark:border-brand-800
           rounded-lg shadow-xl py-1 px-1 min-w-[220px] max-w-sm text-xs animate-in fade-in duration-75"
    style="bottom: calc(100% + 6px); left: 1rem;"
  >
    <div class="px-2 py-1 text-[10px] uppercase font-semibold tracking-wider text-gray-400 border-b border-gray-100 dark:border-gray-800 flex items-center justify-between">
      <span>AutoSuggest</span>
      <span class="text-brand-500 font-mono">Tab ⇥</span>
    </div>

    <div class="mt-1 space-y-0.5">
      {#each suggestions as item, i}
        <!-- svelte-ignore a11y-click-events-have-key-events -->
        <!-- svelte-ignore a11y-no-static-element-interactions -->
        <div
          on:click={() => onSelect(item)}
          on:mouseenter={() => (selectedIndex = i)}
          class="flex items-center justify-between px-2.5 py-1.5 rounded cursor-pointer transition-colors
                 {i === selectedIndex
                   ? 'bg-brand-50 dark:bg-brand-900/40 text-brand-700 dark:text-brand-300 font-medium'
                   : 'text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-800'}"
        >
          <span class="truncate mr-2">{item.displayText || item.text}</span>
          <span
            class="text-[9px] px-1 py-0.5 rounded font-mono uppercase font-semibold
                   {item.source === 'termbase'
                     ? 'bg-emerald-100 dark:bg-emerald-900/40 text-emerald-700 dark:text-emerald-300'
                     : item.source === 'autotext'
                       ? 'bg-amber-100 dark:bg-amber-900/40 text-amber-700 dark:text-amber-300'
                       : 'bg-blue-100 dark:bg-blue-900/40 text-blue-700 dark:text-blue-300'}"
          >
            {item.source === 'termbase' ? 'TB' : item.source === 'autotext' ? '⚡ SNIP' : 'TM'}
          </span>
        </div>
      {/each}
    </div>
  </div>
{/if}
