<script lang="ts">
  /**
   * Aligner Page — Visual Bilingual File & Document Aligner.
   * Powered by Dynamic Time Warping (DTW) with Gale-Church length ratio and numeric anchor detection.
   */
  import { onMount } from 'svelte';
  import { alignTexts, alignFiles, exportAlignmentTmx, importAlignmentToTm, type AlignedPair } from '$lib/ipc/aligner';
  import { listTms, type TmInfo } from '$lib/ipc/tm';
  import { toast } from '$lib/stores/toast';

  let srcLang = 'en';
  let tgtLang = 'ar';
  let mode: 'text' | 'file' = 'text';

  let sourceText = '';
  let targetText = '';
  let sourcePath = '';
  let targetPath = '';

  let aligning = false;
  let exporting = false;
  let importing = false;

  let pairs: AlignedPair[] = [];
  let availableTmsList: TmInfo[] = [];
  let selectedTmId = '';

  onMount(async () => {
    try {
      availableTmsList = await listTms();
      if (availableTmsList.length > 0) {
        selectedTmId = availableTmsList[0].id;
      }
    } catch (e) {
      console.error('Failed to list TMs:', e);
    }
  });

  function loadSampleData() {
    srcLang = 'en';
    tgtLang = 'ar';
    sourceText = `MASTER SERVICES AGREEMENT
1. Scope of Services. The Service Provider shall perform the consulting and localization services described in Exhibit A.
2. Compensation and Payment Terms. The Client agrees to pay a total sum of $15,000 for the deliverables. Payments shall be made within thirty (30) days of receipt of invoice.
3. Term and Termination. This Agreement shall commence on January 15, 2026, and shall continue in full force for twelve (12) months. Either party may terminate with 30 days written notice.
4. Governing Law. This Agreement shall be governed by and construed in accordance with the laws of Delaware.`;

    targetText = `اتفاقية الخدمات الرئيسية
1. نطاق الخدمات. يلتزم مقدم الخدمة بأداء خدمات الاستشارات والتعريب الموضحة في الملحق (أ).
2. شروط التعويض والدفع. يوافق العميل على دفع مبلغ إجمالي قدره 15,000 دولار مقابل المخرجات. يتم السداد في غضون ثلاثين (30) يومًا من استلام الفاتورة.
3. المدة والإنهاء. تبدأ هذه الاتفاقية في 15 يناير 2026، وتستمر سارية المفعول بالكامل لمدة اثني عشر (12) شهرًا. يجوز لأي من الطرفين الإنهاء بإشعار كتابي مدته 30 يومًا.
4. القانون الحاكم. تخضع هذه الاتفاقية وتُفسر وفقًا لقوانين ولاية ديلاوير.`;
  }

  async function handleRunAlignment() {
    aligning = true;
    try {
      if (mode === 'text') {
        if (!sourceText.trim() || !targetText.trim()) {
          throw new Error('Please provide both source and target text.');
        }
        pairs = await alignTexts({
          source_text: sourceText,
          target_text: targetText,
          src_lang: srcLang,
          tgt_lang: tgtLang,
        });
      } else {
        if (!sourcePath.trim() || !targetPath.trim()) {
          throw new Error('Please provide both source and target file paths.');
        }
        pairs = await alignFiles({
          source_path: sourcePath,
          target_path: targetPath,
          src_lang: srcLang,
          tgt_lang: tgtLang,
        });
      }
      toast.success('Alignment Complete', `Successfully aligned ${pairs.length} sentence pairs.`);
    } catch (e: any) {
      toast.error('Alignment Failed', String(e));
    } finally {
      aligning = false;
    }
  }

  function handleToggleConfirm(index: number) {
    pairs[index].confirmed = !pairs[index].confirmed;
    pairs = [...pairs];
  }

  function handleConfirmAll() {
    pairs = pairs.map(p => ({ ...p, confirmed: true }));
  }

  function handleMergeDown(index: number) {
    if (index >= pairs.length - 1) return;
    const current = pairs[index];
    const next = pairs[index + 1];

    const mergedSource = [current.source_text, next.source_text].filter(Boolean).join(' ');
    const mergedTarget = [current.target_text, next.target_text].filter(Boolean).join(' ');

    pairs[index] = {
      ...current,
      source_text: mergedSource,
      target_text: mergedTarget,
      alignment_type: 'merged',
      score: Math.round((current.score + next.score) / 2),
    };

    pairs.splice(index + 1, 1);
    pairs = [...pairs];
  }

  function handleDeletePair(index: number) {
    pairs.splice(index, 1);
    pairs = [...pairs];
  }

  async function handleExportTmx() {
    if (pairs.length === 0) return;
    exporting = true;
    try {
      const outputPath = `aligned_${Date.now()}.tmx`;
      await exportAlignmentTmx({
        pairs,
        src_lang: srcLang,
        tgt_lang: tgtLang,
        output_path: outputPath,
      });
      toast.success('TMX Exported', `Saved to: ${outputPath}`);
    } catch (e: any) {
      toast.error('Export Failed', String(e));
    } finally {
      exporting = false;
    }
  }

  async function handleImportToTm() {
    if (!selectedTmId || pairs.length === 0) return;
    importing = true;
    try {
      const count = await importAlignmentToTm({
        tm_id: selectedTmId,
        pairs,
        src_lang: srcLang,
        tgt_lang: tgtLang,
      });
      toast.success('Imported to TM', `${count} translation units added.`);
    } catch (e: any) {
      toast.error('Import Failed', String(e));
    } finally {
      importing = false;
    }
  }
</script>

<div class="h-full flex flex-col bg-white dark:bg-gray-950 overflow-hidden">
  <!-- Top bar -->
  <header class="px-6 py-4 border-b border-gray-200 dark:border-gray-800 flex items-center justify-between bg-white dark:bg-gray-950 shrink-0">
    <div class="flex items-center gap-3">
      <div class="w-8 h-8 rounded-lg bg-brand-500/10 text-brand-500 flex items-center justify-center text-lg font-bold">
        📐
      </div>
      <div>
        <h1 class="text-base font-semibold text-gray-900 dark:text-white">
          Visual Bilingual File Aligner
        </h1>
        <p class="text-xs text-gray-500 dark:text-gray-400">
          Embedding & length-ratio Dynamic Time Warping (DTW) alignment for legacy translations.
        </p>
      </div>
    </div>

    <!-- Language selector & sample loader -->
    <div class="flex items-center gap-3">
      <div class="flex items-center gap-1.5 text-xs text-gray-600 dark:text-gray-300">
        <span class="font-medium">Src:</span>
        <input
          type="text"
          bind:value={srcLang}
          class="w-16 px-2 py-1 rounded bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 text-center font-mono uppercase"
        />
        <span>→</span>
        <span class="font-medium">Tgt:</span>
        <input
          type="text"
          bind:value={tgtLang}
          class="w-16 px-2 py-1 rounded bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 text-center font-mono uppercase"
        />
      </div>

      <button
        onclick={loadSampleData}
        class="px-2.5 py-1 text-xs border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-800 text-gray-600 dark:text-gray-300 rounded-lg transition-colors"
      >
        📄 Load Sample
      </button>
    </div>
  </header>


  <!-- Main View -->
  <div class="flex-1 flex flex-col overflow-hidden p-6 space-y-4">
    {#if pairs.length === 0}
      <!-- Configuration & Input Form -->
      <div class="bg-gray-50 dark:bg-gray-900/40 border border-gray-200 dark:border-gray-800 rounded-xl p-5 space-y-4 max-w-5xl mx-auto w-full">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <button
              onclick={() => (mode = 'text')}
              class="px-3 py-1.5 text-xs font-medium rounded-lg transition-colors
                     {mode === 'text'
                       ? 'bg-brand-500 text-white shadow-xs'
                       : 'text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-800'}"
            >
              📝 Paste Text
            </button>
            <button
              onclick={() => (mode = 'file')}
              class="px-3 py-1.5 text-xs font-medium rounded-lg transition-colors
                     {mode === 'file'
                       ? 'bg-brand-500 text-white shadow-xs'
                       : 'text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-800'}"
            >
              📁 Document File Paths
            </button>
          </div>

          <button
            onclick={handleRunAlignment}
            disabled={aligning}
            class="px-5 py-2 text-xs font-semibold text-white bg-brand-500 hover:bg-brand-600 disabled:opacity-50 rounded-lg shadow-sm transition-colors flex items-center gap-1.5"
          >
            <span>{aligning ? '⏳ Aligning Sentences…' : '⚡ Run Alignment'}</span>
          </button>
        </div>

        {#if mode === 'text'}
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label for="align-src-text" class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1.5">
                Source Document ({srcLang.toUpperCase()})
              </label>
              <textarea
                id="align-src-text"
                bind:value={sourceText}
                placeholder="Paste source text or contract paragraphs here..."
                rows="14"
                class="w-full text-xs p-3 font-editor bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg focus:outline-none focus:ring-1 focus:ring-brand-500 text-gray-900 dark:text-gray-100 resize-none leading-relaxed"
                dir="auto"
              ></textarea>
            </div>
            <div>
              <label for="align-tgt-text" class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1.5">
                Target Translation ({tgtLang.toUpperCase()})
              </label>
              <textarea
                id="align-tgt-text"
                bind:value={targetText}
                placeholder="Paste target translation paragraphs here..."
                rows="14"
                class="w-full text-xs p-3 font-editor bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg focus:outline-none focus:ring-1 focus:ring-brand-500 text-gray-900 dark:text-gray-100 resize-none leading-relaxed"
                dir="auto"
              ></textarea>
            </div>
          </div>
        {:else}
          <div class="space-y-4 py-4">
            <div>
              <label for="align-src-path" class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1.5">
                Source File Path (.docx, .txt, .html, .md, .srt)
              </label>
              <input
                id="align-src-path"
                type="text"
                bind:value={sourcePath}
                placeholder="e.g. C:/Documents/Contract_EN.docx"
                class="w-full text-xs p-2.5 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg text-gray-900 dark:text-gray-100"
              />
            </div>
            <div>
              <label for="align-tgt-path" class="block text-xs font-medium text-gray-700 dark:text-gray-300 mb-1.5">
                Target File Path (.docx, .txt, .html, .md, .srt)
              </label>
              <input
                id="align-tgt-path"
                type="text"
                bind:value={targetPath}
                placeholder="e.g. C:/Documents/Contract_AR.docx"
                class="w-full text-xs p-2.5 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg text-gray-900 dark:text-gray-100"
              />
            </div>
          </div>
        {/if}
      </div>
    {:else}
      <!-- Alignment Results & Interactive Editor Workbench -->
      <div class="flex items-center justify-between pb-2 shrink-0">
        <div class="flex items-center gap-2">
          <button
            onclick={() => (pairs = [])}
            class="px-2.5 py-1 text-xs border border-gray-200 dark:border-gray-700 text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg"
          >
            ← New Alignment
          </button>
          <button
            onclick={handleConfirmAll}
            class="px-2.5 py-1 text-xs border border-emerald-300 text-emerald-700 dark:text-emerald-300 bg-emerald-50 dark:bg-emerald-950/30 hover:bg-emerald-100 rounded-lg"
          >
            ✓ Confirm All
          </button>
          <span class="text-xs text-gray-400">
            {pairs.filter(p => p.confirmed).length} of {pairs.length} confirmed
          </span>
        </div>

        <div class="flex items-center gap-2">
          <!-- TM selection and import -->
          {#if availableTmsList.length > 0}
            <select
              bind:value={selectedTmId}
              class="text-xs px-2.5 py-1 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg text-gray-800 dark:text-gray-200"
            >
              {#each availableTmsList as tm}
                <option value={tm.id}>TM: {tm.name}</option>
              {/each}
            </select>

            <button
              onclick={handleImportToTm}
              disabled={importing}
              class="px-3 py-1 text-xs font-medium text-white bg-brand-500 hover:bg-brand-600 disabled:opacity-50 rounded-lg shadow-xs"
            >
              {importing ? 'Importing…' : '📥 Import into TM'}
            </button>
          {/if}

          <!-- TMX Export -->
          <button
            onclick={handleExportTmx}
            disabled={exporting}
            class="px-3 py-1 text-xs font-medium border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-800 rounded-lg"
          >
            {exporting ? 'Exporting…' : '💾 Export TMX'}
          </button>
        </div>
      </div>

      <!-- Alignment Grid Table -->
      <div class="flex-1 overflow-y-auto border border-gray-200 dark:border-gray-800 rounded-xl bg-white dark:bg-gray-900 shadow-xs">
        <table class="w-full border-collapse text-left text-xs">
          <thead class="bg-gray-50 dark:bg-gray-900/80 sticky top-0 border-b border-gray-200 dark:border-gray-800 z-10">
            <tr class="text-[11px] uppercase tracking-wider text-gray-500 dark:text-gray-400">
              <th class="py-2 px-3 w-12 text-center">#</th>
              <th class="py-2 px-3 w-[44%] border-r border-gray-200 dark:border-gray-800">Source Sentence</th>
              <th class="py-2 px-3 w-20 text-center border-r border-gray-200 dark:border-gray-800">Match</th>
              <th class="py-2 px-3 w-[44%] border-r border-gray-200 dark:border-gray-800">Target Sentence</th>
              <th class="py-2 px-3 w-20 text-center">Actions</th>
            </tr>
          </thead>
          <tbody class="divide-y divide-gray-100 dark:divide-gray-800">
            {#each pairs as pair, i (pair.id)}
              <tr class="hover:bg-gray-50/50 dark:hover:bg-gray-800/20 transition-colors {pair.confirmed ? '' : 'opacity-60 bg-gray-50/20'}">
                <!-- Checkbox / Row # -->
                <td class="py-2.5 px-3 text-center align-top">
                  <button
                    onclick={() => handleToggleConfirm(i)}
                    class="w-5 h-5 rounded border flex items-center justify-center text-[10px] font-bold transition-colors
                           {pair.confirmed
                             ? 'bg-emerald-500 border-emerald-500 text-white'
                             : 'border-gray-300 dark:border-gray-600 text-transparent'}"
                    title="Toggle Confirmation"
                  >
                    ✓
                  </button>
                  <span class="text-[10px] text-gray-400 font-mono block mt-1">{i + 1}</span>
                </td>

                <!-- Source text -->
                <td class="py-2.5 px-3 align-top border-r border-gray-100 dark:border-gray-800 font-editor leading-relaxed text-gray-800 dark:text-gray-200">
                  <textarea
                    bind:value={pair.source_text}
                    rows="2"
                    class="w-full bg-transparent border-0 outline-none resize-none p-0 text-xs focus:ring-0"
                    dir="auto"
                  ></textarea>
                </td>

                <!-- Match Badge -->
                <td class="py-2.5 px-2 align-top text-center border-r border-gray-100 dark:border-gray-800">
                  <span
                    class="inline-block px-1.5 py-0.5 rounded text-[10px] font-mono font-semibold
                           {pair.score >= 85
                             ? 'bg-emerald-100 dark:bg-emerald-950/60 text-emerald-700 dark:text-emerald-300'
                             : pair.score >= 60
                             ? 'bg-blue-100 dark:bg-blue-950/60 text-blue-700 dark:text-blue-300'
                             : 'bg-amber-100 dark:bg-amber-950/60 text-amber-700 dark:text-amber-300'}"
                  >
                    {pair.score}%
                  </span>
                  <span class="block text-[9px] text-gray-400 font-mono mt-0.5">{pair.alignment_type}</span>
                </td>

                <!-- Target text -->
                <td class="py-2.5 px-3 align-top border-r border-gray-100 dark:border-gray-800 font-editor leading-relaxed text-gray-800 dark:text-gray-200">
                  <textarea
                    bind:value={pair.target_text}
                    rows="2"
                    class="w-full bg-transparent border-0 outline-none resize-none p-0 text-xs focus:ring-0"
                    dir="auto"
                  ></textarea>
                </td>

                <!-- Actions -->
                <td class="py-2.5 px-2 align-top text-center space-x-1">
                  {#if i < pairs.length - 1}
                    <button
                      onclick={() => handleMergeDown(i)}
                      class="text-gray-400 hover:text-brand-600 dark:hover:text-brand-400 p-1"
                      title="Merge with sentence below (⤹)"
                    >
                      ⤹
                    </button>
                  {/if}
                  <button
                    onclick={() => handleDeletePair(i)}
                    class="text-gray-400 hover:text-red-500 p-1"
                    title="Delete alignment pair"
                  >
                    🗑
                  </button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </div>
</div>
