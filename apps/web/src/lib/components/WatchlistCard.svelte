<script lang="ts">
  import type { Stock, StockScore } from '$lib/api';
  import { ScoreGauge } from '$lib/components';

  interface Props {
    symbol: string;
    stock?: Stock | null;
    score?: StockScore | null;
    latestPrice?: number;
    priceChange?: number;
    onRemove?: () => void;
  }

  let { symbol, stock, score, latestPrice, priceChange, onRemove }: Props = $props();

  const getPriceChangeClass = (change: number | undefined) => {
    if (!change) return 'text-surface-500';
    return change > 0 ? 'text-green-500' : change < 0 ? 'text-red-500' : 'text-surface-500';
  };
</script>

<div class="card p-4 hover:ring-2 ring-primary-500 transition-all cursor-grab active:cursor-grabbing">
  <div class="flex items-start justify-between gap-4">
    <!-- Stock Info -->
    <div class="flex-1 min-w-0">
      <a href="/stock/{symbol}" class="anchor">
        <h3 class="h4 font-bold truncate">{symbol}</h3>
      </a>
      {#if stock}
        <p class="text-sm text-surface-600-300-token truncate">{stock.name}</p>
        <p class="text-xs text-surface-500">{stock.sector ?? 'N/A'}</p>
      {/if}
    </div>

    <!-- Price -->
    <div class="text-right">
      {#if latestPrice}
        <p class="font-bold text-lg">{latestPrice.toLocaleString()}</p>
        {#if priceChange !== undefined}
          <p class="text-sm {getPriceChangeClass(priceChange)}">
            {priceChange > 0 ? '+' : ''}{priceChange.toFixed(2)}%
          </p>
        {/if}
      {:else}
        <p class="text-surface-500">-</p>
      {/if}
    </div>

    <!-- Score -->
    {#if score}
      <div class="flex-shrink-0">
        <ScoreGauge score={score.composite_score} label="" size="sm" />
      </div>
    {/if}
  </div>

  <!-- Score Breakdown -->
  {#if score}
    <div class="mt-3 pt-3 border-t border-surface-200 dark:border-surface-700">
      <div class="grid grid-cols-4 gap-2 text-center text-xs">
        <div>
          <p class="text-surface-500">Tech</p>
          <p class="font-semibold">{score.technical_score.toFixed(0)}</p>
        </div>
        <div>
          <p class="text-surface-500">Fund</p>
          <p class="font-semibold">{score.fundamental_score.toFixed(0)}</p>
        </div>
        <div>
          <p class="text-surface-500">Sent</p>
          <p class="font-semibold">{score.sentiment_score.toFixed(0)}</p>
        </div>
        <div>
          <p class="text-surface-500">ML</p>
          <p class="font-semibold">{score.ml_score.toFixed(0)}</p>
        </div>
      </div>
    </div>
  {/if}

  <!-- Actions -->
  <div class="mt-3 flex justify-end gap-2">
    <a href="/stock/{symbol}" class="btn btn-sm variant-ghost-primary">Details</a>
    {#if onRemove}
      <button onclick={onRemove} class="btn btn-sm variant-ghost-error">Remove</button>
    {/if}
  </div>
</div>
