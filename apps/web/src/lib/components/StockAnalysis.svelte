<script lang="ts">
  /**
   * Comprehensive Stock Analysis Component
   * Displays broker summary, technical analysis, and valuation estimates
   * Supports both Indonesian (id) and English (en) languages
   */

  import type { BrokerSummary, TechnicalAnalysis, ValuationEstimate, OverallConclusion, InstitutionalFlowAnalysis } from './StockAnalysis.types';
  import type { StockFreshness } from '$lib/api';
  import InstitutionalFlowAnalysisComponent from './InstitutionalFlowAnalysis.svelte';

  let { 
    symbol = '',
    brokerSummary = null as BrokerSummary | null,
    technical = null as TechnicalAnalysis | null,
    valuation = null as ValuationEstimate | null,
    conclusion = null as OverallConclusion | null,
    freshness = null as StockFreshness | null,
    language = 'en' as 'en' | 'id'
  } = $props();

  // Translations
  const t = {
    en: {
      brokerTitle: '📊 Broker Summary (Orderbook)',
      bigBuyers: 'Big Buyers',
      bigSellers: 'Big Sellers',
      netting: 'Netting',
      accumulation: 'Accumulation',
      distribution: 'Distribution',
      balanced: 'Balanced',
      accumulationAt: 'accumulation at',
      distributionAt: 'distribution at',
      priceHeldAt: 'price held at equilibrium',
      conclusion: 'Conclusion',
      tugOfWar: 'Tug-of-war accumulation vs distribution, consolidation',
      technicalTitle: '📉 Technical Analysis',
      lastPrice: 'Last Price',
      neutral: 'neutral',
      momentum: 'momentum',
      weakening: 'weakening',
      strengthening: 'strengthening',
      priceInCloud: 'price in cloud',
      consolidation: 'consolidation',
      priceAboveCloud: 'price above cloud',
      priceBelowCloud: 'price below cloud',
      support: 'Support',
      resistance: 'Resistance',
      taSummary: 'TA Summary',
      sell: 'Sell',
      buy: 'Buy',
      valuationTitle: '📐 Reasonable Price Estimates',
      forwardEps: 'forward EPS',
      fairPriceRange: 'Fair price range',
      bullCase: 'Bull case',
      overallTitle: '📝 Overall Conclusion',
      strengths: 'Strengths',
      weaknesses: 'Weaknesses',
      strategy: 'Strategy',
      traders: 'Traders',
      investors: 'Growth investors',
      valueInvestors: 'Value investors',
      noData: 'No data available'
    },
    id: {
      brokerTitle: '📊 Ringkasan Broker (Orderbook)',
      bigBuyers: 'Pembeli Besar',
      bigSellers: 'Penjual Besar',
      netting: 'Netting',
      accumulation: 'Akumulasi',
      distribution: 'Distribusi',
      balanced: 'Seimbang',
      accumulationAt: 'akumulasi di',
      distributionAt: 'distribusi di',
      priceHeldAt: 'harga tertahan di keseimbangan',
      conclusion: 'Kesimpulan',
      tugOfWar: 'Tarik-menarik akumulasi vs distribusi, konsolidasi',
      technicalTitle: '📉 Analisis Teknikal',
      lastPrice: 'Harga Terakhir',
      neutral: 'netral',
      momentum: 'momentum',
      weakening: 'melemah',
      strengthening: 'menguat',
      priceInCloud: 'harga dalam awan',
      consolidation: 'konsolidasi',
      priceAboveCloud: 'harga di atas awan',
      priceBelowCloud: 'harga di bawah awan',
      support: 'Support',
      resistance: 'Resistance',
      taSummary: 'Ringkasan TA',
      sell: 'Jual',
      buy: 'Beli',
      valuationTitle: '📐 Estimasi Harga Wajar',
      forwardEps: 'EPS proyeksi',
      fairPriceRange: 'Rentang harga wajar',
      bullCase: 'Skenario bullish',
      overallTitle: '📝 Kesimpulan Keseluruhan',
      strengths: 'Kekuatan',
      weaknesses: 'Kelemahan',
      strategy: 'Strategi',
      traders: 'Trader',
      investors: 'Investor growth',
      valueInvestors: 'Investor value',
      noData: 'Data tidak tersedia'
    }
  };

  let labels = $derived(t[language]);

  const STALE_DAYS = 7;

  function isStale(asOf: string | null | undefined): boolean {
    if (!asOf) return true;
    const ms = Date.now() - new Date(asOf).getTime();
    return ms > STALE_DAYS * 24 * 60 * 60 * 1000;
  }

  function formatAsOf(asOf: string | null | undefined): string {
    if (!asOf) return '-';
    return new Date(asOf).toLocaleString();
  }

  const notableBrokers = new Set([
    // Example broker initials users often track
    'XL',
    // Seeded broker codes in DB
    'BK', 'KZ', 'CS', 'AK', 'GW', 'DP', 'RX', 'ZP',
    'CC', 'SQ', 'NI', 'OD', 'HP', 'KI', 'DX', 'IF', 'LG'
  ]);

  function formatPrice(price: number): string {
    return price.toLocaleString('id-ID');
  }

  function formatRange(low: number, high: number): string {
    return `${formatPrice(low)}–${formatPrice(high)}`;
  }

  function formatIdrCompact(value: number): string {
    const abs = Math.abs(value);
    const sign = value < 0 ? '-' : '+';
    if (abs >= 1e12) return `${sign}${(abs / 1e12).toFixed(2)}T`;
    if (abs >= 1e9) return `${sign}${(abs / 1e9).toFixed(2)}B`;
    if (abs >= 1e6) return `${sign}${(abs / 1e6).toFixed(2)}M`;
    return `${sign}${abs.toFixed(0)}`;
  }

  function brokerLabel(b: { code: string; name?: string | null; category?: string; netValue?: number }): string {
    const parts = [b.code];
    if (b.name) parts.push(b.name);
    if (b.category) parts.push(b.category);
    if (typeof b.netValue === 'number') parts.push(formatIdrCompact(b.netValue));
    return parts.join(' • ');
  }

  function getNetStatusLabel(status: string): string {
    if (status === 'accumulation') return labels.accumulation;
    if (status === 'distribution') return labels.distribution;
    return labels.balanced;
  }

  function getRsiLabel(rsi: number): string {
    if (rsi <= 30) return '→ oversold';
    if (rsi >= 70) return '→ overbought';
    return `→ ${labels.neutral}`;
  }

  function getMacdLabel(signal: string): string {
    return signal === 'negative' 
      ? `→ ${labels.momentum} ${labels.weakening}`
      : `→ ${labels.momentum} ${labels.strengthening}`;
  }

  function getIchimokuLabel(position: string): string {
    if (position === 'in') return `→ ${labels.priceInCloud}`;
    if (position === 'above') return `→ ${labels.priceAboveCloud}`;
    return `→ ${labels.priceBelowCloud}`;
  }
</script>

<div class="space-y-6">
  {#if freshness}
    {@const stalePrices = isStale(freshness.prices_as_of)}
    {@const staleBroker = isStale(freshness.broker_flow_as_of)}
    {@const staleFinancials = isStale(freshness.financials_as_of)}
    {@const staleScores = isStale(freshness.scores_as_of)}
    <div class="card p-4">
      <div class="flex items-center justify-between gap-2 flex-wrap">
        <h3 class="h4 text-slate-900 dark:text-slate-100">Data Freshness</h3>
        <span class="badge {stalePrices || staleBroker || staleFinancials || staleScores ? 'variant-soft-warning' : 'variant-soft-success'}">
          {stalePrices || staleBroker || staleFinancials || staleScores ? `Stale (>${STALE_DAYS}d)` : 'Fresh'}
        </span>
      </div>
      <dl class="mt-3 grid grid-cols-1 md:grid-cols-2 gap-3 text-sm">
        <div class="flex justify-between p-3 rounded-lg bg-slate-50 dark:bg-slate-800">
          <dt class="text-slate-600 dark:text-slate-300">Prices</dt>
          <dd class="font-medium {stalePrices ? 'text-amber-700 dark:text-amber-300' : 'text-slate-900 dark:text-slate-100'}">
            {formatAsOf(freshness.prices_as_of)}
          </dd>
        </div>
        <div class="flex justify-between p-3 rounded-lg bg-slate-50 dark:bg-slate-800">
          <dt class="text-slate-600 dark:text-slate-300">Broker Flow</dt>
          <dd class="font-medium {staleBroker ? 'text-amber-700 dark:text-amber-300' : 'text-slate-900 dark:text-slate-100'}">
            {formatAsOf(freshness.broker_flow_as_of)}
          </dd>
        </div>
        <div class="flex justify-between p-3 rounded-lg bg-slate-50 dark:bg-slate-800">
          <dt class="text-slate-600 dark:text-slate-300">Fundamentals</dt>
          <dd class="font-medium {staleFinancials ? 'text-amber-700 dark:text-amber-300' : 'text-slate-900 dark:text-slate-100'}">
            {formatAsOf(freshness.financials_as_of)}
          </dd>
        </div>
        <div class="flex justify-between p-3 rounded-lg bg-slate-50 dark:bg-slate-800">
          <dt class="text-slate-600 dark:text-slate-300">Scores</dt>
          <dd class="font-medium {staleScores ? 'text-amber-700 dark:text-amber-300' : 'text-slate-900 dark:text-slate-100'}">
            {formatAsOf(freshness.scores_as_of)}
          </dd>
        </div>
      </dl>
    </div>
  {/if}
  <!-- Broker Summary -->
  {#if brokerSummary}
    <div class="card p-4">
      <h3 class="h4 mb-4 text-slate-900 dark:text-slate-100">{labels.brokerTitle}</h3>
      <div class="space-y-3 text-sm">
        <div class="flex flex-wrap gap-2">
          <span class="font-semibold text-slate-700 dark:text-slate-300">{labels.bigBuyers}:</span>
          <div class="flex flex-wrap gap-2">
            {#each brokerSummary.bigBuyers as b (b.code)}
              <span
                class="badge variant-soft-secondary {notableBrokers.has(b.code) ? 'ring-2 ring-primary-500/40' : ''}"
                title={brokerLabel(b)}
              >
                <span class="text-emerald-700 dark:text-emerald-400 font-medium">{b.code}</span>
                {#if typeof b.netValue === 'number'}
                  <span class="ml-1 text-slate-600 dark:text-slate-400">{formatIdrCompact(b.netValue)}</span>
                {/if}
              </span>
            {/each}
          </div>
          <span class="text-slate-600 dark:text-slate-400">
            → {labels.accumulationAt} {formatRange(brokerSummary.priceRange.low, brokerSummary.priceRange.high)}
          </span>
        </div>
        <div class="flex flex-wrap gap-2">
          <span class="font-semibold text-slate-700 dark:text-slate-300">{labels.bigSellers}:</span>
          <div class="flex flex-wrap gap-2">
            {#each brokerSummary.bigSellers as b (b.code)}
              <span
                class="badge variant-soft-secondary {notableBrokers.has(b.code) ? 'ring-2 ring-primary-500/40' : ''}"
                title={brokerLabel(b)}
              >
                <span class="text-rose-700 dark:text-rose-400 font-medium">{b.code}</span>
                {#if typeof b.netValue === 'number'}
                  <span class="ml-1 text-slate-600 dark:text-slate-400">{formatIdrCompact(b.netValue)}</span>
                {/if}
              </span>
            {/each}
          </div>
          <span class="text-slate-600 dark:text-slate-400">
            → {labels.distributionAt} {formatRange(brokerSummary.priceRange.low, brokerSummary.priceRange.high)}
          </span>
        </div>
        {#if typeof brokerSummary.foreignNet === 'number' || typeof brokerSummary.domesticNet === 'number'}
          <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
            {#if typeof brokerSummary.foreignNet === 'number'}
              <div class="p-2 bg-slate-50 dark:bg-slate-800 rounded">
                <span class="text-slate-600 dark:text-slate-400">Foreign net:</span>
                <span class="ml-2 font-medium {brokerSummary.foreignNet >= 0 ? 'text-emerald-700 dark:text-emerald-400' : 'text-rose-700 dark:text-rose-400'}">
                  {formatIdrCompact(brokerSummary.foreignNet)}
                </span>
              </div>
            {/if}
            {#if typeof brokerSummary.domesticNet === 'number'}
              <div class="p-2 bg-slate-50 dark:bg-slate-800 rounded">
                <span class="text-slate-600 dark:text-slate-400">Domestic net:</span>
                <span class="ml-2 font-medium {brokerSummary.domesticNet >= 0 ? 'text-emerald-700 dark:text-emerald-400' : 'text-rose-700 dark:text-rose-400'}">
                  {formatIdrCompact(brokerSummary.domesticNet)}
                </span>
              </div>
            {/if}
          </div>
        {/if}
        <div class="flex flex-wrap gap-2">
          <span class="font-semibold text-slate-700 dark:text-slate-300">{labels.netting}:</span>
          <span class="{brokerSummary.netStatus === 'accumulation' ? 'text-emerald-600 dark:text-emerald-400' : 
                        brokerSummary.netStatus === 'distribution' ? 'text-rose-600 dark:text-rose-400' : 
                        'text-amber-600 dark:text-amber-400'} font-medium">
            {getNetStatusLabel(brokerSummary.netStatus)}
          </span>
          <span class="text-slate-600 dark:text-slate-400">
            → {labels.priceHeldAt} {formatPrice((brokerSummary.priceRange.low + brokerSummary.priceRange.high) / 2)}
          </span>
        </div>
        <div class="mt-3 p-3 bg-slate-100 dark:bg-slate-800 rounded-lg">
          <span class="font-semibold text-slate-700 dark:text-slate-300">👉 {labels.conclusion}:</span>
          <span class="text-slate-600 dark:text-slate-400 ml-2">
            {labels.tugOfWar} {formatRange(brokerSummary.priceRange.low, brokerSummary.priceRange.high)}
          </span>
        </div>
      </div>
    </div>
  {/if}

  {#if brokerSummary?.institutionalAnalysis}
    <InstitutionalFlowAnalysisComponent analysis={brokerSummary.institutionalAnalysis} {language} />
  {/if}

  <!-- Technical Analysis -->
  {#if technical}
    <div class="card p-4">
      <h3 class="h4 mb-4 text-slate-900 dark:text-slate-100">{labels.technicalTitle}</h3>
      <div class="space-y-2 text-sm">
        <div class="grid grid-cols-2 gap-4">
          <div>
            <span class="text-slate-600 dark:text-slate-400">{labels.lastPrice}:</span>
            <span class="font-semibold text-slate-900 dark:text-slate-100 ml-2">
              {formatPrice(technical.lastPrice)}
            </span>
          </div>
          <div>
            <span class="text-slate-600 dark:text-slate-400">RSI:</span>
            <span class="font-medium ml-2 {technical.rsi <= 30 ? 'text-emerald-600 dark:text-emerald-400' : 
                                           technical.rsi >= 70 ? 'text-rose-600 dark:text-rose-400' : 
                                           'text-slate-900 dark:text-slate-100'}">
              {technical.rsi} {getRsiLabel(technical.rsi)}
            </span>
          </div>
          <div>
            <span class="text-slate-600 dark:text-slate-400">MACD:</span>
            <span class="font-medium ml-2 {technical.macdSignal === 'positive' ? 
                                           'text-emerald-600 dark:text-emerald-400' : 
                                           'text-rose-600 dark:text-rose-400'}">
              {technical.macdSignal} {getMacdLabel(technical.macdSignal)}
            </span>
          </div>
          <div>
            <span class="text-slate-600 dark:text-slate-400">Ichimoku:</span>
            <span class="font-medium text-slate-900 dark:text-slate-100 ml-2">
              {getIchimokuLabel(technical.ichimoku.position)} 
              ({formatRange(technical.ichimoku.cloudRange.low, technical.ichimoku.cloudRange.high)})
            </span>
          </div>
        </div>
        
        <div class="mt-4 grid grid-cols-2 gap-4">
          <div>
            <span class="font-semibold text-slate-700 dark:text-slate-300">{labels.support}:</span>
            <span class="text-emerald-600 dark:text-emerald-400 ml-2">
              {technical.support.map(s => formatPrice(s)).join(', ')}
            </span>
          </div>
          <div>
            <span class="font-semibold text-slate-700 dark:text-slate-300">{labels.resistance}:</span>
            <span class="text-rose-600 dark:text-rose-400 ml-2">
              {technical.resistance.map(r => formatPrice(r)).join(', ')}
            </span>
          </div>
        </div>
        
        <div class="mt-3 p-3 bg-slate-100 dark:bg-slate-800 rounded-lg flex gap-4 flex-wrap">
          <span class="font-semibold text-slate-700 dark:text-slate-300">{labels.taSummary}:</span>
          <span class="text-rose-600 dark:text-rose-400">{technical.summary.sell} {labels.sell}</span>
          <span class="text-slate-600 dark:text-slate-400">{technical.summary.neutral} {labels.neutral}</span>
          <span class="text-emerald-600 dark:text-emerald-400">{technical.summary.buy} {labels.buy}</span>
        </div>
      </div>
    </div>
  {/if}

  <!-- Valuation Estimates -->
  {#if valuation}
    <div class="card p-4">
      <h3 class="h4 mb-4 text-slate-900 dark:text-slate-100">{labels.valuationTitle}</h3>
      <div class="space-y-2 text-sm">
        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div class="p-3 bg-slate-50 dark:bg-slate-800 rounded-lg">
            <span class="text-slate-600 dark:text-slate-400">PER ({valuation.forwardEps}x {labels.forwardEps}):</span>
            <span class="block text-lg font-bold text-slate-900 dark:text-slate-100">
              {formatPrice(valuation.perValue)} IDR
            </span>
          </div>
          <div class="p-3 bg-slate-50 dark:bg-slate-800 rounded-lg">
            <span class="text-slate-600 dark:text-slate-400">PBV ({valuation.bookValue}x):</span>
            <span class="block text-lg font-bold text-slate-900 dark:text-slate-100">
              {formatPrice(valuation.pbvValue)} IDR
            </span>
          </div>
          <div class="p-3 bg-slate-50 dark:bg-slate-800 rounded-lg">
            <span class="text-slate-600 dark:text-slate-400">EV/EBITDA ({valuation.evEbitdaValue}x):</span>
            <span class="block text-lg font-bold text-slate-900 dark:text-slate-100">
              {formatPrice(valuation.evEbitdaValue)} IDR
            </span>
          </div>
        </div>
        <div class="mt-3 grid grid-cols-2 gap-4">
          <div class="p-3 bg-emerald-50 dark:bg-emerald-900/30 rounded-lg">
            <span class="text-slate-600 dark:text-slate-400">{labels.fairPriceRange}:</span>
            <span class="block text-lg font-bold text-emerald-700 dark:text-emerald-400">
              {formatRange(valuation.fairPriceRange.low, valuation.fairPriceRange.high)} IDR
            </span>
          </div>
          <div class="p-3 bg-amber-50 dark:bg-amber-900/30 rounded-lg">
            <span class="text-slate-600 dark:text-slate-400">{labels.bullCase}:</span>
            <span class="block text-lg font-bold text-amber-700 dark:text-amber-400">
              {formatRange(valuation.bullCase.low, valuation.bullCase.high)} IDR
            </span>
          </div>
        </div>
      </div>
    </div>
  {/if}

  <!-- Overall Conclusion -->
  {#if conclusion}
    <div class="card p-4">
      <h3 class="h4 mb-4 text-slate-900 dark:text-slate-100">{labels.overallTitle}</h3>
      <div class="space-y-4 text-sm">
        <div>
          <span class="font-semibold text-emerald-700 dark:text-emerald-400">{labels.strengths}:</span>
          <span class="text-slate-600 dark:text-slate-400 ml-2">
            {conclusion.strengths.join(', ')}
          </span>
        </div>
        <div>
          <span class="font-semibold text-rose-700 dark:text-rose-400">{labels.weaknesses}:</span>
          <span class="text-slate-600 dark:text-slate-400 ml-2">
            {conclusion.weaknesses.join(', ')}
          </span>
        </div>
        <div class="mt-3 p-3 bg-slate-100 dark:bg-slate-800 rounded-lg space-y-2">
          <div class="font-semibold text-slate-700 dark:text-slate-300">{labels.strategy}:</div>
          <ul class="list-disc list-inside space-y-1 text-slate-600 dark:text-slate-400">
            <li><span class="font-medium">{labels.traders}:</span> {conclusion.strategy.traders}</li>
            <li><span class="font-medium">{labels.investors}:</span> {conclusion.strategy.investors}</li>
            <li><span class="font-medium">{labels.valueInvestors}:</span> {conclusion.strategy.valueInvestors}</li>
          </ul>
        </div>
      </div>
    </div>
  {/if}
  
  {#if !brokerSummary && !technical && !valuation && !conclusion}
    <div class="card p-8 text-center">
      <p class="text-slate-500 dark:text-slate-400">{labels.noData}</p>
    </div>
  {/if}
</div>
