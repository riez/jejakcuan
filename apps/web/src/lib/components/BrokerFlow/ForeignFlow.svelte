<script lang="ts">
	interface FlowData {
		date: string;
		netForeign: number;
		netInstitutional: number;
		netRetail: number;
	}
	
	export let data: FlowData[] = [];
	export let symbol = '';
	
	$: totalForeign = data.reduce((sum, d) => sum + d.netForeign, 0);
	$: totalInstitutional = data.reduce((sum, d) => sum + d.netInstitutional, 0);
	$: cumulativeForeign = data.reduce((acc, d) => {
		const last = acc.length > 0 ? acc[acc.length - 1] : 0;
		acc.push(last + d.netForeign);
		return acc;
	}, [] as number[]);
	
	function formatValue(value: number): string {
		const abs = Math.abs(value);
		if (abs >= 1e12) return `${(value / 1e12).toFixed(1)}T`;
		if (abs >= 1e9) return `${(value / 1e9).toFixed(1)}B`;
		if (abs >= 1e6) return `${(value / 1e6).toFixed(1)}M`;
		return value.toLocaleString();
	}
	
	function getClass(value: number): string {
		if (value > 0) return 'positive';
		if (value < 0) return 'negative';
		return 'neutral';
	}
</script>

<div class="foreign-flow">
	<div class="header">
		<h3>Foreign & Institutional Flow</h3>
		{#if symbol}
			<span class="symbol">{symbol}</span>
		{/if}
	</div>
	
	<div class="summary">
		<div class="metric {getClass(totalForeign)}">
			<span class="label">Net Foreign</span>
			<span class="value">{formatValue(totalForeign)}</span>
		</div>
		<div class="metric {getClass(totalInstitutional)}">
			<span class="label">Net Institutional</span>
			<span class="value">{formatValue(totalInstitutional)}</span>
		</div>
	</div>
	
	<div class="chart-placeholder">
		<svg viewBox="0 0 400 100" class="flow-chart">
			{#if data.length > 0}
				<line x1="0" y1="50" x2="400" y2="50" stroke="#333" stroke-dasharray="4"/>
				<polyline
					fill="none"
					stroke={totalForeign >= 0 ? '#10b981' : '#ef4444'}
					stroke-width="2"
					points={cumulativeForeign.map((v, i) => {
						const x = (i / (data.length - 1)) * 400;
						const max = Math.max(...cumulativeForeign.map(Math.abs)) || 1;
						const y = 50 - (v / max) * 40;
						return `${x},${y}`;
					}).join(' ')}
				/>
			{:else}
				<text x="200" y="50" text-anchor="middle" fill="#666">No data</text>
			{/if}
		</svg>
	</div>
	
	<div class="legend">
		<span class="legend-item positive">● Inflow</span>
		<span class="legend-item negative">● Outflow</span>
	</div>
</div>

<style>
	.foreign-flow {
		background: var(--card-bg, #1a1a2e);
		border-radius: 8px;
		padding: 1rem;
	}
	
	.header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		margin-bottom: 1rem;
	}
	
	h3 {
		margin: 0;
		font-size: 1.1rem;
		color: var(--text-primary, #fff);
	}
	
	.symbol {
		font-weight: 700;
		color: var(--accent, #6366f1);
	}
	
	.summary {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 1rem;
		margin-bottom: 1rem;
	}
	
	.metric {
		padding: 0.75rem;
		background: var(--bg-secondary, #2a2a4e);
		border-radius: 6px;
		text-align: center;
	}
	
	.metric .label {
		display: block;
		font-size: 0.75rem;
		color: var(--text-secondary, #aaa);
		margin-bottom: 0.25rem;
	}
	
	.metric .value {
		display: block;
		font-size: 1.25rem;
		font-weight: 700;
	}
	
	.metric.positive .value { color: #10b981; }
	.metric.negative .value { color: #ef4444; }
	.metric.neutral .value { color: #6b7280; }
	
	.chart-placeholder {
		height: 100px;
		margin-bottom: 0.5rem;
	}
	
	.flow-chart {
		width: 100%;
		height: 100%;
	}
	
	.legend {
		display: flex;
		justify-content: center;
		gap: 1rem;
		font-size: 0.75rem;
	}
	
	.legend-item.positive { color: #10b981; }
	.legend-item.negative { color: #ef4444; }
</style>
