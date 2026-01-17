<script lang="ts">
  import { onMount } from 'svelte';
  import { ProgressRadial, TabGroup, Tab } from '@skeletonlabs/skeleton';
  import { SignalCard, SignalFilters, StockAnalysis } from '$lib/components';
  import { api } from '$lib/api';
  import type { Stock, StockFreshness, StockScore, FullAnalysisResponse } from '$lib/api';
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
    freshness?: StockFreshness;
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
  let loadingAnalysis = $state<string | null>(null);
  let sectors = $state<string[]>([]);
  let selectedSignal = $state<Signal | null>(null);
  let language = $state<'en' | 'id'>('id');
  let viewTab = $state(0); // 0 = list, 1 = analysis
  let errorMessage = $state<string | null>(null);
  let analysisError = $state<string | null>(null);

  onMount(async () => {
    await loadSignals();
  });

  // Convert API response to component types
  function convertBrokerSummary(data: FullAnalysisResponse['broker_summary']): BrokerSummary | undefined {
    if (!data || (data.big_buyers.length === 0 && data.big_sellers.length === 0)) return undefined;
    return {
      bigBuyers: data.big_buyers.map(b => ({
        code: b.code,
        name: b.name,
        category: b.category,
        avgPrice: b.avg_price,
        buyVolume: b.buy_volume,
        sellVolume: b.sell_volume,
        netVolume: b.net_volume,
        buyValue: b.buy_value,
        sellValue: b.sell_value,
        netValue: b.net_value,
      })),
      bigSellers: data.big_sellers.map(b => ({
        code: b.code,
        name: b.name,
        category: b.category,
        avgPrice: b.avg_price,
        buyVolume: b.buy_volume,
        sellVolume: b.sell_volume,
        netVolume: b.net_volume,
        buyValue: b.buy_value,
        sellValue: b.sell_value,
        netValue: b.net_value,
      })),
      netStatus: data.net_status as 'accumulation' | 'distribution' | 'balanced',
      priceRange: { low: data.price_range.low, high: data.price_range.high },
      foreignNet: data.foreign_net,
      domesticNet: data.domestic_net,
    };
  }

  function convertTechnical(data: FullAnalysisResponse['technical']): TechnicalAnalysis | undefined {
    if (!data) return undefined;
    return {
      lastPrice: data.last_price,
      rsi: data.rsi,
      rsiSignal: data.rsi_signal as 'oversold' | 'neutral' | 'overbought',
      macd: data.macd,
      macdSignal: data.macd_signal.includes('bullish') ? 'positive' : 'negative',
      ichimoku: {
        position: data.ichimoku.position as 'above' | 'in' | 'below',
        cloudRange: { low: data.ichimoku.cloud_range.low, high: data.ichimoku.cloud_range.high }
      },
      support: data.support,
      resistance: data.resistance,
      summary: data.summary
    };
  }

  function convertValuation(data: FullAnalysisResponse['valuation']): ValuationEstimate | undefined {
    if (!data) return undefined;
    return {
      perValue: data.per_value,
      forwardEps: data.forward_eps,
      pbvValue: data.pbv_value,
      bookValue: data.book_value,
      evEbitdaValue: data.ev_ebitda_value,
      fairPriceRange: { low: data.fair_price_range.low, high: data.fair_price_range.high },
      bullCase: { low: data.bull_case.low, high: data.bull_case.high }
    };
  }

  function convertConclusion(data: FullAnalysisResponse['conclusion']): OverallConclusion | undefined {
    if (!data) return undefined;
    return {
      strengths: data.strengths,
      weaknesses: data.weaknesses,
      strategy: {
        traders: data.strategy.traders,
        investors: data.strategy.investors,
        valueInvestors: data.strategy.value_investors
      }
    };
  }

  function determineSignalType(
    score: number,
    technical: TechnicalAnalysis | undefined,
    brokerSummary?: BrokerSummary
  ): 'buy' | 'sell' | 'hold' {
    // Order flow first
    if (brokerSummary) {
      if (brokerSummary.netStatus === 'accumulation' && score >= 50) return 'buy';
      if (brokerSummary.netStatus === 'distribution' && score <= 60) return 'sell';
    }

    if (score >= 70) return 'buy';
    if (score <= 40) return 'sell';
    if (technical) {
      const { summary } = technical;
      if (summary.buy > summary.sell + 3) return 'buy';
      if (summary.sell > summary.buy + 3) return 'sell';
    }
    return 'hold';
  }

  function determineStrength(score: number): 'strong' | 'moderate' | 'weak' {
    if (score >= 80 || score <= 25) return 'strong';
    if (score >= 65 || score <= 40) return 'moderate';
    return 'weak';
  }

  function generateReason(
    technical: TechnicalAnalysis | undefined,
    signalType: 'buy' | 'sell' | 'hold',
    brokerSummary?: BrokerSummary
  ): string {
    const reasons: string[] = [];

    if (brokerSummary) {
      const buyers = brokerSummary.bigBuyers.slice(0, 3).map(b => b.code).join(', ');
      const sellers = brokerSummary.bigSellers.slice(0, 3).map(b => b.code).join(', ');
      if (brokerSummary.netStatus === 'accumulation') {
        reasons.push(
          language === 'id'
            ? `Order flow: akumulasi (${buyers || '-'}).`
            : `Order flow: accumulation (${buyers || '-'}).`
        );
      } else if (brokerSummary.netStatus === 'distribution') {
        reasons.push(
          language === 'id'
            ? `Order flow: distribusi (${sellers || '-'}).`
            : `Order flow: distribution (${sellers || '-'}).`
        );
      } else {
        reasons.push(
          language === 'id'
            ? `Order flow: seimbang (buy: ${buyers || '-'} | sell: ${sellers || '-'}).`
            : `Order flow: balanced (buy: ${buyers || '-'} | sell: ${sellers || '-'}).`
        );
      }
    }

    if (!technical) {
      return reasons.length > 0
        ? reasons.join(' ')
        : (language === 'id' ? 'Data teknikal tidak tersedia.' : 'Technical analysis data unavailable.');
    }
    
    // RSI
    if (technical.rsi <= 30) {
      reasons.push('RSI oversold bounce opportunity');
    } else if (technical.rsi >= 70) {
      reasons.push('RSI overbought warning');
    }
    
    // MACD
    if (technical.macdSignal === 'positive') {
      reasons.push('MACD bullish momentum');
    } else {
      reasons.push('MACD bearish momentum');
    }
    
    // Ichimoku
    if (technical.ichimoku.position === 'above') {
      reasons.push('Price above Ichimoku cloud');
    } else if (technical.ichimoku.position === 'below') {
      reasons.push('Price below Ichimoku cloud');
    }
    
    // TA Summary
    if (technical.summary.buy > technical.summary.sell) {
      reasons.push(`${technical.summary.buy} buy signals vs ${technical.summary.sell} sell signals`);
    } else if (technical.summary.sell > technical.summary.buy) {
      reasons.push(`${technical.summary.sell} sell signals vs ${technical.summary.buy} buy signals`);
    }
    
    return reasons.length > 0 ? reasons.join(' ') : (language === 'id' ? 'Sinyal campuran, pantau.' : 'Mixed signals, monitor closely.');
  }

  function generateIndicators(technical: TechnicalAnalysis | undefined, brokerSummary?: BrokerSummary): string[] {
    const indicators: string[] = [];

    if (brokerSummary) {
      indicators.push(`Order Flow: ${brokerSummary.netStatus}`);
    }

    if (!technical) return indicators;
    
    if (technical.rsi <= 30) indicators.push('RSI Oversold');
    else if (technical.rsi >= 70) indicators.push('RSI Overbought');
    else indicators.push('RSI Neutral');
    
    if (technical.macdSignal === 'positive') indicators.push('MACD Bullish');
    else indicators.push('MACD Bearish');
    
    if (technical.ichimoku.position === 'above') indicators.push('Ichimoku Bullish');
    else if (technical.ichimoku.position === 'below') indicators.push('Ichimoku Bearish');
    else indicators.push('Ichimoku Neutral');
    
    return indicators;
  }

  async function loadSignals() {
    isLoading = true;
    errorMessage = null;
    
    try {
      const watchlist = await api.getWatchlist().catch(() => []);
      const watchlistSymbols = new Set(watchlist.map((w) => w.symbol.toUpperCase()));

      // First, get top stocks with scores
      let scores = await api.getTopScores(100);

      // If scores are missing, trigger a recompute to avoid placeholder signals.
      if (scores.length === 0) {
        await api.recomputeScores().catch(() => undefined);
        scores = await api.getTopScores(100);
      }

      // Exclude symbols already in watchlist (signals = new opportunities)
      scores = scores.filter((s) => !watchlistSymbols.has(s.symbol.toUpperCase())).slice(0, 20);
      
      if (scores.length === 0) {
        // Fallback: get some stocks without scores
        const { stocks } = await api.getStocks(undefined, 10);
        const filteredStocks = stocks.filter((s) => !watchlistSymbols.has(s.symbol.toUpperCase()));
        
        // Create placeholder signals for stocks without scores
        signals = filteredStocks.map((stock, index) => ({
          id: `${stock.symbol}-${index}`,
          symbol: stock.symbol,
          stockName: stock.name,
          type: 'hold' as const,
          strength: 'weak' as const,
          score: 50,
          reason: 'Awaiting analysis data. Click to load detailed analysis.',
          timestamp: new Date(),
          priceAtSignal: 0,
          indicators: [],
          sector: stock.sector || 'Unknown'
        }));
      } else {
        // Create signals from scores
        signals = await Promise.all(scores.map(async (score, index) => {
          try {
            const stock = await api.getStock(score.symbol);
            const compositeScore = Math.round(score.composite_score);
            
            return {
              id: `${score.symbol}-${index}`,
              symbol: score.symbol,
              stockName: stock.name,
              type: determineSignalType(compositeScore, undefined),
              strength: determineStrength(compositeScore),
              score: compositeScore,
              reason: `Composite score: ${compositeScore}. Technical: ${Math.round(score.technical_score)}, Fundamental: ${Math.round(score.fundamental_score)}. Click to view detailed analysis.`,
              timestamp: new Date(score.time),
              priceAtSignal: 0,
              indicators: ['Technical Analysis', 'Fundamental Analysis', 'Sentiment Analysis'],
              sector: stock.sector || 'Unknown'
            } as Signal;
          } catch (err) {
            return {
              id: `${score.symbol}-${index}`,
              symbol: score.symbol,
              stockName: score.symbol,
              type: determineSignalType(Math.round(score.composite_score), undefined),
              strength: determineStrength(Math.round(score.composite_score)),
              score: Math.round(score.composite_score),
              reason: `Score: ${Math.round(score.composite_score)}. Click to view analysis.`,
              timestamp: new Date(score.time),
              priceAtSignal: 0,
              indicators: [],
              sector: 'Unknown'
            } as Signal;
          }
        }));
      }
      
      // Extract unique sectors
      sectors = [...new Set(signals.map(s => s.sector).filter(s => s !== 'Unknown'))].sort();
      
      applyFilters();
    } catch (err) {
      console.error('Failed to load signals:', err);
      errorMessage = err instanceof Error ? err.message : 'Failed to load signals';
    } finally {
      isLoading = false;
    }
  }

  async function loadAnalysisForSignal(signal: Signal): Promise<Signal> {
    try {
      loadingAnalysis = signal.symbol;
      analysisError = null;
      const [analysis, freshness] = await Promise.all([
        api.getFullAnalysis(signal.symbol),
        api.getStockFreshness(signal.symbol).catch(() => null)
      ]);

      if (!analysis) {
        analysisError = language === 'id'
          ? 'Data analisis tidak tersedia untuk saham ini.'
          : 'Analysis data is not available for this stock.';
        return signal;
      }

      const technical = convertTechnical(analysis.technical);
      const brokerSummary = convertBrokerSummary(analysis.broker_summary);
      const signalType = determineSignalType(signal.score, technical, brokerSummary);
      
      return {
        ...signal,
        stockName: analysis.name || signal.stockName,
        type: signalType,
        priceAtSignal: analysis.technical?.last_price || 0,
        reason: generateReason(technical, signalType, brokerSummary),
        indicators: generateIndicators(technical, brokerSummary),
        targetPrice: technical ? technical.resistance[0] : undefined,
        stopLoss: technical ? technical.support[0] : undefined,
        brokerSummary,
        technical: technical,
        valuation: convertValuation(analysis.valuation),
        conclusion: convertConclusion(analysis.conclusion),
        freshness: freshness ?? undefined
      };
    } catch (err) {
      console.error(`Failed to load analysis for ${signal.symbol}:`, err);
      analysisError = err instanceof Error ? err.message : 'Failed to load analysis';
      return signal;
    } finally {
      loadingAnalysis = null;
    }
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

  async function selectSignal(signal: Signal) {
    analysisError = null;
    // Load detailed analysis if not already loaded
    if (!signal.technical) {
      const updatedSignal = await loadAnalysisForSignal(signal);
      // Update the signal in the list
      const idx = signals.findIndex(s => s.id === signal.id);
      if (idx >= 0) {
        signals[idx] = updatedSignal;
        signals = [...signals]; // Trigger reactivity
      }
      selectedSignal = updatedSignal;
    } else {
      selectedSignal = signal;
    }
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

  {#if errorMessage}
    <div class="card variant-soft-error p-4">
      <p class="text-rose-700 dark:text-rose-300">
        <strong>Error:</strong> {errorMessage}
      </p>
      <p class="text-sm text-slate-600 dark:text-slate-400 mt-2">
        {language === 'id' 
          ? 'Pastikan API server berjalan dan Anda sudah login.'
          : 'Make sure the API server is running and you are logged in.'}
      </p>
    </div>
  {/if}

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
            class="cursor-pointer transition-all hover:ring-2 hover:ring-primary-500/50 rounded-lg relative"
            onclick={() => selectSignal(signal)}
            onkeypress={(e) => e.key === 'Enter' && selectSignal(signal)}
            role="button"
            tabindex="0"
          >
            {#if loadingAnalysis === signal.symbol}
              <div class="absolute inset-0 bg-surface-900/30 rounded-lg flex items-center justify-center z-10">
                <ProgressRadial width="w-8" stroke={100} meter="stroke-primary-500" track="stroke-primary-500/30" />
              </div>
            {/if}
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
            {selectedSignal.sector} | {language === 'id' ? 'Harga' : 'Price'}: {selectedSignal.priceAtSignal > 0 ? selectedSignal.priceAtSignal.toLocaleString('id-ID') : 'N/A'} IDR
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
      {#if loadingAnalysis === selectedSignal.symbol}
        <div class="card p-8 text-center">
          <p class="text-slate-500 dark:text-slate-400">
            {language === 'id' 
              ? 'Memuat data analisis...'
              : 'Loading analysis data...'}
          </p>
          <ProgressRadial width="w-8" stroke={100} meter="stroke-primary-500" track="stroke-primary-500/30" class="mx-auto mt-4" />
        </div>
      {:else}
        {#if analysisError}
          <div class="card variant-soft-error p-4">
            <p class="text-rose-700 dark:text-rose-300">
              <strong>{language === 'id' ? 'Gagal memuat analisis:' : 'Failed to load analysis:'}</strong>
              {analysisError}
            </p>
          </div>
        {/if}
        <StockAnalysis 
          symbol={selectedSignal.symbol}
          brokerSummary={selectedSignal.brokerSummary}
          technical={selectedSignal.technical}
          valuation={selectedSignal.valuation}
          conclusion={selectedSignal.conclusion}
          freshness={selectedSignal.freshness}
          {language}
        />
      {/if}
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
