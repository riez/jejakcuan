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
	
	let selectedConglomerate = $state<Conglomerate | null>(null);
	
	function formatMarketCap(value: number): string {
		if (value >= 1e15) return `Rp${(value / 1e15).toFixed(1)}K T`;
		if (value >= 1e12) return `Rp${(value / 1e12).toFixed(0)} T`;
		return `Rp${(value / 1e9).toFixed(0)} B`;
	}
	
	function getScoreClass(score: number): string {
		if (score >= 70) return 'text-green-500';
		if (score >= 50) return 'text-yellow-500';
		return 'text-red-500';
	}
	
	function getScoreBadgeClass(score: number): string {
		if (score >= 70) return 'variant-filled-success';
		if (score >= 50) return 'variant-filled-warning';
		return 'variant-filled-error';
	}
</script>

<svelte:head>
	<title>Conglomerate Tracking | JejakCuan</title>
</svelte:head>

<div class="space-y-6">
	<header>
		<h1 class="h1">Conglomerate Tracking</h1>
		<p class="text-surface-600-300-token">Monitor Indonesian business groups and their holdings</p>
	</header>
	
	<div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
		<!-- Conglomerate List -->
		<div class="lg:col-span-1 space-y-3">
			{#each conglomerates as cong}
				<button 
					class="card p-4 w-full text-left hover:ring-2 ring-primary-500 transition-all {selectedConglomerate?.id === cong.id ? 'ring-2 ring-primary-500 variant-soft-primary' : ''}"
					onclick={() => selectedConglomerate = cong}
				>
					<div class="flex items-center justify-between mb-2">
						<h3 class="h4">{cong.name}</h3>
						<span class="text-2xl font-bold {getScoreClass(cong.avgScore)}">
							{cong.avgScore}
						</span>
					</div>
					<div class="flex gap-4 text-sm text-surface-500">
						<span>{cong.stocks.length} stocks</span>
						<span>{formatMarketCap(cong.totalMarketCap)}</span>
					</div>
				</button>
			{/each}
		</div>
		
		<!-- Detail Panel -->
		<div class="lg:col-span-2">
			<div class="card p-6 min-h-[500px]">
				{#if selectedConglomerate}
					<h2 class="h2 mb-6">{selectedConglomerate.name}</h2>
					
					<!-- Holdings -->
					<div class="mb-6">
						<h4 class="text-sm uppercase text-surface-500 mb-3">Holdings</h4>
						<div class="space-y-3">
							{#each selectedConglomerate.stocks as stock}
								<a href="/stock/{stock.symbol}" class="card variant-soft p-4 block hover:ring-2 ring-primary-500 transition-all">
									<div class="flex items-center justify-between mb-2">
										<div>
											<span class="font-bold text-lg">{stock.symbol}</span>
											<span class="text-surface-600-300-token ml-2">{stock.name}</span>
										</div>
										<span class="badge variant-soft-primary">{stock.weight}%</span>
									</div>
									<div class="flex items-center justify-between text-sm">
										<span class="text-surface-500">{stock.sector}</span>
										<div class="w-32 h-2 bg-surface-300 dark:bg-surface-700 rounded-full overflow-hidden">
											<div class="h-full bg-primary-500 rounded-full" style="width: {stock.weight}%"></div>
										</div>
									</div>
								</a>
							{/each}
						</div>
					</div>
					
					<!-- Stats -->
					<div class="grid grid-cols-2 gap-4">
						<div class="card variant-soft p-4 text-center">
							<p class="text-sm text-surface-500 mb-1">Total Market Cap</p>
							<p class="text-xl font-bold">{formatMarketCap(selectedConglomerate.totalMarketCap)}</p>
						</div>
						<div class="card variant-soft p-4 text-center">
							<p class="text-sm text-surface-500 mb-1">Avg Score</p>
							<p class="text-xl font-bold {getScoreClass(selectedConglomerate.avgScore)}">
								{selectedConglomerate.avgScore}
							</p>
						</div>
					</div>
				{:else}
					<div class="flex items-center justify-center h-full text-surface-500">
						<p>Select a conglomerate to view details</p>
					</div>
				{/if}
			</div>
		</div>
	</div>
</div>
