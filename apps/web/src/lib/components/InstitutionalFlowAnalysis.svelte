<script lang="ts">
  import type { InstitutionalFlowAnalysis, AccumulatorInfo } from './StockAnalysis.types';

  let { 
    analysis = null as InstitutionalFlowAnalysis | null,
    language = 'en' as 'en' | 'id'
  } = $props();

  const t = {
    en: {
      title: 'Institutional Flow Analysis',
      accumulationScore: 'Accumulation Score',
      signalStrength: 'Signal Strength',
      strong: 'Strong Accumulation',
      moderate: 'Moderate Accumulation',
      weak: 'Weak Signal',
      neutral: 'Neutral',
      distribution: 'Distribution',
      coordinatedBuying: 'Coordinated Buying Detected',
      coordinatedBuyingDesc: 'Multiple institutional brokers acting together',
      daysAccumulated: 'Days Net Buying',
      rolling5Day: '5-Day Rolling',
      rolling20Day: '20-Day Rolling',
      totalNet: 'Total Net',
      institutionalNet: 'Institutional Net',
      foreignNet: 'Foreign Net',
      topAccumulators: 'Top Institutional Accumulators',
      broker: 'Broker',
      category: 'Category',
      netValue: 'Net Value',
      netVolume: 'Net Volume',
      noData: 'No bandar analysis data available',
      foreign: 'Foreign',
      local: 'Local',
      signalDescription: 'Analysis'
    },
    id: {
      title: 'Analisis Bandar (Arus Institusional)',
      accumulationScore: 'Skor Akumulasi',
      signalStrength: 'Kekuatan Sinyal',
      strong: 'Akumulasi Kuat',
      moderate: 'Akumulasi Moderat',
      weak: 'Sinyal Lemah',
      neutral: 'Netral',
      distribution: 'Distribusi',
      coordinatedBuying: 'Pembelian Terkoordinasi Terdeteksi',
      coordinatedBuyingDesc: 'Beberapa broker institusional bertindak bersama',
      daysAccumulated: 'Hari Net Buying',
      rolling5Day: '5 Hari Bergulir',
      rolling20Day: '20 Hari Bergulir',
      totalNet: 'Total Net',
      institutionalNet: 'Net Institusional',
      foreignNet: 'Net Asing',
      topAccumulators: 'Top Akumulator Institusional',
      broker: 'Broker',
      category: 'Kategori',
      netValue: 'Nilai Net',
      netVolume: 'Volume Net',
      noData: 'Data analisis institusional tidak tersedia',
      foreign: 'Asing',
      local: 'Lokal',
      signalDescription: 'Analisis'
    }
  };

  let labels = $derived(t[language]);

  function formatIdrCompact(value: number): string {
    const abs = Math.abs(value);
    const sign = value < 0 ? '-' : '+';
    if (abs >= 1e12) return `${sign}Rp${(abs / 1e12).toFixed(2)}T`;
    if (abs >= 1e9) return `${sign}Rp${(abs / 1e9).toFixed(2)}B`;
    if (abs >= 1e6) return `${sign}Rp${(abs / 1e6).toFixed(2)}M`;
    return `${sign}Rp${abs.toLocaleString('id-ID')}`;
  }

  function formatVolume(value: number): string {
    const abs = Math.abs(value);
    const sign = value < 0 ? '-' : '+';
    if (abs >= 1e9) return `${sign}${(abs / 1e9).toFixed(2)}B`;
    if (abs >= 1e6) return `${sign}${(abs / 1e6).toFixed(2)}M`;
    if (abs >= 1e3) return `${sign}${(abs / 1e3).toFixed(1)}K`;
    return `${sign}${abs.toLocaleString('id-ID')}`;
  }

  function getSignalLabel(strength: string): string {
    switch (strength) {
      case 'strong': return labels.strong;
      case 'moderate': return labels.moderate;
      case 'weak': return labels.weak;
      case 'distribution': return labels.distribution;
      default: return labels.neutral;
    }
  }

  function getSignalColor(strength: string): string {
    switch (strength) {
      case 'strong': return 'text-emerald-600 dark:text-emerald-400';
      case 'moderate': return 'text-teal-600 dark:text-teal-400';
      case 'weak': return 'text-amber-600 dark:text-amber-400';
      case 'distribution': return 'text-rose-600 dark:text-rose-400';
      default: return 'text-slate-600 dark:text-slate-400';
    }
  }

  function getScoreColor(score: number): string {
    if (score >= 75) return 'bg-emerald-500';
    if (score >= 60) return 'bg-teal-500';
    if (score >= 40) return 'bg-amber-500';
    if (score >= 25) return 'bg-orange-500';
    return 'bg-rose-500';
  }

  function getCategoryLabel(category: string): string {
    if (category === 'foreign_institutional') return labels.foreign;
    if (category === 'local_institutional') return labels.local;
    return category;
  }
</script>

{#if analysis}
  <div class="card p-4 space-y-4">
    <h3 class="h4 text-slate-900 dark:text-slate-100 flex items-center gap-2">
      <span class="text-xl">🏦</span>
      {labels.title}
    </h3>

    <!-- Score Gauge and Signal Strength -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <!-- Accumulation Score -->
      <div class="p-4 bg-slate-50 dark:bg-slate-800 rounded-lg">
        <div class="flex items-center justify-between mb-2">
          <span class="text-sm font-medium text-slate-600 dark:text-slate-400">{labels.accumulationScore}</span>
          <span class="text-2xl font-bold {getSignalColor(analysis.signalStrength)}">
            {analysis.accumulationScore.toFixed(0)}
          </span>
        </div>
        <!-- Score bar -->
        <div class="w-full h-3 bg-slate-200 dark:bg-slate-700 rounded-full overflow-hidden">
          <div 
            class="h-full {getScoreColor(analysis.accumulationScore)} transition-all duration-500"
            style="width: {analysis.accumulationScore}%"
          ></div>
        </div>
        <div class="flex justify-between text-xs text-slate-500 mt-1">
          <span>0</span>
          <span>50</span>
          <span>100</span>
        </div>
      </div>

      <!-- Signal Strength -->
      <div class="p-4 bg-slate-50 dark:bg-slate-800 rounded-lg">
        <div class="flex items-center justify-between mb-2">
          <span class="text-sm font-medium text-slate-600 dark:text-slate-400">{labels.signalStrength}</span>
        </div>
        <div class="flex items-center gap-3">
          <span class="text-lg font-bold {getSignalColor(analysis.signalStrength)}">
            {getSignalLabel(analysis.signalStrength)}
          </span>
          {#if analysis.coordinatedBuying}
            <span class="badge variant-soft-warning text-xs">
              {labels.coordinatedBuying}
            </span>
          {/if}
        </div>
        <div class="mt-2 text-sm text-slate-600 dark:text-slate-400">
          <span class="font-medium">{labels.daysAccumulated}:</span>
          <span class="ml-1 {analysis.daysAccumulated >= 3 ? 'text-emerald-600 dark:text-emerald-400' : 'text-slate-600 dark:text-slate-400'}">
            {analysis.daysAccumulated} / 5
          </span>
        </div>
      </div>
    </div>

    <!-- Signal Description -->
    <div class="p-3 bg-gradient-to-r from-slate-100 to-slate-50 dark:from-slate-800 dark:to-slate-900 rounded-lg border-l-4 {analysis.isAccumulating ? 'border-emerald-500' : analysis.signalStrength === 'distribution' ? 'border-rose-500' : 'border-amber-500'}">
      <div class="flex items-start gap-2">
        <span class="text-lg">{analysis.isAccumulating ? '📈' : analysis.signalStrength === 'distribution' ? '📉' : '📊'}</span>
        <div>
          <span class="font-medium text-slate-700 dark:text-slate-300">{labels.signalDescription}:</span>
          <p class="text-sm text-slate-600 dark:text-slate-400 mt-1">{analysis.signalDescription}</p>
        </div>
      </div>
    </div>

    <!-- Rolling Flow Comparison -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <!-- 5-Day Rolling -->
      <div class="p-3 bg-slate-50 dark:bg-slate-800 rounded-lg">
        <h4 class="text-sm font-semibold text-slate-700 dark:text-slate-300 mb-3">{labels.rolling5Day}</h4>
        <div class="space-y-2 text-sm">
          <div class="flex justify-between">
            <span class="text-slate-600 dark:text-slate-400">{labels.totalNet}</span>
            <span class="font-medium {analysis.net5Day >= 0 ? 'text-emerald-600 dark:text-emerald-400' : 'text-rose-600 dark:text-rose-400'}">
              {formatIdrCompact(analysis.net5Day)}
            </span>
          </div>
          <div class="flex justify-between">
            <span class="text-slate-600 dark:text-slate-400">{labels.institutionalNet}</span>
            <span class="font-medium {analysis.institutionalNet5Day >= 0 ? 'text-emerald-600 dark:text-emerald-400' : 'text-rose-600 dark:text-rose-400'}">
              {formatIdrCompact(analysis.institutionalNet5Day)}
            </span>
          </div>
          <div class="flex justify-between">
            <span class="text-slate-600 dark:text-slate-400">{labels.foreignNet}</span>
            <span class="font-medium {analysis.foreignNet5Day >= 0 ? 'text-emerald-600 dark:text-emerald-400' : 'text-rose-600 dark:text-rose-400'}">
              {formatIdrCompact(analysis.foreignNet5Day)}
            </span>
          </div>
        </div>
      </div>

      <!-- 20-Day Rolling -->
      <div class="p-3 bg-slate-50 dark:bg-slate-800 rounded-lg">
        <h4 class="text-sm font-semibold text-slate-700 dark:text-slate-300 mb-3">{labels.rolling20Day}</h4>
        <div class="space-y-2 text-sm">
          <div class="flex justify-between">
            <span class="text-slate-600 dark:text-slate-400">{labels.totalNet}</span>
            <span class="font-medium {analysis.net20Day >= 0 ? 'text-emerald-600 dark:text-emerald-400' : 'text-rose-600 dark:text-rose-400'}">
              {formatIdrCompact(analysis.net20Day)}
            </span>
          </div>
          <div class="flex justify-between">
            <span class="text-slate-600 dark:text-slate-400">{labels.institutionalNet}</span>
            <span class="font-medium {analysis.institutionalNet20Day >= 0 ? 'text-emerald-600 dark:text-emerald-400' : 'text-rose-600 dark:text-rose-400'}">
              {formatIdrCompact(analysis.institutionalNet20Day)}
            </span>
          </div>
          <div class="flex justify-between">
            <span class="text-slate-600 dark:text-slate-400">{labels.foreignNet}</span>
            <span class="font-medium {analysis.foreignNet20Day >= 0 ? 'text-emerald-600 dark:text-emerald-400' : 'text-rose-600 dark:text-rose-400'}">
              {formatIdrCompact(analysis.foreignNet20Day)}
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- Top Accumulators Table -->
    {#if analysis.topAccumulators && analysis.topAccumulators.length > 0}
      <div class="overflow-x-auto">
        <h4 class="text-sm font-semibold text-slate-700 dark:text-slate-300 mb-2">{labels.topAccumulators}</h4>
        <table class="w-full text-sm">
          <thead>
            <tr class="text-left text-slate-600 dark:text-slate-400 border-b border-slate-200 dark:border-slate-700">
              <th class="py-2 px-2">{labels.broker}</th>
              <th class="py-2 px-2">{labels.category}</th>
              <th class="py-2 px-2 text-right">{labels.netValue}</th>
              <th class="py-2 px-2 text-right">{labels.netVolume}</th>
            </tr>
          </thead>
          <tbody>
            {#each analysis.topAccumulators as acc (acc.brokerCode)}
              <tr class="border-b border-slate-100 dark:border-slate-800 hover:bg-slate-50 dark:hover:bg-slate-800/50">
                <td class="py-2 px-2">
                  <div class="flex items-center gap-2">
                    <span class="font-medium text-slate-900 dark:text-slate-100">{acc.brokerCode}</span>
                    {#if acc.brokerName}
                      <span class="text-slate-500 dark:text-slate-500 text-xs">{acc.brokerName}</span>
                    {/if}
                    {#if acc.isForeign}
                      <span class="badge variant-soft-primary text-xs">F</span>
                    {/if}
                  </div>
                </td>
                <td class="py-2 px-2">
                  <span class="badge {acc.isForeign ? 'variant-soft-primary' : 'variant-soft-secondary'} text-xs">
                    {getCategoryLabel(acc.category)}
                  </span>
                </td>
                <td class="py-2 px-2 text-right font-medium text-emerald-600 dark:text-emerald-400">
                  {formatIdrCompact(acc.netValue)}
                </td>
                <td class="py-2 px-2 text-right text-slate-600 dark:text-slate-400">
                  {formatVolume(acc.netVolume)}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </div>
{:else}
  <div class="card p-6 text-center">
    <p class="text-slate-500 dark:text-slate-400">{labels.noData}</p>
  </div>
{/if}
