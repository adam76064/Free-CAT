<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import { toast } from '$lib/stores/toast';
  import Icon from '$lib/components/ui/Icon.svelte';

  interface Project {
    id: string;
    name: string;
    src_lang: string;
    tgt_lang: string;
    created_at: string;
    modified_at: string;
    file_count?: number;
    word_count?: number;
    completed_pct?: number;
  }

  let projects: Project[] = [];
  let loading = true;
  let showNewModal = false;

  let newName = '';
  let newSrcLang = 'en-US';
  let newTgtLang = '';
  let creating = false;

  onMount(async () => {
    try {
      projects = await invoke<Project[]>('list_projects');
    } catch (e: any) {
      toast.error('Failed to Load Projects', String(e));
    } finally {
      loading = false;
    }
  });

  async function createProject() {
    if (!newName.trim() || !newTgtLang.trim()) return;
    creating = true;
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
      toast.error('Create Failed', String(e));
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
      toast.error('Import Failed', String(e));
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

<div class="h-full flex flex-col bg-gray-50/50 dark:bg-gray-950">
  <!-- Header -->
  <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900">
    <div class="flex items-center gap-3">
      <div class="w-9 h-9 rounded-xl bg-brand-50 dark:bg-brand-950/60 text-brand-600 dark:text-brand-400 border border-brand-200 dark:border-brand-800 flex items-center justify-center shadow-xs">
        <Icon name="home" size={20} />
      </div>
      <div>
        <h1 class="text-xl font-bold text-gray-900 dark:text-white">Translation Projects</h1>
        <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
          Manage translation projects, packages, and bilingual files
        </p>
      </div>
    </div>
    <div class="flex items-center gap-2">
      <button
        onclick={handleImportFcat}
        class="px-3.5 py-2 border border-gray-300 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 text-gray-700 dark:text-gray-300 text-sm font-medium rounded-xl transition-colors flex items-center gap-2 shadow-xs cursor-pointer"
        title="Import a single-file .fcat project package archive"
      >
        <Icon name="package" size={16} cls="text-brand-600 dark:text-brand-400" />
        <span>Import .fcat</span>
      </button>
      <button
        onclick={() => (showNewModal = true)}
        class="px-4 py-2 bg-brand-500 hover:bg-brand-600 text-white text-sm font-semibold rounded-xl transition-all shadow-sm flex items-center gap-1.5 cursor-pointer"
      >
        <Icon name="plus" size={16} />
        <span>New Project</span>
      </button>
    </div>
  </div>

  <!-- Stats bar -->
  {#if projects.length > 0}
    <div class="px-6 py-2.5 border-b border-gray-200/80 dark:border-gray-800/80 bg-white/70 dark:bg-gray-900/60 backdrop-blur-xs flex items-center gap-6 text-xs text-gray-500 shrink-0">
      <span class="flex items-center gap-1.5">
        <strong class="text-gray-900 dark:text-white font-semibold">{projects.length}</strong> project{projects.length !== 1 ? 's' : ''}
      </span>
      <span class="text-gray-300 dark:text-gray-700">·</span>
      <span class="flex items-center gap-1.5">
        <span class="w-2 h-2 rounded-full bg-emerald-500"></span>
        <span>Offline SQLite WAL active</span>
      </span>
      <span class="text-gray-300 dark:text-gray-700">·</span>
      <span class="text-gray-400">Trados & memoQ compatible</span>
    </div>
  {/if}

  <!-- Project list / Empty state -->
  <div class="flex-1 overflow-y-auto p-6">
    {#if loading}
      <div class="flex flex-col items-center justify-center h-64 text-gray-400 gap-3">
        <div class="w-6 h-6 border-2 border-brand-500 border-t-transparent rounded-full animate-spin"></div>
        <span class="text-sm">Loading projects…</span>
      </div>
    {:else if projects.length === 0}
      <!-- Empty State / Onboarding -->
      <div class="max-w-md mx-auto my-12 p-8 bg-white dark:bg-gray-900 rounded-2xl border border-gray-200 dark:border-gray-800 shadow-sm text-center">
        <div class="w-16 h-16 rounded-2xl bg-brand-50 dark:bg-brand-950/60 text-brand-600 dark:text-brand-400 border border-brand-200 dark:border-brand-800 flex items-center justify-center text-3xl mx-auto mb-4 shadow-sm">
          <Icon name="home" size={32} />
        </div>
        <h2 class="text-lg font-bold text-gray-900 dark:text-white mb-2">Welcome to Free-CAT</h2>
        <p class="text-xs text-gray-500 dark:text-gray-400 leading-relaxed mb-6">
          Your open-source, studio-grade translation workbench. Create a new translation project or import a Trados or Free-CAT package to get started.
        </p>
        <div class="flex flex-col sm:flex-row items-center justify-center gap-3">
          <button
            onclick={() => (showNewModal = true)}
            class="w-full sm:w-auto px-5 py-2.5 bg-brand-500 hover:bg-brand-600 text-white text-xs font-semibold rounded-xl transition-all shadow-sm flex items-center justify-center gap-2 cursor-pointer"
          >
            <Icon name="plus" size={14} />
            <span>Create First Project</span>
          </button>
          <button
            onclick={handleImportFcat}
            class="w-full sm:w-auto px-4 py-2.5 border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 text-gray-700 dark:text-gray-300 text-xs font-medium rounded-xl transition-colors flex items-center justify-center gap-1.5 cursor-pointer"
          >
            <Icon name="package" size={14} />
            <span>Import Package</span>
          </button>
        </div>
      </div>
    {:else}
      <!-- Rich Cards Grid -->
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-5">
        {#each projects as project (project.id)}
          <button
            onclick={() => openProject(project.id)}
            class="text-left p-5 rounded-2xl border border-gray-200 dark:border-gray-800
                   bg-white dark:bg-gray-900 hover:border-brand-400 dark:hover:border-brand-600
                   hover:shadow-lg transition-all duration-200 group relative overflow-hidden flex flex-col justify-between"
          >
            <!-- Hover accent bar -->
            <div class="absolute top-0 left-0 right-0 h-1 bg-gradient-to-r from-brand-400 to-brand-600
                        opacity-0 group-hover:opacity-100 transition-opacity"></div>

            <div>
              <!-- Language pair pill -->
              <div class="flex items-center justify-between mb-3.5">
                <div class="flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-brand-50 dark:bg-brand-950/40
                            border border-brand-200 dark:border-brand-800">
                  <span class="text-xs font-mono font-bold text-brand-700 dark:text-brand-300">{project.src_lang}</span>
                  <span class="text-brand-400 text-xs">→</span>
                  <span class="text-xs font-mono font-bold text-brand-700 dark:text-brand-300">{project.tgt_lang}</span>
                </div>
                <span class="text-[11px] text-gray-400 dark:text-gray-500 font-mono">
                  {formatDate(project.modified_at)}
                </span>
              </div>

              <!-- Project name -->
              <h3 class="font-bold text-gray-900 dark:text-white group-hover:text-brand-600 dark:group-hover:text-brand-400
                         transition-colors truncate text-base mb-2">
                {project.name}
              </h3>
            </div>

            <!-- Footer stats / progress -->
            <div class="mt-4 pt-3 border-t border-gray-100 dark:border-gray-800/80 flex items-center justify-between text-xs text-gray-500 dark:text-gray-400">
              <span class="flex items-center gap-1">
                <span>Open Project</span>
              </span>
              <Icon name="chevron-right" size={14} cls="text-gray-400 group-hover:translate-x-1 transition-transform" />
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
          onclick={() => (showNewModal = false)}
          class="flex-1 px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300
                 border border-gray-300 dark:border-gray-600 rounded-lg
                 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors"
        >
          Cancel
        </button>
        <button
          onclick={createProject}
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
