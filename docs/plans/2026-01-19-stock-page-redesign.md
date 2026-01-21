# Stock Page Redesign Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Transform the stock detail page from a tab-based exploration tool into a decision-support dashboard that answers "should I buy?" within 30 seconds.

**Architecture:** Replace 3-tab layout with single-scroll accordion sections. Add sticky header with signal badge. Create hero Analysis Summary card. Move Data Source panel to footer. Add suspicious activity detection, business context, and news integration with browser automation.

**Tech Stack:** Svelte 5 (runes), SkeletonUI, Tailwind CSS, Rust/Axum API, Python scrapers with Playwright

---

## Phase 1: Core UI Restructure

### Task 1.1: Create StickyStockHeader Component

**Files:**
- Create: `apps/web/src/lib/components/StickyStockHeader.svelte`

**Step 1: Create the component file**

```svelte
<script lang="ts">
  import type { Stock, StockPrice, StockScore } from '$lib/api';

  let {
    stock,
    latestPrice,
    score,
    inWatchlist,
    dataFresh,
    onToggleWatchlist,
    onRefreshData,
  }: {
    stock: Stock;
    latestPrice: StockPrice | null;
    score: StockScore | null;
    inWatchlist: boolean;
    dataFresh: boolean;
    onToggleWatchlist: () => void;
    onRefreshData: () => void;
  } = $props();

  let priceChange = $derived(() => {
    if (!latestPrice) return { value: 0, percent: 0 };
    const prev = latestPrice.open;
    const curr = latestPrice.close;
    return {
      value: curr - prev,
      percent: prev > 0 ? ((curr - prev) / prev) * 100 : 0,
    };
  });

  let signal = $derived(() => {
    if (!score) return { label: 'LOADING', color: 'variant-ghost-surface' };
    const composite = Number(score.composite_score);
    if (composite >= 70) return { label: 'BUY', color: 'variant-filled-success' };
    if (composite >= 50) return { label: 'HOLD', color: 'variant-filled-warning' };
    return { label: 'SELL', color: 'variant-filled-error' };
  });
</script>

<header
  class="sticky top-0 z-50 bg-surface-100-800-token border-b border-surface-300-600-token backdrop-blur-sm"
>
  <div class="container mx-auto px-4 py-3">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-4">
        <a href="/screener" class="btn btn-sm variant-ghost-surface">
          <span class="text-lg">&larr;</span>
        </a>
        <div>
          <div class="flex items-center gap-2">
            <span class="text-2xl font-bold font-mono">{stock.symbol}</span>
            <span class="badge {signal().color} text-sm">{signal().label}</span>
          </div>
          <div class="flex items-center gap-2 text-sm">
            <span class="font-mono font-semibold">
              {latestPrice?.close?.toLocaleString('id-ID') ?? '-'}
            </span>
            <span
              class={priceChange().percent >= 0 ? 'text-success-500' : 'text-error-500'}
            >
              {priceChange().percent >= 0 ? '+' : ''}{priceChange().percent.toFixed(2)}%
            </span>
          </div>
        </div>
      </div>

      <div class="flex items-center gap-2">
        {#if !dataFresh}
          <button
            class="badge variant-soft-warning cursor-pointer"
            onclick={onRefreshData}
          >
            Stale
          </button>
        {/if}
        <button
          class="btn btn-sm {inWatchlist ? 'variant-filled-primary' : 'variant-ghost-primary'}"
          onclick={onToggleWatchlist}
          aria-label={inWatchlist ? 'Remove from watchlist' : 'Add to watchlist'}
        >
          {inWatchlist ? '★' : '☆'}
        </button>
      </div>
    </div>
  </div>
</header>
```

**Step 2: Verify TypeScript compiles**

Run: `cd apps/web && npm run check`
Expected: No errors related to StickyStockHeader

**Step 3: Commit**

```bash
git add apps/web/src/lib/components/StickyStockHeader.svelte
git commit -m "feat(web): add StickyStockHeader component with signal badge"
```

---

### Task 1.2: Create AnalysisSummary Hero Component

**Files:**
- Create: `apps/web/src/lib/components/AnalysisSummary.svelte`

**Step 1: Create the component file**

```svelte
<script lang="ts">
  import type { OverallConclusion, ValuationEstimate, StockScore } from '$lib/api';

  let {
    score,
    valuation,
    conclusion,
    currentPrice,
  }: {
    score: StockScore | null;
    valuation: ValuationEstimate | null;
    conclusion: OverallConclusion | null;
    currentPrice: number;
  } = $props();

  let signal = $derived(() => {
    if (!score) return { label: 'ANALYZING', color: 'bg-surface-400', conviction: 0 };
    const c = Number(score.composite_score);
    if (c >= 80) return { label: 'STRONG BUY', color: 'bg-emerald-500', conviction: c };
    if (c >= 65) return { label: 'BUY', color: 'bg-emerald-400', conviction: c };
    if (c >= 50) return { label: 'HOLD', color: 'bg-amber-400', conviction: c };
    if (c >= 35) return { label: 'SELL', color: 'bg-rose-400', conviction: c };
    return { label: 'STRONG SELL', color: 'bg-rose-500', conviction: c };
  });

  let targetPrice = $derived(() => valuation?.fair_price_estimate ?? null);

  let upside = $derived(() => {
    const target = targetPrice();
    if (!target || !currentPrice || currentPrice === 0) return null;
    return ((target - currentPrice) / currentPrice) * 100;
  });

  let riskLevel = $derived(() => {
    const tech = score?.technical_score ? Number(score.technical_score) : 50;
    if (tech < 40) return 'HIGH';
    if (tech < 60) return 'MEDIUM';
    return 'LOW';
  });

  let thesis = $derived(() => {
    if (!conclusion) return 'Loading analysis...';
    const strengths = conclusion.strengths?.slice(0, 2) ?? [];
    return strengths.length > 0 ? strengths.join('. ') : 'Analysis in progress...';
  });
</script>

<div
  class="card p-6 bg-gradient-to-br from-surface-100 to-surface-200 dark:from-surface-800 dark:to-surface-900"
>
  <div class="flex flex-col md:flex-row items-start gap-6">
    <div class="flex flex-col items-center">
      <div
        class="w-24 h-24 rounded-full flex items-center justify-center {signal().color} text-white shadow-lg"
      >
        <span class="text-lg font-bold text-center leading-tight px-2">
          {signal().label}
        </span>
      </div>
      <span class="mt-2 text-sm text-surface-600-300-token">
        Conviction: {signal().conviction.toFixed(0)}%
      </span>
    </div>

    <div class="flex-1 grid grid-cols-2 gap-4">
      <div>
        <span class="text-sm text-surface-500-400-token">Target Price</span>
        <div class="text-2xl font-bold text-success-600-300-token">
          {targetPrice()?.toLocaleString('id-ID') ?? '-'}
        </div>
        {#if upside() !== null}
          <span
            class="text-sm {upside()! > 0 ? 'text-success-500' : 'text-error-500'}"
          >
            {upside()! > 0 ? '+' : ''}{upside()!.toFixed(1)}% potential
          </span>
        {/if}
      </div>
      <div>
        <span class="text-sm text-surface-500-400-token">Risk Level</span>
        <div
          class="text-xl font-semibold {riskLevel() === 'HIGH'
            ? 'text-error-500'
            : riskLevel() === 'MEDIUM'
              ? 'text-warning-500'
              : 'text-success-500'}"
        >
          {riskLevel()}
        </div>
        <span class="text-sm text-surface-400">
          Technical: {score?.technical_score ? Number(score.technical_score).toFixed(0) : '-'}
        </span>
      </div>
    </div>
  </div>

  <div class="mt-4 p-3 bg-surface-200/50 dark:bg-surface-700/50 rounded-lg">
    <p class="text-sm text-surface-700-200-token">{thesis()}</p>
  </div>
</div>
```

**Step 2: Verify TypeScript compiles**

Run: `cd apps/web && npm run check`
Expected: No errors related to AnalysisSummary

**Step 3: Commit**

```bash
git add apps/web/src/lib/components/AnalysisSummary.svelte
git commit -m "feat(web): add AnalysisSummary hero component with signal display"
```

---

### Task 1.3: Update StockAnalysis.types.ts with New Types

**Files:**
- Modify: `apps/web/src/lib/components/StockAnalysis.types.ts`

**Step 1: Add new type definitions**

Add these types to the existing file:

```typescript
export type TradingSignal = 'StrongBuy' | 'Buy' | 'Hold' | 'Sell' | 'StrongSell';

export interface SignalAnalysis {
  signal: TradingSignal;
  conviction_percent: number;
  thesis: string;
  target_price: number | null;
  stop_loss: number | null;
  upside_percent: number | null;
  downside_percent: number | null;
  risk_reward_ratio: number | null;
  key_catalysts: string[];
  key_risks: string[];
}

export interface SuspiciousActivity {
  detected: boolean;
  activity_type: string;
  description: string;
  severity: 'low' | 'medium' | 'high';
  brokers_involved: string[];
}

export interface CompanyProfile {
  symbol: string;
  name: string;
  description: string | null;
  business_summary: string | null;
  sector: string | null;
  subsector: string | null;
  website: string | null;
  employee_count: number | null;
}

export interface Subsidiary {
  name: string;
  ownership_percent: number;
  business_type: string | null;
  is_consolidated: boolean;
}

export interface CorporateAction {
  id: number;
  symbol: string;
  action_type: string;
  announced_date: string;
  effective_date: string | null;
  ex_date: string | null;
  description: string;
  value: number | null;
  status: string;
}

export interface NewsItem {
  id: number;
  symbol: string;
  title: string;
  summary: string | null;
  source: string;
  url: string;
  published_at: string;
  sentiment: string | null;
  keywords: string[];
}
```

**Step 2: Verify TypeScript compiles**

Run: `cd apps/web && npm run check`
Expected: No errors

**Step 3: Commit**

```bash
git add apps/web/src/lib/components/StockAnalysis.types.ts
git commit -m "feat(web): add new types for signals, business, and news"
```

---

### Task 1.4: Refactor Stock Page Layout

**Files:**
- Modify: `apps/web/src/routes/stock/[symbol]/+page.svelte`

**Step 1: Add new imports at top of script**

Add after existing imports:

```typescript
import StickyStockHeader from '$lib/components/StickyStockHeader.svelte';
import AnalysisSummary from '$lib/components/AnalysisSummary.svelte';
```

**Step 2: Add derived state for data freshness check**

Add after existing state declarations:

```typescript
let isDataFresh = $derived(() => {
  if (!freshness) return true;
  const now = new Date();
  const oneDay = 24 * 60 * 60 * 1000;
  
  const pricesStale = freshness.prices_as_of 
    ? (now.getTime() - new Date(freshness.prices_as_of).getTime()) > oneDay 
    : true;
  const brokerStale = freshness.broker_flow_as_of
    ? (now.getTime() - new Date(freshness.broker_flow_as_of).getTime()) > oneDay
    : true;
    
  return !pricesStale && !brokerStale;
});

let latestPrice = $derived(() => {
  if (!prices || prices.length === 0) return null;
  return prices[prices.length - 1];
});
```

**Step 3: Auto-load analysis on mount**

In the `onMount` function, add after existing data loads:

```typescript
// Auto-load analysis
loadAnalysis();
```

**Step 4: Replace template with new layout**

Replace the entire template section (from first HTML element to end) with:

```svelte
{#if stock}
  <StickyStockHeader
    {stock}
    latestPrice={latestPrice()}
    {score}
    {inWatchlist}
    dataFresh={isDataFresh()}
    onToggleWatchlist={toggleWatchlist}
    onRefreshData={refreshAllSources}
  />

  <main class="container mx-auto px-4 py-6 space-y-4">
    <!-- 1. Analysis Summary Hero -->
    <AnalysisSummary
      {score}
      {valuation}
      {conclusion}
      currentPrice={latestPrice()?.close ?? 0}
    />

    <!-- 2. Broker Flow Analysis -->
    <details class="card" open>
      <summary class="p-4 cursor-pointer font-bold flex items-center justify-between">
        <span>Broker Flow Analysis</span>
        <span class="badge variant-soft">
          {brokerSummary?.net_status ?? 'Loading'}
        </span>
      </summary>
      <div class="p-4 pt-0">
        {#if brokerSummary}
          <InstitutionalFlowAnalysisComponent analysis={brokerSummary.institutional_analysis} />
        {:else}
          <p class="text-surface-500">Loading broker data...</p>
        {/if}
      </div>
    </details>

    <!-- 3. Financial Analysis -->
    <details class="card">
      <summary class="p-4 cursor-pointer font-bold">Financial Analysis</summary>
      <div class="p-4 pt-0">
        {#if fundamentals}
          <FundamentalMetrics data={fundamentals} currentPrice={latestPrice()?.close ?? 0} />
        {:else}
          <p class="text-surface-500">Loading financial data...</p>
        {/if}
      </div>
    </details>

    <!-- 4. Technical Analysis -->
    <details class="card">
      <summary class="p-4 cursor-pointer font-bold">Technical Analysis</summary>
      <div class="p-4 pt-0">
        <PriceChart {prices} height={300} />
        {#if technical}
          <div class="grid grid-cols-2 md:grid-cols-4 gap-3 mt-4">
            <div class="card p-3 text-center">
              <span class="text-xs text-surface-500">RSI (14)</span>
              <div class="text-lg font-bold {technical.rsi < 30 ? 'text-success-500' : technical.rsi > 70 ? 'text-error-500' : ''}">
                {technical.rsi?.toFixed(1) ?? '-'}
              </div>
            </div>
            <div class="card p-3 text-center">
              <span class="text-xs text-surface-500">MACD</span>
              <div class="text-lg font-bold {technical.macd_signal === 'bullish' ? 'text-success-500' : technical.macd_signal === 'bearish' ? 'text-error-500' : ''}">
                {technical.macd_signal ?? '-'}
              </div>
            </div>
            <div class="card p-3 text-center">
              <span class="text-xs text-surface-500">Support</span>
              <div class="text-lg font-bold text-success-500">
                {technical.support?.[0]?.price?.toLocaleString('id-ID') ?? '-'}
              </div>
            </div>
            <div class="card p-3 text-center">
              <span class="text-xs text-surface-500">Resistance</span>
              <div class="text-lg font-bold text-error-500">
                {technical.resistance?.[0]?.price?.toLocaleString('id-ID') ?? '-'}
              </div>
            </div>
          </div>
        {/if}
      </div>
    </details>

    <!-- 5. Data Sources (Collapsed) -->
    <details class="card">
      <summary class="p-4 cursor-pointer text-sm text-surface-500 flex items-center justify-between">
        <span>Data Sources & Refresh</span>
        <span class="badge variant-soft-surface text-xs">
          {isDataFresh() ? 'Fresh' : 'Stale'}
        </span>
      </summary>
      <div class="p-4 pt-0">
        <!-- Existing data source panel content goes here -->
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          <!-- Price Source -->
          <div class="card p-3">
            <div class="flex items-center justify-between mb-2">
              <span class="font-medium text-sm">Prices</span>
              <span class="badge {freshness?.prices_as_of ? 'variant-soft-success' : 'variant-soft-warning'} text-xs">
                {freshness?.prices_as_of ? 'Fresh' : 'No Data'}
              </span>
            </div>
            <button
              class="btn btn-sm variant-ghost-primary w-full"
              onclick={() => triggerSource('price')}
              disabled={sourceLoading.price}
            >
              {sourceLoading.price ? 'Loading...' : 'Refresh'}
            </button>
          </div>
          
          <!-- Broker Source -->
          <div class="card p-3">
            <div class="flex items-center justify-between mb-2">
              <span class="font-medium text-sm">Broker Flow</span>
              <span class="badge {freshness?.broker_flow_as_of ? 'variant-soft-success' : 'variant-soft-warning'} text-xs">
                {freshness?.broker_flow_as_of ? 'Fresh' : 'No Data'}
              </span>
            </div>
            <button
              class="btn btn-sm variant-ghost-primary w-full"
              onclick={() => triggerSource('broker')}
              disabled={sourceLoading.broker}
            >
              {sourceLoading.broker ? 'Loading...' : 'Refresh'}
            </button>
          </div>
          
          <!-- Fundamental Source -->
          <div class="card p-3">
            <div class="flex items-center justify-between mb-2">
              <span class="font-medium text-sm">Fundamentals</span>
              <span class="badge {freshness?.financials_as_of ? 'variant-soft-success' : 'variant-soft-warning'} text-xs">
                {freshness?.financials_as_of ? 'Fresh' : 'No Data'}
              </span>
            </div>
            <button
              class="btn btn-sm variant-ghost-primary w-full"
              onclick={() => triggerSource('fundamental')}
              disabled={sourceLoading.fundamental}
            >
              {sourceLoading.fundamental ? 'Loading...' : 'Refresh'}
            </button>
          </div>
          
          <!-- All Sources -->
          <div class="card p-3">
            <div class="flex items-center justify-between mb-2">
              <span class="font-medium text-sm">All Sources</span>
            </div>
            <button
              class="btn btn-sm variant-filled-primary w-full"
              onclick={refreshAllSources}
            >
              Refresh All
            </button>
          </div>
        </div>
      </div>
    </details>
  </main>

  <!-- Mobile Bottom Action Bar -->
  <div class="fixed bottom-0 left-0 right-0 p-3 bg-surface-100-800-token border-t md:hidden z-40">
    <div class="flex gap-2">
      <button
        class="btn flex-1 {inWatchlist ? 'variant-filled-primary' : 'variant-ghost-primary'}"
        onclick={toggleWatchlist}
      >
        {inWatchlist ? '★ In Watchlist' : '☆ Add to Watchlist'}
      </button>
    </div>
  </div>
  <div class="h-16 md:hidden"></div>
{:else}
  <div class="container mx-auto px-4 py-8">
    <p class="text-center text-surface-500">Loading stock data...</p>
  </div>
{/if}
```

**Step 5: Verify TypeScript compiles**

Run: `cd apps/web && npm run check`
Expected: No errors

**Step 6: Commit**

```bash
git add apps/web/src/routes/stock/[symbol]/+page.svelte
git commit -m "refactor(web): replace tabs with accordion layout and sticky header"
```

---

## Phase 2: Enhanced Analysis Backend

### Task 2.1: Add Trading Signal Types to Analysis API

**Files:**
- Modify: `apps/api/src/routes/analysis.rs`

**Step 1: Add TradingSignal enum and SignalAnalysis struct**

Add after existing imports:

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TradingSignal {
    StrongBuy,
    Buy,
    Hold,
    Sell,
    StrongSell,
}

#[derive(Debug, Serialize)]
pub struct SignalAnalysis {
    pub signal: TradingSignal,
    pub conviction_percent: f64,
    pub thesis: String,
    pub target_price: Option<f64>,
    pub stop_loss: Option<f64>,
    pub upside_percent: Option<f64>,
    pub downside_percent: Option<f64>,
    pub risk_reward_ratio: Option<f64>,
    pub key_catalysts: Vec<String>,
    pub key_risks: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SuspiciousActivity {
    pub detected: bool,
    pub activity_type: String,
    pub description: String,
    pub severity: String,
    pub brokers_involved: Vec<String>,
}
```

**Step 2: Add signal field to InstitutionalFlowAnalysis**

Add to the existing `InstitutionalFlowAnalysis` struct:

```rust
pub suspicious_activity: Option<SuspiciousActivity>,
```

**Step 3: Verify Rust compiles**

Run: `cd apps/api && cargo check`
Expected: No errors (warnings OK)

**Step 4: Commit**

```bash
git add apps/api/src/routes/analysis.rs
git commit -m "feat(api): add TradingSignal and SuspiciousActivity types"
```

---

### Task 2.2: Implement Signal Calculation Logic

**Files:**
- Modify: `apps/api/src/routes/analysis.rs`

**Step 1: Add calculate_trading_signal function**

```rust
fn calculate_trading_signal(
    composite_score: f64,
    technical: &TechnicalResponse,
    valuation: &ValuationResponse,
    broker: &BrokerSummaryResponse,
    current_price: f64,
) -> SignalAnalysis {
    let signal = match composite_score {
        c if c >= 75.0 => TradingSignal::StrongBuy,
        c if c >= 60.0 => TradingSignal::Buy,
        c if c >= 45.0 => TradingSignal::Hold,
        c if c >= 30.0 => TradingSignal::Sell,
        _ => TradingSignal::StrongSell,
    };

    let target_price = valuation.fair_price_estimate;
    
    let stop_loss = technical.support.first()
        .map(|s| s.price)
        .unwrap_or_else(|| current_price * 0.95);

    let upside = target_price.map(|t| ((t - current_price) / current_price) * 100.0);
    let downside = Some(((current_price - stop_loss) / current_price) * 100.0);
    
    let risk_reward = match (upside, downside) {
        (Some(up), Some(down)) if down.abs() > 0.01 => Some(up / down.abs()),
        _ => None,
    };

    let thesis = generate_thesis(broker, technical, valuation);
    let key_catalysts = extract_catalysts(broker, technical);
    let key_risks = extract_risks(technical, valuation);

    SignalAnalysis {
        signal,
        conviction_percent: composite_score,
        thesis,
        target_price,
        stop_loss: Some(stop_loss),
        upside_percent: upside,
        downside_percent: downside,
        risk_reward_ratio: risk_reward,
        key_catalysts,
        key_risks,
    }
}

fn generate_thesis(
    broker: &BrokerSummaryResponse,
    technical: &TechnicalResponse,
    valuation: &ValuationResponse,
) -> String {
    let mut parts = Vec::new();

    if let Some(inst) = &broker.institutional_analysis {
        if inst.is_accumulating && inst.coordinated_buying {
            parts.push("Strong institutional accumulation detected".to_string());
        } else if inst.is_accumulating {
            parts.push("Institutional accumulation ongoing".to_string());
        }
    }

    if technical.rsi < 30.0 {
        parts.push("oversold on RSI".to_string());
    } else if technical.rsi > 70.0 {
        parts.push("overbought on RSI".to_string());
    }

    if let Some(margin) = valuation.margin_of_safety {
        if margin > 20.0 {
            parts.push(format!("{:.0}% margin of safety", margin));
        }
    }

    if parts.is_empty() {
        "Mixed signals - wait for clearer setup".to_string()
    } else {
        parts.join(" with ")
    }
}

fn extract_catalysts(broker: &BrokerSummaryResponse, technical: &TechnicalResponse) -> Vec<String> {
    let mut catalysts = Vec::new();
    
    if let Some(inst) = &broker.institutional_analysis {
        if inst.coordinated_buying {
            catalysts.push("Multiple institutions accumulating".to_string());
        }
        if inst.foreign_net_5_day > 0.0 {
            catalysts.push("Foreign buying interest".to_string());
        }
    }
    
    if technical.macd_signal == "bullish" {
        catalysts.push("MACD bullish crossover".to_string());
    }
    
    catalysts
}

fn extract_risks(technical: &TechnicalResponse, valuation: &ValuationResponse) -> Vec<String> {
    let mut risks = Vec::new();
    
    if technical.rsi > 70.0 {
        risks.push("Overbought conditions".to_string());
    }
    
    if let Some(pe) = valuation.per_analysis.as_ref() {
        if pe.contains("expensive") || pe.contains("premium") {
            risks.push("Valuation stretched".to_string());
        }
    }
    
    risks
}
```

**Step 2: Verify Rust compiles**

Run: `cd apps/api && cargo check`
Expected: No errors (warnings OK)

**Step 3: Commit**

```bash
git add apps/api/src/routes/analysis.rs
git commit -m "feat(api): implement trading signal calculation with thesis generation"
```

---

### Task 2.3: Add Suspicious Activity Detection

**Files:**
- Modify: `apps/api/src/routes/analysis.rs`

**Step 1: Add detect_suspicious_activity function**

```rust
fn detect_suspicious_activity(
    big_buyers: &[BrokerInfo],
    big_sellers: &[BrokerInfo],
    total_volume: i64,
    avg_daily_volume: i64,
) -> Option<SuspiciousActivity> {
    let buy_codes: std::collections::HashSet<_> = big_buyers.iter()
        .map(|b| b.broker_code.as_str())
        .collect();
    let sell_codes: std::collections::HashSet<_> = big_sellers.iter()
        .map(|b| b.broker_code.as_str())
        .collect();
    
    let both_sides: Vec<_> = buy_codes.intersection(&sell_codes).collect();
    
    if !both_sides.is_empty() {
        return Some(SuspiciousActivity {
            detected: true,
            activity_type: "wash_trading_signal".to_string(),
            description: format!(
                "Broker(s) {} appear on both buy and sell sides - possible wash trading",
                both_sides.iter().copied().collect::<Vec<_>>().join(", ")
            ),
            severity: "medium".to_string(),
            brokers_involved: both_sides.iter().map(|s| s.to_string()).collect(),
        });
    }

    if avg_daily_volume > 0 && total_volume > avg_daily_volume * 3 {
        return Some(SuspiciousActivity {
            detected: true,
            activity_type: "unusual_volume".to_string(),
            description: format!(
                "Volume {}x above average - unusual activity",
                total_volume / avg_daily_volume.max(1)
            ),
            severity: "low".to_string(),
            brokers_involved: vec![],
        });
    }

    None
}
```

**Step 2: Integrate into broker summary calculation**

In the function that builds `InstitutionalFlowAnalysis`, add:

```rust
let suspicious = detect_suspicious_activity(
    &big_buyers,
    &big_sellers,
    total_volume,
    avg_daily_volume,
);
// Add to struct: suspicious_activity: suspicious,
```

**Step 3: Verify Rust compiles**

Run: `cd apps/api && cargo check`
Expected: No errors (warnings OK)

**Step 4: Commit**

```bash
git add apps/api/src/routes/analysis.rs
git commit -m "feat(api): add suspicious activity detection for wash trading and volume spikes"
```

---

## Phase 3: Business Context

### Task 3.1: Create Database Migration for Business Tables

**Files:**
- Create: `crates/db/migrations/004_add_business_tables.sql`

**Step 1: Create the migration file**

```sql
-- Business context tables for company information, subsidiaries, and corporate actions

CREATE TABLE IF NOT EXISTS company_profiles (
    symbol TEXT PRIMARY KEY REFERENCES stocks(symbol) ON DELETE CASCADE,
    description TEXT,
    business_summary TEXT,
    website TEXT,
    employee_count INTEGER,
    headquarters TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS subsidiaries (
    id SERIAL PRIMARY KEY,
    parent_symbol TEXT NOT NULL REFERENCES stocks(symbol) ON DELETE CASCADE,
    name TEXT NOT NULL,
    ownership_percent DECIMAL(5,2),
    business_type TEXT,
    is_consolidated BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS corporate_actions (
    id SERIAL PRIMARY KEY,
    symbol TEXT NOT NULL REFERENCES stocks(symbol) ON DELETE CASCADE,
    action_type TEXT NOT NULL,
    announced_date DATE NOT NULL,
    effective_date DATE,
    ex_date DATE,
    description TEXT NOT NULL,
    value DECIMAL(20,4),
    status TEXT DEFAULT 'announced',
    source_url TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS stock_news (
    id SERIAL PRIMARY KEY,
    symbol TEXT NOT NULL REFERENCES stocks(symbol) ON DELETE CASCADE,
    title TEXT NOT NULL,
    summary TEXT,
    source TEXT NOT NULL,
    url TEXT NOT NULL,
    published_at TIMESTAMPTZ NOT NULL,
    sentiment TEXT,
    keywords TEXT[],
    related_corporate_action_id INTEGER REFERENCES corporate_actions(id),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_subsidiaries_parent ON subsidiaries(parent_symbol);
CREATE INDEX IF NOT EXISTS idx_corporate_actions_symbol ON corporate_actions(symbol);
CREATE INDEX IF NOT EXISTS idx_corporate_actions_date ON corporate_actions(effective_date);
CREATE INDEX IF NOT EXISTS idx_stock_news_symbol ON stock_news(symbol);
CREATE INDEX IF NOT EXISTS idx_stock_news_published ON stock_news(published_at);
```

**Step 2: Commit**

```bash
git add crates/db/migrations/004_add_business_tables.sql
git commit -m "feat(db): add business tables for company profiles, subsidiaries, corporate actions, news"
```

---

### Task 3.2: Create BusinessAnalysis Component

**Files:**
- Create: `apps/web/src/lib/components/BusinessAnalysis.svelte`

**Step 1: Create the component**

```svelte
<script lang="ts">
  import type { CompanyProfile, Subsidiary, CorporateAction, NewsItem } from './StockAnalysis.types';

  let {
    profile,
    subsidiaries = [],
    corporateActions = [],
    news = [],
  }: {
    profile: CompanyProfile | null;
    subsidiaries: Subsidiary[];
    corporateActions: CorporateAction[];
    news: NewsItem[];
  } = $props();

  type TimelineItem = 
    | { type: 'action'; date: Date; data: CorporateAction }
    | { type: 'news'; date: Date; data: NewsItem };

  let timeline = $derived(() => {
    const items: TimelineItem[] = [
      ...corporateActions.map((ca) => ({
        type: 'action' as const,
        date: new Date(ca.effective_date ?? ca.announced_date),
        data: ca,
      })),
      ...news.map((n) => ({
        type: 'news' as const,
        date: new Date(n.published_at),
        data: n,
      })),
    ];
    return items.sort((a, b) => b.date.getTime() - a.date.getTime());
  });

  function formatDate(date: Date): string {
    return date.toLocaleDateString('id-ID', {
      day: 'numeric',
      month: 'short',
      year: 'numeric',
    });
  }

  function getActionColor(type: string): string {
    const colors: Record<string, string> = {
      dividend: 'variant-filled-success',
      stock_split: 'variant-filled-primary',
      rights_issue: 'variant-filled-warning',
      acquisition: 'variant-filled-secondary',
    };
    return colors[type] ?? 'variant-filled-surface';
  }
</script>

<div class="space-y-6">
  {#if profile}
    <div class="card p-4">
      <h3 class="h4 mb-3">About {profile.name}</h3>
      <p class="text-sm text-surface-600-300-token">
        {profile.business_summary ?? profile.description ?? 'No description available'}
      </p>

      {#if profile.website}
        <a
          href={profile.website}
          target="_blank"
          rel="noopener noreferrer"
          class="text-sm text-primary-500 hover:underline mt-2 inline-block"
        >
          Visit Website
        </a>
      {/if}
    </div>
  {/if}

  {#if subsidiaries.length > 0}
    <div class="card p-4">
      <h3 class="h4 mb-3">Business Units & Subsidiaries</h3>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
        {#each subsidiaries as sub}
          <div class="p-3 bg-surface-100-800-token rounded-lg">
            <div class="flex justify-between items-start">
              <span class="font-medium">{sub.name}</span>
              <span class="badge variant-soft-primary">{sub.ownership_percent}%</span>
            </div>
            {#if sub.business_type}
              <span class="text-xs text-surface-500">{sub.business_type}</span>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  {/if}

  {#if timeline().length > 0}
    <div class="card p-4">
      <h3 class="h4 mb-3">Timeline & News</h3>
      <div class="relative border-l-2 border-surface-300-600-token pl-4 space-y-4">
        {#each timeline() as item}
          <div class="relative">
            <div
              class="absolute -left-[1.4rem] w-3 h-3 rounded-full {item.type === 'action'
                ? 'bg-primary-500'
                : 'bg-surface-400'}"
            ></div>

            {#if item.type === 'action'}
              {@const action = item.data}
              <div
                class="p-3 bg-primary-50 dark:bg-primary-900/20 rounded-lg border-l-4 border-primary-500"
              >
                <div class="flex justify-between items-start">
                  <div>
                    <span class="badge {getActionColor(action.action_type)} text-xs">
                      {action.action_type}
                    </span>
                    <h4 class="font-medium mt-1">{action.description}</h4>
                  </div>
                  <span class="text-xs text-surface-500">{formatDate(item.date)}</span>
                </div>
                {#if action.value}
                  <span class="text-sm font-mono text-primary-700 dark:text-primary-300">
                    Rp {action.value.toLocaleString('id-ID')}
                  </span>
                {/if}
              </div>
            {:else}
              {@const n = item.data}
              <div class="p-3 bg-surface-100-800-token rounded-lg">
                <div class="flex justify-between items-start gap-2">
                  <a
                    href={n.url}
                    target="_blank"
                    rel="noopener noreferrer"
                    class="font-medium hover:text-primary-500 hover:underline"
                  >
                    {n.title}
                  </a>
                  <span class="text-xs text-surface-500 whitespace-nowrap">
                    {formatDate(item.date)}
                  </span>
                </div>
                <div class="flex items-center gap-2 mt-1">
                  <span class="text-xs text-surface-400">{n.source}</span>
                  {#each n.keywords as kw}
                    <span class="badge variant-soft text-xs">{kw}</span>
                  {/each}
                </div>
              </div>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  {:else}
    <div class="card p-4">
      <p class="text-sm text-surface-500 text-center">No business updates available</p>
    </div>
  {/if}
</div>
```

**Step 2: Verify TypeScript compiles**

Run: `cd apps/web && npm run check`
Expected: No errors

**Step 3: Commit**

```bash
git add apps/web/src/lib/components/BusinessAnalysis.svelte
git commit -m "feat(web): add BusinessAnalysis component with timeline view"
```

---

### Task 3.3: Create Corporate Actions Scraper

**Files:**
- Create: `apps/ml/src/jejakcuan_ml/scrapers/corporate_actions.py`

**Step 1: Create the scraper file**

```python
"""Corporate actions scraper for IDX stocks."""

from dataclasses import dataclass
from datetime import date
from typing import Optional

from loguru import logger

from .base import BaseScraper, ScraperConfig
from ..db import DatabaseClient


@dataclass
class CorporateActionData:
    """Corporate action data."""

    symbol: str
    action_type: str
    announced_date: date
    effective_date: Optional[date]
    ex_date: Optional[date]
    description: str
    value: Optional[float]
    status: str


class CorporateActionsScraper(BaseScraper):
    """Scraper for corporate actions from IDX."""

    IDX_CORPORATE_ACTIONS = "https://www.idx.co.id/primary/ListedCompany/GetCorporateAction"

    def __init__(
        self,
        config: ScraperConfig | None = None,
        db_client: DatabaseClient | None = None,
        symbols: list[str] | None = None,
    ) -> None:
        super().__init__(config, db_client)
        self._symbols = symbols

    def get_name(self) -> str:
        return "Corporate Actions"

    async def scrape(self) -> int:
        """Scrape corporate actions."""
        count = 0

        if self._symbols:
            symbols = self._symbols
        else:
            symbols = self.db.get_all_symbols()

        logger.info(f"Scraping corporate actions for {len(symbols)} stocks")

        for symbol in symbols:
            try:
                actions = await self.fetch_corporate_actions(symbol)
                for action in actions:
                    self._save_action(action)
                    count += 1
            except Exception as e:
                logger.warning(f"Failed to scrape corporate actions for {symbol}: {e}")

        return count

    async def fetch_corporate_actions(
        self,
        symbol: str | None = None,
        from_date: date | None = None,
    ) -> list[CorporateActionData]:
        """Fetch corporate actions for a stock."""
        actions: list[CorporateActionData] = []

        params: dict[str, str | int] = {"page": 1, "pageSize": 50}
        if symbol:
            params["code"] = symbol
        if from_date:
            params["fromDate"] = from_date.isoformat()

        response = await self._fetch_url(self.IDX_CORPORATE_ACTIONS, params=params)
        if response:
            try:
                data = response.json()
                for item in data.get("Results", []):
                    action = self._parse_action(item)
                    if action:
                        actions.append(action)
            except Exception as e:
                logger.debug(f"Failed to parse corporate actions: {e}")

        return actions

    def _parse_action(self, item: dict) -> CorporateActionData | None:
        """Parse a corporate action from IDX API response."""
        symbol = item.get("Code", "")
        if not symbol:
            return None

        return CorporateActionData(
            symbol=symbol,
            action_type=self._normalize_action_type(item.get("Type", "")),
            announced_date=self._parse_date(item.get("AnnouncedDate")) or date.today(),
            effective_date=self._parse_date(item.get("EffectiveDate")),
            ex_date=self._parse_date(item.get("ExDate")),
            description=item.get("Description", ""),
            value=item.get("Value"),
            status=item.get("Status", "announced"),
        )

    def _normalize_action_type(self, raw_type: str) -> str:
        """Normalize corporate action type."""
        mapping = {
            "Cash Dividend": "dividend",
            "Stock Dividend": "stock_dividend",
            "Stock Split": "stock_split",
            "Rights Issue": "rights_issue",
            "Bonus Shares": "bonus_shares",
        }
        return mapping.get(raw_type, raw_type.lower().replace(" ", "_"))

    def _save_action(self, action: CorporateActionData) -> None:
        """Save corporate action to database."""
        self.db.execute(
            """
            INSERT INTO corporate_actions 
            (symbol, action_type, announced_date, effective_date, ex_date, description, value, status)
            VALUES (%s, %s, %s, %s, %s, %s, %s, %s)
            ON CONFLICT DO NOTHING
            """,
            (
                action.symbol,
                action.action_type,
                action.announced_date,
                action.effective_date,
                action.ex_date,
                action.description,
                action.value,
                action.status,
            ),
        )
```

**Step 2: Commit**

```bash
git add apps/ml/src/jejakcuan_ml/scrapers/corporate_actions.py
git commit -m "feat(ml): add corporate actions scraper"
```

---

## Phase 4: News Integration

### Task 4.1: Create News Scraper with Browser Automation

**Files:**
- Create: `apps/ml/src/jejakcuan_ml/scrapers/news_scraper.py`

**Step 1: Create the scraper file**

```python
"""News scraper for Indonesian financial news with optional browser automation."""

from dataclasses import dataclass
from datetime import datetime
from typing import Optional

from bs4 import BeautifulSoup
from loguru import logger

from .base import BaseScraper, ScraperConfig
from ..db import DatabaseClient


@dataclass
class NewsItem:
    """News item data."""

    symbol: str
    title: str
    summary: Optional[str]
    source: str
    url: str
    published_at: datetime
    keywords: list[str]


class NewsScraper(BaseScraper):
    """Scraper for stock news from Indonesian financial news sites."""

    SOURCES = {
        "kontan": "https://investasi.kontan.co.id",
        "bisnis": "https://market.bisnis.com",
    }

    KEYWORD_PATTERNS = [
        ("acquisition", ["akuisisi", "acquire", "acquisition"]),
        ("dividend", ["dividen", "dividend"]),
        ("rights_issue", ["right issue", "rights issue", "HMETD"]),
        ("earnings", ["laba", "profit", "earnings", "rugi", "loss"]),
        ("expansion", ["ekspansi", "expansion", "investasi"]),
        ("debt", ["utang", "debt", "obligasi", "bond"]),
    ]

    def __init__(
        self,
        config: ScraperConfig | None = None,
        db_client: DatabaseClient | None = None,
        symbols: list[str] | None = None,
        use_browser: bool = False,
    ) -> None:
        super().__init__(config, db_client)
        self._symbols = symbols
        self._use_browser = use_browser

    def get_name(self) -> str:
        return "News"

    async def scrape(self) -> int:
        """Scrape news for all symbols."""
        count = 0

        if self._symbols:
            symbols = self._symbols
        else:
            symbols = self.db.get_all_symbols()[:20]

        logger.info(f"Scraping news for {len(symbols)} stocks")

        for symbol in symbols:
            try:
                news_items = await self.fetch_news_for_stock(symbol)
                for item in news_items:
                    self._save_news(item)
                    count += 1
            except Exception as e:
                logger.warning(f"Failed to scrape news for {symbol}: {e}")

        return count

    async def fetch_news_for_stock(self, symbol: str) -> list[NewsItem]:
        """Fetch news for a specific stock."""
        news: list[NewsItem] = []

        if self._use_browser:
            news.extend(await self._fetch_with_browser(symbol))
        else:
            news.extend(await self._fetch_kontan(symbol))
            news.extend(await self._fetch_bisnis(symbol))

        return sorted(news, key=lambda n: n.published_at, reverse=True)[:10]

    async def _fetch_kontan(self, symbol: str) -> list[NewsItem]:
        """Fetch news from Kontan."""
        news: list[NewsItem] = []
        url = f"{self.SOURCES['kontan']}/search/?q={symbol}"

        response = await self._fetch_url(url)
        if not response:
            return news

        try:
            soup = BeautifulSoup(response.text, "html.parser")
            articles = soup.select(".list-news .news-item, .list-berita article")[:5]

            for article in articles:
                title_el = article.select_one("h3 a, .title a")
                date_el = article.select_one(".date, .time")

                if title_el:
                    title = title_el.get_text(strip=True)
                    href = title_el.get("href", "")
                    pub_date = self._parse_indo_date(
                        date_el.get_text(strip=True) if date_el else None
                    )

                    news.append(
                        NewsItem(
                            symbol=symbol,
                            title=title,
                            summary=None,
                            source="kontan",
                            url=href if href.startswith("http") else f"https://kontan.co.id{href}",
                            published_at=pub_date,
                            keywords=self._extract_keywords(title),
                        )
                    )
        except Exception as e:
            logger.debug(f"Failed to parse Kontan news for {symbol}: {e}")

        return news

    async def _fetch_bisnis(self, symbol: str) -> list[NewsItem]:
        """Fetch news from Bisnis Indonesia."""
        news: list[NewsItem] = []
        url = f"{self.SOURCES['bisnis']}/search?q={symbol}"

        response = await self._fetch_url(url)
        if not response:
            return news

        try:
            soup = BeautifulSoup(response.text, "html.parser")
            articles = soup.select(".list-news article, .search-result-item")[:5]

            for article in articles:
                title_el = article.select_one("h2 a, .title a")
                date_el = article.select_one(".date, time")

                if title_el:
                    title = title_el.get_text(strip=True)
                    href = title_el.get("href", "")
                    pub_date = self._parse_indo_date(
                        date_el.get_text(strip=True) if date_el else None
                    )

                    news.append(
                        NewsItem(
                            symbol=symbol,
                            title=title,
                            summary=None,
                            source="bisnis",
                            url=href if href.startswith("http") else f"https://bisnis.com{href}",
                            published_at=pub_date,
                            keywords=self._extract_keywords(title),
                        )
                    )
        except Exception as e:
            logger.debug(f"Failed to parse Bisnis news for {symbol}: {e}")

        return news

    async def _fetch_with_browser(self, symbol: str) -> list[NewsItem]:
        """Fetch news using Playwright browser automation."""
        news: list[NewsItem] = []

        try:
            from playwright.async_api import async_playwright

            async with async_playwright() as p:
                browser = await p.chromium.launch(headless=True)
                page = await browser.new_page()

                search_url = f"https://investasi.kontan.co.id/search/?q={symbol}"
                await page.goto(search_url)

                try:
                    await page.wait_for_selector(".list-news, .list-berita", timeout=10000)
                except Exception:
                    logger.debug(f"Timeout waiting for news list for {symbol}")
                    await browser.close()
                    return news

                articles = await page.query_selector_all(".list-news .news-item, .list-berita article")

                for article in articles[:5]:
                    title_el = await article.query_selector("h3 a, .title a")
                    date_el = await article.query_selector(".date, .time")

                    if title_el:
                        title = await title_el.inner_text()
                        url = await title_el.get_attribute("href") or ""
                        pub_date_str = await date_el.inner_text() if date_el else None

                        news.append(
                            NewsItem(
                                symbol=symbol,
                                title=title,
                                summary=None,
                                source="kontan",
                                url=url if url.startswith("http") else f"https://kontan.co.id{url}",
                                published_at=self._parse_indo_date(pub_date_str),
                                keywords=self._extract_keywords(title),
                            )
                        )

                await browser.close()

        except ImportError:
            logger.warning("Playwright not installed. Run: pip install playwright && playwright install")
        except Exception as e:
            logger.warning(f"Browser automation failed for {symbol}: {e}")

        return news

    def _extract_keywords(self, text: str) -> list[str]:
        """Extract relevant keywords from news title."""
        keywords = []
        text_lower = text.lower()

        for keyword, triggers in self.KEYWORD_PATTERNS:
            if any(t.lower() in text_lower for t in triggers):
                keywords.append(keyword)

        return keywords

    def _parse_indo_date(self, date_str: str | None) -> datetime:
        """Parse Indonesian date string."""
        if not date_str:
            return datetime.now()

        try:
            import re
            from datetime import timedelta

            date_str = date_str.strip().lower()

            if "hari" in date_str or "jam" in date_str or "menit" in date_str:
                return datetime.now()

            months = {
                "januari": 1, "februari": 2, "maret": 3, "april": 4,
                "mei": 5, "juni": 6, "juli": 7, "agustus": 8,
                "september": 9, "oktober": 10, "november": 11, "desember": 12,
                "jan": 1, "feb": 2, "mar": 3, "apr": 4, "may": 5, "jun": 6,
                "jul": 7, "aug": 8, "sep": 9, "oct": 10, "nov": 11, "dec": 12,
            }

            for month_name, month_num in months.items():
                if month_name in date_str:
                    match = re.search(r"(\d{1,2})\s*" + month_name + r"\s*(\d{4})?", date_str)
                    if match:
                        day = int(match.group(1))
                        year = int(match.group(2)) if match.group(2) else datetime.now().year
                        return datetime(year, month_num, day)

        except Exception:
            pass

        return datetime.now()

    def _save_news(self, item: NewsItem) -> None:
        """Save news item to database."""
        self.db.execute(
            """
            INSERT INTO stock_news 
            (symbol, title, summary, source, url, published_at, keywords)
            VALUES (%s, %s, %s, %s, %s, %s, %s)
            ON CONFLICT DO NOTHING
            """,
            (
                item.symbol,
                item.title,
                item.summary,
                item.source,
                item.url,
                item.published_at,
                item.keywords,
            ),
        )
```

**Step 2: Commit**

```bash
git add apps/ml/src/jejakcuan_ml/scrapers/news_scraper.py
git commit -m "feat(ml): add news scraper with optional Playwright browser automation"
```

---

## Phase 5: Technical Chart Enhancements

### Task 5.1: Create Interactive TechnicalChart Component

**Files:**
- Create: `apps/web/src/lib/components/TechnicalChart.svelte`

**Step 1: Create the component**

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import { createChart, type IChartApi, type ISeriesApi } from 'lightweight-charts';
  import type { StockPrice, TechnicalAnalysis } from '$lib/api';

  let {
    prices,
    technical,
    height = 400,
  }: {
    prices: StockPrice[];
    technical: TechnicalAnalysis | null;
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
        background: { type: 'solid', color: 'transparent' },
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

    candleSeries.setData(candleData as any);

    if (showVolume) {
      const volumeData = prices.map((p) => ({
        time: p.time.split('T')[0],
        value: p.volume,
        color: Number(p.close) >= Number(p.open) 
          ? 'rgba(16, 185, 129, 0.5)' 
          : 'rgba(239, 68, 68, 0.5)',
      }));
      volumeSeries.setData(volumeData as any);
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
          {technical.support?.[0]?.price?.toLocaleString('id-ID') ?? '-'}
        </div>
        <span class="text-xs text-surface-400">{technical.support?.[0]?.strength ?? ''}</span>
      </div>

      <div class="card p-3 text-center">
        <span class="text-xs text-surface-500">Resistance</span>
        <div class="text-lg font-bold text-error-500">
          {technical.resistance?.[0]?.price?.toLocaleString('id-ID') ?? '-'}
        </div>
        <span class="text-xs text-surface-400">{technical.resistance?.[0]?.strength ?? ''}</span>
      </div>
    </div>
  {/if}
</div>
```

**Step 2: Verify TypeScript compiles**

Run: `cd apps/web && npm run check`
Expected: No errors

**Step 3: Commit**

```bash
git add apps/web/src/lib/components/TechnicalChart.svelte
git commit -m "feat(web): add interactive TechnicalChart with timeframe switching"
```

---

## Final Verification

### Task: Run Full Build and Type Check

**Step 1: Check Rust API**

Run: `cd apps/api && cargo check`
Expected: Compiles with no errors (warnings OK)

**Step 2: Check TypeScript Frontend**

Run: `cd apps/web && npm run check`
Expected: 0 errors, 0 warnings

**Step 3: Run Linting**

Run: `cd apps/web && npm run lint`
Expected: No errors

**Step 4: Final Commit**

```bash
git add -A
git status
# If any uncommitted changes, commit them
git commit -m "chore: final cleanup for stock page redesign"
```

---

## Success Criteria Checklist

- [ ] User can see signal (BUY/HOLD/SELL) without scrolling
- [ ] Sticky header shows price + change + signal badge
- [ ] Analysis Summary hero displays target price and conviction
- [ ] Broker Flow section shows institutional activity
- [ ] Suspicious activity is flagged when detected
- [ ] Financial metrics are visible in accordion
- [ ] Technical chart has timeframe switching
- [ ] Data Source panel is collapsed at bottom
- [ ] Mobile has fixed bottom action bar
- [ ] All TypeScript compiles without errors
- [ ] All Rust compiles without errors
