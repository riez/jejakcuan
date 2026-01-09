-- Enable TimescaleDB extension
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Stocks master table
CREATE TABLE IF NOT EXISTS stocks (
    id SERIAL PRIMARY KEY,
    symbol VARCHAR(10) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    sector VARCHAR(100),
    subsector VARCHAR(100),
    listing_date DATE,
    market_cap BIGINT,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_stocks_symbol ON stocks(symbol);
CREATE INDEX idx_stocks_sector ON stocks(sector);

-- Stock prices (hypertable for time-series)
CREATE TABLE IF NOT EXISTS stock_prices (
    time TIMESTAMPTZ NOT NULL,
    symbol VARCHAR(10) NOT NULL,
    open NUMERIC(18, 4) NOT NULL,
    high NUMERIC(18, 4) NOT NULL,
    low NUMERIC(18, 4) NOT NULL,
    close NUMERIC(18, 4) NOT NULL,
    volume BIGINT NOT NULL,
    value NUMERIC(20, 2),
    frequency BIGINT,
    CONSTRAINT fk_stock_prices_symbol FOREIGN KEY (symbol) REFERENCES stocks(symbol)
);

SELECT create_hypertable('stock_prices', 'time', if_not_exists => TRUE);
CREATE INDEX idx_stock_prices_symbol_time ON stock_prices(symbol, time DESC);

-- Broker summary data (hypertable)
CREATE TABLE IF NOT EXISTS broker_summary (
    time TIMESTAMPTZ NOT NULL,
    symbol VARCHAR(10) NOT NULL,
    broker_code VARCHAR(4) NOT NULL,
    buy_volume BIGINT DEFAULT 0,
    sell_volume BIGINT DEFAULT 0,
    buy_value NUMERIC(20, 2) DEFAULT 0,
    sell_value NUMERIC(20, 2) DEFAULT 0,
    net_volume BIGINT GENERATED ALWAYS AS (buy_volume - sell_volume) STORED,
    net_value NUMERIC(20, 2) GENERATED ALWAYS AS (buy_value - sell_value) STORED,
    CONSTRAINT fk_broker_summary_symbol FOREIGN KEY (symbol) REFERENCES stocks(symbol)
);

SELECT create_hypertable('broker_summary', 'time', if_not_exists => TRUE);
CREATE INDEX idx_broker_summary_symbol_time ON broker_summary(symbol, time DESC);
CREATE INDEX idx_broker_summary_broker ON broker_summary(broker_code, time DESC);

-- Broker classification
CREATE TABLE IF NOT EXISTS brokers (
    code VARCHAR(4) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    category VARCHAR(50) NOT NULL, -- 'foreign_institutional', 'local_institutional', 'retail'
    weight NUMERIC(3, 2) DEFAULT 1.0, -- Weight for scoring (0.0 - 1.0)
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Insert common broker codes
INSERT INTO brokers (code, name, category, weight) VALUES
    ('BK', 'JP Morgan', 'foreign_institutional', 1.0),
    ('KZ', 'CLSA', 'foreign_institutional', 1.0),
    ('CS', 'Credit Suisse', 'foreign_institutional', 1.0),
    ('AK', 'UBS', 'foreign_institutional', 1.0),
    ('GW', 'HSBC', 'foreign_institutional', 0.9),
    ('DP', 'DBS Vickers', 'foreign_institutional', 0.9),
    ('RX', 'Macquarie', 'foreign_institutional', 0.9),
    ('ZP', 'Maybank', 'foreign_institutional', 0.8),
    ('CC', 'Mandiri Sekuritas', 'local_institutional', 0.8),
    ('SQ', 'BCA Sekuritas', 'local_institutional', 0.8),
    ('NI', 'BNI Sekuritas', 'local_institutional', 0.7),
    ('OD', 'BRI Danareksa', 'local_institutional', 0.7),
    ('HP', 'Henan Putihrai', 'local_institutional', 0.7),
    ('KI', 'Ciptadana', 'local_institutional', 0.7),
    ('DX', 'Bahana', 'local_institutional', 0.6),
    ('IF', 'Samuel', 'local_institutional', 0.6),
    ('LG', 'Trimegah', 'local_institutional', 0.6)
ON CONFLICT (code) DO NOTHING;

-- Financial data (quarterly)
CREATE TABLE IF NOT EXISTS financials (
    id SERIAL PRIMARY KEY,
    symbol VARCHAR(10) NOT NULL,
    period_end DATE NOT NULL,
    revenue NUMERIC(20, 2),
    net_income NUMERIC(20, 2),
    total_assets NUMERIC(20, 2),
    total_equity NUMERIC(20, 2),
    total_debt NUMERIC(20, 2),
    ebitda NUMERIC(20, 2),
    free_cash_flow NUMERIC(20, 2),
    eps NUMERIC(18, 4),
    book_value_per_share NUMERIC(18, 4),
    pe_ratio NUMERIC(10, 2),
    pb_ratio NUMERIC(10, 2),
    ev_ebitda NUMERIC(10, 2),
    roe NUMERIC(10, 4),
    roa NUMERIC(10, 4),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT fk_financials_symbol FOREIGN KEY (symbol) REFERENCES stocks(symbol),
    CONSTRAINT uq_financials_symbol_period UNIQUE (symbol, period_end)
);

CREATE INDEX idx_financials_symbol ON financials(symbol, period_end DESC);

-- Shareholding data
CREATE TABLE IF NOT EXISTS shareholdings (
    id SERIAL PRIMARY KEY,
    symbol VARCHAR(10) NOT NULL,
    reported_date DATE NOT NULL,
    shareholder_name VARCHAR(255) NOT NULL,
    shareholder_type VARCHAR(50), -- 'insider', 'institution', 'public'
    shares_held BIGINT NOT NULL,
    percentage NUMERIC(10, 4) NOT NULL,
    change_shares BIGINT DEFAULT 0,
    change_percentage NUMERIC(10, 4) DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT fk_shareholdings_symbol FOREIGN KEY (symbol) REFERENCES stocks(symbol)
);

CREATE INDEX idx_shareholdings_symbol ON shareholdings(symbol, reported_date DESC);
CREATE INDEX idx_shareholdings_name ON shareholdings(shareholder_name);

-- Computed scores (hypertable)
CREATE TABLE IF NOT EXISTS stock_scores (
    time TIMESTAMPTZ NOT NULL,
    symbol VARCHAR(10) NOT NULL,
    composite_score NUMERIC(5, 2) NOT NULL,
    technical_score NUMERIC(5, 2) NOT NULL,
    fundamental_score NUMERIC(5, 2) NOT NULL,
    sentiment_score NUMERIC(5, 2) DEFAULT 50.0,
    ml_score NUMERIC(5, 2) DEFAULT 50.0,
    -- Score breakdown JSON for transparency
    technical_breakdown JSONB,
    fundamental_breakdown JSONB,
    sentiment_breakdown JSONB,
    ml_breakdown JSONB,
    CONSTRAINT fk_stock_scores_symbol FOREIGN KEY (symbol) REFERENCES stocks(symbol)
);

SELECT create_hypertable('stock_scores', 'time', if_not_exists => TRUE);
CREATE INDEX idx_stock_scores_symbol_time ON stock_scores(symbol, time DESC);
CREATE INDEX idx_stock_scores_composite ON stock_scores(time DESC, composite_score DESC);

-- User watchlist
CREATE TABLE IF NOT EXISTS watchlist (
    id SERIAL PRIMARY KEY,
    symbol VARCHAR(10) NOT NULL,
    sort_order INT DEFAULT 0,
    notes TEXT,
    added_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT fk_watchlist_symbol FOREIGN KEY (symbol) REFERENCES stocks(symbol),
    CONSTRAINT uq_watchlist_symbol UNIQUE (symbol)
);

-- Alert rules
CREATE TABLE IF NOT EXISTS alerts (
    id SERIAL PRIMARY KEY,
    symbol VARCHAR(10),
    alert_type VARCHAR(50) NOT NULL, -- 'price_above', 'price_below', 'score_above', 'volume_spike', etc.
    condition JSONB NOT NULL, -- Flexible condition storage
    is_active BOOLEAN DEFAULT true,
    last_triggered TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT fk_alerts_symbol FOREIGN KEY (symbol) REFERENCES stocks(symbol)
);

CREATE INDEX idx_alerts_active ON alerts(is_active) WHERE is_active = true;

-- Alert history
CREATE TABLE IF NOT EXISTS alert_history (
    id SERIAL PRIMARY KEY,
    alert_id INT REFERENCES alerts(id),
    triggered_at TIMESTAMPTZ DEFAULT NOW(),
    trigger_value JSONB,
    notification_sent BOOLEAN DEFAULT false
);

-- App settings (single row)
CREATE TABLE IF NOT EXISTS settings (
    id INT PRIMARY KEY DEFAULT 1,
    score_weights JSONB DEFAULT '{"technical": 0.4, "fundamental": 0.4, "sentiment": 0.1, "ml": 0.1}',
    api_keys JSONB DEFAULT '{}', -- Encrypted in production
    preferences JSONB DEFAULT '{"theme": "dark", "refresh_interval": 60}',
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    CONSTRAINT single_row CHECK (id = 1)
);

INSERT INTO settings (id) VALUES (1) ON CONFLICT DO NOTHING;

-- Sentiment data (hypertable)
CREATE TABLE IF NOT EXISTS sentiment_data (
    time TIMESTAMPTZ NOT NULL,
    symbol VARCHAR(10) NOT NULL,
    source VARCHAR(50) NOT NULL, -- 'twitter', 'telegram', 'news'
    text_snippet TEXT,
    sentiment VARCHAR(20) NOT NULL, -- 'positive', 'negative', 'neutral'
    confidence NUMERIC(5, 4) NOT NULL,
    CONSTRAINT fk_sentiment_symbol FOREIGN KEY (symbol) REFERENCES stocks(symbol)
);

SELECT create_hypertable('sentiment_data', 'time', if_not_exists => TRUE);
CREATE INDEX idx_sentiment_symbol_time ON sentiment_data(symbol, time DESC);

-- ML predictions (hypertable)
CREATE TABLE IF NOT EXISTS ml_predictions (
    time TIMESTAMPTZ NOT NULL,
    symbol VARCHAR(10) NOT NULL,
    direction VARCHAR(20) NOT NULL, -- 'up', 'down', 'sideways'
    confidence NUMERIC(5, 4) NOT NULL,
    horizon_days INT NOT NULL,
    model_version VARCHAR(50),
    CONSTRAINT fk_ml_predictions_symbol FOREIGN KEY (symbol) REFERENCES stocks(symbol)
);

SELECT create_hypertable('ml_predictions', 'time', if_not_exists => TRUE);
CREATE INDEX idx_ml_predictions_symbol_time ON ml_predictions(symbol, time DESC);

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_stocks_updated_at
    BEFORE UPDATE ON stocks
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_settings_updated_at
    BEFORE UPDATE ON settings
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
