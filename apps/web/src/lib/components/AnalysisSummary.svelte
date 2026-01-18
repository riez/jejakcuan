<script lang="ts">
  import type { OverallConclusion, ValuationEstimate } from './StockAnalysis.types';
  import type { StockScore } from '$lib/api';

  let {
    score,
    valuation,
    conclusion,
    currentPrice,
  }: {
    score: StockScore | null;
    valuation: ValuationEstimate | null;
    conclusion: OverallConclusion | null;
    currentPrice: number;
  } = $props();

  let signal = $derived(() => {
    if (!score) return { label: 'ANALYZING', color: 'bg-surface-400', conviction: 0 };
    const c = Number(score.composite_score);
    if (c >= 80) return { label: 'STRONG BUY', color: 'bg-emerald-500', conviction: c };
    if (c >= 65) return { label: 'BUY', color: 'bg-emerald-400', conviction: c };
    if (c >= 50) return { label: 'HOLD', color: 'bg-amber-400', conviction: c };
    if (c >= 35) return { label: 'SELL', color: 'bg-rose-400', conviction: c };
    return { label: 'STRONG SELL', color: 'bg-rose-500', conviction: c };
  });

  let targetPrice = $derived(() => valuation?.fairPriceRange?.high ?? null);

  let upside = $derived(() => {
    const target = targetPrice();
    if (!target || !currentPrice || currentPrice === 0) return null;
    return ((target - currentPrice) / currentPrice) * 100;
  });

  let riskLevel = $derived(() => {
    const tech = score?.technical_score ? Number(score.technical_score) : 50;
    if (tech < 40) return 'HIGH';
    if (tech < 60) return 'MEDIUM';
    return 'LOW';
  });

  let thesis = $derived(() => {
    if (!conclusion) return 'Loading analysis...';
    const strengths = conclusion.strengths?.slice(0, 2) ?? [];
    return strengths.length > 0 ? strengths.join('. ') : 'Analysis in progress...';
  });
</script>

<div
  class="card p-6 bg-gradient-to-br from-surface-100 to-surface-200 dark:from-surface-800 dark:to-surface-900"
>
  <div class="flex flex-col md:flex-row items-start gap-6">
    <div class="flex flex-col items-center">
      <div
        class="w-24 h-24 rounded-full flex items-center justify-center {signal().color} text-white shadow-lg"
      >
        <span class="text-lg font-bold text-center leading-tight px-2">
          {signal().label}
        </span>
      </div>
      <span class="mt-2 text-sm text-surface-600-300-token">
        Conviction: {signal().conviction.toFixed(0)}%
      </span>
    </div>

    <div class="flex-1 grid grid-cols-2 gap-4">
      <div>
        <span class="text-sm text-surface-500-400-token">Target Price</span>
        <div class="text-2xl font-bold text-success-600-300-token">
          {targetPrice()?.toLocaleString('id-ID') ?? '-'}
        </div>
        {#if upside() !== null}
          <span
            class="text-sm {upside()! > 0 ? 'text-success-500' : 'text-error-500'}"
          >
            {upside()! > 0 ? '+' : ''}{upside()!.toFixed(1)}% potential
          </span>
        {/if}
      </div>
      <div>
        <span class="text-sm text-surface-500-400-token">Risk Level</span>
        <div
          class="text-xl font-semibold {riskLevel() === 'HIGH'
            ? 'text-error-500'
            : riskLevel() === 'MEDIUM'
              ? 'text-warning-500'
              : 'text-success-500'}"
        >
          {riskLevel()}
        </div>
        <span class="text-sm text-surface-400">
          Technical: {score?.technical_score ? Number(score.technical_score).toFixed(0) : '-'}
        </span>
      </div>
    </div>
  </div>

  <div class="mt-4 p-3 bg-surface-200/50 dark:bg-surface-700/50 rounded-lg">
    <p class="text-sm text-surface-700-200-token">{thesis()}</p>
  </div>
</div>
