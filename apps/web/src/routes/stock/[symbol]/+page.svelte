<script lang="ts">
  import { page } from '$app/stores';
  import { onMount, onDestroy } from 'svelte';
  import { ProgressRadial } from '@skeletonlabs/skeleton';
  import { api, type Stock, type StockFreshness, type StockScore, type StockPrice, type FundamentalData, type Job, type StockSourceType, type FullAnalysisResponse } from '$lib/api';
  import { PriceChart, ScoreGauge, FundamentalMetrics, ScoreBreakdown, StockAnalysis, StickyStockHeader, AnalysisSummary } from '$lib/components';
  import InstitutionalFlowAnalysisComponent from '$lib/components/InstitutionalFlowAnalysis.svelte';
  import type { BrokerSummary, TechnicalAnalysis, ValuationEstimate, OverallConclusion, InstitutionalFlowAnalysis, AccumulatorInfo } from '$lib/components/StockAnalysis.types';

  let symbol = $derived($page.params.symbol ?? '');
  let stock = $state<Stock | null>(null);
  let score = $state<StockScore | null>(null);
  let prices = $state<StockPrice[]>([]);
  let fundamentals = $state<FundamentalData | null>(null);
  let freshness = $state<StockFreshness | null>(null);
  let isLoading = $state(true);
  let error = $state<string | null>(null);
  let inWatchlist = $state(false);
  let language = $state<'en' | 'id'>('id');
  
  let brokerSummary = $state<BrokerSummary | null>(null);
  let technical = $state<TechnicalAnalysis | null>(null);
  let valuation = $state<ValuationEstimate | null>(null);
  let conclusion = $state<OverallConclusion | null>(null);
  let analysisLoading = $state(false);
  let analysisError = $state<string | null>(null);

  type SourceKey = 'price' | 'broker' | 'fundamental';

  let sourceLoading = $state<Record<SourceKey, boolean>>({
    price: false,
    broker: false,
    fundamental: false
  });
  let sourceJobs = $state<Record<SourceKey, Job | null>>({
    price: null,
    broker: null,
    fundamental: null
  });
  let sourceErrors = $state<Record<SourceKey, string | null>>({
    price: null,
    broker: null,
    fundamental: null
  });

  let jobPollingIntervals: Record<string, ReturnType<typeof setInterval>> = {};
  let isRefreshingAll = $state(false);

  onMount(async () => {
    if (!symbol) {
      error = 'No symbol provided';
      isLoading = false;
      return;
    }

    try {
      const [stockData, scoreData, priceData, watchlistData, fundamentalData, freshnessData] = await Promise.all([
        api.getStock(symbol),
        api.getStockScore(symbol),
        api.getStockPrices(symbol, 60),
        api.getWatchlist(),
        api.getFundamentals(symbol),
        api.getStockFreshness(symbol)
      ]);

      stock = stockData;
      score = scoreData;
      prices = priceData;
      inWatchlist = watchlistData.some((w) => w.symbol === symbol);
      fundamentals = fundamentalData;
      freshness = freshnessData;
      
      loadAnalysis();
    } catch (e) {
      error = (e as Error).message;
    } finally {
      isLoading = false;
    }
  });

  onDestroy(() => {
    Object.values(jobPollingIntervals).forEach(interval => clearInterval(interval));
  });

  function convertBrokerSummary(data: FullAnalysisResponse['broker_summary']): BrokerSummary | null {
    if (!data || (data.big_buyers.length === 0 && data.big_sellers.length === 0)) return null;
    
    let institutionalAnalysis: InstitutionalFlowAnalysis | null = null;
    if ((data as any).institutional_analysis) {
      const ia = (data as any).institutional_analysis;
      institutionalAnalysis = {
        accumulationScore: ia.accumulation_score,
        isAccumulating: ia.is_accumulating,
        coordinatedBuying: ia.coordinated_buying,
        daysAccumulated: ia.days_accumulated,
        net5Day: ia.net_5_day,
        net20Day: ia.net_20_day,
        institutionalNet5Day: ia.institutional_net_5_day,
        institutionalNet20Day: ia.institutional_net_20_day,
        foreignNet5Day: ia.foreign_net_5_day,
        foreignNet20Day: ia.foreign_net_20_day,
        topAccumulators: (ia.top_accumulators || []).map((acc: any): AccumulatorInfo => ({
          brokerCode: acc.broker_code,
          brokerName: acc.broker_name,
          category: acc.category,
          netValue: acc.net_value,
          netVolume: acc.net_volume,
          isForeign: acc.is_foreign,
        })),
        signalStrength: ia.signal_strength as InstitutionalFlowAnalysis['signalStrength'],
        signalDescription: ia.signal_description,
      };
    }
    
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
      institutionalAnalysis,
    };
  }

  function convertTechnical(data: FullAnalysisResponse['technical']): TechnicalAnalysis | null {
    if (!data) return null;
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

  function convertValuation(data: FullAnalysisResponse['valuation']): ValuationEstimate | null {
    if (!data) return null;
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

  function convertConclusion(data: FullAnalysisResponse['conclusion']): OverallConclusion | null {
    if (!data) return null;
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

  async function loadAnalysis() {
    if (!symbol || analysisLoading) return;
    
    analysisLoading = true;
    analysisError = null;
    
    try {
      const analysis = await api.getFullAnalysis(symbol);
      
      if (!analysis) {
        analysisError = language === 'id'
          ? 'Data analisis tidak tersedia untuk saham ini.'
          : 'Analysis data is not available for this stock.';
        return;
      }

      brokerSummary = convertBrokerSummary(analysis.broker_summary);
      technical = convertTechnical(analysis.technical);
      valuation = convertValuation(analysis.valuation);
      conclusion = convertConclusion(analysis.conclusion);
    } catch (e) {
      console.error(`Failed to load analysis for ${symbol}:`, e);
      analysisError = (e as Error).message;
    } finally {
      analysisLoading = false;
    }
  }

  const STALE_DAYS = 7;

  function isStale(asOf: string | null | undefined): boolean {
    if (!asOf) return true;
    const ms = Date.now() - new Date(asOf).getTime();
    return ms > STALE_DAYS * 24 * 60 * 60 * 1000;
  }

  function formatHoursAgo(asOf: string | null | undefined): string {
    if (!asOf) return 'No data';
    const hours = Math.floor((Date.now() - new Date(asOf).getTime()) / (1000 * 60 * 60));
    if (hours < 1) return 'Just now';
    if (hours < 24) return `${hours}h ago`;
    const days = Math.floor(hours / 24);
    return `${days}d ${hours % 24}h ago`;
  }

  async function toggleWatchlist() {
    if (!symbol) return;

    try {
      if (inWatchlist) {
        await api.removeFromWatchlist(symbol);
        inWatchlist = false;
      } else {
        await api.addToWatchlist(symbol);
        inWatchlist = true;
      }
    } catch (e) {
      error = (e as Error).message;
    }
  }

  async function triggerSource(sourceKey: SourceKey) {
    if (!symbol || sourceLoading[sourceKey]) return;

    sourceLoading[sourceKey] = true;
    sourceErrors[sourceKey] = null;
    sourceJobs[sourceKey] = null;

    try {
      const response = await api.refreshStockSource(symbol, sourceKey as StockSourceType);
      sourceJobs[sourceKey] = response.job;
      startJobPolling(sourceKey, response.job.id);
    } catch (e) {
      sourceErrors[sourceKey] = (e as Error).message;
      sourceLoading[sourceKey] = false;
    }
  }

  function startJobPolling(sourceKey: SourceKey, jobId: string) {
    if (jobPollingIntervals[sourceKey]) {
      clearInterval(jobPollingIntervals[sourceKey]);
    }

    jobPollingIntervals[sourceKey] = setInterval(async () => {
      try {
        const job = await api.getJob(jobId);
        const elapsedSecs = (Date.now() - new Date(job.started_at).getTime()) / 1000;
        sourceJobs[sourceKey] = { ...job, duration_secs: job.duration_secs ?? elapsedSecs };

        if (job.status === 'completed' || job.status === 'failed') {
          clearInterval(jobPollingIntervals[sourceKey]);
          delete jobPollingIntervals[sourceKey];
          sourceLoading[sourceKey] = false;

          if (job.status === 'completed') {
            const freshnessData = await api.getStockFreshness(symbol);
            freshness = freshnessData;
          }
        }
      } catch (e) {
        console.error('Failed to poll job:', e);
      }
    }, 2000);
  }

  async function refreshAllSources() {
    if (!symbol || isRefreshingAll) return;
    isRefreshingAll = true;

    for (const sourceKey of ['price', 'broker', 'fundamental'] as SourceKey[]) {
      await triggerSource(sourceKey);
      await new Promise(resolve => setTimeout(resolve, 500));
    }

    isRefreshingAll = false;
  }

  let latestPrice = $derived(prices.length > 0 ? prices[prices.length - 1] : null);

  let isDataFresh = $derived(() => {
    if (!freshness) return true;
    const now = new Date();
    const oneDay = 24 * 60 * 60 * 1000;
    
    const pricesStale = freshness.prices_as_of 
      ? (now.getTime() - new Date(freshness.prices_as_of).getTime()) > oneDay 
      : true;
    const brokerStale = freshness.broker_flow_as_of
      ? (now.getTime() - new Date(freshness.broker_flow_as_of).getTime()) > oneDay
      : true;
      
    return !pricesStale && !brokerStale;
  });
</script>

<svelte:head>
  <title>{symbol} - JejakCuan</title>
</svelte:head>

{#if isLoading}
  <div class="flex items-center justify-center min-h-screen">
    <ProgressRadial stroke={100} meter="stroke-primary-500" track="stroke-primary-500/30" />
  </div>
{:else if error}
  <div class="container mx-auto px-4 py-8">
    <aside class="alert variant-filled-error">
      <p>{error}</p>
    </aside>
  </div>
{:else if stock}
  <StickyStockHeader
    {stock}
    {latestPrice}
    {score}
    {inWatchlist}
    dataFresh={isDataFresh()}
    onToggleWatchlist={toggleWatchlist}
    onRefreshData={refreshAllSources}
  />

  <main class="container mx-auto px-4 py-6 space-y-4">
    <AnalysisSummary
      {score}
      {valuation}
      {conclusion}
      currentPrice={latestPrice?.close ?? 0}
    />

    <details class="card" open>
      <summary class="p-4 cursor-pointer font-bold flex items-center justify-between">
        <span>Broker Flow Analysis</span>
        <span class="badge variant-soft">
          {brokerSummary?.netStatus ?? 'Loading'}
        </span>
      </summary>
      <div class="p-4 pt-0">
        {#if analysisLoading}
          <div class="flex items-center justify-center py-8">
            <ProgressRadial width="w-8" stroke={100} meter="stroke-primary-500" track="stroke-primary-500/30" />
          </div>
        {:else if brokerSummary?.institutionalAnalysis}
          <InstitutionalFlowAnalysisComponent analysis={brokerSummary.institutionalAnalysis} />
        {:else}
          <p class="text-surface-500 text-center py-4">No broker flow data available</p>
        {/if}
      </div>
    </details>

    <details class="card">
      <summary class="p-4 cursor-pointer font-bold">Financial Analysis</summary>
      <div class="p-4 pt-0">
        {#if fundamentals}
          <FundamentalMetrics data={fundamentals} currentPrice={latestPrice?.close ?? 0} />
        {:else}
          <p class="text-surface-500 text-center py-4">No financial data available</p>
        {/if}
      </div>
    </details>

    <details class="card">
      <summary class="p-4 cursor-pointer font-bold">Technical Analysis</summary>
      <div class="p-4 pt-0">
        <PriceChart {prices} height={300} />
        {#if technical}
          <div class="grid grid-cols-2 md:grid-cols-4 gap-3 mt-4">
            <div class="card p-3 text-center">
              <span class="text-xs text-surface-500">RSI (14)</span>
              <div class="text-lg font-bold {technical.rsi < 30 ? 'text-success-500' : technical.rsi > 70 ? 'text-error-500' : ''}">
                {technical.rsi?.toFixed(1) ?? '-'}
              </div>
            </div>
            <div class="card p-3 text-center">
              <span class="text-xs text-surface-500">MACD</span>
              <div class="text-lg font-bold {technical.macdSignal === 'positive' ? 'text-success-500' : 'text-error-500'}">
                {technical.macdSignal ?? '-'}
              </div>
            </div>
            <div class="card p-3 text-center">
              <span class="text-xs text-surface-500">Support</span>
              <div class="text-lg font-bold text-success-500">
                {technical.support?.[0]?.toLocaleString('id-ID') ?? '-'}
              </div>
            </div>
            <div class="card p-3 text-center">
              <span class="text-xs text-surface-500">Resistance</span>
              <div class="text-lg font-bold text-error-500">
                {technical.resistance?.[0]?.toLocaleString('id-ID') ?? '-'}
              </div>
            </div>
          </div>
        {/if}
      </div>
    </details>

    <details class="card">
      <summary class="p-4 cursor-pointer font-bold">Score Breakdown</summary>
      <div class="p-4 pt-0">
        {#if score}
          <div class="flex flex-wrap justify-around gap-4">
            <ScoreGauge score={score.composite_score} label="Composite" size="lg" />
            <ScoreGauge score={score.technical_score} label="Technical" />
            <ScoreGauge score={score.fundamental_score} label="Fundamental" />
            <ScoreGauge score={score.sentiment_score} label="Sentiment" />
            <ScoreGauge score={score.ml_score} label="ML" />
          </div>
        {:else}
          <p class="text-surface-500 text-center py-4">No score data available</p>
        {/if}
      </div>
    </details>

    <details class="card">
      <summary class="p-4 cursor-pointer text-sm text-surface-500 flex items-center justify-between">
        <span>Data Sources & Refresh</span>
        <span class="badge variant-soft-surface text-xs">
          {isDataFresh() ? 'Fresh' : 'Stale'}
        </span>
      </summary>
      <div class="p-4 pt-0">
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          <div class="card p-3">
            <div class="flex items-center justify-between mb-2">
              <span class="font-medium text-sm">Prices</span>
              <span class="badge {freshness?.prices_as_of ? 'variant-soft-success' : 'variant-soft-warning'} text-xs">
                {freshness?.prices_as_of ? 'Fresh' : 'No Data'}
              </span>
            </div>
            <div class="text-xs text-surface-400 mb-2">
              {formatHoursAgo(freshness?.prices_as_of)}
            </div>
            <button
              class="btn btn-sm variant-ghost-primary w-full"
              onclick={() => triggerSource('price')}
              disabled={sourceLoading.price}
            >
              {sourceLoading.price ? 'Loading...' : 'Refresh'}
            </button>
          </div>
          
          <div class="card p-3">
            <div class="flex items-center justify-between mb-2">
              <span class="font-medium text-sm">Broker Flow</span>
              <span class="badge {freshness?.broker_flow_as_of ? 'variant-soft-success' : 'variant-soft-warning'} text-xs">
                {freshness?.broker_flow_as_of ? 'Fresh' : 'No Data'}
              </span>
            </div>
            <div class="text-xs text-surface-400 mb-2">
              {formatHoursAgo(freshness?.broker_flow_as_of)}
            </div>
            <button
              class="btn btn-sm variant-ghost-primary w-full"
              onclick={() => triggerSource('broker')}
              disabled={sourceLoading.broker}
            >
              {sourceLoading.broker ? 'Loading...' : 'Refresh'}
            </button>
          </div>
          
          <div class="card p-3">
            <div class="flex items-center justify-between mb-2">
              <span class="font-medium text-sm">Fundamentals</span>
              <span class="badge {freshness?.financials_as_of ? 'variant-soft-success' : 'variant-soft-warning'} text-xs">
                {freshness?.financials_as_of ? 'Fresh' : 'No Data'}
              </span>
            </div>
            <div class="text-xs text-surface-400 mb-2">
              {formatHoursAgo(freshness?.financials_as_of)}
            </div>
            <button
              class="btn btn-sm variant-ghost-primary w-full"
              onclick={() => triggerSource('fundamental')}
              disabled={sourceLoading.fundamental}
            >
              {sourceLoading.fundamental ? 'Loading...' : 'Refresh'}
            </button>
          </div>
          
          <div class="card p-3">
            <div class="flex items-center justify-between mb-2">
              <span class="font-medium text-sm">All Sources</span>
            </div>
            <div class="text-xs text-surface-400 mb-2">
              Refresh all data
            </div>
            <button
              class="btn btn-sm variant-filled-primary w-full"
              onclick={refreshAllSources}
              disabled={isRefreshingAll}
            >
              {isRefreshingAll ? 'Refreshing...' : 'Refresh All'}
            </button>
          </div>
        </div>
      </div>
    </details>
  </main>

  <div class="fixed bottom-0 left-0 right-0 p-3 bg-surface-100-800-token border-t md:hidden z-40">
    <div class="flex gap-2">
      <button
        class="btn flex-1 {inWatchlist ? 'variant-filled-primary' : 'variant-ghost-primary'}"
        onclick={toggleWatchlist}
      >
        {inWatchlist ? '★ In Watchlist' : '☆ Add to Watchlist'}
      </button>
    </div>
  </div>
  <div class="h-16 md:hidden"></div>
{:else}
  <div class="container mx-auto px-4 py-8">
    <p class="text-center text-surface-500">Loading stock data...</p>
  </div>
{/if}
