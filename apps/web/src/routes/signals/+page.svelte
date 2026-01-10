<script lang="ts">
  import { onMount } from 'svelte';
  import { SignalCard, SignalFilters } from '$lib/components';

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
    <h1 class="h1">Today's Signals</h1>
    <button 
      onclick={loadSignals}
      class="btn variant-ghost-primary"
      disabled={isLoading}
    >
      {isLoading ? 'Loading...' : 'Refresh'}
    </button>
  </div>

  <!-- Signal Stats -->
  <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
    <div class="card p-4 text-center">
      <p class="text-2xl font-bold">{stats.total}</p>
      <p class="text-sm text-surface-500">Total Signals</p>
    </div>
    <div class="card p-4 text-center">
      <p class="text-2xl font-bold text-green-500">{stats.buySignals}</p>
      <p class="text-sm text-surface-500">Buy Signals</p>
    </div>
    <div class="card p-4 text-center">
      <p class="text-2xl font-bold text-red-500">{stats.sellSignals}</p>
      <p class="text-sm text-surface-500">Sell Signals</p>
    </div>
    <div class="card p-4 text-center">
      <p class="text-2xl font-bold text-primary-500">{stats.strongSignals}</p>
      <p class="text-sm text-surface-500">Strong Signals</p>
    </div>
  </div>

  <!-- Filters -->
  <SignalFilters bind:filters {sectors} onApply={applyFilters} />

  <!-- Signal List -->
  {#if isLoading}
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      {#each [1, 2, 3, 4] as _}
        <div class="card p-4 animate-pulse">
          <div class="h-6 bg-surface-300 dark:bg-surface-700 rounded w-1/4 mb-3"></div>
          <div class="h-4 bg-surface-300 dark:bg-surface-700 rounded w-1/2 mb-2"></div>
          <div class="h-16 bg-surface-300 dark:bg-surface-700 rounded"></div>
        </div>
      {/each}
    </div>
  {:else if filteredSignals.length === 0}
    <div class="card p-8 text-center">
      <p class="text-surface-600-300-token">No signals match your filters.</p>
      <button 
        onclick={() => { filters = { type: 'all', strength: 'all', minScore: 0, sector: '' }; }}
        class="btn variant-ghost-primary mt-4"
      >
        Clear Filters
      </button>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      {#each filteredSignals as signal (signal.id)}
        <SignalCard {signal} />
      {/each}
    </div>
  {/if}

  <!-- Disclaimer -->
  <div class="card variant-soft-warning p-4">
    <p class="text-sm">
      <strong>Disclaimer:</strong> These signals are generated by algorithmic analysis and should not be considered as financial advice. 
      Always do your own research before making investment decisions.
    </p>
  </div>
</div>
