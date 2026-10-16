// src/lib/stores/segments.ts
// Svelte stores for segment data and editor navigation state.

import { writable, derived } from 'svelte/store';
import { qaResults } from './qa';

export type SegmentStatus = 'new' | 'draft' | 'translated' | 'needs_review' | 'reviewed' | 'final';

export interface Segment {
  id: string;
  file_id: string;
  seq_number: number;
  source_text: string;
  target_text: string;
  status: SegmentStatus;
  match_percent: number | null;
  match_origin: string | null;
  locked: boolean;
  bookmarked: boolean;
  comment: string | null;
  created_at: string;
  modified_at: string;
  modified_by: string | null;
}

export type FilterMode = 'all' | 'new' | 'draft' | 'translated' | 'needs_review' | 'reviewed' | 'bookmarked' | 'qa_warning';

// All segments for the current file
export const segments = writable<Segment[]>([]);

// Index of the currently selected segment (0-based)
export const selectedIndex = writable<number>(0);

// Current filter mode
export const filterMode = writable<FilterMode>('all');

// Search query for segment filter
export const segmentSearch = writable<string>('');

// Derived: filtered segments based on filterMode and search
export const filteredSegments = derived(
  [segments, filterMode, segmentSearch, qaResults],
  ([$segments, $filter, $search, $qa]) => {
    let result = $segments;

    if ($filter !== 'all') {
      if ($filter === 'bookmarked') {
        result = result.filter(s => s.bookmarked);
      } else if ($filter === 'qa_warning') {
        const qaSegIds = new Set($qa.map(q => q.segment_id).filter(Boolean));
        result = result.filter(s => qaSegIds.has(s.id));
      } else {
        result = result.filter(s => s.status === $filter);
      }
    }

    if ($search.trim()) {
      const q = $search.toLowerCase();
      result = result.filter(s =>
        s.source_text.toLowerCase().includes(q) ||
        s.target_text.toLowerCase().includes(q)
      );
    }

    return result;
  }
);

// Derived: currently selected segment
export const selectedSegment = derived(
  [filteredSegments, selectedIndex],
  ([$segs, $idx]) => $segs[$idx] ?? null
);

// Derived: progress stats
export const progressStats = derived(segments, ($segs) => {
  const total = $segs.length;
  const translated = $segs.filter(s =>
    s.status === 'translated' || s.status === 'reviewed' || s.status === 'final'
  ).length;
  const reviewed = $segs.filter(s => s.status === 'reviewed' || s.status === 'final').length;
  const pct = total > 0 ? Math.round((translated / total) * 100) : 0;
  return { total, translated, reviewed, pct };
});

// Update a single segment in the store
export function updateSegmentInStore(id: string, changes: Partial<Segment>) {
  segments.update(segs =>
    segs.map(s => s.id === id ? { ...s, ...changes } : s)
  );
}

// Status display helpers
export const STATUS_LABELS: Record<SegmentStatus, string> = {
  new:         'New',
  draft:       'Draft',
  translated:  'Translated',
  needs_review:'Needs Review',
  reviewed:    'Reviewed',
  final:       'Final',
};

export const STATUS_COLORS: Record<SegmentStatus, string> = {
  new:          'bg-gray-100 text-gray-500 dark:bg-gray-800 dark:text-gray-400',
  draft:        'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400',
  translated:   'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400',
  needs_review: 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400',
  reviewed:     'bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400',
  final:        'bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-400',
};
