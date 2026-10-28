<script lang="ts">
  import ShowMore from '$lib/components/ui/ShowMore.svelte';
  // QaPanel — displays QA check results grouped by severity with CSV/HTML export.
  import { qaResults, qaRunning, qaErrors, qaWarnings, QA_CHECK_LABELS } from '$lib/stores/qa';
  import { segments } from '$lib/stores/segments';

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

  function exportQaCsv() {
    if ($qaResults.length === 0) return;
    const segMap = new Map($segments.map((s) => [s.id, s]));
    const rows = [
      ['Segment #', 'Severity', 'Check', 'Message', 'Source Text', 'Target Text'],
      ...$qaResults.map((r) => {
        const seg = r.segment_id ? segMap.get(r.segment_id) : undefined;
        return [
          seg ? String(seg.seq_number) : '',
          r.severity.toUpperCase(),
          QA_CHECK_LABELS[r.check_name] ?? r.check_name,
          `"${(r.message || '').replace(/"/g, '""')}"`,
          `"${(seg?.source_text || '').replace(/"/g, '""')}"`,
          `"${(seg?.target_text || '').replace(/"/g, '""')}"`,
        ];
      }),
    ];
    const csvContent = rows.map((e) => e.join(',')).join('\n');
    const blob = new Blob([csvContent], { type: 'text/csv;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `FreeCAT_QA_Report_${new Date().toISOString().slice(0, 10)}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  }

  function exportQaHtml() {
    if ($qaResults.length === 0) return;
    const segMap = new Map($segments.map((s) => [s.id, s]));
    const errCount = $qaErrors.length;
    const warnCount = $qaWarnings.length;
    const dateStr = new Date().toLocaleString();

    const html = `<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <title>Free-CAT QA Report</title>
  <style>
    body { font-family: system-ui, -apple-system, sans-serif; padding: 24px; color: #1f2937; line-height: 1.5; }
    h1 { font-size: 20px; margin-bottom: 4px; }
    .meta { font-size: 12px; color: #6b7280; margin-bottom: 16px; }
    .badge { display: inline-block; padding: 2px 8px; border-radius: 4px; font-weight: 600; font-size: 11px; }
    .badge-err { background: #fee2e2; color: #991b1b; }
    .badge-warn { background: #fef3c7; color: #92400e; }
    table { width: 100%; border-collapse: collapse; margin-top: 16px; font-size: 12px; }
    th, td { border: 1px solid #e5e7eb; padding: 8px 10px; text-align: left; vertical-align: top; }
    th { background: #f9fafb; font-weight: 600; }
    tr:nth-child(even) { background: #fafafa; }
  </style>
</head>
<body>
  <h1>Free-CAT Quality Assurance Report</h1>
  <div class="meta">Generated: ${dateStr} • Errors: ${errCount} • Warnings: ${warnCount}</div>
  <table>
    <thead>
      <tr>
        <th style="width: 50px;">Seg #</th>
        <th style="width: 70px;">Severity</th>
        <th style="width: 140px;">Check</th>
        <th>Message</th>
        <th>Source</th>
        <th>Target</th>
      </tr>
    </thead>
    <tbody>
      ${$qaResults
        .map((r) => {
          const seg = r.segment_id ? segMap.get(r.segment_id) : undefined;
          return `<tr>
          <td>${seg ? seg.seq_number : '-'}</td>
          <td><span class="badge ${r.severity === 'error' ? 'badge-err' : 'badge-warn'}">${r.severity.toUpperCase()}</span></td>
          <td>${QA_CHECK_LABELS[r.check_name] ?? r.check_name}</td>
          <td>${r.message}</td>
          <td>${seg?.source_text ?? ''}</td>
          <td>${seg?.target_text ?? ''}</td>
        </tr>`;
        })
        .join('')}
    </tbody>
  </table>
</body>
</html>`;

    const blob = new Blob([html], { type: 'text/html;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `FreeCAT_QA_Report_${new Date().toISOString().slice(0, 10)}.html`;
    a.click();
    URL.revokeObjectURL(url);
  }
</script>

<div class="flex flex-col h-full">
  <!-- Header -->
  <div class="flex items-center justify-between px-3 py-2 border-b border-gray-200 dark:border-gray-800 shrink-0">
    <div class="flex items-center gap-1.5">
      <span class="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">QA</span>
      {#if $qaResults.length > 0}
        <div class="flex items-center gap-1 ml-1">
          <button
            type="button"
            onclick={exportQaCsv}
            class="px-1.5 py-0.5 text-[10px] bg-gray-100 hover:bg-gray-200 dark:bg-gray-800 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-300 rounded cursor-pointer transition-colors"
            title="Export QA Report to CSV"
          >
            CSV
          </button>
          <button
            type="button"
            onclick={exportQaHtml}
            class="px-1.5 py-0.5 text-[10px] bg-gray-100 hover:bg-gray-200 dark:bg-gray-800 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-300 rounded cursor-pointer transition-colors"
            title="Export QA Report to HTML Print"
          >
            HTML
          </button>
        </div>
      {/if}
    </div>
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
               text-white rounded-md transition-colors cursor-pointer"
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
            class="w-full text-left px-3 py-2 hover:bg-red-50 dark:hover:bg-red-900/10 transition-colors cursor-pointer"
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
            class="w-full text-left px-3 py-2 hover:bg-yellow-50 dark:hover:bg-yellow-900/10 transition-colors cursor-pointer"
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
