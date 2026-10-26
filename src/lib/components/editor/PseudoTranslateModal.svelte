<script lang="ts">
  import { dialog } from '$lib/actions/dialog';
  /**
   * PseudoTranslateModal — Pre-Flight DTP & Layout Simulation Studio
   * Inspired by SDL Trados Studio Pseudo-translation (Chapter 45).
   * Tests target text expansion, UI clipping, and font encoding without cloud costs.
   */
  import { createEventDispatcher } from 'svelte';
  import { currentFile, currentProject } from '$lib/stores/project';
  import { pseudotranslateFile, type PseudoTranslateResult } from '$lib/ipc/project';

  export let isOpen: boolean = false;

  const dispatch = createEventDispatcher<{
    close: void;
    applied: { count: number };
  }>();

  let expansionPct = 30;
  let addBrackets = true;
  let useDiacritics = true;
  let overwriteConfirmed = false;

  let isRunning = false;
  let result: PseudoTranslateResult | null = null;
  let error = '';

  async function handleRun() {
    if (!$currentFile) return;
    isRunning = true;
    error = '';
    result = null;

    try {
      const res = await pseudotranslateFile({
        file_id: $currentFile.id,
        expansion_pct: expansionPct,
        add_brackets: addBrackets,
        use_diacritics: useDiacritics,
        overwrite_confirmed: overwriteConfirmed,
      });
      result = res;
      dispatch('applied', { count: res.segments_transformed });
    } catch (e: any) {
      error = String(e);
    } finally {
      isRunning = false;
    }
  }

  function handleClose() {
    dispatch('close');
  }

  // Live preview string
  $: sampleText = 'Welcome to Free-CAT. System configuration is loaded successfully.';
  $: samplePreview = generateSamplePreview(sampleText, expansionPct, addBrackets, useDiacritics);

  function generateSamplePreview(text: string, expansion: number, brackets: boolean, diacritics: boolean): string {
    let converted = diacritics
      ? text
          .replace(/[aA]/g, 'ä')
          .replace(/[eE]/g, 'é')
          .replace(/[iI]/g, 'ï')
          .replace(/[oO]/g, 'ö')
          .replace(/[uU]/g, 'ü')
          .replace(/[cC]/g, 'ç')
      : text;
    if (expansion > 0) {
      const extraChars = Math.round(text.length * (expansion / 100));
      const pad = ' ~!@#$%^&*()_+=~'.repeat(Math.ceil(extraChars / 16)).slice(0, extraChars);
      converted += pad;
    }
    return brackets ? `[~${converted}~]` : converted;
  }
</script>

{#if isOpen}
  <!-- Backdrop -->
  <div
    class="fixed inset-0 bg-black/60 backdrop-blur-xs z-50 flex items-center justify-center p-4 animate-in fade-in duration-150"
    role="presentation"
    onclick={(e) => { if (e.target === e.currentTarget) handleClose(); }}
  >
    <!-- Modal Card -->
    <div
      use:dialog={{ label: "Pseudo-translation" }}
      class="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-xl shadow-2xl max-w-lg w-full overflow-hidden flex flex-col max-h-[90vh]"
    >
      <!-- Header -->
      <div class="flex items-center justify-between px-5 py-3.5 border-b border-gray-200 dark:border-gray-800 bg-gray-50 dark:bg-gray-800/50">
        <div class="flex items-center gap-2">
          <span class="text-lg">🧪</span>
          <div>
            <h3 class="text-sm font-bold text-gray-900 dark:text-white">
              Pseudo-Translation Simulation Studio
            </h3>
            <p class="text-xs text-gray-500 dark:text-gray-400">
              Trados Chapter 45 — Pre-Flight DTP & UI Layout Stress Testing
            </p>
          </div>
        </div>
        <button
          type="button"
          onclick={handleClose}
          class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 text-sm p-1 rounded-md"
        >
          ✕
        </button>
      </div>

      <!-- Body -->
      <div class="p-5 space-y-4 overflow-y-auto text-xs text-gray-700 dark:text-gray-300">
        <!-- Target File -->
        <div class="p-2.5 rounded-lg bg-gray-100 dark:bg-gray-800/60 border border-gray-200 dark:border-gray-700/60 flex items-center justify-between">
          <span class="font-medium text-gray-600 dark:text-gray-400">Target File:</span>
          <span class="font-semibold text-gray-900 dark:text-white font-mono">
            {$currentFile?.filename || 'Active File'}
          </span>
        </div>

        <!-- Expansion Factor Slider -->
        <div class="space-y-1.5">
          <div class="flex items-center justify-between">
            <label for="expansion-range" class="font-semibold text-gray-800 dark:text-gray-200">
              Target Expansion Rate: <span class="text-brand-600 font-bold font-mono">+{expansionPct}%</span>
            </label>
            <span class="text-[10px] text-gray-400">
              {expansionPct <= 20 ? 'French/Spanish (+20%)' : expansionPct <= 35 ? 'Standard German (+30%)' : 'Finnish/Russian (+50%)'}
            </span>
          </div>
          <input
            id="expansion-range"
            type="range"
            min="0"
            max="100"
            step="5"
            bind:value={expansionPct}
            class="w-full accent-brand-500 cursor-pointer"
          />
          <div class="flex justify-between text-[10px] text-gray-400 font-mono">
            <span>0%</span>
            <span>+25%</span>
            <span>+50%</span>
            <span>+100%</span>
          </div>
        </div>

        <!-- Options Checkboxes -->
        <div class="space-y-2 pt-1 border-t border-gray-200 dark:border-gray-800">
          <label class="flex items-center gap-2 cursor-pointer">
            <input type="checkbox" bind:checked={useDiacritics} class="rounded text-brand-600 accent-brand-500" />
            <span class="font-medium text-gray-900 dark:text-gray-200">Substitute Accented European Diacritics (ä, é, ö, ç, ñ)</span>
          </label>

          <label class="flex items-center gap-2 cursor-pointer">
            <input type="checkbox" bind:checked={addBrackets} class="rounded text-brand-600 accent-brand-500" />
            <span class="font-medium text-gray-900 dark:text-gray-200">Enclose in Boundary Brackets <code class="text-[11px] bg-gray-100 dark:bg-gray-800 px-1 rounded">[~ ... ~]</code></span>
          </label>

          <label class="flex items-center gap-2 cursor-pointer">
            <input type="checkbox" bind:checked={overwriteConfirmed} class="rounded text-brand-600 accent-brand-500" />
            <span class="text-amber-600 dark:text-amber-400 font-medium">Overwrite already translated/reviewed segments</span>
          </label>
        </div>

        <!-- Live Sample Preview Box -->
        <div class="space-y-1.5 pt-2 border-t border-gray-200 dark:border-gray-800">
          <span class="text-[11px] font-semibold text-gray-500 uppercase tracking-wider">Live Simulation Preview:</span>
          <div class="p-3 bg-gray-900 text-emerald-400 font-mono text-xs rounded-lg border border-gray-700 break-words shadow-inner">
            {samplePreview}
          </div>
        </div>

        <!-- Result message -->
        {#if result}
          <div class="p-3 bg-emerald-50 dark:bg-emerald-950/40 border border-emerald-300 dark:border-emerald-800 rounded-lg text-emerald-800 dark:text-emerald-300">
            <p class="font-bold">✓ Simulation Completed Successfully!</p>
            <p class="text-[11px] mt-0.5">
              Transformed <strong>{result.segments_transformed}</strong> segments ({result.total_words} words). All inline tags were strictly preserved.
            </p>
          </div>
        {/if}

        {#if error}
          <div class="p-3 bg-red-50 dark:bg-red-950/40 border border-red-300 dark:border-red-800 rounded-lg text-red-800 dark:text-red-300">
            <p class="font-bold">Error executing simulation:</p>
            <p class="text-[11px] mt-0.5">{error}</p>
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <div class="px-5 py-3 border-t border-gray-200 dark:border-gray-800 bg-gray-50 dark:bg-gray-850 flex items-center justify-between">
        <button
          type="button"
          onclick={handleClose}
          class="px-3 py-1.5 text-xs font-medium rounded-lg border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
        >
          Close
        </button>

        <button
          type="button"
          onclick={handleRun}
          disabled={isRunning}
          class="px-4 py-1.5 text-xs font-bold rounded-lg bg-brand-600 text-white hover:bg-brand-500 disabled:opacity-50 transition-colors shadow-sm flex items-center gap-1.5"
        >
          {#if isRunning}
            <span class="w-3 h-3 rounded-full border-2 border-white/40 border-t-white animate-spin"></span>
            <span>Simulating…</span>
          {:else}
            <span>🚀 Run Pseudo-Translation</span>
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}
