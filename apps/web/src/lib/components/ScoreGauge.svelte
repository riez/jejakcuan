<script lang="ts">
  let { score = 50, label = 'Score', size = 'md' } = $props();

  const sizeClasses = {
    sm: 'w-16 h-16 text-lg',
    md: 'w-24 h-24 text-2xl',
    lg: 'w-32 h-32 text-3xl'
  };

  const getColor = (s: number) => {
    if (s >= 70) return 'stroke-green-500';
    if (s >= 50) return 'stroke-yellow-500';
    return 'stroke-red-500';
  };

  const getTextColor = (s: number) => {
    if (s >= 70) return 'text-green-500';
    if (s >= 50) return 'text-yellow-500';
    return 'text-red-500';
  };

  // SVG arc calculations
  const radius = 40;
  const circumference = 2 * Math.PI * radius;
  const dashOffset = $derived(circumference - (score / 100) * circumference);
</script>

<div class="flex flex-col items-center gap-1">
  <div class="relative {sizeClasses[size as keyof typeof sizeClasses] || sizeClasses.md}">
    <svg class="w-full h-full transform -rotate-90" viewBox="0 0 100 100">
      <!-- Background circle -->
      <circle
        cx="50"
        cy="50"
        r={radius}
        fill="none"
        stroke="currentColor"
        stroke-width="8"
        class="text-surface-300 dark:text-surface-700"
      />
      <!-- Progress circle -->
      <circle
        cx="50"
        cy="50"
        r={radius}
        fill="none"
        stroke-width="8"
        stroke-linecap="round"
        class={getColor(score)}
        style="stroke-dasharray: {circumference}; stroke-dashoffset: {dashOffset};"
      />
    </svg>
    <div class="absolute inset-0 flex items-center justify-center {getTextColor(score)} font-bold">
      {score.toFixed(0)}
    </div>
  </div>
  <span class="text-sm text-surface-600-300-token">{label}</span>
</div>
