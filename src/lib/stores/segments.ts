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

export interface AdvancedFilterCriteria {
  isActive: boolean;
  statuses: SegmentStatus[];
  lockedOnly?: boolean | null;
  bookmarkedOnly?: boolean;
  qaIssuesOnly?: boolean;
  commentsOnly?: boolean;
  matchTiers: string[];
  structureTypes: string[];
  sourceRegex?: string;
  targetRegex?: string;
  caseSensitive?: boolean;
  invertMatch?: boolean;
  samplePercent: number;
  sampleSeed?: number;
}

export const defaultAdvancedFilter: AdvancedFilterCriteria = {
  isActive: false,
  statuses: [],
  lockedOnly: null,
  bookmarkedOnly: false,
  qaIssuesOnly: false,
  commentsOnly: false,
  matchTiers: [],
  structureTypes: [],
  sourceRegex: '',
  targetRegex: '',
  caseSensitive: false,
  invertMatch: false,
  samplePercent: 100,
  sampleSeed: 42,
};

export const advancedFilter = writable<AdvancedFilterCriteria>(defaultAdvancedFilter);

// All segments for the current file
export const segments = writable<Segment[]>([]);

// Index of the currently selected segment (0-based)
export const selectedIndex = writable<number>(0);

// Current filter mode
export const filterMode = writable<FilterMode>('all');

// Search query for segment filter
export const segmentSearch = writable<string>('');

function getStructureCodeStr(text: string): string {
  const t = text.trim();
  if (t.startsWith('# ') || t.startsWith('<h1>')) return 'H1';
  if (t.startsWith('## ') || t.startsWith('<h2>')) return 'H2';
  if (t.startsWith('### ') || t.startsWith('<h3>')) return 'H3';
  if (t.startsWith('- ') || t.startsWith('* ') || /^\d+\.\s/.test(t) || t.startsWith('<li>')) return 'LI';
  if (t.includes('|') || t.startsWith('<td>') || t.startsWith('<th>')) return 'TC';
  if (t.toLowerCase().includes('footnote') || /\[\^\d+\]/.test(t)) return 'FN';
  return 'P';
}

function hashString(str: string): number {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    hash = (hash << 5) - hash + str.charCodeAt(i);
    hash |= 0;
  }
  return Math.abs(hash);
}

// Derived: filtered segments based on filterMode, search, and advancedFilter
export const filteredSegments = derived(
  [segments, filterMode, segmentSearch, qaResults, advancedFilter],
  ([$segments, $filter, $search, $qa, $adv]) => {
    let result = $segments;

    if ($adv && $adv.isActive) {
      const qaSegIds = new Set($qa.map(q => q.segment_id).filter(Boolean));
      let srcRe: RegExp | null = null;
      let tgtRe: RegExp | null = null;
      try {
        if ($adv.sourceRegex?.trim()) srcRe = new RegExp($adv.sourceRegex, $adv.caseSensitive ? '' : 'i');
      } catch (e) { /* ignore invalid regex */ }
      try {
        if ($adv.targetRegex?.trim()) tgtRe = new RegExp($adv.targetRegex, $adv.caseSensitive ? '' : 'i');
      } catch (e) { /* ignore invalid regex */ }

      return result.filter(s => {
        let match = true;

        if ($adv.statuses.length > 0 && !$adv.statuses.includes(s.status)) match = false;
        if ($adv.lockedOnly === true && !s.locked) match = false;
        if ($adv.lockedOnly === false && s.locked) match = false;
        if ($adv.bookmarkedOnly && !s.bookmarked) match = false;
        if ($adv.qaIssuesOnly && !qaSegIds.has(s.id)) match = false;
        if ($adv.commentsOnly && (!s.comment || !s.comment.trim())) match = false;

        if ($adv.matchTiers.length > 0) {
          const inTiers = $adv.matchTiers.some(t => {
            if (t === '100') return s.match_percent === 100;
            if (t === 'fuzzy') return s.match_percent !== null && s.match_percent < 100;
            if (t === 'frag') return s.match_origin === 'fragment';
            if (t === 'mt') return s.match_origin === 'mt';
            if (t === 'auto_propagate') return s.match_origin === 'auto_propagate';
            return false;
          });
          if (!inTiers) match = false;
        }

        if ($adv.structureTypes.length > 0) {
          const code = getStructureCodeStr(s.source_text);
          if (!$adv.structureTypes.includes(code)) match = false;
        }

        if (srcRe && !srcRe.test(s.source_text)) match = false;
        if (tgtRe && !tgtRe.test(s.target_text)) match = false;

        if ($adv.samplePercent < 100) {
          const seed = $adv.sampleSeed ?? 42;
          const val = hashString(s.id + seed) % 100;
          if (val >= $adv.samplePercent) match = false;
        }

        return $adv.invertMatch ? !match : match;
      });
    }

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
