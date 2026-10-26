<script lang="ts">
  import { dialog } from '$lib/actions/dialog';
  // QuoteInvoiceModal.svelte — Freelancer Quoting & Invoicing Suite
  import { getDetailedAnalysis, type DetailedAnalysisReport, type AnalysisBand } from '$lib/ipc/project';
  import { currentProject, currentFiles } from '$lib/stores/project';
  import { onMount } from 'svelte';

  interface Props {
    isOpen: boolean;
    onClose: () => void;
    projectId: string;
    fileIds: string[];
  }

  let { isOpen, onClose, projectId, fileIds }: Props = $props();

  let mode = $state<'quote' | 'invoice'>('invoice');
  let loading = $state(false);
  let error = $state<string | null>(null);
  let report = $state<DetailedAnalysisReport | null>(null);

  // Business / Freelancer details
  let freelancerName = $state('Professional Translator');
  let freelancerEmail = $state('translator@freecat.org');
  let freelancerTaxId = $state('VAT: 123456789');

  // Client details
  let clientName = $state('Acme Localization Corp');
  let clientAddress = $state('100 Global Way, London, UK');
  let poNumber = $state('PO-88219');
  let docNumber = $state('INV-2026-042');
  let issueDate = $state(new Date().toISOString().slice(0, 10));
  let dueDate = $state(new Date(Date.now() + 14 * 86400000).toISOString().slice(0, 10));

  // Pricing & Currency
  let currency = $state('EUR €');
  let baseWordRate = $state(0.12);
  let taxRatePct = $state(0);
  let minimumFee = $state(0);
  let customBandRates = $state<Record<string, number>>({});

  $effect(() => {
    if (isOpen && projectId && (!report || report.project_id !== projectId)) {
      fetchAnalysis();
    }
  });

  async function fetchAnalysis() {
    loading = true;
    error = null;
    try {
      const res = await getDetailedAnalysis(projectId, fileIds);
      report = res;
      // Initialize band rates from report defaults
      const rates: Record<string, number> = {};
      for (const b of res.bands) {
        rates[b.id] = Math.round(b.default_rate_pct * 100);
      }
      customBandRates = rates;
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  // Calculated totals
  let bandCalculations = $derived(() => {
    if (!report) return [];
    return report.bands.map(b => {
      const ratePct = customBandRates[b.id] ?? Math.round(b.default_rate_pct * 100);
      const effectiveWords = (b.words * ratePct) / 100;
      const subtotal = effectiveWords * baseWordRate;
      return {
        ...b,
        ratePct,
        effectiveWords,
        subtotal,
      };
    });
  });

  let totalRawWords = $derived(report?.total_words ?? 0);
  let totalWeightedWords = $derived(bandCalculations().reduce((acc, b) => acc + b.effectiveWords, 0));
  let subtotalAmount = $derived(bandCalculations().reduce((acc, b) => acc + b.subtotal, 0));
  let appliedSubtotal = $derived(Math.max(subtotalAmount, minimumFee));
  let taxAmount = $derived((appliedSubtotal * taxRatePct) / 100);
  let grandTotal = $derived(appliedSubtotal + taxAmount);

  function handlePrint() {
    window.print();
  }
</script>

{#if isOpen}
  <div use:dialog={{ label: "Quote and invoice" }} class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-xs overflow-y-auto">
    <div
      class="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-xl shadow-2xl w-full max-w-4xl max-h-[90vh] flex flex-col overflow-hidden"
    >
      <!-- Header (Hidden in print) -->
      <div class="no-print px-5 py-4 border-b border-gray-200 dark:border-gray-800 flex items-center justify-between bg-gray-50/50 dark:bg-gray-850">
        <div class="flex items-center gap-3">
          <div class="w-9 h-9 rounded-lg bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 flex items-center justify-center text-xl font-bold">
            💼
          </div>
          <div>
            <h2 class="text-base font-semibold text-gray-900 dark:text-gray-100 flex items-center gap-2">
              <span>Freelancer Quoting & Invoicing Suite</span>
              <span class="text-xs px-2 py-0.5 rounded-full bg-emerald-100 dark:bg-emerald-950/50 text-emerald-700 dark:text-emerald-300 font-normal">
                Trados CAT Matrix
              </span>
            </h2>
            <p class="text-xs text-gray-500 dark:text-gray-400">
              Calculate weighted word counts, customize client rate matrices, and print official PDF quotes & invoices.
            </p>
          </div>
        </div>

        <div class="flex items-center gap-2">
          <div class="inline-flex rounded-lg border border-gray-300 dark:border-gray-700 p-0.5 bg-gray-100 dark:bg-gray-800 text-xs">
            <button
              onclick={() => mode = 'invoice'}
              class="px-3 py-1 rounded-md font-medium transition-colors {mode === 'invoice' ? 'bg-white dark:bg-gray-700 text-emerald-600 dark:text-emerald-400 shadow-xs' : 'text-gray-500'}"
            >
              Invoice
            </button>
            <button
              onclick={() => mode = 'quote'}
              class="px-3 py-1 rounded-md font-medium transition-colors {mode === 'quote' ? 'bg-white dark:bg-gray-700 text-emerald-600 dark:text-emerald-400 shadow-xs' : 'text-gray-500'}"
            >
              Quote / Estimate
            </button>
          </div>

          <button
            onclick={handlePrint}
            class="px-3 py-1.5 bg-emerald-600 hover:bg-emerald-700 text-white rounded-lg text-xs font-medium shadow-xs transition-colors flex items-center gap-1.5"
          >
            🖨️ Print / Save PDF
          </button>

          <button
            onclick={onClose}
            class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 p-1.5 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors ml-1"
            aria-label="Close"
          >
            ✕
          </button>
        </div>
      </div>

      <!-- Controls & Parameters Toolbar (Hidden in print) -->
      <div class="no-print p-4 bg-gray-50 dark:bg-gray-800/40 border-b border-gray-200 dark:border-gray-800 grid grid-cols-2 sm:grid-cols-4 gap-3 text-xs">
        <label class="flex flex-col gap-1">
          <span class="text-gray-500 font-medium">Currency:</span>
          <select
            bind:value={currency}
            class="px-2 py-1.5 rounded border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-800 dark:text-gray-200"
          >
            <option value="EUR €">EUR (€)</option>
            <option value="USD $">USD ($)</option>
            <option value="GBP £">GBP (£)</option>
            <option value="CAD $">CAD ($)</option>
            <option value="AUD $">AUD ($)</option>
            <option value="JPY ¥">JPY (¥)</option>
            <option value="CHF Fr">CHF (Fr)</option>
          </select>
        </label>

        <label class="flex flex-col gap-1">
          <span class="text-gray-500 font-medium">Base Word Rate:</span>
          <input
            type="number"
            step="0.005"
            bind:value={baseWordRate}
            class="px-2 py-1.5 rounded border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-800 dark:text-gray-200 font-mono"
          />
        </label>

        <label class="flex flex-col gap-1">
          <span class="text-gray-500 font-medium">Tax / VAT (%):</span>
          <input
            type="number"
            step="1"
            bind:value={taxRatePct}
            class="px-2 py-1.5 rounded border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-800 dark:text-gray-200 font-mono"
          />
        </label>

        <label class="flex flex-col gap-1">
          <span class="text-gray-500 font-medium">Min Fee ({currency.split(' ')[1]}):</span>
          <input
            type="number"
            step="5"
            bind:value={minimumFee}
            class="px-2 py-1.5 rounded border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 text-gray-800 dark:text-gray-200 font-mono"
          />
        </label>
      </div>

      <!-- Printable Invoice Document Canvas -->
      <div class="flex-1 overflow-y-auto p-8 printable-document bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100">
        {#if loading}
          <div class="flex flex-col items-center justify-center h-64 text-gray-400 gap-2">
            <div class="w-8 h-8 border-2 border-emerald-500 border-t-transparent rounded-full animate-spin"></div>
            <span>Calculating word count bands and TM leverage…</span>
          </div>
        {:else if error}
          <div class="p-4 bg-red-50 text-red-600 rounded-lg text-sm border border-red-200">
            {error}
          </div>
        {:else if report}
          <!-- Printable Header -->
          <div class="flex justify-between items-start border-b border-gray-300 dark:border-gray-700 pb-6 mb-6">
            <div>
              <h1 class="text-2xl font-bold tracking-tight text-gray-900 dark:text-white uppercase">
                {mode === 'invoice' ? 'Translation Invoice' : 'Translation Quotation'}
              </h1>
              <div class="text-xs text-gray-500 dark:text-gray-400 mt-1 space-y-0.5">
                <div>Document: <span class="font-mono font-semibold">{docNumber}</span></div>
                <div>Date: {issueDate}</div>
                {#if mode === 'invoice'}
                  <div>Payment Due: <span class="font-semibold text-gray-700 dark:text-gray-300">{dueDate}</span></div>
                {/if}
              </div>
            </div>

            <!-- Freelancer Info -->
            <div class="text-right text-xs text-gray-600 dark:text-gray-400">
              <input
                bind:value={freelancerName}
                class="font-bold text-sm text-right text-gray-900 dark:text-gray-100 bg-transparent border-0 border-b border-transparent hover:border-gray-300 focus:border-brand-500 p-0 block ml-auto w-48 text-right"
              />
              <input
                bind:value={freelancerEmail}
                class="bg-transparent border-0 border-b border-transparent hover:border-gray-300 focus:border-brand-500 p-0 block ml-auto w-48 text-right text-gray-500"
              />
              <input
                bind:value={freelancerTaxId}
                class="bg-transparent border-0 border-b border-transparent hover:border-gray-300 focus:border-brand-500 p-0 block ml-auto w-48 text-right text-gray-500 font-mono"
              />
            </div>
          </div>

          <!-- Bill To & Project Info -->
          <div class="grid grid-cols-2 gap-8 mb-6 text-xs">
            <div>
              <span class="uppercase tracking-wider font-semibold text-gray-400 text-[10px] block mb-1">
                {mode === 'invoice' ? 'Billed To:' : 'Quotation For:'}
              </span>
              <input
                bind:value={clientName}
                class="font-semibold text-gray-900 dark:text-gray-100 bg-transparent border-0 border-b border-transparent hover:border-gray-300 focus:border-brand-500 p-0 block w-full"
              />
              <input
                bind:value={clientAddress}
                class="text-gray-500 bg-transparent border-0 border-b border-transparent hover:border-gray-300 focus:border-brand-500 p-0 block w-full mt-0.5"
              />
              <div class="flex items-center gap-1 mt-1 text-gray-500">
                <span>PO / Ref:</span>
                <input
                  bind:value={poNumber}
                  class="font-mono bg-transparent border-0 border-b border-transparent hover:border-gray-300 focus:border-brand-500 p-0 w-32"
                />
              </div>
            </div>

            <div class="bg-gray-50 dark:bg-gray-800/40 p-3 rounded-lg border border-gray-200 dark:border-gray-800 space-y-1">
              <div class="text-[10px] uppercase font-semibold text-gray-400 tracking-wider">Project Details</div>
              <div class="font-medium text-gray-800 dark:text-gray-200">
                {$currentProject?.name || 'Translation Project'}
              </div>
              <div class="text-gray-500 text-[11px]">
                Language Pair: <span class="font-semibold font-mono">{$currentProject?.src_lang.toUpperCase()} → {$currentProject?.tgt_lang.toUpperCase()}</span>
              </div>
              <div class="text-gray-500 text-[11px]">
                Scope: <span class="font-mono">{report.total_segments}</span> segments / <span class="font-mono">{report.total_words}</span> source words
              </div>
            </div>
          </div>

          <!-- Detailed CAT Breakdown Table -->
          <table class="w-full text-xs text-left mb-6">
            <thead>
              <tr class="border-b-2 border-gray-300 dark:border-gray-700 text-gray-500 dark:text-gray-400 uppercase text-[10px] tracking-wider">
                <th class="pb-2 font-semibold">Match Category</th>
                <th class="pb-2 font-semibold text-center">Range</th>
                <th class="pb-2 font-semibold text-right">Segments</th>
                <th class="pb-2 font-semibold text-right">Source Words</th>
                <th class="pb-2 font-semibold text-right">Pay %</th>
                <th class="pb-2 font-semibold text-right">Weighted Words</th>
                <th class="pb-2 font-semibold text-right">Line Total ({currency.split(' ')[1]})</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-200 dark:divide-gray-800">
              {#each bandCalculations() as row}
                <tr class="hover:bg-gray-50/50 dark:hover:bg-gray-800/20">
                  <td class="py-2.5 font-medium text-gray-900 dark:text-gray-100">
                    {row.name}
                  </td>
                  <td class="py-2.5 text-center text-gray-500 font-mono text-[11px]">
                    {row.match_range}
                  </td>
                  <td class="py-2.5 text-right text-gray-600 dark:text-gray-400 font-mono">
                    {row.segments}
                  </td>
                  <td class="py-2.5 text-right font-mono font-medium text-gray-800 dark:text-gray-200">
                    {row.words.toLocaleString()}
                  </td>
                  <td class="py-2.5 text-right">
                    <span class="font-mono text-gray-600 dark:text-gray-400">
                      {row.ratePct}%
                    </span>
                  </td>
                  <td class="py-2.5 text-right font-mono text-emerald-700 dark:text-emerald-400 font-medium">
                    {Math.round(row.effectiveWords).toLocaleString()}
                  </td>
                  <td class="py-2.5 text-right font-mono font-semibold text-gray-900 dark:text-gray-100">
                    {row.subtotal.toFixed(2)}
                  </td>
                </tr>
              {/each}
            </tbody>
            <tfoot>
              <tr class="border-t-2 border-gray-300 dark:border-gray-700 font-semibold text-gray-900 dark:text-gray-100">
                <td class="pt-3" colspan="2">Total Project Scope</td>
                <td class="pt-3 text-right font-mono">{report.total_segments}</td>
                <td class="pt-3 text-right font-mono">{totalRawWords.toLocaleString()}</td>
                <td class="pt-3 text-right text-gray-400">—</td>
                <td class="pt-3 text-right font-mono text-emerald-700 dark:text-emerald-400">
                  {Math.round(totalWeightedWords).toLocaleString()}
                </td>
                <td class="pt-3 text-right font-mono text-base font-bold">
                  {subtotalAmount.toFixed(2)}
                </td>
              </tr>
            </tfoot>
          </table>

          <!-- Financial Summary Section -->
          <div class="flex justify-end mb-8">
            <div class="w-72 space-y-2 text-xs">
              <div class="flex justify-between text-gray-600 dark:text-gray-400">
                <span>Weighted Words Subtotal:</span>
                <span class="font-mono">{currency.split(' ')[1]} {subtotalAmount.toFixed(2)}</span>
              </div>

              {#if minimumFee > subtotalAmount}
                <div class="flex justify-between text-amber-600 dark:text-amber-400">
                  <span>Minimum Fee Adjustment:</span>
                  <span class="font-mono">+{currency.split(' ')[1]} {(minimumFee - subtotalAmount).toFixed(2)}</span>
                </div>
              {/if}

              {#if taxRatePct > 0}
                <div class="flex justify-between text-gray-600 dark:text-gray-400">
                  <span>VAT / Tax ({taxRatePct}%):</span>
                  <span class="font-mono">{currency.split(' ')[1]} {taxAmount.toFixed(2)}</span>
                </div>
              {/if}

              <div class="border-t border-gray-300 dark:border-gray-700 pt-2 flex justify-between text-base font-bold text-gray-900 dark:text-white">
                <span>Total Amount Due:</span>
                <span class="font-mono text-emerald-600 dark:text-emerald-400">
                  {currency.split(' ')[1]} {grandTotal.toFixed(2)}
                </span>
              </div>
            </div>
          </div>

          <!-- Payment Terms / Notes -->
          <div class="border-t border-gray-200 dark:border-gray-800 pt-4 text-[11px] text-gray-500 space-y-1">
            <div class="font-semibold text-gray-700 dark:text-gray-300">Payment Terms & Notes:</div>
            <div>Payment is requested within 14 days of invoice issuance. Bank transfer details provided upon request.</div>
            <div class="italic">Generated with Free-CAT — 100% Offline, Zero Telemetry Translation Environment.</div>
          </div>
        {/if}
      </div>

      <!-- Footer (Hidden in print) -->
      <div class="no-print p-4 border-t border-gray-200 dark:border-gray-800 bg-gray-50 dark:bg-gray-850 flex justify-between items-center text-xs">
        <span class="text-gray-500">
          Raw words: <strong>{totalRawWords.toLocaleString()}</strong> | Weighted billable words: <strong>{Math.round(totalWeightedWords).toLocaleString()}</strong>
        </span>
        <button
          onclick={onClose}
          class="px-4 py-1.5 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 text-gray-800 dark:text-gray-200 rounded font-medium transition-colors"
        >
          Close
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  @media print {
    :global(body) {
      background: white !important;
      color: black !important;
    }
    .no-print {
      display: none !important;
    }
    .printable-document {
      padding: 0 !important;
      margin: 0 !important;
      overflow: visible !important;
      background: white !important;
      color: black !important;
    }
  }
</style>
