<script lang="ts">
  import '../app.css';
  import { AppShell, AppBar } from '@skeletonlabs/skeleton';
  
  let { children } = $props();
</script>

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
      </svelte:fragment>
    </AppBar>
  </svelte:fragment>
  
  <main class="container mx-auto p-4">
    {@render children()}
  </main>
</AppShell>
