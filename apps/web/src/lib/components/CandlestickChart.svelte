<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { createChart, ColorType, type IChartApi, type ISeriesApi } from 'lightweight-charts';

  interface CandleData {
    time: string;
    open: number;
    high: number;
    low: number;
    close: number;
  }

  interface VolumeData {
    time: string;
    value: number;
    color: string;
  }

  let {
    data = [] as CandleData[],
    ema20 = [] as { time: string; value: number }[],
    height = 400
  } = $props();

  let chartContainer: HTMLDivElement;
  let chart: IChartApi | null = null;
  let candleSeries: ISeriesApi<'Candlestick'> | null = null;
  let emaSeries: ISeriesApi<'Line'> | null = null;
  let volumeSeries: ISeriesApi<'Histogram'> | null = null;

  onMount(() => {
    if (!chartContainer) return;

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
    candleSeries = chart.addCandlestickSeries({
      upColor: '#22c55e',
      downColor: '#ef4444',
      borderDownColor: '#ef4444',
      borderUpColor: '#22c55e',
      wickDownColor: '#ef4444',
      wickUpColor: '#22c55e'
    });

    // EMA line
    emaSeries = chart.addLineSeries({
      color: '#3b82f6',
      lineWidth: 2,
      title: 'EMA20'
    });

    // Volume histogram (using separate price scale)
    volumeSeries = chart.addHistogramSeries({
      color: '#6366f1',
      priceFormat: {
        type: 'volume'
      },
      priceScaleId: 'volume'
    });

    chart.priceScale('volume').applyOptions({
      scaleMargins: {
        top: 0.8,
        bottom: 0
      }
    });

    // Set data
    updateData();

    // Handle resize
    const resizeObserver = new ResizeObserver(() => {
      if (chart && chartContainer) {
        chart.applyOptions({ width: chartContainer.clientWidth });
      }
    });
    resizeObserver.observe(chartContainer);

    return () => {
      resizeObserver.disconnect();
    };
  });

  onDestroy(() => {
    if (chart) {
      chart.remove();
      chart = null;
    }
  });

  function updateData() {
    if (!candleSeries || !emaSeries || !volumeSeries) return;

    // Format candle data
    const formattedCandles = data.map((d) => ({
      time: d.time.split('T')[0],
      open: d.open,
      high: d.high,
      low: d.low,
      close: d.close
    }));

    candleSeries.setData(formattedCandles as Parameters<typeof candleSeries.setData>[0]);

    // Format EMA data
    if (ema20.length > 0) {
      const formattedEma = ema20.map((d) => ({
        time: d.time.split('T')[0],
        value: d.value
      }));
      emaSeries.setData(formattedEma as Parameters<typeof emaSeries.setData>[0]);
    }

    // Format volume data with color based on price change
    const volumeData: VolumeData[] = data.map((d) => ({
      time: d.time.split('T')[0],
      value: 0, // Will be set from actual volume data
      color: d.close >= d.open ? 'rgba(34, 197, 94, 0.5)' : 'rgba(239, 68, 68, 0.5)'
    }));

    // Note: Volume should come from the price data
    // For now, use a placeholder - in real usage, pass volume separately
    volumeSeries.setData(volumeData as Parameters<typeof volumeSeries.setData>[0]);

    // Fit content
    if (chart) {
      chart.timeScale().fitContent();
    }
  }

  // React to data changes
  $effect(() => {
    if (data && candleSeries) {
      updateData();
    }
  });
</script>

<div bind:this={chartContainer} class="w-full rounded-lg overflow-hidden"></div>
