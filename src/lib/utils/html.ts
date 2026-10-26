// src/lib/utils/html.ts
// Safe HTML construction for {@html} blocks.
//
// Segment text comes from imported files (XLIFF, TMX, DOCX, ...) and must be
// treated as untrusted. Every helper here escapes the raw text itself and only
// adds our own markup around it, always working on raw text first so that
// highlighting can never match inside markup we generated.

export function escapeHtml(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

export function escapeRegExp(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

/**
 * Escape `text`, wrapping every match of `re` (which must be global) with
 * `wrap(escapedMatch)`. Non-matching and matching parts are escaped separately.
 */
export function escapeAndWrap(
  text: string,
  re: RegExp,
  wrap: (escapedMatch: string) => string,
): string {
  let out = '';
  let last = 0;
  for (const m of text.matchAll(re)) {
    if (m[0].length === 0) continue;
    const idx = m.index ?? 0;
    out += escapeHtml(text.slice(last, idx)) + wrap(escapeHtml(m[0]));
    last = idx + m[0].length;
  }
  return out + escapeHtml(text.slice(last));
}

const CONCORDANCE_MARK =
  'bg-amber-200 dark:bg-amber-900/80 text-amber-950 dark:text-amber-100 rounded px-0.5 font-bold';

/** Concordance search: highlight the query (case-insensitive) in `text`. */
export function highlightMatches(text: string, query: string): string {
  if (!text) return '';
  if (!query) return escapeHtml(text);
  const re = new RegExp(escapeRegExp(query), 'gi');
  return escapeAndWrap(text, re, (m) => `<mark class="${CONCORDANCE_MARK}">${m}</mark>`);
}

const TAG_PILL =
  'inline-flex items-center gap-0.5 font-mono text-[11px] font-semibold bg-amber-100 text-amber-900 dark:bg-amber-950/70 dark:text-amber-300 border border-amber-300/80 dark:border-amber-700/80 px-1.5 py-0.2 rounded select-none mx-0.5 tracking-tight cursor-default';
const SPOTLIGHT_MARK =
  'bg-cyan-200 dark:bg-cyan-900/90 text-cyan-950 dark:text-cyan-100 ring-2 ring-cyan-500 rounded px-1 animate-pulse font-bold';

/**
 * Source cell: render `{n}` placeholders as tag pills and spotlight an
 * optional token. The token is only searched for in plain text, never inside
 * a tag pill, so the generated markup can't be corrupted.
 */
export function renderSource(text: string, highlightToken: string | null): string {
  const spot =
    highlightToken && highlightToken.trim() && !highlightToken.startsWith('{')
      ? new RegExp(escapeRegExp(highlightToken), 'g')
      : null;

  let out = '';
  let last = 0;
  const plain = (chunk: string) =>
    spot
      ? escapeAndWrap(chunk, spot, (m) => `<mark class="${SPOTLIGHT_MARK}">${m}</mark>`)
      : escapeHtml(chunk);

  for (const m of text.matchAll(/\{([^}]+)\}/g)) {
    const idx = m.index ?? 0;
    const label = escapeHtml(m[1]);
    out += plain(text.slice(last, idx));
    out += `<span class="${TAG_PILL}" title="Tag placeholder {${label}}">🏷️${label}</span>`;
    last = idx + m[0].length;
  }
  return out + plain(text.slice(last));
}

/** TM panel: mark words of the TM source that differ from the current source. */
export function highlightDiff(tmSrc: string, curSrc: string): string {
  if (tmSrc === curSrc) return escapeHtml(tmSrc);
  const cur = new Set(curSrc.split(' '));
  return tmSrc
    .split(' ')
    .map((w) =>
      cur.has(w)
        ? escapeHtml(w)
        : `<mark class="bg-red-100 dark:bg-red-900/30 rounded px-0.5">${escapeHtml(w)}</mark>`,
    )
    .join(' ');
}
