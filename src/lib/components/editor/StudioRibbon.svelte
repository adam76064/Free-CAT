<script lang="ts">
  /**
   * StudioRibbon.svelte — Professional Trados-style Workflow Ribbon Bar
   * Organizes all translation engineering and review actions into clean, logical tabs:
   * [Home] [Review] [View] [Tools]
   * Supports collapsible/pinned states for maximum screen real estate.
   */
  import { createEventDispatcher } from 'svelte';
  import { settings } from '$lib/stores/settings';
  import { advancedFilter } from '$lib/stores/segments';

  export let isQuickInsertOpen = true;
  export let isResultsPaneOpen = true;
  export let isTtsPlaying = false;
  export let hasSelectedSegment = false;
  export let isSegmentLocked = false;
  export let hasAutoPatch = false;

  const dispatch = createEventDispatcher<{
    confirm: void;
    copySource: void;
    clearTarget: void;
    cycleCase: void;
    toggleLock: void;
    toggleQuickInsert: void;
    openQuickPlace: void;
    openFind: void;
    openReplace: void;
    pretranslate: void;
    autoPatch: void;
    toggleTrackChanges: void;
    openTqa: void;
    openQaDrawer: void;
    openNotesDrawer: void;
    exportDocx: void;
    importDocx: void;
    toggleAdvancedFilter: void;
    setLayout: 'side-by-side' | 'stacked' | 'zen';
    zoomIn: void;
    zoomOut: void;
    toggleOpticalBoost: void;
    toggleResultsPane: void;
    openConcordance: void;
    openWebLookup: void;
    openAutoText: void;
    openAte: void;
    openPseudoTranslate: void;
    openInvoice: void;
    toggleTts: void;
    openFlowPreview: void;
    batchCopySource: void;
    batchClearDraft: void;
    batchLockConfirmed: void;
    batchUnlockAll: void;
  }>();

  type RibbonTab = 'home' | 'review' | 'view' | 'tools';
  let activeTab: RibbonTab = 'home';
  let isPinned = true;
  let isCollapsed = false;

  function toggleCollapse() {
    isCollapsed = !isCollapsed;
  }
</script>

<div class="border-b border-gray-200 dark:border-gray-800 bg-white dark:bg-gray-900 select-none shadow-2xs shrink-0">
  <!-- Ribbon Tab Bar -->
  <div class="flex items-center justify-between px-3 h-8 bg-gray-50/90 dark:bg-gray-950/80 border-b border-gray-200/80 dark:border-gray-800">
    <div class="flex items-center gap-0.5">
      <button
        type="button"
        onclick={() => { activeTab = 'home'; if (isCollapsed) isCollapsed = false; }}
        class="px-3 py-1 text-xs font-semibold rounded-t-md transition-colors cursor-pointer
               {activeTab === 'home' && !isCollapsed
                 ? 'bg-white dark:bg-gray-900 text-brand-600 dark:text-brand-400 border-t-2 border-brand-500 shadow-2xs'
                 : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'}"
      >
        Home
      </button>

      <button
        type="button"
        onclick={() => { activeTab = 'review'; if (isCollapsed) isCollapsed = false; }}
        class="px-3 py-1 text-xs font-semibold rounded-t-md transition-colors cursor-pointer
               {activeTab === 'review' && !isCollapsed
                 ? 'bg-white dark:bg-gray-900 text-brand-600 dark:text-brand-400 border-t-2 border-brand-500 shadow-2xs'
                 : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'}"
      >
        Review
      </button>

      <button
        type="button"
        onclick={() => { activeTab = 'view'; if (isCollapsed) isCollapsed = false; }}
        class="px-3 py-1 text-xs font-semibold rounded-t-md transition-colors cursor-pointer
               {activeTab === 'view' && !isCollapsed
                 ? 'bg-white dark:bg-gray-900 text-brand-600 dark:text-brand-400 border-t-2 border-brand-500 shadow-2xs'
                 : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'}"
      >
        View
      </button>

      <button
        type="button"
        onclick={() => { activeTab = 'tools'; if (isCollapsed) isCollapsed = false; }}
        class="px-3 py-1 text-xs font-semibold rounded-t-md transition-colors cursor-pointer
               {activeTab === 'tools' && !isCollapsed
                 ? 'bg-white dark:bg-gray-900 text-brand-600 dark:text-brand-400 border-t-2 border-brand-500 shadow-2xs'
                 : 'text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'}"
      >
        Advanced / Tools
      </button>
    </div>

    <!-- Right: Ribbon Minimize / Pin button -->
    <div class="flex items-center gap-1.5">
      <button
        type="button"
        onclick={() => dispatch('toggleResultsPane')}
        class="px-2 py-0.5 text-[11px] rounded font-medium border border-gray-200 dark:border-gray-700 text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors flex items-center gap-1 cursor-pointer"
        title="Toggle Upper Translation Results & Term Recognition Pane"
      >
        <span>{isResultsPaneOpen ? '▲ Hide Results Pane' : '▼ Show Results Pane'}</span>
      </button>

      <button
        type="button"
        onclick={toggleCollapse}
        class="w-6 h-6 flex items-center justify-center rounded text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 hover:bg-gray-200 dark:hover:bg-gray-800 text-xs transition-transform cursor-pointer"
        title={isCollapsed ? 'Expand Ribbon' : 'Collapse Ribbon (Double-click or click)'}
      >
        {isCollapsed ? '▼' : '▲'}
      </button>
    </div>
  </div>

  <!-- Ribbon Body (Grouped Toolstrips) -->
  {#if !isCollapsed}
    <div class="px-3 py-1.5 flex items-center gap-4 overflow-x-auto bg-white dark:bg-gray-900 text-xs min-h-[58px]">
      {#if activeTab === 'home'}
        <!-- Group 1: Segment Confirmation & Navigation -->
        <div class="flex items-center gap-1.5 pr-3 border-r border-gray-200 dark:border-gray-800 shrink-0">
          <button
            type="button"
            onclick={() => dispatch('confirm')}
            disabled={!hasSelectedSegment}
            class="flex flex-col items-center justify-center px-3 py-1 rounded-lg bg-emerald-50 dark:bg-emerald-950/40 border border-emerald-300 dark:border-emerald-700 hover:bg-emerald-100 dark:hover:bg-emerald-900/60 disabled:opacity-40 disabled:cursor-not-allowed transition-colors cursor-pointer group"
            title="Confirm Segment & Add to TM (Ctrl+Enter)"
          >
            <span class="text-emerald-600 dark:text-emerald-400 text-base leading-none">✓</span>
            <span class="text-[10px] font-bold text-emerald-800 dark:text-emerald-200 mt-0.5">Confirm</span>
          </button>

          <div class="flex flex-col gap-1">
            <button
              type="button"
              onclick={() => dispatch('copySource')}
              class="px-2 py-0.5 rounded text-[11px] text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors flex items-center gap-1 cursor-pointer"
              title="Copy Source to Target (Ctrl+Shift+S)"
            >
              <span>📋 Copy Source</span>
              <kbd class="text-[9px] text-gray-400 font-mono">Ctrl+Shift+S</kbd>
            </button>
            <button
              type="button"
              onclick={() => dispatch('clearTarget')}
              class="px-2 py-0.5 rounded text-[11px] text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors flex items-center gap-1 cursor-pointer"
              title="Clear Target Segment (Alt+Del)"
            >
              <span>🗑 Clear Target</span>
              <kbd class="text-[9px] text-gray-400 font-mono">Alt+Del</kbd>
            </button>
          </div>
        </div>

        <!-- Group 2: Editing & Placeables -->
        <div class="flex items-center gap-1.5 pr-3 border-r border-gray-200 dark:border-gray-800 shrink-0">
          <div class="flex flex-col gap-1">
            <button
              type="button"
              onclick={() => dispatch('openQuickPlace')}
              class="px-2 py-0.5 rounded text-[11px] text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors flex items-center gap-1 cursor-pointer"
              title="QuickPlace Tags, Numbers, Dates (Ctrl+,)"
            >
              <span>🏷️ QuickPlace</span>
              <kbd class="text-[9px] text-gray-400 font-mono">Ctrl+,</kbd>
            </button>
            <button
              type="button"
              onclick={() => dispatch('toggleQuickInsert')}
              class="px-2 py-0.5 rounded text-[11px] transition-colors flex items-center gap-1 cursor-pointer
                     {isQuickInsertOpen ? 'bg-amber-100 text-amber-800 dark:bg-amber-950/60 dark:text-amber-200 font-bold' : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800'}"
              title="Toggle QuickInsert Typographic Bar (Ctrl+Shift+I)"
            >
              <span>⚡ QuickInsert Bar</span>
              <kbd class="text-[9px] text-gray-400 font-mono">Ctrl+Shift+I</kbd>
            </button>
          </div>

          <div class="flex flex-col gap-1">
            <button
              type="button"
              onclick={() => dispatch('cycleCase')}
              class="px-2 py-0.5 rounded text-[11px] text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors flex items-center gap-1 cursor-pointer"
              title="Cycle Case (Shift+F3)"
            >
              <span>Aa Cycle Case</span>
              <kbd class="text-[9px] text-gray-400 font-mono">Shift+F3</kbd>
            </button>
            <button
              type="button"
              onclick={() => dispatch('toggleLock')}
              class="px-2 py-0.5 rounded text-[11px] transition-colors flex items-center gap-1 cursor-pointer
                     {isSegmentLocked ? 'text-rose-600 font-bold bg-rose-50 dark:bg-rose-950/40' : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800'}"
              title="Lock/Unlock Segment (Ctrl+L)"
            >
              <span>{isSegmentLocked ? '🔒 Locked' : '🔓 Lock Segment'}</span>
              <kbd class="text-[9px] text-gray-400 font-mono">Ctrl+L</kbd>
            </button>
          </div>
        </div>

        <!-- Group 3: TM Automation & Repair -->
        <div class="flex items-center gap-1.5 pr-3 border-r border-gray-200 dark:border-gray-800 shrink-0">
          <button
            type="button"
            onclick={() => dispatch('pretranslate')}
            class="flex flex-col items-center justify-center px-2.5 py-1 rounded-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors cursor-pointer"
            title="Pre-translate 100% exact matches across document"
          >
            <span class="text-amber-500 text-sm leading-none">⚡</span>
            <span class="text-[10px] font-semibold text-gray-700 dark:text-gray-300 mt-0.5">Pre-Translate</span>
          </button>

          <button
            type="button"
            onclick={() => dispatch('autoPatch')}
            class="flex flex-col items-center justify-center px-2.5 py-1 rounded-lg border transition-colors cursor-pointer
                   {hasAutoPatch ? 'border-brand-500 bg-brand-50 dark:bg-brand-950/50 shadow-2xs font-semibold' : 'border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800'}"
            title="Auto-Patch fuzzy matches with FMR number/term substitution"
          >
            <span class="text-blue-500 text-sm leading-none">🔧</span>
            <span class="text-[10px] font-semibold text-gray-700 dark:text-gray-300 mt-0.5">Auto-Patch FMR</span>
          </button>
        </div>

        <!-- Group 4: Search & Navigation -->
        <div class="flex items-center gap-1.5 shrink-0">
          <button
            type="button"
            onclick={() => dispatch('openFind')}
            class="px-2 py-1 rounded-lg border border-gray-200 dark:border-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors flex items-center gap-1 cursor-pointer"
            title="Find & Replace in Document (Ctrl+F / Ctrl+H)"
          >
            <span>🔍 Find / Replace</span>
            <kbd class="text-[9px] text-gray-400 font-mono">Ctrl+H</kbd>
          </button>

          <button
            type="button"
            onclick={() => dispatch('openConcordance')}
            class="px-2 py-1 rounded-lg border border-gray-200 dark:border-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors flex items-center gap-1 cursor-pointer"
            title="TM Concordance Search (F3)"
          >
            <span>📖 Concordance</span>
            <kbd class="text-[9px] text-gray-400 font-mono">F3</kbd>
          </button>
        </div>

      {:else if activeTab === 'review'}
        <!-- Group 1: Track Changes -->
        <div class="flex items-center gap-2 pr-3 border-r border-gray-200 dark:border-gray-800 shrink-0">
          <button
            type="button"
            onclick={() => dispatch('toggleTrackChanges')}
            class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg border transition-colors cursor-pointer
                   {$settings.revisionMode ? 'bg-amber-100 text-amber-900 border-amber-300 dark:bg-amber-950/60 dark:text-amber-200 font-bold' : 'border-gray-200 dark:border-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50'}"
            title="Toggle Track Changes / Revision Mode"
          >
            <span class="text-sm">📝</span>
            <div class="text-left">
              <div class="text-[11px] font-bold">Track Changes</div>
              <div class="text-[9px] text-gray-500">{$settings.revisionMode ? 'ACTIVE (Recording diffs)' : 'Disabled'}</div>
            </div>
          </button>
        </div>

        <!-- Group 2: Quality & Verification -->
        <div class="flex items-center gap-1.5 pr-3 border-r border-gray-200 dark:border-gray-800 shrink-0">
          <button
            type="button"
            onclick={() => dispatch('openQaDrawer')}
            class="flex flex-col items-center justify-center px-3 py-1 rounded-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors cursor-pointer"
            title="Run QA Checker 3.0 Verification"
          >
            <span class="text-emerald-500 text-sm leading-none">✓</span>
            <span class="text-[10px] font-semibold text-gray-700 dark:text-gray-300 mt-0.5">QA Checker</span>
          </button>

          <button
            type="button"
            onclick={() => dispatch('openTqa')}
            class="flex flex-col items-center justify-center px-3 py-1 rounded-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors cursor-pointer"
            title="TAUS DQF-MQM Translation Quality Assessment Scorecard"
          >
            <span class="text-purple-500 text-sm leading-none">🛡️</span>
            <span class="text-[10px] font-semibold text-gray-700 dark:text-gray-300 mt-0.5">TQA Scorecard</span>
          </button>
        </div>

        <!-- Group 3: External Word (.docx) Review -->
        <div class="flex items-center gap-1.5 pr-3 border-r border-gray-200 dark:border-gray-800 shrink-0">
          <button
            type="button"
            onclick={() => dispatch('exportDocx')}
            class="px-2.5 py-1 rounded-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 text-gray-700 dark:text-gray-300 transition-colors flex items-center gap-1.5 cursor-pointer"
            title="Export 4-column Bilingual Word Table (.docx) for external reviewer"
          >
            <span class="text-blue-500">📄</span>
            <span>Export for Word Review</span>
          </button>

          <button
            type="button"
            onclick={() => dispatch('importDocx')}
            class="px-2.5 py-1 rounded-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 text-gray-700 dark:text-gray-300 transition-colors flex items-center gap-1.5 cursor-pointer"
            title="Import Review Retrofit (.docx) & update modified segments"
          >
            <span class="text-amber-500">📥</span>
            <span>Import Retrofit (.docx)</span>
          </button>
        </div>

        <!-- Group 4: Segment Notes & Comments -->
        <div class="flex items-center gap-1.5 shrink-0">
          <button
            type="button"
            onclick={() => dispatch('openNotesDrawer')}
            class="px-2.5 py-1 rounded-lg border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 text-gray-700 dark:text-gray-300 transition-colors flex items-center gap-1.5 cursor-pointer"
            title="View or Add Segment Note & Audit History"
          >
            <span class="text-blue-500">💬</span>
            <span>Segment Notes</span>
          </button>
        </div>

      {:else if activeTab === 'view'}
        <!-- Group 1: Workspace Layout Modes -->
        <div class="flex items-center gap-1 pr-3 border-r border-gray-200 dark:border-gray-800 shrink-0">
          <button
            type="button"
            onclick={() => dispatch('setLayout', 'side-by-side')}
            class="px-2.5 py-1 rounded-md text-[11px] font-medium transition-colors cursor-pointer
                   {$settings.layoutMode === 'side-by-side' ? 'bg-brand-500 text-white font-bold shadow-2xs' : 'text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800'}"
          >
            Side-by-Side
          </button>
          <button
            type="button"
            onclick={() => dispatch('setLayout', 'stacked')}
            class="px-2.5 py-1 rounded-md text-[11px] font-medium transition-colors cursor-pointer
                   {$settings.layoutMode === 'stacked' ? 'bg-brand-500 text-white font-bold shadow-2xs' : 'text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800'}"
          >
            Stacked
          </button>
          <button
            type="button"
            onclick={() => dispatch('setLayout', 'zen')}
            class="px-2.5 py-1 rounded-md text-[11px] font-medium transition-colors cursor-pointer
                   {$settings.layoutMode === 'zen' ? 'bg-purple-600 text-white font-bold shadow-2xs' : 'text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800'}"
          >
            Zen Focus
          </button>
        </div>

        <!-- Group 2: Display Filtering -->
        <div class="flex items-center gap-1.5 pr-3 border-r border-gray-200 dark:border-gray-800 shrink-0">
          <button
            type="button"
            onclick={() => dispatch('toggleAdvancedFilter')}
            class="px-2.5 py-1 rounded-lg border transition-colors flex items-center gap-1.5 cursor-pointer
                   {$advancedFilter.isActive ? 'bg-brand-500 text-white border-brand-600 font-bold' : 'border-gray-200 dark:border-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50'}"
            title="Trados Advanced Display Filter 2.0 Studio"
          >
            <span>🎯 Display Filter 2.0</span>
            {#if $advancedFilter.isActive}
              <span class="w-1.5 h-1.5 rounded-full bg-emerald-300 animate-ping"></span>
            {/if}
          </button>
        </div>

        <!-- Group 3: Font Zoom & Optical Boost -->
        <div class="flex items-center gap-1.5 pr-3 border-r border-gray-200 dark:border-gray-800 shrink-0">
          <span class="text-[10px] text-gray-400 font-medium">Font:</span>
          <button
            type="button"
            onclick={() => dispatch('zoomOut')}
            class="w-6 h-6 flex items-center justify-center rounded border border-gray-200 dark:border-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 font-bold cursor-pointer"
            title="Decrease font size"
          >-</button>
          <span class="font-mono text-xs px-1 font-semibold">{$settings.fontSize}px</span>
          <button
            type="button"
            onclick={() => dispatch('zoomIn')}
            class="w-6 h-6 flex items-center justify-center rounded border border-gray-200 dark:border-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 font-bold cursor-pointer"
            title="Increase font size"
          >+</button>

          <button
            type="button"
            onclick={() => dispatch('toggleOpticalBoost')}
            class="ml-1 px-2 py-0.5 rounded text-[10px] font-medium border transition-colors cursor-pointer
                   {$settings.targetOpticalBoost ? 'bg-blue-100 text-blue-800 border-blue-300 dark:bg-blue-900/50 dark:text-blue-200 font-bold' : 'border-gray-200 dark:border-gray-700 text-gray-500 hover:text-gray-700'}"
            title="Optical Font Boost (+2px for non-Latin / RTL scripts)"
          >
            Optical +2
          </button>
        </div>

        <!-- Group 4: Live Flow Preview -->
        <div class="flex items-center gap-1.5 shrink-0">
          <button
            type="button"
            onclick={() => dispatch('openFlowPreview')}
            class="px-2.5 py-1 rounded-lg border border-gray-200 dark:border-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors flex items-center gap-1.5 cursor-pointer"
            title="Toggle Live In-Context Flow Preview"
          >
            <span>👁 Flow Preview</span>
          </button>
        </div>

      {:else if activeTab === 'tools'}
        <!-- Group 1: Lookup & Reference -->
        <div class="flex items-center gap-1.5 pr-3 border-r border-gray-200 dark:border-gray-800 shrink-0">
          <button
            type="button"
            onclick={() => dispatch('openWebLookup')}
            class="px-2.5 py-1 rounded-lg border border-gray-200 dark:border-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors flex items-center gap-1.5 cursor-pointer"
            title="Web Lookup! Concordance across IATE, DeepL, Linguee, ProZ (Ctrl+F8 / Alt+W)"
          >
            <span class="text-cyan-500">🌐</span>
            <span>Web Lookup!</span>
            <kbd class="text-[9px] text-gray-400 font-mono">Alt+W</kbd>
          </button>

          <button
            type="button"
            onclick={() => dispatch('openAutoText')}
            class="px-2.5 py-1 rounded-lg border border-gray-200 dark:border-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors flex items-center gap-1.5 cursor-pointer"
            title="AutoText Shorthand & Abbreviation Dictionary Manager"
          >
            <span class="text-amber-500">⚡</span>
            <span>AutoText Shorthand</span>
          </button>
        </div>

        <!-- Group 2: Terminology & Simulation -->
        <div class="flex items-center gap-1.5 pr-3 border-r border-gray-200 dark:border-gray-800 shrink-0">
          <button
            type="button"
            onclick={() => dispatch('openAte')}
            class="px-2.5 py-1 rounded-lg border border-gray-200 dark:border-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors flex items-center gap-1.5 cursor-pointer"
            title="Automatic Terminology Extraction (C-Value)"
          >
            <span class="text-purple-500">🏷️</span>
            <span>Extract Terms (ATE)</span>
          </button>

          <button
            type="button"
            onclick={() => dispatch('openPseudoTranslate')}
            class="px-2.5 py-1 rounded-lg border border-gray-200 dark:border-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors flex items-center gap-1.5 cursor-pointer"
            title="Pre-flight Pseudo-Translation Simulation"
          >
            <span class="text-pink-500">🧪</span>
            <span>Pseudo-Translation</span>
          </button>

          <button
            type="button"
            onclick={() => dispatch('openInvoice')}
            class="px-2.5 py-1 rounded-lg border border-gray-200 dark:border-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors flex items-center gap-1.5 cursor-pointer"
            title="Quoting & Invoicing (Weighted Analysis Bands)"
          >
            <span class="text-emerald-500">💼</span>
            <span>Quoting & Billing</span>
          </button>
        </div>

        <!-- Group 3: Audio Proofreading & Batch Actions -->
        <div class="flex items-center gap-1.5 shrink-0">
          <button
            type="button"
            onclick={() => dispatch('toggleTts')}
            class="px-2.5 py-1 rounded-lg border transition-colors flex items-center gap-1.5 cursor-pointer
                   {isTtsPlaying ? 'bg-purple-600 text-white border-purple-700 animate-pulse font-bold' : 'border-gray-200 dark:border-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50'}"
            title="Acoustic Proofreading Speech Synthesis (Ctrl+Alt+R)"
          >
            <span>{isTtsPlaying ? '⏸ Stop Audio' : '🔊 Listen Proofread'}</span>
            <kbd class="text-[9px] text-gray-400 font-mono">Ctrl+Alt+R</kbd>
          </button>

          <button
            type="button"
            onclick={() => dispatch('batchCopySource')}
            class="px-2 py-1 rounded-lg border border-gray-200 dark:border-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 text-[11px] transition-colors cursor-pointer"
            title="Copy Source to all Empty Target Segments (Alt+Shift+Insert)"
          >
            📋 Pre-fill Empty Targets
          </button>

          <button
            type="button"
            onclick={() => dispatch('batchLockConfirmed')}
            class="px-2 py-1 rounded-lg border border-gray-200 dark:border-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-50 text-[11px] transition-colors cursor-pointer"
            title="Lock All Confirmed / 100% Segments"
          >
            🔒 Lock Confirmed
          </button>
        </div>
      {/if}
    </div>
  {/if}
</div>
