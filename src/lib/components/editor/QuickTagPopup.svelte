<script lang="ts">
  // QuickTagPopup.svelte — Keyboard HUD for tag insertion (Ctrl+,)
  export let visible = false;
  export let tags: string[] = [];
  export let selectedIndex = 0;
  export let onSelect: (tag: string) => void;
  export let onClose: () => void = () => {};

  function handleKeydown(e: KeyboardEvent) {
    if (!visible) return;
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIndex = (selectedIndex + 1) % tags.length;
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIndex = (selectedIndex - 1 + tags.length) % tags.length;
    } else if (e.key === 'Enter' || e.key === 'Tab') {
      e.preventDefault();
      if (tags[selectedIndex]) {
        onSelect(tags[selectedIndex]);
      }
    } else if (e.key === 'Escape') {
      e.preventDefault();
      onClose();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if visible && tags.length > 0}
  <div
    class="absolute left-3 top-1 z-30 w-52 rounded-xl bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-700
           shadow-2xl py-1 text-xs animate-in fade-in zoom-in-95 backdrop-blur-md"
  >
    <div class="px-2.5 py-1 text-[10px] font-semibold text-gray-400 dark:text-gray-500 uppercase tracking-wider border-b border-gray-100 dark:border-gray-800 flex items-center justify-between">
      <span>Missing Tags (Ctrl+,)</span>
      <span class="text-[9px] font-mono bg-gray-100 dark:bg-gray-800 px-1 rounded">↵ insert</span>
    </div>

    <div class="max-h-48 overflow-y-auto py-0.5">
      {#each tags as tag, i}
        <button
          type="button"
          on:click|preventDefault={() => onSelect(tag)}
          class="w-full text-left px-2.5 py-1.5 flex items-center justify-between transition-colors cursor-pointer
                 {i === selectedIndex
                   ? 'bg-brand-500 text-white font-medium'
                   : 'text-gray-800 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-800'}"
        >
          <span class="font-mono text-xs font-bold flex items-center gap-1.5">
            <span class="px-1 py-0.2 rounded text-[10px] {i === selectedIndex ? 'bg-brand-600 text-white' : 'bg-amber-100 text-amber-900 dark:bg-amber-900/60 dark:text-amber-200'}">
              {tag}
            </span>
            <span class="text-[11px] font-normal opacity-90">Tag {tag.replace(/[{}]/g, '')}</span>
          </span>
          <kbd class="text-[9px] font-mono opacity-70">#{i + 1}</kbd>
        </button>
      {/each}
    </div>
  </div>
{/if}
