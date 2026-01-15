<script lang="ts">
  import { onMount } from 'svelte';
  import { ProgressRadial, ProgressBar } from '@skeletonlabs/skeleton';
  import { SectorHeatmap, TopMovers, IndexSummary } from '$lib/components';

  // Mock data - in production, fetch from API
  interface IndexData {
    name: string;
    value: number;
    change: number;
    changePercent: number;
    high: number;
    low: number;
    volume: number;
  }

  interface SectorData {
    name: string;
    change: number;
    stocks: number;
  }

  interface Mover {
    symbol: string;
    name: string;
    price: number;
    change: number;
  }

  let ihsg = $state<IndexData | null>(null);
  let sectors = $state<SectorData[]>([]);
  let gainers = $state<Mover[]>([]);
  let losers = $state<Mover[]>([]);
  let isLoading = $state(true);
  let lastUpdate = $state<Date | null>(null);

  onMount(async () => {
    await loadMarketData();
  });

  async function loadMarketData() {
    isLoading = true;
    
    // TODO: Replace with actual API calls
    // Simulated data for now
    await new Promise(resolve => setTimeout(resolve, 500));
    
    ihsg = {
      name: 'IHSG',
      value: 7245.32,
      change: 45.67,
      changePercent: 0.63,
      high: 7268.54,
      low: 7198.21,
      volume: 12_500_000_000,
    };
    
    sectors = [
      { name: 'Banking', change: 1.25, stocks: 45 },
      { name: 'Mining', change: -0.85, stocks: 38 },
      { name: 'Consumer', change: 0.42, stocks: 52 },
      { name: 'Infrastructure', change: 2.15, stocks: 28 },
      { name: 'Telecoms', change: -0.32, stocks: 15 },
      { name: 'Property', change: 1.68, stocks: 42 },
      { name: 'Energy', change: -1.45, stocks: 22 },
      { name: 'Healthcare', change: 0.95, stocks: 18 },
      { name: 'Technology', change: 3.25, stocks: 12 },
      { name: 'Industrial', change: 0.15, stocks: 35 },
      { name: 'Basic Materials', change: -0.55, stocks: 25 },
      { name: 'Transportation', change: 1.85, stocks: 20 },
    ];
    
    gainers = [
      { symbol: 'GOTO', name: 'GoTo Gojek Tokopedia', price: 86, change: 7.50 },
      { symbol: 'BRIS', name: 'Bank BRI Syariah', price: 2450, change: 6.52 },
      { symbol: 'ARTO', name: 'Bank Jago', price: 3250, change: 5.18 },
      { symbol: 'EMTK', name: 'Elang Mahkota Teknologi', price: 1875, change: 4.75 },
      { symbol: 'MDKA', name: 'Merdeka Copper Gold', price: 2680, change: 4.28 },
    ];
    
    losers = [
      { symbol: 'ADRO', name: 'Adaro Energy', price: 2340, change: -4.50 },
      { symbol: 'ITMG', name: 'Indo Tambangraya Megah', price: 27800, change: -3.82 },
      { symbol: 'PTBA', name: 'Bukit Asam', price: 2580, change: -3.35 },
      { symbol: 'INDY', name: 'Indika Energy', price: 1425, change: -2.74 },
      { symbol: 'BYAN', name: 'Bayan Resources', price: 14500, change: -2.36 },
    ];
    
    lastUpdate = new Date();
    isLoading = false;
  }
</script>

<svelte:head>
  <title>Market Overview - JejakCuan</title>
</svelte:head>

<div class="space-y-6">
  <div class="flex items-center justify-between">
    <h1 class="h1">Market Overview</h1>
    <div class="flex items-center gap-4">
      {#if lastUpdate}
        <p class="text-sm text-surface-500">
          Last updated: {lastUpdate.toLocaleTimeString()}
        </p>
      {/if}
      <button 
        onclick={loadMarketData}
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

  <!-- IHSG Summary -->
  <IndexSummary index={ihsg} />

  <!-- Market Stats -->
  <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
    <div class="card p-4 text-center">
      <p class="text-sm text-surface-500">Advancing</p>
      <p class="text-2xl font-bold text-green-500">285</p>
    </div>
    <div class="card p-4 text-center">
      <p class="text-sm text-surface-500">Declining</p>
      <p class="text-2xl font-bold text-red-500">198</p>
    </div>
    <div class="card p-4 text-center">
      <p class="text-sm text-surface-500">Unchanged</p>
      <p class="text-2xl font-bold text-surface-500">124</p>
    </div>
    <div class="card p-4 text-center">
      <p class="text-sm text-surface-500">Active Stocks</p>
      <p class="text-2xl font-bold">607</p>
    </div>
  </div>

  <!-- Sector Heatmap -->
  <div class="card p-4">
    <h2 class="h3 mb-4">Sector Performance</h2>
    <SectorHeatmap {sectors} />
  </div>

  <!-- Top Movers -->
  <TopMovers {gainers} {losers} />

  <!-- Market Breadth using ProgressBar -->
  <div class="card p-4">
    <h2 class="h3 mb-4">Market Breadth</h2>
    <ProgressBar 
      value={47} 
      max={100} 
      height="h-6"
      meter="bg-gradient-to-r from-green-500 via-yellow-500 to-red-500"
      track="bg-surface-200-700-token"
    />
    <div class="flex justify-between mt-2 text-sm">
      <span class="text-green-500">47% Advancing</span>
      <span class="text-surface-500">20% Unchanged</span>
      <span class="text-red-500">33% Declining</span>
    </div>
  </div>
</div>
