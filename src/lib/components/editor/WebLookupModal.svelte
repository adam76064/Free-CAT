<script lang="ts">
  import { dialog } from '$lib/actions/dialog';
  /**
   * WebLookupModal.svelte — Studio Web Lookup! & Multi-Resource Concordance Omnibar
   * (Trados Studio Manual Chapter 30, pp. 278-280)
   * Instant 1-click external lookup across top terminology, MT, and dictionary portals.
   */
  import { createEventDispatcher, onMount } from 'svelte';

  export let isOpen = false;
  export let initialQuery = '';
  export let srcLang = 'en';
  export let tgtLang = 'es';

  const dispatch = createEventDispatcher();

  let query = '';
  let inputEl: HTMLInputElement;

  $: if (isOpen) {
    query = initialQuery.trim();
    setTimeout(() => {
      inputEl?.focus();
      inputEl?.select();
    }, 50);
  }

  interface WebProvider {
    id: string;
    num: number;
    name: string;
    icon: string;
    category: 'search' | 'mt' | 'concordance' | 'dict' | 'wiki';
    urlGenerator: (q: string, sl: string, tl: string) => string;
    description: string;
  }

  const providers: WebProvider[] = [
    {
      id: 'google_search',
      num: 1,
      name: 'Google Search',
      icon: '🔍',
      category: 'search',
      description: 'Web concordance & contextual examples',
      urlGenerator: (q) => `https://www.google.com/search?q=${encodeURIComponent(q)}`
    },
    {
      id: 'deepl',
      num: 2,
      name: 'DeepL Translator',
      icon: '🧠',
      category: 'mt',
      description: 'High-accuracy neural machine translation',
      urlGenerator: (q, sl, tl) => {
        const s = sl.split('-')[0].toLowerCase();
        const t = tl.split('-')[0].toLowerCase();
        return `https://www.deepl.com/translator#${s}/${t}/${encodeURIComponent(q)}`;
      }
    },
    {
      id: 'google_translate',
      num: 3,
      name: 'Google Translate',
      icon: '🌐',
      category: 'mt',
      description: 'Broad multilingual machine translation',
      urlGenerator: (q, sl, tl) => {
        const s = sl.split('-')[0].toLowerCase();
        const t = tl.split('-')[0].toLowerCase();
        return `https://translate.google.com/?sl=${s}&tl=${t}&text=${encodeURIComponent(q)}&op=translate`;
      }
    },
    {
      id: 'linguee',
      num: 4,
      name: 'Linguee',
      icon: '📚',
      category: 'concordance',
      description: 'Bilingual sentence pairs & authentic corpus',
      urlGenerator: (q, sl, tl) => {
        const s = sl.split('-')[0].toLowerCase();
        const t = tl.split('-')[0].toLowerCase();
        return `https://www.linguee.com/${s}-${t}/search?source=auto&query=${encodeURIComponent(q)}`;
      }
    },
    {
      id: 'proz_kudoz',
      num: 5,
      name: 'ProZ KudoZ',
      icon: '💬',
      category: 'dict',
      description: 'Professional human translator terminology forum',
      urlGenerator: (q) => `https://www.proz.com/kudoz/search?q=${encodeURIComponent(q)}`
    },
    {
      id: 'iate',
      num: 6,
      name: 'IATE (EU Terminology)',
      icon: '🇪🇺',
      category: 'dict',
      description: 'Interactive Terminology for Europe legal/tech corpus',
      urlGenerator: (q) => `https://iate.europa.eu/search-result?term=${encodeURIComponent(q)}`
    },
    {
      id: 'wordreference',
      num: 7,
      name: 'WordReference',
      icon: '📖',
      category: 'dict',
      description: 'Comprehensive multilingual dictionary & discussion',
      urlGenerator: (q, sl, tl) => {
        const s = sl.split('-')[0].toLowerCase();
        const t = tl.split('-')[0].toLowerCase();
        return `https://www.wordreference.com/${s}${t}/${encodeURIComponent(q)}`;
      }
    },
    {
      id: 'wikipedia',
      num: 8,
      name: 'Wikipedia',
      icon: '🏛️',
      category: 'wiki',
      description: 'Encyclopedic concepts & cross-language interwiki',
      urlGenerator: (q, sl) => {
        const s = sl.split('-')[0].toLowerCase();
        return `https://${s}.wikipedia.org/wiki/Special:Search?search=${encodeURIComponent(q)}`;
      }
    },
    {
      id: 'wiktionary',
      num: 9,
      name: 'Wiktionary',
      icon: '📜',
      category: 'wiki',
      description: 'Morphology, inflections, and etymology definitions',
      urlGenerator: (q, sl) => {
        const s = sl.split('-')[0].toLowerCase();
        return `https://${s}.wiktionary.org/wiki/Special:Search?search=${encodeURIComponent(q)}`;
      }
    }
  ];

  function openProvider(provider: WebProvider) {
    if (!query.trim()) return;
    const url = provider.urlGenerator(query.trim(), srcLang, tgtLang);
    window.open(url, '_blank');
    dispatch('close');
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!isOpen) return;

    if (e.key === 'Escape') {
      dispatch('close');
    } else if (e.key === 'Enter') {
      e.preventDefault();
      // Default to Google or DeepL
      if (providers.length > 0) {
        openProvider(providers[0]);
      }
    } else if (e.altKey && e.key >= '1' && e.key <= '9') {
      const idx = parseInt(e.key, 10) - 1;
      if (providers[idx]) {
        e.preventDefault();
        openProvider(providers[idx]);
      }
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if isOpen}
  <!-- Backdrop -->
  <div use:dialog={{ label: "Web lookup and concordance" }} class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4 animate-in fade-in duration-100">
    <div class="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-xl shadow-2xl max-w-xl w-full flex flex-col overflow-hidden">
      <!-- Header -->
      <div class="px-5 py-3.5 border-b border-gray-200 dark:border-gray-800 flex items-center justify-between bg-gray-50/80 dark:bg-gray-900/80">
        <div class="flex items-center gap-2.5">
          <span class="text-xl">🌐</span>
          <div>
            <h3 class="text-sm font-bold text-gray-900 dark:text-gray-100">
              Web Lookup! & Concordance Omnibar
            </h3>
            <p class="text-[11px] text-gray-500 dark:text-gray-400">
              Query external dictionaries, forums, and corpora across <span class="font-mono font-bold uppercase">{srcLang}</span> ➔ <span class="font-mono font-bold uppercase">{tgtLang}</span>
            </p>
          </div>
        </div>
        <button
          type="button"
          on:click={() => dispatch('close')}
          class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 text-lg leading-none p-1 cursor-pointer"
        >
          ✕
        </button>
      </div>

      <!-- Search Input Area -->
      <div class="p-4 border-b border-gray-100 dark:border-gray-800 bg-white dark:bg-gray-950">
        <div class="relative">
          <input
            bind:this={inputEl}
            type="text"
            bind:value={query}
            placeholder="Type word, phrase, or terminology to look up..."
            class="w-full text-sm font-editor px-3.5 py-2.5 pl-10 rounded-lg bg-gray-50 dark:bg-gray-900 border border-gray-300 dark:border-gray-700 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-brand-500 font-medium"
          />
          <span class="absolute left-3.5 top-3 text-sm text-gray-400">🔎</span>
          {#if query}
            <button
              on:click={() => (query = '')}
              class="absolute right-3 top-3 text-xs text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 cursor-pointer"
            >
              ✕
            </button>
          {/if}
        </div>
      </div>

      <!-- Providers Grid -->
      <div class="p-4 grid grid-cols-1 sm:grid-cols-2 gap-2 max-h-[50vh] overflow-y-auto bg-gray-50/50 dark:bg-gray-900/30">
        {#each providers as prov}
          <button
            type="button"
            on:click={() => openProvider(prov)}
            class="flex items-start gap-2.5 p-2.5 rounded-lg border border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 hover:border-brand-500 dark:hover:border-brand-500 hover:shadow-xs transition-all text-left cursor-pointer group"
          >
            <div class="w-7 h-7 rounded-md bg-gray-100 dark:bg-gray-800 flex items-center justify-center text-sm shrink-0 group-hover:scale-110 transition-transform">
              {prov.icon}
            </div>
            <div class="flex-1 min-w-0">
              <div class="flex items-center justify-between">
                <span class="text-xs font-bold text-gray-900 dark:text-gray-100 group-hover:text-brand-600 dark:group-hover:text-brand-400">
                  {prov.name}
                </span>
                <span class="font-mono text-[10px] text-gray-400 bg-gray-100 dark:bg-gray-800 px-1.5 py-0.2 rounded border border-gray-200 dark:border-gray-700">
                  Alt+{prov.num}
                </span>
              </div>
              <div class="text-[10px] text-gray-500 dark:text-gray-400 truncate mt-0.5">
                {prov.description}
              </div>
            </div>
          </button>
        {/each}
      </div>

      <!-- Footer -->
      <div class="px-5 py-3 border-t border-gray-200 dark:border-gray-800 bg-gray-50/80 dark:bg-gray-900/80 flex items-center justify-between text-xs text-gray-500">
        <div class="flex items-center gap-1.5">
          <kbd class="font-mono bg-gray-200 dark:bg-gray-800 px-1 rounded text-[10px]">Enter</kbd>
          <span>to launch primary portal</span>
          <span class="mx-1 text-gray-300 dark:text-gray-700">|</span>
          <kbd class="font-mono bg-gray-200 dark:bg-gray-800 px-1 rounded text-[10px]">Alt+1..9</kbd>
          <span>for direct portal jump</span>
        </div>
        <button
          type="button"
          on:click={() => dispatch('close')}
          class="px-3.5 py-1 text-xs font-semibold rounded bg-gray-200 dark:bg-gray-800 hover:bg-gray-300 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-300 cursor-pointer"
        >
          Close
        </button>
      </div>
    </div>
  </div>
{/if}
