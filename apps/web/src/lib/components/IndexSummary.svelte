<script lang="ts">
  interface IndexData {
    name: string;
    value: number;
    change: number;
    changePercent: number;
    high: number;
    low: number;
    volume: number;
  }

  let { index = null as IndexData | null } = $props();
</script>

{#if index}
  <div class="card p-6">
    <div class="flex items-start justify-between">
      <div>
        <h2 class="h2">{index.name}</h2>
        <p class="text-4xl font-bold mt-2">{index.value.toLocaleString()}</p>
      </div>
      <div class="text-right">
        <p class="text-2xl font-bold {index.change >= 0 ? 'text-green-500' : 'text-red-500'}">
          {index.change >= 0 ? '+' : ''}{index.change.toFixed(2)}
        </p>
        <p class="text-lg {index.changePercent >= 0 ? 'text-green-500' : 'text-red-500'}">
          ({index.changePercent >= 0 ? '+' : ''}{index.changePercent.toFixed(2)}%)
        </p>
      </div>
    </div>
    
    <div class="grid grid-cols-3 gap-4 mt-6 pt-4 border-t border-surface-200 dark:border-surface-700">
      <div>
        <p class="text-sm text-surface-500">High</p>
        <p class="font-bold text-green-500">{index.high.toLocaleString()}</p>
      </div>
      <div>
        <p class="text-sm text-surface-500">Low</p>
        <p class="font-bold text-red-500">{index.low.toLocaleString()}</p>
      </div>
      <div>
        <p class="text-sm text-surface-500">Volume</p>
        <p class="font-bold">{(index.volume / 1_000_000_000).toFixed(2)}B</p>
      </div>
    </div>
  </div>
{:else}
  <div class="card p-6 animate-pulse">
    <div class="h-8 bg-surface-300 dark:bg-surface-700 rounded w-1/4 mb-4"></div>
    <div class="h-12 bg-surface-300 dark:bg-surface-700 rounded w-1/3"></div>
  </div>
{/if}
