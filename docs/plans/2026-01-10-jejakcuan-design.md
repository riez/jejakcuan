# JejakCuan - Indonesian Stock Tracking System Design

**Date:** 2026-01-10  
**Status:** Approved  
**Author:** AI-assisted design session

---

## Executive Summary

JejakCuan is a comprehensive Indonesian stock tracking and analysis system combining technical analysis (Order Flow, EMA, Fibonacci), fundamental analysis (P/E, EV/EBITDA, DCF), ML predictions (LSTM), and sentiment analysis (IndoBERT) into a unified scoring dashboard for personal investment decisions.

---

## Requirements Summary

| Aspect | Decision |
|--------|----------|
| **User** | Personal investor (single-user with auth protection) |
| **Data Sources** | Paid APIs (Sectors.app ~$50-100/mo) with free fallbacks (yfinance, scraping) |
| **Deployment** | Self-hosted VPS, K3s, GitHub Actions CI/CD |
| **Stack** | Rust backend, SvelteKit frontend (Svelte 5 + Skeleton UI), Moon monorepo |
| **Analysis** | Equal weight Technical + Fundamental with transparent scoring |
| **UI Priority** | Screener-first, with Watchlist, Market Overview, and Signals views |
| **ML** | LSTM prediction as core feature from v1 |
| **Sentiment** | IndoBERT integration, optional but enhances accuracy when configured |
| **Timeline** | No pressure - build it right, quality over speed |

---

## 1. Monorepo Structure (Moon)

```
jejakcuan/
├── moon.yml                    # Moon workspace config
├── .moon/                      # Moon cache and config
│
├── apps/
│   ├── api/                    # Rust backend (Axum)
│   │   ├── moon.yml
│   │   └── Cargo.toml
│   │
│   ├── web/                    # SvelteKit frontend
│   │   ├── moon.yml
│   │   └── package.json
│   │
│   └── ml/                     # Python ML service (LSTM, IndoBERT)
│       ├── moon.yml
│       └── pyproject.toml
│
├── crates/                     # Shared Rust libraries
│   ├── core/                   # Domain models, scoring engine
│   ├── data-sources/           # API adapters (Sectors, yfinance, scrapers)
│   ├── technical/              # Technical indicators (EMA, Fibonacci, OFI)
│   ├── fundamental/            # P/E, EV/EBITDA, DCF calculations
│   └── db/                     # TimescaleDB/PostgreSQL access
│
├── packages/                   # Shared TypeScript packages
│   └── ui/                     # Shared Skeleton UI components
│
├── infra/                      # Infrastructure
│   ├── k3s/                    # Kubernetes manifests
│   └── docker/                 # Dockerfiles
│
├── .github/
│   └── workflows/              # GitHub Actions CI/CD
│
└── docs/                       # Architecture decisions, API specs
```

**Key decisions:**
- **3 apps**: Rust API (core), SvelteKit web, Python ML (isolated for heavy ML libs)
- **Shared crates**: Reusable Rust logic across potential future CLI tools
- **ML as separate service**: Python ecosystem for LSTM/IndoBERT is mature; Rust calls it via HTTP/gRPC

---

## 2. Data Architecture

### Data Source Layer

```
Priority 1 (Free):
• Yahoo Finance (yfinance) - OHLCV, basic fundamentals
• IDX website scraping - Broker summary, stock list
• KSEI AKSes scraping - Shareholding data
• OJK scraping - Insider ownership reports

Priority 2 (Paid, when configured):
• Sectors.app API - Comprehensive IDX data, financials
• TwelveData API - Real-time streaming (optional upgrade)

Priority 3 (Sentiment, when configured):
• Twitter/X API - #saham, #IHSG hashtags
• Telegram scraping - Top stock communities
• News RSS - Kontan, CNBC Indonesia, Bisnis Indonesia
```

### Storage Layer

```
TimescaleDB (PostgreSQL extension):
• stock_prices (hypertable) - OHLCV time-series
• broker_summary (hypertable) - Daily broker net buy/sell
• order_flow (hypertable) - OFI, OBI metrics

PostgreSQL (regular tables):
• stocks - Master list (~800 IDX stocks)
• financials - Quarterly P/E, EV/EBITDA, revenue
• shareholdings - Ownership %, insider changes
• watchlist - User's tracked stocks
• alerts - User-configured alert rules
• scores - Computed daily scores with breakdown

Redis:
• Latest prices (sub-second access)
• Session/auth tokens
• Rate limiting counters
```

### Data Refresh Strategy

| Data Type | Frequency | Source Priority |
|-----------|-----------|-----------------|
| OHLCV prices | Every 1-5 min (market hours) | Sectors.app → yfinance → IDX scrape |
| Broker summary | Daily EOD | IDX scrape (free) |
| Financials | Quarterly | Sectors.app → yfinance |
| Shareholding | Daily check | KSEI/OJK scrape |
| Sentiment | Hourly | Twitter API → Telegram → News RSS |

---

## 3. Scoring Engine Architecture

### Component Breakdown

```
TECHNICAL SCORE (0-100)              Weight: 40% (configurable)
├── Order Flow Imbalance (OFI)       └─ 25% of technical
├── Order Book Imbalance (OBI)       └─ 20% of technical
├── Broker Accumulation Score        └─ 25% of technical
│   └─ Foreign institutional (BK,KZ,CS) = highest weight
│   └─ Local institutional (CC,SQ,KI,HP) = medium weight
│   └─ Retail brokers = lowest weight
├── EMA 20 Position                  └─ 15% of technical
│   └─ Price above EMA = bullish
│   └─ EMA slope direction
└── Fibonacci Retracement            └─ 15% of technical
    └─ Near 38.2%, 50%, 61.8% support levels

FUNDAMENTAL SCORE (0-100)            Weight: 40% (configurable)
├── P/E Ratio                        └─ 30% of fundamental
│   └─ vs sector average, vs historical
├── EV/EBITDA                        └─ 30% of fundamental
│   └─ vs sector peers
└── DCF Valuation                    └─ 40% of fundamental
    └─ Intrinsic value vs current price
    └─ Margin of safety %

SENTIMENT SCORE (0-100)              Weight: 10% (configurable)
├── Social Volume Spike              └─ 30% of sentiment
├── IndoBERT Sentiment               └─ 50% of sentiment
│   └─ Positive/Negative/Neutral classification
└── News Flow                        └─ 20% of sentiment
    └─ Tier 1 sources weighted higher
* Defaults to neutral (50) if not configured

ML PREDICTION SCORE (0-100)          Weight: 10% (configurable)
├── LSTM Price Direction             └─ 70% of ML
│   └─ Probability of up/down/sideways
└── Confidence Level                 └─ 30% of ML
    └─ Model certainty affects weight
* Defaults to neutral (50) if model not ready
```

### Score Calculation

```
FINAL COMPOSITE SCORE = Σ(component × weight)

Score Interpretation:
• 80-100: STRONG_BUY - Multiple strong signals aligned
• 65-79:  BUY - Favorable conditions
• 50-64:  HOLD/WATCH - Mixed signals
• 35-49:  WEAK - Unfavorable conditions
• 0-34:   AVOID - Strong negative signals

Score Transparency:
• Every score shows full breakdown
• Each component shows raw value + normalized score
• Weights are user-configurable via settings
```

---

## 4. ML Service Architecture

### Framework

- **Framework:** FastAPI + PyTorch
- **Communication:** HTTP REST (Rust API calls ML service)

### Model 1: LSTM Price Predictor

```
Input Features (per stock, 60-day lookback):
• OHLCV normalized
• Technical indicators (EMA20, RSI, MACD, BB)
• Volume ratios (RVOL, OBV)
• Broker flow scores
• Sector index correlation

Architecture:
• Input layer (feature_count × 60 timesteps)
• LSTM(128) + Dropout(0.2)
• LSTM(64) + Dropout(0.2)
• Dense(32) + ReLU
• Dense(3) + Softmax (up/down/sideways)

Output:
• Direction probability (up/down/sideways)
• Confidence score (0-1)
• 5-day and 20-day horizon predictions
```

### Model 2: IndoBERT Sentiment Analyzer

```
Base: indobenchmark/indobert-base-p1

Fine-tuned on:
• ID-SMSA dataset (3,288 stock tweets)
• Custom labeled Telegram/Stockbit data

Input: Indonesian text (tweets, posts, news headlines)

Output:
• Sentiment: positive/negative/neutral
• Confidence score (0-1)
• Extracted stock tickers mentioned
```

### Training Strategy

- Walk-forward validation (not random split)
- Retrain weekly with new data
- Per-stock models for top 100 liquid stocks
- Sector-generic model for remaining stocks

### API Endpoints

```
POST /predict/price    - LSTM prediction for stock
POST /predict/batch    - Batch predictions (all stocks)
POST /sentiment/text   - Analyze single text
POST /sentiment/batch  - Analyze multiple texts
GET  /models/status    - Model health and last train date
POST /models/retrain   - Trigger manual retrain
```

### Graceful Degradation

- If ML service down → Rust API returns neutral scores
- If model not yet trained → Returns "not_ready" status
- Confidence < 0.6 → Score weighted down automatically

---

## 5. Frontend Architecture (SvelteKit + Skeleton UI)

### Technology Stack

- **Framework:** SvelteKit (Svelte 5)
- **UI Components:** Skeleton UI
- **Charting:** Lightweight Charts (TradingView open-source)
- **State:** Svelte 5 runes ($state, $derived, $effect)
- **Data Fetching:** Server-side load + SSE for real-time

### Page Structure

```
/login                 - Simple auth (single user)

/                      - SCREENER (landing page)
┌─────────────────────────────────────────────────────────┐
│ Filters: Sector | Score Range | Price | Volume | Custom │
├─────────────────────────────────────────────────────────┤
│ Sortable Table:                                         │
│ Stock | Price | Δ% | Score | Tech | Fund | Sent | ML    │
│ BBCA  | 9850  | +1.2| 78    | 82   | 75   | 71   | 80   │
│ TLKM  | 3420  | -0.5| 65    | 60   | 72   | 58   | 70   │
│ ...                                                     │
├─────────────────────────────────────────────────────────┤
│ Click row → Slide-over panel with quick details         │
│ Double-click → Navigate to full stock detail page       │
└─────────────────────────────────────────────────────────┘

/watchlist             - WATCHLIST
┌─────────────────────────────────────────────────────────┐
│ Your tracked stocks with alert status                   │
│ Drag-drop reorder, quick add/remove                     │
│ Alert rules per stock (price, score threshold)          │
└─────────────────────────────────────────────────────────┘

/market                - MARKET OVERVIEW
┌─────────────────────────────────────────────────────────┐
│ IHSG chart + key stats                                  │
│ Sector heatmap (green/red by performance)               │
│ Foreign flow summary (NBSA today/week/month)            │
│ Top gainers / Top losers / Most active                  │
└─────────────────────────────────────────────────────────┘

/signals               - TODAY'S SIGNALS
┌─────────────────────────────────────────────────────────┐
│ Stocks crossing score thresholds today                  │
│ Unusual broker activity detected                        │
│ Ownership change alerts (KSEI/OJK)                      │
│ ML high-confidence predictions                          │
└─────────────────────────────────────────────────────────┘

/stock/[symbol]        - STOCK DETAIL
┌─────────────────────────────────────────────────────────┐
│ Price chart (candlestick + EMA20 + Fibonacci)           │
│ Score breakdown (visual radar/bar chart)                │
│ Fundamentals table (P/E, EV/EBITDA, DCF)                │
│ Broker flow chart (accumulation/distribution)           │
│ Ownership history (major shareholders)                  │
│ Recent news/sentiment                                   │
│ ML prediction with confidence                           │
└─────────────────────────────────────────────────────────┘

/settings              - SETTINGS
┌─────────────────────────────────────────────────────────┐
│ API keys (Sectors.app, Twitter, etc)                    │
│ Scoring weights adjustment                              │
│ Alert preferences (email/push)                          │
│ Theme (dark/light)                                      │
└─────────────────────────────────────────────────────────┘
```

### Real-Time Updates

- SSE connection to /api/stream/prices
- Live price updates on screener/watchlist
- Toast notifications for triggered alerts

### UI/UX Decisions

- Dark mode default (easier on eyes for market watching)
- Skeleton UI components: DataTable, AppBar, SlideToggle, ProgressRadial for scores
- Lightweight Charts for TradingView-quality charts without the cost
- Mobile-responsive but desktop-optimized (primary use case)

---

## 6. Infrastructure & Deployment

### K3s Cluster Layout

```
Namespace: jejakcuan

┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│   web       │  │   api       │  │   ml        │
│  (SvelteKit)│  │  (Rust/Axum)│  │  (Python)   │
│  Port: 3000 │  │  Port: 8080 │  │  Port: 8000 │
│  1 replica  │  │  1 replica  │  │  1 replica  │
└──────┬──────┘  └──────┬──────┘  └──────┬──────┘
       │                │                │
       └────────────────┼────────────────┘
                        │
                  ┌─────▼─────┐
                  │  Traefik  │  (Ingress)
                  │  + TLS    │  (Let's Encrypt)
                  └─────┬─────┘
                        │
            https://jejakcuan.yourdomain.com
```

### Stateful Services

```
┌─────────────────────┐  ┌─────────────────────┐
│   TimescaleDB       │  │   Redis             │
│   (PostgreSQL 16)   │  │   (Cache + Sessions)│
│   PVC: 50Gi         │  │   PVC: 1Gi          │
│   Port: 5432        │  │   Port: 6379        │
└─────────────────────┘  └─────────────────────┘
```

### CronJobs (Kubernetes)

| Job | Schedule | Description |
|-----|----------|-------------|
| data-sync-prices | */5 9-16 * * 1-5 | Every 5min during market hours |
| data-sync-broker | 0 17 * * 1-5 | Daily EOD |
| data-sync-ownership | 0 18 * * 1-5 | Daily evening |
| score-compute | 0 6 * * 1-5 | Daily pre-market |
| ml-retrain | 0 2 * * 0 | Weekly Sunday |
| sentiment-scrape | 0 * * * * | Hourly |
| db-backup | 0 3 * * * | Daily 3AM |

### GitHub Actions CI/CD

```
.github/workflows/

ci.yml (on push/PR):
├── Lint (Rust: clippy, TS: eslint, Python: ruff)
├── Test (cargo test, vitest, pytest)
├── Type check (cargo check, svelte-check, mypy)
└── Build verification

deploy.yml (on push to main):
├── Build Docker images (multi-arch)
├── Push to GitHub Container Registry (ghcr.io)
├── SSH to VPS
└── kubectl rollout restart
```

### Required Secrets

```
VPS_HOST, VPS_SSH_KEY
KUBECONFIG (base64 encoded)
GHCR_TOKEN (container registry)
```

### Authentication

Simple single-user auth (not OAuth complexity):

- Username/password stored as hashed env var
- Argon2 password hashing
- JWT tokens (short-lived access + longer refresh)
- HttpOnly cookies for web sessions
- Rate limiting on login attempts

---

## 7. Implementation Phases

### Phase 1: Foundation (Weeks 1-3)

**Goal:** Monorepo setup + basic data pipeline + minimal UI

**Tasks:**
- 1.1 Moon monorepo initialization (moon.yml, toolchain configs, Rust workspace, SvelteKit app, Python scaffold)
- 1.2 Database setup (TimescaleDB schema, migrations with sqlx, Docker compose for local dev)
- 1.3 Free data source: Yahoo Finance adapter (fetch IDX stock list, OHLCV historical data, basic scheduler)
- 1.4 Minimal API + UI (GET /stocks endpoints, basic screener page, auth scaffold)

**Checkpoint:**
- [ ] Can view list of ~800 IDX stocks with prices
- [ ] Data refreshes automatically
- [ ] Login protects dashboard

---

### Phase 2: Technical Analysis (Weeks 4-6)

**Goal:** Technical indicators + broker flow + technical scoring

**Tasks:**
- 2.1 Technical indicators crate (EMA, Fibonacci, RSI, MACD, Bollinger Bands, OBV, VPT)
- 2.2 IDX broker summary scraper (daily net buy/sell, broker classification, accumulation score)
- 2.3 Order flow metrics (OBI, OFI, store in TimescaleDB)
- 2.4 Technical score engine (combine into 0-100, configurable weights, breakdown API)
- 2.5 UI: Stock detail page (candlestick chart, Fibonacci levels, broker flow chart)

**Checkpoint:**
- [ ] Screener sortable by technical score
- [ ] Stock detail shows chart + indicators + broker flow
- [ ] Can see why a stock has its technical score

---

### Phase 3: Fundamental Analysis (Weeks 7-9)

**Goal:** Fundamental metrics + DCF model + fundamental scoring

**Tasks:**
- 3.1 Financial data ingestion (Sectors.app adapter, Yahoo Finance fallback, quarterly storage)
- 3.2 Fundamental metrics crate (P/E ratio, EV/EBITDA, sector peer comparison)
- 3.3 DCF valuation model (FCF projection, WACC, intrinsic value, margin of safety)
- 3.4 Fundamental score engine (combine into 0-100, sector-relative scoring)
- 3.5 Shareholding data (KSEI/OJK scraper, insider changes detection, concentration alerts)
- 3.6 UI: Fundamentals display (financials table, DCF assumptions, ownership history)

**Checkpoint:**
- [ ] Screener filterable by P/E, EV/EBITDA
- [ ] DCF valuation shows intrinsic value vs price
- [ ] Ownership changes visible and alertable

---

### Phase 4: ML Prediction (Weeks 10-12)

**Goal:** LSTM model training + prediction API + integration

**Tasks:**
- 4.1 ML service setup (FastAPI scaffold, PyTorch, feature engineering pipeline)
- 4.2 LSTM model development (data preparation, walk-forward training, evaluation metrics)
- 4.3 Training pipeline (historical data export, automated weekly retraining, model versioning)
- 4.4 Prediction API (/predict/price endpoint, batch predictions, confidence scores)
- 4.5 Rust API integration (ML service client, graceful fallback, composite score integration)
- 4.6 UI: ML predictions display (prediction badge, confidence indicator, historical accuracy)

**Checkpoint:**
- [ ] ML predictions visible on all stocks
- [ ] Composite score includes ML component
- [ ] Model retrains weekly automatically

---

### Phase 5: Sentiment Analysis (Weeks 13-15)

**Goal:** IndoBERT sentiment + social scraping + integration

**Tasks:**
- 5.1 IndoBERT model setup (load indobert-base-p1, fine-tune on ID-SMSA, stock ticker NER)
- 5.2 Social media scrapers (Twitter/X API, News RSS, Telegram optional)
- 5.3 Sentiment processing pipeline (text ingestion, IndoBERT inference, stock-level aggregation)
- 5.4 Sentiment API (/sentiment/:symbol endpoint, recent mentions, sentiment trend)
- 5.5 UI: Sentiment display (sentiment gauge, mentions feed, screener integration)

**Checkpoint:**
- [ ] Sentiment scores visible (when sources configured)
- [ ] System works without sentiment (graceful fallback)
- [ ] Can see which posts drove sentiment score

---

### Phase 6: Dashboard Completion (Weeks 16-18)

**Goal:** Watchlist, Market Overview, Signals, Settings

**Tasks:**
- 6.1 Watchlist feature (add/remove stocks, custom alert rules, drag-drop reorder)
- 6.2 Market overview page (IHSG chart, sector heatmap, foreign flow summary, top movers)
- 6.3 Signals page (score threshold crossings, unusual broker activity, ownership alerts, ML predictions)
- 6.4 Settings page (API key configuration, scoring weights, alert preferences, theme toggle)
- 6.5 Notifications (in-app toast, email alerts optional, push notifications optional)

**Checkpoint:**
- [ ] All 4 main pages functional
- [ ] Watchlist persists and alerts work
- [ ] Settings changes reflect immediately

---

### Phase 7: Testing & Backtesting (Weeks 19-21)

**Goal:** Comprehensive tests, backtesting, validation

**Tasks:**
- 7.1 Unit tests (scoring engine logic, technical indicator calculations, DCF model correctness)
- 7.2 Integration tests (data pipeline end-to-end, API endpoint coverage, ML service integration)
- 7.3 Backtesting framework (historical score computation, strategy performance metrics)
- 7.4 Documentation (architecture decision records, API documentation, scoring methodology)

**Checkpoint:**
- [ ] >80% test coverage on critical paths
- [ ] Can backtest scoring strategy
- [ ] Documentation sufficient for future maintenance

---

### Phase 8: Production Hardening (Weeks 22-24)

**Goal:** K3s deployment, CI/CD, monitoring, security

**Tasks:**
- 8.1 Dockerfiles (multi-stage builds, size optimization, security scanning)
- 8.2 K3s manifests (Deployments, Services, Ingress, ConfigMaps, Secrets, PVCs, CronJobs)
- 8.3 GitHub Actions (CI pipeline, CD pipeline, container registry push)
- 8.4 Monitoring (health check endpoints, basic metrics, error alerting)
- 8.5 Security hardening (rate limiting, input validation audit, secrets management, TLS)
- 8.6 Backup & recovery (database backup CronJob, external storage, recovery procedure)

**Checkpoint:**
- [ ] Deployed and accessible at https://your-domain
- [ ] Auto-deploys on push to main
- [ ] Backups running daily
- [ ] Survives server restart

---

## 8. Technical Specifications

### Rust Crates Dependencies

```toml
# apps/api/Cargo.toml
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio", "chrono", "uuid"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
argon2 = "0.5"
jsonwebtoken = "9"
reqwest = { version = "0.11", features = ["json"] }
redis = { version = "0.24", features = ["tokio-comp"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

### SvelteKit Dependencies

```json
{
  "dependencies": {
    "@skeletonlabs/skeleton": "^2.0.0",
    "@skeletonlabs/tw-plugin": "^0.3.0",
    "lightweight-charts": "^4.0.0"
  },
  "devDependencies": {
    "@sveltejs/kit": "^2.0.0",
    "svelte": "^5.0.0",
    "tailwindcss": "^3.4.0",
    "typescript": "^5.0.0",
    "vitest": "^1.0.0"
  }
}
```

### Python ML Dependencies

```toml
# apps/ml/pyproject.toml
[project]
dependencies = [
    "fastapi>=0.109.0",
    "uvicorn>=0.27.0",
    "torch>=2.1.0",
    "transformers>=4.36.0",
    "pandas>=2.1.0",
    "numpy>=1.26.0",
    "scikit-learn>=1.4.0",
    "psycopg2-binary>=2.9.0",
    "httpx>=0.26.0",
]
```

---

## 9. Data Source API Reference

### Yahoo Finance (yfinance)

```python
# IDX stocks use .JK suffix
import yfinance as yf
stock = yf.Ticker("BBCA.JK")
hist = stock.history(period="1y")
info = stock.info  # P/E, market cap, etc.
```

### Sectors.app API

```
Base URL: https://api.sectors.app/v1/
Auth: Bearer token in header

Endpoints:
GET /companies/                    # List all companies
GET /companies/{symbol}/           # Company details
GET /companies/{symbol}/financials # Financial statements
GET /companies/{symbol}/prices     # Historical prices
GET /market/movers                 # Top gainers/losers
```

### IDX Broker Summary (Scraping)

```
URL: https://www.idx.co.id/en/market-data/stock-summary/broker-summary/
Data: Daily aggregate buy/sell volume per broker code per stock
Format: HTML table, requires parsing
```

### KSEI AKSes (Scraping)

```
URL: https://akses.ksei.co.id/
Auth: Investor account required
Data: Shareholding details, ownership mutations
Format: Web interface, requires authenticated scraping
```

---

## 10. Broker Code Reference

### Foreign Institutional (Highest Weight)

| Code | Name |
|------|------|
| BK | JP Morgan |
| KZ | CLSA |
| CS | Credit Suisse |
| AK | UBS |
| GW | HSBC |
| DP | DBS Vickers |
| RX | Macquarie |
| ZP | Maybank |

### Local Institutional (Medium Weight)

| Code | Name |
|------|------|
| CC | Mandiri Sekuritas |
| SQ | BCA Sekuritas |
| NI | BNI Sekuritas |
| OD | BRI Danareksa |
| HP | Henan Putihrai |
| KI | Ciptadana |
| DX | Bahana |
| IF | Samuel |
| LG | Trimegah |

---

## 11. Risk Considerations

### Technical Risks

- **Data source reliability:** Free sources may have rate limits or go offline
- **ML model accuracy:** Financial prediction is inherently uncertain
- **Scraping fragility:** Website changes can break scrapers

### Mitigations

- Multiple data source fallbacks
- Confidence scores on all predictions
- Monitoring and alerting for data freshness
- Regular scraper maintenance

### Legal Compliance

- Use only public data (no insider information)
- Comply with Indonesia PDP Law for any personal data
- No automated trading execution (analysis only)
- Document all data sources for audit trail

---

## Appendix A: IDX Market Hours

- **Pre-opening:** 08:45 - 09:00 WIB
- **Session 1:** 09:00 - 12:00 WIB
- **Lunch break:** 12:00 - 13:30 WIB
- **Session 2:** 13:30 - 16:00 WIB (15:00 on Friday)
- **Market days:** Monday - Friday (excluding holidays)

---

## Appendix B: Scoring Formulas

### Technical Score Components

```
OFI = ΔV_bid - ΔV_ask
OBI = (Σ bid_volume - Σ ask_volume) / (Σ bid_volume + Σ ask_volume)

EMA_20 = price_above_ema ? 1 : 0 + ema_slope_positive ? 0.5 : 0

Fibonacci_score = distance_to_nearest_level(38.2%, 50%, 61.8%)

Broker_score = Σ(broker_net_buy × broker_weight) / total_volume
```

### Fundamental Score Components

```
PE_score = normalize(sector_avg_PE / stock_PE, 0, 2)
EVEBITDA_score = normalize(sector_avg / stock_ratio, 0, 2)

DCF_intrinsic = Σ(FCF_t / (1 + WACC)^t) + terminal_value
Margin_of_safety = (intrinsic - price) / intrinsic
DCF_score = normalize(margin_of_safety, -0.5, 0.5)
```

---

*Document generated from design session on 2026-01-10*
