<script lang="ts">
  interface SectorData {
    name: string;
    change: number;
    stocks: number;
  }

  let { sectors = [] as SectorData[] } = $props();

  const getColor = (change: number) => {
    if (change >= 3) return 'bg-green-600';
    if (change >= 1.5) return 'bg-green-500';
    if (change >= 0.5) return 'bg-green-400';
    if (change >= 0) return 'bg-green-300';
    if (change >= -0.5) return 'bg-red-300';
    if (change >= -1.5) return 'bg-red-400';
    if (change >= -3) return 'bg-red-500';
    return 'bg-red-600';
  };

  const getTextColor = (change: number) => {
    if (Math.abs(change) >= 1.5) return 'text-white';
    return 'text-surface-900';
  };
</script>

<div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-2">
  {#each sectors as sector}
    <div 
      class="p-4 rounded-lg {getColor(sector.change)} {getTextColor(sector.change)} transition-all hover:scale-105 cursor-pointer"
    >
      <p class="font-bold text-sm truncate">{sector.name}</p>
      <p class="text-2xl font-bold">
        {sector.change >= 0 ? '+' : ''}{sector.change.toFixed(2)}%
      </p>
      <p class="text-xs opacity-80">{sector.stocks} stocks</p>
    </div>
  {/each}
</div>
