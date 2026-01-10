<script lang="ts">
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
  }

  let { signal }: { signal: Signal } = $props();

  const typeColors = {
    buy: 'variant-filled-success',
    sell: 'variant-filled-error',
    hold: 'variant-filled-warning',
  };

  const strengthIcons = {
    strong: '↑↑↑',
    moderate: '↑↑',
    weak: '↑',
  };
</script>

<div class="card p-4 hover:ring-2 ring-primary-500 transition-all">
  <div class="flex items-start justify-between gap-4">
    <!-- Signal Type Badge -->
    <div class="flex items-center gap-3">
      <span class="badge {typeColors[signal.type]} text-lg px-3 py-1">
        {signal.type.toUpperCase()}
      </span>
      <span class="text-surface-500" title={signal.strength}>
        {strengthIcons[signal.strength]}
      </span>
    </div>
    
    <!-- Score -->
    <div class="text-right">
      <p class="text-2xl font-bold {signal.score >= 70 ? 'text-green-500' : signal.score >= 50 ? 'text-yellow-500' : 'text-red-500'}">
        {signal.score}
      </p>
      <p class="text-xs text-surface-500">Score</p>
    </div>
  </div>

  <!-- Stock Info -->
  <div class="mt-3">
    <a href="/stock/{signal.symbol}" class="anchor">
      <h3 class="h4 font-bold">{signal.symbol}</h3>
    </a>
    <p class="text-sm text-surface-600-300-token">{signal.stockName}</p>
  </div>

  <!-- Reason -->
  <p class="mt-2 text-sm bg-surface-100 dark:bg-surface-800 p-2 rounded">
    {signal.reason}
  </p>

  <!-- Price Targets -->
  <div class="mt-3 grid grid-cols-3 gap-2 text-sm">
    <div>
      <p class="text-surface-500">Entry</p>
      <p class="font-mono font-bold">{signal.priceAtSignal.toLocaleString()}</p>
    </div>
    {#if signal.targetPrice}
      <div>
        <p class="text-surface-500">Target</p>
        <p class="font-mono font-bold text-green-500">{signal.targetPrice.toLocaleString()}</p>
      </div>
    {/if}
    {#if signal.stopLoss}
      <div>
        <p class="text-surface-500">Stop Loss</p>
        <p class="font-mono font-bold text-red-500">{signal.stopLoss.toLocaleString()}</p>
      </div>
    {/if}
  </div>

  <!-- Indicators -->
  {#if signal.indicators.length > 0}
    <div class="mt-3 flex flex-wrap gap-1">
      {#each signal.indicators as indicator}
        <span class="badge variant-soft-primary text-xs">{indicator}</span>
      {/each}
    </div>
  {/if}

  <!-- Timestamp -->
  <p class="mt-3 text-xs text-surface-500">
    {signal.timestamp.toLocaleString()}
  </p>
</div>
