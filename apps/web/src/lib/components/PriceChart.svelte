<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { createChart, ColorType, type IChartApi } from 'lightweight-charts';

  interface PriceData {
    time: string;
    open: number;
    high: number;
    low: number;
    close: number;
    volume: number;
  }

  let {
    prices = [] as PriceData[],
    height = 400,
    showVolume = true,
    showEma = true
  } = $props();

  let chartContainer: HTMLDivElement;
  let chart: IChartApi | null = null;

  function deduplicateAndSort(data: PriceData[]): PriceData[] {
    const byDate = new Map<string, PriceData>();
    for (const p of data) {
      const dateKey = p.time.split('T')[0];
      byDate.set(dateKey, p);
    }
    return Array.from(byDate.values()).sort((a, b) => 
      a.time.split('T')[0].localeCompare(b.time.split('T')[0])
    );
  }

  onMount(() => {
    if (!chartContainer || prices.length === 0) return;

    const cleanPrices = deduplicateAndSort(prices);
    if (cleanPrices.length === 0) return;

    chart = createChart(chartContainer, {
      layout: {
        background: { type: ColorType.Solid, color: 'transparent' },
        textColor: '#9ca3af'
      },
      grid: {
        vertLines: { color: 'rgba(156, 163, 175, 0.1)' },
        horzLines: { color: 'rgba(156, 163, 175, 0.1)' }
      },
      width: chartContainer.clientWidth,
      height: height,
      crosshair: {
        mode: 1
      },
      rightPriceScale: {
        borderColor: 'rgba(156, 163, 175, 0.2)'
      },
      timeScale: {
        borderColor: 'rgba(156, 163, 175, 0.2)',
        timeVisible: true
      }
    });

    // Candlestick series
    const candleSeries = chart.addCandlestickSeries({
      upColor: '#22c55e',
      downColor: '#ef4444',
      borderDownColor: '#ef4444',
      borderUpColor: '#22c55e',
      wickDownColor: '#ef4444',
      wickUpColor: '#22c55e'
    });

    const candleData = cleanPrices.map((p) => ({
      time: p.time.split('T')[0],
      open: p.open,
      high: p.high,
      low: p.low,
      close: p.close
    }));
    candleSeries.setData(candleData as Parameters<typeof candleSeries.setData>[0]);

    if (showEma && cleanPrices.length >= 20) {
      const emaValues = calculateEMA(
        cleanPrices.map((p) => p.close),
        20
      );
      const emaSeries = chart.addLineSeries({
        color: '#3b82f6',
        lineWidth: 2,
        title: 'EMA20'
      });

      const emaData = emaValues
        .map((value, i) => ({
          time: cleanPrices[i].time.split('T')[0],
          value: value
        }))
        .filter((d) => d.value > 0);

      emaSeries.setData(emaData as Parameters<typeof emaSeries.setData>[0]);
    }

    if (showVolume) {
      const volumeSeries = chart.addHistogramSeries({
        color: '#6366f1',
        priceFormat: { type: 'volume' },
        priceScaleId: 'volume'
      });

      chart.priceScale('volume').applyOptions({
        scaleMargins: { top: 0.8, bottom: 0 }
      });

      const volumeData = cleanPrices.map((p) => ({
        time: p.time.split('T')[0],
        value: p.volume,
        color: p.close >= p.open ? 'rgba(34, 197, 94, 0.5)' : 'rgba(239, 68, 68, 0.5)'
      }));

      volumeSeries.setData(volumeData as Parameters<typeof volumeSeries.setData>[0]);
    }

    chart.timeScale().fitContent();

    // Handle resize
    const resizeObserver = new ResizeObserver(() => {
      if (chart && chartContainer) {
        chart.applyOptions({ width: chartContainer.clientWidth });
      }
    });
    resizeObserver.observe(chartContainer);

    return () => resizeObserver.disconnect();
  });

  onDestroy(() => {
    if (chart) {
      chart.remove();
      chart = null;
    }
  });

  // Simple EMA calculation
  function calculateEMA(prices: number[], period: number): number[] {
    const k = 2 / (period + 1);
    const emaValues: number[] = new Array(period - 1).fill(0);

    // First EMA is SMA
    let ema = prices.slice(0, period).reduce((a, b) => a + b, 0) / period;
    emaValues.push(ema);

    for (let i = period; i < prices.length; i++) {
      ema = prices[i] * k + ema * (1 - k);
      emaValues.push(ema);
    }

    return emaValues;
  }
</script>

<div bind:this={chartContainer} class="w-full rounded-lg overflow-hidden"></div>
