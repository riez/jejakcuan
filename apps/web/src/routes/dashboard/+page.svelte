<script lang="ts">
	import { onMount } from 'svelte';
	import ScoreGauge from '$lib/components/ScoreGauge.svelte';
	import SignalCard from '$lib/components/SignalCard.svelte';
	import { BrokerFlowChart, ForeignFlow } from '$lib/components/BrokerFlow';
	
	interface Signal {
		id: string;
		symbol: string;
		type: string;
		message: string;
		priority: 'critical' | 'high' | 'medium' | 'low';
		timestamp: string;
	}
	
	interface Score {
		symbol: string;
		technical: number;
		fundamental: number;
		composite: number;
	}
	
	let signals: Signal[] = [
		{ id: '1', symbol: 'BBCA', type: 'technical', message: 'RSI oversold, potential bounce', priority: 'high', timestamp: new Date().toISOString() },
		{ id: '2', symbol: 'BBRI', type: 'broker', message: '4 institutional brokers accumulating', priority: 'high', timestamp: new Date().toISOString() },
		{ id: '3', symbol: 'TLKM', type: 'wyckoff', message: 'Wyckoff spring detected', priority: 'critical', timestamp: new Date().toISOString() },
	];
	
	let topScores: Score[] = [
		{ symbol: 'BBCA', technical: 78, fundamental: 85, composite: 81 },
		{ symbol: 'BMRI', technical: 72, fundamental: 80, composite: 76 },
		{ symbol: 'ASII', technical: 68, fundamental: 75, composite: 71 },
	];
	
	let loading = false;
	
	function getPriorityColor(priority: string): string {
		switch (priority) {
			case 'critical': return '#dc2626';
			case 'high': return '#ea580c';
			case 'medium': return '#ca8a04';
			default: return '#16a34a';
		}
	}
</script>

<svelte:head>
	<title>Dashboard | JejakCuan</title>
</svelte:head>

<div class="dashboard">
	<header>
		<h1>Signal Dashboard</h1>
		<p class="subtitle">Real-time market signals and analysis</p>
	</header>
	
	<div class="grid">
		<section class="signals-section">
			<h2>Active Signals</h2>
			<div class="signal-list">
				{#each signals as signal}
					<div class="signal-item" style="border-left-color: {getPriorityColor(signal.priority)}">
						<div class="signal-header">
							<span class="symbol">{signal.symbol}</span>
							<span class="badge {signal.priority}">{signal.priority}</span>
						</div>
						<p class="message">{signal.message}</p>
						<span class="type">{signal.type}</span>
					</div>
				{/each}
			</div>
		</section>
		
		<section class="scores-section">
			<h2>Top Scoring Stocks</h2>
			<div class="score-list">
				{#each topScores as score}
					<div class="score-item">
						<span class="symbol">{score.symbol}</span>
						<div class="scores">
							<div class="score-bar">
								<span class="label">Tech</span>
								<div class="bar" style="width: {score.technical}%; background: #6366f1;"></div>
								<span class="value">{score.technical}</span>
							</div>
							<div class="score-bar">
								<span class="label">Fund</span>
								<div class="bar" style="width: {score.fundamental}%; background: #10b981;"></div>
								<span class="value">{score.fundamental}</span>
							</div>
							<div class="score-bar">
								<span class="label">Total</span>
								<div class="bar" style="width: {score.composite}%; background: #f59e0b;"></div>
								<span class="value">{score.composite}</span>
							</div>
						</div>
					</div>
				{/each}
			</div>
		</section>
		
		<section class="broker-section">
			<h2>Market Flow</h2>
			<ForeignFlow 
				data={[
					{ date: '2024-01-10', netForeign: 100000000000, netInstitutional: 50000000000, netRetail: -30000000000 },
					{ date: '2024-01-11', netForeign: -50000000000, netInstitutional: 80000000000, netRetail: -20000000000 },
					{ date: '2024-01-12', netForeign: 200000000000, netInstitutional: 100000000000, netRetail: -50000000000 },
				]}
			/>
		</section>
		
		<section class="quick-actions">
			<h2>Quick Actions</h2>
			<div class="actions">
				<a href="/signals" class="action-btn">View All Signals</a>
				<a href="/watchlist" class="action-btn">My Watchlist</a>
				<a href="/alerts" class="action-btn">Alert Settings</a>
			</div>
		</section>
	</div>
</div>

<style>
	.dashboard {
		max-width: 1400px;
		margin: 0 auto;
		padding: 2rem;
	}
	
	header {
		margin-bottom: 2rem;
	}
	
	h1 {
		margin: 0;
		font-size: 2rem;
		color: #fff;
	}
	
	.subtitle {
		color: #9ca3af;
		margin: 0.5rem 0 0;
	}
	
	h2 {
		margin: 0 0 1rem;
		font-size: 1.2rem;
		color: #fff;
	}
	
	.grid {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 1.5rem;
	}
	
	section {
		background: #1a1a2e;
		border-radius: 12px;
		padding: 1.5rem;
	}
	
	.signal-item {
		background: #252543;
		border-radius: 8px;
		padding: 1rem;
		margin-bottom: 0.75rem;
		border-left: 4px solid;
	}
	
	.signal-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 0.5rem;
	}
	
	.symbol {
		font-weight: 700;
		color: #fff;
	}
	
	.badge {
		font-size: 0.7rem;
		padding: 0.2rem 0.5rem;
		border-radius: 4px;
		text-transform: uppercase;
	}
	
	.badge.critical { background: #dc2626; color: #fff; }
	.badge.high { background: #ea580c; color: #fff; }
	.badge.medium { background: #ca8a04; color: #000; }
	.badge.low { background: #16a34a; color: #fff; }
	
	.message {
		margin: 0 0 0.5rem;
		color: #d1d5db;
		font-size: 0.9rem;
	}
	
	.type {
		font-size: 0.75rem;
		color: #6b7280;
		text-transform: capitalize;
	}
	
	.score-item {
		background: #252543;
		border-radius: 8px;
		padding: 1rem;
		margin-bottom: 0.75rem;
	}
	
	.score-item .symbol {
		display: block;
		margin-bottom: 0.75rem;
	}
	
	.score-bar {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		margin-bottom: 0.5rem;
	}
	
	.score-bar .label {
		width: 40px;
		font-size: 0.75rem;
		color: #9ca3af;
	}
	
	.score-bar .bar {
		height: 8px;
		border-radius: 4px;
		transition: width 0.3s;
	}
	
	.score-bar .value {
		width: 30px;
		font-size: 0.8rem;
		color: #fff;
		text-align: right;
	}
	
	.actions {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}
	
	.action-btn {
		display: block;
		padding: 0.75rem 1rem;
		background: #252543;
		color: #fff;
		text-decoration: none;
		border-radius: 8px;
		text-align: center;
		transition: background 0.2s;
	}
	
	.action-btn:hover {
		background: #6366f1;
	}
	
	@media (max-width: 900px) {
		.grid {
			grid-template-columns: 1fr;
		}
	}
</style>
