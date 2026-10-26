<script lang="ts">
  /**
   * QuickInsertBar — Typographic & Special Character QuickInsert Strip
   * Inspired by SDL Trados Studio QuickInsert Toolbar (Chapter 31 & Annex C/D).
   * Provides 1-click insertion for professional typographer marks, whitespace characters,
   * quotation marks, dashes, directional isolates, and symbols.
   */
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher<{
    insert: string;
    close: void;
  }>();

  interface QuickInsertItem {
    char: string;
    label: string;
    tooltip: string;
    badge?: string;
  }

  interface QuickInsertGroup {
    name: string;
    items: QuickInsertItem[];
  }

  const GROUPS: QuickInsertGroup[] = [
    {
      name: 'Spaces',
      items: [
        { char: '\u00A0', label: 'NBSP', tooltip: 'Non-Breaking Space (U+00A0)', badge: 'Space' },
        { char: '\u202F', label: 'NNBSP', tooltip: 'Narrow Non-Breaking Space (U+202F) - French Punctuation', badge: 'Narrow' },
        { char: '\u2009', label: 'THIN', tooltip: 'Thin Space (U+2009)' },
        { char: '\u200B', label: 'ZWSP', tooltip: 'Zero-Width Space (U+200B)' },
      ],
    },
    {
      name: 'Dashes & Punct',
      items: [
        { char: '—', label: '—', tooltip: 'Em Dash (U+2014)' },
        { char: '–', label: '–', tooltip: 'En Dash (U+2013)' },
        { char: '…', label: '…', tooltip: 'Horizontal Ellipsis (U+2026)' },
        { char: '•', label: '•', tooltip: 'Bullet (U+2022)' },
      ],
    },
    {
      name: 'Quotes & Guillemets',
      items: [
        { char: '«', label: '«', tooltip: 'Left-Pointing Double Angle Quotation Mark (U+00AB)' },
        { char: '»', label: '»', tooltip: 'Right-Pointing Double Angle Quotation Mark (U+00BB)' },
        { char: '‹', label: '‹', tooltip: 'Single Left-Pointing Guillemet (U+2039)' },
        { char: '›', label: '›', tooltip: 'Single Right-Pointing Guillemet (U+203A)' },
        { char: '“', label: '“', tooltip: 'Left Double Curly Quote (U+201C)' },
        { char: '”', label: '”', tooltip: 'Right Double Curly Quote (U+201D)' },
        { char: '‘', label: '‘', tooltip: 'Left Single Curly Quote (U+2018)' },
        { char: '’', label: '’', tooltip: 'Right Single Curly Quote / Apostrophe (U+2019)' },
      ],
    },
    {
      name: 'Legal & Symbols',
      items: [
        { char: '§', label: '§', tooltip: 'Section Sign (U+00A7)' },
        { char: '°', label: '°', tooltip: 'Degree Sign (U+00B0)' },
        { char: '±', label: '±', tooltip: 'Plus-Minus Sign (U+00B1)' },
        { char: '×', label: '×', tooltip: 'Multiplication Sign (U+00D7)' },
        { char: '™', label: '™', tooltip: 'Trade Mark Sign (U+2122)' },
        { char: '®', label: '®', tooltip: 'Registered Sign (U+00AE)' },
        { char: '©', label: '©', tooltip: 'Copyright Sign (U+00A9)' },
      ],
    },
    {
      name: 'BiDi & Currency',
      items: [
        { char: '\u200E', label: 'LRM', tooltip: 'Left-to-Right Mark (U+200E)' },
        { char: '\u200F', label: 'RLM', tooltip: 'Right-to-Left Mark (U+200F)' },
        { char: '€', label: '€', tooltip: 'Euro Sign (U+20AC)' },
        { char: '£', label: '£', tooltip: 'Pound Sign (U+00A3)' },
        { char: '¥', label: '¥', tooltip: 'Yen Sign (U+00A5)' },
      ],
    },
  ];

  function handleInsert(char: string) {
    dispatch('insert', char);
  }
</script>

<div class="flex items-center gap-2 px-3 py-1.5 bg-gray-50 dark:bg-gray-900 border-b border-gray-200 dark:border-gray-800 text-xs overflow-x-auto select-none shrink-0 shadow-inner">
  <div class="flex items-center gap-1.5 text-gray-500 dark:text-gray-400 font-semibold uppercase tracking-wider text-[10px] shrink-0 mr-1">
    <span>⚡ QuickInsert:</span>
  </div>

  <div class="flex items-center gap-3 min-w-0">
    {#each GROUPS as group}
      <div class="flex items-center gap-1 shrink-0 pl-1.5 border-l border-gray-200 dark:border-gray-700">
        {#each group.items as item}
          <button
            type="button"
            on:click={() => handleInsert(item.char)}
            title="{item.tooltip} — click to insert at cursor"
            class="px-1.5 py-0.5 text-xs font-mono font-medium rounded bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 text-gray-800 dark:text-gray-200 hover:bg-brand-500 hover:text-white dark:hover:bg-brand-600 dark:hover:text-white hover:border-brand-600 transition-colors shadow-2xs active:scale-95 cursor-pointer"
          >
            {item.label}
          </button>
        {/each}
      </div>
    {/each}
  </div>

  <div class="ml-auto shrink-0 pl-2">
    <button aria-label="Hide QuickInsert Bar (Ctrl+Shift+I to toggle)"
      type="button"
      on:click={() => dispatch('close')}
      title="Hide QuickInsert Bar (Ctrl+Shift+I to toggle)"
      class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 p-0.5 text-xs rounded hover:bg-gray-200 dark:hover:bg-gray-800 transition-colors"
    >
      ✕
    </button>
  </div>
</div>
