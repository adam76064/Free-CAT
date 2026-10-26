<script lang="ts">
  import ShowMore from '$lib/components/ui/ShowMore.svelte';
  // QaPanel — displays QA check results grouped by severity.
  import { qaResults, qaRunning, qaErrors, qaWarnings, QA_CHECK_LABELS } from '$lib/stores/qa';

  interface Props {
    onJumpToSegment: (segmentId: string) => void;
    onRunQa: () => void;
  }
  let { onJumpToSegment, onRunQa }: Props = $props();

  const QA_PAGE = 200;
  let errorLimit = $state(QA_PAGE);
  let warningLimit = $state(QA_PAGE);
  // A fresh QA run replaces the results: start from the first page again
  $effect(() => {
    $qaResults;
    errorLimit = QA_PAGE;
    warningLimit = QA_PAGE;
  });
</script>

<div class="flex flex-col h-full">
  <!-- Header -->
  <div class="flex items-center justify-between px-3 py-2 border-b border-gray-200 dark:border-gray-800 shrink-0">
    <span class="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">QA</span>
    <div class="flex items-center gap-2">
      {#if $qaErrors.length > 0}
        <span class="px-1.5 py-0.5 text-xs rounded bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400 font-medium">
          {$qaErrors.length} error{$qaErrors.length !== 1 ? 's' : ''}
        </span>
      {/if}
      {#if $qaWarnings.length > 0}
        <span class="px-1.5 py-0.5 text-xs rounded bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400 font-medium">
          {$qaWarnings.length} warn
        </span>
      {/if}
      <button
        onclick={onRunQa}
        disabled={$qaRunning}
        class="px-2 py-1 text-xs bg-brand-500 hover:bg-brand-600 disabled:opacity-50
               text-white rounded-md transition-colors"
      >
        {$qaRunning ? 'Running…' : 'Run QA'}
      </button>
    </div>
  </div>

  <!-- Results list -->
  <div class="flex-1 overflow-y-auto">
    {#if $qaResults.length === 0}
      <div class="flex flex-col items-center justify-center h-24 text-gray-400 text-sm gap-1">
        <span>No QA issues</span>
        <span class="text-xs">Click "Run QA" to check</span>
      </div>
    {:else}
      <!-- Errors first -->
      {#if $qaErrors.length > 0}
        <div class="px-3 pt-2 pb-1">
          <span class="text-xs font-semibold text-red-600 dark:text-red-400 uppercase tracking-wide">
            Errors ({$qaErrors.length})
          </span>
        </div>
        {#each $qaErrors.slice(0, errorLimit) as result}
          <button
            onclick={() => result.segment_id && onJumpToSegment(result.segment_id)}
            class="w-full text-left px-3 py-2 hover:bg-red-50 dark:hover:bg-red-900/10 transition-colors"
          >
            <div class="flex items-start gap-2">
              <span class="text-red-500 mt-0.5 shrink-0">●</span>
              <div>
                <div class="text-xs font-medium text-gray-700 dark:text-gray-300">
                  {QA_CHECK_LABELS[result.check_name] ?? result.check_name}
                </div>
                <div class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">{result.message}</div>
              </div>
            </div>
          </button>
        {/each}
        <ShowMore shown={errorLimit} total={$qaErrors.length} step={QA_PAGE} onmore={() => (errorLimit += QA_PAGE)} />
      {/if}

      <!-- Warnings -->
      {#if $qaWarnings.length > 0}
        <div class="px-3 pt-2 pb-1">
          <span class="text-xs font-semibold text-yellow-600 dark:text-yellow-400 uppercase tracking-wide">
            Warnings ({$qaWarnings.length})
          </span>
        </div>
        {#each $qaWarnings.slice(0, warningLimit) as result}
          <button
            onclick={() => result.segment_id && onJumpToSegment(result.segment_id)}
            class="w-full text-left px-3 py-2 hover:bg-yellow-50 dark:hover:bg-yellow-900/10 transition-colors"
          >
            <div class="flex items-start gap-2">
              <span class="text-yellow-500 mt-0.5 shrink-0">▲</span>
              <div>
                <div class="text-xs font-medium text-gray-700 dark:text-gray-300">
                  {QA_CHECK_LABELS[result.check_name] ?? result.check_name}
                </div>
                <div class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">{result.message}</div>
              </div>
            </div>
          </button>
        {/each}
        <ShowMore shown={warningLimit} total={$qaWarnings.length} step={QA_PAGE} onmore={() => (warningLimit += QA_PAGE)} />
      {/if}
    {/if}
  </div>
</div>
