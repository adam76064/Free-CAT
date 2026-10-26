// src/lib/utils/batchPlans.ts
// Pure "what would this batch action change?" rules for the segment grid.
// Each plan returns the patch to apply to a segment, or null to skip it.
// Keeping them free of stores and IPC makes the rules easy to test.
import type { Segment } from '$lib/stores/segments';

export type SegmentPatch = Partial<Pick<Segment, 'target_text' | 'status' | 'locked'>>;
export type BatchPlan = (seg: Segment) => SegmentPatch | null;

export const batchPlans = {
  /** Fill empty, unlocked targets with a copy of the source (as drafts). */
  copySourceToEmpty: (seg) =>
    !seg.target_text.trim() && !seg.locked
      ? { target_text: seg.source_text, status: 'draft' }
      : null,

  /** Clear unlocked draft targets back to "new". */
  clearDrafts: (seg) =>
    seg.status === 'draft' && !seg.locked
      ? { target_text: '', status: 'new' }
      : null,

  /** Lock translated or 100%-match segments that aren't locked yet. */
  lockConfirmed: (seg) =>
    (seg.status === 'translated' || seg.match_percent === 100) && !seg.locked
      ? { locked: true }
      : null,

  /** Unlock everything that is locked. */
  unlockAll: (seg) => (seg.locked ? { locked: false } : null),
} satisfies Record<string, BatchPlan>;

/** The segments a plan would touch, paired with their patch. */
export function planBatch(
  segments: readonly Segment[],
  plan: BatchPlan,
): { segment: Segment; patch: SegmentPatch }[] {
  const out: { segment: Segment; patch: SegmentPatch }[] = [];
  for (const segment of segments) {
    const patch = plan(segment);
    if (patch) out.push({ segment, patch });
  }
  return out;
}
