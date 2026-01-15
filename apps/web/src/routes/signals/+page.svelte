<script lang="ts">
  import { onMount } from 'svelte';
  import { ProgressRadial, TabGroup, Tab } from '@skeletonlabs/skeleton';
  import { SignalCard, SignalFilters, StockAnalysis } from '$lib/components';
  import type { BrokerSummary, TechnicalAnalysis, ValuationEstimate, OverallConclusion } from '$lib/components/StockAnalysis.types';

  interface Signal {
    id: string;
    symbol: string;
    stockName: string;
    type: 'buy' | 'sell' | 'hold';
    strength: 'strong' | 'moderate' | 'weak';
    score: number;
    reason: string;
    timestamp: Date;
    priceAtSignal: number;
    targetPrice?: number;
    stopLoss?: number;
    indicators: string[];
    sector: string;
    // Comprehensive analysis data
    brokerSummary?: BrokerSummary;
    technical?: TechnicalAnalysis;
    valuation?: ValuationEstimate;
    conclusion?: OverallConclusion;
  }

  interface Filters {
    type: 'all' | 'buy' | 'sell' | 'hold';
    strength: 'all' | 'strong' | 'moderate' | 'weak';
    minScore: number;
    sector: string;
  }

  let signals = $state<Signal[]>([]);
  let filteredSignals = $state<Signal[]>([]);
  let filters = $state<Filters>({ type: 'all', strength: 'all', minScore: 0, sector: '' });
  let isLoading = $state(true);
  let sectors = $state<string[]>([]);
  let selectedSignal = $state<Signal | null>(null);
  let language = $state<'en' | 'id'>('id');
  let viewTab = $state(0); // 0 = list, 1 = analysis

  onMount(async () => {
    await loadSignals();
  });

  async function loadSignals() {
    isLoading = true;
    
    // TODO: Replace with actual API call
    await new Promise(resolve => setTimeout(resolve, 500));
    
    signals = [
      {
        id: '1',
        symbol: 'BBCA',
        stockName: 'Bank Central Asia',
        type: 'buy',
        strength: 'strong',
        score: 82,
        reason: 'Strong accumulation pattern detected. Foreign net buy for 5 consecutive days. RSI oversold bounce.',
        timestamp: new Date(),
        priceAtSignal: 9250,
        targetPrice: 10000,
        stopLoss: 8800,
        indicators: ['EMA Bullish', 'RSI Oversold', 'MACD Cross', 'Volume Surge'],
        sector: 'Banking',
        brokerSummary: {
          bigBuyers: [{ code: 'BK', avgPrice: 9250 }, { code: 'KZ', avgPrice: 9200 }, { code: 'CS', avgPrice: 9180 }],
          bigSellers: [{ code: 'CC', avgPrice: 9280 }, { code: 'YP', avgPrice: 9300 }],
          netStatus: 'accumulation',
          priceRange: { low: 9100, high: 9350 }
        },
        technical: {
          lastPrice: 9250,
          rsi: 42,
          rsiSignal: 'neutral',
          macd: 15,
          macdSignal: 'positive',
          ichimoku: { position: 'above', cloudRange: { low: 8800, high: 9100 } },
          support: [9000, 8800, 8500],
          resistance: [9500, 9800, 10000],
          summary: { sell: 4, neutral: 8, buy: 14 }
        },
        valuation: {
          perValue: 8500,
          forwardEps: 14,
          pbvValue: 7800,
          bookValue: 2.8,
          evEbitdaValue: 7200,
          fairPriceRange: { low: 7200, high: 8500 },
          bullCase: { low: 10000, high: 11000 }
        },
        conclusion: {
          strengths: ['Strong net interest margin', 'Market leader position', 'Solid capital adequacy'],
          weaknesses: ['Premium valuation', 'Interest rate sensitivity'],
          strategy: {
            traders: 'Monitor support 9000, entry on bounce with target 9800',
            investors: 'Hold for long-term, strong fundamentals',
            valueInvestors: 'Wait for correction to fair value range'
          }
        }
      },
      {
        id: '2',
        symbol: 'TLKM',
        stockName: 'Telkom Indonesia',
        type: 'buy',
        strength: 'moderate',
        score: 68,
        reason: 'Breakout above resistance with increasing volume. Dividend yield attractive.',
        timestamp: new Date(Date.now() - 3600000),
        priceAtSignal: 4120,
        targetPrice: 4500,
        stopLoss: 3900,
        indicators: ['Breakout', 'Volume Increase', 'Fib Support'],
        sector: 'Telecoms',
        brokerSummary: {
          bigBuyers: [{ code: 'AK', avgPrice: 4100 }, { code: 'YU', avgPrice: 4080 }],
          bigSellers: [{ code: 'CC', avgPrice: 4150 }, { code: 'BK', avgPrice: 4140 }],
          netStatus: 'balanced',
          priceRange: { low: 4000, high: 4200 }
        },
        technical: {
          lastPrice: 4120,
          rsi: 55,
          rsiSignal: 'neutral',
          macd: 8,
          macdSignal: 'positive',
          ichimoku: { position: 'in', cloudRange: { low: 3950, high: 4180 } },
          support: [4000, 3850, 3700],
          resistance: [4300, 4500, 4800],
          summary: { sell: 6, neutral: 10, buy: 10 }
        }
      },
      {
        id: '3',
        symbol: 'ADRO',
        stockName: 'Adaro Energy',
        type: 'sell',
        strength: 'strong',
        score: 75,
        reason: 'Distribution pattern forming. Coal prices declining. Foreign selling accelerating.',
        timestamp: new Date(Date.now() - 7200000),
        priceAtSignal: 2450,
        targetPrice: 2100,
        stopLoss: 2600,
        indicators: ['EMA Bearish', 'Volume Distribution', 'Sector Weak'],
        sector: 'Mining',
        brokerSummary: {
          bigBuyers: [{ code: 'RX', avgPrice: 2420 }],
          bigSellers: [{ code: 'BK', avgPrice: 2480 }, { code: 'KZ', avgPrice: 2470 }, { code: 'CS', avgPrice: 2460 }],
          netStatus: 'distribution',
          priceRange: { low: 2380, high: 2500 }
        },
        technical: {
          lastPrice: 2450,
          rsi: 38,
          rsiSignal: 'neutral',
          macd: -25,
          macdSignal: 'negative',
          ichimoku: { position: 'below', cloudRange: { low: 2550, high: 2700 } },
          support: [2350, 2200, 2000],
          resistance: [2550, 2700, 2900],
          summary: { sell: 14, neutral: 6, buy: 6 }
        },
        conclusion: {
          strengths: ['Strong cash flow', 'Low production costs'],
          weaknesses: ['Coal price decline', 'ESG concerns', 'Foreign selling pressure'],
          strategy: {
            traders: 'Short opportunity, stop loss above 2600',
            investors: 'Reduce exposure, sector headwinds',
            valueInvestors: 'Avoid until commodity cycle turns'
          }
        }
      },
      {
        id: '4',
        symbol: 'ASII',
        stockName: 'Astra International',
        type: 'hold',
        strength: 'weak',
        score: 52,
        reason: 'Consolidating near support. Wait for clearer direction before entry.',
        timestamp: new Date(Date.now() - 10800000),
        priceAtSignal: 5425,
        indicators: ['Consolidation', 'Mixed Signals'],
        sector: 'Industrial',
        technical: {
          lastPrice: 5425,
          rsi: 48,
          rsiSignal: 'neutral',
          macd: -5,
          macdSignal: 'negative',
          ichimoku: { position: 'in', cloudRange: { low: 5300, high: 5600 } },
          support: [5200, 5000, 4800],
          resistance: [5600, 5800, 6000],
          summary: { sell: 8, neutral: 12, buy: 6 }
        }
      },
      {
        id: '5',
        symbol: 'BMRI',
        stockName: 'Bank Mandiri',
        type: 'buy',
        strength: 'moderate',
        score: 71,
        reason: 'Strong quarterly earnings. Net interest margin expansion. Government infrastructure spending beneficiary.',
        timestamp: new Date(Date.now() - 14400000),
        priceAtSignal: 6450,
        targetPrice: 7000,
        stopLoss: 6100,
        indicators: ['Earnings Beat', 'Fundamental Strong', 'Sector Leader'],
        sector: 'Banking',
        brokerSummary: {
          bigBuyers: [{ code: 'BK', avgPrice: 6420 }, { code: 'AK', avgPrice: 6400 }, { code: 'YU', avgPrice: 6380 }],
          bigSellers: [{ code: 'CC', avgPrice: 6480 }, { code: 'ZP', avgPrice: 6500 }],
          netStatus: 'accumulation',
          priceRange: { low: 6300, high: 6550 }
        },
        technical: {
          lastPrice: 6450,
          rsi: 58,
          rsiSignal: 'neutral',
          macd: 22,
          macdSignal: 'positive',
          ichimoku: { position: 'above', cloudRange: { low: 6100, high: 6350 } },
          support: [6200, 6000, 5800],
          resistance: [6700, 7000, 7300],
          summary: { sell: 5, neutral: 9, buy: 12 }
        },
        valuation: {
          perValue: 5800,
          forwardEps: 12,
          pbvValue: 5200,
          bookValue: 2.2,
          evEbitdaValue: 4800,
          fairPriceRange: { low: 4800, high: 5800 },
          bullCase: { low: 7000, high: 8000 }
        },
        conclusion: {
          strengths: ['Earnings growth', 'Government backing', 'Infrastructure exposure'],
          weaknesses: ['Economic sensitivity', 'NPL risk'],
          strategy: {
            traders: 'Buy on dips to 6200 support',
            investors: 'Accumulate, solid long-term outlook',
            valueInvestors: 'Consider at fair value levels'
          }
        }
      },
    ];
    
    // Extract unique sectors
    sectors = [...new Set(signals.map(s => s.sector))].sort();
    
    applyFilters();
    isLoading = false;
  }

  function applyFilters() {
    filteredSignals = signals.filter(signal => {
      if (filters.type !== 'all' && signal.type !== filters.type) return false;
      if (filters.strength !== 'all' && signal.strength !== filters.strength) return false;
      if (signal.score < filters.minScore) return false;
      if (filters.sector && signal.sector !== filters.sector) return false;
      return true;
    });
  }

  function selectSignal(signal: Signal) {
    selectedSignal = signal;
    viewTab = 1;
  }

  // Reactively apply filters when they change
  $effect(() => {
    applyFilters();
  });

  // Stats
  let stats = $derived({
    total: signals.length,
    buySignals: signals.filter(s => s.type === 'buy').length,
    sellSignals: signals.filter(s => s.type === 'sell').length,
    strongSignals: signals.filter(s => s.strength === 'strong').length,
  });
</script>

<svelte:head>
  <title>Signals - JejakCuan</title>
</svelte:head>

<div class="space-y-6">
  <div class="flex items-center justify-between flex-wrap gap-4">
    <h1 class="h1 text-slate-900 dark:text-slate-100">Today's Signals</h1>
    <div class="flex items-center gap-4">
      <!-- Language Toggle -->
      <div class="flex items-center gap-2">
        <button 
          onclick={() => language = 'id'}
          class="btn btn-sm {language === 'id' ? 'variant-filled-primary' : 'variant-ghost-surface'}"
        >
          🇮🇩 ID
        </button>
        <button 
          onclick={() => language = 'en'}
          class="btn btn-sm {language === 'en' ? 'variant-filled-primary' : 'variant-ghost-surface'}"
        >
          🇺🇸 EN
        </button>
      </div>
      <button 
        onclick={loadSignals}
        class="btn variant-ghost-primary"
        disabled={isLoading}
      >
        {#if isLoading}
          <ProgressRadial width="w-5" stroke={100} meter="stroke-primary-500" track="stroke-primary-500/30" />
          <span>Loading...</span>
        {:else}
          Refresh
        {/if}
      </button>
    </div>
  </div>

  <!-- Signal Stats -->
  <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
    <div class="card p-4 text-center">
      <p class="text-2xl font-bold text-slate-900 dark:text-slate-100">{stats.total}</p>
      <p class="text-sm text-slate-600 dark:text-slate-400">Total Signals</p>
    </div>
    <div class="card p-4 text-center">
      <p class="text-2xl font-bold text-emerald-600 dark:text-emerald-400">{stats.buySignals}</p>
      <p class="text-sm text-slate-600 dark:text-slate-400">Buy Signals</p>
    </div>
    <div class="card p-4 text-center">
      <p class="text-2xl font-bold text-rose-600 dark:text-rose-400">{stats.sellSignals}</p>
      <p class="text-sm text-slate-600 dark:text-slate-400">Sell Signals</p>
    </div>
    <div class="card p-4 text-center">
      <p class="text-2xl font-bold text-blue-600 dark:text-blue-400">{stats.strongSignals}</p>
      <p class="text-sm text-slate-600 dark:text-slate-400">Strong Signals</p>
    </div>
  </div>

  <!-- Tab Navigation -->
  <TabGroup>
    <Tab bind:group={viewTab} name="list" value={0}>📋 {language === 'id' ? 'Daftar Signal' : 'Signal List'}</Tab>
    <Tab bind:group={viewTab} name="analysis" value={1} disabled={!selectedSignal}>
      📊 {language === 'id' ? 'Analisis Detail' : 'Detailed Analysis'}
      {#if selectedSignal}
        <span class="badge variant-soft-primary ml-2">{selectedSignal.symbol}</span>
      {/if}
    </Tab>
  </TabGroup>

  {#if viewTab === 0}
    <!-- Filters -->
    <SignalFilters bind:filters {sectors} onApply={applyFilters} />

    <!-- Signal List -->
    {#if isLoading}
      <div class="flex items-center justify-center p-8">
        <ProgressRadial stroke={100} meter="stroke-primary-500" track="stroke-primary-500/30" />
      </div>
    {:else if filteredSignals.length === 0}
      <div class="card p-8 text-center">
        <p class="text-slate-600 dark:text-slate-400">
          {language === 'id' ? 'Tidak ada signal yang sesuai filter.' : 'No signals match your filters.'}
        </p>
        <button 
          onclick={() => { filters = { type: 'all', strength: 'all', minScore: 0, sector: '' }; }}
          class="btn variant-ghost-primary mt-4"
        >
          {language === 'id' ? 'Hapus Filter' : 'Clear Filters'}
        </button>
      </div>
    {:else}
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        {#each filteredSignals as signal (signal.id)}
          <div 
            class="cursor-pointer transition-all hover:ring-2 hover:ring-primary-500/50 rounded-lg"
            onclick={() => selectSignal(signal)}
            onkeypress={(e) => e.key === 'Enter' && selectSignal(signal)}
            role="button"
            tabindex="0"
          >
            <SignalCard {signal} />
          </div>
        {/each}
      </div>
    {/if}
  {:else if viewTab === 1 && selectedSignal}
    <!-- Detailed Analysis View -->
    <div class="space-y-4">
      <div class="flex items-center justify-between">
        <div>
          <button 
            onclick={() => viewTab = 0}
            class="anchor-high-contrast text-sm"
          >
            &larr; {language === 'id' ? 'Kembali ke Daftar' : 'Back to List'}
          </button>
          <h2 class="h2 mt-2 text-slate-900 dark:text-slate-100">
            {selectedSignal.symbol} - {selectedSignal.stockName}
          </h2>
          <p class="text-slate-600 dark:text-slate-400">
            {selectedSignal.sector} | {language === 'id' ? 'Harga' : 'Price'}: {selectedSignal.priceAtSignal.toLocaleString('id-ID')} IDR
          </p>
        </div>
        <div class="flex items-center gap-2">
          <span class="{selectedSignal.type === 'buy' ? 'signal-buy' : selectedSignal.type === 'sell' ? 'signal-sell' : 'signal-hold'}">
            {selectedSignal.type.toUpperCase()}
          </span>
          <span class="badge variant-soft text-slate-900 dark:text-slate-100">{selectedSignal.strength}</span>
          <span class="badge variant-filled-primary">{selectedSignal.score}/100</span>
        </div>
      </div>

      <!-- Signal Summary Card -->
      <div class="card p-4">
        <h3 class="h4 mb-2 text-slate-900 dark:text-slate-100">
          {language === 'id' ? '📌 Ringkasan Signal' : '📌 Signal Summary'}
        </h3>
        <p class="text-slate-600 dark:text-slate-400">{selectedSignal.reason}</p>
        <div class="flex flex-wrap gap-2 mt-3">
          {#each selectedSignal.indicators as indicator}
            <span class="badge variant-soft-secondary">{indicator}</span>
          {/each}
        </div>
        {#if selectedSignal.targetPrice || selectedSignal.stopLoss}
          <div class="grid grid-cols-2 gap-4 mt-4">
            {#if selectedSignal.targetPrice}
              <div class="p-2 bg-emerald-50 dark:bg-emerald-900/30 rounded">
                <span class="text-sm text-slate-600 dark:text-slate-400">
                  {language === 'id' ? 'Target' : 'Target'}:
                </span>
                <span class="font-bold text-emerald-700 dark:text-emerald-400 ml-2">
                  {selectedSignal.targetPrice.toLocaleString('id-ID')} IDR
                </span>
              </div>
            {/if}
            {#if selectedSignal.stopLoss}
              <div class="p-2 bg-rose-50 dark:bg-rose-900/30 rounded">
                <span class="text-sm text-slate-600 dark:text-slate-400">
                  {language === 'id' ? 'Stop Loss' : 'Stop Loss'}:
                </span>
                <span class="font-bold text-rose-700 dark:text-rose-400 ml-2">
                  {selectedSignal.stopLoss.toLocaleString('id-ID')} IDR
                </span>
              </div>
            {/if}
          </div>
        {/if}
      </div>

      <!-- Comprehensive Analysis -->
      <StockAnalysis 
        symbol={selectedSignal.symbol}
        brokerSummary={selectedSignal.brokerSummary}
        technical={selectedSignal.technical}
        valuation={selectedSignal.valuation}
        conclusion={selectedSignal.conclusion}
        {language}
      />
    </div>
  {/if}

  <!-- Disclaimer -->
  <div class="card variant-soft-warning p-4">
    <p class="text-sm text-amber-800 dark:text-amber-300">
      <strong>Disclaimer:</strong> 
      {language === 'id' 
        ? 'Sinyal ini dihasilkan oleh analisis algoritma dan tidak boleh dianggap sebagai saran keuangan. Selalu lakukan riset sendiri sebelum membuat keputusan investasi.'
        : 'These signals are generated by algorithmic analysis and should not be considered as financial advice. Always do your own research before making investment decisions.'}
    </p>
  </div>
</div>
