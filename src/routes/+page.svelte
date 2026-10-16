<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';

  interface Project {
    id: string;
    name: string;
    src_lang: string;
    tgt_lang: string;
    created_at: string;
    modified_at: string;
  }

  let projects: Project[] = [];
  let loading = true;
  let showNewModal = false;

  let newName = '';
  let newSrcLang = 'en-US';
  let newTgtLang = '';
  let creating = false;
  let error = '';

  onMount(async () => {
    try {
      projects = await invoke<Project[]>('list_projects');
    } catch (e: any) {
      error = String(e);
    } finally {
      loading = false;
    }
  });

  async function createProject() {
    if (!newName.trim() || !newTgtLang.trim()) return;
    creating = true;
    error = '';
    try {
      const project = await invoke<Project>('create_project', {
        args: { name: newName, src_lang: newSrcLang, tgt_lang: newTgtLang }
      });
      projects = [project, ...projects];
      showNewModal = false;
      newName = '';
      newTgtLang = '';
      goto(`/project/${project.id}`);
    } catch (e: any) {
      error = String(e);
    } finally {
      creating = false;
    }
  }

  function openProject(id: string) {
    goto(`/project/${id}`);
  }

  function formatDate(iso: string) {
    return new Date(iso).toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: 'numeric' });
  }

  async function handleImportFcat() {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: 'Free-CAT Package (*.fcat)', extensions: ['fcat'] }],
      });
      if (!selected) return;
      const fcatPath = typeof selected === 'string' ? selected : (selected as string[])[0];
      const importedProj = await invoke<Project>('import_fcat', { fcatPath });
      projects = [importedProj, ...projects];
      goto(`/project/${importedProj.id}`);
    } catch (e: any) {
      error = `Failed to import package: ${e}`;
    }
  }

  const COMMON_LANGS: [string, string][] = [
    ['en-US', 'English (US)'], ['en-GB', 'English (UK)'],
    ['fr-FR', 'French'], ['de-DE', 'German'], ['es-ES', 'Spanish'],
    ['it-IT', 'Italian'], ['pt-BR', 'Portuguese (BR)'], ['nl-NL', 'Dutch'],
    ['ru-RU', 'Russian'], ['ar-SA', 'Arabic'], ['zh-CN', 'Chinese (Simplified)'],
    ['zh-TW', 'Chinese (Traditional)'], ['ja-JP', 'Japanese'], ['ko-KR', 'Korean'],
    ['pl-PL', 'Polish'], ['sv-SE', 'Swedish'], ['tr-TR', 'Turkish'],
  ];
</script>

<div class="h-full flex flex-col">
  <!-- Header -->
  <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-950">
    <div>
      <h1 class="text-xl font-semibold text-gray-900 dark:text-white">Projects</h1>
      <p class="text-sm text-gray-500 dark:text-gray-400 mt-0.5">
        {projects.length} project{projects.length !== 1 ? 's' : ''}
      </p>
    </div>
    <div class="flex items-center gap-2">
      <button
        on:click={handleImportFcat}
        class="px-3.5 py-2 border border-gray-300 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 text-gray-700 dark:text-gray-300 text-sm font-medium rounded-lg transition-colors flex items-center gap-1.5"
        title="Import a single-file .fcat project package archive"
      >
        <span>📦 Import .fcat Package</span>
      </button>
      <button
        on:click={() => (showNewModal = true)}
        class="px-4 py-2 bg-brand-500 hover:bg-brand-600 text-white text-sm font-medium rounded-lg transition-colors"
      >
        + New Project
      </button>
    </div>
  </div>

  {#if error}
    <div class="mx-6 mt-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg text-sm text-red-600 dark:text-red-400">
      {error}
    </div>
  {/if}

  <!-- Project list -->
  <div class="flex-1 overflow-y-auto p-6">
    {#if loading}
      <div class="flex items-center justify-center h-48 text-gray-400">Loading…</div>
    {:else if projects.length === 0}
      <div class="flex flex-col items-center justify-center h-64 text-center">
        <div class="text-5xl mb-4">📄</div>
        <h2 class="text-lg font-medium text-gray-700 dark:text-gray-300">No projects yet</h2>
        <p class="text-sm text-gray-500 dark:text-gray-400 mt-1 mb-4">
          Create your first translation project to get started.
        </p>
        <button
          on:click={() => (showNewModal = true)}
          class="px-4 py-2 bg-brand-500 hover:bg-brand-600 text-white text-sm font-medium rounded-lg transition-colors"
        >
          Create Project
        </button>
      </div>
    {:else}
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
        {#each projects as project (project.id)}
          <button
            on:click={() => openProject(project.id)}
            class="text-left p-4 rounded-xl border border-gray-200 dark:border-gray-800
                   bg-white dark:bg-gray-900
                   hover:border-brand-300 dark:hover:border-brand-700
                   hover:shadow-md transition-all group"
          >
            <div class="flex items-start justify-between mb-3">
              <div class="w-10 h-10 rounded-lg bg-brand-100 dark:bg-brand-900/30 flex items-center justify-center text-lg">
                📝
              </div>
              <span class="text-xs text-gray-400 dark:text-gray-500">
                {formatDate(project.modified_at)}
              </span>
            </div>
            <h3 class="font-semibold text-gray-900 dark:text-white group-hover:text-brand-600 dark:group-hover:text-brand-400 transition-colors truncate">
              {project.name}
            </h3>
            <div class="flex items-center gap-2 mt-2">
              <span class="px-2 py-0.5 text-xs rounded-md bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-400 font-mono">
                {project.src_lang}
              </span>
              <span class="text-gray-400">→</span>
              <span class="px-2 py-0.5 text-xs rounded-md bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-400 font-mono">
                {project.tgt_lang}
              </span>
            </div>
          </button>
        {/each}
      </div>
    {/if}
  </div>
</div>

<!-- New Project Modal -->
{#if showNewModal}
  <div class="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4">
    <div class="bg-white dark:bg-gray-900 rounded-2xl shadow-2xl w-full max-w-md p-6">
      <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-5">New Project</h2>

      <div class="space-y-4">
        <div>
          <label for="proj-name" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
            Project Name
          </label>
          <input
            id="proj-name"
            bind:value={newName}
            placeholder="e.g. Annual Report 2025"
            class="w-full px-3 py-2 rounded-lg border border-gray-300 dark:border-gray-700
                   bg-white dark:bg-gray-800 text-gray-900 dark:text-white
                   focus:outline-none focus:ring-2 focus:ring-brand-500 text-sm"
          />
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div>
            <label for="src-lang" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Source Language
            </label>
            <select
              id="src-lang"
              bind:value={newSrcLang}
              class="w-full px-3 py-2 rounded-lg border border-gray-300 dark:border-gray-700
                     bg-white dark:bg-gray-800 text-gray-900 dark:text-white
                     focus:outline-none focus:ring-2 focus:ring-brand-500 text-sm"
            >
              {#each COMMON_LANGS as [code, name]}
                <option value={code}>{name}</option>
              {/each}
            </select>
          </div>

          <div>
            <label for="tgt-lang" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Target Language
            </label>
            <select
              id="tgt-lang"
              bind:value={newTgtLang}
              class="w-full px-3 py-2 rounded-lg border border-gray-300 dark:border-gray-700
                     bg-white dark:bg-gray-800 text-gray-900 dark:text-white
                     focus:outline-none focus:ring-2 focus:ring-brand-500 text-sm"
            >
              <option value="">Select language…</option>
              {#each COMMON_LANGS as [code, name]}
                <option value={code}>{name}</option>
              {/each}
            </select>
          </div>
        </div>
      </div>

      <div class="flex gap-3 mt-6">
        <button
          on:click={() => (showNewModal = false)}
          class="flex-1 px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300
                 border border-gray-300 dark:border-gray-600 rounded-lg
                 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors"
        >
          Cancel
        </button>
        <button
          on:click={createProject}
          disabled={!newName.trim() || !newTgtLang || creating}
          class="flex-1 px-4 py-2 text-sm font-medium text-white bg-brand-500
                 hover:bg-brand-600 disabled:opacity-50 disabled:cursor-not-allowed
                 rounded-lg transition-colors"
        >
          {creating ? 'Creating…' : 'Create Project'}
        </button>
      </div>
    </div>
  </div>
{/if}
