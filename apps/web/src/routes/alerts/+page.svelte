<script lang="ts">
	interface AlertConfig {
		id: string;
		name: string;
		enabled: boolean;
		type: 'price' | 'technical' | 'broker' | 'wyckoff';
		threshold?: number;
		symbols: string[];
		channels: ('email' | 'telegram' | 'push')[];
	}
	
	let alerts: AlertConfig[] = [
		{
			id: '1',
			name: 'RSI Oversold Alert',
			enabled: true,
			type: 'technical',
			threshold: 30,
			symbols: ['BBCA', 'BBRI', 'BMRI', 'TLKM'],
			channels: ['telegram', 'push'],
		},
		{
			id: '2',
			name: 'Coordinated Buying',
			enabled: true,
			type: 'broker',
			threshold: 3,
			symbols: [],
			channels: ['telegram', 'email'],
		},
		{
			id: '3',
			name: 'Wyckoff Spring',
			enabled: true,
			type: 'wyckoff',
			symbols: [],
			channels: ['telegram', 'push'],
		},
		{
			id: '4',
			name: 'Price Alert - BBCA',
			enabled: false,
			type: 'price',
			threshold: 10000,
			symbols: ['BBCA'],
			channels: ['push'],
		},
	];
	
	let showNewAlert = false;
	let newAlertType: 'price' | 'technical' | 'broker' | 'wyckoff' = 'technical';
	
	function toggleAlert(id: string) {
		alerts = alerts.map(a => 
			a.id === id ? { ...a, enabled: !a.enabled } : a
		);
	}
	
	function deleteAlert(id: string) {
		alerts = alerts.filter(a => a.id !== id);
	}
	
	function getTypeIcon(type: string): string {
		switch (type) {
			case 'price': return '💰';
			case 'technical': return '📊';
			case 'broker': return '🏦';
			case 'wyckoff': return '📈';
			default: return '🔔';
		}
	}
	
	function getChannelIcon(channel: string): string {
		switch (channel) {
			case 'email': return '✉️';
			case 'telegram': return '📱';
			case 'push': return '🔔';
			default: return '📬';
		}
	}
</script>

<svelte:head>
	<title>Alert Management | JejakCuan</title>
</svelte:head>

<div class="page">
	<header>
		<div>
			<h1>Alert Management</h1>
			<p class="subtitle">Configure your trading alerts and notifications</p>
		</div>
		<button class="btn-primary" on:click={() => showNewAlert = true}>
			+ New Alert
		</button>
	</header>
	
	<div class="alert-list">
		{#each alerts as alert}
			<div class="alert-card" class:disabled={!alert.enabled}>
				<div class="alert-header">
					<span class="type-icon">{getTypeIcon(alert.type)}</span>
					<h3>{alert.name}</h3>
					<label class="toggle">
						<input 
							type="checkbox" 
							checked={alert.enabled}
							on:change={() => toggleAlert(alert.id)}
						/>
						<span class="slider"></span>
					</label>
				</div>
				
				<div class="alert-body">
					<div class="meta-row">
						<span class="label">Type:</span>
						<span class="value type-{alert.type}">{alert.type}</span>
					</div>
					
					{#if alert.threshold}
						<div class="meta-row">
							<span class="label">Threshold:</span>
							<span class="value">{alert.threshold}</span>
						</div>
					{/if}
					
					<div class="meta-row">
						<span class="label">Symbols:</span>
						<span class="value">
							{#if alert.symbols.length > 0}
								{alert.symbols.join(', ')}
							{:else}
								All watchlist
							{/if}
						</span>
					</div>
					
					<div class="channels">
						{#each alert.channels as channel}
							<span class="channel" title={channel}>
								{getChannelIcon(channel)}
							</span>
						{/each}
					</div>
				</div>
				
				<div class="alert-actions">
					<button class="btn-edit">Edit</button>
					<button class="btn-delete" on:click={() => deleteAlert(alert.id)}>Delete</button>
				</div>
			</div>
		{/each}
	</div>
	
	{#if showNewAlert}
		<div class="modal-overlay" on:click={() => showNewAlert = false} on:keypress={() => {}}>
			<div class="modal" on:click|stopPropagation on:keypress={() => {}}>
				<h2>Create New Alert</h2>
				
				<div class="form-group">
					<label>Alert Type</label>
					<select bind:value={newAlertType}>
						<option value="technical">Technical Indicator</option>
						<option value="broker">Broker Flow</option>
						<option value="wyckoff">Wyckoff Pattern</option>
						<option value="price">Price Target</option>
					</select>
				</div>
				
				<div class="form-group">
					<label>Alert Name</label>
					<input type="text" placeholder="My Alert" />
				</div>
				
				<div class="form-group">
					<label>Notification Channels</label>
					<div class="checkbox-group">
						<label><input type="checkbox" checked /> Telegram</label>
						<label><input type="checkbox" /> Email</label>
						<label><input type="checkbox" checked /> Push</label>
					</div>
				</div>
				
				<div class="modal-actions">
					<button class="btn-cancel" on:click={() => showNewAlert = false}>Cancel</button>
					<button class="btn-primary">Create Alert</button>
				</div>
			</div>
		</div>
	{/if}
</div>

<style>
	.page {
		max-width: 900px;
		margin: 0 auto;
		padding: 2rem;
	}
	
	header {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
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
	
	.btn-primary {
		background: #6366f1;
		color: #fff;
		border: none;
		padding: 0.75rem 1.5rem;
		border-radius: 8px;
		font-weight: 600;
		cursor: pointer;
		transition: background 0.2s;
	}
	
	.btn-primary:hover {
		background: #4f46e5;
	}
	
	.alert-list {
		display: flex;
		flex-direction: column;
		gap: 1rem;
	}
	
	.alert-card {
		background: #1a1a2e;
		border-radius: 12px;
		padding: 1.25rem;
		border: 2px solid transparent;
		transition: border-color 0.2s;
	}
	
	.alert-card:hover {
		border-color: #374151;
	}
	
	.alert-card.disabled {
		opacity: 0.6;
	}
	
	.alert-header {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		margin-bottom: 1rem;
	}
	
	.type-icon {
		font-size: 1.5rem;
	}
	
	.alert-header h3 {
		flex: 1;
		margin: 0;
		font-size: 1.1rem;
		color: #fff;
	}
	
	.toggle {
		position: relative;
		width: 48px;
		height: 24px;
	}
	
	.toggle input {
		opacity: 0;
		width: 0;
		height: 0;
	}
	
	.slider {
		position: absolute;
		cursor: pointer;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background: #374151;
		border-radius: 24px;
		transition: 0.3s;
	}
	
	.slider::before {
		content: '';
		position: absolute;
		height: 18px;
		width: 18px;
		left: 3px;
		bottom: 3px;
		background: white;
		border-radius: 50%;
		transition: 0.3s;
	}
	
	.toggle input:checked + .slider {
		background: #10b981;
	}
	
	.toggle input:checked + .slider::before {
		transform: translateX(24px);
	}
	
	.alert-body {
		margin-bottom: 1rem;
	}
	
	.meta-row {
		display: flex;
		gap: 0.5rem;
		margin-bottom: 0.5rem;
		font-size: 0.9rem;
	}
	
	.meta-row .label {
		color: #6b7280;
		min-width: 80px;
	}
	
	.meta-row .value {
		color: #d1d5db;
	}
	
	.type-technical { color: #6366f1; }
	.type-broker { color: #10b981; }
	.type-wyckoff { color: #f59e0b; }
	.type-price { color: #ec4899; }
	
	.channels {
		display: flex;
		gap: 0.5rem;
		margin-top: 0.75rem;
	}
	
	.channel {
		font-size: 1.25rem;
	}
	
	.alert-actions {
		display: flex;
		gap: 0.5rem;
	}
	
	.btn-edit, .btn-delete {
		padding: 0.5rem 1rem;
		border: none;
		border-radius: 6px;
		cursor: pointer;
		font-size: 0.85rem;
		transition: background 0.2s;
	}
	
	.btn-edit {
		background: #374151;
		color: #fff;
	}
	
	.btn-edit:hover {
		background: #4b5563;
	}
	
	.btn-delete {
		background: transparent;
		color: #ef4444;
	}
	
	.btn-delete:hover {
		background: rgba(239, 68, 68, 0.1);
	}
	
	.modal-overlay {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background: rgba(0, 0, 0, 0.7);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 100;
	}
	
	.modal {
		background: #1a1a2e;
		border-radius: 12px;
		padding: 2rem;
		width: 100%;
		max-width: 450px;
	}
	
	.modal h2 {
		margin: 0 0 1.5rem;
		color: #fff;
	}
	
	.form-group {
		margin-bottom: 1.25rem;
	}
	
	.form-group label {
		display: block;
		margin-bottom: 0.5rem;
		color: #9ca3af;
		font-size: 0.9rem;
	}
	
	.form-group input[type="text"],
	.form-group select {
		width: 100%;
		padding: 0.75rem;
		background: #252543;
		border: 1px solid #374151;
		border-radius: 8px;
		color: #fff;
		font-size: 1rem;
	}
	
	.checkbox-group {
		display: flex;
		gap: 1rem;
	}
	
	.checkbox-group label {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		color: #d1d5db;
	}
	
	.modal-actions {
		display: flex;
		justify-content: flex-end;
		gap: 0.75rem;
		margin-top: 1.5rem;
	}
	
	.btn-cancel {
		padding: 0.75rem 1.5rem;
		background: transparent;
		border: 1px solid #374151;
		border-radius: 8px;
		color: #9ca3af;
		cursor: pointer;
	}
	
	.btn-cancel:hover {
		background: #252543;
	}
</style>
