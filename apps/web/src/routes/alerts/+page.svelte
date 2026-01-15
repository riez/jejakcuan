<script lang="ts">
	import { SlideToggle, getModalStore } from '@skeletonlabs/skeleton';
	import type { ModalSettings } from '@skeletonlabs/skeleton';

	const modalStore = getModalStore();

	interface AlertConfig {
		id: string;
		name: string;
		enabled: boolean;
		type: 'price' | 'technical' | 'broker' | 'wyckoff';
		threshold?: number;
		symbols: string[];
		channels: ('email' | 'telegram' | 'push')[];
	}
	
	let alerts = $state<AlertConfig[]>([
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
	]);
	
	let showNewAlert = $state(false);
	let newAlertType = $state<'price' | 'technical' | 'broker' | 'wyckoff'>('technical');
	let newAlertName = $state('');
	let newAlertChannels = $state({ telegram: true, email: false, push: true });
	
	function toggleAlert(id: string) {
		alerts = alerts.map(a => 
			a.id === id ? { ...a, enabled: !a.enabled } : a
		);
	}
	
	function deleteAlert(id: string, name: string) {
		const modal: ModalSettings = {
			type: 'confirm',
			title: 'Delete Alert',
			body: `Are you sure you want to delete "${name}"?`,
			response: (confirmed: boolean) => {
				if (confirmed) {
					alerts = alerts.filter(a => a.id !== id);
				}
			}
		};
		modalStore.trigger(modal);
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
	
	function getTypeClass(type: string): string {
		switch (type) {
			case 'technical': return 'variant-soft-primary';
			case 'broker': return 'variant-soft-success';
			case 'wyckoff': return 'variant-soft-warning';
			case 'price': return 'variant-soft-tertiary';
			default: return 'variant-soft';
		}
	}
	
	function createAlert() {
		const channels: ('email' | 'telegram' | 'push')[] = [];
		if (newAlertChannels.telegram) channels.push('telegram');
		if (newAlertChannels.email) channels.push('email');
		if (newAlertChannels.push) channels.push('push');
		
		const newAlert: AlertConfig = {
			id: Date.now().toString(),
			name: newAlertName || 'New Alert',
			enabled: true,
			type: newAlertType,
			symbols: [],
			channels,
		};
		
		alerts = [...alerts, newAlert];
		showNewAlert = false;
		newAlertName = '';
		newAlertChannels = { telegram: true, email: false, push: true };
	}
</script>

<svelte:head>
	<title>Alert Management | JejakCuan</title>
</svelte:head>

<div class="space-y-6 max-w-4xl mx-auto">
	<header class="flex items-center justify-between">
		<div>
			<h1 class="h1">Alert Management</h1>
			<p class="text-surface-600-300-token">Configure your trading alerts and notifications</p>
		</div>
		<button class="btn variant-filled-primary" onclick={() => showNewAlert = true}>
			+ New Alert
		</button>
	</header>
	
	<!-- Alert List -->
	<div class="space-y-4">
		{#each alerts as alert}
			<div class="card p-6 {!alert.enabled ? 'opacity-60' : ''} hover:ring-2 ring-primary-500 transition-all">
				<div class="flex items-center gap-4 mb-4">
					<span class="text-2xl">{getTypeIcon(alert.type)}</span>
					<h3 class="h4 flex-1">{alert.name}</h3>
					<div class="flex items-center gap-2">
						<SlideToggle 
							name={`alert-${alert.id}`}
							checked={alert.enabled}
							on:change={() => toggleAlert(alert.id)}
							size="sm"
						/>
						<span class="text-sm text-surface-500">{alert.enabled ? 'Enabled' : 'Disabled'}</span>
					</div>
				</div>
				
				<div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-4">
					<div>
						<span class="text-sm text-surface-500">Type:</span>
						<span class="badge {getTypeClass(alert.type)} ml-2 capitalize">{alert.type}</span>
					</div>
					
					{#if alert.threshold}
						<div>
							<span class="text-sm text-surface-500">Threshold:</span>
							<span class="ml-2 font-medium">{alert.threshold}</span>
						</div>
					{/if}
					
					<div>
						<span class="text-sm text-surface-500">Symbols:</span>
						<span class="ml-2">
							{#if alert.symbols.length > 0}
								{alert.symbols.join(', ')}
							{:else}
								<span class="text-surface-500">All watchlist</span>
							{/if}
						</span>
					</div>
				</div>
				
				<!-- Channels -->
				<div class="flex items-center gap-2 mb-4">
					<span class="text-sm text-surface-500">Channels:</span>
					{#each alert.channels as channel}
						<span class="badge variant-soft text-lg" title={channel}>
							{getChannelIcon(channel)}
						</span>
					{/each}
				</div>
				
				<!-- Actions -->
				<div class="flex gap-2">
					<button class="btn btn-sm variant-ghost-surface">Edit</button>
					<button class="btn btn-sm variant-ghost-error" onclick={() => deleteAlert(alert.id, alert.name)}>Delete</button>
				</div>
			</div>
		{/each}
	</div>
	
	{#if alerts.length === 0}
		<div class="card p-8 text-center">
			<p class="text-surface-600-300-token mb-4">No alerts configured yet.</p>
			<button class="btn variant-filled-primary" onclick={() => showNewAlert = true}>
				Create Your First Alert
			</button>
		</div>
	{/if}
	
	<!-- New Alert Modal -->
	{#if showNewAlert}
		<!-- svelte-ignore a11y_click_events_have_key_events a11y_interactive_supports_focus -->
		<div class="fixed inset-0 bg-surface-backdrop-token flex items-center justify-center z-50 p-4" onclick={() => showNewAlert = false} role="dialog" aria-modal="true" aria-labelledby="modal-title">
			<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_noninteractive_element_interactions -->
			<div class="card p-6 w-full max-w-md" onclick={(e) => e.stopPropagation()}>
				<h2 id="modal-title" class="h3 mb-6">Create New Alert</h2>
				
				<form onsubmit={(e) => { e.preventDefault(); createAlert(); }} class="space-y-4">
					<label class="label">
						<span>Alert Type</span>
						<select class="select" bind:value={newAlertType}>
							<option value="technical">Technical Indicator</option>
							<option value="broker">Broker Flow</option>
							<option value="wyckoff">Wyckoff Pattern</option>
							<option value="price">Price Target</option>
						</select>
					</label>
					
					<label class="label">
						<span>Alert Name</span>
						<input type="text" class="input" placeholder="My Alert" bind:value={newAlertName} />
					</label>
					
					<fieldset>
						<legend class="text-sm text-surface-500 mb-2">Notification Channels</legend>
						<div class="flex gap-4">
							<label class="flex items-center gap-2">
								<input type="checkbox" class="checkbox" bind:checked={newAlertChannels.telegram} />
								<span>Telegram</span>
							</label>
							<label class="flex items-center gap-2">
								<input type="checkbox" class="checkbox" bind:checked={newAlertChannels.email} />
								<span>Email</span>
							</label>
							<label class="flex items-center gap-2">
								<input type="checkbox" class="checkbox" bind:checked={newAlertChannels.push} />
								<span>Push</span>
							</label>
						</div>
					</fieldset>
					
					<div class="flex justify-end gap-2 pt-4">
						<button type="button" class="btn variant-ghost-surface" onclick={() => showNewAlert = false}>Cancel</button>
						<button type="submit" class="btn variant-filled-primary">Create Alert</button>
					</div>
				</form>
			</div>
		</div>
	{/if}
</div>
