<script lang="ts">
  // The main CAT editor page — loads segments and renders the SegmentGrid.
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { segments, selectedIndex } from '$lib/stores/segments';
  import { currentProject, currentFiles, currentFileId, currentFile, projectSettings } from '$lib/stores/project';
  import { getSegments } from '$lib/ipc/segment';
  import { openProject } from '$lib/ipc/project';
  import { listTms } from '$lib/ipc/tm';
  import { availableTms } from '$lib/stores/tm';
  import SegmentGrid from '$lib/components/editor/SegmentGrid.svelte';
  import { exportFile } from '$lib/ipc/project';
  import { parseSettings } from '$lib/stores/project';

  const projectId = $derived($page.params.id);
  const fileId    = $derived($page.url.searchParams.get('file'));

  let loading = $state(true);
  let error   = $state<string | null>(null);
  let exporting = $state(false);

  onMount(async () => {
    if (!fileId) { goto(`/project/${projectId}`); return; }

    try {
      // Load project if not already loaded
      if (!$currentProject || $currentProject.id !== projectId) {
        const [proj, files] = await openProject(projectId);
        currentProject.set(proj);
        currentFiles.set(files);
        projectSettings.set(parseSettings(proj.settings));
      }

      currentFileId.set(fileId);

      // Load TMs
      const tms = await listTms();
      availableTms.set(tms);

      // Load segments
      const segs = await getSegments(fileId, projectId);
      segments.set(segs);
      selectedIndex.set(0);
    } catch (e: any) {
      error = e.toString();
    } finally {
      loading = false;
    }
  });

  async function handleExport() {
    if (!$currentProject || !fileId) return;
    exporting = true;
    try {
      const outPath = await exportFile({
        file_id: fileId,
        project_id: $currentProject.id,
      });
      // TODO: show toast "Exported to {outPath}"
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
      onclick={() => goto(`/project/${projectId}`)}
      class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
      title="Back to project"
    >
      ← 
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

    <!-- Export button -->
    <button
      onclick={handleExport}
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
    {:else if error}
      <div class="flex items-center justify-center h-full">
        <div class="text-red-500 text-sm">{error}</div>
      </div>
    {:else}
      <SegmentGrid />
    {/if}
  </div>

</div>
