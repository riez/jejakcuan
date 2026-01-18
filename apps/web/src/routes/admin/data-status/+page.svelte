<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { ProgressBar, ProgressRadial } from '@skeletonlabs/skeleton';
	import { api, type DataSourcesResponse, type GranularDataSource, type TriggerResponse, type CategoryTriggerResponse, type DataStatusResponse, type DataSummary, type Job } from '$lib/api';

	let data: DataSourcesResponse | null = null;
	let legacySummary: DataSummary | null = null;
	let loading = true;
	let error: string | null = null;
	let refreshInterval: ReturnType<typeof setInterval> | null = null;
	
	let sourceLoadingStates: Record<string, boolean> = {};
	let sourceErrors: Record<string, string | null> = {};
	let triggerMessages: Record<string, TriggerResponse | null> = {};
	let categoryLoading: Record<string, boolean> = {};
	let categoryMessages: Record<string, CategoryTriggerResponse | null> = {};
	let expandedCategories: Record<string, boolean> = {
		broker: true,
		prices: true,
		fundamentals: true,
		scores: true
	};
	let bulkRefreshing = false;
	
	let activeJobs: Record<string, Job> = {};
	let jobPollingIntervals: Record<string, ReturnType<typeof setInterval>> = {};

	async function fetchStatus() {
		try {
			error = null;
			const [sourcesData, legacyData] = await Promise.all([
				api.getDataSources(),
				api.getDataStatus()
			]);
			data = sourcesData;
			legacySummary = legacyData.summary;
			
			if (data) {
				data.sources.forEach(s => {
					if (!(s.id in sourceLoadingStates)) {
						sourceLoadingStates[s.id] = false;
						sourceErrors[s.id] = null;
					}
				});
			}
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to fetch data status';
		} finally {
			loading = false;
		}
	}

	async function triggerSource(sourceId: string) {
		sourceLoadingStates[sourceId] = true;
		sourceErrors[sourceId] = null;
		triggerMessages[sourceId] = null;
		
		try {
			const response = await api.triggerDataSource(sourceId);
			triggerMessages[sourceId] = response;
			
			if (response.job_id && response.job) {
				activeJobs[sourceId] = response.job;
				startJobPolling(sourceId, response.job_id);
			} else {
				sourceLoadingStates[sourceId] = false;
				await new Promise(resolve => setTimeout(resolve, 1000));
				const updated = await api.getDataSource(sourceId);
				updateSourceInData(sourceId, updated);
			}
		} catch (e) {
			sourceErrors[sourceId] = e instanceof Error ? e.message : 'Trigger failed';
			sourceLoadingStates[sourceId] = false;
		}
	}
	
	function startJobPolling(sourceId: string, jobId: string) {
		if (jobPollingIntervals[sourceId]) {
			clearInterval(jobPollingIntervals[sourceId]);
		}
		
		jobPollingIntervals[sourceId] = setInterval(async () => {
			try {
				const job = await api.getJob(jobId);
				activeJobs[sourceId] = job;
				
				if (job.status === 'completed' || job.status === 'failed') {
					clearInterval(jobPollingIntervals[sourceId]);
					delete jobPollingIntervals[sourceId];
					sourceLoadingStates[sourceId] = false;
					
					triggerMessages[sourceId] = {
						source_id: sourceId,
						status: job.status,
						message: job.message || (job.status === 'completed' ? 'Job completed successfully' : 'Job failed'),
						command: job.command,
						started_at: job.started_at,
						job_id: job.id,
						job: job
					};
					
					await new Promise(resolve => setTimeout(resolve, 500));
					const updated = await api.getDataSource(sourceId);
					updateSourceInData(sourceId, updated);
				}
			} catch (e) {
				console.error('Failed to poll job:', e);
			}
		}, 2000);
	}
	
	function updateSourceInData(sourceId: string, updated: GranularDataSource) {
		if (data) {
			const idx = data.sources.findIndex(s => s.id === sourceId);
			if (idx >= 0) {
				data.sources[idx] = updated;
				data = { ...data };
			}
		}
	}

	async function triggerCategoryRefresh(category: string) {
		categoryLoading[category] = true;
		categoryMessages[category] = null;
		
		try {
			const response = await api.triggerCategory(category);
			categoryMessages[category] = response;
			
			await new Promise(resolve => setTimeout(resolve, 1000));
			await fetchStatus();
		} catch (e) {
			console.error(`Failed to trigger category ${category}:`, e);
		} finally {
			categoryLoading[category] = false;
		}
	}

	async function refreshAllSources() {
		if (!data) return;
		bulkRefreshing = true;
		
		for (const category of ['broker', 'prices', 'fundamentals', 'scores']) {
			await triggerCategoryRefresh(category);
		}
		
		bulkRefreshing = false;
		await fetchStatus();
	}

	function toggleCategory(category: string) {
		expandedCategories[category] = !expandedCategories[category];
	}

	function getStatusColor(status: string): string {
		switch (status) {
			case 'fresh':
			case 'healthy':
				return 'variant-filled-success';
			case 'stale':
			case 'warning':
				return 'variant-filled-warning';
			case 'outdated':
			case 'degraded':
			case 'no_data':
				return 'variant-filled-error';
			case 'not_configured':
				return 'variant-filled-surface';
			case 'running':
				return 'variant-filled-primary';
			default:
				return 'variant-filled-surface';
		}
	}

	function getStatusBorder(status: string): string {
		switch (status) {
			case 'fresh':
				return 'border-l-green-500';
			case 'stale':
				return 'border-l-yellow-500';
			case 'outdated':
			case 'no_data':
				return 'border-l-red-500';
			case 'not_configured':
				return 'border-l-gray-400';
			default:
				return 'border-l-gray-500';
		}
	}

	function getCategoryIcon(category: string): string {
		switch (category) {
			case 'broker':
				return '👥';
			case 'prices':
				return '📈';
			case 'fundamentals':
				return '📊';
			case 'scores':
				return '🎯';
			default:
				return '📦';
		}
	}

	function formatTime(isoString: string | null): string {
		if (!isoString) return 'Never';
		const date = new Date(isoString);
		return date.toLocaleString();
	}

	function formatHoursAgo(hours: number | null): string {
		if (hours === null) return 'N/A';
		if (hours < 1) return 'Just now';
		if (hours < 24) return `${hours}h ago`;
		const days = Math.floor(hours / 24);
		return `${days}d ${hours % 24}h ago`;
	}

	function formatNumber(num: number): string {
		return num.toLocaleString();
	}

	function getCoverage(count: number, total: number): number {
		if (total === 0) return 0;
		return Math.round((count / total) * 100);
	}

	function dismissSourceMessage(sourceId: string) {
		triggerMessages[sourceId] = null;
		delete activeJobs[sourceId];
		activeJobs = { ...activeJobs };
	}

	function dismissCategoryMessage(category: string) {
		categoryMessages[category] = null;
	}

	function getSourcesByCategory(category: string): GranularDataSource[] {
		if (!data?.by_category) return [];
		return data.by_category[category] || [];
	}

	onMount(() => {
		fetchStatus();
		refreshInterval = setInterval(fetchStatus, 30000);
	});

	onDestroy(() => {
		if (refreshInterval) {
			clearInterval(refreshInterval);
		}
		Object.values(jobPollingIntervals).forEach(interval => clearInterval(interval));
	});
</script>

<svelte:head>
	<title>Data Status | JejakCuan Admin</title>
</svelte:head>

<div class="space-y-6">
	<header class="flex items-center justify-between flex-wrap gap-4">
		<div>
			<h1 class="h1">Data Status</h1>
			<p class="text-surface-600-300-token">Monitor and manage individual data sources</p>
		</div>
		<div class="flex gap-2">
			<button 
				class="btn variant-filled-primary" 
				on:click={fetchStatus} 
				disabled={loading}
			>
				{#if loading}
					<ProgressRadial width="w-4" stroke={100} meter="stroke-white" track="stroke-white/30" />
				{:else}
					<span>↻</span>
				{/if}
				<span>Refresh Status</span>
			</button>
			<button 
				class="btn variant-filled-tertiary" 
				on:click={refreshAllSources} 
				disabled={bulkRefreshing || loading}
			>
				{#if bulkRefreshing}
					<ProgressRadial width="w-4" stroke={100} meter="stroke-white" track="stroke-white/30" />
					<span>Refreshing All...</span>
				{:else}
					<span>Refresh All Sources</span>
				{/if}
			</button>
		</div>
	</header>

	{#if error}
		<div class="alert variant-filled-error">
			<span>{error}</span>
		</div>
	{/if}

	{#if loading && !data}
		<div class="flex flex-col items-center justify-center p-12 gap-4">
			<ProgressRadial width="w-12" stroke={100} meter="stroke-primary-500" track="stroke-primary-500/30" />
			<span class="text-surface-500">Loading data status...</span>
		</div>
	{:else if data}
		<div class="card p-4 flex items-center justify-between flex-wrap gap-4">
			<div class="flex items-center gap-4">
				<span class="badge text-lg px-4 py-2 {getStatusColor(data.overall_status)}">
					{data.overall_status.toUpperCase()}
				</span>
				<span class="text-surface-500">
					Last checked: {formatTime(data.timestamp)}
				</span>
			</div>
			<div class="flex items-center gap-4">
				<div class="text-sm text-surface-500">
					<span class="font-semibold text-green-500">{data.summary.fresh_sources}</span> fresh |
					<span class="font-semibold text-yellow-500">{data.summary.stale_sources}</span> stale |
					<span class="font-semibold">{data.summary.configured_sources}/{data.summary.total_sources}</span> configured
				</div>
				<span class="text-surface-500 text-sm">Auto-refresh: 30s</span>
			</div>
		</div>

		{#each ['broker', 'prices', 'fundamentals', 'scores'] as category}
			{@const categorySources = getSourcesByCategory(category)}
			{@const categorySummary = data.summary.categories.find(c => c.category === category)}
			{@const isExpanded = expandedCategories[category]}
			{@const isCategoryLoading = categoryLoading[category]}
			{@const categoryMessage = categoryMessages[category]}
			
			<div class="card">
				<div class="p-4 flex items-center justify-between">
					<button
						class="flex items-center gap-3 hover:bg-surface-500/10 transition-colors rounded-lg p-2 -m-2"
						on:click={() => toggleCategory(category)}
					>
						<span class="text-2xl">{getCategoryIcon(category)}</span>
						<div class="text-left">
							<h2 class="h3">{categorySummary?.display_name || category}</h2>
							<p class="text-sm text-surface-500">
								{categorySummary?.total || 0} sources |
								<span class="text-green-500">{categorySummary?.fresh || 0} fresh</span> |
								<span class="text-yellow-500">{categorySummary?.stale || 0} stale</span>
								{#if categorySummary?.not_configured}
									| <span class="text-gray-400">{categorySummary.not_configured} not configured</span>
								{/if}
							</p>
						</div>
						<span class="text-2xl transition-transform ml-4" class:rotate-180={isExpanded}>▼</span>
					</button>
					<button
						class="btn btn-sm variant-ghost-primary ml-4"
						on:click={() => triggerCategoryRefresh(category)}
						disabled={isCategoryLoading || bulkRefreshing}
						title="Refresh all {category} sources"
					>
						{#if isCategoryLoading}
							<ProgressRadial width="w-4" stroke={100} meter="stroke-primary-500" track="stroke-primary-500/30" />
						{:else}
							<span>↻ Refresh All</span>
						{/if}
					</button>
				</div>

				{#if categoryMessage}
					<div class="mx-4 mb-2 p-3 rounded bg-tertiary-500/20 text-tertiary-700 dark:text-tertiary-300 text-sm">
						<div class="flex justify-between items-start">
							<div>
								<div class="font-medium">Category Refresh: {categoryMessage.triggered.length} triggered, {categoryMessage.skipped.length} skipped</div>
								{#if categoryMessage.skipped.length > 0}
									<div class="text-xs mt-1">
										Skipped: {categoryMessage.skipped.map(s => `${s.source_id} (${s.reason})`).join(', ')}
									</div>
								{/if}
							</div>
							<button 
								class="text-tertiary-500 hover:text-tertiary-700 text-lg leading-none"
								on:click={() => dismissCategoryMessage(category)}
							>×</button>
						</div>
					</div>
				{/if}

				{#if isExpanded}
					<div class="p-4 pt-0 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
						{#each categorySources as source (source.id)}
							{@const isLoading = sourceLoadingStates[source.id]}
							{@const sourceError = sourceErrors[source.id]}
							{@const message = triggerMessages[source.id]}
							{@const activeJob = activeJobs[source.id]}
							
							<div class="card p-4 border-l-4 {getStatusBorder(source.status)} relative">
								{#if isLoading && activeJob}
									<div class="absolute inset-0 bg-surface-900/40 backdrop-blur-sm rounded-r-lg flex items-center justify-center z-10">
										<div class="flex flex-col items-center gap-2 p-4 text-center">
											<ProgressRadial width="w-10" stroke={100} meter="stroke-primary-500" track="stroke-primary-500/30" />
											<span class="text-sm font-medium">Running...</span>
											{#if activeJob.duration_secs}
												<span class="text-xs text-surface-400">{activeJob.duration_secs.toFixed(1)}s elapsed</span>
											{/if}
										</div>
									</div>
								{:else if isLoading}
									<div class="absolute inset-0 bg-surface-900/20 backdrop-blur-sm rounded-r-lg flex items-center justify-center z-10">
										<div class="flex flex-col items-center gap-2">
											<ProgressRadial width="w-8" stroke={100} meter="stroke-primary-500" track="stroke-primary-500/30" />
											<span class="text-sm font-medium">Starting...</span>
										</div>
									</div>
								{/if}
								
								<div class="flex items-start justify-between mb-2">
									<div class="flex-1">
										<h3 class="font-bold text-sm">{source.name}</h3>
										<p class="text-xs text-surface-500 mt-0.5">{source.description}</p>
									</div>
									<div class="flex items-center gap-1 ml-2">
										<span class="badge text-xs {getStatusColor(source.status)}">{source.status.replace('_', ' ')}</span>
									</div>
								</div>

								{#if !source.config_status.is_configured}
									<div class="mb-3 p-2 rounded bg-warning-500/20 text-warning-700 dark:text-warning-300 text-xs">
										<div class="font-medium">Not Configured</div>
										<div class="mt-1">Missing: {source.config_status.missing_fields.join(', ')}</div>
									</div>
								{/if}
								
								{#if sourceError}
									<div class="mb-3 p-2 rounded bg-error-500/20 text-error-500 text-xs">
										{sourceError}
									</div>
								{/if}
								
							{#if message}
								{@const isSuccess = message.status === 'completed'}
								{@const isFailed = message.status === 'failed'}
								{@const isStarted = message.status === 'started'}
								<div class="mb-3 p-2 rounded text-xs {isSuccess ? 'bg-green-500/20 text-green-700 dark:text-green-300' : isFailed ? 'bg-error-500/20 text-error-500' : isStarted ? 'bg-blue-500/20 text-blue-700 dark:text-blue-300' : 'bg-tertiary-500/20 text-tertiary-700 dark:text-tertiary-300'}">
									<div class="flex justify-between items-start gap-2">
										<div class="flex-1 min-w-0">
											<div class="font-medium flex items-center gap-2">
												{#if isSuccess}
													<span>✓</span>
												{:else if isFailed}
													<span>✗</span>
												{:else if isStarted}
													<span class="animate-pulse">●</span>
												{/if}
												{message.status.toUpperCase()}
												{#if message.job?.duration_secs}
													<span class="text-surface-400 font-normal">({message.job.duration_secs.toFixed(1)}s)</span>
												{/if}
											</div>
											<div class="mt-1">{message.message}</div>
											{#if message.job?.output && (isSuccess || isFailed)}
												<details class="mt-2">
													<summary class="cursor-pointer hover:opacity-80">View output</summary>
													<pre class="mt-1 p-2 bg-surface-900/30 rounded overflow-x-auto text-[10px] max-h-32 overflow-y-auto whitespace-pre-wrap">{message.job.output}</pre>
												</details>
											{/if}
										</div>
										<button 
											class="hover:opacity-70 text-lg leading-none flex-shrink-0"
											on:click={() => dismissSourceMessage(source.id)}
										>×</button>
									</div>
								</div>
							{/if}
								
								<div class="space-y-1 text-xs">
									<div class="flex justify-between">
										<span class="text-surface-500">Records:</span>
										<span class="font-mono">{formatNumber(source.record_count)}</span>
									</div>
									<div class="flex justify-between">
										<span class="text-surface-500">Last Update:</span>
										<span class="font-mono">{formatHoursAgo(source.freshness_hours)}</span>
									</div>
								</div>

								<div class="mt-3 pt-3 border-t border-surface-500/20 flex justify-end">
									{#if source.can_trigger}
										<button 
											class="btn btn-sm variant-ghost-primary"
											on:click={() => triggerSource(source.id)}
											disabled={isLoading || bulkRefreshing || !source.config_status.is_configured}
											title={source.config_status.is_configured ? `Trigger ${source.name}` : 'Configure this source first'}
										>
											<span class="text-sm" class:animate-spin={isLoading}>↻</span>
											<span>Trigger</span>
										</button>
									{:else}
										<span class="text-xs text-surface-400 italic">Auto-computed</span>
									{/if}
								</div>
							</div>
						{/each}
					</div>
				{/if}
			</div>
		{/each}

		{#if legacySummary}
			<div class="card p-6">
				<h2 class="h3 mb-4">Coverage Summary</h2>
				<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
					<div class="text-center">
						<div class="text-3xl font-bold text-primary-500">{legacySummary.total_stocks}</div>
						<div class="text-surface-500">Total Active Stocks</div>
					</div>
					<div>
						<div class="flex justify-between mb-1">
							<span>Price Data</span>
							<span class="font-mono">{legacySummary.stocks_with_prices}/{legacySummary.total_stocks}</span>
						</div>
						<ProgressBar
							value={getCoverage(legacySummary.stocks_with_prices, legacySummary.total_stocks)}
							max={100}
							height="h-3"
							meter="bg-primary-500"
							track="bg-surface-300-600-token"
						/>
						<div class="text-right text-sm text-surface-500">
							{getCoverage(legacySummary.stocks_with_prices, legacySummary.total_stocks)}%
						</div>
					</div>
					<div>
						<div class="flex justify-between mb-1">
							<span>Score Data</span>
							<span class="font-mono">{legacySummary.stocks_with_scores}/{legacySummary.total_stocks}</span>
						</div>
						<ProgressBar
							value={getCoverage(legacySummary.stocks_with_scores, legacySummary.total_stocks)}
							max={100}
							height="h-3"
							meter="bg-green-500"
							track="bg-surface-300-600-token"
						/>
						<div class="text-right text-sm text-surface-500">
							{getCoverage(legacySummary.stocks_with_scores, legacySummary.total_stocks)}%
						</div>
					</div>
					<div>
						<div class="flex justify-between mb-1">
							<span>Broker Data</span>
							<span class="font-mono">{legacySummary.stocks_with_broker_data}/{legacySummary.total_stocks}</span>
						</div>
						<ProgressBar
							value={getCoverage(legacySummary.stocks_with_broker_data, legacySummary.total_stocks)}
							max={100}
							height="h-3"
							meter="bg-warning-500"
							track="bg-surface-300-600-token"
						/>
						<div class="text-right text-sm text-surface-500">
							{getCoverage(legacySummary.stocks_with_broker_data, legacySummary.total_stocks)}%
						</div>
					</div>
				</div>
			</div>

			<div class="card p-6">
				<h2 class="h3 mb-4">Price Data Range</h2>
				<div class="grid grid-cols-2 gap-4">
					<div class="card variant-soft p-4">
						<div class="text-surface-500 mb-1">Oldest Data</div>
						<div class="font-mono">{formatTime(legacySummary.oldest_price_data)}</div>
					</div>
					<div class="card variant-soft p-4">
						<div class="text-surface-500 mb-1">Newest Data</div>
						<div class="font-mono">{formatTime(legacySummary.newest_price_data)}</div>
					</div>
				</div>
			</div>
		{/if}

		<div class="card p-4 variant-soft-surface">
			<h3 class="font-bold mb-2">Configuration Notes</h3>
			<div class="text-sm space-y-2 text-surface-600-300-token">
				<p><strong>TwelveData:</strong> Set <code class="code">TWELVEDATA_API_KEY</code> environment variable</p>
				<p><strong>Sectors.app:</strong> Set <code class="code">SECTORS_API_KEY</code> environment variable</p>
				<p><strong>Computed Scores:</strong> Use <code class="code">POST /api/stocks/scores/recompute</code> to refresh all scores</p>
				<p><strong>Python Scrapers:</strong> Run the displayed command in your terminal to trigger data refresh</p>
			</div>
		</div>
	{/if}
</div>
