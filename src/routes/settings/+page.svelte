<script lang="ts">
  // Settings page — user preferences.
  import { settings } from '$lib/stores/settings';

  let s = $state({ ...$settings });

  function save() {
    settings.set(s);
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
        <div class="px-4 py-3">
          <label for="deepl-key" class="block text-sm font-medium text-gray-900 dark:text-white mb-1">DeepL API Key</label>
          <input id="deepl-key" bind:value={s.deepl_api_key} type="password" placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx:fx" class="w-full px-3 py-1.5 text-sm rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:outline-none focus:ring-1 focus:ring-brand-500" />
          <p class="text-xs text-gray-400 mt-1">Free tier: api-free.deepl.com (key ends in :fx)</p>
        </div>
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

    <!-- Save button -->
    <button
      on:click={save}
      class="w-full px-4 py-2.5 bg-brand-500 hover:bg-brand-600 text-white font-medium rounded-lg transition-colors"
    >
      Save Settings
    </button>
  </div>
</div>
