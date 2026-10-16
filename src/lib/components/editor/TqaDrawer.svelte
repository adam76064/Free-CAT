<script lang="ts">
  /**
   * TqaDrawer — Interactive TAUS DQF-MQM Translation Quality Assessment Studio
   * Inspired by SDL Trados Studio Chapter 40 (Verification & TQA).
   * Computes standardized MQM Quality Scores against industry tolerance thresholds (>= 95% PASS).
   */
  import { onMount, createEventDispatcher } from 'svelte';
  import { currentProject } from '$lib/stores/project';
  import { selectedSegment, segments, progressStats } from '$lib/stores/segments';
  import {
    addTqaError,
    deleteTqaError,
    getTqaScorecard,
    getSegmentTqaErrors,
    type TqaError,
    type TqaScorecard
  } from '$lib/ipc/tqa';

  export let isOpen: boolean = false;

  const dispatch = createEventDispatcher<{
    close: void;
    errorLogged: TqaError;
  }>();

  let scorecard: TqaScorecard | null = null;
  let activeSegmentErrors: TqaError[] = [];
  let loading = false;

  // New error form state
  let category = 'accuracy';
  let subType = 'mistranslation';
  let severity: 'minor' | 'major' | 'critical' = 'minor';
  let comment = '';
  let submitting = false;

  const CATEGORIES = [
    {
      id: 'accuracy',
      label: 'Accuracy',
      subTypes: ['mistranslation', 'omission', 'addition', 'untranslated'],
    },
    {
      id: 'fluency',
      label: 'Fluency',
      subTypes: ['grammar', 'spelling', 'punctuation', 'typo'],
    },
    {
      id: 'terminology',
      label: 'Terminology',
      subTypes: ['glossary_noncompliance', 'inconsistent_term', 'wrong_context'],
    },
    {
      id: 'style',
      label: 'Style',
      subTypes: ['awkward_phrasing', 'wrong_tone_register', 'unnatural_flow'],
    },
    {
      id: 'design',
      label: 'Design & Formatting',
      subTypes: ['tag_misplacement', 'whitespace_error', 'truncation'],
    },
  ];

  $: activeCategoryObj = CATEGORIES.find(c => c.id === category) || CATEGORIES[0];

  $: {
    if (activeCategoryObj && !activeCategoryObj.subTypes.includes(subType)) {
      subType = activeCategoryObj.subTypes[0];
    }
  }

  $: totalWords = $segments.reduce((acc, s) => acc + s.source_text.trim().split(/\s+/).filter(Boolean).length, 0);

  export async function refresh() {
    if (!$currentProject) return;
    loading = true;
    try {
      scorecard = await getTqaScorecard($currentProject.id, totalWords || 100);
      if ($selectedSegment) {
        activeSegmentErrors = await getSegmentTqaErrors($selectedSegment.id);
      } else {
        activeSegmentErrors = [];
      }
    } catch (e) {
      console.error('Failed to load TQA scorecard:', e);
    } finally {
      loading = false;
    }
  }

  $: if (isOpen && $selectedSegment) {
    refresh();
  }

  async function handleAddError() {
    if (!$currentProject || !$selectedSegment) return;
    submitting = true;
    try {
      const err = await addTqaError({
        project_id: $currentProject.id,
        segment_id: $selectedSegment.id,
        category,
        sub_type: subType,
        severity,
        comment: comment.trim() || undefined,
      });
      comment = '';
      await refresh();
      dispatch('errorLogged', err);
    } catch (e: any) {
      alert(`Failed to log TQA error: ${e}`);
    } finally {
      submitting = false;
    }
  }

  async function handleDeleteError(id?: number) {
    if (!id) return;
    try {
      await deleteTqaError(id);
      await refresh();
    } catch (e: any) {
      alert(`Failed to delete error: ${e}`);
    }
  }
</script>

{#if isOpen}
  <div class="fixed inset-0 bg-black/50 backdrop-blur-xs z-50 flex justify-end animate-in fade-in duration-150">
    <div class="w-full max-w-md bg-white dark:bg-gray-900 h-full shadow-2xl flex flex-col border-l border-gray-200 dark:border-gray-800 animate-in slide-in-from-right duration-200">
      
      <!-- Header -->
      <div class="flex items-center justify-between px-4 py-3 border-b border-gray-200 dark:border-gray-800 bg-gray-50 dark:bg-gray-850 shrink-0">
        <div class="flex items-center gap-2">
          <span class="text-base">🛡️</span>
          <div>
            <h3 class="text-xs font-bold text-gray-900 dark:text-white uppercase tracking-wider">
              Translation Quality Assessment (TQA)
            </h3>
            <p class="text-[10px] text-gray-500 dark:text-gray-400">
              TAUS DQF / MQM Metric Evaluation Studio
            </p>
          </div>
        </div>
        <button
          type="button"
          on:click={() => dispatch('close')}
          class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 text-sm p-1 rounded-md"
        >
          ✕
        </button>
      </div>

      <!-- Live Scorecard Banner -->
      <div class="px-4 py-3 bg-gradient-to-r from-gray-50 to-gray-100 dark:from-gray-800/80 dark:to-gray-900/80 border-b border-gray-200 dark:border-gray-800 shrink-0">
        <div class="flex items-center justify-between">
          <div>
            <span class="text-[10px] text-gray-500 uppercase tracking-wider font-semibold">MQM Quality Score</span>
            <div class="flex items-baseline gap-2 mt-0.5">
              <span class="text-2xl font-black font-mono {scorecard?.passed ? 'text-emerald-600 dark:text-emerald-400' : 'text-red-600 dark:text-red-400'}">
                {scorecard ? scorecard.score_pct.toFixed(1) : '100.0'}%
              </span>
              <span class="text-xs px-2 py-0.5 rounded-full font-bold uppercase tracking-wider {scorecard?.passed ? 'bg-emerald-100 text-emerald-800 dark:bg-emerald-950 dark:text-emerald-300' : 'bg-red-100 text-red-800 dark:bg-red-950 dark:text-red-300'}">
                {scorecard?.passed ? 'PASS' : 'FAIL'}
              </span>
            </div>
          </div>

          <div class="text-right text-[11px] font-mono space-y-0.5 text-gray-600 dark:text-gray-400">
            <div>Minor (-1): <strong class="text-gray-900 dark:text-gray-200">{scorecard?.minor_count ?? 0}</strong></div>
            <div>Major (-5): <strong class="text-amber-600 dark:text-amber-400">{scorecard?.major_count ?? 0}</strong></div>
            <div>Critical (-25): <strong class="text-red-600 dark:text-red-400">{scorecard?.critical_count ?? 0}</strong></div>
          </div>
        </div>

        <div class="mt-2 text-[10px] text-gray-500 flex justify-between border-t border-gray-200/60 dark:border-gray-700/60 pt-1.5">
          <span>Threshold: ≥ 95.0% (0 Critical)</span>
          <span>Total Penalties: -{scorecard?.total_penalty ?? 0} pts</span>
        </div>
      </div>

      <!-- Drawer Content -->
      <div class="flex-1 overflow-y-auto p-4 space-y-4 text-xs">
        
        <!-- Active Segment Context -->
        {#if $selectedSegment}
          <div class="p-3 bg-gray-50 dark:bg-gray-800/50 rounded-lg border border-gray-200 dark:border-gray-700 space-y-1.5">
            <div class="flex items-center justify-between text-[10px] text-gray-500">
              <span class="font-bold uppercase tracking-wider">Evaluating Segment #{ $selectedSegment.seq_number }</span>
              <span class="font-mono">{ $selectedSegment.id.slice(0, 8) }…</span>
            </div>
            <p class="text-gray-800 dark:text-gray-200 text-xs italic line-clamp-2">
              "{ $selectedSegment.target_text || '— empty target —' }"
            </p>
          </div>
        {/if}

        <!-- Add Error Form -->
        <div class="p-3.5 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 space-y-3 shadow-xs">
          <h4 class="font-bold text-gray-900 dark:text-white uppercase tracking-wider text-[11px] flex items-center gap-1.5">
            <span>➕</span> Log Quality Issue
          </h4>

          <!-- Category Selection -->
          <div class="space-y-1">
            <label class="text-[10px] font-semibold text-gray-500 uppercase tracking-wider">MQM Category</label>
            <select
              bind:value={category}
              class="w-full px-2 py-1.5 text-xs rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-750 text-gray-900 dark:text-white focus:ring-1 focus:ring-brand-500"
            >
              {#each CATEGORIES as cat}
                <option value={cat.id}>{cat.label}</option>
              {/each}
            </select>
          </div>

          <!-- Sub-Type Selection -->
          <div class="space-y-1">
            <label class="text-[10px] font-semibold text-gray-500 uppercase tracking-wider">Issue Type</label>
            <select
              bind:value={subType}
              class="w-full px-2 py-1.5 text-xs rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-750 text-gray-900 dark:text-white focus:ring-1 focus:ring-brand-500"
            >
              {#each activeCategoryObj.subTypes as st}
                <option value={st}>{st.replace(/_/g, ' ')}</option>
              {/each}
            </select>
          </div>

          <!-- Severity Selection -->
          <div class="space-y-1">
            <label class="text-[10px] font-semibold text-gray-500 uppercase tracking-wider">Severity & Weight</label>
            <div class="grid grid-cols-3 gap-1.5">
              <button
                type="button"
                on:click={() => (severity = 'minor')}
                class="py-1 px-1.5 text-[11px] font-semibold rounded border text-center transition-colors
                       {severity === 'minor' ? 'bg-blue-500 text-white border-blue-600 shadow-xs' : 'border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300'}"
              >
                Minor (-1)
              </button>
              <button
                type="button"
                on:click={() => (severity = 'major')}
                class="py-1 px-1.5 text-[11px] font-semibold rounded border text-center transition-colors
                       {severity === 'major' ? 'bg-amber-500 text-white border-amber-600 shadow-xs' : 'border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300'}"
              >
                Major (-5)
              </button>
              <button
                type="button"
                on:click={() => (severity = 'critical')}
                class="py-1 px-1.5 text-[11px] font-semibold rounded border text-center transition-colors
                       {severity === 'critical' ? 'bg-red-600 text-white border-red-700 shadow-xs' : 'border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300'}"
              >
                Critical (-25)
              </button>
            </div>
          </div>

          <!-- Comment Textarea -->
          <div class="space-y-1">
            <label class="text-[10px] font-semibold text-gray-500 uppercase tracking-wider">Reviewer Comment</label>
            <textarea
              bind:value={comment}
              rows="2"
              placeholder="e.g. Inaccurate translation of technical term 'latch'..."
              class="w-full px-2 py-1.5 text-xs rounded border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-750 text-gray-900 dark:text-white focus:ring-1 focus:ring-brand-500"
            ></textarea>
          </div>

          <!-- Submit Button -->
          <button
            type="button"
            on:click={handleAddError}
            disabled={submitting || !$selectedSegment}
            class="w-full py-1.5 px-3 rounded-lg bg-brand-600 hover:bg-brand-500 disabled:opacity-50 text-white font-bold text-xs transition-colors shadow-sm cursor-pointer"
          >
            {submitting ? 'Logging Issue…' : 'Log TQA Error on Segment'}
          </button>
        </div>

        <!-- Errors on this Segment -->
        <div class="space-y-2">
          <div class="flex items-center justify-between text-[11px] font-bold text-gray-700 dark:text-gray-300">
            <span>Errors on Current Segment ({activeSegmentErrors.length}):</span>
          </div>

          {#if activeSegmentErrors.length === 0}
            <div class="p-3 text-center text-gray-400 text-xs italic bg-gray-50 dark:bg-gray-800/30 rounded border border-dashed border-gray-200 dark:border-gray-700">
              No quality errors recorded for this segment.
            </div>
          {:else}
            <div class="space-y-1.5">
              {#each activeSegmentErrors as err}
                <div class="p-2.5 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 flex items-start justify-between gap-2 shadow-2xs">
                  <div class="space-y-0.5 min-w-0">
                    <div class="flex items-center gap-1.5">
                      <span class="px-1.5 py-0.2 rounded text-[10px] font-bold uppercase
                        {err.severity === 'critical' ? 'bg-red-100 text-red-700 dark:bg-red-950 dark:text-red-300' : err.severity === 'major' ? 'bg-amber-100 text-amber-700 dark:bg-amber-950 dark:text-amber-300' : 'bg-blue-100 text-blue-700 dark:bg-blue-950 dark:text-blue-300'}">
                        {err.severity} (-{err.penalty})
                      </span>
                      <span class="font-bold text-gray-900 dark:text-white text-xs capitalize">{err.category}</span>
                      <span class="text-gray-400 text-[10px]">({err.sub_type})</span>
                    </div>
                    {#if err.comment}
                      <p class="text-gray-600 dark:text-gray-400 text-xs mt-0.5">{err.comment}</p>
                    {/if}
                  </div>
                  <button
                    type="button"
                    on:click={() => handleDeleteError(err.id)}
                    class="text-gray-400 hover:text-red-500 p-1 text-xs transition-colors"
                    title="Delete Error"
                  >
                    🗑️
                  </button>
                </div>
              {/each}
            </div>
          {/if}
        </div>

        <!-- All Project Issues Summary -->
        {#if scorecard && scorecard.errors.length > 0}
          <div class="pt-2 border-t border-gray-200 dark:border-gray-800 space-y-2">
            <span class="text-[11px] font-bold text-gray-700 dark:text-gray-300">
              All Project Quality Issues ({scorecard.errors.length}):
            </span>
            <div class="space-y-1 max-h-48 overflow-y-auto">
              {#each scorecard.errors as err}
                <div class="px-2 py-1.5 rounded bg-gray-50 dark:bg-gray-800/40 border border-gray-200 dark:border-gray-700/60 flex items-center justify-between text-xs">
                  <div class="flex items-center gap-1.5 truncate">
                    <span class="text-[10px] font-mono px-1 rounded bg-gray-200 dark:bg-gray-700">Seg #{err.segment_id.slice(0, 6)}</span>
                    <span class="font-medium text-gray-900 dark:text-gray-200 capitalize">{err.category}</span>
                    <span class="text-gray-400 text-[10px]">({err.severity})</span>
                  </div>
                  <span class="text-red-500 font-mono text-[11px]">-{err.penalty}</span>
                </div>
              {/each}
            </div>
          </div>
        {/if}

      </div>
    </div>
  </div>
{/if}
