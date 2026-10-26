import { describe, it } from 'vitest';
import assert from 'node:assert/strict';
import { escapeHtml, highlightMatches, renderSource, highlightDiff } from './html';

describe('html helpers', () => {
  it('escape untrusted text, never corrupt markup, and highlight correctly', () => {
    const evil = '<img src=x onerror=alert(1)>';
    // no raw "<" from untrusted input may survive anywhere
    for (const out of [
      escapeHtml(evil),
      highlightMatches(evil, 'img'),
      highlightMatches(`a ${evil} b`, 'a'),
      renderSource(evil, null),
      renderSource(`x {1} ${evil}`, 'src'),
      highlightDiff(evil, 'other'),
      highlightDiff(`ok ${evil}`, 'ok'),
    ]) {
      const stripped = out.replace(/<\/?(mark|span)\b[^>]*>/g, '');
      assert.ok(!/[<>]/.test(stripped), 'raw angle bracket leaked: ' + out);
      assert.ok(!/<img/i.test(out));
    }
    // quotes in tag labels cannot break out of the title attribute
    const q = renderSource('{a" onmouseover="x}', null);
    assert.ok(!/title="[^"]*"[^>]*onmouseover/.test(q), q);
    assert.ok(q.includes('&quot;'));

    // highlighting never corrupts generated markup
    const r = renderSource('Use font span class {1} here', 'span');
    assert.equal((r.match(/<span /g) || []).length, 1, r);            // only the one tag pill
    assert.equal((r.match(/<mark /g) || []).length, 1, r);            // only the plain-text "span"
    assert.ok(r.includes('title="Tag placeholder {1}"'));
    const r2 = renderSource('font {1}', 'font');
    assert.ok(r2.startsWith('<mark ') && r2.includes('>font</mark>'), r2);
    assert.ok(!r2.includes('<mark', r2.indexOf('</mark>') + 7));

    // matching behaviour
    assert.equal(highlightMatches('Hello hello', 'HELLO').match(/<mark/g)!.length, 2);
    assert.equal(highlightMatches('a.b', 'a.b'), highlightMatches('a.b', 'a.b'));
    assert.ok(!highlightMatches('axb', 'a.b').includes('<mark'));      // regex chars are literal
    assert.equal(highlightMatches('R&D', 'R&D'), '<mark class="bg-amber-200 dark:bg-amber-900/80 text-amber-950 dark:text-amber-100 rounded px-0.5 font-bold">R&amp;D</mark>');
    assert.equal(highlightMatches('plain & text', ''), 'plain &amp; text');
    // Arabic passes through untouched and is highlightable
    assert.ok(highlightMatches('الذاكرة الترجمية', 'الترجمية').includes('>الترجمية</mark>'));
    assert.equal(renderSource('', null), '');
    assert.equal(highlightDiff('same', 'same'), 'same');
  });
});
