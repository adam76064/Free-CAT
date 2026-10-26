import { describe, it, expect } from 'vitest';
import { extractTags, textStats, cycleCase, getStructureCode } from './segmentText';

describe('extractTags', () => {
  it('returns unique tags in first-seen order', () => {
    expect(extractTags('a {1} b {2} c {1} {/2}')).toEqual(['{1}', '{2}', '{/2}']);
    expect(extractTags('')).toEqual([]);
    expect(extractTags('no tags here')).toEqual([]);
  });
});

describe('textStats', () => {
  it('counts chars and words and computes expansion', () => {
    expect(textStats('one two', 'one two three four')).toEqual({
      srcChars: 7, srcWords: 2, tgtChars: 18, tgtWords: 4, expansionRatio: 157,
    });
  });
  it('handles empty sides and extra whitespace', () => {
    expect(textStats('', 'abc').expansionRatio).toBe(0);
    expect(textStats('abc', '').expansionRatio).toBe(0);
    expect(textStats('  a   b ', '').srcWords).toBe(2);
  });
  it('counts Arabic words', () => {
    expect(textStats('hello world', 'مرحبا بالعالم').tgtWords).toBe(2);
  });
});

describe('cycleCase', () => {
  it('cycles lower → UPPER → Title → lower (the old code got stuck at UPPER)', () => {
    const a = cycleCase('hello big world');
    expect(a).toBe('HELLO BIG WORLD');
    const b = cycleCase(a);
    expect(b).toBe('Hello Big World');
    const c = cycleCase(b);
    expect(c).toBe('hello big world');
  });
  it('title-cases non-ASCII letters and keeps apostrophes inside words', () => {
    expect(cycleCase('ÉTÉ DÉJÀ VU')).toBe('Été Déjà Vu');
    expect(cycleCase("DON'T STOP")).toBe("Don't Stop");
  });
  it('treats mixed case as "start over at lower"', () => {
    expect(cycleCase('hELLo')).toBe('hello');
  });
  it('leaves caseless scripts unchanged', () => {
    expect(cycleCase('الذاكرة الترجمية')).toBe('الذاكرة الترجمية');
  });
  it('handles single words and empty input', () => {
    expect(cycleCase('word')).toBe('WORD');
    expect(cycleCase('WORD')).toBe('Word');
    expect(cycleCase('')).toBe('');
  });
});

describe('getStructureCode', () => {
  it('classifies headings, lists, tables, footnotes and paragraphs', () => {
    expect(getStructureCode('# Title').code).toBe('H1');
    expect(getStructureCode('## Sub').code).toBe('H2');
    expect(getStructureCode('### Sub').code).toBe('H3');
    expect(getStructureCode('- item').code).toBe('LI');
    expect(getStructureCode('3. item').code).toBe('LI');
    expect(getStructureCode('a | b').code).toBe('TC');
    expect(getStructureCode('see [^1]').code).toBe('FN');
    expect(getStructureCode('Plain sentence.').code).toBe('P');
    expect(getStructureCode('  ## padded').code).toBe('H2');
  });
});
