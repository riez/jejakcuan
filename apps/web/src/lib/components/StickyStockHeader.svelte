<script lang="ts">
  import type { Stock, StockPrice, StockScore } from '$lib/api';

  let {
    stock,
    latestPrice,
    score,
    inWatchlist,
    dataFresh,
    onToggleWatchlist,
    onRefreshData,
  }: {
    stock: Stock;
    latestPrice: StockPrice | null;
    score: StockScore | null;
    inWatchlist: boolean;
    dataFresh: boolean;
    onToggleWatchlist: () => void;
    onRefreshData: () => void;
  } = $props();

  let priceChange = $derived(() => {
    if (!latestPrice) return { value: 0, percent: 0 };
    const prev = latestPrice.open;
    const curr = latestPrice.close;
    return {
      value: curr - prev,
      percent: prev > 0 ? ((curr - prev) / prev) * 100 : 0,
    };
  });

  let signal = $derived(() => {
    if (!score) return { label: 'LOADING', color: 'variant-ghost-surface' };
    const composite = Number(score.composite_score);
    if (composite >= 70) return { label: 'BUY', color: 'variant-filled-success' };
    if (composite >= 50) return { label: 'HOLD', color: 'variant-filled-warning' };
    return { label: 'SELL', color: 'variant-filled-error' };
  });
</script>

<header
  class="sticky top-0 z-50 bg-surface-100-800-token border-b border-surface-300-600-token backdrop-blur-sm"
>
  <div class="container mx-auto px-4 py-3">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-4">
        <a href="/screener" class="btn btn-sm variant-ghost-surface">
          <span class="text-lg">&larr;</span>
        </a>
        <div>
          <div class="flex items-center gap-2">
            <span class="text-2xl font-bold font-mono">{stock.symbol}</span>
            <span class="badge {signal().color} text-sm">{signal().label}</span>
          </div>
          <div class="flex items-center gap-2 text-sm">
            <span class="font-mono font-semibold">
              {latestPrice?.close?.toLocaleString('id-ID') ?? '-'}
            </span>
            <span
              class={priceChange().percent >= 0 ? 'text-success-500' : 'text-error-500'}
            >
              {priceChange().percent >= 0 ? '+' : ''}{priceChange().percent.toFixed(2)}%
            </span>
          </div>
        </div>
      </div>

      <div class="flex items-center gap-2">
        {#if !dataFresh}
          <button
            class="badge variant-soft-warning cursor-pointer"
            onclick={onRefreshData}
          >
            Stale
          </button>
        {/if}
        <button
          class="btn btn-sm {inWatchlist ? 'variant-filled-primary' : 'variant-ghost-primary'}"
          onclick={onToggleWatchlist}
          aria-label={inWatchlist ? 'Remove from watchlist' : 'Add to watchlist'}
        >
          {inWatchlist ? '★' : '☆'}
        </button>
      </div>
    </div>
  </div>
</header>
