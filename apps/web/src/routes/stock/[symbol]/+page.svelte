<script lang="ts">
  import { page } from '$app/stores';
  import { onMount, onDestroy } from 'svelte';
  import { TabGroup, Tab, ProgressRadial } from '@skeletonlabs/skeleton';
  import { api, type Stock, type StockFreshness, type StockScore, type StockPrice, type FundamentalData, type Job, type StockSourceType } from '$lib/api';
  import { PriceChart, ScoreGauge, FundamentalMetrics, ScoreBreakdown } from '$lib/components';

  let symbol = $derived($page.params.symbol ?? '');
  let stock = $state<Stock | null>(null);
  let score = $state<StockScore | null>(null);
  let prices = $state<StockPrice[]>([]);
  let fundamentals = $state<FundamentalData | null>(null);
  let freshness = $state<StockFreshness | null>(null);
  let isLoading = $state(true);
  let error = $state<string | null>(null);
  let inWatchlist = $state(false);
  let tabSet = $state(0);

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
    } catch (e) {
      error = (e as Error).message;
    } finally {
      isLoading = false;
    }
  });

  onDestroy(() => {
    Object.values(jobPollingIntervals).forEach(interval => clearInterval(interval));
  });

  const STALE_DAYS = 7;

  function isStale(asOf: string | null | undefined): boolean {
    if (!asOf) return true;
    const ms = Date.now() - new Date(asOf).getTime();
    return ms > STALE_DAYS * 24 * 60 * 60 * 1000;
  }

  function formatAsOf(asOf: string | null | undefined): string {
    if (!asOf) return 'Never';
    return new Date(asOf).toLocaleString();
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

  function dismissSourceJob(sourceKey: SourceKey) {
    sourceJobs[sourceKey] = null;
    sourceErrors[sourceKey] = null;
  }

  function getStatusBadge(hasData: boolean, isDataStale: boolean): { text: string; class: string } {
    if (!hasData) return { text: 'No Data', class: 'variant-soft-error' };
    if (isDataStale) return { text: 'Stale', class: 'variant-soft-warning' };
    return { text: 'Fresh', class: 'variant-soft-success' };
  }

  function getStatusBorder(hasData: boolean, isDataStale: boolean): string {
    if (!hasData) return 'border-l-red-500 bg-red-50/50 dark:bg-red-900/20';
    if (isDataStale) return 'border-l-amber-500 bg-amber-50/50 dark:bg-amber-900/20';
    return 'border-l-green-500 bg-green-50/50 dark:bg-green-900/20';
  }

  let latestPrice = $derived(prices.length > 0 ? prices[prices.length - 1] : null);
  let priceChange = $derived(() => {
    if (prices.length < 2) return { value: 0, percent: 0 };
    const latest = prices[prices.length - 1];
    const previous = prices[prices.length - 2];
    const change = latest.close - previous.close;
    const percent = (change / previous.close) * 100;
    return { value: change, percent };
  });

  let recentPrices = $derived(() => prices.slice().reverse().slice(0, 10));
</script>

<svelte:head>
  <title>{symbol} - JejakCuan</title>
</svelte:head>

<div class="space-y-6">
  <div class="flex items-start justify-between">
    <div>
      <a href="/" class="text-sm font-medium text-primary-600 dark:text-primary-400 hover:text-primary-700 dark:hover:text-primary-300 underline underline-offset-2">&larr; Back to Screener</a>
      <div class="flex items-baseline gap-4 mt-2">
        <h1 class="h1">{symbol}</h1>
        {#if latestPrice}
          <span class="text-2xl font-semibold text-slate-900 dark:text-slate-100">
            {latestPrice.close.toLocaleString()}
          </span>
          <span
            class="text-lg font-bold"
            class:text-emerald-600={priceChange().value >= 0}
            class:dark:text-emerald-400={priceChange().value >= 0}
            class:text-rose-600={priceChange().value < 0}
            class:dark:text-rose-400={priceChange().value < 0}
          >
            {priceChange().value >= 0 ? '+' : ''}{priceChange().percent.toFixed(2)}%
          </span>
        {/if}
      </div>
      {#if stock}
        <p class="text-slate-600 dark:text-slate-300">{stock.name}</p>
      {/if}
    </div>

    <button
      onclick={toggleWatchlist}
      class="btn {inWatchlist ? 'variant-filled-warning' : 'variant-ghost-primary'}"
    >
      {inWatchlist ? 'Remove from Watchlist' : 'Add to Watchlist'}
    </button>
  </div>

  {#if error}
    <aside class="alert variant-filled-error">
      <p>{error}</p>
    </aside>
  {/if}

  {#if freshness}
    {@const stalePrices = isStale(freshness.prices_as_of)}
    {@const staleBroker = isStale(freshness.broker_flow_as_of)}
    {@const staleFinancials = isStale(freshness.financials_as_of)}
    {@const staleScores = isStale(freshness.scores_as_of)}
    {@const hasPrices = !!freshness.prices_as_of}
    {@const hasBroker = !!freshness.broker_flow_as_of}
    {@const hasFinancials = !!freshness.financials_as_of}
    {@const hasScores = !!freshness.scores_as_of}
    {@const anyStale = stalePrices || staleBroker || staleFinancials || staleScores}
    
    <div class="card p-4">
      <div class="flex items-center justify-between gap-2 flex-wrap mb-4">
        <div>
          <h3 class="h3">Data Sources</h3>
          <p class="text-sm text-surface-500 mt-1">Trigger individual data sources or refresh all</p>
        </div>
        <div class="flex items-center gap-2">
          <span class="badge {anyStale ? 'variant-soft-warning' : 'variant-soft-success'}">
            {anyStale ? `Some Stale (>${STALE_DAYS}d)` : 'All Fresh'}
          </span>
          <button
            onclick={refreshAllSources}
            disabled={isRefreshingAll || sourceLoading.price || sourceLoading.broker || sourceLoading.fundamental}
            class="btn btn-sm {isRefreshingAll ? 'variant-ghost-surface' : 'variant-filled-tertiary'}"
          >
            {#if isRefreshingAll}
              <ProgressRadial width="w-4" stroke={100} meter="stroke-white" track="stroke-white/30" />
              <span>Refreshing All...</span>
            {:else}
              Refresh All
            {/if}
          </button>
          <a href="/admin/data-status" class="btn btn-sm variant-ghost-primary">
            Admin Panel
          </a>
        </div>
      </div>

      {#if anyStale}
        <div class="mb-4 p-3 rounded-lg bg-amber-50 dark:bg-amber-900/30 text-amber-900 dark:text-amber-200 text-sm">
          Some data is older than {STALE_DAYS} days or missing. Click "Trigger" to refresh individual sources.
        </div>
      {/if}

      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        {#if true}
          {@const priceStatus = getStatusBadge(hasPrices, stalePrices)}
          {@const priceJob = sourceJobs.price}
          {@const priceLoading = sourceLoading.price}
          {@const priceError = sourceErrors.price}
          <div class="p-4 rounded-lg border-l-4 {getStatusBorder(hasPrices, stalePrices)} relative">
          {#if priceLoading && priceJob}
            <div class="absolute inset-0 bg-surface-900/40 backdrop-blur-sm rounded-r-lg flex items-center justify-center z-10">
              <div class="flex flex-col items-center gap-2 p-4 text-center">
                <ProgressRadial width="w-10" stroke={100} meter="stroke-primary-500" track="stroke-primary-500/30" />
                <span class="text-sm font-medium">Running...</span>
                {#if priceJob.duration_secs}
                  <span class="text-xs text-surface-400">{priceJob.duration_secs.toFixed(1)}s elapsed</span>
                {/if}
              </div>
            </div>
          {/if}
          
          <div class="flex items-center justify-between mb-2">
            <h4 class="font-bold text-slate-900 dark:text-slate-100">Yahoo Finance</h4>
            <span class="badge text-xs {priceStatus.class}">{priceStatus.text}</span>
          </div>
          <div class="text-sm text-slate-600 dark:text-slate-300 mb-2">
            <strong>Provides:</strong> OHLCV prices, volume, historical data
          </div>
          <div class="text-xs text-slate-500 mb-2">
            <strong>Used in:</strong> Price chart, technical indicators
          </div>
          <div class="text-xs font-mono text-slate-500 mb-3">
            Updated: {formatHoursAgo(freshness.prices_as_of)}
          </div>

          {#if priceError}
            <div class="mb-3 p-2 rounded bg-error-500/20 text-error-500 text-xs">{priceError}</div>
          {/if}

          {#if priceJob && !priceLoading}
            {@const isSuccess = priceJob.status === 'completed'}
            {@const isFailed = priceJob.status === 'failed'}
            <div class="mb-3 p-2 rounded text-xs {isSuccess ? 'bg-green-500/20 text-green-700 dark:text-green-300' : isFailed ? 'bg-error-500/20 text-error-500' : 'bg-tertiary-500/20 text-tertiary-700'}">
              <div class="flex justify-between items-start gap-2">
                <div class="flex-1 min-w-0">
                  <div class="font-medium flex items-center gap-2">
                    {isSuccess ? '✓' : '✗'} {priceJob.status.toUpperCase()}
                    {#if priceJob.duration_secs}
                      <span class="text-surface-400 font-normal">({priceJob.duration_secs.toFixed(1)}s)</span>
                    {/if}
                  </div>
                  {#if priceJob.output}
                    <details class="mt-1">
                      <summary class="cursor-pointer hover:opacity-80">View output</summary>
                      <pre class="mt-1 p-2 bg-surface-900/30 rounded overflow-x-auto text-[10px] max-h-24 overflow-y-auto whitespace-pre-wrap">{priceJob.output}</pre>
                    </details>
                  {/if}
                </div>
                <button class="hover:opacity-70 text-lg leading-none" onclick={() => dismissSourceJob('price')}>×</button>
              </div>
            </div>
          {/if}

          <div class="flex justify-end">
            <button
              class="btn btn-sm variant-ghost-primary"
              onclick={() => triggerSource('price')}
              disabled={priceLoading || isRefreshingAll}
            >
              <span class="text-sm" class:animate-spin={priceLoading}>↻</span>
              <span>Trigger</span>
            </button>
          </div>
        </div>
        {/if}

        {#if true}
          {@const brokerStatus = getStatusBadge(hasBroker, staleBroker)}
          {@const brokerJob = sourceJobs.broker}
          {@const brokerLoading = sourceLoading.broker}
          {@const brokerError = sourceErrors.broker}
          <div class="p-4 rounded-lg border-l-4 {getStatusBorder(hasBroker, staleBroker)} relative">
          {#if brokerLoading && brokerJob}
            <div class="absolute inset-0 bg-surface-900/40 backdrop-blur-sm rounded-r-lg flex items-center justify-center z-10">
              <div class="flex flex-col items-center gap-2 p-4 text-center">
                <ProgressRadial width="w-10" stroke={100} meter="stroke-primary-500" track="stroke-primary-500/30" />
                <span class="text-sm font-medium">Running...</span>
                {#if brokerJob.duration_secs}
                  <span class="text-xs text-surface-400">{brokerJob.duration_secs.toFixed(1)}s elapsed</span>
                {/if}
              </div>
            </div>
          {/if}

          <div class="flex items-center justify-between mb-2">
            <h4 class="font-bold text-slate-900 dark:text-slate-100">Stockbit/IDX</h4>
            <span class="badge text-xs {brokerStatus.class}">{brokerStatus.text}</span>
          </div>
          <div class="text-sm text-slate-600 dark:text-slate-300 mb-2">
            <strong>Provides:</strong> Broker flow, foreign/domestic net
          </div>
          <div class="text-xs text-slate-500 mb-2">
            <strong>Used in:</strong> Broker analysis, sentiment score
          </div>
          <div class="text-xs font-mono text-slate-500 mb-3">
            Updated: {formatHoursAgo(freshness.broker_flow_as_of)}
          </div>

          {#if brokerError}
            <div class="mb-3 p-2 rounded bg-error-500/20 text-error-500 text-xs">{brokerError}</div>
          {/if}

          {#if brokerJob && !brokerLoading}
            {@const isSuccess = brokerJob.status === 'completed'}
            {@const isFailed = brokerJob.status === 'failed'}
            <div class="mb-3 p-2 rounded text-xs {isSuccess ? 'bg-green-500/20 text-green-700 dark:text-green-300' : isFailed ? 'bg-error-500/20 text-error-500' : 'bg-tertiary-500/20 text-tertiary-700'}">
              <div class="flex justify-between items-start gap-2">
                <div class="flex-1 min-w-0">
                  <div class="font-medium flex items-center gap-2">
                    {isSuccess ? '✓' : '✗'} {brokerJob.status.toUpperCase()}
                    {#if brokerJob.duration_secs}
                      <span class="text-surface-400 font-normal">({brokerJob.duration_secs.toFixed(1)}s)</span>
                    {/if}
                  </div>
                  {#if brokerJob.output}
                    <details class="mt-1">
                      <summary class="cursor-pointer hover:opacity-80">View output</summary>
                      <pre class="mt-1 p-2 bg-surface-900/30 rounded overflow-x-auto text-[10px] max-h-24 overflow-y-auto whitespace-pre-wrap">{brokerJob.output}</pre>
                    </details>
                  {/if}
                </div>
                <button class="hover:opacity-70 text-lg leading-none" onclick={() => dismissSourceJob('broker')}>×</button>
              </div>
            </div>
          {/if}

          <div class="flex justify-end">
            <button
              class="btn btn-sm variant-ghost-primary"
              onclick={() => triggerSource('broker')}
              disabled={brokerLoading || isRefreshingAll}
            >
              <span class="text-sm" class:animate-spin={brokerLoading}>↻</span>
              <span>Trigger</span>
            </button>
          </div>
        </div>
        {/if}

        {#if true}
          {@const fundStatus = getStatusBadge(hasFinancials, staleFinancials)}
          {@const fundJob = sourceJobs.fundamental}
          {@const fundLoading = sourceLoading.fundamental}
          {@const fundError = sourceErrors.fundamental}
          <div class="p-4 rounded-lg border-l-4 {getStatusBorder(hasFinancials, staleFinancials)} relative">
          {#if fundLoading && fundJob}
            <div class="absolute inset-0 bg-surface-900/40 backdrop-blur-sm rounded-r-lg flex items-center justify-center z-10">
              <div class="flex flex-col items-center gap-2 p-4 text-center">
                <ProgressRadial width="w-10" stroke={100} meter="stroke-primary-500" track="stroke-primary-500/30" />
                <span class="text-sm font-medium">Running...</span>
                {#if fundJob.duration_secs}
                  <span class="text-xs text-surface-400">{fundJob.duration_secs.toFixed(1)}s elapsed</span>
                {/if}
              </div>
            </div>
          {/if}

          <div class="flex items-center justify-between mb-2">
            <h4 class="font-bold text-slate-900 dark:text-slate-100">yfinance/Sectors.app</h4>
            <span class="badge text-xs {fundStatus.class}">{fundStatus.text}</span>
          </div>
          <div class="text-sm text-slate-600 dark:text-slate-300 mb-2">
            <strong>Provides:</strong> P/E, P/B, ROE, ROA, margins
          </div>
          <div class="text-xs text-slate-500 mb-2">
            <strong>Used in:</strong> Fundamental tab, valuation metrics
          </div>
          <div class="text-xs font-mono text-slate-500 mb-3">
            Updated: {formatHoursAgo(freshness.financials_as_of)}
          </div>

          {#if fundError}
            <div class="mb-3 p-2 rounded bg-error-500/20 text-error-500 text-xs">{fundError}</div>
          {/if}

          {#if fundJob && !fundLoading}
            {@const isSuccess = fundJob.status === 'completed'}
            {@const isFailed = fundJob.status === 'failed'}
            <div class="mb-3 p-2 rounded text-xs {isSuccess ? 'bg-green-500/20 text-green-700 dark:text-green-300' : isFailed ? 'bg-error-500/20 text-error-500' : 'bg-tertiary-500/20 text-tertiary-700'}">
              <div class="flex justify-between items-start gap-2">
                <div class="flex-1 min-w-0">
                  <div class="font-medium flex items-center gap-2">
                    {isSuccess ? '✓' : '✗'} {fundJob.status.toUpperCase()}
                    {#if fundJob.duration_secs}
                      <span class="text-surface-400 font-normal">({fundJob.duration_secs.toFixed(1)}s)</span>
                    {/if}
                  </div>
                  {#if fundJob.output}
                    <details class="mt-1">
                      <summary class="cursor-pointer hover:opacity-80">View output</summary>
                      <pre class="mt-1 p-2 bg-surface-900/30 rounded overflow-x-auto text-[10px] max-h-24 overflow-y-auto whitespace-pre-wrap">{fundJob.output}</pre>
                    </details>
                  {/if}
                </div>
                <button class="hover:opacity-70 text-lg leading-none" onclick={() => dismissSourceJob('fundamental')}>×</button>
              </div>
            </div>
          {/if}

          <div class="flex justify-end">
            <button
              class="btn btn-sm variant-ghost-primary"
              onclick={() => triggerSource('fundamental')}
              disabled={fundLoading || isRefreshingAll}
            >
              <span class="text-sm" class:animate-spin={fundLoading}>↻</span>
              <span>Trigger</span>
            </button>
          </div>
        </div>
        {/if}

        {#if true}
          {@const scoreStatus = getStatusBadge(hasScores, staleScores)}
          <div class="p-4 rounded-lg border-l-4 {getStatusBorder(hasScores, staleScores)}">
          <div class="flex items-center justify-between mb-2">
            <h4 class="font-bold text-slate-900 dark:text-slate-100">Computed Scores</h4>
            <span class="badge text-xs {scoreStatus.class}">{scoreStatus.text}</span>
          </div>
          <div class="text-sm text-slate-600 dark:text-slate-300 mb-2">
            <strong>Provides:</strong> Technical, fundamental, sentiment, ML scores
          </div>
          <div class="text-xs text-slate-500 mb-2">
            <strong>Used in:</strong> Score gauges, composite score, signals
          </div>
          <div class="text-xs font-mono text-slate-500 mb-3">
            Updated: {formatHoursAgo(freshness.scores_as_of)}
          </div>
          <div class="flex justify-end">
            <span class="text-xs text-surface-400 italic">Auto-computed from other sources</span>
          </div>
        </div>
        {/if}
      </div>
    </div>
  {/if}

  {#if isLoading}
    <div class="flex items-center justify-center p-8">
      <ProgressRadial stroke={100} meter="stroke-primary-500" track="stroke-primary-500/30" />
    </div>
  {:else if stock}
    <TabGroup>
      <Tab bind:group={tabSet} name="technical" value={0}>Technical</Tab>
      <Tab bind:group={tabSet} name="fundamental" value={1}>Fundamental</Tab>
    </TabGroup>

    {#if tabSet === 0}
      <div class="card p-4">
        <h3 class="h3 mb-4">Price Chart (60 Days)</h3>
        {#if prices.length > 0}
          <PriceChart {prices} height={400} showVolume={true} showEma={true} />
        {:else}
          <p class="text-surface-600-300-token p-8 text-center">No price data available</p>
        {/if}
      </div>

      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div class="card p-6">
          <h3 class="h3 mb-6">Score Breakdown</h3>
          {#if score}
            <div class="flex flex-wrap justify-around gap-4">
              <ScoreGauge score={score.composite_score} label="Composite" size="lg" />
              <ScoreGauge score={score.technical_score} label="Technical" />
              <ScoreGauge score={score.fundamental_score} label="Fundamental" />
              <ScoreGauge score={score.sentiment_score} label="Sentiment" />
              <ScoreGauge score={score.ml_score} label="ML" />
            </div>

            <div class="mt-6 p-4 rounded-lg bg-surface-100-800-token">
              <p class="text-sm">
                {#if score.composite_score >= 70}
                  <span class="text-green-500 font-semibold">STRONG BUY</span> - Multiple strong signals aligned
                {:else if score.composite_score >= 50}
                  <span class="text-yellow-500 font-semibold">HOLD/WATCH</span> - Mixed signals, monitor closely
                {:else}
                  <span class="text-red-500 font-semibold">WEAK</span> - Unfavorable conditions
                {/if}
              </p>
            </div>
          {:else}
            <p class="text-surface-600-300-token text-center">No score data available</p>
          {/if}
        </div>

        <div class="card p-6">
          <h3 class="h3 mb-4">Stock Info</h3>
          <dl class="space-y-3">
            <div class="flex justify-between">
              <dt class="text-surface-600-300-token">Sector</dt>
              <dd class="font-medium">{stock.sector ?? '-'}</dd>
            </div>
            <div class="flex justify-between">
              <dt class="text-surface-600-300-token">Subsector</dt>
              <dd class="font-medium">{stock.subsector ?? '-'}</dd>
            </div>
            <div class="flex justify-between">
              <dt class="text-surface-600-300-token">Status</dt>
              <dd>
                <span class="badge {stock.is_active ? 'variant-filled-success' : 'variant-filled-error'}">
                  {stock.is_active ? 'Active' : 'Inactive'}
                </span>
              </dd>
            </div>
            {#if latestPrice}
              <hr class="opacity-20" />
              <div class="flex justify-between">
                <dt class="text-slate-600 dark:text-slate-300">Open</dt>
                <dd class="font-medium text-slate-900 dark:text-slate-100">{latestPrice.open.toLocaleString()}</dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-slate-600 dark:text-slate-300">High</dt>
                <dd class="font-semibold text-emerald-600 dark:text-emerald-400">{latestPrice.high.toLocaleString()}</dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-slate-600 dark:text-slate-300">Low</dt>
                <dd class="font-semibold text-rose-600 dark:text-rose-400">{latestPrice.low.toLocaleString()}</dd>
              </div>
              <div class="flex justify-between">
                <dt class="text-slate-600 dark:text-slate-300">Volume</dt>
                <dd class="font-medium text-slate-900 dark:text-slate-100">{latestPrice.volume.toLocaleString()}</dd>
              </div>
            {/if}
          </dl>
        </div>
      </div>

      <div class="card p-4">
        <h3 class="h3 mb-4">Recent Price History</h3>
        {#if prices.length > 0}
          <div class="table-container">
            <table class="table table-hover">
              <thead>
                <tr>
                  <th class="text-slate-900 dark:text-slate-100">Date</th>
                  <th class="text-slate-900 dark:text-slate-100 text-right">Open</th>
                  <th class="text-slate-900 dark:text-slate-100 text-right">High</th>
                  <th class="text-slate-900 dark:text-slate-100 text-right">Low</th>
                  <th class="text-slate-900 dark:text-slate-100 text-right">Close</th>
                  <th class="text-slate-900 dark:text-slate-100 text-right">Volume</th>
                </tr>
              </thead>
              <tbody>
                {#each recentPrices() as price}
                  <tr>
                    <td class="text-slate-700 dark:text-slate-300">{new Date(price.time).toLocaleDateString()}</td>
                    <td class="text-right font-tabular text-slate-700 dark:text-slate-300">{price.open.toLocaleString()}</td>
                    <td class="text-right font-tabular text-emerald-600 dark:text-emerald-400">{price.high.toLocaleString()}</td>
                    <td class="text-right font-tabular text-rose-600 dark:text-rose-400">{price.low.toLocaleString()}</td>
                    <td class="text-right font-tabular font-semibold text-slate-900 dark:text-slate-100">{price.close.toLocaleString()}</td>
                    <td class="text-right font-tabular text-slate-600 dark:text-slate-400">{price.volume.toLocaleString()}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {:else}
          <p class="text-slate-500 dark:text-slate-400 text-center p-4">No price history available</p>
        {/if}
      </div>
    {:else}
      <div class="space-y-6">
        <div class="card p-4">
          <h3 class="h3 mb-4">Valuation Metrics</h3>
          <FundamentalMetrics data={fundamentals} currentPrice={latestPrice?.close ?? 0} />
        </div>

        {#if score && fundamentals}
          {@const peScore = fundamentals.pe_ratio ? (fundamentals.pe_ratio < 15 ? 80 : fundamentals.pe_ratio < 25 ? 60 : 40) : 50}
          {@const pbScore = fundamentals.pb_ratio ? (fundamentals.pb_ratio < 1 ? 85 : fundamentals.pb_ratio < 2 ? 70 : 50) : 50}
          {@const roeScore = fundamentals.roe ? (fundamentals.roe > 15 ? 80 : fundamentals.roe > 10 ? 65 : 45) : 50}
          {@const roaScore = fundamentals.roa ? (fundamentals.roa > 10 ? 75 : fundamentals.roa > 5 ? 60 : 45) : 50}
          <div class="card p-6">
            <h3 class="h3 mb-4">Fundamental Score Breakdown</h3>
            <ScoreBreakdown
              components={[
                { name: 'Valuation (P/E)', score: peScore, weight: 0.3, signals: [fundamentals.pe_ratio ? `P/E: ${fundamentals.pe_ratio.toFixed(1)}` : 'No P/E data'] },
                { name: 'Book Value (P/B)', score: pbScore, weight: 0.25, signals: [fundamentals.pb_ratio ? `P/B: ${fundamentals.pb_ratio.toFixed(2)}` : 'No P/B data'] },
                { name: 'Profitability (ROE)', score: roeScore, weight: 0.25, signals: [fundamentals.roe ? `ROE: ${fundamentals.roe.toFixed(1)}%` : 'No ROE data'] },
                { name: 'Efficiency (ROA)', score: roaScore, weight: 0.2, signals: [fundamentals.roa ? `ROA: ${fundamentals.roa.toFixed(1)}%` : 'No ROA data'] }
              ]}
              totalScore={score.fundamental_score}
            />
          </div>
        {:else if score}
          <div class="card p-6 text-center text-surface-500">
            <p>No fundamental breakdown available</p>
          </div>
        {/if}

        <div class="card p-6">
          <h3 class="h3 mb-4">Stock Info</h3>
          <dl class="space-y-3">
            <div class="flex justify-between">
              <dt class="text-surface-600-300-token">Sector</dt>
              <dd class="font-medium">{stock.sector ?? '-'}</dd>
            </div>
            <div class="flex justify-between">
              <dt class="text-surface-600-300-token">Subsector</dt>
              <dd class="font-medium">{stock.subsector ?? '-'}</dd>
            </div>
            <div class="flex justify-between">
              <dt class="text-surface-600-300-token">Status</dt>
              <dd>
                <span class="badge {stock.is_active ? 'variant-filled-success' : 'variant-filled-error'}">
                  {stock.is_active ? 'Active' : 'Inactive'}
                </span>
              </dd>
            </div>
          </dl>
        </div>
      </div>
    {/if}
  {/if}
</div>
