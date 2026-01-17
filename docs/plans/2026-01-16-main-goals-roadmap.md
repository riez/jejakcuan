# Main Goals Gap Closure Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task.

**Goal:** Close the remaining gaps from `main-goals.md` by wiring real data ingestion + integrating orderflow/orderbook analytics into scoring and API surfaces.

**Architecture:** JejakCuan is a monorepo: Rust API (`apps/api`) reads TimescaleDB (`crates/db` + SQL migrations) and exposes endpoints consumed by SvelteKit web (`apps/web`). Python (`apps/ml`) provides (a) optional ML FastAPI service and (b) scrapers CLI that can populate the DB. Technical indicators live in Rust (`crates/technical`), and scoring lives in Rust (`crates/core`).

**Tech Stack:** Rust 2021 + Axum + SQLx + TimescaleDB/Postgres + Redis; SvelteKit (Svelte 5) + SkeletonUI + Vite; Python 3.12 + FastAPI + optional Torch/Transformers + psycopg2 scrapers.

---

## 0) Current State Map (What Exists Today)

### Data + Storage
- TimescaleDB schema exists with `stock_prices`, `broker_summary`, `financials`, `shareholdings`, `sentiment_data`, `ml_predictions`, and `stock_scores` in `crates/db/migrations/001_initial_schema.sql`.
- DB repositories exist for inserts/reads:
  - Prices: `crates/db/src/repositories/prices.rs`
  - Broker flow aggregates: `crates/db/src/repositories/broker_summary.rs`
  - Scores: `crates/db/src/repositories/scores.rs`

### Analytics
- Technical indicator implementations (Rust): `crates/technical/src/*`.
  - Orderflow functions already implemented + unit-tested: `crates/technical/src/orderflow.rs` (OBI/OFI/VAMP + helpers like volume split).
  - Wyckoff detection exists: `crates/technical/src/wyckoff.rs`.
- Technical score engine supports optional OBI/OFI inputs: `crates/core/src/technical_score.rs`.
- API score computation currently leaves OBI/OFI unset (`None`), so orderflow component stays neutral: `apps/api/src/routes/stocks.rs`.

### API Surface
- Axum API is wired with stock + analysis endpoints:
  - Routing: `apps/api/src/lib.rs`
  - Technical analysis endpoint (RSI/MACD/Bollinger/Ichimoku): `apps/api/src/routes/analysis.rs`
  - Broker flow endpoint: `apps/api/src/routes/analysis.rs`
  - Score recompute: `apps/api/src/routes/stocks.rs`
- SSE streaming exists but is placeholder-only heartbeats (not connected to broadcasters): `apps/api/src/routes/streaming.rs`.

### Ingestion
- Python scrapers exist and can populate DB tables:
  - CLI: `apps/ml/src/jejakcuan_ml/scrapers/cli.py` (published as `jejakcuan-scrape` in `apps/ml/pyproject.toml`).
  - Price history via Yahoo Finance: `apps/ml/src/jejakcuan_ml/scrapers/price_history.py`.
  - Broker flow via Indopremier/Stockbit/IDX fallbacks: `apps/ml/src/jejakcuan_ml/scrapers/broker_flow.py`.
  - Fundamentals via IDX scraping: `apps/ml/src/jejakcuan_ml/scrapers/idx.py`.
- Rust data-source adapters exist but are not wired into any running worker:
  - TwelveData WebSocket client: `crates/data-sources/src/twelvedata/websocket.rs`.
  - Broker summary scraper: `crates/data-sources/src/broker/scraper.rs`.
  - Shareholding scraper exists but OJK is stubbed and KSEI likely requires auth: `crates/data-sources/src/shareholding/scraper.rs`.

### Web
- Web client is wired to Rust API endpoints via `apps/web/src/lib/api.ts`.
- Some pages still use mock data (Market overview): `apps/web/src/routes/market/+page.svelte`.

---

## 1) Hard Reality Check: “Orderbook” Data Requirement

`main-goals.md` describes OBI/OFI on *order book depth* (L2). Today, the repo has **calculations** but not a **trusted depth data source** or a DB schema for storing depth snapshots.

**Decision required before true orderbook analytics can be implemented:**
1) **Paid provider** (IDX Denodo / broker feed / vendor) with L2 sizes + incremental updates.
2) **Scrape-based** (higher fragility) source that exposes depth.
3) **MVP proxy**: compute “orderflow pressure” from OHLCV (no L2) and treat L2 orderbook as a later milestone.

**Recommendation:** implement MVP orderflow proxy now (fully doable with existing `stock_prices`), while designing interfaces + DB schema for future true orderbook.

---

# Milestone A — Make “Order Flow” Real (No L2 Required)

### Task 1: Add OHLCV-derived order-imbalance proxy

**Files:**
- Modify: `crates/technical/src/orderflow.rs`
- Test: `crates/technical/src/orderflow.rs`

**Step 1: Write failing test**

Add tests demonstrating imbalance direction:

```rust
#[test]
fn test_ohlc_imbalance_proxy() {
    // Close near high => buy pressure
    let obi = calculate_ohlc_imbalance_proxy(dec!(110), dec!(100), dec!(109), 1000);
    assert!(obi > dec!(0.5));

    // Close near low => sell pressure
    let obi = calculate_ohlc_imbalance_proxy(dec!(110), dec!(100), dec!(101), 1000);
    assert!(obi < dec!(-0.5));

    // No range => neutral
    let obi = calculate_ohlc_imbalance_proxy(dec!(100), dec!(100), dec!(100), 1000);
    assert_eq!(obi, Decimal::ZERO);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p jejakcuan-technical test_ohlc_imbalance_proxy`

Expected: FAIL with “cannot find function `calculate_ohlc_imbalance_proxy`”.

**Step 3: Write minimal implementation**

Implement in `crates/technical/src/orderflow.rs` using existing `split_volume`:
- Compute `(buy_vol - sell_vol) / (buy_vol + sell_vol)` as Decimal in [-1, 1]
- Return 0 when denom is 0

**Step 4: Run test to verify it passes**

Run: `cargo test -p jejakcuan-technical test_ohlc_imbalance_proxy`

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/technical/src/orderflow.rs
git commit -m "feat(technical): add OHLCV orderflow imbalance proxy"
```

---

### Task 2: Add trend function for proxy OFI (time-series slope)

**Files:**
- Modify: `crates/technical/src/orderflow.rs`
- Test: `crates/technical/src/orderflow.rs`

**Step 1: Write failing test**

```rust
#[test]
fn test_orderflow_trend_proxy() {
    let series = vec![dec!(-0.2), dec!(0.0), dec!(0.3)];
    let trend = calculate_trend_normalized(&series);
    assert!(trend > Decimal::ZERO);
}
```

**Step 2: Run test**

Run: `cargo test -p jejakcuan-technical test_orderflow_trend_proxy`

Expected: FAIL.

**Step 3: Implement minimal `calculate_trend_normalized`**

Implementation suggestion:
- if len < 2 => 0
- trend = last - first
- clamp to [-1, 1]

**Step 4: Run test**

Run: `cargo test -p jejakcuan-technical test_orderflow_trend_proxy`

Expected: PASS.

**Step 5: Commit**

```bash
git add crates/technical/src/orderflow.rs
git commit -m "feat(technical): add normalized orderflow trend helper"
```

---

### Task 3: Wire proxy OBI/OFI into technical scoring

**Files:**
- Modify: `apps/api/src/routes/stocks.rs`
- Modify: `crates/technical/src/orderflow.rs` (only if helpers needed)
- Test: `apps/api/tests/api_integration_test.rs` OR add a new unit-testable helper in `apps/api/src/routes/stocks.rs`

**Step 1: Write failing unit test around a pure helper**

Refactor `apps/api/src/routes/stocks.rs` to extract a pure function:

```rust
fn compute_orderflow_inputs_from_ohlcv(
    highs: &[Decimal],
    lows: &[Decimal],
    closes: &[Decimal],
    volumes: &[i64],
) -> (Option<Decimal>, Option<Decimal>) {
    // returns (obi, ofi_trend)
}
```

Test it with a small synthetic dataset and assert:
- OBI is Some
- OFI trend is Some and positive

**Step 2: Run tests to verify fail**

Run: `cargo test -p jejakcuan-api compute_orderflow_inputs_from_ohlcv`

Expected: FAIL (missing helper).

**Step 3: Implement helper + connect to `TechnicalScoreInput`**

In `compute_and_insert_score` (in `apps/api/src/routes/stocks.rs`):
- Compute `(obi, ofi_trend)` from last ~60 bars (or available)
- Set `technical_input.obi = obi` and `technical_input.ofi_trend = ofi_trend`

**Step 4: Run tests**

Run: `cargo test -p jejakcuan-api`

Expected: PASS.

**Step 5: Commit**

```bash
git add apps/api/src/routes/stocks.rs
git commit -m "feat(api): populate orderflow inputs in technical scoring"
```

---

# Milestone B — Orderbook Support (Interfaces + Schema First)

### Task 4: Add DB schema for orderbook snapshots and derived metrics

**Files:**
- Create: `crates/db/migrations/003_orderbook_schema.sql`

**Step 1: Write migration (schema-only)**

Create new tables (proposal):
- `orderbook_l1` hypertable: best bid/ask and sizes (if available)
- `orderflow_metrics` hypertable: computed OBI/OFI/VAMP + metadata

Example SQL (adjust naming to team preference):

```sql
CREATE TABLE IF NOT EXISTS orderbook_l1 (
  time TIMESTAMPTZ NOT NULL,
  symbol VARCHAR(10) NOT NULL,
  bid_price NUMERIC(18,4),
  bid_size BIGINT,
  ask_price NUMERIC(18,4),
  ask_size BIGINT,
  source VARCHAR(50) NOT NULL,
  CONSTRAINT fk_orderbook_l1_symbol FOREIGN KEY (symbol) REFERENCES stocks(symbol)
);
SELECT create_hypertable('orderbook_l1', 'time', if_not_exists => TRUE);
CREATE INDEX idx_orderbook_l1_symbol_time ON orderbook_l1(symbol, time DESC);

CREATE TABLE IF NOT EXISTS orderflow_metrics (
  time TIMESTAMPTZ NOT NULL,
  symbol VARCHAR(10) NOT NULL,
  obi NUMERIC(10,6),
  ofi NUMERIC(20,6),
  ofi_cumulative NUMERIC(20,6),
  vamp NUMERIC(18,6),
  source VARCHAR(50) NOT NULL,
  CONSTRAINT fk_orderflow_metrics_symbol FOREIGN KEY (symbol) REFERENCES stocks(symbol)
);
SELECT create_hypertable('orderflow_metrics', 'time', if_not_exists => TRUE);
CREATE INDEX idx_orderflow_metrics_symbol_time ON orderflow_metrics(symbol, time DESC);
```

**Step 2: Manual apply (dev)**

Run: `psql -h localhost -U jejakcuan -d jejakcuan -f crates/db/migrations/003_orderbook_schema.sql`

Expected: Tables created.

**Step 3: Commit**

```bash
git add crates/db/migrations/003_orderbook_schema.sql
git commit -m "feat(db): add orderbook + orderflow hypertables"
```

---

### Task 5: Define an orderbook provider interface (Rust)

**Files:**
- Create: `crates/data-sources/src/orderbook/mod.rs`
- Modify: `crates/data-sources/src/lib.rs`
- Test: `crates/data-sources/tests/orderbook_test.rs`

**Step 1: Write failing test**

Create a minimal mock provider that returns a snapshot, assert it matches schema.

**Step 2: Implement trait + model**

Example:

```rust
pub struct L1Snapshot {
  pub time: chrono::DateTime<chrono::Utc>,
  pub symbol: String,
  pub bid_price: Option<Decimal>,
  pub bid_size: Option<i64>,
  pub ask_price: Option<Decimal>,
  pub ask_size: Option<i64>,
  pub source: String,
}

#[async_trait]
pub trait OrderBookProvider {
  async fn fetch_l1(&self, symbol: &str) -> Result<L1Snapshot, DataSourceError>;
}
```

**Step 3: Run tests**

Run: `cargo test -p jejakcuan-data-sources`

**Step 4: Commit**

```bash
git add crates/data-sources/src/orderbook crates/data-sources/src/lib.rs crates/data-sources/tests/orderbook_test.rs
git commit -m "feat(data-sources): add orderbook provider interface"
```

---

# Milestone C — Make Data Freshness Real (Ingestion)

### Task 6: Standardize ingestion path (choose Python scrapers vs Rust workers)

**Decision Task (no code):**
- Option A (fast): use Python scrapers (`jejakcuan-scrape`) as the canonical ingestion mechanism for now.
- Option B (consolidate): build Rust worker binaries around `crates/data-sources` and deprecate Python scrapers.

**Recommendation:** Option A short-term (fast), Option B medium-term (less duplication).

---

### Task 7: Document + validate DB population via Python scrapers

**Files:**
- Modify: `run.sh`
- Optional create: `docs/runbooks/ingestion.md` (only if desired)

**Step 1: Add a `run.sh` command to seed/populate data**

Plan: add `./run.sh ingest` that runs:

```bash
cd apps/ml
python3 -m venv .venv
source .venv/bin/activate
pip install -e ".[scrapers]"
jejakcuan-scrape all --days 365
```

**Step 2: Manual verification**

Run:
- `SELECT COUNT(*) FROM stock_prices;`
- `SELECT COUNT(*) FROM broker_summary;`
- `SELECT COUNT(*) FROM financials;`

**Step 3: Commit**

```bash
git add run.sh
git commit -m "chore: add ingestion helper command"
```

---

# Milestone D — Follow-up Plans (Create separate plan docs)

These are large enough that they should each get their own `docs/plans/YYYY-MM-DD-...md`:

1) **True orderbook ingestion** (L2 provider integration + OFI by level)
2) **Shareholding/OJK integration** (current OJK scraper is stubbed)
3) **Sentiment** (IndoBERT inference + ingestion into `sentiment_data`)
4) **ML integration** (Rust API calling FastAPI + storing `ml_predictions`)
5) **Real SSE streaming** (broadcast latest prices/scores/alerts)

---

## Execution Options

Plan complete and saved to `docs/plans/2026-01-16-main-goals-roadmap.md`. Two execution options:

1) **Subagent-Driven (this session)** - dispatch fresh subagent per task, review between tasks
2) **Parallel Session (separate)** - open a new session and run `superpowers:executing-plans`

Which approach do you want?