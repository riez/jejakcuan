<script lang="ts">
  import '../app.css';
  import {
    AppShell,
    AppBar,
    Modal,
    Toast,
    Drawer,
    ProgressRadial,
    initializeStores,
    getDrawerStore
  } from '@skeletonlabs/skeleton';
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/stores';
  import { auth } from '$lib/stores/auth';

  // Initialize Skeleton stores for Modal, Toast, and Drawer
  initializeStores();

  let { children } = $props();

  onMount(() => {
    auth.init();
  });

  // Redirect to login if not authenticated (except on login page)
  $effect(() => {
    if (!$auth.isLoading && !$auth.isAuthenticated && $page.url.pathname !== '/login') {
      goto('/login');
    }
  });

  async function handleLogout() {
    await auth.logout();
    goto('/login');
  }
</script>

<!-- Skeleton Overlay Components -->
<Modal />
<Toast />
<Drawer />

{#if $auth.isAuthenticated}
  <AppShell>
    <svelte:fragment slot="header">
      <AppBar>
        <svelte:fragment slot="lead">
          <strong class="text-xl">JejakCuan</strong>
        </svelte:fragment>
        <svelte:fragment slot="trail">
          <a href="/" class="btn btn-sm variant-ghost-surface">Screener</a>
          <a href="/watchlist" class="btn btn-sm variant-ghost-surface">Watchlist</a>
          <a href="/market" class="btn btn-sm variant-ghost-surface">Market</a>
          <a href="/signals" class="btn btn-sm variant-ghost-surface">Signals</a>
          <a href="/admin/data-status" class="btn btn-sm variant-ghost-tertiary">Data Status</a>
          <button onclick={handleLogout} class="btn btn-sm variant-ghost-error">Logout</button>
        </svelte:fragment>
      </AppBar>
    </svelte:fragment>

    <main class="container mx-auto p-4">
      {@render children()}
    </main>
  </AppShell>
{:else}
  <main class="container mx-auto p-4">
    {#if $auth.isLoading}
      <div class="flex items-center justify-center min-h-[50vh]">
        <ProgressRadial stroke={100} meter="stroke-primary-500" track="stroke-primary-500/30" />
      </div>
    {:else}
      {@render children()}
    {/if}
  </main>
{/if}
