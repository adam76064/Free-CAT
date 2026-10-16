<script lang="ts">
  // Project dashboard: shows files, progress, actions, and detailed analysis report.
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { open, save } from '@tauri-apps/plugin-dialog';
  import { currentProject, currentFiles, projectSettings, parseSettings } from '$lib/stores/project';
  import { openProject, importFile, exportFile, analyzeProject, importSdlppx, exportSdlrpx, exportFcat } from '$lib/ipc/project';
  import { availableTms } from '$lib/stores/tm';
  import { listTms } from '$lib/ipc/tm';
  import QuoteInvoiceModal from '$lib/components/analysis/QuoteInvoiceModal.svelte';

  $: projectId = $page.params.id as string;

  let loading = true;
  let importing = false;
  let exportingSdlrpx = false;
  let exportingFcat = false;
  let showInvoiceModal = false;
  let analysis: any = null;
  let error = '';
  let showAnalysisModal = false;

  onMount(async () => {
    const pid = projectId;
    if (!pid) return;
    try {
      const [proj, files] = await openProject(pid);
      currentProject.set(proj);
      currentFiles.set(files);
      projectSettings.set(parseSettings(proj.settings));

      const tms = await listTms();
      availableTms.set(tms);

      if (files.length > 0) {
        analysis = await analyzeProject({
          project_id: proj.id,
          file_ids: files.map((f) => f.id),
        });
      }
    } catch (e: any) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  async function handleImportFile() {
    if (!$currentProject) return;
    const filePath = await open({
      title: 'Import Source File or Trados Package',
      filters: [
        {
          name: 'All Supported Formats',
          extensions: ['txt', 'po', 'pot', 'xliff', 'xlf', 'sdlxliff', 'sdlppx', 'docx', 'xlsx', 'pptx', 'json', 'yaml', 'yml', 'html', 'htm', 'md', 'markdown', 'srt', 'vtt']
        },
        { name: 'Trados Studio Package (*.sdlppx)', extensions: ['sdlppx'] },
        { name: 'Word Documents (*.docx)', extensions: ['docx'] },
        { name: 'Excel Spreadsheets (*.xlsx)', extensions: ['xlsx'] },
        { name: 'PowerPoint Presentations (*.pptx)', extensions: ['pptx'] },
        { name: 'JSON / YAML Localization (*.json, *.yaml, *.yml)', extensions: ['json', 'yaml', 'yml'] },
        { name: 'HTML Documents (*.html, *.htm)', extensions: ['html', 'htm'] },
        { name: 'Markdown (*.md, *.markdown)', extensions: ['md', 'markdown'] },
        { name: 'Subtitles (*.srt, *.vtt)', extensions: ['srt', 'vtt'] },
        { name: 'Gettext (*.po, *.pot)', extensions: ['po', 'pot'] },
        { name: 'XLIFF (*.xliff, *.xlf, *.sdlxliff)', extensions: ['xliff', 'xlf', 'sdlxliff'] },
        { name: 'Plain Text (*.txt)', extensions: ['txt'] },
      ],
    });

    if (!filePath || typeof filePath !== 'string') return;

    importing = true;
    error = '';
    try {
      if (filePath.toLowerCase().endsWith('.sdlppx')) {
        const results = await importSdlppx({
          project_id: $currentProject.id,
          package_path: filePath,
        });
        currentFiles.update((files) => [...results.map(r => r.file), ...files]);
      } else {
        const result = await importFile({
          project_id: $currentProject.id,
          file_path: filePath,
        });
        currentFiles.update((files) => [result.file, ...files]);
      }

      analysis = await analyzeProject({
        project_id: $currentProject.id,
        file_ids: $currentFiles.map((f) => f.id),
      });
    } catch (e: any) {
      error = `Import failed: ${e}`;
    } finally {
      importing = false;
    }
  }

  async function handleExportSdlrpx() {
    if (!$currentProject) return;
    exportingSdlrpx = true;
    error = '';
    try {
      const outPath = await exportSdlrpx({ project_id: $currentProject.id });
      alert(`Exported Trados return package to: ${outPath}`);
    } catch (e: any) {
      error = `Return package export failed: ${e}`;
    } finally {
      exportingSdlrpx = false;
    }
  }

  async function handleExportFcat() {
    if (!$currentProject) return;
    exportingFcat = true;
    error = '';
    try {
      const sanitizedName = $currentProject.name.replace(/[^a-zA-Z0-9_-]/g, '_');
      const outPath = `${sanitizedName}.fcat`;
      await exportFcat($currentProject.id, outPath);
      alert(`Project successfully exported to single-file .fcat archive:\n${outPath}`);
    } catch (e: any) {
      error = `Exporting .fcat failed: ${e}`;
    } finally {
      exportingFcat = false;
    }
  }

  function openEditor(fileId: string) {
    goto(`/project/${projectId}/editor?file=${fileId}`);
  }

  async function handleExportSingleFile(file: any) {
    if (!$currentProject) return;
    try {
      const ext = file.filename.split('.').pop() || '';
      const chosenPath = await save({
        defaultPath: file.filename,
        filters: ext ? [{ name: ext.toUpperCase(), extensions: [ext] }] : [],
      });
      if (!chosenPath) return;

      const outPath = await exportFile({
        file_id: file.id,
        project_id: $currentProject.id,
        out_path: chosenPath,
      });
      alert(`Export Successful!\nTranslated document saved to:\n${outPath}`);
    } catch (e: any) {
      alert(`Export failed: ${e}`);
    }
  }

  function formatDate(iso: string) {
    return new Date(iso).toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' });
  }

  const FILE_ICONS: Record<string, string> = {
    txt: '📄', po: '🌐', pot: '🌐', docx: '📝',
    xlsx: '📊', pptx: '📽', json: '⚙️', yaml: '⚙️', yml: '⚙️',
    html: '🌐', htm: '🌐', md: '📑', markdown: '📑',
    srt: '🎬', vtt: '🎬',
    xliff: '🔧', xlf: '🔧', sdlxliff: '🔧', mqxliff: '🔧',
  };

  interface AnalysisCard {
    label: string;
    key: string;
    color: string;
    suffix?: string;
  }

  const analysisSummary: AnalysisCard[] = [
    { label: 'Total Words',    key: 'total_words',      color: 'bg-gray-100 dark:bg-gray-800' },
    { label: 'Exact Matches',  key: 'exact_words',      color: 'bg-green-50 dark:bg-green-900/20' },
    { label: 'Fuzzy Matches',  key: 'fuzzy_words',      color: 'bg-blue-50 dark:bg-blue-900/20' },
    { label: 'TM Savings',     key: 'tm_savings_pct',   color: 'bg-brand-50 dark:bg-brand-900/20', suffix: '%' },
  ];
</script>

<div class="h-full overflow-y-auto bg-gray-50 dark:bg-gray-950">
  {#if loading}
    <div class="flex items-center justify-center h-full text-gray-400">Loading project…</div>
  {:else if error}
    <div class="flex items-center justify-center h-full">
      <div class="text-red-500 text-sm">{error}</div>
    </div>
  {:else if $currentProject}
    <div class="max-w-4xl mx-auto px-6 py-8">

      <!-- Breadcrumb + header -->
      <div class="mb-8">
        <div class="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400 mb-1">
          <button on:click={() => goto('/')} class="hover:text-brand-500 transition-colors">Projects</button>
          <span>/</span>
          <span class="text-gray-900 dark:text-white font-medium">{$currentProject.name}</span>
        </div>
        <div class="flex items-center justify-between">
          <div>
            <h1 class="text-2xl font-semibold text-gray-900 dark:text-white">{$currentProject.name}</h1>
            <div class="flex items-center gap-3 mt-1">
              <span class="text-sm text-gray-500 dark:text-gray-400">
                <span class="font-mono bg-gray-100 dark:bg-gray-800 px-1.5 py-0.5 rounded">{$currentProject.src_lang}</span>
                <span class="mx-1.5 text-gray-400">→</span>
                <span class="font-mono bg-gray-100 dark:bg-gray-800 px-1.5 py-0.5 rounded">{$currentProject.tgt_lang}</span>
              </span>
              <span class="text-xs text-gray-400">Modified {formatDate($currentProject.modified_at)}</span>
            </div>
          </div>
          <div class="flex items-center gap-2">
            <button
              on:click={() => showInvoiceModal = true}
              class="px-3.5 py-2 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700
                     hover:bg-gray-50 dark:hover:bg-gray-750 text-gray-700 dark:text-gray-200
                     text-sm font-medium rounded-lg transition-colors shadow-xs flex items-center gap-1.5"
              title="Open Freelancer Quoting & Invoicing Suite"
            >
              <span>💼 Quoting & Invoicing</span>
            </button>
            <button
              on:click={handleExportFcat}
              disabled={exportingFcat}
              class="px-3.5 py-2 bg-indigo-600 hover:bg-indigo-700 disabled:opacity-50
                     text-white text-sm font-medium rounded-lg transition-colors shadow-sm flex items-center gap-1.5"
              title="Export complete project into a portable .fcat single-file archive"
            >
              <span>{exportingFcat ? 'Packaging…' : '📦 Export .fcat'}</span>
            </button>
            {#if analysis}
              <button
                on:click={() => showAnalysisModal = true}
                class="px-3.5 py-2 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700
                       hover:bg-gray-50 dark:hover:bg-gray-750 text-gray-700 dark:text-gray-200
                       text-sm font-medium rounded-lg transition-colors shadow-xs"
              >
                📊 Analysis Report
              </button>
            {/if}
            {#if $currentFiles.some(f => f.filename.endsWith('.sdlxliff') || f.format === 'sdlxliff')}
              <button
                on:click={handleExportSdlrpx}
                disabled={exportingSdlrpx}
                class="px-3.5 py-2 bg-emerald-600 hover:bg-emerald-700 disabled:opacity-50
                       text-white text-sm font-medium rounded-lg transition-colors shadow-sm flex items-center gap-1.5"
                title="Create a Trados return package (.sdlrpx) for delivery"
              >
                <span>{exportingSdlrpx ? 'Packing…' : '📦 Return Package (.sdlrpx)'}</span>
              </button>
            {/if}
            <button
              on:click={handleImportFile}
              disabled={importing}
              class="px-4 py-2 bg-brand-500 hover:bg-brand-600 disabled:opacity-50
                     text-white text-sm font-medium rounded-lg transition-colors shadow-sm"
            >
              {importing ? 'Importing…' : '+ Import File'}
            </button>
          </div>
        </div>
      </div>

      <!-- Analysis summary cards -->
      {#if analysis}
        <div class="grid grid-cols-4 gap-4 mb-8">
          {#each analysisSummary as card}
            <div class="p-4 rounded-xl {card.color} border border-black/5 dark:border-white/5">
              <div class="text-2xl font-bold text-gray-900 dark:text-white">
                {(analysis[card.key] ?? 0).toLocaleString()}{card.suffix ?? ''}
              </div>
              <div class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">{card.label}</div>
            </div>
          {/each}
        </div>
      {/if}

      <!-- Files list -->
      <div>
        <h2 class="text-base font-semibold text-gray-900 dark:text-white mb-3">
          Files ({$currentFiles.length})
        </h2>

        {#if $currentFiles.length === 0}
          <div class="border-2 border-dashed border-gray-200 dark:border-gray-700 rounded-xl
                      flex flex-col items-center justify-center py-12 gap-3 bg-white dark:bg-gray-900">
            <span class="text-4xl">📂</span>
            <p class="text-sm font-medium text-gray-700 dark:text-gray-300">
              No files imported yet
            </p>
            <p class="text-xs text-gray-500 dark:text-gray-400">
              Support Word (.docx), HTML (.html), Markdown (.md), Subtitles (.srt/.vtt), XLIFF, PO, and TXT.
            </p>
            <button
              on:click={handleImportFile}
              class="mt-2 px-4 py-1.5 bg-brand-500 hover:bg-brand-600 text-white text-xs font-medium rounded-lg transition-colors"
            >
              Import your first file
            </button>
          </div>
        {:else}
          <div class="space-y-2">
            {#each $currentFiles as file (file.id)}
              <div class="flex items-center gap-4 p-4 bg-white dark:bg-gray-900 rounded-xl
                          border border-gray-200 dark:border-gray-800
                          hover:border-brand-300 dark:hover:border-brand-700 transition-colors shadow-xs">
                <span class="text-2xl">{FILE_ICONS[file.format] ?? '📄'}</span>
                <div class="flex-1 min-w-0">
                  <div class="font-medium text-gray-900 dark:text-white truncate">{file.filename}</div>
                  <div class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                    {file.seg_count} segments · {file.word_count.toLocaleString()} words · <span class="font-mono">{file.format.toUpperCase()}</span>
                  </div>
                </div>
                <div class="flex items-center gap-2">
                  <button
                    on:click={() => handleExportSingleFile(file)}
                    class="px-3 py-1.5 text-xs font-medium border border-gray-300 dark:border-gray-700
                           hover:bg-gray-50 dark:hover:bg-gray-800 text-gray-700 dark:text-gray-300 rounded-lg transition-colors shadow-xs"
                    title="Export translated file to your computer"
                  >
                    ⬇ Export
                  </button>
                  <button
                    on:click={() => openEditor(file.id)}
                    class="px-3.5 py-1.5 text-sm font-medium bg-brand-500 hover:bg-brand-600
                           text-white rounded-lg transition-colors shadow-xs"
                  >
                    Translate →
                  </button>
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<!-- Detailed Analysis Report Modal -->
{#if showAnalysisModal && analysis}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4">
    <div class="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 shadow-2xl max-w-2xl w-full p-6 animate-in fade-in zoom-in-95">
      <div class="flex items-center justify-between pb-4 border-b border-gray-200 dark:border-gray-800 mb-4">
        <div>
          <h3 class="text-lg font-bold text-gray-900 dark:text-white">Translation Analysis Report</h3>
          <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
            Project: {$currentProject?.name} ({$currentProject?.src_lang} → {$currentProject?.tgt_lang})
          </p>
        </div>
        <button
          on:click={() => showAnalysisModal = false}
          class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 text-sm p-1 rounded"
        >
          ✕
        </button>
      </div>

      <!-- Analysis Table -->
      <div class="overflow-x-auto mb-6">
        <table class="w-full text-xs text-left">
          <thead class="bg-gray-50 dark:bg-gray-800/80 text-gray-500 dark:text-gray-400 uppercase font-semibold">
            <tr>
              <th class="py-2.5 px-3 rounded-l">Match Category</th>
              <th class="py-2.5 px-3 text-right">Segments</th>
              <th class="py-2.5 px-3 text-right">Words</th>
              <th class="py-2.5 px-3 text-right rounded-r">Percentage</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-gray-100 dark:divide-gray-800">
            <tr>
              <td class="py-2 px-3 font-medium text-emerald-600 dark:text-emerald-400">100% Exact Match</td>
              <td class="py-2 px-3 text-right text-gray-700 dark:text-gray-300">{analysis.exact_segments.toLocaleString()}</td>
              <td class="py-2 px-3 text-right font-medium text-gray-900 dark:text-white">{analysis.exact_words.toLocaleString()}</td>
              <td class="py-2 px-3 text-right text-gray-500">
                {analysis.total_words > 0 ? ((analysis.exact_words / analysis.total_words) * 100).toFixed(1) : 0}%
              </td>
            </tr>
            <tr>
              <td class="py-2 px-3 font-medium text-blue-600 dark:text-blue-400">75%–99% Fuzzy Match</td>
              <td class="py-2 px-3 text-right text-gray-700 dark:text-gray-300">{analysis.fuzzy_segments.toLocaleString()}</td>
              <td class="py-2 px-3 text-right font-medium text-gray-900 dark:text-white">{analysis.fuzzy_words.toLocaleString()}</td>
              <td class="py-2 px-3 text-right text-gray-500">
                {analysis.total_words > 0 ? ((analysis.fuzzy_words / analysis.total_words) * 100).toFixed(1) : 0}%
              </td>
            </tr>
            <tr>
              <td class="py-2 px-3 font-medium text-amber-600 dark:text-amber-400">New / No Match (&lt; 75%)</td>
              <td class="py-2 px-3 text-right text-gray-700 dark:text-gray-300">{analysis.new_segments.toLocaleString()}</td>
              <td class="py-2 px-3 text-right font-medium text-gray-900 dark:text-white">{analysis.new_words.toLocaleString()}</td>
              <td class="py-2 px-3 text-right text-gray-500">
                {analysis.total_words > 0 ? ((analysis.new_words / analysis.total_words) * 100).toFixed(1) : 0}%
              </td>
            </tr>
            <tr class="font-bold bg-gray-50/50 dark:bg-gray-800/40">
              <td class="py-2.5 px-3 text-gray-900 dark:text-white">Total</td>
              <td class="py-2.5 px-3 text-right text-gray-900 dark:text-white">{analysis.total_segments.toLocaleString()}</td>
              <td class="py-2.5 px-3 text-right text-gray-900 dark:text-white">{analysis.total_words.toLocaleString()}</td>
              <td class="py-2.5 px-3 text-right text-gray-900 dark:text-white">100.0%</td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- TM Leverage Summary -->
      <div class="p-4 rounded-xl bg-brand-50 dark:bg-brand-950/30 border border-brand-200 dark:border-brand-900 flex items-center justify-between mb-6">
        <div>
          <div class="text-xs font-semibold text-brand-800 dark:text-brand-300 uppercase tracking-wide">
            Estimated TM Leverage / Savings
          </div>
          <div class="text-xs text-brand-600 dark:text-brand-400 mt-0.5">
            Based on 100% exact matches + 50% weighted fuzzy matches
          </div>
        </div>
        <div class="text-3xl font-extrabold text-brand-600 dark:text-brand-400">
          {analysis.tm_savings_pct}%
        </div>
      </div>

      <div class="flex justify-end">
        <button
          on:click={() => showAnalysisModal = false}
          class="px-4 py-2 bg-brand-500 hover:bg-brand-600 text-white text-xs font-medium rounded-lg transition-colors"
        >
          Close Report
        </button>
      </div>
    </div>
  </div>
{/if}

<QuoteInvoiceModal
  isOpen={showInvoiceModal}
  onClose={() => showInvoiceModal = false}
  projectId={projectId}
  fileIds={$currentFiles.map(f => f.id)}
/>
