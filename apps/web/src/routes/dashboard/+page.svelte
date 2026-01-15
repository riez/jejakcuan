<script lang="ts">
	import { onMount } from 'svelte';
	import ScoreGauge from '$lib/components/ScoreGauge.svelte';
	import SignalCard from '$lib/components/SignalCard.svelte';
	import { BrokerFlowChart, ForeignFlow } from '$lib/components/BrokerFlow';
	
	interface Signal {
		id: string;
		symbol: string;
		stockName: string;
		type: 'buy' | 'sell' | 'hold';
		strength: 'strong' | 'moderate' | 'weak';
		score: number;
		reason: string;
		timestamp: Date;
		priceAtSignal: number;
		targetPrice?: number;
		stopLoss?: number;
		indicators: string[];
	}
	
	interface Score {
		symbol: string;
		technical: number;
		fundamental: number;
		composite: number;
	}
	
	let signals: Signal[] = [
		{ id: '1', symbol: 'BBCA', stockName: 'Bank Central Asia', type: 'buy', strength: 'strong', score: 82, reason: 'RSI oversold, potential bounce', timestamp: new Date(), priceAtSignal: 9250, indicators: ['RSI Oversold'] },
		{ id: '2', symbol: 'BBRI', stockName: 'Bank Rakyat Indonesia', type: 'buy', strength: 'moderate', score: 71, reason: '4 institutional brokers accumulating', timestamp: new Date(), priceAtSignal: 4850, indicators: ['Broker Flow'] },
		{ id: '3', symbol: 'TLKM', stockName: 'Telkom Indonesia', type: 'sell', strength: 'strong', score: 75, reason: 'Wyckoff spring detected', timestamp: new Date(), priceAtSignal: 4120, indicators: ['Wyckoff'] },
	];
	
	let topScores: Score[] = [
		{ symbol: 'BBCA', technical: 78, fundamental: 85, composite: 81 },
		{ symbol: 'BMRI', technical: 72, fundamental: 80, composite: 76 },
		{ symbol: 'ASII', technical: 68, fundamental: 75, composite: 71 },
	];
	
	let loading = false;
	
	function getPriorityClass(priority: string): string {
		switch (priority) {
			case 'critical': return 'variant-filled-error';
			case 'high': return 'variant-filled-warning';
			case 'medium': return 'variant-soft-warning';
			default: return 'variant-filled-success';
		}
	}
	
	function getSignalTypeClass(type: string): string {
		switch (type) {
			case 'buy': return 'variant-filled-success';
			case 'sell': return 'variant-filled-error';
			default: return 'variant-filled-warning';
		}
	}
</script>

<svelte:head>
	<title>Dashboard | JejakCuan</title>
</svelte:head>

<div class="space-y-6">
	<header class="flex items-center justify-between">
		<div>
			<h1 class="h1">Signal Dashboard</h1>
			<p class="text-surface-600-300-token">Real-time market signals and analysis</p>
		</div>
	</header>
	
	<div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
		<!-- Active Signals -->
		<div class="card p-6">
			<h2 class="h3 mb-4">Active Signals</h2>
			<div class="space-y-3">
				{#each signals as signal}
					<div class="card variant-soft p-4 border-l-4 {signal.type === 'buy' ? 'border-l-green-500' : signal.type === 'sell' ? 'border-l-red-500' : 'border-l-yellow-500'}">
						<div class="flex items-center justify-between mb-2">
							<a href="/stock/{signal.symbol}" class="anchor font-bold text-lg">{signal.symbol}</a>
							<span class="badge {getSignalTypeClass(signal.type)}">{signal.type.toUpperCase()}</span>
						</div>
						<p class="text-surface-600-300-token text-sm mb-2">{signal.reason}</p>
						<div class="flex items-center gap-2">
							{#each signal.indicators as indicator}
								<span class="badge variant-soft-primary text-xs">{indicator}</span>
							{/each}
						</div>
					</div>
				{/each}
			</div>
		</div>
		
		<!-- Top Scoring Stocks -->
		<div class="card p-6">
			<h2 class="h3 mb-4">Top Scoring Stocks</h2>
			<div class="space-y-4">
				{#each topScores as score}
					<div class="card variant-soft p-4">
						<div class="flex items-center justify-between mb-3">
							<a href="/stock/{score.symbol}" class="anchor font-bold">{score.symbol}</a>
							<span class="badge {score.composite >= 70 ? 'variant-filled-success' : score.composite >= 50 ? 'variant-filled-warning' : 'variant-filled-error'}">
								{score.composite}
							</span>
						</div>
						<div class="space-y-2">
							<div class="flex items-center gap-2">
								<span class="text-surface-500 text-sm w-16">Tech</span>
								<div class="flex-1 h-2 bg-surface-300 dark:bg-surface-700 rounded-full overflow-hidden">
									<div class="h-full bg-primary-500 rounded-full" style="width: {score.technical}%"></div>
								</div>
								<span class="text-sm w-8">{score.technical}</span>
							</div>
							<div class="flex items-center gap-2">
								<span class="text-surface-500 text-sm w-16">Fund</span>
								<div class="flex-1 h-2 bg-surface-300 dark:bg-surface-700 rounded-full overflow-hidden">
									<div class="h-full bg-green-500 rounded-full" style="width: {score.fundamental}%"></div>
								</div>
								<span class="text-sm w-8">{score.fundamental}</span>
							</div>
							<div class="flex items-center gap-2">
								<span class="text-surface-500 text-sm w-16">Total</span>
								<div class="flex-1 h-2 bg-surface-300 dark:bg-surface-700 rounded-full overflow-hidden">
									<div class="h-full bg-warning-500 rounded-full" style="width: {score.composite}%"></div>
								</div>
								<span class="text-sm w-8">{score.composite}</span>
							</div>
						</div>
					</div>
				{/each}
			</div>
		</div>
		
		<!-- Market Flow -->
		<div class="card p-6">
			<h2 class="h3 mb-4">Market Flow</h2>
			<ForeignFlow 
				data={[
					{ date: '2024-01-10', netForeign: 100000000000, netInstitutional: 50000000000, netRetail: -30000000000 },
					{ date: '2024-01-11', netForeign: -50000000000, netInstitutional: 80000000000, netRetail: -20000000000 },
					{ date: '2024-01-12', netForeign: 200000000000, netInstitutional: 100000000000, netRetail: -50000000000 },
				]}
			/>
		</div>
		
		<!-- Quick Actions -->
		<div class="card p-6">
			<h2 class="h3 mb-4">Quick Actions</h2>
			<div class="flex flex-col gap-3">
				<a href="/signals" class="btn variant-soft-primary w-full">
					<span>📊</span>
					<span>View All Signals</span>
				</a>
				<a href="/watchlist" class="btn variant-soft-secondary w-full">
					<span>⭐</span>
					<span>My Watchlist</span>
				</a>
				<a href="/alerts" class="btn variant-soft-tertiary w-full">
					<span>🔔</span>
					<span>Alert Settings</span>
				</a>
				<a href="/market" class="btn variant-soft-surface w-full">
					<span>📈</span>
					<span>Market Overview</span>
				</a>
			</div>
		</div>
	</div>
</div>
