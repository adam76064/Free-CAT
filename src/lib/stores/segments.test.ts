import { describe, it, expect, beforeEach } from 'vitest';
import { get } from 'svelte/store';
import { segments, updateSegmentsInStore, type Segment } from './segments';

const make = (n: number): Segment[] =>
  Array.from({ length: n }, (_, i) => ({
    id: `s${i}`, source_text: `src ${i}`, target_text: '', status: 'new', locked: false, match_percent: 0,
  }) as Segment);

describe('updateSegmentsInStore', () => {
  beforeEach(() => segments.set(make(5)));

  it('applies only the listed changes and leaves other rows identical (same object)', () => {
    const before = get(segments);
    updateSegmentsInStore(new Map([['s1', { target_text: 'x', status: 'draft' }], ['s3', { locked: true }]]));
    const after = get(segments);
    expect(after[1]).toMatchObject({ target_text: 'x', status: 'draft', source_text: 'src 1' });
    expect(after[3].locked).toBe(true);
    expect(after[0]).toBe(before[0]);
    expect(after[2]).toBe(before[2]);
  });

  it('is a no-op (no store update) for an empty change set', () => {
    const before = get(segments);
    updateSegmentsInStore(new Map());
    expect(get(segments)).toBe(before);
  });

  it('handles a whole-file batch in a single pass (50k rows)', () => {
    segments.set(make(50_000));
    const changes = new Map(get(segments).map((s) => [s.id, { target_text: 'done' } as Partial<Segment>]));
    const t = performance.now();
    updateSegmentsInStore(changes);
    expect(performance.now() - t).toBeLessThan(1000); // the per-segment version is O(n²) and would take minutes
    expect(get(segments)[49_999].target_text).toBe('done');
  });
});
