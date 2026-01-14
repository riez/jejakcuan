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
	
	export let data: BrokerData[] = [];
	export let title = 'Broker Flow';
	export let showTop = 10;
	
	$: topBuyers = [...data]
		.filter(b => b.netValue > 0)
		.sort((a, b) => b.netValue - a.netValue)
		.slice(0, showTop);
	
	$: topSellers = [...data]
		.filter(b => b.netValue < 0)
		.sort((a, b) => a.netValue - b.netValue)
		.slice(0, showTop);
	
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
	
	$: maxBuy = Math.max(...topBuyers.map(b => b.netValue), 1);
	$: maxSell = Math.max(...topSellers.map(b => Math.abs(b.netValue)), 1);
</script>

<div class="broker-flow">
	<h3>{title}</h3>
	
	<div class="flow-grid">
		<div class="buyers">
			<h4>Top Buyers</h4>
			{#each topBuyers as broker}
				<div class="broker-row">
					<span class="code" title={broker.name}>{broker.code}</span>
					<div class="bar-container">
						<div 
							class="bar buy" 
							style="width: {getBarWidth(broker.netValue, maxBuy)}%"
						></div>
					</div>
					<span class="value">{formatValue(broker.netValue)}</span>
				</div>
			{/each}
			{#if topBuyers.length === 0}
				<p class="empty">No net buyers</p>
			{/if}
		</div>
		
		<div class="sellers">
			<h4>Top Sellers</h4>
			{#each topSellers as broker}
				<div class="broker-row">
					<span class="code" title={broker.name}>{broker.code}</span>
					<div class="bar-container">
						<div 
							class="bar sell" 
							style="width: {getBarWidth(broker.netValue, maxSell)}%"
						></div>
					</div>
					<span class="value">{formatValue(broker.netValue)}</span>
				</div>
			{/each}
			{#if topSellers.length === 0}
				<p class="empty">No net sellers</p>
			{/if}
		</div>
	</div>
</div>

<style>
	.broker-flow {
		background: var(--card-bg, #1a1a2e);
		border-radius: 8px;
		padding: 1rem;
	}
	
	h3 {
		margin: 0 0 1rem;
		font-size: 1.1rem;
		color: var(--text-primary, #fff);
	}
	
	h4 {
		margin: 0 0 0.5rem;
		font-size: 0.9rem;
		color: var(--text-secondary, #aaa);
	}
	
	.flow-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 1.5rem;
	}
	
	.broker-row {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		margin-bottom: 0.5rem;
		font-size: 0.85rem;
	}
	
	.code {
		width: 30px;
		font-weight: 600;
		color: var(--text-primary, #fff);
	}
	
	.bar-container {
		flex: 1;
		height: 8px;
		background: var(--bg-secondary, #2a2a4e);
		border-radius: 4px;
		overflow: hidden;
	}
	
	.bar {
		height: 100%;
		border-radius: 4px;
		transition: width 0.3s ease;
	}
	
	.bar.buy {
		background: linear-gradient(90deg, #10b981, #34d399);
	}
	
	.bar.sell {
		background: linear-gradient(90deg, #ef4444, #f87171);
	}
	
	.value {
		width: 60px;
		text-align: right;
		font-size: 0.8rem;
		color: var(--text-secondary, #aaa);
	}
	
	.empty {
		color: var(--text-muted, #666);
		font-size: 0.85rem;
		font-style: italic;
	}
	
	@media (max-width: 600px) {
		.flow-grid {
			grid-template-columns: 1fr;
		}
	}
</style>
