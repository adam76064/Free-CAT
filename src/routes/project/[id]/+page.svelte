<script lang="ts">
  // Project dashboard: shows files, progress, and actions.
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { open } from '@tauri-apps/plugin-dialog';
  import { currentProject, currentFiles, projectSettings, parseSettings } from '$lib/stores/project';
  import { openProject, importFile, exportFile, analyzeProject } from '$lib/ipc/project';
  import { availableTms } from '$lib/stores/tm';
  import { listTms } from '$lib/ipc/tm';

  $: projectId = $page.params.id as string;

  let loading = true;
  let importing = false;
  let analysis: any = null;
  let error = '';

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
      title: 'Import Source File',
      filters: [
        { name: 'All Supported', extensions: ['txt', 'po', 'pot', 'xliff', 'xlf', 'sdlxliff', 'docx'] },
        { name: 'Plain Text', extensions: ['txt'] },
        { name: 'PO/POT', extensions: ['po', 'pot'] },
        { name: 'XLIFF', extensions: ['xliff', 'xlf', 'sdlxliff'] },
        { name: 'Word', extensions: ['docx'] },
      ],
    });

    if (!filePath || typeof filePath !== 'string') return;

    importing = true;
    try {
      const result = await importFile({
        project_id: $currentProject.id,
        file_path: filePath,
      });
      currentFiles.update((files) => [result.file, ...files]);

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

  function openEditor(fileId: string) {
    goto(`/project/${projectId}/editor?file=${fileId}`);
  }

  function formatDate(iso: string) {
    return new Date(iso).toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' });
  }

  const FILE_ICONS: Record<string, string> = {
    txt: '📄', po: '🌐', pot: '🌐', docx: '📝',
    xliff: '🔧', sdlxliff: '🔧', mqxliff: '🔧',
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
                <span class="font-mono">{$currentProject.src_lang}</span>
                <span class="mx-1.5">→</span>
                <span class="font-mono">{$currentProject.tgt_lang}</span>
              </span>
              <span class="text-xs text-gray-400">Modified {formatDate($currentProject.modified_at)}</span>
            </div>
          </div>
          <button
            on:click={handleImportFile}
            disabled={importing}
            class="px-4 py-2 bg-brand-500 hover:bg-brand-600 disabled:opacity-50
                   text-white text-sm font-medium rounded-lg transition-colors"
          >
            {importing ? 'Importing…' : '+ Import File'}
          </button>
        </div>
      </div>

      <!-- Analysis cards -->
      {#if analysis}
        <div class="grid grid-cols-4 gap-4 mb-8">
          {#each analysisSummary as card}
            <div class="p-4 rounded-xl {card.color}">
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
                      flex flex-col items-center justify-center py-12 gap-3">
            <span class="text-3xl">📂</span>
            <p class="text-sm text-gray-500 dark:text-gray-400">
              No files yet. Import a source file to start translating.
            </p>
          </div>
        {:else}
          <div class="space-y-2">
            {#each $currentFiles as file (file.id)}
              <div class="flex items-center gap-4 p-4 bg-white dark:bg-gray-900 rounded-xl
                          border border-gray-200 dark:border-gray-800
                          hover:border-brand-300 dark:hover:border-brand-700 transition-colors">
                <span class="text-2xl">{FILE_ICONS[file.format] ?? '📄'}</span>
                <div class="flex-1 min-w-0">
                  <div class="font-medium text-gray-900 dark:text-white truncate">{file.filename}</div>
                  <div class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                    {file.seg_count} segments · {file.word_count.toLocaleString()} words · {file.format.toUpperCase()}
                  </div>
                </div>
                <button
                  on:click={() => openEditor(file.id)}
                  class="px-3 py-1.5 text-sm font-medium bg-brand-500 hover:bg-brand-600
                         text-white rounded-lg transition-colors"
                >
                  Translate →
                </button>
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>
