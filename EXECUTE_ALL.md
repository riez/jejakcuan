# JejakCuan Full Execution Plan

## Autonomous Execution Prompt

Use this prompt to execute all phases automatically:

```
@ralph: Execute the complete JejakCuan implementation from MASTER_PROMPT.md

PROJECT: /Users/rz/Code/github/riez/jejakcuan
REFERENCE: main-goals.md (specification), MASTER_PROMPT.md (tasks)

EXECUTION RULES:
1. Execute phases 1-9 in strict order
2. Within each phase, complete tasks sequentially
3. For each task:
   a. Select appropriate agent (lisa for research, code-implementer for building)
   b. Follow TDD: write tests first, then implementation
   c. Run quality gates (cargo test, pnpm test, pytest)
   d. Commit with descriptive message
   e. Update todo status
4. If blocked:
   a. Try bart for creative alternatives
   b. Create bd issue if still stuck
   c. Move to next task, return later
5. After each phase:
   a. Run full test suite: moon run :test
   b. Push to remote: git push
   c. Verify phase deliverables work
6. Continue until ALL phases complete
7. Final verification:
   a. All 40 tasks [completed]
   b. All tests pass
   c. System runs end-to-end

COMPLETION PROMISE: <promise>COMPLETE</promise>
Only return this when entire system is production-ready.
```

---

## Phase Execution Checklist

### PHASE 1: Data Infrastructure
```
1.1 [pending] TASK: Sectors.app API client | AGENT: lisa→code-implementer | OUTPUT: crates/data-sources/src/sectors/
1.2 [pending] TASK: TwelveData WebSocket | AGENT: lisa→code-implementer | OUTPUT: crates/data-sources/src/twelvedata/
1.3 [pending] TASK: KSEI shareholding scraper | AGENT: lisa→marge→code-implementer | OUTPUT: crates/data-sources/src/shareholding/
1.4 [pending] TASK: IDX broker summary scraper | AGENT: lisa→code-implementer | OUTPUT: crates/data-sources/src/broker/
```
**Phase 1 Gate:** `cargo test -p data-sources` passes, real data flows

### PHASE 2: Broker Analysis
```
2.1 [pending] TASK: Rolling accumulation detection | AGENT: code-implementer | OUTPUT: crates/data-sources/src/broker/analysis.rs
2.2 [pending] TASK: Institutional flow alerts | AGENT: code-implementer | OUTPUT: crates/core/src/alerts/broker_alerts.rs
```
**Phase 2 Gate:** Accumulation scores calculate correctly from real broker data

### PHASE 3: Technical Indicators
```
3.1 [pending] TASK: Wyckoff phase detection | AGENT: lisa→code-implementer | OUTPUT: crates/technical/src/wyckoff.rs
3.2 [pending] TASK: Volume Price Trend (VPT) | AGENT: code-implementer | OUTPUT: crates/technical/src/vpt.rs
3.3 [pending] TASK: Relative Volume (RVOL) | AGENT: code-implementer | OUTPUT: crates/technical/src/rvol.rs
```
**Phase 3 Gate:** `cargo test -p technical` passes, all indicators compute

### PHASE 4: Scoring & Alerts
```
4.1 [pending] TASK: Weighted signal scoring | AGENT: code-implementer | OUTPUT: crates/core/src/scoring.rs
4.2 [pending] TASK: Alert system architecture | AGENT: lisa→code-implementer | OUTPUT: crates/core/src/alerts/
4.3 [pending] TASK: Notification channels | AGENT: code-implementer | OUTPUT: apps/api/src/notifications/
```
**Phase 4 Gate:** Alerts trigger on test data, notifications deliver

### PHASE 5: Social Sentiment
```
5.1 [pending] TASK: Telegram monitoring | AGENT: lisa→marge→code-implementer | OUTPUT: apps/ml/src/jejakcuan_ml/telegram/
5.2 [pending] TASK: Stockbit integration | AGENT: lisa→code-implementer | OUTPUT: apps/ml/src/jejakcuan_ml/stockbit/
5.3 [pending] TASK: IndoBERT sentiment | AGENT: code-implementer | OUTPUT: apps/ml/src/jejakcuan_ml/models/sentiment.py
5.4 [pending] TASK: Pump-and-dump detection | AGENT: lisa→code-implementer | OUTPUT: apps/ml/src/jejakcuan_ml/detection/
```
**Phase 5 Gate:** `pytest apps/ml` passes, sentiment scores compute

### PHASE 6: ML/Prediction
```
6.1 [pending] TASK: LSTM price prediction | AGENT: code-implementer | OUTPUT: apps/ml/src/jejakcuan_ml/models/lstm.py
6.2 [pending] TASK: Feature engineering | AGENT: code-implementer | OUTPUT: apps/ml/src/jejakcuan_ml/features/
6.3 [pending] TASK: Anomaly detection | AGENT: lisa→code-implementer | OUTPUT: apps/ml/src/jejakcuan_ml/models/anomaly.py
6.4 [pending] TASK: DTW pattern matching | AGENT: lisa→code-implementer | OUTPUT: apps/ml/src/jejakcuan_ml/patterns/
```
**Phase 6 Gate:** Models train and inference runs, accuracy targets met

### PHASE 7: Infrastructure
```
7.1 [pending] TASK: Kafka event streaming | AGENT: lisa→code-implementer | OUTPUT: infra/kafka/
7.2 [pending] TASK: TimescaleDB optimization | AGENT: code-implementer | OUTPUT: crates/db/ + migrations
7.3 [pending] TASK: Redis caching | AGENT: code-implementer | OUTPUT: crates/cache/
7.4 [pending] TASK: Real-time SSE | AGENT: code-implementer | OUTPUT: apps/api/src/routes/streaming.rs
```
**Phase 7 Gate:** Data flows through Kafka→TimescaleDB→Redis→SSE

### PHASE 8: Frontend Dashboard
```
8.1 [pending] TASK: Broker flow visualization | AGENT: bart→code-implementer | OUTPUT: apps/web/src/lib/components/BrokerFlow/
8.2 [pending] TASK: Signal dashboard | AGENT: bart→code-implementer | OUTPUT: apps/web/src/routes/dashboard/
8.3 [pending] TASK: Conglomerate tracking | AGENT: bart→code-implementer | OUTPUT: apps/web/src/routes/conglomerates/
8.4 [pending] TASK: Alert management UI | AGENT: code-implementer | OUTPUT: apps/web/src/routes/alerts/
```
**Phase 8 Gate:** `pnpm test` passes, UI renders all components

### PHASE 9: Compliance & Production
```
9.1 [pending] TASK: Audit logging | AGENT: marge→code-implementer | OUTPUT: crates/audit/
9.2 [pending] TASK: PDP Law compliance | AGENT: marge | OUTPUT: docs/compliance/
9.3 [pending] TASK: Production readiness | AGENT: marge→homer→code-implementer | OUTPUT: infra/k8s/
```
**Phase 9 Gate:** Security audit passes, load test succeeds, deployed

---

## Quick Start Commands

### Execute Everything (Autonomous)
```
@ralph: Execute the complete JejakCuan implementation from MASTER_PROMPT.md for /Users/rz/Code/github/riez/jejakcuan. Follow EXECUTE_ALL.md checklist. Don't stop until <promise>COMPLETE</promise>.
```

### Execute Single Phase
```
@orchestrator: Execute PHASE 3 (Technical Indicators) from EXECUTE_ALL.md for /Users/rz/Code/github/riez/jejakcuan
```

### Execute Single Task
```
@orchestrator: Execute Task 3.1 (Wyckoff phase detection) from MASTER_PROMPT.md
```

### Resume from Checkpoint
```
@ralph: Resume JejakCuan implementation from EXECUTE_ALL.md. Check todo status, find first [pending] task, continue from there until <promise>COMPLETE</promise>.
```

---

## Progress Tracking

Update this section as phases complete:

| Phase | Status | Completed | Notes |
|-------|--------|-----------|-------|
| 1 | pending | 0/4 | |
| 2 | pending | 0/2 | |
| 3 | pending | 0/3 | |
| 4 | pending | 0/3 | |
| 5 | pending | 0/4 | |
| 6 | pending | 0/4 | |
| 7 | pending | 0/4 | |
| 8 | pending | 0/4 | |
| 9 | pending | 0/3 | |
| **TOTAL** | **pending** | **0/31** | |

---

## Troubleshooting

### If Agent Gets Stuck
```
@bart: Find creative alternative for [describe blocker] in jejakcuan project
```

### If Tests Fail
```
@orchestrator: Debug failing tests in [module], use systematic-debugging skill
```

### If Security Concern
```
@marge: Review [file/feature] for security issues before proceeding
```

### If Need Parallel Processing
```
@homer: Process all [files/tasks] in parallel for jejakcuan
```
