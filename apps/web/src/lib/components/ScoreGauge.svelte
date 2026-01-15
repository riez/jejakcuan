<script lang="ts">
  import { ProgressRadial } from '@skeletonlabs/skeleton';

  let { score = 50, label = 'Score', size = 'md' } = $props();

  const sizeClasses = {
    sm: 'w-12',
    md: 'w-20',
    lg: 'w-28'
  };

  const getMeterClass = (s: number) => {
    if (s >= 70) return 'stroke-success-500';
    if (s >= 50) return 'stroke-warning-500';
    return 'stroke-error-500';
  };

  const getTrackClass = (s: number) => {
    if (s >= 70) return 'stroke-success-500/20';
    if (s >= 50) return 'stroke-warning-500/20';
    return 'stroke-error-500/20';
  };

  const getTextClass = (s: number) => {
    if (s >= 70) return 'text-success-500';
    if (s >= 50) return 'text-warning-500';
    return 'text-error-500';
  };
</script>

<div class="flex flex-col items-center gap-1">
  <ProgressRadial
    value={score}
    width={sizeClasses[size as keyof typeof sizeClasses] || sizeClasses.md}
    stroke={120}
    meter={getMeterClass(score)}
    track={getTrackClass(score)}
    strokeLinecap="round"
    font={150}
  >
    <span class="font-bold font-tabular {getTextClass(score)}">{score.toFixed(0)}</span>
  </ProgressRadial>
  {#if label}
    <span class="text-sm text-surface-400">{label}</span>
  {/if}
</div>
