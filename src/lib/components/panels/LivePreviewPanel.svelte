<script lang="ts">
  // LivePreviewPanel.svelte — Real-time in-context document flow preview
  import { segments, selectedIndex, type Segment } from '$lib/stores/segments';
  import { currentFile, currentProject } from '$lib/stores/project';

  interface Props {
    onSelectSegment?: (index: number) => void;
  }

  let { onSelectSegment }: Props = $props();

  let viewMode = $state<'target' | 'source' | 'bilingual'>('target');
  let fontSize = $state<'sm' | 'base' | 'lg'>('base');

  let activeIndex = $derived($selectedIndex);
  let totalSegments = $derived($segments.length);
  let translatedCount = $derived($segments.filter(s => s.status === 'translated').length);
  let progressPct = $derived(totalSegments > 0 ? Math.round((translatedCount / totalSegments) * 100) : 0);

  function handleSegmentClick(index: number) {
    if (onSelectSegment) {
      onSelectSegment(index);
    }
  }
</script>

<div class="flex flex-col h-full bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100">
  <!-- Preview Toolbar -->
  <div class="p-2.5 border-b border-gray-200 dark:border-gray-800 bg-gray-50 dark:bg-gray-850 flex items-center justify-between gap-2 shrink-0 text-xs">
    <div class="flex items-center gap-1.5">
      <span class="font-semibold text-gray-700 dark:text-gray-300">View:</span>
      <div class="inline-flex rounded-lg border border-gray-200 dark:border-gray-700 p-0.5 bg-gray-100 dark:bg-gray-800">
        <button
          onclick={() => viewMode = 'target'}
          class="px-2 py-0.5 rounded text-[11px] font-medium transition-colors {viewMode === 'target' ? 'bg-white dark:bg-gray-700 text-brand-600 dark:text-brand-400 shadow-xs' : 'text-gray-600 dark:text-gray-400'}"
        >
          Target Flow
        </button>
        <button
          onclick={() => viewMode = 'bilingual'}
          class="px-2 py-0.5 rounded text-[11px] font-medium transition-colors {viewMode === 'bilingual' ? 'bg-white dark:bg-gray-700 text-brand-600 dark:text-brand-400 shadow-xs' : 'text-gray-600 dark:text-gray-400'}"
        >
          Bilingual
        </button>
        <button
          onclick={() => viewMode = 'source'}
          class="px-2 py-0.5 rounded text-[11px] font-medium transition-colors {viewMode === 'source' ? 'bg-white dark:bg-gray-700 text-brand-600 dark:text-brand-400 shadow-xs' : 'text-gray-600 dark:text-gray-400'}"
        >
          Source
        </button>
      </div>
    </div>

    <div class="flex items-center gap-2">
      <!-- Font size toggle -->
      <div class="flex items-center gap-0.5 text-[11px] text-gray-500">
        <button
          onclick={() => fontSize = 'sm'}
          class="px-1.5 py-0.5 rounded {fontSize === 'sm' ? 'font-bold text-brand-600 bg-gray-200 dark:bg-gray-700' : ''}"
        >
          A-
        </button>
        <button
          onclick={() => fontSize = 'base'}
          class="px-1.5 py-0.5 rounded {fontSize === 'base' ? 'font-bold text-brand-600 bg-gray-200 dark:bg-gray-700' : ''}"
        >
          A
        </button>
        <button
          onclick={() => fontSize = 'lg'}
          class="px-1.5 py-0.5 rounded {fontSize === 'lg' ? 'font-bold text-brand-600 bg-gray-200 dark:bg-gray-700' : ''}"
        >
          A+
        </button>
      </div>

      <span class="text-[11px] font-mono text-gray-400">
        {progressPct}% Done
      </span>
    </div>
  </div>

  <!-- Document Flow Canvas -->
  <div class="flex-1 overflow-y-auto p-4 leading-relaxed font-editor select-text
              {fontSize === 'sm' ? 'text-xs leading-5' : fontSize === 'lg' ? 'text-base leading-7' : 'text-sm leading-6'}">
    {#if $segments.length === 0}
      <div class="flex items-center justify-center h-48 text-gray-400 text-sm">
        No segments available for preview.
      </div>
    {:else if viewMode === 'target'}
      <!-- Continuous Document Flow in Target Language -->
      <div class="prose dark:prose-invert max-w-none space-y-3">
        <p class="text-justify">
          {#each $segments as seg, idx (seg.id)}
            <span
              id="preview-seg-{idx}"
              onclick={() => handleSegmentClick(idx)}
              onkeydown={(e) => e.key === 'Enter' && handleSegmentClick(idx)}
              role="button"
              tabindex="0"
              title="Segment #{seg.seq_number} ({seg.status}) - Click to jump"
              class="cursor-pointer transition-colors duration-150 rounded px-1 py-0.5 inline mr-1
                     {idx === activeIndex
                       ? 'bg-brand-500/20 text-brand-900 dark:text-brand-200 ring-2 ring-brand-500/50 font-medium'
                       : seg.status === 'translated'
                       ? 'hover:bg-gray-100 dark:hover:bg-gray-800'
                       : 'text-gray-400 dark:text-gray-500 italic hover:bg-amber-50 dark:hover:bg-amber-950/30'}"
            >
              {seg.target_text ? seg.target_text : `[#${seg.seq_number}: ${seg.source_text}]`}
            </span>
          {/each}
        </p>
      </div>
    {:else if viewMode === 'source'}
      <!-- Continuous Document Flow in Source Language -->
      <div class="prose dark:prose-invert max-w-none space-y-3">
        <p class="text-justify text-gray-800 dark:text-gray-200">
          {#each $segments as seg, idx (seg.id)}
            <span
              id="preview-seg-{idx}"
              onclick={() => handleSegmentClick(idx)}
              onkeydown={(e) => e.key === 'Enter' && handleSegmentClick(idx)}
              role="button"
              tabindex="0"
              title="Segment #{seg.seq_number} - Click to jump"
              class="cursor-pointer transition-colors duration-150 rounded px-1 py-0.5 inline mr-1
                     {idx === activeIndex
                       ? 'bg-brand-500/20 text-brand-900 dark:text-brand-200 ring-2 ring-brand-500/50 font-medium'
                       : 'hover:bg-gray-100 dark:hover:bg-gray-800'}"
            >
              {seg.source_text}
            </span>
          {/each}
        </p>
      </div>
    {:else}
      <!-- Bilingual Paragraph Flow -->
      <div class="space-y-3">
        {#each $segments as seg, idx (seg.id)}
          <div
            id="preview-seg-{idx}"
            onclick={() => handleSegmentClick(idx)}
            onkeydown={(e) => e.key === 'Enter' && handleSegmentClick(idx)}
            role="button"
            tabindex="0"
            class="p-2.5 rounded-lg border transition-all cursor-pointer
                   {idx === activeIndex
                     ? 'border-brand-500 bg-brand-50/50 dark:bg-brand-950/20 shadow-xs ring-1 ring-brand-500/40'
                     : 'border-gray-200 dark:border-gray-800 hover:bg-gray-50 dark:hover:bg-gray-850'}"
          >
            <div class="flex items-center justify-between text-[11px] text-gray-400 mb-1">
              <span class="font-mono">#{seg.seq_number}</span>
              <span class="capitalize {seg.status === 'translated' ? 'text-emerald-600 dark:text-emerald-400 font-semibold' : ''}">{seg.status}</span>
            </div>
            <div class="text-xs text-gray-500 dark:text-gray-400 mb-1.5 font-normal">
              {seg.source_text}
            </div>
            <div class="text-sm text-gray-900 dark:text-gray-100 font-medium">
              {seg.target_text || '—'}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Footer info -->
  <div class="p-2 border-t border-gray-200 dark:border-gray-800 bg-gray-50 dark:bg-gray-850 flex items-center justify-between text-[11px] text-gray-500">
    <span>{$currentFile?.filename || 'Document'}</span>
    <span>Click any sentence to jump to segment</span>
  </div>
</div>
