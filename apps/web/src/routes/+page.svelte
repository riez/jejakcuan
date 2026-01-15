<script lang="ts">
  import { onMount } from 'svelte';
  import { ProgressRadial } from '@skeletonlabs/skeleton';
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
      const stocksResponse = await api.getStocks();
      stocks = stocksResponse.stocks;
      const scoresResponse = await api.getTopScores(500);
      scores = new Map(scoresResponse.map((s) => [s.symbol, s]));
    } catch (e) {
      error = (e as Error).message;
    } finally {
      isLoading = false;
    }
  });

  function getScoreColor(score: number | null): string {
    if (score === null) return 'text-slate-400';
    if (score >= 70) return 'text-emerald-500 dark:text-emerald-400 font-bold';
    if (score >= 50) return 'text-amber-500 dark:text-amber-400';
    return 'text-rose-500 dark:text-rose-400';
  }

  function navigateToStock(symbol: string) {
    goto(`/stock/${symbol}`);
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
      <div class="table-container">
        <table class="table table-hover">
          <thead>
            <tr>
              <th class="text-slate-900 dark:text-slate-100">Symbol</th>
              <th class="text-slate-900 dark:text-slate-100">Name</th>
              <th class="text-slate-900 dark:text-slate-100">Sector</th>
              <th class="text-slate-900 dark:text-slate-100 text-right">Score</th>
              <th class="text-slate-900 dark:text-slate-100 text-right">Tech</th>
              <th class="text-slate-900 dark:text-slate-100 text-right">Fund</th>
              <th class="text-slate-900 dark:text-slate-100 text-right">Sent</th>
              <th class="text-slate-900 dark:text-slate-100 text-right">ML</th>
            </tr>
          </thead>
          <tbody>
            {#each stocksWithScores() as stock (stock.symbol)}
              <tr 
                class="cursor-pointer hover:bg-primary-500/10"
                onclick={() => navigateToStock(stock.symbol)}
              >
                <td class="font-mono font-bold text-primary-600 dark:text-primary-400">{stock.symbol}</td>
                <td class="text-slate-700 dark:text-slate-300">{stock.name.length > 35 ? stock.name.substring(0, 35) + '...' : stock.name}</td>
                <td class="text-slate-600 dark:text-slate-400">{stock.sector ?? '-'}</td>
                <td class="text-right font-tabular {getScoreColor(stock.composite_score)}">{stock.composite_score?.toFixed(0) ?? '-'}</td>
                <td class="text-right font-tabular {getScoreColor(stock.technical_score)}">{stock.technical_score?.toFixed(0) ?? '-'}</td>
                <td class="text-right font-tabular {getScoreColor(stock.fundamental_score)}">{stock.fundamental_score?.toFixed(0) ?? '-'}</td>
                <td class="text-right font-tabular {getScoreColor(stock.sentiment_score)}">{stock.sentiment_score?.toFixed(0) ?? '-'}</td>
                <td class="text-right font-tabular {getScoreColor(stock.ml_score)}">{stock.ml_score?.toFixed(0) ?? '-'}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </div>
</div>
