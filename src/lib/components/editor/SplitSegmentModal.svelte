<script lang="ts">
  import type { Segment } from '$lib/stores/segments';

  let {
    segment,
    onconfirm,
    oncancel,
  }: {
    segment: Segment;
    /** Called with the character index at which to split the source text. */
    onconfirm: (splitIndex: number) => void;
    oncancel: () => void;
  } = $props();

  // The parent mounts this fresh for each segment, so seeding from the prop once is intended.
  // svelte-ignore state_referenced_locally
  let pos = $state(Math.max(1, Math.floor(segment.source_text.length / 2)));

  const before = $derived(segment.source_text.slice(0, pos));
  const after = $derived(segment.source_text.slice(pos));

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      e.preventDefault();
      oncancel();
    }
  }
</script>

<svelte:window onkeydown={onKeydown} />

<div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4">
  <div
    role="dialog"
    aria-modal="true"
    aria-labelledby="split-dialog-title"
    class="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 shadow-2xl max-w-lg w-full p-6 animate-in fade-in zoom-in-95"
  >
    <h3 id="split-dialog-title" class="text-base font-semibold text-gray-900 dark:text-white mb-1">
      Split Segment #{segment.seq_number}
    </h3>
    <p class="text-xs text-gray-500 dark:text-gray-400 mb-4">
      Move the slider to choose the split boundary between Part 1 and Part 2.
    </p>

    <!-- Live split preview text -->
    <div dir="auto" class="p-3 bg-gray-50 dark:bg-gray-800/80 rounded-lg text-sm mb-4 font-editor leading-relaxed border border-gray-200 dark:border-gray-700">
      <span class="text-blue-700 dark:text-blue-300 bg-blue-50 dark:bg-blue-900/40 rounded px-1">{before}</span>
      <span class="text-red-500 font-bold px-0.5" aria-hidden="true">|</span>
      <span class="text-emerald-700 dark:text-emerald-300 bg-emerald-50 dark:bg-emerald-900/40 rounded px-1">{after}</span>
    </div>

    <input
      type="range"
      min="1"
      max={Math.max(1, segment.source_text.length - 1)}
      bind:value={pos}
      aria-label="Split position (character index)"
      class="w-full accent-brand-500 mb-4 cursor-pointer"
    />

    <div class="grid grid-cols-2 gap-3 mb-6 text-xs">
      <div class="p-2.5 rounded bg-blue-50/50 dark:bg-blue-950/20 border border-blue-200 dark:border-blue-900">
        <div class="font-semibold text-blue-700 dark:text-blue-400 mb-1">Segment {segment.seq_number} (Part 1)</div>
        <div dir="auto" class="text-gray-800 dark:text-gray-200 italic line-clamp-3">{before}</div>
      </div>
      <div class="p-2.5 rounded bg-emerald-50/50 dark:bg-emerald-950/20 border border-emerald-200 dark:border-emerald-900">
        <div class="font-semibold text-emerald-700 dark:text-emerald-400 mb-1">Segment {segment.seq_number + 1} (Part 2)</div>
        <div dir="auto" class="text-gray-800 dark:text-gray-200 italic line-clamp-3">{after}</div>
      </div>
    </div>

    <div class="flex items-center justify-end gap-2">
      <button
        type="button"
        onclick={oncancel}
        class="px-3 py-1.5 text-xs text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg transition-colors"
      >
        Cancel
      </button>
      <button
        type="button"
        onclick={() => onconfirm(pos)}
        class="px-4 py-1.5 text-xs font-medium text-white bg-brand-500 hover:bg-brand-600 rounded-lg shadow-sm transition-colors"
      >
        Split Segment
      </button>
    </div>
  </div>
</div>
