<script lang="ts">
  import { termMatches, tbLoading } from '$lib/stores/termbase';
  import { currentProject, projectSettings } from '$lib/stores/project';
  import { addTerm, type TermEntry } from '$lib/ipc/termbase';

  export let onInsertTerm: (term: string) => void = () => {};

  let showAdd = false;
  let newTerm = '';
  let newDef = '';
  let newLang = '';
  let adding = false;
  let addError = '';

  $: if ($currentProject && !newLang) {
    newLang = $currentProject.tgt_lang;
  }

  async function handleAddTerm() {
    if (!newTerm.trim() || !$projectSettings.tb_ids.length) return;
    adding = true;
    addError = '';
    try {
      await addTerm({
        tb_id: $projectSettings.tb_ids[0],
        lang: newLang,
        term: newTerm.trim(),
        definition: newDef.trim() || undefined,
        status: 'preferred',
      });
      newTerm = '';
      newDef = '';
      showAdd = false;
    } catch (e: any) {
      addError = String(e);
    } finally {
      adding = false;
    }
  }

  const STATUS_CLASSES: Record<string, string> = {
    preferred: 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-300',
    admitted:  'bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-300',
    forbidden: 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-300',
  };
</script>

<div class="h-full flex flex-col bg-white dark:bg-gray-950 text-xs">
  <!-- Panel header -->
  <div class="flex items-center justify-between px-3 py-2 border-b border-gray-200 dark:border-gray-800 shrink-0">
    <div class="flex items-center gap-1.5 font-semibold text-gray-700 dark:text-gray-300">
      <span>📖</span>
      <span>Termbase</span>
      {#if $termMatches.length > 0}
        <span class="px-1.5 py-0.2 rounded-full bg-brand-100 dark:bg-brand-900/40 text-brand-600 dark:text-brand-400 font-mono text-[10px]">
          {$termMatches.length}
        </span>
      {/if}
    </div>

    {#if $projectSettings.tb_ids.length > 0}
      <button
        on:click={() => (showAdd = !showAdd)}
        class="text-brand-500 hover:text-brand-600 font-medium text-xs transition-colors"
      >
        {showAdd ? 'Cancel' : '+ Add Term'}
      </button>
    {/if}
  </div>

  <!-- Add term inline form -->
  {#if showAdd}
    <div class="p-3 bg-brand-50/50 dark:bg-brand-900/10 border-b border-brand-100 dark:border-brand-900/30 space-y-2">
      <div>
        <label for="tb-new-term" class="block text-[11px] font-medium text-gray-700 dark:text-gray-300 mb-0.5">Term</label>
        <input
          id="tb-new-term"
          bind:value={newTerm}
          placeholder="e.g. cloud computing"
          class="w-full px-2 py-1 text-xs rounded border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800"
        />
      </div>
      <div>
        <label for="tb-new-def" class="block text-[11px] font-medium text-gray-700 dark:text-gray-300 mb-0.5">Translation / Definition</label>
        <input
          id="tb-new-def"
          bind:value={newDef}
          placeholder="e.g. computación en la nube"
          class="w-full px-2 py-1 text-xs rounded border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800"
        />
      </div>
      {#if addError}
        <p class="text-red-500 text-[10px]">{addError}</p>
      {/if}
      <button
        on:click={handleAddTerm}
        disabled={!newTerm.trim() || adding}
        class="w-full py-1 bg-brand-500 hover:bg-brand-600 disabled:opacity-50 text-white rounded text-xs font-medium"
      >
        {adding ? 'Saving…' : 'Save Term'}
      </button>
    </div>
  {/if}

  <!-- Term list -->
  <div class="flex-1 overflow-y-auto p-2 space-y-1.5">
    {#if $tbLoading}
      <div class="text-center text-gray-400 py-6">Searching glossary…</div>
    {:else if $termMatches.length === 0}
      <div class="text-center text-gray-400 py-6">
        No glossary terms recognized in segment.
      </div>
    {:else}
      {#each $termMatches as term}
        <div class="p-2 rounded-lg border border-gray-100 dark:border-gray-800 bg-gray-50 dark:bg-gray-900/50 hover:border-brand-200 dark:hover:border-brand-800 transition-colors">
          <div class="flex items-center justify-between gap-1 mb-1">
            <span class="font-semibold text-gray-900 dark:text-white truncate">
              {term.term}
            </span>
            <span class="px-1.5 py-0.5 rounded text-[10px] font-medium shrink-0 {STATUS_CLASSES[term.status] ?? 'bg-gray-100 text-gray-600'}">
              {term.status}
            </span>
          </div>

          {#if term.definition}
            <div class="flex items-center justify-between gap-2 mt-1">
              <span class="text-gray-600 dark:text-gray-300 italic truncate">
                {term.definition}
              </span>
              <button
                on:click={() => onInsertTerm(term.definition || term.term)}
                class="px-1.5 py-0.5 rounded bg-brand-50 hover:bg-brand-100 dark:bg-brand-900/30 dark:hover:bg-brand-900/50 text-brand-600 dark:text-brand-300 font-medium text-[10px] shrink-0"
                title="Insert into target editor"
              >
                Insert ↵
              </button>
            </div>
          {/if}

          {#if term.note}
            <p class="text-[10px] text-gray-400 mt-1">{term.note}</p>
          {/if}
        </div>
      {/each}
    {/if}
  </div>
</div>
