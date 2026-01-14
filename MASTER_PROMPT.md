# JejakCuan Master Implementation Prompt

## Project Overview

Build a comprehensive Indonesian stock tracking system that combines:
- Real-time broker flow analysis (bandarmology)
- Shareholding pattern detection
- Social media sentiment monitoring in Bahasa Indonesia
- Technical accumulation signals
- ML-based price prediction

**Repository:** `/Users/rz/Code/github/riez/jejakcuan`
**Reference:** `main-goals.md` (60KB comprehensive specification)

## Technology Stack

- **Rust** (crates/): Core logic, technical indicators, data sources, scoring
- **SvelteKit** (apps/web/): Frontend dashboard
- **Python** (apps/ml/): ML models, sentiment analysis
- **Axum** (apps/api/): REST API
- **TimescaleDB**: Time-series storage
- **Moon**: Monorepo orchestration
- **bd (beads)**: Issue tracking

## Current State Assessment

### Already Implemented
- Technical indicators: RSI, MACD, EMA, Bollinger, OBI, OFI, VAMP, ADL, OBV
- Broker classification and HHI calculation (placeholder data)
- Shareholding models and analysis structure
- Yahoo Finance client
- Database schema and repositories
- Basic API routes (auth, stocks, watchlist)
- Frontend components (ScoreGauge, SignalCard)
- ML scaffolding (LSTM, sentiment endpoints)

### Needs Implementation
- Real data source integrations (Sectors.app, TwelveData, IDX)
- Wyckoff phase detection
- Social sentiment monitoring (Telegram, Stockbit, Kaskus)
- Alert system
- Real-time streaming (Kafka, SSE)
- Production ML models
- Full frontend dashboard

---

## PHASE 1: DATA INFRASTRUCTURE

### Task 1.1: Sectors.app API Integration
```
TASK: Implement Sectors.app API client for IDX market data
AGENT: lisa (research) → code-implementer (build)
METHOD: Research API docs → Design client → TDD implementation
OUTPUT: crates/data-sources/src/sectors/ module with full test coverage

Requirements:
- REST client with retry logic and rate limiting
- Endpoints: company financials, quarterly reports, sector analysis, top movers
- Response models matching their JSON schema
- Error handling for API limits and failures
- Integration with existing DataSourceError

Reference main-goals.md:
"Sectors.app has emerged as the premier Indonesian-focused API provider"
```

### Task 1.2: TwelveData WebSocket Streaming
```
TASK: Implement TwelveData WebSocket client for real-time IDX prices
AGENT: lisa (research) → code-implementer (build)
METHOD: Research WebSocket API → Design reconnection logic → TDD
OUTPUT: crates/data-sources/src/twelvedata/ module

Requirements:
- WebSocket client with auto-reconnection
- 170ms latency target
- Support 1-minute to 1-month intervals
- Subscription management for multiple symbols
- Backpressure handling
- Integration with Kafka for downstream processing

Reference main-goals.md:
"TwelveData provides real-time IDX data through WebSocket streaming with 170ms latency"
```

### Task 1.3: KSEI Shareholding Scraper
```
TASK: Implement KSEI AKSes portal scraper for ownership data
AGENT: lisa (research legality) → marge (security review) → code-implementer
METHOD: Research portal structure → Design scraper → Legal review → TDD
OUTPUT: crates/data-sources/src/shareholding/scraper.rs (replace placeholder)

Requirements:
- Web scraper for AKSes portal (no official API)
- Extract: shareholder names, percentages, mutation history
- 30-day historical data support
- Rate limiting to avoid blocks
- Respect robots.txt and ToS
- PDP Law compliance (public data only)

Reference main-goals.md:
"KSEI's AKSes portal requires web scraping for automated access since no official API exists"
```

### Task 1.4: IDX Broker Summary Scraper
```
TASK: Implement actual broker summary data scraping (replace placeholder)
AGENT: lisa (research sources) → code-implementer (build)
METHOD: Identify reliable source → Design scraper → TDD
OUTPUT: crates/data-sources/src/broker/scraper.rs (functional implementation)

Requirements:
- Scrape daily broker summary from IDX or Stockbit
- Parse broker codes (HP, KI, CC, BK, etc.)
- Extract: buy_volume, sell_volume, buy_value, sell_value per broker per stock
- Historical data storage
- Handle site structure changes gracefully

Reference main-goals.md:
"The IDX publishes daily aggregate trading data by broker code"
Current state: scraper.rs returns empty vec![] - needs real implementation
```

---

## PHASE 2: BROKER ANALYSIS (BANDARMOLOGY)

### Task 2.1: Rolling Accumulation Detection
```
TASK: Implement rolling 5-day and 20-day accumulation detection
AGENT: code-implementer
METHOD: TDD with edge cases
OUTPUT: crates/data-sources/src/broker/analysis.rs

Requirements:
- Calculate net positions over rolling windows
- Detect when multiple institutional codes show coordinated buying
- Track persistence of accumulation (days_accumulated)
- Generate accumulation_score trending over time

Reference main-goals.md:
"monitoring top 10 broker activities daily, calculating net positions over rolling 5-day and 20-day windows"
```

### Task 2.2: Institutional Flow Alerts
```
TASK: Alert when institutional brokers show coordinated activity
AGENT: code-implementer
METHOD: TDD
OUTPUT: crates/core/src/alerts/broker_alerts.rs

Requirements:
- Trigger when 3+ institutional codes (BK, KZ, CC, SQ, HP, KI) net buy same stock
- Weight by broker category (foreign institutional > local institutional > retail)
- Configurable thresholds
- Integration with alert system (Phase 4)

Reference main-goals.md:
"foreign institutional brokers (BK, KZ, CS, AK) receiving highest credibility ratings"
```

---

## PHASE 3: TECHNICAL INDICATORS

### Task 3.1: Wyckoff Phase Detection
```
TASK: Implement Wyckoff accumulation phase detection (A through E)
AGENT: lisa (research methodology) → code-implementer (build)
METHOD: Research Wyckoff theory → Design state machine → TDD
OUTPUT: crates/technical/src/wyckoff.rs

Requirements:
- Detect 5 phases:
  - Phase A: Selling climax (stopping volume)
  - Phase B: Building cause (sideways, volume decline)
  - Phase C: Spring test of support
  - Phase D: Markup preparation (subtle strength)
  - Phase E: Breakout to markup
- Signals:
  - Price range < 50% of 60-day volatility
  - Volume < 70% of 60-day average
  - Daily close std dev < 2%
- State machine tracking phase transitions
- Confidence score per phase

Reference main-goals.md:
"Wyckoff accumulation phases progress through five stages"
```

### Task 3.2: Volume Price Trend (VPT)
```
TASK: Implement VPT indicator for accumulation detection
AGENT: code-implementer
METHOD: TDD
OUTPUT: crates/technical/src/vpt.rs

Requirements:
- Formula: VPT(t) = VPT(t-1) + Volume × [(Close - Close_prev) / Close_prev]
- Detect divergence: rising VPT with flat prices = accumulation
- Integration with scoring system

Reference main-goals.md:
"Rising VPT with flat prices indicates smart money accumulation"
```

### Task 3.3: Relative Volume (RVOL)
```
TASK: Implement RVOL for unusual activity detection
AGENT: code-implementer
METHOD: TDD
OUTPUT: crates/technical/src/rvol.rs

Requirements:
- Formula: RVOL = current_volume / average_same_period_volume
- Thresholds: >2.0 high activity, >3.0 extreme activity
- Separate morning (09:00-12:00) vs afternoon (13:30-16:00) analysis
- Volume spike detection: volume > MA + (2 × std_dev)

Reference main-goals.md:
"RVOL above 2.0 signals high activity and above 3.0 indicates extreme activity"
```

---

## PHASE 4: SCORING & ALERTS

### Task 4.1: Weighted Signal Scoring
```
TASK: Implement comprehensive weighted scoring system
AGENT: code-implementer
METHOD: TDD
OUTPUT: crates/core/src/scoring.rs (enhance existing)

Requirements:
- Combine all signals with weights:
  - OBI > 0.2: +2 points
  - Accumulation pattern: +3 points
  - Volume spike: +1 point
  - Institutional buying (broker): +2 points
  - Positive OFI trend: +1 point
  - Bullish price-volume divergence: +2 points
  - Wyckoff accumulation phase: +1 point
- Thresholds:
  - 8+ points: STRONG_BUY
  - 5-7 points: MODERATE_BUY
  - 3-4 points: WATCH
  - <3 points: NEUTRAL

Reference main-goals.md:
"Total scores above 8 points trigger STRONG_BUY signals"
```

### Task 4.2: Alert System Architecture
```
TASK: Implement comprehensive alert system
AGENT: lisa (design) → code-implementer (build)
METHOD: Design event-driven architecture → TDD
OUTPUT: crates/core/src/alerts/ module + apps/api/src/routes/alerts.rs

Requirements:
- Alert types:
  - Threshold alerts (price crosses level)
  - Percentage change (X% in Y timeframe)
  - Technical indicators (RSI/MACD crossovers)
  - Volume alerts (unusual trading)
  - Pattern recognition (Wyckoff phase changes)
  - Broker accumulation alerts
- Rate limiting (max 5 alerts/hour per user)
- Priority levels (critical, high, medium, low)
- Do-Not-Disturb periods

Reference main-goals.md:
"six alert types: threshold alerts, percentage change, technical indicators, volume alerts, pattern recognition, anomaly detection"
```

### Task 4.3: Notification Channels
```
TASK: Implement multi-channel notification delivery
AGENT: code-implementer
METHOD: TDD
OUTPUT: apps/api/src/notifications/ module

Requirements:
- Channels: Email, Push, In-app, Telegram bot
- Retry with exponential backoff
- Circuit breakers for external services
- User preferences storage
- Aggregation (combine related alerts)
```

---

## PHASE 5: SOCIAL SENTIMENT

### Task 5.1: Telegram Community Monitoring
```
TASK: Monitor Indonesian stock Telegram communities
AGENT: lisa (research API/legality) → marge (review) → code-implementer
METHOD: Research Telegram Bot API → Design monitor → Legal review → Build
OUTPUT: apps/ml/src/jejakcuan_ml/telegram/ module

Requirements:
- Target communities:
  - Investor Saham Pemula (197,800 members)
  - Kapten Saham 707 (129,526 members)
  - Sahamku Investasiku (107,635 members)
- Extract: stock mentions, sentiment, volume of discussion
- Detect coordinated pump signals
- Rate limiting and ToS compliance

Reference main-goals.md:
"Telegram dominates Indonesian stock market discussions with the top 10 communities commanding over 500,000 combined members"
```

### Task 5.2: Stockbit Stream Integration
```
TASK: Integrate Stockbit social trading data
AGENT: lisa (research API availability) → code-implementer
METHOD: Research → Design client → Build
OUTPUT: apps/ml/src/jejakcuan_ml/stockbit/ module

Requirements:
- Extract: discussion volume, sentiment per stock
- Track bandarmology indicators if API available
- Correlate social activity with price movements

Reference main-goals.md:
"Stockbit Stream combines social trading with 1+ million members"
```

### Task 5.3: IndoBERT Sentiment Analysis
```
TASK: Implement Indonesian language sentiment analysis
AGENT: code-implementer
METHOD: TDD with Indonesian test data
OUTPUT: apps/ml/src/jejakcuan_ml/models/sentiment.py (enhance existing)

Requirements:
- Deploy IndoBERT model for Indonesian text
- Handle trader slang: "gorengan", "bandar", "cuan", "cutloss"
- Target 60%+ accuracy on stock comments
- Named Entity Recognition for stock code extraction
- Batch processing for efficiency

Reference main-goals.md:
"IndoBERT (Indonesian BERT) achieves 60.35% accuracy on 1,289 stock comments"
```

### Task 5.4: Pump-and-Dump Detection
```
TASK: Detect pump-and-dump manipulation patterns
AGENT: lisa (research patterns) → code-implementer
METHOD: Research historical cases → Design detector → TDD
OUTPUT: apps/ml/src/jejakcuan_ml/detection/pump_dump.py

Requirements:
- Combine signals:
  - Multiple Telegram groups discussing same illiquid stock
  - Sudden volume spike without news
  - Price spike with no fundamental support
  - Coordinated retail broker activity
- Cross-reference with IDX UMA (Unusual Market Activity) list
- Confidence scoring
- Alert generation

Reference main-goals.md:
"41 suspected pump-and-dump stocks in January 2020, with 14 different manipulation periods detected"
```

---

## PHASE 6: ML/PREDICTION

### Task 6.1: LSTM Price Prediction
```
TASK: Implement production LSTM model for price prediction
AGENT: code-implementer
METHOD: TDD with walk-forward validation
OUTPUT: apps/ml/src/jejakcuan_ml/models/lstm.py (enhance existing)

Requirements:
- Architecture: 2-3 LSTM layers (128-64 neurons), Dropout(0.2)
- Features:
  - OHLCV data (1-2 years history)
  - Technical indicators (SMA, EMA, RSI, MACD, Bollinger)
  - Volume data
  - Sentiment scores
  - Broker accumulation scores
- Walk-forward validation (not random train-test split)
- Target: 93% accuracy (per research)
- Model versioning and A/B testing support

Reference main-goals.md:
"LSTM networks provide the primary recommendation for stock prediction, achieving 93% accuracy"
```

### Task 6.2: Feature Engineering Pipeline
```
TASK: Build feature store for ML models
AGENT: code-implementer
METHOD: TDD
OUTPUT: apps/ml/src/jejakcuan_ml/features/ module (enhance existing)

Requirements:
- Consistent feature generation for training and inference
- Features:
  - Volume z-scores
  - Price momentum (5, 10, 20 day)
  - Broker concentration (HHI)
  - Sentiment aggregates (7-day, 30-day)
  - Technical indicator values
- Real-time feature computation via Kafka Streams
- Feature versioning

Reference main-goals.md:
"110+ features including volume z-scores, price momentum pre-announcement, broker concentration"
```

### Task 6.3: Anomaly Detection
```
TASK: Implement Random Forest for anomaly/insider trading detection
AGENT: lisa (research features) → code-implementer
METHOD: Research → Feature selection → TDD
OUTPUT: apps/ml/src/jejakcuan_ml/models/anomaly.py

Requirements:
- Random Forest classifier
- Key features:
  - Volume spikes 5-15 days before announcements
  - Sudden broker composition changes
  - Unusual trading in quiet periods
  - Price momentum vs market
- K-Means clustering for pattern grouping
- Explainability for compliance

Reference main-goals.md:
"Random Forest classifiers trained on 110+ features... achieve 95%+ accuracy"
```

### Task 6.4: Pattern Matching with DTW
```
TASK: Implement Dynamic Time Warping for historical pattern matching
AGENT: lisa (research) → code-implementer
METHOD: Research DTW algorithms → TDD
OUTPUT: apps/ml/src/jejakcuan_ml/patterns/dtw.py

Requirements:
- Compare current price/volume patterns to historical accumulation episodes
- Find similar setups before past breakouts
- Configurable similarity threshold
- Performance optimization for real-time use

Reference main-goals.md:
"Dynamic Time Warping (DTW) for pattern matching across historical episodes"
```

---

## PHASE 7: INFRASTRUCTURE

### Task 7.1: Kafka Event Streaming
```
TASK: Set up Kafka for real-time event processing
AGENT: lisa (architecture) → code-implementer
METHOD: Design topics → Implement producers/consumers → Test
OUTPUT: infra/kafka/ + Rust/Python Kafka clients

Requirements:
- Topics:
  - price-updates (partitioned by symbol)
  - broker-summaries
  - alerts
  - sentiment-scores
- Producers: data source clients
- Consumers: storage, API, ML inference
- Exactly-once semantics where needed
- Docker Compose setup for local dev

Reference main-goals.md:
"Apache Kafka as the central nervous system... handles millions of messages per second"
```

### Task 7.2: TimescaleDB Optimization
```
TASK: Optimize TimescaleDB for production workloads
AGENT: code-implementer
METHOD: Benchmark → Optimize → Test
OUTPUT: crates/db/ enhancements + migrations

Requirements:
- Hypertables partitioned by time
- Compression policies (90%+ storage savings)
- Continuous aggregates for common queries
- Indexes: (symbol, time DESC)
- Batch inserts (1000-10000 records)
- Connection pooling optimization

Reference main-goals.md:
"TimescaleDB outperforms InfluxDB by 6.5x on high-cardinality workloads"
```

### Task 7.3: Redis Caching Layer
```
TASK: Implement Redis caching for low-latency access
AGENT: code-implementer
METHOD: TDD
OUTPUT: crates/cache/ module + apps/api integration

Requirements:
- Cache latest prices (sub-millisecond access)
- User sessions and preferences
- Real-time aggregations (moving averages, top gainers)
- Data structures: Strings, Hashes, Sorted Sets
- TTL-based expiration
- Cache invalidation strategy

Reference main-goals.md:
"Redis serves three critical functions: caching latest prices, storing user sessions, maintaining real-time aggregations"
```

### Task 7.4: Real-time SSE/WebSocket
```
TASK: Implement real-time price streaming to clients
AGENT: code-implementer
METHOD: TDD
OUTPUT: apps/api/src/routes/streaming.rs

Requirements:
- Server-Sent Events (SSE) for price broadcasts
- Automatic reconnection support
- Event-id for resuming from disconnections
- Subscription management (subscribe/unsubscribe symbols)
- Backpressure handling

Reference main-goals.md:
"Server-Sent Events (SSE) emerge as the recommended approach for stock price broadcasts"
```

---

## PHASE 8: FRONTEND DASHBOARD

### Task 8.1: Broker Flow Visualization
```
TASK: Build broker flow visualization component
AGENT: bart (creative design) → code-implementer
METHOD: Design mockup → Implement → Test
OUTPUT: apps/web/src/lib/components/BrokerFlow/

Requirements:
- Stacked bar chart: net buy/sell by broker category
- Color coding: green (accumulation), red (distribution)
- Foreign vs domestic flow toggle
- Historical trend view (5-day, 20-day)
- HHI concentration indicator
- Interactive tooltips with broker details

Reference main-goals.md:
"Bandar Volume indicator displays daily net buy/sell by top brokers in green (accumulation) and red (distribution)"
```

### Task 8.2: Accumulation Signal Dashboard
```
TASK: Build main signal dashboard
AGENT: bart (design) → code-implementer
METHOD: Design → Implement → Test
OUTPUT: apps/web/src/routes/dashboard/

Requirements:
- Signal cards with weighted scores
- Wyckoff phase indicator
- Technical indicator summary (RSI, MACD, OBI)
- Broker accumulation status
- Sentiment gauge
- Alert history
- Watchlist quick view

Reference main-goals.md:
"Total scores above 8 points trigger STRONG_BUY signals, 5-7 points generate MODERATE_BUY"
```

### Task 8.3: Conglomerate Tracking View
```
TASK: Build conglomerate portfolio tracking
AGENT: bart (design) → code-implementer
METHOD: Design → Implement → Test
OUTPUT: apps/web/src/routes/conglomerates/

Requirements:
- Track major conglomerates:
  - Prajogo Pangestu (BRPT, TPIA, CUAN, BREN, PTRO)
  - Salim Group (INDF, ICBP, LSIP, SIMP, META, DMMX)
  - Lippo Group (LPKR, LPCK, SILO, MLPL, MAPI)
  - Hartono/Djarum (BBCA, TOWR)
  - Astra Group
- Insider transaction timeline
- Portfolio value tracking
- Corporate action calendar
- Ownership concentration alerts

Reference main-goals.md:
"Prajogo Pangestu has emerged as Indonesia's most successful stock market operator"
```

### Task 8.4: Alert Management UI
```
TASK: Build alert configuration and history UI
AGENT: code-implementer
METHOD: TDD
OUTPUT: apps/web/src/routes/alerts/

Requirements:
- Create/edit/delete alerts
- Alert type selection (threshold, pattern, volume, etc.)
- Notification channel preferences
- Alert history with details
- Snooze/dismiss functionality
- Rate limit visibility
```

---

## PHASE 9: COMPLIANCE & PRODUCTION

### Task 9.1: Audit Logging
```
TASK: Implement comprehensive audit logging
AGENT: marge (security review) → code-implementer
METHOD: Security review → Design → Implement
OUTPUT: crates/audit/ module

Requirements:
- Log all data source access (URL, timestamp, data lineage)
- Log user actions (searches, alerts, exports)
- Immutable audit trail
- Retention policies
- Query interface for compliance review

Reference main-goals.md:
"maintain audit logs of all data sources documenting URLs, access timestamps, and data lineage"
```

### Task 9.2: PDP Law Compliance Review
```
TASK: Review and ensure PDP Law compliance
AGENT: marge
METHOD: Security audit → Document findings → Remediate
OUTPUT: docs/compliance/pdp-law.md + code fixes

Requirements:
- Verify only public data is collected
- No individual client data access
- Data retention policy implementation
- Privacy-by-default architecture
- DPIA if handling sensitive data at scale
- Document legal basis for each data source

Reference main-goals.md:
"Personal Data Protection Law No. 27/2022... Personal financial data receives classification as Specific (Sensitive) Personal Data"
```

### Task 9.3: Production Readiness
```
TASK: Production hardening and deployment
AGENT: marge (security) → homer (batch testing) → code-implementer
METHOD: Security audit → Load test → Deploy
OUTPUT: infra/k8s/ + deployment docs

Requirements:
- Kubernetes manifests with auto-scaling
- Multi-AZ deployment
- Health checks and failover
- Prometheus/Grafana monitoring
- ELK stack for logs
- Disaster recovery (RTO <5min, RPO <1min)
- Load testing (1000+ concurrent users)
- Security hardening (encryption, auth)

Reference main-goals.md:
"RPO under 1 minute through write-ahead logging, RTO under 5 minutes with hot standby"
```

---

## EXECUTION INSTRUCTIONS

### For Each Task, Follow This Format:
```
N. [status] TASK: <description> | AGENT: <agent> | METHOD: <approach> | OUTPUT: <deliverable>
```

### Agent Selection Guide:
- **lisa**: Research-heavy tasks, understanding APIs, legal review
- **bart**: Creative solutions, UI/UX design, stuck situations
- **marge**: Security review, compliance, dangerous operations
- **homer**: Batch processing, parallel execution
- **ralph**: Persistent loops, retry until success
- **code-implementer**: TDD implementation
- **orchestrator**: Complex multi-step routing

### Quality Gates (Run Before Completing Each Task):
```bash
# Rust
cargo check --workspace
cargo test --workspace
cargo clippy --workspace

# Python
cd apps/ml && python -m pytest
cd apps/ml && ruff check .

# TypeScript/Svelte
cd apps/web && pnpm test
cd apps/web && pnpm check

# Full build
moon run :build
```

### Issue Tracking:
```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --status in_progress
bd close <id>         # Complete work
bd sync && git push   # ALWAYS push when done
```

---

## SUCCESS CRITERIA

The system is complete when:
1. All 40 tasks pass quality gates
2. Real-time data flows from sources → storage → API → frontend
3. Alerts trigger correctly based on configured conditions
4. ML models achieve target accuracy (LSTM 93%, sentiment 60%+)
5. Dashboard displays all signals and visualizations
6. Compliance documentation is complete
7. System handles 1000+ concurrent users
8. All tests pass, no critical security issues

---

## START COMMAND

To begin implementation, use:
```
@orchestrator: Execute PHASE 1 of MASTER_PROMPT.md for /Users/rz/Code/github/riez/jejakcuan
```

Or for a specific task:
```
@orchestrator: Execute Task 3.1 (Wyckoff Phase Detection) from MASTER_PROMPT.md
```
