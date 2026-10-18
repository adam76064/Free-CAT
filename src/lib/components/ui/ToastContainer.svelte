<script lang="ts">
  /**
   * ToastContainer.svelte — Global toast notification container.
   * Mounted once in +layout.svelte. Renders stacked slide-in toasts
   * in the top-right corner of the screen.
   */
  import { toast } from '$lib/stores/toast';

  const icons = { success: '✓', error: '✕', warning: '⚠', info: 'ℹ' };

  const wrapperCls: Record<string, string> = {
    success: 'bg-emerald-50 border-emerald-300 text-emerald-900 dark:bg-emerald-950/90 dark:border-emerald-700 dark:text-emerald-100',
    error:   'bg-red-50 border-red-300 text-red-900 dark:bg-red-950/90 dark:border-red-700 dark:text-red-100',
    warning: 'bg-amber-50 border-amber-300 text-amber-900 dark:bg-amber-950/90 dark:border-amber-700 dark:text-amber-100',
    info:    'bg-blue-50 border-blue-300 text-blue-900 dark:bg-blue-950/90 dark:border-blue-700 dark:text-blue-100',
  };

  const dotCls: Record<string, string> = {
    success: 'bg-emerald-500',
    error:   'bg-red-500',
    warning: 'bg-amber-500',
    info:    'bg-blue-500',
  };
</script>

<!-- Fixed to top-right, pointer-events-none so it doesn't block clicks -->
<div class="fixed top-4 right-4 z-[9999] flex flex-col gap-2 pointer-events-none w-80">
  {#each $toast as t (t.id)}
    <div
      class="pointer-events-auto flex items-start gap-3 px-4 py-3 rounded-xl border shadow-2xl
             backdrop-blur-sm {wrapperCls[t.type]}"
    >
      <!-- Icon dot -->
      <div class="w-5 h-5 rounded-full {dotCls[t.type]} text-white flex items-center justify-center
                  text-[11px] font-black shrink-0 mt-0.5 shadow-sm">
        {icons[t.type]}
      </div>

      <!-- Content -->
      <div class="flex-1 min-w-0">
        <div class="text-sm font-bold leading-snug">{t.title}</div>
        {#if t.message}
          <div class="text-xs mt-0.5 opacity-80 break-words leading-snug">{t.message}</div>
        {/if}
      </div>

      <!-- Dismiss button -->
      <button
        type="button"
        onclick={() => toast.dismiss(t.id)}
        class="shrink-0 text-current opacity-40 hover:opacity-100 transition-opacity cursor-pointer text-sm leading-none mt-0.5"
        aria-label="Dismiss"
      >✕</button>
    </div>
  {/each}
</div>
