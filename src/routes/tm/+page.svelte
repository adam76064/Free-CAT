<script lang="ts">
  // TM Manager page — list, create, import/export TMs.
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { open, save } from '@tauri-apps/plugin-dialog';
  import { availableTms } from '$lib/stores/tm';
  import { listTms, createTm, deleteTm, importTmx, exportTmx } from '$lib/ipc/tm';
  import { toast } from '$lib/stores/toast';
  import Icon from '$lib/components/ui/Icon.svelte';

  let loading = $state(true);
  let showCreate = $state(false);
  let creating = $state(false);
  let newName = $state('');
  let newSrcLang = $state('en-US');
  let newTgtLang = $state('');

  const LANGS = [
    ['en-US','English (US)'],['en-GB','English (UK)'],['fr-FR','French'],
    ['de-DE','German'],['es-ES','Spanish'],['it-IT','Italian'],
    ['pt-BR','Portuguese (BR)'],['nl-NL','Dutch'],['ru-RU','Russian'],
    ['ar-SA','Arabic'],['zh-CN','Chinese (Simplified)'],['ja-JP','Japanese'],
  ];

  onMount(async () => {
    try {
      availableTms.set(await listTms());
    } finally {
      loading = false;
    }
  });

  async function handleCreate() {
    if (!newName.trim() || !newTgtLang) return;
    creating = true;
    try {
      await createTm({ name: newName, src_lang: newSrcLang, tgt_lang: newTgtLang });
      availableTms.set(await listTms());
      showCreate = false;
      const createdName = newName;
      newName = '';
      toast.success('TM Created', `Created "${createdName}".`);
    } catch (e) {
      toast.error('Creation Failed', String(e));
    } finally {
      creating = false;
    }
  }

  async function handleDelete(id: string, name: string) {
    if (!confirm(`Delete TM "${name}"? This cannot be undone.`)) return;
    try {
      await deleteTm(id);
      availableTms.set(await listTms());
      toast.success('TM Deleted', `Removed "${name}".`);
    } catch (e) {
      toast.error('Delete Failed', String(e));
    }
  }

  async function handleImportTmx(id: string) {
    const path = await open({ filters: [{ name: 'TMX', extensions: ['tmx'] }] });
    if (!path || typeof path !== 'string') return;
    try {
      const count = await importTmx(id, path);
      availableTms.set(await listTms());
      toast.success('TMX Imported', `Imported ${count.toLocaleString()} translation units.`);
    } catch (e) {
      toast.error('Import Failed', String(e));
    }
  }

  async function handleExportTmx(id: string, name: string) {
    const path = await save({ defaultPath: `${name}.tmx`, filters: [{ name: 'TMX', extensions: ['tmx'] }] });
    if (!path) return;
    try {
      const count = await exportTmx(id, path);
      toast.success('TMX Exported', `Exported ${count.toLocaleString()} units to ${path}`);
    } catch (e) {
      toast.error('Export Failed', String(e));
    }
  }
</script>

<div class="h-full flex flex-col bg-gray-50/50 dark:bg-gray-950">
  <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900">
    <div class="flex items-center gap-3">
      <div class="w-9 h-9 rounded-xl bg-brand-50 dark:bg-brand-950/60 text-brand-600 dark:text-brand-400 border border-brand-200 dark:border-brand-800 flex items-center justify-center shadow-xs">
        <Icon name="brain" size={20} />
      </div>
      <div>
        <h1 class="text-xl font-bold text-gray-900 dark:text-white">Translation Memories</h1>
        <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
          {$availableTms.length} database{$availableTms.length !== 1 ? 's' : ''} with sub-millisecond SQLite WAL fuzzy search
        </p>
      </div>
    </div>
    <button
      onclick={() => showCreate = true}
      class="px-4 py-2 bg-brand-500 hover:bg-brand-600 text-white text-sm font-semibold rounded-xl transition-all shadow-sm flex items-center gap-1.5 cursor-pointer"
    >
      <Icon name="plus" size={16} />
      <span>New TM</span>
    </button>
  </div>

  <div class="flex-1 overflow-y-auto p-6">
    {#if loading}
      <div class="flex flex-col items-center justify-center h-48 text-gray-400 gap-3">
        <div class="w-6 h-6 border-2 border-brand-500 border-t-transparent rounded-full animate-spin"></div>
        <span class="text-xs">Loading Translation Memories…</span>
      </div>
    {:else if $availableTms.length === 0}
      <div class="max-w-md mx-auto my-12 p-8 bg-white dark:bg-gray-900 rounded-2xl border border-gray-200 dark:border-gray-800 shadow-sm text-center">
        <div class="w-16 h-16 rounded-2xl bg-brand-50 dark:bg-brand-950/60 text-brand-600 dark:text-brand-400 border border-brand-200 dark:border-brand-800 flex items-center justify-center text-3xl mx-auto mb-4 shadow-sm">
          <Icon name="brain" size={32} />
        </div>
        <h2 class="text-lg font-bold text-gray-900 dark:text-white mb-2">No Translation Memories yet</h2>
        <p class="text-xs text-gray-500 dark:text-gray-400 leading-relaxed mb-6">
          Create your first TM or import existing TMX files from Trados, memoQ, or OmegaT to unlock instant leverage and fuzzy matching.
        </p>
        <button
          onclick={() => (showCreate = true)}
          class="px-5 py-2.5 bg-brand-500 hover:bg-brand-600 text-white text-xs font-semibold rounded-xl transition-all shadow-sm inline-flex items-center gap-2 cursor-pointer"
        >
          <Icon name="plus" size={14} />
          <span>Create TM</span>
        </button>
      </div>
    {:else}
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {#each $availableTms as tm (tm.id)}
          <div class="p-4 bg-white dark:bg-gray-900 rounded-2xl border border-gray-200 dark:border-gray-800 shadow-2xs hover:border-gray-300 dark:hover:border-gray-700 transition-all flex flex-col justify-between">
            <div>
              <div class="flex items-center justify-between mb-2">
                <span class="font-bold text-sm text-gray-900 dark:text-white truncate" title={tm.name}>
                  {tm.name}
                </span>
                <span class="px-2 py-0.5 rounded-md font-mono text-[11px] font-bold bg-brand-50 text-brand-700 dark:bg-brand-950 dark:text-brand-300 border border-brand-200 dark:border-brand-800">
                  {tm.src_lang} → {tm.tgt_lang}
                </span>
              </div>
              <div class="text-xs text-gray-500 dark:text-gray-400 font-mono mb-4">
                {tm.tu_count.toLocaleString()} translation unit{tm.tu_count !== 1 ? 's' : ''}
              </div>
            </div>

            <div class="flex items-center justify-between pt-3 border-t border-gray-100 dark:border-gray-800/80">
              <button
                onclick={() => goto(`/tm/${tm.id}`)}
                class="px-3 py-1.5 text-xs bg-brand-50 hover:bg-brand-100 text-brand-700 dark:bg-brand-950 dark:hover:bg-brand-900 dark:text-brand-300 border border-brand-200 dark:border-brand-800 rounded-lg font-semibold transition-colors flex items-center gap-1.5 cursor-pointer shadow-2xs"
                title="Open TM Maintenance Studio"
              >
                <span>🛠 Maintenance</span>
              </button>
              <div class="flex items-center gap-1.5">
                <button
                  onclick={() => handleImportTmx(tm.id)}
                  class="px-2.5 py-1 text-xs border border-gray-200 dark:border-gray-700 text-gray-600 dark:text-gray-400 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors cursor-pointer"
                  title="Import TMX File"
                >
                  Import
                </button>
                <button
                  onclick={() => handleExportTmx(tm.id, tm.name)}
                  class="px-2.5 py-1 text-xs border border-gray-200 dark:border-gray-700 text-gray-600 dark:text-gray-400 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors cursor-pointer"
                  title="Export TMX File"
                >
                  Export
                </button>
                <button
                  onclick={() => handleDelete(tm.id, tm.name)}
                  class="p-1 text-gray-400 hover:text-red-500 rounded-lg transition-colors cursor-pointer"
                  title="Delete TM"
                >
                  <Icon name="trash" size={14} />
                </button>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<!-- Create TM Modal -->
{#if showCreate}
  <div class="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4">
    <div class="bg-white dark:bg-gray-900 rounded-2xl shadow-2xl w-full max-w-md p-6">
      <h2 class="text-lg font-semibold text-gray-900 dark:text-white mb-5">Create Translation Memory</h2>
      <div class="space-y-4">
        <div>
          <label for="tm-name" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Name</label>
          <input id="tm-name" bind:value={newName} placeholder="e.g. Legal EN→FR" class="w-full px-3 py-2 rounded-lg border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-sm focus:outline-none focus:ring-2 focus:ring-brand-500" />
        </div>
        <div class="grid grid-cols-2 gap-3">
          <div>
            <label for="tm-src" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Source</label>
            <select id="tm-src" bind:value={newSrcLang} class="w-full px-3 py-2 rounded-lg border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-sm focus:outline-none focus:ring-2 focus:ring-brand-500">
              {#each LANGS as [code, name]}<option value={code}>{name}</option>{/each}
            </select>
          </div>
          <div>
            <label for="tm-tgt" class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Target</label>
            <select id="tm-tgt" bind:value={newTgtLang} class="w-full px-3 py-2 rounded-lg border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-sm focus:outline-none focus:ring-2 focus:ring-brand-500">
              <option value="">Select…</option>
              {#each LANGS as [code, name]}<option value={code}>{name}</option>{/each}
            </select>
          </div>
        </div>
      </div>
      <div class="flex gap-3 mt-6">
        <button onclick={() => (showCreate = false)} class="flex-1 px-4 py-2 text-sm font-medium border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800">Cancel</button>
        <button onclick={handleCreate} disabled={!newName.trim() || !newTgtLang || creating} class="flex-1 px-4 py-2 text-sm font-medium text-white bg-brand-500 hover:bg-brand-600 disabled:opacity-50 rounded-lg">
          {creating ? 'Creating…' : 'Create'}
        </button>
      </div>
    </div>
  </div>
{/if}
