<script lang="ts">
  import { onMount } from 'svelte';
  import { Table, ProgressRadial } from '@skeletonlabs/skeleton';
  import type { TableSource } from '@skeletonlabs/skeleton';
  import { api, type Stock, type StockScore } from '$lib/api';
  import { goto } from '$app/navigation';

  let stocks = $state<Stock[]>([]);
  let scores = $state<Map<string, StockScore>>(new Map());
  let isLoading = $state(true);
  let error = $state<string | null>(null);
  let searchTerm = $state('');
  let selectedSector = $state<string | null>(null);

  // Combine stocks with their scores
  let stocksWithScores = $derived(() => {
    return stocks
      .filter((stock) => {
        const matchesSearch =
          !searchTerm ||
          stock.symbol.toLowerCase().includes(searchTerm.toLowerCase()) ||
          stock.name.toLowerCase().includes(searchTerm.toLowerCase());
        const matchesSector = !selectedSector || stock.sector === selectedSector;
        return matchesSearch && matchesSector;
      })
      .map((stock) => {
        const score = scores.get(stock.symbol);
        return {
          ...stock,
          composite_score: score?.composite_score ?? null,
          technical_score: score?.technical_score ?? null,
          fundamental_score: score?.fundamental_score ?? null,
          sentiment_score: score?.sentiment_score ?? null,
          ml_score: score?.ml_score ?? null
        };
      })
      .sort((a, b) => (b.composite_score ?? 0) - (a.composite_score ?? 0));
  });

  // Get unique sectors
  let sectors = $derived(() => {
    const sectorSet = new Set(stocks.map((s) => s.sector).filter(Boolean));
    return Array.from(sectorSet).sort();
  });

  onMount(async () => {
    try {
      // Fetch stocks
      const stocksResponse = await api.getStocks();
      stocks = stocksResponse.stocks;

      // Fetch scores
      const scoresResponse = await api.getTopScores(500);
      scores = new Map(scoresResponse.map((s) => [s.symbol, s]));
    } catch (e) {
      error = (e as Error).message;
    } finally {
      isLoading = false;
    }
  });

  function getScoreClass(score: number | null): string {
    if (score === null) return 'variant-soft';
    if (score >= 70) return 'variant-filled-success';
    if (score >= 50) return 'variant-filled-warning';
    return 'variant-filled-error';
  }

  // Table source for Skeleton Table component
  let tableSource = $derived<TableSource>({
    head: ['Symbol', 'Name', 'Sector', 'Score', 'Tech', 'Fund', 'Sent', 'ML'],
    body: stocksWithScores().map((stock) => [
      stock.symbol,
      stock.name.length > 30 ? stock.name.substring(0, 30) + '...' : stock.name,
      stock.sector ?? '-',
      stock.composite_score?.toFixed(0) ?? '-',
      stock.technical_score?.toFixed(0) ?? '-',
      stock.fundamental_score?.toFixed(0) ?? '-',
      stock.sentiment_score?.toFixed(0) ?? '-',
      stock.ml_score?.toFixed(0) ?? '-'
    ]),
    meta: stocksWithScores().map((stock) => [stock.symbol])
  });

  function handleTableSelect(e: CustomEvent<string[]>) {
    const symbol = e.detail[0];
    if (symbol) {
      goto(`/stock/${symbol}`);
    }
  }
</script>

<svelte:head>
  <title>JejakCuan - Stock Screener</title>
</svelte:head>

<div class="space-y-4">
  <div class="flex items-center justify-between">
    <h1 class="h1">Stock Screener</h1>
    <span class="badge variant-soft">{stocksWithScores().length} stocks</span>
  </div>

  {#if error}
    <aside class="alert variant-filled-error">
      <div class="alert-message">
        <p>{error}</p>
      </div>
    </aside>
  {/if}

  <!-- Filters -->
  <div class="card p-4">
    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
      <label class="label">
        <span>Search</span>
        <input
          type="text"
          bind:value={searchTerm}
          class="input"
          placeholder="Search by symbol or name..."
        />
      </label>

      <label class="label">
        <span>Sector</span>
        <select bind:value={selectedSector} class="select">
          <option value={null}>All Sectors</option>
          {#each sectors() as sector}
            <option value={sector}>{sector}</option>
          {/each}
        </select>
      </label>
    </div>
  </div>

  <!-- Stocks Table -->
  <div class="card">
    {#if isLoading}
      <div class="p-8 flex items-center justify-center">
        <ProgressRadial stroke={100} meter="stroke-primary-500" track="stroke-primary-500/30" />
      </div>
    {:else}
      <Table
        source={tableSource}
        interactive={true}
        on:selected={handleTableSelect}
        regionHeadCell="text-left"
        regionBodyCell="text-left"
      />
    {/if}
  </div>
</div>
