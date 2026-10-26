import { describe, it, expect } from 'vitest';
import { batchPlans, planBatch } from './batchPlans';
import type { Segment } from '$lib/stores/segments';

const seg = (over: Partial<Segment>): Segment =>
  ({ id: 'x', source_text: 'src', target_text: '', status: 'new', locked: false, match_percent: 0, ...over }) as Segment;

describe('batchPlans', () => {
  it('copySourceToEmpty fills only empty, unlocked targets (whitespace counts as empty)', () => {
    expect(batchPlans.copySourceToEmpty(seg({}))).toEqual({ target_text: 'src', status: 'draft' });
    expect(batchPlans.copySourceToEmpty(seg({ target_text: '   ' }))).not.toBeNull();
    expect(batchPlans.copySourceToEmpty(seg({ target_text: 'done' }))).toBeNull();
    expect(batchPlans.copySourceToEmpty(seg({ locked: true }))).toBeNull();
  });

  it('clearDrafts only touches unlocked drafts', () => {
    expect(batchPlans.clearDrafts(seg({ status: 'draft', target_text: 'x' }))).toEqual({ target_text: '', status: 'new' });
    expect(batchPlans.clearDrafts(seg({ status: 'draft', locked: true }))).toBeNull();
    expect(batchPlans.clearDrafts(seg({ status: 'translated' }))).toBeNull();
  });

  it('lockConfirmed locks translated or 100% segments that are unlocked', () => {
    expect(batchPlans.lockConfirmed(seg({ status: 'translated' }))).toEqual({ locked: true });
    expect(batchPlans.lockConfirmed(seg({ match_percent: 100 }))).toEqual({ locked: true });
    expect(batchPlans.lockConfirmed(seg({ status: 'translated', locked: true }))).toBeNull();
    expect(batchPlans.lockConfirmed(seg({ status: 'draft', match_percent: 85 }))).toBeNull();
  });

  it('unlockAll unlocks only locked segments', () => {
    expect(batchPlans.unlockAll(seg({ locked: true }))).toEqual({ locked: false });
    expect(batchPlans.unlockAll(seg({}))).toBeNull();
  });

  it('planBatch pairs each affected segment with its patch and preserves order', () => {
    const list = [seg({ id: 'a' }), seg({ id: 'b', target_text: 'x' }), seg({ id: 'c' })];
    const plan = planBatch(list, batchPlans.copySourceToEmpty);
    expect(plan.map((p) => p.segment.id)).toEqual(['a', 'c']);
  });
});
