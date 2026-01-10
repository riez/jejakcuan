<script lang="ts">
  interface Mover {
    symbol: string;
    name: string;
    price: number;
    change: number;
  }

  let { 
    gainers = [] as Mover[], 
    losers = [] as Mover[],
    title = 'Top Movers'
  } = $props();
</script>

<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
  <!-- Top Gainers -->
  <div class="card p-4">
    <h3 class="h4 mb-3 text-green-500">Top Gainers</h3>
    {#if gainers.length === 0}
      <p class="text-surface-500 text-sm">No data</p>
    {:else}
      <div class="space-y-2">
        {#each gainers.slice(0, 5) as mover}
          <div class="flex items-center justify-between p-2 rounded bg-surface-100 dark:bg-surface-800">
            <div>
              <a href="/stock/{mover.symbol}" class="anchor font-bold">{mover.symbol}</a>
              <p class="text-xs text-surface-500 truncate max-w-[150px]">{mover.name}</p>
            </div>
            <div class="text-right">
              <p class="font-mono">{mover.price.toLocaleString()}</p>
              <p class="text-green-500 font-bold">+{mover.change.toFixed(2)}%</p>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Top Losers -->
  <div class="card p-4">
    <h3 class="h4 mb-3 text-red-500">Top Losers</h3>
    {#if losers.length === 0}
      <p class="text-surface-500 text-sm">No data</p>
    {:else}
      <div class="space-y-2">
        {#each losers.slice(0, 5) as mover}
          <div class="flex items-center justify-between p-2 rounded bg-surface-100 dark:bg-surface-800">
            <div>
              <a href="/stock/{mover.symbol}" class="anchor font-bold">{mover.symbol}</a>
              <p class="text-xs text-surface-500 truncate max-w-[150px]">{mover.name}</p>
            </div>
            <div class="text-right">
              <p class="font-mono">{mover.price.toLocaleString()}</p>
              <p class="text-red-500 font-bold">{mover.change.toFixed(2)}%</p>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>
