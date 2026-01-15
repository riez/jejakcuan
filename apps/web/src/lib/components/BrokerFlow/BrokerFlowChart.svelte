<script lang="ts">
	import { onMount } from 'svelte';
	
	interface BrokerData {
		code: string;
		name: string;
		buyValue: number;
		sellValue: number;
		netValue: number;
		buyLot: number;
		sellLot: number;
	}
	
	let { data = [] as BrokerData[], title = 'Broker Flow', showTop = 10 } = $props();
	
	let topBuyers = $derived([...data]
		.filter(b => b.netValue > 0)
		.sort((a, b) => b.netValue - a.netValue)
		.slice(0, showTop));
	
	let topSellers = $derived([...data]
		.filter(b => b.netValue < 0)
		.sort((a, b) => a.netValue - b.netValue)
		.slice(0, showTop));
	
	function formatValue(value: number): string {
		const abs = Math.abs(value);
		if (abs >= 1e12) return `${(value / 1e12).toFixed(1)}T`;
		if (abs >= 1e9) return `${(value / 1e9).toFixed(1)}B`;
		if (abs >= 1e6) return `${(value / 1e6).toFixed(1)}M`;
		return value.toLocaleString();
	}
	
	function getBarWidth(value: number, max: number): number {
		return (Math.abs(value) / max) * 100;
	}
	
	let maxBuy = $derived(Math.max(...topBuyers.map(b => b.netValue), 1));
	let maxSell = $derived(Math.max(...topSellers.map(b => Math.abs(b.netValue)), 1));
</script>

<div class="card p-4">
	<h3 class="h4 mb-4">{title}</h3>
	
	<div class="grid grid-cols-1 md:grid-cols-2 gap-6">
		<!-- Top Buyers -->
		<div>
			<h4 class="text-sm text-surface-500 mb-3">Top Buyers</h4>
			{#if topBuyers.length > 0}
				<div class="space-y-2">
					{#each topBuyers as broker}
						<div class="flex items-center gap-2 text-sm">
							<span class="w-8 font-bold text-surface-900 dark:text-white" title={broker.name}>{broker.code}</span>
							<div class="flex-1 h-2 bg-surface-200 dark:bg-surface-700 rounded-full overflow-hidden">
								<div 
									class="h-full bg-gradient-to-r from-green-500 to-green-400 rounded-full"
									style="width: {getBarWidth(broker.netValue, maxBuy)}%"
								></div>
							</div>
							<span class="w-16 text-right text-surface-600-300-token">{formatValue(broker.netValue)}</span>
						</div>
					{/each}
				</div>
			{:else}
				<p class="text-surface-500 text-sm italic">No net buyers</p>
			{/if}
		</div>
		
		<!-- Top Sellers -->
		<div>
			<h4 class="text-sm text-surface-500 mb-3">Top Sellers</h4>
			{#if topSellers.length > 0}
				<div class="space-y-2">
					{#each topSellers as broker}
						<div class="flex items-center gap-2 text-sm">
							<span class="w-8 font-bold text-surface-900 dark:text-white" title={broker.name}>{broker.code}</span>
							<div class="flex-1 h-2 bg-surface-200 dark:bg-surface-700 rounded-full overflow-hidden">
								<div 
									class="h-full bg-gradient-to-r from-red-500 to-red-400 rounded-full"
									style="width: {getBarWidth(broker.netValue, maxSell)}%"
								></div>
							</div>
							<span class="w-16 text-right text-surface-600-300-token">{formatValue(broker.netValue)}</span>
						</div>
					{/each}
				</div>
			{:else}
				<p class="text-surface-500 text-sm italic">No net sellers</p>
			{/if}
		</div>
	</div>
</div>
