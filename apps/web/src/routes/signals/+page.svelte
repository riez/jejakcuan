<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { ProgressRadial } from '@skeletonlabs/skeleton';
  import { SignalCard, SignalFilters } from '$lib/components';
  import { api } from '$lib/api';
  import type { Stock, StockFreshness, StockScore } from '$lib/api';

  interface Signal {
    id: string;
    symbol: string;
    stockName: string;
    type: 'buy' | 'sell' | 'hold';
    strength: 'strong' | 'moderate' | 'weak';
    score: number;
    reason: string;
    timestamp: Date;
    priceAtSignal: number;
    targetPrice?: number;
    stopLoss?: number;
    indicators: string[];
    sector: string;
  }

  interface Filters {
    type: 'all' | 'buy' | 'sell' | 'hold';
    strength: 'all' | 'strong' | 'moderate' | 'weak';
    minScore: number;
    sector: string;
  }

  let signals = $state<Signal[]>([]);
  let filteredSignals = $state<Signal[]>([]);
  let filters = $state<Filters>({ type: 'all', strength: 'all', minScore: 0, sector: '' });
  let isLoading = $state(true);
  let sectors = $state<string[]>([]);
  let language = $state<'en' | 'id'>('id');
  let errorMessage = $state<string | null>(null);

  onMount(async () => {
    await loadSignals();
  });

  function determineSignalType(score: number): 'buy' | 'sell' | 'hold' {
    if (score >= 70) return 'buy';
    if (score <= 40) return 'sell';
    return 'hold';
  }

  function determineStrength(score: number): 'strong' | 'moderate' | 'weak' {
    if (score >= 80 || score <= 25) return 'strong';
    if (score >= 65 || score <= 40) return 'moderate';
    return 'weak';
  }

  async function loadSignals() {
    isLoading = true;
    errorMessage = null;
    
    try {
      const watchlist = await api.getWatchlist().catch(() => []);
      const watchlistSymbols = new Set(watchlist.map((w) => w.symbol.toUpperCase()));

      // First, get top stocks with scores
      let scores = await api.getTopScores(100);

      // If scores are missing, trigger a recompute to avoid placeholder signals.
      if (scores.length === 0) {
        await api.recomputeScores().catch(() => undefined);
        scores = await api.getTopScores(100);
      }

      // Exclude symbols already in watchlist (signals = new opportunities)
      scores = scores.filter((s) => !watchlistSymbols.has(s.symbol.toUpperCase())).slice(0, 20);
      
      if (scores.length === 0) {
        // Fallback: get some stocks without scores
        const { stocks } = await api.getStocks(undefined, 10);
        const filteredStocks = stocks.filter((s) => !watchlistSymbols.has(s.symbol.toUpperCase()));
        
        // Create placeholder signals for stocks without scores
        signals = filteredStocks.map((stock, index) => ({
          id: `${stock.symbol}-${index}`,
          symbol: stock.symbol,
          stockName: stock.name,
          type: 'hold' as const,
          strength: 'weak' as const,
          score: 50,
          reason: language === 'id' 
            ? 'Menunggu data analisis. Klik untuk melihat detail.'
            : 'Awaiting analysis data. Click to view details.',
          timestamp: new Date(),
          priceAtSignal: 0,
          indicators: [],
          sector: stock.sector || 'Unknown'
        }));
      } else {
        // Create signals from scores
        signals = await Promise.all(scores.map(async (score, index) => {
          try {
            const stock = await api.getStock(score.symbol);
            const compositeScore = Math.round(score.composite_score);
            
            return {
              id: `${score.symbol}-${index}`,
              symbol: score.symbol,
              stockName: stock.name,
              type: determineSignalType(compositeScore),
              strength: determineStrength(compositeScore),
              score: compositeScore,
              reason: language === 'id'
                ? `Skor komposit: ${compositeScore}. Teknikal: ${Math.round(score.technical_score)}, Fundamental: ${Math.round(score.fundamental_score)}. Klik untuk detail.`
                : `Composite score: ${compositeScore}. Technical: ${Math.round(score.technical_score)}, Fundamental: ${Math.round(score.fundamental_score)}. Click for details.`,
              timestamp: new Date(score.time),
              priceAtSignal: 0,
              indicators: ['Technical Analysis', 'Fundamental Analysis', 'Sentiment Analysis'],
              sector: stock.sector || 'Unknown'
            } as Signal;
          } catch (err) {
            return {
              id: `${score.symbol}-${index}`,
              symbol: score.symbol,
              stockName: score.symbol,
              type: determineSignalType(Math.round(score.composite_score)),
              strength: determineStrength(Math.round(score.composite_score)),
              score: Math.round(score.composite_score),
              reason: language === 'id'
                ? `Skor: ${Math.round(score.composite_score)}. Klik untuk detail.`
                : `Score: ${Math.round(score.composite_score)}. Click for details.`,
              timestamp: new Date(score.time),
              priceAtSignal: 0,
              indicators: [],
              sector: 'Unknown'
            } as Signal;
          }
        }));
      }
      
      // Extract unique sectors
      sectors = [...new Set(signals.map(s => s.sector).filter(s => s !== 'Unknown'))].sort();
      
      applyFilters();
    } catch (err) {
      console.error('Failed to load signals:', err);
      errorMessage = err instanceof Error ? err.message : 'Failed to load signals';
    } finally {
      isLoading = false;
    }
  }

  function applyFilters() {
    filteredSignals = signals.filter(signal => {
      if (filters.type !== 'all' && signal.type !== filters.type) return false;
      if (filters.strength !== 'all' && signal.strength !== filters.strength) return false;
      if (signal.score < filters.minScore) return false;
      if (filters.sector && signal.sector !== filters.sector) return false;
      return true;
    });
  }

  // Reactively apply filters when they change
  $effect(() => {
    applyFilters();
  });

  // Stats
  let stats = $derived({
    total: signals.length,
    buySignals: signals.filter(s => s.type === 'buy').length,
    sellSignals: signals.filter(s => s.type === 'sell').length,
    strongSignals: signals.filter(s => s.strength === 'strong').length,
  });
</script>

<svelte:head>
  <title>Signals - JejakCuan</title>
</svelte:head>

<div class="space-y-6">
  <div class="flex items-center justify-between flex-wrap gap-4">
    <div>
      <h1 class="h1 text-slate-900 dark:text-slate-100">
        {language === 'id' ? 'Signal Hari Ini' : "Today's Signals"}
      </h1>
      <p class="text-slate-600 dark:text-slate-400 text-sm mt-1">
        {language === 'id' 
          ? 'Klik signal untuk melihat analisis detail di halaman saham.'
          : 'Click a signal to view detailed analysis on the stock page.'}
      </p>
    </div>
    <div class="flex items-center gap-4">
      <!-- Language Toggle -->
      <div class="flex items-center gap-2">
        <button 
          onclick={() => language = 'id'}
          class="btn btn-sm {language === 'id' ? 'variant-filled-primary' : 'variant-ghost-surface'}"
        >
          ID
        </button>
        <button 
          onclick={() => language = 'en'}
          class="btn btn-sm {language === 'en' ? 'variant-filled-primary' : 'variant-ghost-surface'}"
        >
          EN
        </button>
      </div>
      <button 
        onclick={loadSignals}
        class="btn variant-ghost-primary"
        disabled={isLoading}
      >
        {#if isLoading}
          <ProgressRadial width="w-5" stroke={100} meter="stroke-primary-500" track="stroke-primary-500/30" />
          <span>{language === 'id' ? 'Memuat...' : 'Loading...'}</span>
        {:else}
          {language === 'id' ? 'Perbarui' : 'Refresh'}
        {/if}
      </button>
    </div>
  </div>

  {#if errorMessage}
    <div class="card variant-soft-error p-4">
      <p class="text-rose-700 dark:text-rose-300">
        <strong>Error:</strong> {errorMessage}
      </p>
      <p class="text-sm text-slate-600 dark:text-slate-400 mt-2">
        {language === 'id' 
          ? 'Pastikan API server berjalan dan Anda sudah login.'
          : 'Make sure the API server is running and you are logged in.'}
      </p>
    </div>
  {/if}

  <!-- Signal Stats -->
  <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
    <div class="card p-4 text-center">
      <p class="text-2xl font-bold text-slate-900 dark:text-slate-100">{stats.total}</p>
      <p class="text-sm text-slate-600 dark:text-slate-400">Total Signals</p>
    </div>
    <div class="card p-4 text-center">
      <p class="text-2xl font-bold text-emerald-600 dark:text-emerald-400">{stats.buySignals}</p>
      <p class="text-sm text-slate-600 dark:text-slate-400">Buy Signals</p>
    </div>
    <div class="card p-4 text-center">
      <p class="text-2xl font-bold text-rose-600 dark:text-rose-400">{stats.sellSignals}</p>
      <p class="text-sm text-slate-600 dark:text-slate-400">Sell Signals</p>
    </div>
    <div class="card p-4 text-center">
      <p class="text-2xl font-bold text-blue-600 dark:text-blue-400">{stats.strongSignals}</p>
      <p class="text-sm text-slate-600 dark:text-slate-400">Strong Signals</p>
    </div>
  </div>

  <!-- Filters -->
  <SignalFilters bind:filters {sectors} onApply={applyFilters} />

  <!-- Signal List -->
  {#if isLoading}
    <div class="flex items-center justify-center p-8">
      <ProgressRadial stroke={100} meter="stroke-primary-500" track="stroke-primary-500/30" />
    </div>
  {:else if filteredSignals.length === 0}
    <div class="card p-8 text-center">
      <p class="text-slate-600 dark:text-slate-400">
        {language === 'id' ? 'Tidak ada signal yang sesuai filter.' : 'No signals match your filters.'}
      </p>
      <button 
        onclick={() => { filters = { type: 'all', strength: 'all', minScore: 0, sector: '' }; }}
        class="btn variant-ghost-primary mt-4"
      >
        {language === 'id' ? 'Hapus Filter' : 'Clear Filters'}
      </button>
    </div>
  {:else}
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      {#each filteredSignals as signal (signal.id)}
        <SignalCard {signal} />
      {/each}
    </div>
  {/if}

  <!-- Disclaimer -->
  <div class="card variant-soft-warning p-4">
    <p class="text-sm text-amber-800 dark:text-amber-300">
      <strong>Disclaimer:</strong> 
      {language === 'id' 
        ? 'Sinyal ini dihasilkan oleh analisis algoritma dan tidak boleh dianggap sebagai saran keuangan. Selalu lakukan riset sendiri sebelum membuat keputusan investasi.'
        : 'These signals are generated by algorithmic analysis and should not be considered as financial advice. Always do your own research before making investment decisions.'}
    </p>
  </div>
</div>
