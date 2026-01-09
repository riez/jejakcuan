<script lang="ts">
  import { onMount } from 'svelte';
  import { api, type WatchlistItem, type Stock } from '$lib/api';

  let watchlist = $state<(WatchlistItem & { stock?: Stock })[]>([]);
  let isLoading = $state(true);
  let error = $state<string | null>(null);

  onMount(async () => {
    await loadWatchlist();
  });

  async function loadWatchlist() {
    try {
      const items = await api.getWatchlist();
      // Fetch stock details for each item
      const itemsWithStocks = await Promise.all(
        items.map(async (item) => {
          try {
            const stock = await api.getStock(item.symbol);
            return { ...item, stock };
          } catch {
            return item;
          }
        })
      );
      watchlist = itemsWithStocks;
    } catch (e) {
      error = (e as Error).message;
    } finally {
      isLoading = false;
    }
  }

  async function removeItem(symbol: string) {
    try {
      await api.removeFromWatchlist(symbol);
      watchlist = watchlist.filter((w) => w.symbol !== symbol);
    } catch (e) {
      error = (e as Error).message;
    }
  }
</script>

<svelte:head>
  <title>Watchlist - JejakCuan</title>
</svelte:head>

<div class="space-y-4">
  <h1 class="h1">Watchlist</h1>

  {#if error}
    <aside class="alert variant-filled-error">
      <p>{error}</p>
    </aside>
  {/if}

  {#if isLoading}
    <p>Loading...</p>
  {:else if watchlist.length === 0}
    <div class="card p-8 text-center">
      <p class="text-surface-600-300-token">Your watchlist is empty.</p>
      <a href="/" class="btn variant-filled-primary mt-4">Browse Stocks</a>
    </div>
  {:else}
    <div class="card">
      <div class="table-container">
        <table class="table table-hover">
          <thead>
            <tr>
              <th>Symbol</th>
              <th>Name</th>
              <th>Added</th>
              <th class="text-right">Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each watchlist as item}
              <tr>
                <td>
                  <a href="/stock/{item.symbol}" class="anchor font-bold">
                    {item.symbol}
                  </a>
                </td>
                <td>{item.stock?.name ?? '-'}</td>
                <td class="text-surface-600-300-token">
                  {new Date(item.added_at).toLocaleDateString()}
                </td>
                <td class="text-right">
                  <button
                    onclick={() => removeItem(item.symbol)}
                    class="btn btn-sm variant-ghost-error"
                  >
                    Remove
                  </button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  {/if}
</div>
