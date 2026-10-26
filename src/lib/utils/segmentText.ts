// src/lib/utils/segmentText.ts — pure text helpers for the segment editor.

/** Unique `{n}` / `{/n}` style placeholders in `text`, in order of first appearance. */
export function extractTags(text: string): string[] {
  if (!text) return [];
  const matches = text.match(/\{[^}]+\}/g);
  return matches ? Array.from(new Set(matches)) : [];
}

export interface SegmentStats {
  srcChars: number;
  srcWords: number;
  tgtChars: number;
  tgtWords: number;
  /** Target length vs source length, in percent (0 when either side is empty). */
  expansionRatio: number;
}

const countWords = (s: string) => s.trim().split(/\s+/).filter(Boolean).length;

export function textStats(source: string, target: string): SegmentStats {
  const src = source || '';
  const tgt = target || '';
  const srcChars = src.length;
  const tgtChars = tgt.length;
  return {
    srcChars,
    srcWords: countWords(src),
    tgtChars,
    tgtWords: countWords(tgt),
    expansionRatio:
      srcChars > 0 && tgtChars > 0 ? Math.round(((tgtChars - srcChars) / srcChars) * 100) : 0,
  };
}

/**
 * Next case in the cycle: lower → UPPER → Title → lower.
 * Unicode-aware (é, ß, Greek, Cyrillic...). Caseless scripts such as Arabic
 * come back unchanged. Mixed-case text starts the cycle at lower.
 */
export function cycleCase(text: string): string {
  const lower = text.toLowerCase();
  const upper = text.toUpperCase();
  if (text === lower) return upper; // lower → UPPER (no-op for caseless text)
  if (text === upper) {
    // UPPER → Title: first letter of each word upper, the rest lower.
    return lower.replace(/(^|[^\p{L}\p{N}'’])(\p{L})/gu, (_m, pre: string, ch: string) => pre + ch.toUpperCase());
  }
  return lower; // Title / mixed → lower
}

export interface StructureCode {
  code: string;
  label: string;
  color: string;
}

const PURPLE = 'text-purple-600 dark:text-purple-400 bg-purple-50 dark:bg-purple-950/50 border-purple-200 dark:border-purple-800';
const BLUE = 'text-blue-600 dark:text-blue-400 bg-blue-50 dark:bg-blue-950/50 border-blue-200 dark:border-blue-800';
const EMERALD = 'text-emerald-600 dark:text-emerald-400 bg-emerald-50 dark:bg-emerald-950/50 border-emerald-200 dark:border-emerald-800';
const ORANGE = 'text-orange-600 dark:text-orange-400 bg-orange-50 dark:bg-orange-950/50 border-orange-200 dark:border-orange-800';
const GRAY = 'text-gray-500 dark:text-gray-400 bg-gray-50 dark:bg-gray-800/50 border-gray-200 dark:border-gray-700';

/** Structural role of a segment (heading, list item, table cell, footnote, paragraph). */
export function getStructureCode(text: string): StructureCode {
  const t = text.trim();
  if (t.startsWith('# ') || t.startsWith('<h1>')) return { code: 'H1', label: 'Heading 1', color: PURPLE };
  if (t.startsWith('## ') || t.startsWith('<h2>')) return { code: 'H2', label: 'Heading 2', color: PURPLE };
  if (t.startsWith('### ') || t.startsWith('<h3>')) return { code: 'H3', label: 'Heading 3', color: PURPLE };
  if (t.startsWith('- ') || t.startsWith('* ') || /^\d+\.\s/.test(t) || t.startsWith('<li>')) {
    return { code: 'LI', label: 'List Item', color: BLUE };
  }
  if (t.includes('|') || t.startsWith('<td>') || t.startsWith('<th>')) {
    return { code: 'TC', label: 'Table Cell', color: EMERALD };
  }
  if (t.toLowerCase().includes('footnote') || /\[\^\d+\]/.test(t)) {
    return { code: 'FN', label: 'Footnote', color: ORANGE };
  }
  return { code: 'P', label: 'Paragraph', color: GRAY };
}
