<script lang="ts">
  import { ProgressBar } from '@skeletonlabs/skeleton';

  interface ScoreBreakdown {
    name: string;
    score: number;
    weight: number;
    signals: string[];
  }

  let {
    components = [] as ScoreBreakdown[],
    totalScore = 0
  } = $props();

  const getBarColor = (score: number) => {
    if (score >= 70) return 'bg-green-500';
    if (score >= 50) return 'bg-yellow-500';
    return 'bg-red-500';
  };
</script>

<div class="space-y-4">
  {#each components as component}
    <div>
      <div class="flex justify-between items-center mb-1">
        <span class="text-sm font-medium">{component.name}</span>
        <span class="text-sm">
          {component.score.toFixed(0)}
          <span class="text-surface-500">({(component.weight * 100).toFixed(0)}%)</span>
        </span>
      </div>
      <ProgressBar
        value={component.score}
        max={100}
        height="h-2"
        meter={getBarColor(component.score)}
        track="bg-surface-200-700-token"
      />
      {#if component.signals.length > 0}
        <div class="mt-1 text-xs text-surface-500">
          {component.signals[0]}
        </div>
      {/if}
    </div>
  {/each}

  <div class="pt-2 border-t border-surface-200 dark:border-surface-700">
    <div class="flex justify-between items-center">
      <span class="font-semibold">Total Score</span>
      <span class="font-bold text-lg {totalScore >= 70 ? 'text-green-500' : totalScore >= 50 ? 'text-yellow-500' : 'text-red-500'}">
        {totalScore.toFixed(0)}
      </span>
    </div>
  </div>
</div>
