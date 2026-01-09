<script lang="ts">
  interface FundamentalData {
    pe_ratio: number | null;
    pb_ratio: number | null;
    ev_ebitda: number | null;
    roe: number | null;
    roa: number | null;
    profit_margin: number | null;
    debt_to_equity: number | null;
    dcf_intrinsic_value: number | null;
    dcf_margin_of_safety: number | null;
  }

  let {
    data = null as FundamentalData | null,
    currentPrice = 0
  } = $props();

  const formatNumber = (n: number | null, decimals = 2) => {
    if (n === null || n === undefined) return '-';
    return n.toFixed(decimals);
  };

  const formatPercent = (n: number | null) => {
    if (n === null || n === undefined) return '-';
    return `${n.toFixed(2)}%`;
  };

  const getColorClass = (value: number | null, threshold: number, higher_is_better = true) => {
    if (value === null) return 'text-surface-600-300-token';
    if (higher_is_better) {
      return value >= threshold ? 'text-green-500' : value < threshold * 0.5 ? 'text-red-500' : 'text-yellow-500';
    } else {
      return value <= threshold ? 'text-green-500' : value > threshold * 2 ? 'text-red-500' : 'text-yellow-500';
    }
  };
</script>

{#if data}
  <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
    <!-- Valuation Ratios -->
    <div class="card p-4">
      <h4 class="text-xs uppercase text-surface-500 mb-2">P/E Ratio</h4>
      <p class="text-2xl font-bold {getColorClass(data.pe_ratio, 15, false)}">
        {formatNumber(data.pe_ratio)}
      </p>
      <p class="text-xs text-surface-500">Lower is better</p>
    </div>

    <div class="card p-4">
      <h4 class="text-xs uppercase text-surface-500 mb-2">P/B Ratio</h4>
      <p class="text-2xl font-bold {getColorClass(data.pb_ratio, 1.5, false)}">
        {formatNumber(data.pb_ratio)}
      </p>
      <p class="text-xs text-surface-500">&lt;1 = below book value</p>
    </div>

    <div class="card p-4">
      <h4 class="text-xs uppercase text-surface-500 mb-2">EV/EBITDA</h4>
      <p class="text-2xl font-bold {getColorClass(data.ev_ebitda, 10, false)}">
        {formatNumber(data.ev_ebitda)}
      </p>
      <p class="text-xs text-surface-500">&lt;8 attractive</p>
    </div>

    <div class="card p-4">
      <h4 class="text-xs uppercase text-surface-500 mb-2">D/E Ratio</h4>
      <p class="text-2xl font-bold {getColorClass(data.debt_to_equity, 1.0, false)}">
        {formatNumber(data.debt_to_equity)}
      </p>
      <p class="text-xs text-surface-500">&lt;1.0 preferred</p>
    </div>

    <!-- Profitability -->
    <div class="card p-4">
      <h4 class="text-xs uppercase text-surface-500 mb-2">ROE</h4>
      <p class="text-2xl font-bold {getColorClass(data.roe, 15, true)}">
        {formatPercent(data.roe)}
      </p>
      <p class="text-xs text-surface-500">&gt;15% is good</p>
    </div>

    <div class="card p-4">
      <h4 class="text-xs uppercase text-surface-500 mb-2">ROA</h4>
      <p class="text-2xl font-bold {getColorClass(data.roa, 10, true)}">
        {formatPercent(data.roa)}
      </p>
      <p class="text-xs text-surface-500">&gt;10% is good</p>
    </div>

    <div class="card p-4">
      <h4 class="text-xs uppercase text-surface-500 mb-2">Profit Margin</h4>
      <p class="text-2xl font-bold {getColorClass(data.profit_margin, 10, true)}">
        {formatPercent(data.profit_margin)}
      </p>
      <p class="text-xs text-surface-500">&gt;10% is healthy</p>
    </div>

    <!-- DCF Valuation -->
    {#if data.dcf_intrinsic_value}
      <div class="card p-4 col-span-2 md:col-span-1">
        <h4 class="text-xs uppercase text-surface-500 mb-2">DCF Value</h4>
        <p class="text-2xl font-bold">
          {data.dcf_intrinsic_value.toLocaleString()}
        </p>
        <p class="text-xs {data.dcf_margin_of_safety && data.dcf_margin_of_safety > 0 ? 'text-green-500' : 'text-red-500'}">
          {#if data.dcf_margin_of_safety}
            {data.dcf_margin_of_safety > 0 ? '+' : ''}{formatPercent(data.dcf_margin_of_safety)} margin
          {:else}
            vs {currentPrice.toLocaleString()} price
          {/if}
        </p>
      </div>
    {/if}
  </div>
{:else}
  <div class="card p-8 text-center text-surface-500">
    <p>No fundamental data available</p>
  </div>
{/if}
