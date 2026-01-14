<script lang="ts">
	interface Conglomerate {
		id: string;
		name: string;
		stocks: {
			symbol: string;
			name: string;
			sector: string;
			weight: number;
		}[];
		totalMarketCap: number;
		avgScore: number;
	}
	
	let conglomerates: Conglomerate[] = [
		{
			id: 'djarum',
			name: 'Djarum Group',
			stocks: [
				{ symbol: 'BBCA', name: 'Bank Central Asia', sector: 'Banking', weight: 45 },
				{ symbol: 'GGRM', name: 'Gudang Garam', sector: 'Consumer', weight: 35 },
				{ symbol: 'HMSP', name: 'HM Sampoerna', sector: 'Consumer', weight: 20 },
			],
			totalMarketCap: 1500000000000000,
			avgScore: 78,
		},
		{
			id: 'salim',
			name: 'Salim Group',
			stocks: [
				{ symbol: 'INDF', name: 'Indofood Sukses Makmur', sector: 'Consumer', weight: 40 },
				{ symbol: 'ICBP', name: 'Indofood CBP', sector: 'Consumer', weight: 30 },
				{ symbol: 'INTP', name: 'Indocement', sector: 'Basic Materials', weight: 30 },
			],
			totalMarketCap: 800000000000000,
			avgScore: 72,
		},
		{
			id: 'lippo',
			name: 'Lippo Group',
			stocks: [
				{ symbol: 'LPKR', name: 'Lippo Karawaci', sector: 'Property', weight: 50 },
				{ symbol: 'MLPL', name: 'Multipolar', sector: 'Trade', weight: 25 },
				{ symbol: 'MPPA', name: 'Matahari', sector: 'Retail', weight: 25 },
			],
			totalMarketCap: 200000000000000,
			avgScore: 45,
		},
		{
			id: 'sinarmas',
			name: 'Sinarmas Group',
			stocks: [
				{ symbol: 'BSDE', name: 'Bumi Serpong Damai', sector: 'Property', weight: 35 },
				{ symbol: 'DSNG', name: 'Dharma Satya Nusantara', sector: 'Plantation', weight: 35 },
				{ symbol: 'SMMA', name: 'Sinarmas Multiartha', sector: 'Finance', weight: 30 },
			],
			totalMarketCap: 350000000000000,
			avgScore: 58,
		},
	];
	
	let selectedConglomerate: Conglomerate | null = null;
	
	function formatMarketCap(value: number): string {
		if (value >= 1e15) return `Rp${(value / 1e15).toFixed(1)}K T`;
		if (value >= 1e12) return `Rp${(value / 1e12).toFixed(0)} T`;
		return `Rp${(value / 1e9).toFixed(0)} B`;
	}
	
	function getScoreColor(score: number): string {
		if (score >= 70) return '#10b981';
		if (score >= 50) return '#f59e0b';
		return '#ef4444';
	}
</script>

<svelte:head>
	<title>Conglomerate Tracking | JejakCuan</title>
</svelte:head>

<div class="page">
	<header>
		<h1>Conglomerate Tracking</h1>
		<p class="subtitle">Monitor Indonesian business groups and their holdings</p>
	</header>
	
	<div class="grid">
		<div class="conglomerate-list">
			{#each conglomerates as cong}
				<button 
					class="cong-card" 
					class:selected={selectedConglomerate?.id === cong.id}
					on:click={() => selectedConglomerate = cong}
				>
					<div class="cong-header">
						<h3>{cong.name}</h3>
						<span class="score" style="color: {getScoreColor(cong.avgScore)}">
							{cong.avgScore}
						</span>
					</div>
					<div class="cong-meta">
						<span>{cong.stocks.length} stocks</span>
						<span>{formatMarketCap(cong.totalMarketCap)}</span>
					</div>
				</button>
			{/each}
		</div>
		
		<div class="detail-panel">
			{#if selectedConglomerate}
				<h2>{selectedConglomerate.name}</h2>
				
				<div class="holdings">
					<h4>Holdings</h4>
					<div class="holdings-list">
						{#each selectedConglomerate.stocks as stock}
							<a href="/stock/{stock.symbol}" class="holding-item">
								<div class="holding-main">
									<span class="symbol">{stock.symbol}</span>
									<span class="name">{stock.name}</span>
								</div>
								<div class="holding-meta">
									<span class="sector">{stock.sector}</span>
									<span class="weight">{stock.weight}%</span>
								</div>
								<div class="weight-bar">
									<div class="fill" style="width: {stock.weight}%"></div>
								</div>
							</a>
						{/each}
					</div>
				</div>
				
				<div class="stats">
					<div class="stat">
						<span class="label">Total Market Cap</span>
						<span class="value">{formatMarketCap(selectedConglomerate.totalMarketCap)}</span>
					</div>
					<div class="stat">
						<span class="label">Avg Score</span>
						<span class="value" style="color: {getScoreColor(selectedConglomerate.avgScore)}">
							{selectedConglomerate.avgScore}
						</span>
					</div>
				</div>
			{:else}
				<div class="placeholder">
					<p>Select a conglomerate to view details</p>
				</div>
			{/if}
		</div>
	</div>
</div>

<style>
	.page {
		max-width: 1200px;
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
	
	.grid {
		display: grid;
		grid-template-columns: 350px 1fr;
		gap: 1.5rem;
	}
	
	.conglomerate-list {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}
	
	.cong-card {
		background: #1a1a2e;
		border: 2px solid transparent;
		border-radius: 12px;
		padding: 1rem;
		cursor: pointer;
		transition: all 0.2s;
		text-align: left;
	}
	
	.cong-card:hover {
		border-color: #4338ca;
	}
	
	.cong-card.selected {
		border-color: #6366f1;
		background: #252543;
	}
	
	.cong-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}
	
	.cong-header h3 {
		margin: 0;
		font-size: 1rem;
		color: #fff;
	}
	
	.score {
		font-size: 1.5rem;
		font-weight: 700;
	}
	
	.cong-meta {
		display: flex;
		gap: 1rem;
		margin-top: 0.5rem;
		font-size: 0.8rem;
		color: #9ca3af;
	}
	
	.detail-panel {
		background: #1a1a2e;
		border-radius: 12px;
		padding: 1.5rem;
		min-height: 400px;
	}
	
	.detail-panel h2 {
		margin: 0 0 1.5rem;
		color: #fff;
	}
	
	.detail-panel h4 {
		margin: 0 0 1rem;
		color: #9ca3af;
		font-size: 0.9rem;
	}
	
	.holding-item {
		display: block;
		background: #252543;
		border-radius: 8px;
		padding: 1rem;
		margin-bottom: 0.75rem;
		text-decoration: none;
		transition: background 0.2s;
	}
	
	.holding-item:hover {
		background: #2d2d5a;
	}
	
	.holding-main {
		display: flex;
		gap: 1rem;
		margin-bottom: 0.5rem;
	}
	
	.holding-main .symbol {
		font-weight: 700;
		color: #fff;
	}
	
	.holding-main .name {
		color: #d1d5db;
	}
	
	.holding-meta {
		display: flex;
		justify-content: space-between;
		font-size: 0.8rem;
		margin-bottom: 0.5rem;
	}
	
	.sector {
		color: #6b7280;
	}
	
	.weight {
		color: #6366f1;
		font-weight: 600;
	}
	
	.weight-bar {
		height: 4px;
		background: #374151;
		border-radius: 2px;
	}
	
	.weight-bar .fill {
		height: 100%;
		background: #6366f1;
		border-radius: 2px;
	}
	
	.stats {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 1rem;
		margin-top: 1.5rem;
	}
	
	.stat {
		background: #252543;
		padding: 1rem;
		border-radius: 8px;
		text-align: center;
	}
	
	.stat .label {
		display: block;
		font-size: 0.75rem;
		color: #9ca3af;
		margin-bottom: 0.25rem;
	}
	
	.stat .value {
		font-size: 1.25rem;
		font-weight: 700;
		color: #fff;
	}
	
	.placeholder {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
		color: #6b7280;
	}
	
	@media (max-width: 800px) {
		.grid {
			grid-template-columns: 1fr;
		}
	}
</style>
