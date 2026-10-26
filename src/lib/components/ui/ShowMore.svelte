<script lang="ts">
  /**
   * ShowMore — "Showing N of M · Show more" footer for long lists.
   * Lists render only the first N rows so the DOM stays small; each click reveals another page.
   */
  let {
    shown,
    total,
    step = 300,
    onmore,
  }: { shown: number; total: number; step?: number; onmore: () => void } = $props();

  const remaining = $derived(Math.max(0, total - shown));
</script>

{#if remaining > 0}
  <div class="flex items-center justify-center gap-3 py-3 text-xs text-gray-500 dark:text-gray-400">
    <span>Showing {Math.min(shown, total).toLocaleString()} of {total.toLocaleString()}</span>
    <button
      type="button"
      onclick={onmore}
      class="px-3 py-1 rounded-lg border border-gray-200 dark:border-gray-700 text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
    >
      Show {Math.min(step, remaining).toLocaleString()} more
    </button>
  </div>
{/if}
