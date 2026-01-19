-- Detailed financial statement tables for comprehensive company financial data
-- Stores income statements, balance sheets, and cash flow statements with historical tracking

-- Income Statements (Annual and Quarterly)
CREATE TABLE IF NOT EXISTS income_statements (
    id SERIAL PRIMARY KEY,
    symbol TEXT NOT NULL REFERENCES stocks(symbol) ON DELETE CASCADE,
    fiscal_year INTEGER NOT NULL,
    fiscal_quarter INTEGER, -- NULL for annual reports, 1-4 for quarterly
    period_end DATE NOT NULL,
    currency TEXT DEFAULT 'IDR',
    
    -- Revenue section
    revenue BIGINT,
    cost_of_revenue BIGINT,
    gross_profit BIGINT,
    
    -- Operating section
    operating_expenses BIGINT,
    selling_general_admin BIGINT,
    research_development BIGINT,
    depreciation_amortization BIGINT,
    operating_income BIGINT, -- operating_pnl in Sectors API
    
    -- Non-operating section
    interest_income BIGINT,
    interest_expense BIGINT,
    other_income_expense BIGINT,
    
    -- Bottom line
    earnings_before_tax BIGINT,
    tax_expense BIGINT,
    net_income BIGINT, -- earnings in Sectors API
    
    -- Per share
    eps DECIMAL(12,4),
    eps_diluted DECIMAL(12,4),
    shares_outstanding BIGINT,
    
    -- Margins (calculated or from API)
    gross_margin DECIMAL(8,4),
    operating_margin DECIMAL(8,4),
    net_margin DECIMAL(8,4),
    
    -- Metadata
    source TEXT DEFAULT 'sectors',
    raw_data JSONB, -- Store original API response for reference
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(symbol, fiscal_year, fiscal_quarter)
);

-- Balance Sheets (Annual and Quarterly)
CREATE TABLE IF NOT EXISTS balance_sheets (
    id SERIAL PRIMARY KEY,
    symbol TEXT NOT NULL REFERENCES stocks(symbol) ON DELETE CASCADE,
    fiscal_year INTEGER NOT NULL,
    fiscal_quarter INTEGER, -- NULL for annual reports
    period_end DATE NOT NULL,
    currency TEXT DEFAULT 'IDR',
    
    -- Current Assets
    cash_and_equivalents BIGINT,
    short_term_investments BIGINT,
    accounts_receivable BIGINT,
    inventory BIGINT,
    prepaid_expenses BIGINT,
    other_current_assets BIGINT,
    total_current_assets BIGINT,
    
    -- Non-current Assets
    property_plant_equipment BIGINT,
    accumulated_depreciation BIGINT,
    intangible_assets BIGINT,
    goodwill BIGINT,
    long_term_investments BIGINT,
    other_non_current_assets BIGINT,
    total_non_current_assets BIGINT,
    
    -- Total Assets
    total_assets BIGINT,
    
    -- Current Liabilities
    accounts_payable BIGINT,
    short_term_debt BIGINT,
    current_portion_long_term_debt BIGINT,
    accrued_liabilities BIGINT,
    deferred_revenue BIGINT,
    other_current_liabilities BIGINT,
    total_current_liabilities BIGINT,
    
    -- Non-current Liabilities
    long_term_debt BIGINT,
    deferred_tax_liabilities BIGINT,
    pension_obligations BIGINT,
    other_non_current_liabilities BIGINT,
    total_non_current_liabilities BIGINT,
    
    -- Total Liabilities
    total_liabilities BIGINT,
    total_debt BIGINT, -- short_term + long_term debt
    
    -- Shareholders' Equity
    common_stock BIGINT,
    additional_paid_in_capital BIGINT,
    retained_earnings BIGINT,
    treasury_stock BIGINT,
    accumulated_other_comprehensive_income BIGINT,
    minority_interest BIGINT,
    total_equity BIGINT,
    
    -- Ratios (calculated or from API)
    current_ratio DECIMAL(8,4),
    quick_ratio DECIMAL(8,4),
    debt_to_equity DECIMAL(8,4),
    debt_to_assets DECIMAL(8,4),
    
    -- Metadata
    source TEXT DEFAULT 'sectors',
    raw_data JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(symbol, fiscal_year, fiscal_quarter)
);

-- Cash Flow Statements (Annual and Quarterly)
CREATE TABLE IF NOT EXISTS cash_flow_statements (
    id SERIAL PRIMARY KEY,
    symbol TEXT NOT NULL REFERENCES stocks(symbol) ON DELETE CASCADE,
    fiscal_year INTEGER NOT NULL,
    fiscal_quarter INTEGER, -- NULL for annual reports
    period_end DATE NOT NULL,
    currency TEXT DEFAULT 'IDR',
    
    -- Operating Activities
    net_income BIGINT,
    depreciation_amortization BIGINT,
    stock_based_compensation BIGINT,
    deferred_taxes BIGINT,
    change_in_working_capital BIGINT,
    change_in_receivables BIGINT,
    change_in_inventory BIGINT,
    change_in_payables BIGINT,
    other_operating_activities BIGINT,
    operating_cash_flow BIGINT,
    
    -- Investing Activities
    capital_expenditure BIGINT, -- Usually negative
    acquisitions BIGINT,
    purchase_of_investments BIGINT,
    sale_of_investments BIGINT,
    other_investing_activities BIGINT,
    investing_cash_flow BIGINT,
    
    -- Financing Activities
    debt_issuance BIGINT,
    debt_repayment BIGINT,
    equity_issuance BIGINT,
    equity_repurchase BIGINT,
    dividends_paid BIGINT,
    other_financing_activities BIGINT,
    financing_cash_flow BIGINT,
    
    -- Summary
    net_change_in_cash BIGINT,
    beginning_cash BIGINT,
    ending_cash BIGINT,
    free_cash_flow BIGINT, -- operating_cash_flow + capex
    
    -- Metadata
    source TEXT DEFAULT 'sectors',
    raw_data JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(symbol, fiscal_year, fiscal_quarter)
);

-- Financial Ratios History (for tracking key metrics over time)
CREATE TABLE IF NOT EXISTS financial_ratios (
    id SERIAL PRIMARY KEY,
    symbol TEXT NOT NULL REFERENCES stocks(symbol) ON DELETE CASCADE,
    fiscal_year INTEGER NOT NULL,
    fiscal_quarter INTEGER,
    period_end DATE NOT NULL,
    
    -- Profitability Ratios
    roe DECIMAL(10,4), -- Return on Equity
    roa DECIMAL(10,4), -- Return on Assets
    roic DECIMAL(10,4), -- Return on Invested Capital
    gross_margin DECIMAL(10,4),
    operating_margin DECIMAL(10,4),
    net_margin DECIMAL(10,4),
    ebitda_margin DECIMAL(10,4),
    
    -- Liquidity Ratios
    current_ratio DECIMAL(10,4),
    quick_ratio DECIMAL(10,4),
    cash_ratio DECIMAL(10,4),
    
    -- Leverage Ratios
    debt_to_equity DECIMAL(10,4),
    debt_to_assets DECIMAL(10,4),
    interest_coverage DECIMAL(10,4),
    debt_to_ebitda DECIMAL(10,4),
    
    -- Efficiency Ratios
    asset_turnover DECIMAL(10,4),
    inventory_turnover DECIMAL(10,4),
    receivables_turnover DECIMAL(10,4),
    days_sales_outstanding DECIMAL(10,4),
    days_inventory DECIMAL(10,4),
    
    -- Valuation Ratios (requires price data)
    pe_ratio DECIMAL(10,4),
    pb_ratio DECIMAL(10,4),
    ps_ratio DECIMAL(10,4),
    ev_to_ebitda DECIMAL(10,4),
    dividend_yield DECIMAL(10,4),
    payout_ratio DECIMAL(10,4),
    
    -- Per Share Metrics
    eps DECIMAL(12,4),
    book_value_per_share DECIMAL(12,4),
    revenue_per_share DECIMAL(12,4),
    free_cash_flow_per_share DECIMAL(12,4),
    
    -- Growth Rates (YoY)
    revenue_growth DECIMAL(10,4),
    earnings_growth DECIMAL(10,4),
    eps_growth DECIMAL(10,4),
    
    -- Metadata
    source TEXT DEFAULT 'sectors',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(symbol, fiscal_year, fiscal_quarter)
);

-- Indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_income_stmt_symbol ON income_statements(symbol);
CREATE INDEX IF NOT EXISTS idx_income_stmt_year ON income_statements(fiscal_year DESC);
CREATE INDEX IF NOT EXISTS idx_income_stmt_period ON income_statements(symbol, fiscal_year DESC, fiscal_quarter);

CREATE INDEX IF NOT EXISTS idx_balance_sheet_symbol ON balance_sheets(symbol);
CREATE INDEX IF NOT EXISTS idx_balance_sheet_year ON balance_sheets(fiscal_year DESC);
CREATE INDEX IF NOT EXISTS idx_balance_sheet_period ON balance_sheets(symbol, fiscal_year DESC, fiscal_quarter);

CREATE INDEX IF NOT EXISTS idx_cash_flow_symbol ON cash_flow_statements(symbol);
CREATE INDEX IF NOT EXISTS idx_cash_flow_year ON cash_flow_statements(fiscal_year DESC);
CREATE INDEX IF NOT EXISTS idx_cash_flow_period ON cash_flow_statements(symbol, fiscal_year DESC, fiscal_quarter);

CREATE INDEX IF NOT EXISTS idx_fin_ratios_symbol ON financial_ratios(symbol);
CREATE INDEX IF NOT EXISTS idx_fin_ratios_period ON financial_ratios(symbol, fiscal_year DESC, fiscal_quarter);

-- Function to update timestamps
CREATE OR REPLACE FUNCTION update_financial_statement_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Triggers for auto-updating timestamps
DROP TRIGGER IF EXISTS update_income_stmt_timestamp ON income_statements;
CREATE TRIGGER update_income_stmt_timestamp
    BEFORE UPDATE ON income_statements
    FOR EACH ROW EXECUTE FUNCTION update_financial_statement_timestamp();

DROP TRIGGER IF EXISTS update_balance_sheet_timestamp ON balance_sheets;
CREATE TRIGGER update_balance_sheet_timestamp
    BEFORE UPDATE ON balance_sheets
    FOR EACH ROW EXECUTE FUNCTION update_financial_statement_timestamp();

DROP TRIGGER IF EXISTS update_cash_flow_timestamp ON cash_flow_statements;
CREATE TRIGGER update_cash_flow_timestamp
    BEFORE UPDATE ON cash_flow_statements
    FOR EACH ROW EXECUTE FUNCTION update_financial_statement_timestamp();
