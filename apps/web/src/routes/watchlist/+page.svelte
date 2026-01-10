<script lang="ts">
  import { onMount } from 'svelte';
  import { flip } from 'svelte/animate';
  import { api, type WatchlistItem, type Stock, type StockScore } from '$lib/api';
  import { WatchlistCard } from '$lib/components';

  interface WatchlistItemFull extends WatchlistItem {
    stock?: Stock;
    score?: StockScore;
    latestPrice?: number;
    priceChange?: number;
  }

  let watchlist = $state<WatchlistItemFull[]>([]);
  let isLoading = $state(true);
  let error = $state<string | null>(null);
  let draggedIndex = $state<number | null>(null);
  let viewMode = $state<'cards' | 'table'>('cards');

  onMount(async () => {
    await loadWatchlist();
  });

  async function loadWatchlist() {
    try {
      const items = await api.getWatchlist();

      // Fetch stock details, scores, and prices in parallel
      const itemsWithData = await Promise.all(
        items.map(async (item) => {
          const result: WatchlistItemFull = { ...item };

          try {
            const [stock, score, prices] = await Promise.all([
              api.getStock(item.symbol),
              api.getStockScore(item.symbol).catch(() => null),
              api.getStockPrices(item.symbol, 2).catch(() => [])
            ]);

            result.stock = stock;
            result.score = score ?? undefined;

            if (prices.length > 0) {
              result.latestPrice = prices[prices.length - 1].close;
              if (prices.length > 1) {
                const prev = prices[prices.length - 2].close;
                result.priceChange = ((result.latestPrice - prev) / prev) * 100;
              }
            }
          } catch {
            // Ignore individual fetch errors
          }

          return result;
        })
      );

      watchlist = itemsWithData;
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

  // Drag and drop handlers
  function handleDragStart(e: DragEvent, index: number) {
    draggedIndex = index;
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = 'move';
    }
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    if (e.dataTransfer) {
      e.dataTransfer.dropEffect = 'move';
    }
  }

  function handleDrop(e: DragEvent, dropIndex: number) {
    e.preventDefault();
    if (draggedIndex === null || draggedIndex === dropIndex) {
      draggedIndex = null;
      return;
    }

    // Reorder the array
    const newList = [...watchlist];
    const [draggedItem] = newList.splice(draggedIndex, 1);
    newList.splice(dropIndex, 0, draggedItem);
    watchlist = newList;
    draggedIndex = null;

    // TODO: Persist order to backend
  }

  function handleDragEnd() {
    draggedIndex = null;
  }

  // Sort functions
  function sortByScore(ascending: boolean = false) {
    watchlist = [...watchlist].sort((a, b) => {
      const scoreA = a.score?.composite_score ?? 0;
      const scoreB = b.score?.composite_score ?? 0;
      return ascending ? scoreA - scoreB : scoreB - scoreA;
    });
  }

  function sortByChange(ascending: boolean = false) {
    watchlist = [...watchlist].sort((a, b) => {
      const changeA = a.priceChange ?? 0;
      const changeB = b.priceChange ?? 0;
      return ascending ? changeA - changeB : changeB - changeA;
    });
  }
</script>

<svelte:head>
  <title>Watchlist - JejakCuan</title>
</svelte:head>

<div class="space-y-4">
  <div class="flex items-center justify-between flex-wrap gap-4">
    <h1 class="h1">Watchlist</h1>

    <div class="flex items-center gap-2">
      <!-- View toggle -->
      <div class="btn-group variant-ghost">
        <button class={viewMode === 'cards' ? 'variant-filled' : ''} onclick={() => (viewMode = 'cards')}>
          Cards
        </button>
        <button class={viewMode === 'table' ? 'variant-filled' : ''} onclick={() => (viewMode = 'table')}>
          Table
        </button>
      </div>

      <!-- Sort dropdown -->
      <select
        class="select w-auto"
        onchange={(e) => {
          const value = (e.target as HTMLSelectElement).value;
          if (value === 'score-desc') sortByScore(false);
          else if (value === 'score-asc') sortByScore(true);
          else if (value === 'change-desc') sortByChange(false);
          else if (value === 'change-asc') sortByChange(true);
        }}
      >
        <option value="">Sort by...</option>
        <option value="score-desc">Score (High to Low)</option>
        <option value="score-asc">Score (Low to High)</option>
        <option value="change-desc">Change (High to Low)</option>
        <option value="change-asc">Change (Low to High)</option>
      </select>
    </div>
  </div>

  {#if error}
    <aside class="alert variant-filled-error">
      <p>{error}</p>
    </aside>
  {/if}

  {#if isLoading}
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {#each [1, 2, 3] as _}
        <div class="card p-4 animate-pulse">
          <div class="h-6 bg-surface-300 dark:bg-surface-700 rounded w-1/3 mb-2"></div>
          <div class="h-4 bg-surface-300 dark:bg-surface-700 rounded w-2/3"></div>
        </div>
      {/each}
    </div>
  {:else if watchlist.length === 0}
    <div class="card p-8 text-center">
      <p class="text-surface-600-300-token mb-4">Your watchlist is empty.</p>
      <a href="/" class="btn variant-filled-primary">Browse Stocks</a>
    </div>
  {:else if viewMode === 'cards'}
    <!-- Card View with Drag-Drop -->
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {#each watchlist as item, index (item.symbol)}
        <div
          role="listitem"
          draggable="true"
          ondragstart={(e) => handleDragStart(e, index)}
          ondragover={handleDragOver}
          ondrop={(e) => handleDrop(e, index)}
          ondragend={handleDragEnd}
          class:opacity-50={draggedIndex === index}
          animate:flip={{ duration: 200 }}
        >
          <WatchlistCard
            symbol={item.symbol}
            stock={item.stock}
            score={item.score}
            latestPrice={item.latestPrice}
            priceChange={item.priceChange}
            onRemove={() => removeItem(item.symbol)}
          />
        </div>
      {/each}
    </div>
  {:else}
    <!-- Table View -->
    <div class="card">
      <div class="table-container">
        <table class="table table-hover">
          <thead>
            <tr>
              <th>Symbol</th>
              <th>Name</th>
              <th class="text-right">Price</th>
              <th class="text-right">Change</th>
              <th class="text-right">Score</th>
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
                <td class="truncate max-w-[200px]">{item.stock?.name ?? '-'}</td>
                <td class="text-right font-mono">
                  {item.latestPrice?.toLocaleString() ?? '-'}
                </td>
                <td class="text-right">
                  {#if item.priceChange !== undefined}
                    <span
                      class={item.priceChange > 0
                        ? 'text-green-500'
                        : item.priceChange < 0
                          ? 'text-red-500'
                          : ''}
                    >
                      {item.priceChange > 0 ? '+' : ''}{item.priceChange.toFixed(2)}%
                    </span>
                  {:else}
                    -
                  {/if}
                </td>
                <td class="text-right">
                  {#if item.score}
                    <span
                      class="badge {item.score.composite_score >= 70
                        ? 'variant-filled-success'
                        : item.score.composite_score >= 50
                          ? 'variant-filled-warning'
                          : 'variant-filled-error'}"
                    >
                      {item.score.composite_score.toFixed(0)}
                    </span>
                  {:else}
                    -
                  {/if}
                </td>
                <td class="text-right">
                  <button onclick={() => removeItem(item.symbol)} class="btn btn-sm variant-ghost-error">
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

  <!-- Summary Stats -->
  {#if watchlist.length > 0}
    {@const avgScore =
      watchlist.reduce((sum, w) => sum + (w.score?.composite_score ?? 0), 0) / watchlist.length}
    {@const gainers = watchlist.filter((w) => (w.priceChange ?? 0) > 0).length}
    {@const losers = watchlist.filter((w) => (w.priceChange ?? 0) < 0).length}
    <div class="card p-4">
      <h3 class="h4 mb-2">Portfolio Summary</h3>
      <div class="grid grid-cols-2 md:grid-cols-4 gap-4 text-center">
        <div>
          <p class="text-2xl font-bold">{watchlist.length}</p>
          <p class="text-sm text-surface-500">Stocks</p>
        </div>
        <div>
          <p
            class="text-2xl font-bold {avgScore >= 70
              ? 'text-green-500'
              : avgScore >= 50
                ? 'text-yellow-500'
                : 'text-red-500'}"
          >
            {avgScore.toFixed(0)}
          </p>
          <p class="text-sm text-surface-500">Avg Score</p>
        </div>
        <div>
          <p class="text-2xl font-bold text-green-500">{gainers}</p>
          <p class="text-sm text-surface-500">Gainers</p>
        </div>
        <div>
          <p class="text-2xl font-bold text-red-500">{losers}</p>
          <p class="text-sm text-surface-500">Losers</p>
        </div>
      </div>
    </div>
  {/if}
</div>
