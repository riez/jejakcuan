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
