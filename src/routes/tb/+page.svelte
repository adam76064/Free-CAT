<script lang="ts">
  // src/routes/tb/+page.svelte — Studio-grade Termbase Management Workbench
  import { onMount } from 'svelte';
  import { open, save } from '@tauri-apps/plugin-dialog';
  import {
    listTbs, createTb, deleteTb, importTbx, exportTbx, getTerms, addTerm, deleteTerm,
    type TbInfo, type TermEntry
  } from '$lib/ipc/termbase';
  import { toast } from '$lib/stores/toast';
  import Icon from '$lib/components/ui/Icon.svelte';

  let termbases = $state<TbInfo[]>([]);
  let selectedTbId = $state<string | null>(null);
  let terms = $state<TermEntry[]>([]);
  let loading = $state(true);
  let termsLoading = $state(false);

  // Search filter
  let termFilter = $state('');

  // New Termbase Modal state
  let showCreateTbModal = $state(false);
  let newTbName = $state('');
  let creatingTb = $state(false);

  // New Term Modal state
  let showAddTermModal = $state(false);
  let newTermText = $state('');
  let newTermLang = $state('en-US');
  let newTermDef = $state('');
  let newTermDomain = $state('');
  let newTermStatus = $state('preferred');
  let newTermNote = $state('');
  let addingTerm = $state(false);

  const LANGS = [
    ['en-US', 'English (US)'], ['en-GB', 'English (UK)'], ['ar-SA', 'Arabic'],
    ['fr-FR', 'French'], ['de-DE', 'German'], ['es-ES', 'Spanish'],
    ['it-IT', 'Italian'], ['pt-BR', 'Portuguese (BR)'], ['nl-NL', 'Dutch'],
    ['ru-RU', 'Russian'], ['zh-CN', 'Chinese (Simplified)'], ['ja-JP', 'Japanese'],
  ];

  onMount(async () => {
    await loadTermbases();
  });

  async function loadTermbases() {
    loading = true;
    try {
      termbases = await listTbs();
      if (termbases.length > 0 && !selectedTbId) {
        await selectTb(termbases[0].id);
      }
    } catch (e) {
      console.error('Failed to load termbases:', e);
    } finally {
      loading = false;
    }
  }

  async function selectTb(id: string) {
    selectedTbId = id;
    termsLoading = true;
    try {
      terms = await getTerms(id);
    } catch (e) {
      console.error('Failed to load terms:', e);
      terms = [];
    } finally {
      termsLoading = false;
    }
  }

  async function handleCreateTb() {
    if (!newTbName.trim()) return;
    creatingTb = true;
    try {
      const created = await createTb(newTbName.trim());
      showCreateTbModal = false;
      newTbName = '';
      await loadTermbases();
      if (created?.id) {
        await selectTb(created.id);
      }
      toast.success('Termbase Created', `Created termbase "${newTbName.trim()}"`);
    } catch (e) {
      toast.error('Creation Failed', String(e));
    } finally {
      creatingTb = false;
    }
  }

  async function handleDeleteTb(tb: TbInfo) {
    if (!confirm(`Delete termbase "${tb.name}" and all its terms? This action cannot be undone.`)) return;
    try {
      await deleteTb(tb.id);
      if (selectedTbId === tb.id) {
        selectedTbId = null;
        terms = [];
      }
      await loadTermbases();
      toast.success('Termbase Deleted', `Removed "${tb.name}".`);
    } catch (e) {
      toast.error('Delete Failed', String(e));
    }
  }

  async function handleImportTbx(tbId: string) {
    const path = await open({
      title: 'Import TBX Glossary',
      filters: [{ name: 'TBX File', extensions: ['tbx', 'xml'] }]
    });
    if (!path || typeof path !== 'string') return;
    try {
      const count = await importTbx(tbId, path);
      toast.success('TBX Imported', `Successfully imported ${count} terms.`);
      await loadTermbases();
      if (selectedTbId === tbId) {
        await selectTb(tbId);
      }
    } catch (e) {
      toast.error('Import Failed', String(e));
    }
  }

  async function handleExportTbx(tb: TbInfo) {
    const path = await save({
      defaultPath: `${tb.name}.tbx`,
      filters: [{ name: 'TBX Glossary', extensions: ['tbx'] }]
    });
    if (!path) return;
    try {
      const count = await exportTbx(tb.id, path);
      toast.success('TBX Exported', `Exported ${count} terms to ${path}`);
    } catch (e) {
      toast.error('Export Failed', String(e));
    }
  }

  async function handleAddTerm() {
    if (!selectedTbId || !newTermText.trim()) return;
    addingTerm = true;
    try {
      await addTerm({
        tb_id: selectedTbId,
        term: newTermText.trim(),
        lang: newTermLang,
        definition: newTermDef.trim() || undefined,
        domain: newTermDomain.trim() || undefined,
        status: newTermStatus,
        note: newTermNote.trim() || undefined,
      });
      showAddTermModal = false;
      const addedText = newTermText.trim();
      newTermText = '';
      newTermDef = '';
      newTermDomain = '';
      newTermNote = '';
      await selectTb(selectedTbId);
      await loadTermbases();
      toast.success('Term Added', `Saved "${addedText}".`);
    } catch (e) {
      toast.error('Save Failed', String(e));
    } finally {
      addingTerm = false;
    }
  }

  async function handleDeleteTerm(termId: number | undefined) {
    if (!selectedTbId || termId === undefined) return;
    try {
      await deleteTerm(selectedTbId, termId);
      await selectTb(selectedTbId);
      await loadTermbases();
      toast.success('Term Deleted', 'Term removed from termbase.');
    } catch (e) {
      toast.error('Delete Failed', String(e));
    }
  }

  let selectedTb = $derived(termbases.find(t => t.id === selectedTbId));
  let filteredTerms = $derived(
    terms.filter(t => {
      if (!termFilter.trim()) return true;
      const q = termFilter.toLowerCase();
      return (
        t.term.toLowerCase().includes(q) ||
        (t.definition && t.definition.toLowerCase().includes(q)) ||
        (t.domain && t.domain.toLowerCase().includes(q))
      );
    })
  );
</script>

<div class="h-full flex overflow-hidden bg-gray-50 dark:bg-gray-950">

  <!-- Left Sidebar: Termbases List -->
  <aside class="w-80 border-r border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 flex flex-col shrink-0">
    <div class="p-4 border-b border-gray-200 dark:border-gray-800 flex items-center justify-between">
      <div class="flex items-center gap-2">
        <span class="text-xl">📖</span>
        <h1 class="text-base font-semibold text-gray-900 dark:text-white">Termbases</h1>
        <span class="px-2 py-0.5 rounded-full text-xs font-medium bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-400">
          {termbases.length}
        </span>
      </div>
      <button
        onclick={() => (showCreateTbModal = true)}
        class="px-2.5 py-1 text-xs font-semibold bg-brand-500 hover:bg-brand-600 text-white rounded-lg transition-colors flex items-center gap-1 shadow-xs"
        title="Create a new glossary termbase"
      >
        <span>+ New</span>
      </button>
    </div>

    <div class="flex-1 overflow-y-auto p-3 space-y-2">
      {#if loading}
        <div class="text-center py-8 text-xs text-gray-400">Loading termbases…</div>
      {:else if termbases.length === 0}
        <div class="text-center py-12 px-4 text-gray-400">
          <div class="text-3xl mb-2">📚</div>
          <p class="text-xs text-gray-500 mb-3">No termbases found. Create one or import TBX to enable glossary recognition.</p>
          <button
            onclick={() => (showCreateTbModal = true)}
            class="px-3 py-1.5 text-xs bg-brand-500 text-white rounded-lg hover:bg-brand-600 transition-colors"
          >
            + Create Termbase
          </button>
        </div>
      {:else}
        {#each termbases as tb (tb.id)}
          <!-- svelte-ignore a11y_click_events_have_key_events -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            onclick={() => selectTb(tb.id)}
            class="p-3 rounded-xl border transition-all cursor-pointer group relative
                   {selectedTbId === tb.id
                     ? 'bg-brand-50/80 dark:bg-brand-900/30 border-brand-300 dark:border-brand-700 shadow-xs'
                     : 'bg-white dark:bg-gray-900/50 border-gray-200 dark:border-gray-800 hover:border-gray-300 dark:hover:border-gray-700'}"
          >
            <div class="flex items-center justify-between mb-1.5">
              <span class="font-medium text-sm text-gray-900 dark:text-white truncate max-w-[170px]" title={tb.name}>
                {tb.name}
              </span>
              <span class="text-[11px] px-1.5 py-0.5 rounded-md font-mono bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-400">
                {tb.count} {tb.count === 1 ? 'term' : 'terms'}
              </span>
            </div>

            <div class="flex items-center gap-1.5 pt-1 border-t border-gray-100 dark:border-gray-800/60">
              <button
                onclick={(e) => { e.stopPropagation(); handleImportTbx(tb.id); }}
                class="px-2 py-0.5 text-[10px] rounded border border-gray-200 dark:border-gray-700 text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
                title="Import TBX XML"
              >
                ↑ Import
              </button>
              <button
                onclick={(e) => { e.stopPropagation(); handleExportTbx(tb); }}
                class="px-2 py-0.5 text-[10px] rounded border border-gray-200 dark:border-gray-700 text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
                title="Export TBX XML"
              >
                ↓ Export
              </button>
              <button
                onclick={(e) => { e.stopPropagation(); handleDeleteTb(tb); }}
                class="ml-auto px-1.5 py-0.5 text-[10px] rounded text-red-500 hover:bg-red-50 dark:hover:bg-red-950/40 transition-colors"
                title="Delete termbase"
              >
                🗑
              </button>
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </aside>

  <!-- Right Panel: Active Termbase Workbench -->
  <main class="flex-1 flex flex-col overflow-hidden">
    {#if selectedTb}
      <!-- Active TB Header & Toolbar -->
      <div class="px-6 py-4 bg-white dark:bg-gray-950 border-b border-gray-200 dark:border-gray-800 flex items-center justify-between shrink-0">
        <div>
          <div class="flex items-center gap-3">
            <h2 class="text-lg font-semibold text-gray-900 dark:text-white">{selectedTb.name}</h2>
            <span class="px-2.5 py-0.5 rounded-full text-xs font-semibold bg-brand-100 dark:bg-brand-900/40 text-brand-700 dark:text-brand-300">
              {terms.length} entries
            </span>
          </div>
          <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5">
            Terminology entries are matched automatically in real-time inside the bilingual editor and QA checker.
          </p>
        </div>

        <div class="flex items-center gap-3">
          <input
            bind:value={termFilter}
            placeholder="Search terms, definitions, domains…"
            class="px-3 py-1.5 text-xs rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900 text-gray-900 dark:text-white w-64 focus:outline-none focus:ring-1 focus:ring-brand-500"
          />
          <button
            onclick={() => (showAddTermModal = true)}
            class="px-3 py-1.5 bg-brand-500 hover:bg-brand-600 text-white text-xs font-semibold rounded-lg transition-colors flex items-center gap-1.5 shadow-xs"
          >
            <span>+ Add Term</span>
          </button>
        </div>
      </div>

      <!-- Terms Table -->
      <div class="flex-1 overflow-y-auto p-6">
        {#if termsLoading}
          <div class="text-center py-12 text-sm text-gray-400">Loading terms…</div>
        {:else if filteredTerms.length === 0}
          <div class="flex flex-col items-center justify-center h-64 text-center text-gray-400 gap-2">
            <span class="text-4xl">🏷️</span>
            {#if termFilter.trim()}
              <p class="text-sm">No terms match "{termFilter}".</p>
            {:else}
              <p class="text-sm">This termbase has no terms yet.</p>
              <button
                onclick={() => (showAddTermModal = true)}
                class="mt-2 px-3 py-1.5 bg-brand-500 text-white text-xs font-semibold rounded-lg hover:bg-brand-600 transition-colors"
              >
                + Add First Term
              </button>
            {/if}
          </div>
        {:else}
          <div class="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 overflow-hidden shadow-xs">
            <table class="w-full text-left border-collapse">
              <thead>
                <tr class="bg-gray-50/80 dark:bg-gray-800/40 border-b border-gray-200 dark:border-gray-800 text-[11px] font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                  <th class="py-3 px-4">Term</th>
                  <th class="py-3 px-4">Language</th>
                  <th class="py-3 px-4">Definition / Translation</th>
                  <th class="py-3 px-4">Domain</th>
                  <th class="py-3 px-4">Status</th>
                  <th class="py-3 px-4 text-right">Actions</th>
                </tr>
              </thead>
              <tbody class="divide-y divide-gray-100 dark:divide-gray-800/60 text-xs">
                {#each filteredTerms as t (t.id ?? t.term)}
                  <tr class="hover:bg-gray-50 dark:hover:bg-gray-800/40 transition-colors">
                    <td class="py-3 px-4 font-semibold text-gray-900 dark:text-white font-editor">
                      {t.term}
                    </td>
                    <td class="py-3 px-4 text-gray-500 font-mono text-[11px]">
                      {t.lang}
                    </td>
                    <td class="py-3 px-4 text-gray-700 dark:text-gray-300 font-editor max-w-md">
                      {t.definition || '—'}
                      {#if t.note}
                        <span class="block text-[11px] text-gray-400 italic mt-0.5">{t.note}</span>
                      {/if}
                    </td>
                    <td class="py-3 px-4">
                      {#if t.domain}
                        <span class="px-2 py-0.5 rounded-full text-[10px] font-medium bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-300">
                          {t.domain}
                        </span>
                      {:else}
                        <span class="text-gray-400">—</span>
                      {/if}
                    </td>
                    <td class="py-3 px-4">
                      {#if t.status === 'preferred'}
                        <span class="px-2 py-0.5 rounded-full text-[10px] font-semibold bg-emerald-100 text-emerald-800 dark:bg-emerald-950/40 dark:text-emerald-300">
                          Preferred
                        </span>
                      {:else if t.status === 'admitted'}
                        <span class="px-2 py-0.5 rounded-full text-[10px] font-semibold bg-blue-100 text-blue-800 dark:bg-blue-950/40 dark:text-blue-300">
                          Admitted
                        </span>
                      {:else if t.status === 'deprecated' || t.status === 'forbidden'}
                        <span class="px-2 py-0.5 rounded-full text-[10px] font-semibold bg-red-100 text-red-800 dark:bg-red-950/40 dark:text-red-300">
                          Forbidden
                        </span>
                      {:else}
                        <span class="px-2 py-0.5 rounded-full text-[10px] font-semibold bg-gray-100 text-gray-700 dark:bg-gray-800 dark:text-gray-300">
                          {t.status}
                        </span>
                      {/if}
                    </td>
                    <td class="py-3 px-4 text-right">
                      <button
                        onclick={() => handleDeleteTerm(t.id)}
                        class="p-1 rounded text-gray-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-950/40 transition-colors"
                        title="Delete term"
                      >
                        🗑
                      </button>
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {/if}
      </div>
    {:else}
      <!-- No TB selected -->
      <div class="flex-1 flex flex-col items-center justify-center p-8 text-center text-gray-400">
        <span class="text-5xl mb-4">📖</span>
        <h3 class="text-base font-semibold text-gray-700 dark:text-gray-300 mb-1">Select or Create a Termbase</h3>
        <p class="text-xs text-gray-500 max-w-sm mb-4">
          Termbases allow you to maintain glossaries, enforce client-approved terminology, and prevent forbidden terms during QA checks.
        </p>
        <button
          onclick={() => (showCreateTbModal = true)}
          class="px-4 py-2 bg-brand-500 hover:bg-brand-600 text-white text-xs font-semibold rounded-lg transition-colors shadow-xs"
        >
          + Create Termbase
        </button>
      </div>
    {/if}
  </main>
</div>

<!-- Create Termbase Modal -->
{#if showCreateTbModal}
  <div class="fixed inset-0 bg-black/50 backdrop-blur-xs z-50 flex items-center justify-center p-4">
    <div class="bg-white dark:bg-gray-900 rounded-2xl shadow-2xl w-full max-w-md p-6 border border-gray-200 dark:border-gray-800">
      <h3 class="text-base font-semibold text-gray-900 dark:text-white mb-4">Create New Termbase</h3>
      <div class="space-y-4">
        <div>
          <label for="tb-name" class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Termbase Name</label>
          <input
            id="tb-name"
            bind:value={newTbName}
            placeholder="e.g. Legal & Contract Glossary"
            class="w-full px-3 py-2 text-xs rounded-lg border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-brand-500"
          />
        </div>
        <div class="flex justify-end gap-2 pt-2">
          <button
            onclick={() => (showCreateTbModal = false)}
            class="px-3 py-1.5 text-xs rounded-lg border border-gray-300 dark:border-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
          >
            Cancel
          </button>
          <button
            onclick={handleCreateTb}
            disabled={creatingTb || !newTbName.trim()}
            class="px-4 py-1.5 text-xs font-semibold bg-brand-500 hover:bg-brand-600 text-white rounded-lg transition-colors disabled:opacity-50"
          >
            {creatingTb ? 'Creating…' : 'Create Termbase'}
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

<!-- Add Term Modal -->
{#if showAddTermModal}
  <div class="fixed inset-0 bg-black/50 backdrop-blur-xs z-50 flex items-center justify-center p-4">
    <div class="bg-white dark:bg-gray-900 rounded-2xl shadow-2xl w-full max-w-lg p-6 border border-gray-200 dark:border-gray-800">
      <h3 class="text-base font-semibold text-gray-900 dark:text-white mb-4">Add Term to Glossary</h3>
      <div class="space-y-3">
        <div class="grid grid-cols-3 gap-3">
          <div class="col-span-2">
            <label for="new-term" class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Term (Source)</label>
            <input
              id="new-term"
              bind:value={newTermText}
              placeholder="e.g. intellectual property"
              class="w-full px-3 py-2 text-xs rounded-lg border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-brand-500 font-editor"
            />
          </div>
          <div>
            <label for="new-lang" class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Language</label>
            <select
              id="new-lang"
              bind:value={newTermLang}
              class="w-full px-3 py-2 text-xs rounded-lg border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-brand-500"
            >
              {#each LANGS as [code, label]}
                <option value={code}>{label}</option>
              {/each}
            </select>
          </div>
        </div>

        <div>
          <label for="new-def" class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Definition / Translation Equivalent</label>
          <input
            id="new-def"
            bind:value={newTermDef}
            placeholder="e.g. الملكية الفكرية / Propriété intellectuelle"
            class="w-full px-3 py-2 text-xs rounded-lg border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-brand-500 font-editor"
          />
        </div>

        <div class="grid grid-cols-2 gap-3">
          <div>
            <label for="new-domain" class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Domain / Subject</label>
            <input
              id="new-domain"
              bind:value={newTermDomain}
              placeholder="e.g. Legal, Medical, IT"
              class="w-full px-3 py-2 text-xs rounded-lg border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-brand-500"
            />
          </div>
          <div>
            <label for="new-status" class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Approval Status</label>
            <select
              id="new-status"
              bind:value={newTermStatus}
              class="w-full px-3 py-2 text-xs rounded-lg border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-brand-500"
            >
              <option value="preferred">Preferred (Approved)</option>
              <option value="admitted">Admitted (Acceptable)</option>
              <option value="deprecated">Forbidden / Deprecated</option>
            </select>
          </div>
        </div>

        <div>
          <label for="new-note" class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1">Usage Notes (Optional)</label>
          <input
            id="new-note"
            bind:value={newTermNote}
            placeholder="Context, grammatical notes, or restrictions…"
            class="w-full px-3 py-2 text-xs rounded-lg border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-brand-500"
          />
        </div>

        <div class="flex justify-end gap-2 pt-3">
          <button
            onclick={() => (showAddTermModal = false)}
            class="px-3 py-1.5 text-xs rounded-lg border border-gray-300 dark:border-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
          >
            Cancel
          </button>
          <button
            onclick={handleAddTerm}
            disabled={addingTerm || !newTermText.trim()}
            class="px-4 py-1.5 text-xs font-semibold bg-brand-500 hover:bg-brand-600 text-white rounded-lg transition-colors disabled:opacity-50"
          >
            {addingTerm ? 'Saving…' : 'Save Term'}
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
