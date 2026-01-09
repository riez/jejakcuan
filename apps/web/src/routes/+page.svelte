<script lang="ts">
  import { onMount } from 'svelte';
  import { api, type Stock, type StockScore } from '$lib/api';

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
      <div class="p-8 text-center">
        <p>Loading stocks...</p>
      </div>
    {:else}
      <div class="table-container">
        <table class="table table-hover">
          <thead>
            <tr>
              <th>Symbol</th>
              <th>Name</th>
              <th>Sector</th>
              <th class="text-center">Score</th>
              <th class="text-center">Tech</th>
              <th class="text-center">Fund</th>
              <th class="text-center">Sent</th>
              <th class="text-center">ML</th>
            </tr>
          </thead>
          <tbody>
            {#each stocksWithScores() as stock}
              <tr>
                <td>
                  <a href="/stock/{stock.symbol}" class="anchor font-bold">
                    {stock.symbol}
                  </a>
                </td>
                <td class="max-w-[200px] truncate">{stock.name}</td>
                <td class="text-surface-600-300-token">{stock.sector ?? '-'}</td>
                <td class="text-center">
                  <span class="badge {getScoreClass(stock.composite_score)}">
                    {stock.composite_score?.toFixed(0) ?? '-'}
                  </span>
                </td>
                <td class="text-center text-sm">{stock.technical_score?.toFixed(0) ?? '-'}</td>
                <td class="text-center text-sm">{stock.fundamental_score?.toFixed(0) ?? '-'}</td>
                <td class="text-center text-sm">{stock.sentiment_score?.toFixed(0) ?? '-'}</td>
                <td class="text-center text-sm">{stock.ml_score?.toFixed(0) ?? '-'}</td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </div>
</div>
