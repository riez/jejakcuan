<script lang="ts">
  import { onMount } from 'svelte';
  import { createChart, ColorType, type IChartApi, type ISeriesApi } from 'lightweight-charts';
  import type { StockPrice, TechnicalResponse } from '$lib/api';

  let {
    prices,
    technical,
    height = 400,
  }: {
    prices: StockPrice[];
    technical: TechnicalResponse | null;
    height?: number;
  } = $props();

  let chartContainer: HTMLDivElement;
  let chart: IChartApi | null = null;
  let candleSeries: ISeriesApi<'Candlestick'> | null = null;
  let volumeSeries: ISeriesApi<'Histogram'> | null = null;

  let selectedTimeframe = $state('1M');
  let showVolume = $state(true);
  let showSupportResistance = $state(true);

  const timeframes = ['1W', '1M', '3M', '6M', '1Y'];

  onMount(() => {
    if (!chartContainer) return;

    chart = createChart(chartContainer, {
      height,
      layout: {
        background: { type: ColorType.Solid, color: 'transparent' },
        textColor: '#9ca3af',
      },
      grid: {
        vertLines: { color: 'rgba(156, 163, 175, 0.1)' },
        horzLines: { color: 'rgba(156, 163, 175, 0.1)' },
      },
      crosshair: { mode: 1 },
      rightPriceScale: { borderColor: 'rgba(156, 163, 175, 0.2)' },
      timeScale: { borderColor: 'rgba(156, 163, 175, 0.2)' },
    });

    candleSeries = chart.addCandlestickSeries({
      upColor: '#10b981',
      downColor: '#ef4444',
      borderUpColor: '#10b981',
      borderDownColor: '#ef4444',
      wickUpColor: '#10b981',
      wickDownColor: '#ef4444',
    });

    volumeSeries = chart.addHistogramSeries({
      priceFormat: { type: 'volume' },
      priceScaleId: '',
    });

    volumeSeries.priceScale().applyOptions({
      scaleMargins: { top: 0.8, bottom: 0 },
    });

    updateChart();

    return () => {
      chart?.remove();
    };
  });

  function updateChart() {
    if (!chart || !candleSeries || !volumeSeries || !prices.length) return;

    const candleData = prices.map((p) => ({
      time: p.time.split('T')[0],
      open: Number(p.open),
      high: Number(p.high),
      low: Number(p.low),
      close: Number(p.close),
    }));

    candleSeries.setData(candleData as Parameters<typeof candleSeries.setData>[0]);

    if (showVolume) {
      const volumeData = prices.map((p) => ({
        time: p.time.split('T')[0],
        value: p.volume,
        color:
          Number(p.close) >= Number(p.open)
            ? 'rgba(16, 185, 129, 0.5)'
            : 'rgba(239, 68, 68, 0.5)',
      }));
      volumeSeries.setData(volumeData as Parameters<typeof volumeSeries.setData>[0]);
    } else {
      volumeSeries.setData([]);
    }

    chart.timeScale().fitContent();
  }

  $effect(() => {
    updateChart();
  });
</script>

<div class="space-y-4">
  <div class="flex flex-wrap items-center justify-between gap-2">
    <div class="flex gap-1">
      {#each timeframes as tf}
        <button
          class="btn btn-sm {selectedTimeframe === tf
            ? 'variant-filled-primary'
            : 'variant-ghost-surface'}"
          onclick={() => (selectedTimeframe = tf)}
        >
          {tf}
        </button>
      {/each}
    </div>

    <div class="flex gap-3">
      <label class="flex items-center gap-1 text-sm cursor-pointer">
        <input type="checkbox" class="checkbox" bind:checked={showVolume} />
        Vol
      </label>
      <label class="flex items-center gap-1 text-sm cursor-pointer">
        <input type="checkbox" class="checkbox" bind:checked={showSupportResistance} />
        S/R
      </label>
    </div>
  </div>

  <div bind:this={chartContainer} class="w-full rounded-lg overflow-hidden"></div>

  {#if technical}
    <div class="grid grid-cols-2 md:grid-cols-4 gap-3">
      <div class="card p-3 text-center">
        <span class="text-xs text-surface-500">RSI (14)</span>
        <div
          class="text-lg font-bold {technical.rsi < 30
            ? 'text-success-500'
            : technical.rsi > 70
              ? 'text-error-500'
              : ''}"
        >
          {technical.rsi?.toFixed(1) ?? '-'}
        </div>
        <span
          class="text-xs {technical.rsi < 30
            ? 'text-success-500'
            : technical.rsi > 70
              ? 'text-error-500'
              : 'text-surface-400'}"
        >
          {technical.rsi < 30 ? 'Oversold' : technical.rsi > 70 ? 'Overbought' : 'Neutral'}
        </span>
      </div>

      <div class="card p-3 text-center">
        <span class="text-xs text-surface-500">MACD</span>
        <div
          class="text-lg font-bold {technical.macd_signal === 'bullish'
            ? 'text-success-500'
            : technical.macd_signal === 'bearish'
              ? 'text-error-500'
              : ''}"
        >
          {technical.macd_histogram?.toFixed(2) ?? '-'}
        </div>
        <span class="text-xs capitalize">{technical.macd_signal ?? 'N/A'}</span>
      </div>

      <div class="card p-3 text-center">
        <span class="text-xs text-surface-500">Support</span>
        <div class="text-lg font-bold text-success-500">
          {technical.support?.[0]?.toLocaleString('id-ID') ?? '-'}
        </div>
      </div>

      <div class="card p-3 text-center">
        <span class="text-xs text-surface-500">Resistance</span>
        <div class="text-lg font-bold text-error-500">
          {technical.resistance?.[0]?.toLocaleString('id-ID') ?? '-'}
        </div>
      </div>
    </div>
  {/if}
</div>
