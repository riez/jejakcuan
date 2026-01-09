<script lang="ts">
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import { api, type Stock, type StockScore, type StockPrice } from '$lib/api';

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
        api.getStockPrices(symbol, 30),
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

  function getScoreClass(score: number | null): string {
    if (score === null) return 'variant-soft';
    if (score >= 70) return 'variant-filled-success';
    if (score >= 50) return 'variant-filled-warning';
    return 'variant-filled-error';
  }
</script>

<svelte:head>
  <title>{symbol} - JejakCuan</title>
</svelte:head>

<div class="space-y-4">
  <div class="flex items-center justify-between">
    <div>
      <a href="/" class="anchor text-sm">&larr; Back to Screener</a>
      <h1 class="h1">{symbol}</h1>
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
    <p>Loading...</p>
  {:else if stock}
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
      <!-- Score Card -->
      <div class="card p-4">
        <h3 class="h3 mb-4">Score Breakdown</h3>
        {#if score}
          <div class="space-y-3">
            <div class="flex justify-between items-center">
              <span>Composite Score</span>
              <span class="badge text-lg {getScoreClass(score.composite_score)}">
                {score.composite_score.toFixed(0)}
              </span>
            </div>
            <hr class="opacity-20" />
            <div class="flex justify-between items-center">
              <span>Technical</span>
              <span class="badge {getScoreClass(score.technical_score)}">
                {score.technical_score.toFixed(0)}
              </span>
            </div>
            <div class="flex justify-between items-center">
              <span>Fundamental</span>
              <span class="badge {getScoreClass(score.fundamental_score)}">
                {score.fundamental_score.toFixed(0)}
              </span>
            </div>
            <div class="flex justify-between items-center">
              <span>Sentiment</span>
              <span class="badge {getScoreClass(score.sentiment_score)}">
                {score.sentiment_score.toFixed(0)}
              </span>
            </div>
            <div class="flex justify-between items-center">
              <span>ML Prediction</span>
              <span class="badge {getScoreClass(score.ml_score)}">
                {score.ml_score.toFixed(0)}
              </span>
            </div>
          </div>
        {:else}
          <p class="text-surface-600-300-token">No score data available</p>
        {/if}
      </div>

      <!-- Info Card -->
      <div class="card p-4">
        <h3 class="h3 mb-4">Stock Info</h3>
        <dl class="space-y-2">
          <div class="flex justify-between">
            <dt class="text-surface-600-300-token">Sector</dt>
            <dd>{stock.sector ?? '-'}</dd>
          </div>
          <div class="flex justify-between">
            <dt class="text-surface-600-300-token">Subsector</dt>
            <dd>{stock.subsector ?? '-'}</dd>
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

    <!-- Price History -->
    <div class="card p-4">
      <h3 class="h3 mb-4">Recent Prices ({prices.length} days)</h3>
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
                  <td class="text-right">{price.high.toLocaleString()}</td>
                  <td class="text-right">{price.low.toLocaleString()}</td>
                  <td class="text-right">{price.close.toLocaleString()}</td>
                  <td class="text-right">{price.volume.toLocaleString()}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        </div>
      {:else}
        <p class="text-surface-600-300-token">No price history available</p>
      {/if}
    </div>
  {/if}
</div>
