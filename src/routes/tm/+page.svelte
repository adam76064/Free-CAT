<script lang="ts">
  // TM Manager page — list, create, import/export TMs.
  import { onMount } from 'svelte';
  import { open, save } from '@tauri-apps/plugin-dialog';
  import { availableTms } from '$lib/stores/tm';
  import { listTms, createTm, deleteTm, importTmx, exportTmx } from '$lib/ipc/tm';

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
      newName = '';
    } finally {
      creating = false;
    }
  }

  async function handleDelete(id: string, name: string) {
    if (!confirm(`Delete TM "${name}"? This cannot be undone.`)) return;
    await deleteTm(id);
    availableTms.set(await listTms());
  }

  async function handleImportTmx(id: string) {
    const path = await open({ filters: [{ name: 'TMX', extensions: ['tmx'] }] });
    if (!path || typeof path !== 'string') return;
    const count = await importTmx(id, path);
    availableTms.set(await listTms());
    alert(`Imported ${count} translation units.`);
  }

  async function handleExportTmx(id: string, name: string) {
    const path = await save({ defaultPath: `${name}.tmx`, filters: [{ name: 'TMX', extensions: ['tmx'] }] });
    if (!path) return;
    const count = await exportTmx(id, path);
    alert(`Exported ${count} translation units to ${path}`);
  }
</script>

<div class="h-full flex flex-col">
  <div class="flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-950">
    <div>
      <h1 class="text-xl font-semibold text-gray-900 dark:text-white">Translation Memories</h1>
      <p class="text-sm text-gray-500 dark:text-gray-400 mt-0.5">{$availableTms.length} TM{$availableTms.length !== 1 ? 's' : ''}</p>
    </div>
    <button
      onclick={() => showCreate = true}
      class="px-4 py-2 bg-brand-500 hover:bg-brand-600 text-white text-sm font-medium rounded-lg transition-colors"
    >
      + Create TM
    </button>
  </div>

  <div class="flex-1 overflow-y-auto p-6">
    {#if loading}
      <div class="text-gray-400 text-sm text-center mt-8">Loading…</div>
    {:else if $availableTms.length === 0}
      <div class="flex flex-col items-center justify-center h-48 gap-3 text-center">
        <span class="text-4xl">🧠</span>
        <p class="text-sm text-gray-500">No TMs yet. Create one to start building your memory.</p>
      </div>
    {:else}
      <div class="space-y-3">
        {#each $availableTms as tm (tm.id)}
          <div class="flex items-center gap-4 p-4 bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800">
            <div class="flex-1 min-w-0">
              <div class="font-medium text-gray-900 dark:text-white">{tm.name}</div>
              <div class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
                <span class="font-mono">{tm.src_lang} → {tm.tgt_lang}</span>
                · {tm.tu_count.toLocaleString()} TUs
              </div>
            </div>
            <div class="flex items-center gap-2">
              <button
                onclick={() => handleImportTmx(tm.id)}
                class="px-2.5 py-1.5 text-xs border border-gray-200 dark:border-gray-700
                       text-gray-600 dark:text-gray-400 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800"
              >
                ↑ Import TMX
              </button>
              <button
                onclick={() => handleExportTmx(tm.id, tm.name)}
                class="px-2.5 py-1.5 text-xs border border-gray-200 dark:border-gray-700
                       text-gray-600 dark:text-gray-400 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800"
              >
                ↓ Export TMX
              </button>
              <button
                onclick={() => handleDelete(tm.id, tm.name)}
                class="px-2.5 py-1.5 text-xs text-red-500 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg"
              >
                Delete
              </button>
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
          <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Name</label>
          <input bind:value={newName} placeholder="e.g. Legal EN→FR" class="w-full px-3 py-2 rounded-lg border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-sm focus:outline-none focus:ring-2 focus:ring-brand-500" />
        </div>
        <div class="grid grid-cols-2 gap-3">
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Source</label>
            <select bind:value={newSrcLang} class="w-full px-3 py-2 rounded-lg border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-sm focus:outline-none focus:ring-2 focus:ring-brand-500">
              {#each LANGS as [code, name]}<option value={code}>{name}</option>{/each}
            </select>
          </div>
          <div>
            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">Target</label>
            <select bind:value={newTgtLang} class="w-full px-3 py-2 rounded-lg border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-sm focus:outline-none focus:ring-2 focus:ring-brand-500">
              <option value="">Select…</option>
              {#each LANGS as [code, name]}<option value={code}>{name}</option>{/each}
            </select>
          </div>
        </div>
      </div>
      <div class="flex gap-3 mt-6">
        <button onclick={() => showCreate = false} class="flex-1 px-4 py-2 text-sm font-medium border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800">Cancel</button>
        <button onclick={handleCreate} disabled={!newName.trim() || !newTgtLang || creating} class="flex-1 px-4 py-2 text-sm font-medium text-white bg-brand-500 hover:bg-brand-600 disabled:opacity-50 rounded-lg">
          {creating ? 'Creating…' : 'Create'}
        </button>
      </div>
    </div>
  </div>
{/if}
