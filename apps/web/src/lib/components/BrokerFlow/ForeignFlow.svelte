<script lang="ts">
	interface FlowData {
		date: string;
		netForeign: number;
		netInstitutional: number;
		netRetail: number;
	}
	
	let { data = [] as FlowData[], symbol = '' } = $props();
	
	let totalForeign = $derived(data.reduce((sum, d) => sum + d.netForeign, 0));
	let totalInstitutional = $derived(data.reduce((sum, d) => sum + d.netInstitutional, 0));
	let cumulativeForeign = $derived(data.reduce((acc, d) => {
		const last = acc.length > 0 ? acc[acc.length - 1] : 0;
		acc.push(last + d.netForeign);
		return acc;
	}, [] as number[]));
	
	function formatValue(value: number): string {
		const abs = Math.abs(value);
		if (abs >= 1e12) return `${(value / 1e12).toFixed(1)}T`;
		if (abs >= 1e9) return `${(value / 1e9).toFixed(1)}B`;
		if (abs >= 1e6) return `${(value / 1e6).toFixed(1)}M`;
		return value.toLocaleString();
	}
	
	function getValueClass(value: number): string {
		if (value > 0) return 'text-green-500';
		if (value < 0) return 'text-red-500';
		return 'text-surface-500';
	}
</script>

<div class="card variant-soft p-4">
	<div class="flex items-center justify-between mb-4">
		<h3 class="h4">Foreign & Institutional Flow</h3>
		{#if symbol}
			<span class="badge variant-filled-primary">{symbol}</span>
		{/if}
	</div>
	
	<div class="grid grid-cols-2 gap-4 mb-4">
		<div class="card p-4 text-center">
			<p class="text-xs text-surface-500 mb-1">Net Foreign</p>
			<p class="text-xl font-bold {getValueClass(totalForeign)}">{formatValue(totalForeign)}</p>
		</div>
		<div class="card p-4 text-center">
			<p class="text-xs text-surface-500 mb-1">Net Institutional</p>
			<p class="text-xl font-bold {getValueClass(totalInstitutional)}">{formatValue(totalInstitutional)}</p>
		</div>
	</div>
	
	<!-- Simple SVG Chart -->
	<div class="h-24 mb-2">
		<svg viewBox="0 0 400 100" class="w-full h-full">
			{#if data.length > 0}
				<line x1="0" y1="50" x2="400" y2="50" stroke="currentColor" stroke-dasharray="4" class="text-surface-300 dark:text-surface-700"/>
				<polyline
					fill="none"
					stroke={totalForeign >= 0 ? '#10b981' : '#ef4444'}
					stroke-width="2"
					points={cumulativeForeign.map((v, i) => {
						const x = data.length > 1 ? (i / (data.length - 1)) * 400 : 200;
						const max = Math.max(...cumulativeForeign.map(Math.abs)) || 1;
						const y = 50 - (v / max) * 40;
						return `${x},${y}`;
					}).join(' ')}
				/>
			{:else}
				<text x="200" y="50" text-anchor="middle" class="fill-surface-500 text-sm">No data</text>
			{/if}
		</svg>
	</div>
	
	<div class="flex justify-center gap-4 text-xs">
		<span class="text-green-500">● Inflow</span>
		<span class="text-red-500">● Outflow</span>
	</div>
</div>
