<script lang="ts">
  interface Filters {
    type: 'all' | 'buy' | 'sell' | 'hold';
    strength: 'all' | 'strong' | 'moderate' | 'weak';
    minScore: number;
    sector: string;
  }

  let { 
    filters = $bindable<Filters>({ type: 'all', strength: 'all', minScore: 0, sector: '' }),
    sectors = [] as string[],
    onApply = () => {},
  } = $props();
</script>

<div class="card p-4">
  <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
    <!-- Signal Type -->
    <div>
      <label class="label">
        <span class="text-sm">Signal Type</span>
        <select class="select" bind:value={filters.type}>
          <option value="all">All</option>
          <option value="buy">Buy</option>
          <option value="sell">Sell</option>
          <option value="hold">Hold</option>
        </select>
      </label>
    </div>

    <!-- Strength -->
    <div>
      <label class="label">
        <span class="text-sm">Strength</span>
        <select class="select" bind:value={filters.strength}>
          <option value="all">All</option>
          <option value="strong">Strong</option>
          <option value="moderate">Moderate</option>
          <option value="weak">Weak</option>
        </select>
      </label>
    </div>

    <!-- Min Score -->
    <div>
      <label class="label">
        <span class="text-sm">Min Score: {filters.minScore}</span>
        <input 
          type="range" 
          class="range"
          min="0" 
          max="100" 
          step="10"
          bind:value={filters.minScore}
        />
      </label>
    </div>

    <!-- Sector -->
    <div>
      <label class="label">
        <span class="text-sm">Sector</span>
        <select class="select" bind:value={filters.sector}>
          <option value="">All Sectors</option>
          {#each sectors as sector}
            <option value={sector}>{sector}</option>
          {/each}
        </select>
      </label>
    </div>
  </div>
</div>
