<script lang="ts">
  /**
   * SegmentGrid — the main bilingual editor grid.
   * Handles TM lookup & Termbase lookup on selection change, auto-save,
   * segment confirmation with auto-propagation, keyboard navigation,
   * filter/search, pre-translation, find & replace, split/merge, and tabbed right panel.
   */
  import { tick, onMount, onDestroy } from 'svelte';
  import {
    segments, selectedIndex, filteredSegments, selectedSegment,
    filterMode, segmentSearch, updateSegmentInStore,
    progressStats, type FilterMode, type Segment
  } from '$lib/stores/segments';
  import { tmMatches, tmLoading } from '$lib/stores/tm';
  import { termMatches, tbLoading } from '$lib/stores/termbase';
  import { currentProject, currentFile, currentFiles, projectSettings } from '$lib/stores/project';
  import { qaResults, qaErrors } from '$lib/stores/qa';
  import SegmentRow from './SegmentRow.svelte';
  import FindReplaceBar from './FindReplaceBar.svelte';
  import TmPanel from '$lib/components/panels/TmPanel.svelte';
  import TbPanel from '$lib/components/panels/TbPanel.svelte';
  import QaPanel from '$lib/components/panels/QaPanel.svelte';
  import AiPanel from '$lib/components/panels/AiPanel.svelte';
  import CommentsPanel from '$lib/components/panels/CommentsPanel.svelte';
  import LivePreviewPanel from '$lib/components/panels/LivePreviewPanel.svelte';
  import TermExtractionModal from '$lib/components/panels/TermExtractionModal.svelte';
  import QuoteInvoiceModal from '$lib/components/analysis/QuoteInvoiceModal.svelte';
  import AdvancedDisplayFilterPanel from './AdvancedDisplayFilterPanel.svelte';
  import ConcordanceModal from './ConcordanceModal.svelte';
  import CommandPalette from './CommandPalette.svelte';
  import QuickInsertBar from './QuickInsertBar.svelte';
  import PseudoTranslateModal from './PseudoTranslateModal.svelte';
  import TqaDrawer from './TqaDrawer.svelte';
  import AutoTextManagerModal from './AutoTextManagerModal.svelte';
  import WebLookupModal from './WebLookupModal.svelte';
  import StudioRibbon from './StudioRibbon.svelte';
  import StudioResultsPane from './StudioResultsPane.svelte';
  import { autoTextEntries } from '$lib/stores/autotext';
  import { SuggestionTrie, type SuggestionItem } from '$lib/utils/suggestTrie';
  import { queryTm, addToTm } from '$lib/ipc/tm';
  import { queryTb } from '$lib/ipc/termbase';
  import { updateSegment, confirmSegment, getSegments, splitSegment, mergeSegments, toggleBookmark, toggleSegmentLock, editSourceSegment } from '$lib/ipc/segment';
  import { settings } from '$lib/stores/settings';
  import { advancedFilter } from '$lib/stores/segments';
  import { runQa, preTranslate, exportBilingualReview, importBilingualReview } from '$lib/ipc/project';
  import { open } from '@tauri-apps/plugin-dialog';
  import { queryAiAssistant } from '$lib/ipc/ai';

  // Auto-save debounce handle
  let autoSaveTimer: ReturnType<typeof setTimeout>;
  let gridContainer: HTMLDivElement;
  let isResultsPaneOpen = true;
  let resultsPaneHeight = 150;
  let isDrawerOpen = false;
  let activeDrawerTab: 'qa' | 'comments' | 'ai' | 'preview' = 'qa';
  let isCommandPaletteOpen = false;
  let isTermExtractionOpen = false;
  let isQuoteInvoiceOpen = false;
  let isAdvancedFilterOpen = false;
  let isConcordanceOpen = false;
  let isQuickInsertOpen = true;
  let isPseudoTranslateOpen = false;
  let isTqaOpen = false;
  let isAutoTextOpen = false;
  let isWebLookupOpen = false;
  let webLookupQuery = '';
  let isTtsPlaying = false;
  let isTtsContinuous = false;
  let ttsRate = 1.0;
  let focusIsOnSource = false;
  let tqaDrawerComponent: TqaDrawer;
  let concordanceInitialQuery = '';
  let pretranslating = false;
  let isToolsMenuOpen = false;
  let activeAutoPatch: string | null = null;

  // Session velocity tracking
  let sessionStartTime = Date.now();
  let sessionConfirmedWords = 0;
  $: sessionElapsedHours = Math.max(0.005, (Date.now() - sessionStartTime) / (1000 * 60 * 60));
  $: sessionVelocityWordsPerHour = Math.round(sessionConfirmedWords / sessionElapsedHours);

  // Active segment word/char metrics
  $: activeSegSrcWords = $selectedSegment ? $selectedSegment.source_text.trim().split(/\s+/).filter(Boolean).length : 0;
  $: activeSegSrcChars = $selectedSegment ? $selectedSegment.source_text.length : 0;
  $: activeSegTgtWords = $selectedSegment ? $selectedSegment.target_text.trim().split(/\s+/).filter(Boolean).length : 0;

  // Draggable splitters state
  let isResizingSource = false;
  let isResizingInspector = false;
  let startX = 0;
  let startRatio = 50;
  let startInspectorWidth = 340;

  function handleStartSourceResize(e: PointerEvent) {
    e.preventDefault();
    isResizingSource = true;
    startX = e.clientX;
    startRatio = $settings.sourceRatio;
    window.addEventListener('pointermove', handlePointerMove);
    window.addEventListener('pointerup', handlePointerUp);
  }

  function handleStartInspectorResize(e: PointerEvent) {
    e.preventDefault();
    isResizingInspector = true;
    startX = e.clientX;
    startInspectorWidth = $settings.inspectorWidth;
    window.addEventListener('pointermove', handlePointerMove);
    window.addEventListener('pointerup', handlePointerUp);
  }

  function handlePointerMove(e: PointerEvent) {
    if (isResizingSource && gridContainer) {
      const gridRect = gridContainer.getBoundingClientRect();
      const availableWidth = gridRect.width - 44; // minus seq column
      const deltaX = e.clientX - startX;
      const newRatio = Math.round(startRatio + (deltaX / availableWidth) * 100);
      const clamped = Math.max(25, Math.min(75, newRatio));
      settings.update(s => ({ ...s, sourceRatio: clamped }));
    } else if (isResizingInspector) {
      const deltaX = startX - e.clientX;
      const newWidth = Math.max(260, Math.min(650, startInspectorWidth + deltaX));
      settings.update(s => ({ ...s, inspectorWidth: newWidth }));
    }
  }

  function handlePointerUp() {
    isResizingSource = false;
    isResizingInspector = false;
    window.removeEventListener('pointermove', handlePointerMove);
    window.removeEventListener('pointerup', handlePointerUp);
  }

  async function handleAiRefineAction(action: string, selectedText: string) {
    const seg = $selectedSegment;
    if (!seg || !$currentProject) return;
    try {
      const res = await queryAiAssistant({
        prompt: `Please ${action} this text: "${selectedText}"`,
        source_text: seg.source_text,
        target_text: seg.target_text,
        src_lang: $currentProject.src_lang,
        tgt_lang: $currentProject.tgt_lang,
        action: action === 'rephrase' ? 'rephrase' : 'ask',
        config: {
          provider: $settings.ollama_url ? 'ollama' : 'openai',
          endpoint: $settings.ollama_url || 'http://localhost:11434',
          model: 'llama3.2',
          api_key: undefined,
        }
      });
      if (res && res.reply) {
        let cleaned = res.reply.trim();
        if (cleaned.startsWith('"') && cleaned.endsWith('"')) {
          cleaned = cleaned.slice(1, -1);
        }
        const newTarget = seg.target_text.replace(selectedText, cleaned);
        handleTargetChange(seg.id, newTarget);
      }
    } catch (e) {
      console.warn('AI Refine failed:', e);
    }
  }

  $: documentText = $segments.map(s => s.source_text).join('. ');

  // In-memory Trie for AutoSuggest
  const suggestTrie = new SuggestionTrie();
  let activeSuggestions: SuggestionItem[] = [];

  $: {
    suggestTrie.clear();
    for (const m of $termMatches) {
      if (m.term) {
        suggestTrie.insert(m.term, {
          text: m.term,
          source: 'termbase',
          matchScore: 100,
          displayText: m.definition ? `${m.term} (${m.definition})` : m.term,
        });
      }
    }
    for (const tm of $tmMatches) {
      if (tm.target_text) {
        const words = tm.target_text.split(/\s+/).filter(w => w.length > 2);
        for (const w of words) {
          suggestTrie.insert(w, {
            text: w,
            source: 'tm',
            matchScore: tm.score_pct,
          });
        }
      }
    }
    for (const at of $autoTextEntries) {
      suggestTrie.insert(at.trigger, {
        text: at.expansion,
        source: 'autotext',
        matchScore: 100,
        displayText: `${at.trigger} ➔ ${at.expansion}`,
      });
    }
  }

  function handleQuerySuggestions(word: string) {
    activeSuggestions = suggestTrie.searchPrefix(word, 5);
  }

  // Find & Replace state
  let isFindOpen = false;
  let isReplaceOpen = false;

  // Split Segment modal state
  let segmentToSplit: Segment | null = null;
  let splitSliderPos = 1;

  // Text-to-Speech (TTS) Acoustic Proofreading
  function stopTts() {
    if (typeof window !== 'undefined' && 'speechSynthesis' in window) {
      window.speechSynthesis.cancel();
    }
    isTtsPlaying = false;
  }

  function speakText(text: string, lang: string, onEnd?: () => void) {
    if (typeof window === 'undefined' || !('speechSynthesis' in window)) return;
    window.speechSynthesis.cancel();
    if (!text.trim()) {
      isTtsPlaying = false;
      return;
    }

    const utter = new SpeechSynthesisUtterance(text);
    utter.rate = ttsRate;
    utter.lang = lang || 'en';

    try {
      const voices = window.speechSynthesis.getVoices();
      const matched = voices.find(v => v.lang.toLowerCase().startsWith(lang.toLowerCase().slice(0, 2)));
      if (matched) utter.voice = matched;
    } catch (e) {
      /* ignore */
    }

    utter.onend = () => {
      isTtsPlaying = false;
      if (onEnd) onEnd();
    };
    utter.onerror = () => {
      isTtsPlaying = false;
    };

    isTtsPlaying = true;
    window.speechSynthesis.speak(utter);
  }

  function toggleTtsPlayback() {
    if (isTtsPlaying) {
      stopTts();
      return;
    }
    playCurrentSegmentTts();
  }

  function playCurrentSegmentTts() {
    const seg = $selectedSegment;
    if (!seg) return;
    const textToRead = seg.target_text.trim() || seg.source_text.trim();
    const lang = seg.target_text.trim() ? ($currentProject?.tgt_lang || 'en') : ($currentProject?.src_lang || 'en');

    speakText(textToRead, lang, () => {
      if (isTtsContinuous && $selectedIndex < $filteredSegments.length - 1) {
        selectedIndex.update(i => i + 1);
        scrollToSelected();
        setTimeout(() => {
          if (isTtsContinuous) playCurrentSegmentTts();
        }, 300);
      }
    });
  }

  // Bookmark Jump Navigation (F2 / Shift+F2)
  function jumpToNextBookmark() {
    const list = $filteredSegments;
    if (list.length === 0) return;
    const startIdx = $selectedIndex;
    for (let offset = 1; offset <= list.length; offset++) {
      const idx = (startIdx + offset) % list.length;
      if (list[idx].bookmarked) {
        selectedIndex.set(idx);
        scrollToSelected();
        return;
      }
    }
  }

  function jumpToPrevBookmark() {
    const list = $filteredSegments;
    if (list.length === 0) return;
    const startIdx = $selectedIndex;
    for (let offset = 1; offset <= list.length; offset++) {
      const idx = (startIdx - offset + list.length) % list.length;
      if (list[idx].bookmarked) {
        selectedIndex.set(idx);
        scrollToSelected();
        return;
      }
    }
  }

  // Toggle Source / Target Focus (F6)
  function toggleSourceTargetFocus() {
    focusIsOnSource = !focusIsOnSource;
    const activeRowEl = gridContainer?.querySelector('[aria-selected="true"]');
    if (!activeRowEl) return;
    if (focusIsOnSource) {
      const editSourceBtn = activeRowEl.querySelector('button[title*="Edit Source"]') as HTMLButtonElement;
      editSourceBtn?.click();
    } else {
      const targetBox = activeRowEl.querySelector('[role="textbox"]') as HTMLElement;
      targetBox?.focus();
    }
  }

  // Batch Segment Operations
  async function handleBatchCopySourceToEmptyTarget() {
    if (!$currentProject || !$currentFile) return;
    let count = 0;
    for (const seg of $segments) {
      if (!seg.target_text.trim() && !seg.locked) {
        updateSegmentInStore(seg.id, {
          target_text: seg.source_text,
          status: 'draft',
        });
        await updateSegment({
          segment_id: seg.id,
          target_text: seg.source_text,
          status: 'draft',
        });
        count++;
      }
    }
    alert(`Batch Copy Source: ${count} empty segments filled with source text.`);
  }

  async function handleBatchClearDraftTargets() {
    if (!$currentProject || !$currentFile) return;
    let count = 0;
    for (const seg of $segments) {
      if (seg.status === 'draft' && !seg.locked) {
        updateSegmentInStore(seg.id, {
          target_text: '',
          status: 'new',
        });
        await updateSegment({
          segment_id: seg.id,
          target_text: '',
          status: 'new',
        });
        count++;
      }
    }
    alert(`Batch Clear: ${count} draft segments cleared.`);
  }

  async function handleBatchLockConfirmed() {
    if (!$currentProject || !$currentFile) return;
    let count = 0;
    for (const seg of $segments) {
      if ((seg.status === 'translated' || seg.match_percent === 100) && !seg.locked) {
        await toggleSegmentLock(seg.id);
        updateSegmentInStore(seg.id, { locked: true });
        count++;
      }
    }
    alert(`Batch Lock: ${count} confirmed/100% segments locked.`);
  }

  async function handleBatchUnlockAll() {
    if (!$currentProject || !$currentFile) return;
    let count = 0;
    for (const seg of $segments) {
      if (seg.locked) {
        await toggleSegmentLock(seg.id);
        updateSegmentInStore(seg.id, { locked: false });
        count++;
      }
    }
    alert(`Batch Unlock: ${count} segments unlocked.`);
  }

  // Global keyboard shortcuts (Ctrl+F, Ctrl+H, Ctrl+\, Ctrl+K, Ctrl+L, F2, F3, F6, Ctrl+F8, Ctrl+Alt+R)
  function handleWindowKeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'f' && !e.shiftKey) {
      e.preventDefault();
      isFindOpen = true;
      isReplaceOpen = false;
    } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'h') {
      e.preventDefault();
      isFindOpen = true;
      isReplaceOpen = true;
    } else if ((e.ctrlKey || e.metaKey) && e.key === '\\') {
      e.preventDefault();
      isDrawerOpen = !isDrawerOpen;
    } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
      e.preventDefault();
      isCommandPaletteOpen = !isCommandPaletteOpen;
    } else if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'l' && !e.shiftKey) {
      e.preventDefault();
      settings.update(s => {
        const next = s.layoutMode === 'side-by-side' ? 'stacked' : s.layoutMode === 'stacked' ? 'zen' : 'side-by-side';
        return { ...s, layoutMode: next };
      });
    } else if (e.key === 'F3') {
      e.preventDefault();
      const sel = window.getSelection()?.toString().trim() || '';
      concordanceInitialQuery = sel || ($selectedSegment?.source_text.slice(0, 60) || '');
      isConcordanceOpen = true;
    } else if ((e.ctrlKey && e.key === 'F8') || (e.altKey && e.key.toLowerCase() === 'w')) {
      e.preventDefault();
      const sel = window.getSelection()?.toString().trim() || '';
      webLookupQuery = sel || ($selectedSegment?.source_text || '');
      isWebLookupOpen = true;
    } else if (e.key === 'F2' && !e.shiftKey && !e.altKey) {
      e.preventDefault();
      jumpToNextBookmark();
    } else if (e.key === 'F2' && e.shiftKey) {
      e.preventDefault();
      jumpToPrevBookmark();
    } else if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key.toLowerCase() === 'b') {
      e.preventDefault();
      if ($selectedSegment) handleToggleBookmark($selectedSegment.id);
    } else if ((e.ctrlKey || e.metaKey) && e.altKey && e.key.toLowerCase() === 'r') {
      e.preventDefault();
      toggleTtsPlayback();
    } else if (e.key === 'F6') {
      e.preventDefault();
      toggleSourceTargetFocus();
    } else if (e.altKey && e.shiftKey && (e.key === 'Insert' || e.code === 'Insert')) {
      e.preventDefault();
      handleBatchCopySourceToEmptyTarget();
    } else if (e.altKey && e.shiftKey && (e.key === 'Delete' || e.code === 'Delete')) {
      e.preventDefault();
      handleBatchClearDraftTargets();
    }
  }

  function handleInsertConcordance(targetText: string) {
    if (!$selectedSegment || $selectedSegment.locked) return;
    const current = $selectedSegment.target_text || '';
    const newText = current ? `${current} ${targetText}` : targetText;
    handleTargetChange($selectedSegment.id, newText);
  }

  onMount(() => {
    window.addEventListener('keydown', handleWindowKeydown);
  });

  onDestroy(() => {
    stopTts();
    window.removeEventListener('keydown', handleWindowKeydown);
    window.removeEventListener('pointermove', handlePointerMove);
    window.removeEventListener('pointerup', handlePointerUp);
  });

  // TM and TB lookup whenever selected segment changes
  $: if ($selectedSegment && $currentProject) {
    lookupTm($selectedSegment.source_text);
    if ($projectSettings.tb_ids && $projectSettings.tb_ids.length > 0) {
      lookupTb($selectedSegment.source_text);
    }
  }

  async function lookupTm(sourceText: string) {
    if (!$currentProject) return;
    activeAutoPatch = null;
    tmLoading.set(true);
    try {
      const matches = await queryTm({
        source_text: sourceText,
        src_lang: $currentProject.src_lang,
        tgt_lang: $currentProject.tgt_lang,
        tm_ids: $projectSettings.tm_ids || [],
        threshold: $projectSettings.fuzzy_threshold || 0.7,
      });
      tmMatches.set(matches);

      // Auto-detect FMR repaired suggestion from top match
      if (matches.length > 0 && matches[0].repaired_target && matches[0].repaired_target.trim()) {
        activeAutoPatch = matches[0].repaired_target;
      }
    } catch (e) {
      console.error('TM lookup failed:', e);
      tmMatches.set([]);
    } finally {
      tmLoading.set(false);
    }
  }

  function handleApplyAutoPatch(patchText: string) {
    const seg = $selectedSegment;
    if (!seg) return;
    updateSegmentInStore(seg.id, { target_text: patchText, status: 'draft' });
    scheduleAutoSave(seg.id, patchText, 'draft');
    activeAutoPatch = null;
  }

  async function handleAutoPatchAll() {
    if (!$currentProject || !$currentFile) return;
    let patchedCount = 0;
    for (const seg of $segments) {
      if (!seg.target_text.trim() && seg.status === 'new') {
        try {
          const matches = await queryTm({
            source_text: seg.source_text,
            src_lang: $currentProject.src_lang,
            tgt_lang: $currentProject.tgt_lang,
            tm_ids: $projectSettings.tm_ids || [],
            threshold: 0.7,
          });
          if (matches.length > 0 && matches[0].repaired_target) {
            updateSegmentInStore(seg.id, {
              target_text: matches[0].repaired_target,
              status: 'draft',
              match_percent: matches[0].score_pct,
              match_origin: 'tm_repaired',
            });
            await updateSegment({
              segment_id: seg.id,
              target_text: matches[0].repaired_target,
              status: 'draft',
            });
            patchedCount++;
          }
        } catch (e) {
          /* ignore */
        }
      }
    }
    alert(`Auto-patch complete: ${patchedCount} fuzzy segments repaired and filled!`);
  }

  async function lookupTb(sourceText: string) {
    if (!$currentProject) return;
    tbLoading.set(true);
    try {
      const matches = await queryTb({
        text: sourceText,
        lang: $currentProject.src_lang,
        tb_ids: $projectSettings.tb_ids,
      });
      termMatches.set(matches);
    } catch (e) {
      console.error('TB lookup failed:', e);
      termMatches.set([]);
    } finally {
      tbLoading.set(false);
    }
  }

  function applyTmMatch(targetText: string) {
    const seg = $selectedSegment;
    if (!seg) return;
    updateSegmentInStore(seg.id, { target_text: targetText, status: 'draft' });
    scheduleAutoSave(seg.id, targetText, 'draft');
  }

  function handleInsertTerm(termText: string) {
    const seg = $selectedSegment;
    if (!seg) return;
    const newText = seg.target_text ? `${seg.target_text} ${termText}` : termText;
    updateSegmentInStore(seg.id, { target_text: newText, status: 'draft' });
    scheduleAutoSave(seg.id, newText, 'draft');
  }

  function handleTargetChange(segId: string, text: string) {
    const status = text.trim() ? 'draft' : 'new';
    updateSegmentInStore(segId, { target_text: text, status });
    scheduleAutoSave(segId, text, status);
  }

  function scheduleAutoSave(segId: string, text: string, status: string) {
    clearTimeout(autoSaveTimer);
    autoSaveTimer = setTimeout(async () => {
      try {
        await updateSegment({ segment_id: segId, target_text: text, status });
      } catch (e) {
        console.error('Auto-save failed:', e);
      }
    }, $settings.autoSaveIntervalMs);
  }

  async function handleConfirm(seg: { id: string; source_text: string; target_text: string }) {
    if (!seg || !$currentProject) return;
    if (!seg.target_text.trim()) return;

    clearTimeout(autoSaveTimer);
    updateSegmentInStore(seg.id, { status: 'translated', target_text: seg.target_text });
    sessionConfirmedWords += seg.target_text.trim().split(/\s+/).filter(Boolean).length;

    try {
      const propagated = await confirmSegment({
        segment_id: seg.id,
        target_text: seg.target_text,
        project_id: $currentProject.id,
        source_text: seg.source_text,
        src_lang: $currentProject.src_lang,
        tgt_lang: $currentProject.tgt_lang,
      });

      const targetTmIds = ($projectSettings.tm_ids && $projectSettings.tm_ids.length > 0)
        ? $projectSettings.tm_ids
        : [`proj-tm-${$currentProject.id}`];

      for (const tmId of targetTmIds) {
        await addToTm({
          tm_id: tmId,
          source_text: seg.source_text,
          target_text: seg.target_text,
          src_lang: $currentProject.src_lang,
          tgt_lang: $currentProject.tgt_lang,
        }).catch(() => {/* silent */});
      }

      // If auto-propagated to repeated segments, refresh file segments
      if (propagated > 0 && $currentFile) {
        const refreshed = await getSegments($currentFile.id, $currentProject.id);
        segments.set(refreshed);
      }
    } catch (e) {
      console.error('Confirm failed:', e);
    }

    // Trados navigation: jump to the next unconfirmed, unlocked segment
    const currentIdx = $selectedIndex;
    let nextIdx = -1;
    for (let k = currentIdx + 1; k < $filteredSegments.length; k++) {
      if ($filteredSegments[k].status !== 'translated' && !$filteredSegments[k].locked) {
        nextIdx = k;
        break;
      }
    }
    if (nextIdx === -1) {
      for (let k = 0; k < currentIdx; k++) {
        if ($filteredSegments[k].status !== 'translated' && !$filteredSegments[k].locked) {
          nextIdx = k;
          break;
        }
      }
    }
    if (nextIdx !== -1) {
      selectedIndex.set(nextIdx);
    } else {
      selectedIndex.update(i => Math.min(i + 1, $filteredSegments.length - 1));
    }
    await tick();
    scrollToSelected();
  }

  function handleGridKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      selectedIndex.update(i => Math.min(i + 1, $filteredSegments.length - 1));
      scrollToSelected();
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      selectedIndex.update(i => Math.max(i - 1, 0));
      scrollToSelected();
    }
  }

  function scrollToSelected() {
    if (!gridContainer) return;
    const rowHeight = 52;
    const top = $selectedIndex * rowHeight;
    const containerHeight = gridContainer.clientHeight;
    if (top < gridContainer.scrollTop || top + rowHeight > gridContainer.scrollTop + containerHeight) {
      gridContainer.scrollTop = top - containerHeight / 2;
    }
  }

  function jumpToSegment(segId: string) {
    const idx = $filteredSegments.findIndex(s => s.id === segId);
    if (idx !== -1) {
      selectedIndex.set(idx);
      scrollToSelected();
    }
  }

  // Split Segment Handlers
  function openSplitModal(segId: string) {
    const seg = $segments.find(s => s.id === segId);
    if (!seg || seg.source_text.length <= 1) return;
    segmentToSplit = seg;
    splitSliderPos = Math.max(1, Math.floor(seg.source_text.length / 2));
  }

  async function confirmSplit() {
    if (!segmentToSplit || !$currentFile || !$currentProject) return;
    try {
      await splitSegment({
        segment_id: segmentToSplit.id,
        split_index: splitSliderPos,
      });
      const reloaded = await getSegments($currentFile.id, $currentProject.id);
      segments.set(reloaded);
      segmentToSplit = null;
    } catch (e: any) {
      alert(`Split segment failed: ${e}`);
    }
  }

  // Merge Handlers
  async function handleMergeNext(segId: string) {
    if (!$currentFile || !$currentProject) return;
    const idx = $segments.findIndex(s => s.id === segId);
    if (idx === -1 || idx >= $segments.length - 1) return;
    const nextSeg = $segments[idx + 1];
    try {
      await mergeSegments({
        segment_id_1: segId,
        segment_id_2: nextSeg.id,
      });
      const reloaded = await getSegments($currentFile.id, $currentProject.id);
      segments.set(reloaded);
    } catch (e: any) {
      alert(`Merge failed: ${e}`);
    }
  }

  async function handleMergePrev(segId: string) {
    if (!$currentFile || !$currentProject) return;
    const idx = $segments.findIndex(s => s.id === segId);
    if (idx <= 0) return;
    const prevSeg = $segments[idx - 1];
    try {
      await mergeSegments({
        segment_id_1: prevSeg.id,
        segment_id_2: segId,
      });
      const reloaded = await getSegments($currentFile.id, $currentProject.id);
      segments.set(reloaded);
    } catch (e: any) {
      alert(`Merge failed: ${e}`);
    }
  }

  // Bookmark Handler
  async function handleToggleBookmark(segId: string) {
    try {
      const newState = await toggleBookmark(segId);
      updateSegmentInStore(segId, { bookmarked: newState });
    } catch (e) {
      console.error('Bookmark toggle failed:', e);
    }
  }

  // Lock Handler (Ctrl+L)
  async function handleToggleLock(segId: string) {
    try {
      const newState = await toggleSegmentLock(segId);
      updateSegmentInStore(segId, { locked: newState });
    } catch (e) {
      console.error('Lock toggle failed:', e);
    }
  }

  async function handleSourceChange(segId: string, newSource: string) {
    try {
      const updated = await editSourceSegment({ segment_id: segId, source_text: newSource });
      updateSegmentInStore(segId, { source_text: updated.source_text });
    } catch (e: any) {
      alert(`Failed to update source segment: ${e}`);
    }
  }

  function handleQuickInsertChar(e: CustomEvent<string>) {
    const char = e.detail;
    const seg = $selectedSegment;
    if (!seg || seg.locked) return;
    const activeEl = document.activeElement;
    if (activeEl && (activeEl.getAttribute('role') === 'textbox' || activeEl.getAttribute('contenteditable') === 'true')) {
      const sel = window.getSelection();
      if (sel && sel.rangeCount > 0 && activeEl.contains(sel.anchorNode)) {
        const range = sel.getRangeAt(0);
        range.deleteContents();
        const textNode = document.createTextNode(char);
        range.insertNode(textNode);
        range.setStartAfter(textNode);
        range.collapse(true);
        sel.removeAllRanges();
        sel.addRange(range);
        const newText = activeEl.textContent || '';
        handleTargetChange(seg.id, newText);
        return;
      }
    }
    const current = seg.target_text || '';
    const newText = current + char;
    handleTargetChange(seg.id, newText);
  }

  function getFileName(fileId: string): string | null {
    if (!$currentFiles || $currentFiles.length <= 1) return null;
    const f = $currentFiles.find(file => file.id === fileId);
    return f ? f.filename : null;
  }

  async function handleRunQa() {
    if (!$currentProject || !$currentFile) return;
    try {
      const results = await runQa({
        project_id: $currentProject.id,
        file_ids: [$currentFile.id],
        tb_ids: $projectSettings.tb_ids,
      });
      qaResults.set(results);
    } catch (e) {
      console.error('QA failed:', e);
    }
  }

  async function handlePretranslate() {
    if (!$currentProject || !$currentFile) return;
    pretranslating = true;
    try {
      const res = await preTranslate({
        project_id: $currentProject.id,
        file_ids: [$currentFile.id],
        min_score: 1.0,
      });
      const reloaded = await getSegments($currentFile.id, $currentProject.id);
      segments.set(reloaded);
      alert(`Pre-translation complete: ${res.pretranslated} segments filled!`);
    } catch (e) {
      console.error('Pre-translate failed:', e);
    } finally {
      pretranslating = false;
    }
  }

  async function handleExportBilingualReviewDocx() {
    if (!$currentProject || !$currentFile) return;
    try {
      const out = await exportBilingualReview({
        project_id: $currentProject.id,
        file_id: $currentFile.id,
      });
      alert(`Bilingual review Word document exported successfully:\n${out}`);
    } catch (e: any) {
      alert(`Failed to export bilingual review document: ${e}`);
    }
  }

  async function handleImportBilingualReviewDocx() {
    if (!$currentFile || !$currentProject) return;
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: 'Word Document (*.docx)', extensions: ['docx'] }],
      });
      if (!selected) return;
      const docxPath = typeof selected === 'string' ? selected : (selected as string[])[0];
      const res = await importBilingualReview({
        file_id: $currentFile.id,
        docx_path: docxPath,
      });
      const fresh = await getSegments($currentFile.id, $currentProject.id);
      segments.set(fresh);
      alert(`Retrofit Import Complete!\n• ${res.updated_count} segments updated\n• ${res.unchanged_count} segments unchanged`);
    } catch (e: any) {
      alert(`Retrofit import failed: ${e}`);
    }
  }

  const FILTERS: { label: string; value: FilterMode }[] = [
    { label: 'All', value: 'all' },
    { label: 'New', value: 'new' },
    { label: 'Draft', value: 'draft' },
    { label: 'Translated', value: 'translated' },
    { label: 'Review', value: 'needs_review' },
    { label: '★', value: 'bookmarked' },
    { label: 'QA Issues', value: 'qa_warning' },
  ];
</script>

<div class="flex flex-col h-full relative">
  <!-- 1. Studio Ribbon Bar across the top -->
  <StudioRibbon
    {isQuickInsertOpen}
    {isResultsPaneOpen}
    {isTtsPlaying}
    hasSelectedSegment={!!$selectedSegment}
    isSegmentLocked={$selectedSegment ? $selectedSegment.locked : false}
    hasAutoPatch={!!activeAutoPatch}
    on:confirm={() => { if ($selectedSegment) handleConfirm($selectedSegment); }}
    on:copySource={() => { if ($selectedSegment) handleTargetChange($selectedSegment.id, $selectedSegment.source_text); }}
    on:clearTarget={() => { if ($selectedSegment) handleTargetChange($selectedSegment.id, ''); }}
    on:cycleCase={() => { /* SegmentRow Shift+F3 handles or dispatch */ }}
    on:toggleLock={() => { if ($selectedSegment) handleToggleLock($selectedSegment.id); }}
    on:toggleQuickInsert={() => (isQuickInsertOpen = !isQuickInsertOpen)}
    on:openQuickPlace={() => { /* quickplace shortcut Ctrl+, */ }}
    on:openFind={() => { isFindOpen = true; isReplaceOpen = false; }}
    on:openReplace={() => { isFindOpen = true; isReplaceOpen = true; }}
    on:pretranslate={handlePretranslate}
    on:autoPatch={handleAutoPatchAll}
    on:toggleTrackChanges={() => settings.update(s => ({ ...s, revisionMode: !s.revisionMode }))}
    on:openTqa={() => (isTqaOpen = true)}
    on:openQaDrawer={() => { isDrawerOpen = true; activeDrawerTab = 'qa'; handleRunQa(); }}
    on:openNotesDrawer={() => { isDrawerOpen = true; activeDrawerTab = 'comments'; }}
    on:exportDocx={handleExportBilingualReviewDocx}
    on:importDocx={handleImportBilingualReviewDocx}
    on:toggleAdvancedFilter={() => (isAdvancedFilterOpen = !isAdvancedFilterOpen)}
    on:setLayout={(e) => settings.update(s => ({ ...s, layoutMode: e.detail }))}
    on:zoomIn={() => settings.update(s => ({ ...s, fontSize: Math.min(24, s.fontSize + 1) }))}
    on:zoomOut={() => settings.update(s => ({ ...s, fontSize: Math.max(12, s.fontSize - 1) }))}
    on:toggleOpticalBoost={() => settings.update(s => ({ ...s, targetOpticalBoost: !s.targetOpticalBoost }))}
    on:toggleResultsPane={() => (isResultsPaneOpen = !isResultsPaneOpen)}
    on:openConcordance={() => {
      const sel = window.getSelection()?.toString().trim() || '';
      concordanceInitialQuery = sel || ($selectedSegment?.source_text.slice(0, 60) || '');
      isConcordanceOpen = true;
    }}
    on:openWebLookup={() => {
      const sel = window.getSelection()?.toString().trim() || '';
      webLookupQuery = sel || ($selectedSegment?.source_text || '');
      isWebLookupOpen = true;
    }}
    on:openAutoText={() => (isAutoTextOpen = true)}
    on:openAte={() => (isTermExtractionOpen = true)}
    on:openPseudoTranslate={() => (isPseudoTranslateOpen = true)}
    on:openInvoice={() => (isQuoteInvoiceOpen = true)}
    on:toggleTts={toggleTtsPlayback}
    on:openFlowPreview={() => { isDrawerOpen = true; activeDrawerTab = 'preview'; }}
    on:batchCopySource={handleBatchCopySourceToEmptyTarget}
    on:batchClearDraft={handleBatchClearDraftTargets}
    on:batchLockConfirmed={handleBatchLockConfirmed}
    on:batchUnlockAll={handleBatchUnlockAll}
  />

  <!-- 2. Studio Upper Translation Results & Term Recognition Pane -->
  <StudioResultsPane
    isOpen={isResultsPaneOpen}
    bind:height={resultsPaneHeight}
    {activeAutoPatch}
    on:applyTm={(e) => applyTmMatch(e.detail)}
    on:applyAutoPatch={(e) => handleApplyAutoPatch(e.detail)}
    on:insertTerm={(e) => handleInsertTerm(e.detail)}
    on:openConcordance={() => (isConcordanceOpen = true)}
    on:openTermManager={() => { /* goto /tb */ }}
    on:close={() => (isResultsPaneOpen = false)}
  />

  <!-- 3. Compact Ergonomic Filter & Status Strip -->
  <div class="h-8 px-3 border-b border-gray-200 dark:border-gray-800 bg-gray-50/80 dark:bg-gray-950/80 flex items-center justify-between text-xs shrink-0 select-none">
    <!-- Filter Tabs & Quick Search -->
    <div class="flex items-center gap-2 min-w-0">
      <div class="flex items-center gap-0.5 bg-gray-200/60 dark:bg-gray-800/80 rounded p-0.5 shrink-0">
        {#each FILTERS as f}
          <button
            type="button"
            on:click={() => filterMode.set(f.value)}
            class="px-2 py-0.5 text-[11px] rounded font-medium transition-colors flex items-center gap-1 cursor-pointer
                   {$filterMode === f.value
                     ? 'bg-white dark:bg-gray-700 text-gray-900 dark:text-white shadow-2xs font-bold'
                     : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'}"
          >
            <span>{f.label}</span>
            {#if f.value === 'qa_warning' && $qaResults.length > 0}
              <span class="px-1 py-0.1 rounded-full bg-red-500 text-white text-[9px] font-bold leading-none">
                {$qaResults.length}
              </span>
            {/if}
          </button>
        {/each}
      </div>

      <div class="relative">
        <input
          bind:value={$segmentSearch}
          placeholder="Filter rows… (Ctrl+F)"
          class="pl-6 pr-2 py-0.5 text-[11px] rounded border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-900 text-gray-900 dark:text-white w-36 focus:outline-none focus:ring-1 focus:ring-brand-500 font-mono"
        />
        <span class="absolute left-1.5 top-1 text-[10px] text-gray-400">🔍</span>
      </div>

      <button
        type="button"
        on:click={() => (isAdvancedFilterOpen = !isAdvancedFilterOpen)}
        class="px-2 py-0.5 text-[11px] rounded border transition-colors flex items-center gap-1 cursor-pointer
               {$advancedFilter.isActive
                 ? 'bg-brand-500 text-white border-brand-600 font-bold'
                 : 'border-gray-200 dark:border-gray-700 text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800'}"
        title="Trados Advanced Display Filter 2.0"
      >
        <span>🎯 Filter 2.0</span>
        {#if $advancedFilter.isActive}
          <span class="w-1.5 h-1.5 rounded-full bg-emerald-300 animate-ping"></span>
        {/if}
      </button>
    </div>

    <!-- Right: Progress Telemetry -->
    <div class="flex items-center gap-3">
      <div class="flex items-center gap-1.5 text-[11px] text-gray-500 dark:text-gray-400">
        <span>Progress:</span>
        <div class="w-20 h-1.5 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
          <div class="h-full bg-emerald-500 rounded-full transition-all" style="width: {$progressStats.pct}%"></div>
        </div>
        <span class="tabular-nums font-mono font-bold text-gray-800 dark:text-gray-200">{$progressStats.pct}%</span>
        <span class="text-[10px] text-gray-400">({$progressStats.translated}/{$progressStats.total})</span>
      </div>
    </div>
  </div>

  <!-- Find & Replace Floating Toolbar -->
  {#if isFindOpen}
    <FindReplaceBar
      segments={$filteredSegments}
      activeSegmentIndex={$selectedIndex}
      {isReplaceOpen}
      onSelectSegment={(idx) => {
        selectedIndex.set(idx);
        scrollToSelected();
      }}
      onUpdateSegment={(segId, newTarget) => {
        handleTargetChange(segId, newTarget);
      }}
      onClose={() => {
        isFindOpen = false;
        isReplaceOpen = false;
      }}
    />
  {/if}

  <!-- Advanced Display Filter 2.0 Studio Panel -->
  {#if isAdvancedFilterOpen}
    <AdvancedDisplayFilterPanel
      isOpen={isAdvancedFilterOpen}
      onClose={() => (isAdvancedFilterOpen = false)}
    />
  {/if}

  <!-- QuickInsert Typographic Strip -->
  {#if isQuickInsertOpen}
    <QuickInsertBar
      on:insert={handleQuickInsertChar}
      on:close={() => (isQuickInsertOpen = false)}
    />
  {/if}

  <!-- Grid + right panel -->
  <div class="flex flex-1 overflow-hidden">

    <!-- Segment grid -->
    <!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
    <div
      bind:this={gridContainer}
      class="flex-1 overflow-y-auto"
      on:keydown={handleGridKeydown}
      tabindex="-1"
      role="grid"
      aria-label="Translation editor"
    >
      <!-- Column headers -->
      <div
        class="grid sticky top-0 z-10 bg-gray-50 dark:bg-gray-900 border-b-2 border-gray-200 dark:border-gray-800"
        style={$settings.layoutMode === 'stacked'
          ? 'grid-template-columns: 3.25rem 1fr;'
          : `grid-template-columns: 3.25rem minmax(200px, ${$settings.sourceRatio}%) minmax(200px, 1fr);`}
      >
        <div class="py-1.5 px-1 text-xs text-gray-400 text-center">#</div>
        {#if $settings.layoutMode === 'stacked'}
          <div class="py-1.5 px-3 text-xs font-semibold text-gray-600 dark:text-gray-400">
            Source ({$currentProject?.src_lang ?? ''}) ➔ Target ({$currentProject?.tgt_lang ?? ''})
          </div>
        {:else}
          <div class="py-1.5 px-3 text-xs font-semibold text-gray-600 dark:text-gray-400 border-r border-gray-200 dark:border-gray-700 relative flex items-center justify-between">
            <span>Source ({$currentProject?.src_lang ?? ''})</span>
            <!-- Draggable Splitter Handle -->
            <div
              class="absolute right-0 top-0 bottom-0 w-2.5 cursor-col-resize hover:bg-brand-500/50 transition-colors z-20 flex items-center justify-center opacity-60 hover:opacity-100"
              on:pointerdown={handleStartSourceResize}
              title="Drag to adjust Source/Target ratio"
            >
              <div class="w-0.5 h-3 bg-gray-400 dark:bg-gray-500 rounded"></div>
            </div>
          </div>
          <div class="py-1.5 px-3 text-xs font-semibold text-gray-600 dark:text-gray-400">
            Target ({$currentProject?.tgt_lang ?? ''})
          </div>
        {/if}
      </div>

      <!-- Segment rows -->
      {#if $filteredSegments.length === 0}
        <div class="flex items-center justify-center h-48 text-gray-400 text-sm">
          No segments match the filter
        </div>
      {:else}
        {#each $filteredSegments as segment, i (segment.id)}
          {#if (i === 0 || segment.file_id !== $filteredSegments[i - 1].file_id) && getFileName(segment.file_id)}
            <!-- Multi-File Virtual Document File Divider Banner -->
            <div class="px-3.5 py-1.5 bg-gradient-to-r from-brand-50 via-gray-50 to-white dark:from-brand-950/60 dark:via-gray-900 dark:to-gray-950 border-y border-brand-200 dark:border-brand-900/80 flex items-center justify-between text-xs select-none sticky top-[33px] z-10 shadow-2xs">
              <div class="flex items-center gap-2">
                <span class="text-sm">📄</span>
                <span class="font-bold text-gray-900 dark:text-white font-mono">{getFileName(segment.file_id)}</span>
                <span class="text-[9px] px-1.5 py-0.2 rounded-full bg-brand-100 dark:bg-brand-900/60 text-brand-800 dark:text-brand-300 font-semibold uppercase tracking-wider">
                  Document Section
                </span>
              </div>
              <div class="text-[10px] text-gray-400 font-mono">
                ID: {segment.file_id.slice(0, 8)}…
              </div>
            </div>
          {/if}

          <SegmentRow
            {segment}
            isSelected={i === $selectedIndex}
            canMergePrev={i > 0}
            canMergeNext={i < $filteredSegments.length - 1}
            activeSuggestions={activeSuggestions}
            autoPatchSuggestion={i === $selectedIndex ? activeAutoPatch : null}
            onApplyAutoPatch={handleApplyAutoPatch}
            onQuerySuggestions={handleQuerySuggestions}
            onSelect={() => selectedIndex.set(i)}
            onConfirm={handleConfirm}
            onTargetChange={handleTargetChange}
            onSourceChange={handleSourceChange}
            onSplit={openSplitModal}
            onMergeNext={handleMergeNext}
            onMergePrev={handleMergePrev}
            onToggleBookmark={handleToggleBookmark}
            onToggleLock={handleToggleLock}
            onAiRefineAction={handleAiRefineAction}
          />
        {/each}
      {/if}
    </div>

    <!-- Slide-over Inspector Drawer for QA, Comments/Notes, AI Discourse Copilot, and Flow Preview -->
    {#if isDrawerOpen}
      <div
        class="shrink-0 border-l border-gray-200 dark:border-gray-800 flex flex-col bg-white dark:bg-gray-950 relative shadow-xl z-20"
        style="width: {$settings.inspectorWidth}px;"
      >
        <!-- Left Draggable Splitter Handle -->
        <div
          class="absolute -left-1 top-0 bottom-0 w-2.5 cursor-col-resize hover:bg-brand-500/50 transition-colors z-30 flex items-center justify-center opacity-60 hover:opacity-100"
          on:pointerdown={handleStartInspectorResize}
          title="Drag to resize Drawer"
        >
          <div class="w-0.5 h-4 bg-gray-400 dark:bg-gray-500 rounded"></div>
        </div>

        <!-- Drawer Header with Tabs & Close button -->
        <div class="flex items-center justify-between border-b border-gray-200 dark:border-gray-800 bg-gray-100/75 dark:bg-gray-900/90 px-2 py-1.5 shrink-0">
          <div class="flex items-center gap-1 bg-gray-200/60 dark:bg-gray-800/80 rounded p-0.5">
            <button
              type="button"
              on:click={() => (activeDrawerTab = 'qa')}
              class="px-2 py-1 text-[11px] font-medium rounded transition-colors flex items-center gap-1 cursor-pointer
                     {activeDrawerTab === 'qa'
                       ? 'bg-white dark:bg-gray-700 text-gray-900 dark:text-white shadow-2xs font-bold'
                       : 'text-gray-500 hover:text-gray-800 dark:hover:text-gray-200'}"
            >
              <span>✓ QA</span>
              {#if $qaResults.length > 0}
                <span class="w-3.5 h-3.5 rounded-full bg-red-500 text-white text-[9px] font-bold flex items-center justify-center leading-none">
                  {$qaResults.length}
                </span>
              {/if}
            </button>
            <button
              type="button"
              on:click={() => (activeDrawerTab = 'comments')}
              class="px-2 py-1 text-[11px] font-medium rounded transition-colors flex items-center gap-1 cursor-pointer
                     {activeDrawerTab === 'comments'
                       ? 'bg-white dark:bg-gray-700 text-blue-600 dark:text-blue-400 shadow-2xs font-bold'
                       : 'text-gray-500 hover:text-gray-800 dark:hover:text-gray-200'}"
            >
              <span>💬 Notes</span>
              {#if $selectedSegment?.comment}
                <span class="w-1.5 h-1.5 rounded-full bg-blue-500"></span>
              {/if}
            </button>
            <button
              type="button"
              on:click={() => (activeDrawerTab = 'ai')}
              class="px-2 py-1 text-[11px] font-medium rounded transition-colors flex items-center gap-1 cursor-pointer
                     {activeDrawerTab === 'ai'
                       ? 'bg-white dark:bg-gray-700 text-purple-600 dark:text-purple-400 shadow-2xs font-bold'
                       : 'text-gray-500 hover:text-gray-800 dark:hover:text-gray-200'}"
            >
              <span>✨ AI</span>
            </button>
            <button
              type="button"
              on:click={() => (activeDrawerTab = 'preview')}
              class="px-2 py-1 text-[11px] font-medium rounded transition-colors flex items-center gap-1 cursor-pointer
                     {activeDrawerTab === 'preview'
                       ? 'bg-white dark:bg-gray-700 text-emerald-600 dark:text-emerald-400 shadow-2xs font-bold'
                       : 'text-gray-500 hover:text-gray-800 dark:hover:text-gray-200'}"
            >
              <span>👁 Flow</span>
            </button>
          </div>

          <button
            type="button"
            on:click={() => (isDrawerOpen = false)}
            class="w-6 h-6 rounded flex items-center justify-center text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 hover:bg-gray-200 dark:hover:bg-gray-800 text-xs font-bold cursor-pointer transition-colors"
            title="Close Drawer"
          >
            ✕
          </button>
        </div>

        <!-- Drawer Tab content -->
        <div class="flex-1 overflow-hidden">
          {#if activeDrawerTab === 'qa'}
            <QaPanel onJumpToSegment={jumpToSegment} onRunQa={handleRunQa} />
          {:else if activeDrawerTab === 'comments'}
            <CommentsPanel />
          {:else if activeDrawerTab === 'ai'}
            <AiPanel onApplyTarget={(txt) => {
              const seg = $selectedSegment;
              if (seg) handleTargetChange(seg.id, txt);
            }} />
          {:else if activeDrawerTab === 'preview'}
            <LivePreviewPanel onSelectSegment={(idx) => { selectedIndex.set(idx); scrollToSelected(); }} />
          {/if}
        </div>
      </div>
    {/if}

  </div>

  <!-- Bottom Ergonomic Studio Status Bar -->
  <div class="h-8 border-t border-gray-200 dark:border-gray-800 bg-gray-50/95 dark:bg-gray-900/95 backdrop-blur-xs px-3 flex items-center justify-between text-xs text-gray-500 shrink-0 select-none z-10">
    <!-- Left: Velocity & Active Segment Metrics -->
    <div class="flex items-center gap-3">
      <span class="flex items-center gap-1.5 font-medium text-emerald-600 dark:text-emerald-400" title="Live translation velocity for current session">
        <span>⚡</span>
        <span class="tabular-nums font-mono font-semibold">{sessionVelocityWordsPerHour}</span> w/h
      </span>
      <span class="text-gray-300 dark:text-gray-700">|</span>
      <span class="text-gray-600 dark:text-gray-400 text-[11px]">
        Active: <strong class="text-gray-900 dark:text-gray-100">{activeSegSrcWords}</strong> src words ({activeSegSrcChars} chars) ➔ <strong class="text-gray-900 dark:text-gray-100">{activeSegTgtWords}</strong> tgt words
      </span>
      <span class="text-gray-300 dark:text-gray-700">|</span>
      <!-- Acoustic Proofreader Controls -->
      <div class="flex items-center gap-1.5 text-[11px]">
        <button
          type="button"
          on:click={toggleTtsPlayback}
          class="px-2 py-0.5 rounded font-semibold transition-colors flex items-center gap-1 cursor-pointer
                 {isTtsPlaying ? 'bg-purple-600 text-white animate-pulse' : 'bg-gray-200 dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-300'}"
          title="Text-to-Speech Acoustic Proofreading (Ctrl+Alt+R)"
        >
          <span>{isTtsPlaying ? '⏸ Stop' : '🔊 Listen'}</span>
        </button>
        <button
          type="button"
          on:click={() => (isTtsContinuous = !isTtsContinuous)}
          class="px-1.5 py-0.5 rounded text-[10px] cursor-pointer {isTtsContinuous ? 'bg-purple-100 dark:bg-purple-950/60 text-purple-700 dark:text-purple-300 font-bold' : 'text-gray-400 hover:text-gray-600'}"
          title="Auto-read next segment continuously"
        >
          Continuous: {isTtsContinuous ? 'ON' : 'OFF'}
        </button>
        <button
          type="button"
          on:click={() => { ttsRate = ttsRate === 1.0 ? 1.2 : ttsRate === 1.2 ? 1.5 : 1.0; }}
          class="px-1 py-0.5 rounded text-[10px] font-mono text-gray-500 hover:text-gray-800 dark:hover:text-gray-200 cursor-pointer"
          title="Playback speed"
        >
          {ttsRate}x
        </button>
      </div>
    </div>

    <!-- Right: Revision Mode, Layout Switcher, Zoom, WAL Sync -->
    <div class="flex items-center gap-2.5">
      <!-- Revision Mode Toggle -->
      <button
        type="button"
        on:click={() => settings.update(s => ({ ...s, revisionMode: !s.revisionMode }))}
        class="px-2 py-0.5 rounded text-[11px] font-medium transition-colors flex items-center gap-1 cursor-pointer
               {$settings.revisionMode ? 'bg-amber-100 text-amber-800 dark:bg-amber-900/50 dark:text-amber-200 font-bold border border-amber-300' : 'text-gray-500 hover:text-gray-800 dark:hover:text-gray-200'}"
        title="Toggle Track Changes / Revision Mode"
      >
        <span>📝 Track Changes:</span>
        <span>{$settings.revisionMode ? 'ON' : 'OFF'}</span>
      </button>

      <span class="text-gray-300 dark:text-gray-700">|</span>

      <!-- Layout Switcher (Ctrl+L) -->
      <div class="flex items-center gap-0.5 bg-gray-200/60 dark:bg-gray-800 rounded p-0.5" title="Switch Workspace Layout (Ctrl+L)">
        <button
          type="button"
          on:click={() => settings.update(s => ({ ...s, layoutMode: 'side-by-side' }))}
          class="px-1.5 py-0.5 text-[10px] rounded transition-colors cursor-pointer {$settings.layoutMode === 'side-by-side' ? 'bg-white dark:bg-gray-700 font-bold text-brand-600 shadow-2xs' : 'text-gray-500 hover:text-gray-800'}"
        >
          Side-by-Side
        </button>
        <button
          type="button"
          on:click={() => settings.update(s => ({ ...s, layoutMode: 'stacked' }))}
          class="px-1.5 py-0.5 text-[10px] rounded transition-colors cursor-pointer {$settings.layoutMode === 'stacked' ? 'bg-white dark:bg-gray-700 font-bold text-brand-600 shadow-2xs' : 'text-gray-500 hover:text-gray-800'}"
        >
          Stacked
        </button>
        <button
          type="button"
          on:click={() => settings.update(s => ({ ...s, layoutMode: 'zen' }))}
          class="px-1.5 py-0.5 text-[10px] rounded transition-colors cursor-pointer {$settings.layoutMode === 'zen' ? 'bg-white dark:bg-gray-700 font-bold text-purple-600 shadow-2xs' : 'text-gray-500 hover:text-gray-800'}"
        >
          Zen
        </button>
      </div>

      <span class="text-gray-300 dark:text-gray-700">|</span>

      <!-- Font size & Optical boost -->
      <div class="flex items-center gap-1 font-mono text-[11px]">
        <button
          type="button"
          on:click={() => settings.update(s => ({ ...s, fontSize: Math.max(12, s.fontSize - 1) }))}
          class="w-5 h-5 flex items-center justify-center rounded hover:bg-gray-200 dark:hover:bg-gray-800 font-bold cursor-pointer"
          title="Decrease font size"
        >-</button>
        <span class="px-1">{$settings.fontSize}px</span>
        <button
          type="button"
          on:click={() => settings.update(s => ({ ...s, fontSize: Math.min(24, s.fontSize + 1) }))}
          class="w-5 h-5 flex items-center justify-center rounded hover:bg-gray-200 dark:hover:bg-gray-800 font-bold cursor-pointer"
          title="Increase font size"
        >+</button>
        <button
          type="button"
          on:click={() => settings.update(s => ({ ...s, targetOpticalBoost: !s.targetOpticalBoost }))}
          class="px-1.5 py-0.5 rounded text-[10px] transition-colors ml-1 cursor-pointer {$settings.targetOpticalBoost ? 'bg-blue-100 text-blue-800 dark:bg-blue-900/50 dark:text-blue-200 font-semibold' : 'text-gray-400 hover:text-gray-600'}"
          title="Optical Font Boost (+2px for non-Latin / RTL scripts)"
        >
          Optical +2
        </button>
      </div>

      <span class="text-gray-300 dark:text-gray-700">|</span>

      <!-- WAL Sync Status -->
      <div class="flex items-center gap-1 text-[11px] text-gray-500 dark:text-gray-400" title="SQLite WAL mode active">
        <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
        <span>WAL Synced</span>
      </div>
    </div>
  </div>
</div>

<CommandPalette
  isOpen={isCommandPaletteOpen}
  onToggleInspector={() => (isDrawerOpen = !isDrawerOpen)}
  onTogglePreview={() => {
    activeDrawerTab = 'preview';
    isDrawerOpen = true;
  }}
  onOpenFind={() => { isFindOpen = true; isReplaceOpen = false; }}
  onOpenReplace={() => { isFindOpen = true; isReplaceOpen = true; }}
  onRunQa={handleRunQa}
  onExtractTerms={() => (isTermExtractionOpen = true)}
  onOpenInvoice={() => (isQuoteInvoiceOpen = true)}
  onSplit={() => { if ($selectedSegment) openSplitModal($selectedSegment.id); }}
  onCopySource={() => { if ($selectedSegment) handleTargetChange($selectedSegment.id, $selectedSegment.source_text); }}
  onClearTarget={() => { if ($selectedSegment) handleTargetChange($selectedSegment.id, ''); }}
/>

<TermExtractionModal
  isOpen={isTermExtractionOpen}
  onClose={() => (isTermExtractionOpen = false)}
  documentText={documentText}
/>

<QuoteInvoiceModal
  isOpen={isQuoteInvoiceOpen}
  onClose={() => (isQuoteInvoiceOpen = false)}
  projectId={$currentProject?.id || ''}
  fileIds={$currentFile?.id ? [$currentFile.id] : []}
/>

<!-- Split Segment Modal -->
{#if segmentToSplit}
  <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4">
    <div class="bg-white dark:bg-gray-900 rounded-xl border border-gray-200 dark:border-gray-800 shadow-2xl max-w-lg w-full p-6 animate-in fade-in zoom-in-95">
      <h3 class="text-base font-semibold text-gray-900 dark:text-white mb-1">
        Split Segment #{segmentToSplit.seq_number}
      </h3>
      <p class="text-xs text-gray-500 dark:text-gray-400 mb-4">
        Move the slider to choose the split boundary between Part 1 and Part 2.
      </p>

      <!-- Live split preview text -->
      <div class="p-3 bg-gray-50 dark:bg-gray-800/80 rounded-lg text-sm mb-4 font-editor leading-relaxed border border-gray-200 dark:border-gray-700">
        <span class="text-blue-700 dark:text-blue-300 bg-blue-50 dark:bg-blue-900/40 rounded px-1">
          {segmentToSplit.source_text.slice(0, splitSliderPos)}
        </span>
        <span class="text-red-500 font-bold px-0.5">|</span>
        <span class="text-emerald-700 dark:text-emerald-300 bg-emerald-50 dark:bg-emerald-900/40 rounded px-1">
          {segmentToSplit.source_text.slice(splitSliderPos)}
        </span>
      </div>

      <input
        type="range"
        min="1"
        max={Math.max(1, segmentToSplit.source_text.length - 1)}
        bind:value={splitSliderPos}
        class="w-full accent-brand-500 mb-4 cursor-pointer"
      />

      <div class="grid grid-cols-2 gap-3 mb-6 text-xs">
        <div class="p-2.5 rounded bg-blue-50/50 dark:bg-blue-950/20 border border-blue-200 dark:border-blue-900">
          <div class="font-semibold text-blue-700 dark:text-blue-400 mb-1">Segment {segmentToSplit.seq_number} (Part 1)</div>
          <div class="text-gray-800 dark:text-gray-200 italic line-clamp-3">{segmentToSplit.source_text.slice(0, splitSliderPos)}</div>
        </div>
        <div class="p-2.5 rounded bg-emerald-50/50 dark:bg-emerald-950/20 border border-emerald-200 dark:border-emerald-900">
          <div class="font-semibold text-emerald-700 dark:text-emerald-400 mb-1">Segment {segmentToSplit.seq_number + 1} (Part 2)</div>
          <div class="text-gray-800 dark:text-gray-200 italic line-clamp-3">{segmentToSplit.source_text.slice(splitSliderPos)}</div>
        </div>
      </div>

      <div class="flex items-center justify-end gap-2">
        <button
          on:click={() => segmentToSplit = null}
          class="px-3 py-1.5 text-xs text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg transition-colors"
        >
          Cancel
        </button>
        <button
          on:click={confirmSplit}
          class="px-4 py-1.5 text-xs font-medium text-white bg-brand-500 hover:bg-brand-600 rounded-lg shadow-sm transition-colors"
        >
          Split Segment
        </button>
      </div>
    </div>
  </div>
{/if}

<ConcordanceModal
  isOpen={isConcordanceOpen}
  initialQuery={concordanceInitialQuery}
  onInsert={handleInsertConcordance}
  onClose={() => (isConcordanceOpen = false)}
/>

<PseudoTranslateModal
  isOpen={isPseudoTranslateOpen}
  on:close={() => (isPseudoTranslateOpen = false)}
  on:applied={async () => {
    if ($currentFile && $currentProject) {
      const fresh = await getSegments($currentFile.id, $currentProject.id);
      segments.set(fresh);
    }
  }}
/>

<TqaDrawer
  bind:this={tqaDrawerComponent}
  isOpen={isTqaOpen}
  on:close={() => (isTqaOpen = false)}
/>

<AutoTextManagerModal
  isOpen={isAutoTextOpen}
  on:close={() => (isAutoTextOpen = false)}
/>

<WebLookupModal
  isOpen={isWebLookupOpen}
  initialQuery={webLookupQuery}
  srcLang={$currentProject?.src_lang || 'en'}
  tgtLang={$currentProject?.tgt_lang || 'es'}
  on:close={() => (isWebLookupOpen = false)}
/>
