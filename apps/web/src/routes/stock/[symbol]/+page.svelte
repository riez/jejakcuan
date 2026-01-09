<script lang="ts">
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import { api, type Stock, type StockScore, type StockPrice } from '$lib/api';
  import { PriceChart, ScoreGauge } from '$lib/components';

  let symbol = $derived($page.params.symbol ?? '');
  let stock = $state<Stock | null>(null);
  let score = $state<StockScore | null>(null);
  let prices = $state<StockPrice[]>([]);
  let isLoading = $state(true);
  let error = $state<string | null>(null);
  let inWatchlist = $state(false);

  onMount(async () => {
    if (!symbol) {
      error = 'No symbol provided';
      isLoading = false;
      return;
    }

    try {
      const [stockData, scoreData, priceData, watchlistData] = await Promise.all([
        api.getStock(symbol),
        api.getStockScore(symbol),
        api.getStockPrices(symbol, 60),
        api.getWatchlist()
      ]);

      stock = stockData;
      score = scoreData;
      prices = priceData;
      inWatchlist = watchlistData.some((w) => w.symbol === symbol);
    } catch (e) {
      error = (e as Error).message;
    } finally {
      isLoading = false;
    }
  });

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

  // Get latest price info
  let latestPrice = $derived(prices.length > 0 ? prices[prices.length - 1] : null);
  let priceChange = $derived(() => {
    if (prices.length < 2) return { value: 0, percent: 0 };
    const latest = prices[prices.length - 1];
    const previous = prices[prices.length - 2];
    const change = latest.close - previous.close;
    const percent = (change / previous.close) * 100;
    return { value: change, percent };
  });
</script>

<svelte:head>
  <title>{symbol} - JejakCuan</title>
</svelte:head>

<div class="space-y-6">
  <!-- Header -->
  <div class="flex items-start justify-between">
    <div>
      <a href="/" class="anchor text-sm">&larr; Back to Screener</a>
      <div class="flex items-baseline gap-4 mt-2">
        <h1 class="h1">{symbol}</h1>
        {#if latestPrice}
          <span class="text-2xl font-semibold">
            {latestPrice.close.toLocaleString()}
          </span>
          <span
            class="text-lg"
            class:text-green-500={priceChange().value >= 0}
            class:text-red-500={priceChange().value < 0}
          >
            {priceChange().value >= 0 ? '+' : ''}{priceChange().percent.toFixed(2)}%
          </span>
        {/if}
      </div>
      {#if stock}
        <p class="text-surface-600-300-token">{stock.name}</p>
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

  {#if isLoading}
    <div class="card p-8 text-center">
      <p>Loading...</p>
    </div>
  {:else if stock}
    <!-- Price Chart -->
    <div class="card p-4">
      <h3 class="h3 mb-4">Price Chart (60 Days)</h3>
      {#if prices.length > 0}
        <PriceChart {prices} height={400} showVolume={true} showEma={true} />
      {:else}
        <p class="text-surface-600-300-token p-8 text-center">No price data available</p>
      {/if}
    </div>

    <!-- Score Section -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <!-- Score Gauges -->
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

          <!-- Score interpretation -->
          <div class="mt-6 p-4 rounded-lg bg-surface-100-800-token">
            <p class="text-sm">
              {#if score.composite_score >= 70}
                <span class="text-green-500 font-semibold">STRONG BUY</span> - Multiple strong signals
                aligned
              {:else if score.composite_score >= 50}
                <span class="text-yellow-500 font-semibold">HOLD/WATCH</span> - Mixed signals, monitor
                closely
              {:else}
                <span class="text-red-500 font-semibold">WEAK</span> - Unfavorable conditions
              {/if}
            </p>
          </div>
        {:else}
          <p class="text-surface-600-300-token text-center">No score data available</p>
        {/if}
      </div>

      <!-- Stock Info -->
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
              <dt class="text-surface-600-300-token">Open</dt>
              <dd class="font-medium">{latestPrice.open.toLocaleString()}</dd>
            </div>
            <div class="flex justify-between">
              <dt class="text-surface-600-300-token">High</dt>
              <dd class="font-medium text-green-500">{latestPrice.high.toLocaleString()}</dd>
            </div>
            <div class="flex justify-between">
              <dt class="text-surface-600-300-token">Low</dt>
              <dd class="font-medium text-red-500">{latestPrice.low.toLocaleString()}</dd>
            </div>
            <div class="flex justify-between">
              <dt class="text-surface-600-300-token">Volume</dt>
              <dd class="font-medium">{latestPrice.volume.toLocaleString()}</dd>
            </div>
          {/if}
        </dl>
      </div>
    </div>

    <!-- Price History Table -->
    <div class="card p-4">
      <h3 class="h3 mb-4">Recent Price History</h3>
      {#if prices.length > 0}
        <div class="table-container">
          <table class="table table-compact">
            <thead>
              <tr>
                <th>Date</th>
                <th class="text-right">Open</th>
                <th class="text-right">High</th>
                <th class="text-right">Low</th>
                <th class="text-right">Close</th>
                <th class="text-right">Volume</th>
              </tr>
            </thead>
            <tbody>
              {#each prices.slice().reverse().slice(0, 10) as price}
                <tr>
                  <td>{new Date(price.time).toLocaleDateString()}</td>
                  <td class="text-right">{price.open.toLocaleString()}</td>
                  <td class="text-right text-green-500">{price.high.toLocaleString()}</td>
                  <td class="text-right text-red-500">{price.low.toLocaleString()}</td>
                  <td class="text-right font-medium">{price.close.toLocaleString()}</td>
                  <td class="text-right text-surface-600-300-token">{price.volume.toLocaleString()}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {:else}
        <p class="text-surface-600-300-token text-center p-4">No price history available</p>
      {/if}
    </div>
  {/if}
</div>
