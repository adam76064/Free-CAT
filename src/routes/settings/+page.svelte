<script lang="ts">
  // Settings page — user preferences.
  import { onMount } from 'svelte';
  import { settings } from '$lib/stores/settings';
  import { setSecret, hasSecret, type SecretName } from '$lib/ipc/secrets';

  let s = $state({ ...$settings });

  // API keys live in the OS keychain and are write-only from the UI:
  // we can store or remove one, and see whether one is set, but never read it back.
  const KEY_FIELDS: { name: SecretName; label: string; placeholder: string; hint: string }[] = [
    { name: 'deepl', label: 'DeepL API Key', placeholder: 'xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx:fx', hint: 'Free tier: api-free.deepl.com (key ends in :fx)' },
    { name: 'google_translate', label: 'Google Translate API Key', placeholder: 'AIza…', hint: 'Cloud Translation API v2 key' },
    { name: 'openai', label: 'OpenAI API Key', placeholder: 'sk-…', hint: 'Used by the AI panel when the OpenAI provider is selected' },
  ];
  let keyStored = $state<Record<string, boolean>>({ deepl: false, google_translate: false, openai: false });
  let keyDraft = $state<Record<string, string>>({ deepl: '', google_translate: '', openai: '' });
  let keyError = $state<string | null>(null);

  onMount(async () => {
    for (const { name } of KEY_FIELDS) {
      try { keyStored[name] = await hasSecret(name); } catch { /* keychain unavailable */ }
    }
  });

  async function removeKey(name: SecretName) {
    keyError = null;
    try {
      await setSecret(name, '');
      keyStored[name] = false;
    } catch (e) {
      keyError = String(e);
    }
  }

  async function save() {
    settings.set(s);
    keyError = null;
    for (const { name } of KEY_FIELDS) {
      const value = keyDraft[name]?.trim();
      if (!value) continue;
      try {
        await setSecret(name, value);
        keyStored[name] = true;
        keyDraft[name] = '';
      } catch (e) {
        keyError = String(e);
      }
    }
  }
</script>

<div class="h-full overflow-y-auto">
  <div class="max-w-2xl mx-auto px-6 py-8">
    <h1 class="text-xl font-semibold text-gray-900 dark:text-white mb-6">Settings</h1>

    <!-- Appearance -->
    <section class="mb-8">
      <h2 class="text-sm font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-4">Appearance</h2>
      <div class="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 divide-y divide-gray-100 dark:divide-gray-800">
        <div class="flex items-center justify-between px-4 py-3">
          <div>
            <div class="text-sm font-medium text-gray-900 dark:text-white">Theme</div>
            <div class="text-xs text-gray-500">Light, dark, or follow system</div>
          </div>
          <select bind:value={s.theme} class="text-sm px-2 py-1.5 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:outline-none focus:ring-1 focus:ring-brand-500">
            <option value="system">System</option>
            <option value="light">Light</option>
            <option value="dark">Dark</option>
          </select>
        </div>
        <div class="flex items-center justify-between px-4 py-3">
          <div>
            <div class="text-sm font-medium text-gray-900 dark:text-white">Editor Font Size</div>
            <div class="text-xs text-gray-500">Segment text size in pixels</div>
          </div>
          <input type="number" bind:value={s.fontSize} min="11" max="24" class="w-16 text-sm px-2 py-1.5 text-center rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:outline-none focus:ring-1 focus:ring-brand-500" />
        </div>
      </div>
    </section>

    <!-- Translation -->
    <section class="mb-8">
      <h2 class="text-sm font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-4">Translation</h2>
      <div class="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 divide-y divide-gray-100 dark:divide-gray-800">
        <div class="flex items-center justify-between px-4 py-3">
          <div>
            <div class="text-sm font-medium text-gray-900 dark:text-white">Default Fuzzy Threshold</div>
            <div class="text-xs text-gray-500">Minimum TM match to show (0–100%)</div>
          </div>
          <div class="flex items-center gap-2">
            <input type="range" bind:value={s.defaultFuzzyThreshold} min="0.5" max="1.0" step="0.05" class="w-24" />
            <span class="text-sm font-mono text-gray-700 dark:text-gray-300 w-10 text-right">
              {Math.round(s.defaultFuzzyThreshold * 100)}%
            </span>
          </div>
        </div>
        <div class="flex items-center justify-between px-4 py-3">
          <div>
            <div class="text-sm font-medium text-gray-900 dark:text-white">Auto-save Interval</div>
            <div class="text-xs text-gray-500">Milliseconds between auto-saves</div>
          </div>
          <input type="number" bind:value={s.autoSaveIntervalMs} min="500" max="10000" step="500" class="w-20 text-sm px-2 py-1.5 text-center rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:outline-none focus:ring-1 focus:ring-brand-500" />
        </div>
      </div>
    </section>

    <!-- Machine Translation -->
    <section class="mb-8">
      <h2 class="text-sm font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-4">Machine Translation APIs</h2>
      <div class="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 divide-y divide-gray-100 dark:divide-gray-800">
        {#each KEY_FIELDS as key (key.name)}
          <div class="px-4 py-3">
            <div class="flex items-center justify-between mb-1">
              <label for="key-{key.name}" class="block text-sm font-medium text-gray-900 dark:text-white">{key.label}</label>
              {#if keyStored[key.name]}
                <span class="flex items-center gap-2 text-xs text-green-600 dark:text-green-400">
                  Stored in system keychain
                  <button type="button" onclick={() => removeKey(key.name)} class="underline text-gray-500 hover:text-red-500">Remove</button>
                </span>
              {/if}
            </div>
            <input
              id="key-{key.name}"
              bind:value={keyDraft[key.name]}
              type="password"
              autocomplete="off"
              placeholder={keyStored[key.name] ? '•••••••• (enter a new key to replace)' : key.placeholder}
              class="w-full px-3 py-1.5 text-sm rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:outline-none focus:ring-1 focus:ring-brand-500"
            />
            <p class="text-xs text-gray-400 mt-1">{key.hint}</p>
          </div>
        {/each}
        {#if keyError}
          <div class="px-4 py-2 text-xs text-red-500">{keyError}</div>
        {/if}
        <div class="px-4 py-3">
          <label for="libre-url" class="block text-sm font-medium text-gray-900 dark:text-white mb-1">LibreTranslate URL</label>
          <input id="libre-url" bind:value={s.libretranslate_url} placeholder="http://localhost:5000" class="w-full px-3 py-1.5 text-sm rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:outline-none focus:ring-1 focus:ring-brand-500" />
          <p class="text-xs text-gray-400 mt-1">Self-hosted LibreTranslate instance URL</p>
        </div>
        <div class="px-4 py-3">
          <label for="ollama-url" class="block text-sm font-medium text-gray-900 dark:text-white mb-1">Ollama URL</label>
          <input id="ollama-url" bind:value={s.ollama_url} placeholder="http://localhost:11434" class="w-full px-3 py-1.5 text-sm rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:outline-none focus:ring-1 focus:ring-brand-500" />
          <p class="text-xs text-gray-400 mt-1">Local Ollama for AI-assisted translation (Phase 2)</p>
        </div>
      </div>
    </section>

    <!-- Quality Assurance -->
    <section class="mb-8">
      <h2 class="text-sm font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-4">Quality Assurance</h2>
      <div class="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 divide-y divide-gray-100 dark:divide-gray-800">
        <div class="flex items-center justify-between px-4 py-3">
          <div>
            <div class="text-sm font-medium text-gray-900 dark:text-white">Auto-run QA on Confirm</div>
            <div class="text-xs text-gray-500">Run QA checks automatically each time a segment is confirmed</div>
          </div>
          <input type="checkbox" bind:checked={s.autoRunQa} class="rounded accent-brand-500 w-4 h-4" />
        </div>
        <div class="flex items-center justify-between px-4 py-3">
          <div>
            <div class="text-sm font-medium text-gray-900 dark:text-white">Check Numbers</div>
            <div class="text-xs text-gray-500">Flag mismatched numbers between source and target</div>
          </div>
          <input type="checkbox" bind:checked={s.qaCheckNumbers} class="rounded accent-brand-500 w-4 h-4" />
        </div>
        <div class="flex items-center justify-between px-4 py-3">
          <div>
            <div class="text-sm font-medium text-gray-900 dark:text-white">Check Punctuation</div>
            <div class="text-xs text-gray-500">Flag missing final punctuation (period, question, exclamation)</div>
          </div>
          <input type="checkbox" bind:checked={s.qaCheckPunctuation} class="rounded accent-brand-500 w-4 h-4" />
        </div>
        <div class="flex items-center justify-between px-4 py-3">
          <div>
            <div class="text-sm font-medium text-gray-900 dark:text-white">Check Double Spaces</div>
            <div class="text-xs text-gray-500">Flag double or leading/trailing spaces in target</div>
          </div>
          <input type="checkbox" bind:checked={s.qaCheckDoubleSpaces} class="rounded accent-brand-500 w-4 h-4" />
        </div>
        <div class="flex items-center justify-between px-4 py-3">
          <div>
            <div class="text-sm font-medium text-gray-900 dark:text-white">Check Tag Integrity</div>
            <div class="text-xs text-gray-500">Flag missing or extra inline tags {'{1}'}, {'{/1}'} etc.</div>
          </div>
          <input type="checkbox" bind:checked={s.qaCheckTags} class="rounded accent-brand-500 w-4 h-4" />
        </div>
      </div>
    </section>

    <!-- Translation Workflow -->
    <section class="mb-8">
      <h2 class="text-sm font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-4">Translation Workflow</h2>
      <div class="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 divide-y divide-gray-100 dark:divide-gray-800">
        <div class="flex items-center justify-between px-4 py-3">
          <div>
            <div class="text-sm font-medium text-gray-900 dark:text-white">Auto-Propagate Confirmed Segments</div>
            <div class="text-xs text-gray-500">Automatically fill identical source segments when one is confirmed</div>
          </div>
          <input type="checkbox" bind:checked={s.autoPropagate} class="rounded accent-brand-500 w-4 h-4" />
        </div>
        <div class="flex items-center justify-between px-4 py-3">
          <div>
            <div class="text-sm font-medium text-gray-900 dark:text-white">Revision Mode (Track Changes)</div>
            <div class="text-xs text-gray-500">Show redline diff of edits against original confirmed target text</div>
          </div>
          <input type="checkbox" bind:checked={s.revisionMode} class="rounded accent-brand-500 w-4 h-4" />
        </div>
        <div class="flex items-center justify-between px-4 py-3">
          <div>
            <div class="text-sm font-medium text-gray-900 dark:text-white">Optical Font Boost for RTL/CJK</div>
            <div class="text-xs text-gray-500">Add +2px to target font size for Arabic, Hebrew, CJK scripts</div>
          </div>
          <input type="checkbox" bind:checked={s.targetOpticalBoost} class="rounded accent-brand-500 w-4 h-4" />
        </div>
        <div class="flex items-center justify-between px-4 py-3">
          <div>
            <div class="text-sm font-medium text-gray-900 dark:text-white">Spellcheck / Linter</div>
            <div class="text-xs text-gray-500">Enable native OS spellcheck on target segments</div>
          </div>
          <input type="checkbox" bind:checked={s.spellcheckEnabled} class="rounded accent-brand-500 w-4 h-4" />
        </div>
      </div>
    </section>

    <!-- Keyboard Shortcuts Reference -->
    <section class="mb-8">
      <h2 class="text-sm font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider mb-4">Keyboard Shortcuts</h2>
      <div class="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 overflow-hidden">
        <table class="w-full text-xs">
          <thead class="bg-gray-50 dark:bg-gray-800/80 text-gray-500 dark:text-gray-400 uppercase font-semibold">
            <tr>
              <th class="py-2.5 px-4 text-left">Action</th>
              <th class="py-2.5 px-4 text-right">Shortcut</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-gray-100 dark:divide-gray-800 text-gray-700 dark:text-gray-300">
            {#each [
              ['Confirm Segment & Advance', 'Tab / Enter'],
              ['Find in File', 'Ctrl+F'],
              ['Find & Replace', 'Ctrl+H'],
              ['Copy Source to Target', 'Ctrl+Shift+S'],
              ['Clear Target', 'Alt+Delete'],
              ['Cycle Case (lower/UPPER/Title)', 'Shift+F3'],
              ['Toggle Segment Lock', 'Ctrl+L'],
              ['Toggle Bookmark', 'Ctrl+B'],
              ['Merge with Next Segment', 'Ctrl+J'],
              ['Merge with Previous Segment', 'Ctrl+Shift+J'],
              ['Split Segment', 'Ctrl+Alt+S'],
              ['QuickPlace Tag HUD', 'Ctrl+,'],
              ['Edit Source Text', 'Alt+F2'],
              ['Accept Revision', 'Ctrl+Shift+A'],
              ['Reject Revision', 'Ctrl+Shift+R'],
              ['Command Palette', 'Ctrl+K'],
              ['Pre-Translate File', 'Ctrl+F8'],
              ['Run QA Check', 'Ctrl+Alt+R'],
              ['Filter: Show All Segments', 'Ctrl+\\'],
              ['Focus Filter Bar', 'F6'],
            ] as [action, shortcut]}
              <tr class="hover:bg-gray-50/50 dark:hover:bg-gray-800/30">
                <td class="py-2 px-4">{action}</td>
                <td class="py-2 px-4 text-right">
                  <kbd class="font-mono text-[11px] bg-gray-100 dark:bg-gray-800 px-1.5 py-0.5 rounded border border-gray-200 dark:border-gray-700">{shortcut}</kbd>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </section>

    <!-- Save button -->
    <button
      onclick={save}
      class="w-full px-4 py-2.5 bg-brand-500 hover:bg-brand-600 text-white font-medium rounded-lg transition-colors"
    >
      Save Settings
    </button>
  </div>
</div>
