<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { segments, selectedIndex } from '$lib/stores/segments';
  import { currentProject, currentFiles, currentFileId, currentFile, projectSettings } from '$lib/stores/project';
  import { getSegments } from '$lib/ipc/segment';
  import { openProject, exportFile } from '$lib/ipc/project';
  import { listTms } from '$lib/ipc/tm';
  import { availableTms } from '$lib/stores/tm';
  import SegmentGrid from '$lib/components/editor/SegmentGrid.svelte';
  import { parseSettings } from '$lib/stores/project';

  $: projectId = $page.params.id as string;
  $: fileId = $page.url.searchParams.get('file');

  let loading = true;
  let error = '';
  let exporting = false;

  onMount(async () => {
    const pid = projectId;
    const fid = fileId;

    if (!fid) {
      goto(`/project/${pid}`);
      return;
    }

    try {
      if (!$currentProject || $currentProject.id !== pid) {
        const [proj, files] = await openProject(pid);
        currentProject.set(proj);
        currentFiles.set(files);
        projectSettings.set(parseSettings(proj.settings));
      }

      currentFileId.set(fid);

      const tms = await listTms();
      availableTms.set(tms);

      const segs = await getSegments(fid, pid);
      segments.set(segs);
      selectedIndex.set(0);
    } catch (e: any) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  async function handleExport() {
    const pid = projectId;
    const fid = fileId;
    if (!pid || !fid) return;
    exporting = true;
    try {
      const outPath = await exportFile({ file_id: fid, project_id: pid });
      console.log('Exported to:', outPath);
    } catch (e: any) {
      error = `Export failed: ${e}`;
    } finally {
      exporting = false;
    }
  }
</script>

<div class="h-full flex flex-col bg-white dark:bg-gray-950">

  <!-- Editor toolbar -->
  <header class="flex items-center gap-3 px-4 py-2 border-b border-gray-200 dark:border-gray-800 shrink-0 bg-white dark:bg-gray-950">
    <button
      on:click={() => goto(`/project/${projectId}`)}
      class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors text-sm"
      title="Back to project"
    >
      ← Back
    </button>

    <div class="flex-1 min-w-0">
      {#if $currentFile}
        <div class="flex items-center gap-2">
          <span class="font-medium text-gray-900 dark:text-white text-sm truncate">
            {$currentFile.filename}
          </span>
          <span class="text-xs text-gray-500 dark:text-gray-400">
            {$currentProject?.src_lang} → {$currentProject?.tgt_lang}
          </span>
        </div>
      {/if}
    </div>

    {#if error}
      <span class="text-xs text-red-500">{error}</span>
    {/if}

    <button
      on:click={handleExport}
      disabled={exporting}
      class="px-3 py-1.5 text-xs font-medium border border-gray-300 dark:border-gray-600
             text-gray-700 dark:text-gray-300 rounded-lg
             hover:bg-gray-50 dark:hover:bg-gray-800 disabled:opacity-50 transition-colors"
    >
      {exporting ? 'Exporting…' : '⬇ Export'}
    </button>
  </header>

  <!-- Body -->
  <div class="flex-1 overflow-hidden">
    {#if loading}
      <div class="flex items-center justify-center h-full text-gray-400 text-sm">
        Loading segments…
      </div>
    {:else if error && !loading}
      <div class="flex items-center justify-center h-full">
        <div class="text-red-500 text-sm">{error}</div>
      </div>
    {:else}
      <SegmentGrid />
    {/if}
  </div>

</div>
