<script lang="ts">
  /**
   * CommentsPanel — Displays segment notes, comments, and change audit history.
   */
  import { selectedSegment, updateSegmentInStore } from '$lib/stores/segments';
  import { updateSegmentComment, getSegmentHistory, type SegmentHistoryRecord } from '$lib/ipc/segment';

  let currentComment = '';
  let saving = false;
  let historyRecords: SegmentHistoryRecord[] = [];
  let loadingHistory = false;

  $: if ($selectedSegment) {
    currentComment = $selectedSegment.comment || '';
    loadHistory($selectedSegment.id);
  } else {
    currentComment = '';
    historyRecords = [];
  }

  async function loadHistory(segmentId: string) {
    loadingHistory = true;
    try {
      historyRecords = await getSegmentHistory(segmentId);
    } catch (e) {
      console.error('Failed to load segment history:', e);
      historyRecords = [];
    } finally {
      loadingHistory = false;
    }
  }

  async function handleSaveComment() {
    if (!$selectedSegment) return;
    saving = true;
    try {
      const trimmed = currentComment.trim() || null;
      await updateSegmentComment({
        segment_id: $selectedSegment.id,
        comment: trimmed
      });
      updateSegmentInStore($selectedSegment.id, { comment: trimmed });
    } catch (e) {
      console.error('Failed to update comment:', e);
    } finally {
      saving = false;
    }
  }
</script>

<div class="h-full flex flex-col p-4 space-y-5 overflow-y-auto bg-gray-50/50 dark:bg-gray-900/30">
  <!-- Segment Note section -->
  <div>
    <div class="flex items-center justify-between mb-2">
      <h4 class="text-xs font-semibold text-gray-700 dark:text-gray-300 uppercase tracking-wider flex items-center gap-1.5">
        <span>💬</span>
        <span>Segment Note</span>
        {#if $selectedSegment}
          <span class="text-gray-400 font-mono text-[11px]">#{$selectedSegment.seq_number}</span>
        {/if}
      </h4>
      {#if $selectedSegment}
        <button
          on:click={handleSaveComment}
          disabled={saving}
          class="px-2.5 py-1 text-xs font-medium bg-brand-500 hover:bg-brand-600 disabled:opacity-50 text-white rounded shadow-xs transition-colors"
        >
          {saving ? 'Saving…' : 'Save Note'}
        </button>
      {/if}
    </div>

    {#if $selectedSegment}
      <textarea
        bind:value={currentComment}
        placeholder="Add instructions, client feedback, or notes for this segment..."
        rows="3"
        class="w-full text-xs p-2.5 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700
               rounded-lg focus:outline-none focus:ring-1 focus:ring-brand-500 text-gray-800 dark:text-gray-200 resize-none leading-relaxed"
      ></textarea>
    {:else}
      <p class="text-xs text-gray-400 italic">Select a segment to view or add notes.</p>
    {/if}
  </div>

  <!-- Audit History section -->
  <div class="flex-1 flex flex-col min-h-0">
    <h4 class="text-xs font-semibold text-gray-700 dark:text-gray-300 uppercase tracking-wider mb-2 flex items-center gap-1.5">
      <span>⏱</span>
      <span>Audit History & Revisions</span>
    </h4>

    {#if loadingHistory}
      <div class="py-4 text-center text-xs text-gray-400">Loading history…</div>
    {:else if historyRecords.length === 0}
      <div class="py-6 text-center text-xs text-gray-400 border border-dashed border-gray-200 dark:border-gray-800 rounded-lg">
        No revision history recorded for this segment yet.
      </div>
    {:else}
      <div class="space-y-2 overflow-y-auto pr-1">
        {#each historyRecords as record}
          <div class="p-2.5 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700/80 rounded-lg text-xs space-y-1">
            <div class="flex items-center justify-between text-[11px] text-gray-400">
              <span class="font-medium text-brand-600 dark:text-brand-400 capitalize">{record.action}</span>
              <span class="font-mono">{record.timestamp ? record.timestamp.replace('T', ' ').replace('Z', '') : ''}</span>
            </div>
            <div class="text-gray-800 dark:text-gray-200 font-editor leading-snug break-words">
              {record.target_text}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>
