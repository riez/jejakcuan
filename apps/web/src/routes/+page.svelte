<script lang="ts">
  // Placeholder for screener data
  const stocks = [
    { symbol: 'BBCA', name: 'Bank Central Asia', price: 9850, change: 1.2, score: 78 },
    { symbol: 'TLKM', name: 'Telkom Indonesia', price: 3420, change: -0.5, score: 65 },
    { symbol: 'ASII', name: 'Astra International', price: 5425, change: 0.8, score: 72 },
  ];
</script>

<svelte:head>
  <title>JejakCuan - Stock Screener</title>
</svelte:head>

<div class="space-y-4">
  <h1 class="h1">Stock Screener</h1>
  
  <div class="card p-4">
    <header class="card-header">
      <h3 class="h3">IDX Stocks</h3>
    </header>
    <section class="p-4">
      <div class="table-container">
        <table class="table table-hover">
          <thead>
            <tr>
              <th>Symbol</th>
              <th>Name</th>
              <th class="text-right">Price</th>
              <th class="text-right">Change %</th>
              <th class="text-right">Score</th>
            </tr>
          </thead>
          <tbody>
            {#each stocks as stock}
              <tr>
                <td class="font-bold">{stock.symbol}</td>
                <td>{stock.name}</td>
                <td class="text-right">{stock.price.toLocaleString()}</td>
                <td class="text-right" class:text-green-500={stock.change > 0} class:text-red-500={stock.change < 0}>
                  {stock.change > 0 ? '+' : ''}{stock.change}%
                </td>
                <td class="text-right">
                  <span class="badge" class:variant-filled-success={stock.score >= 70} class:variant-filled-warning={stock.score >= 50 && stock.score < 70} class:variant-filled-error={stock.score < 50}>
                    {stock.score}
                  </span>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </section>
  </div>
</div>
