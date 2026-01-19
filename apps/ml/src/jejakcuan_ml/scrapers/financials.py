"""Financial statements scraper using Sectors.app API."""

import os
from dataclasses import dataclass
from datetime import date
from decimal import Decimal
from typing import Any

from loguru import logger

from .base import BaseScraper, ScraperConfig
from .database import DatabaseClient


@dataclass
class IncomeStatementData:
    symbol: str
    fiscal_year: int
    fiscal_quarter: int | None
    period_end: date
    revenue: int | None
    gross_profit: int | None
    operating_income: int | None
    earnings_before_tax: int | None
    tax_expense: int | None
    net_income: int | None
    eps: Decimal | None
    gross_margin: Decimal | None
    operating_margin: Decimal | None
    net_margin: Decimal | None
    raw_data: dict[str, Any] | None = None


@dataclass
class BalanceSheetData:
    symbol: str
    fiscal_year: int
    fiscal_quarter: int | None
    period_end: date
    total_assets: int | None
    total_liabilities: int | None
    total_equity: int | None
    total_debt: int | None
    current_ratio: Decimal | None
    debt_to_equity: Decimal | None
    raw_data: dict[str, Any] | None = None


@dataclass
class CashFlowData:
    symbol: str
    fiscal_year: int
    fiscal_quarter: int | None
    period_end: date
    operating_cash_flow: int | None
    capital_expenditure: int | None
    free_cash_flow: int | None
    raw_data: dict[str, Any] | None = None


@dataclass
class FinancialRatiosData:
    symbol: str
    fiscal_year: int
    fiscal_quarter: int | None
    period_end: date
    roe: Decimal | None
    roa: Decimal | None
    gross_margin: Decimal | None
    operating_margin: Decimal | None
    net_margin: Decimal | None
    current_ratio: Decimal | None
    debt_to_equity: Decimal | None
    eps: Decimal | None


class FinancialsScraper(BaseScraper):
    """Scraper for company financials from Sectors.app API."""

    SECTORS_BASE_URL = "https://api.sectors.app/v1"

    def __init__(
        self,
        config: ScraperConfig | None = None,
        db_client: DatabaseClient | None = None,
        symbols: list[str] | None = None,
        api_key: str | None = None,
    ) -> None:
        cfg = config or ScraperConfig(
            requests_per_minute=20,
            min_delay=2.0,
            max_delay=4.0,
        )
        super().__init__(cfg, db_client)
        self._symbols = symbols
        self._api_key = api_key or os.environ.get("SECTORS_API_KEY", "")

        if self._api_key:
            self.config.extra_headers["Authorization"] = self._api_key

    def get_name(self) -> str:
        return "Financial Statements"

    async def scrape(self) -> int:
        count = 0

        if not self._api_key:
            logger.error("SECTORS_API_KEY not set")
            return 0

        if self._symbols:
            symbols = self._symbols
        else:
            symbols = self.db.get_all_symbols()

        logger.info(f"Scraping financials for {len(symbols)} stocks")

        for symbol in symbols:
            try:
                records = await self.fetch_and_save_financials(symbol)
                count += records
                logger.debug(f"Saved {records} financial records for {symbol}")
            except Exception as e:
                logger.warning(f"Failed to scrape financials for {symbol}: {e}")

        return count

    async def fetch_and_save_financials(self, symbol: str) -> int:
        """Fetch and save all financial data for a symbol."""
        url = f"{self.SECTORS_BASE_URL}/companies/{symbol}/"
        response = await self._fetch_json(url)

        if not response:
            return 0

        count = 0
        financials = response.get("financials", {})
        historical = financials.get("historical_financials", [])

        for data in historical:
            year = data.get("year")
            if not year:
                continue

            period_end = date(year, 12, 31)

            income_stmt = self._extract_income_statement(symbol, year, period_end, data)
            if income_stmt:
                self._save_income_statement(income_stmt)
                count += 1

            balance_sheet = self._extract_balance_sheet(symbol, year, period_end, data)
            if balance_sheet:
                self._save_balance_sheet(balance_sheet)
                count += 1

            cash_flow = self._extract_cash_flow(symbol, year, period_end, data)
            if cash_flow:
                self._save_cash_flow(cash_flow)
                count += 1

            ratios = self._extract_ratios(symbol, year, period_end, data)
            if ratios:
                self._save_ratios(ratios)
                count += 1

        return count

    def _extract_income_statement(
        self,
        symbol: str,
        year: int,
        period_end: date,
        data: dict[str, Any],
    ) -> IncomeStatementData | None:
        revenue = data.get("revenue")
        earnings = data.get("earnings")

        if revenue is None and earnings is None:
            return None

        gross_profit = data.get("gross_profit")
        operating_income = data.get("operating_pnl")
        ebt = data.get("earnings_before_tax")
        tax = data.get("tax")

        gross_margin = None
        if gross_profit and revenue:
            gross_margin = Decimal(str(gross_profit / revenue))

        operating_margin = None
        if operating_income and revenue:
            operating_margin = Decimal(str(operating_income / revenue))

        net_margin = data.get("net_profit_margin")
        if net_margin:
            net_margin = Decimal(str(net_margin))

        eps = data.get("eps")
        if eps:
            eps = Decimal(str(eps))

        return IncomeStatementData(
            symbol=symbol,
            fiscal_year=year,
            fiscal_quarter=None,
            period_end=period_end,
            revenue=revenue,
            gross_profit=gross_profit,
            operating_income=operating_income,
            earnings_before_tax=ebt,
            tax_expense=tax,
            net_income=earnings,
            eps=eps,
            gross_margin=gross_margin,
            operating_margin=operating_margin,
            net_margin=net_margin,
            raw_data=data,
        )

    def _extract_balance_sheet(
        self,
        symbol: str,
        year: int,
        period_end: date,
        data: dict[str, Any],
    ) -> BalanceSheetData | None:
        total_assets = data.get("total_assets")
        total_equity = data.get("total_equity")

        if total_assets is None and total_equity is None:
            return None

        total_liabilities = data.get("total_liabilities")
        total_debt = data.get("total_debt")

        current_ratio = data.get("current_ratio")
        if current_ratio:
            current_ratio = Decimal(str(current_ratio))

        debt_to_equity = data.get("debt_to_equity")
        if debt_to_equity:
            debt_to_equity = Decimal(str(debt_to_equity))

        return BalanceSheetData(
            symbol=symbol,
            fiscal_year=year,
            fiscal_quarter=None,
            period_end=period_end,
            total_assets=total_assets,
            total_liabilities=total_liabilities,
            total_equity=total_equity,
            total_debt=total_debt,
            current_ratio=current_ratio,
            debt_to_equity=debt_to_equity,
            raw_data=data,
        )

    def _extract_cash_flow(
        self,
        symbol: str,
        year: int,
        period_end: date,
        data: dict[str, Any],
    ) -> CashFlowData | None:
        ocf = data.get("operating_cash_flow")
        fcf = data.get("free_cash_flow")
        capex = data.get("capex")

        if ocf is None and fcf is None:
            return None

        return CashFlowData(
            symbol=symbol,
            fiscal_year=year,
            fiscal_quarter=None,
            period_end=period_end,
            operating_cash_flow=ocf,
            capital_expenditure=capex,
            free_cash_flow=fcf,
            raw_data=data,
        )

    def _extract_ratios(
        self,
        symbol: str,
        year: int,
        period_end: date,
        data: dict[str, Any],
    ) -> FinancialRatiosData:
        def to_decimal(val: Any) -> Decimal | None:
            return Decimal(str(val)) if val is not None else None

        return FinancialRatiosData(
            symbol=symbol,
            fiscal_year=year,
            fiscal_quarter=None,
            period_end=period_end,
            roe=to_decimal(data.get("roe")),
            roa=to_decimal(data.get("roa")),
            gross_margin=to_decimal(data.get("gross_profit_margin")),
            operating_margin=to_decimal(data.get("operating_margin")),
            net_margin=to_decimal(data.get("net_profit_margin")),
            current_ratio=to_decimal(data.get("current_ratio")),
            debt_to_equity=to_decimal(data.get("debt_to_equity")),
            eps=to_decimal(data.get("eps")),
        )

    def _save_income_statement(self, stmt: IncomeStatementData) -> None:
        import json

        with self.db.cursor() as cur:
            cur.execute(
                """
                INSERT INTO income_statements 
                (symbol, fiscal_year, fiscal_quarter, period_end, revenue, gross_profit, 
                 operating_income, earnings_before_tax, tax_expense, net_income, eps,
                 gross_margin, operating_margin, net_margin, source, raw_data)
                VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s)
                ON CONFLICT (symbol, fiscal_year, fiscal_quarter) 
                DO UPDATE SET
                    revenue = EXCLUDED.revenue,
                    gross_profit = EXCLUDED.gross_profit,
                    operating_income = EXCLUDED.operating_income,
                    earnings_before_tax = EXCLUDED.earnings_before_tax,
                    tax_expense = EXCLUDED.tax_expense,
                    net_income = EXCLUDED.net_income,
                    eps = EXCLUDED.eps,
                    gross_margin = EXCLUDED.gross_margin,
                    operating_margin = EXCLUDED.operating_margin,
                    net_margin = EXCLUDED.net_margin,
                    raw_data = EXCLUDED.raw_data,
                    updated_at = NOW()
                """,
                (
                    stmt.symbol,
                    stmt.fiscal_year,
                    stmt.fiscal_quarter,
                    stmt.period_end,
                    stmt.revenue,
                    stmt.gross_profit,
                    stmt.operating_income,
                    stmt.earnings_before_tax,
                    stmt.tax_expense,
                    stmt.net_income,
                    stmt.eps,
                    stmt.gross_margin,
                    stmt.operating_margin,
                    stmt.net_margin,
                    "sectors",
                    json.dumps(stmt.raw_data) if stmt.raw_data else None,
                ),
            )

    def _save_balance_sheet(self, bs: BalanceSheetData) -> None:
        import json

        with self.db.cursor() as cur:
            cur.execute(
                """
                INSERT INTO balance_sheets 
                (symbol, fiscal_year, fiscal_quarter, period_end, total_assets, 
                 total_liabilities, total_equity, total_debt, current_ratio, 
                 debt_to_equity, source, raw_data)
                VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s)
                ON CONFLICT (symbol, fiscal_year, fiscal_quarter)
                DO UPDATE SET
                    total_assets = EXCLUDED.total_assets,
                    total_liabilities = EXCLUDED.total_liabilities,
                    total_equity = EXCLUDED.total_equity,
                    total_debt = EXCLUDED.total_debt,
                    current_ratio = EXCLUDED.current_ratio,
                    debt_to_equity = EXCLUDED.debt_to_equity,
                    raw_data = EXCLUDED.raw_data,
                    updated_at = NOW()
                """,
                (
                    bs.symbol,
                    bs.fiscal_year,
                    bs.fiscal_quarter,
                    bs.period_end,
                    bs.total_assets,
                    bs.total_liabilities,
                    bs.total_equity,
                    bs.total_debt,
                    bs.current_ratio,
                    bs.debt_to_equity,
                    "sectors",
                    json.dumps(bs.raw_data) if bs.raw_data else None,
                ),
            )

    def _save_cash_flow(self, cf: CashFlowData) -> None:
        import json

        with self.db.cursor() as cur:
            cur.execute(
                """
                INSERT INTO cash_flow_statements 
                (symbol, fiscal_year, fiscal_quarter, period_end, operating_cash_flow,
                 capital_expenditure, free_cash_flow, source, raw_data)
                VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s)
                ON CONFLICT (symbol, fiscal_year, fiscal_quarter)
                DO UPDATE SET
                    operating_cash_flow = EXCLUDED.operating_cash_flow,
                    capital_expenditure = EXCLUDED.capital_expenditure,
                    free_cash_flow = EXCLUDED.free_cash_flow,
                    raw_data = EXCLUDED.raw_data,
                    updated_at = NOW()
                """,
                (
                    cf.symbol,
                    cf.fiscal_year,
                    cf.fiscal_quarter,
                    cf.period_end,
                    cf.operating_cash_flow,
                    cf.capital_expenditure,
                    cf.free_cash_flow,
                    "sectors",
                    json.dumps(cf.raw_data) if cf.raw_data else None,
                ),
            )

    def _save_ratios(self, r: FinancialRatiosData) -> None:
        with self.db.cursor() as cur:
            cur.execute(
                """
                INSERT INTO financial_ratios 
                (symbol, fiscal_year, fiscal_quarter, period_end, roe, roa, 
                 gross_margin, operating_margin, net_margin, current_ratio,
                 debt_to_equity, eps, source)
                VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s)
                ON CONFLICT (symbol, fiscal_year, fiscal_quarter)
                DO UPDATE SET
                    roe = EXCLUDED.roe,
                    roa = EXCLUDED.roa,
                    gross_margin = EXCLUDED.gross_margin,
                    operating_margin = EXCLUDED.operating_margin,
                    net_margin = EXCLUDED.net_margin,
                    current_ratio = EXCLUDED.current_ratio,
                    debt_to_equity = EXCLUDED.debt_to_equity,
                    eps = EXCLUDED.eps
                """,
                (
                    r.symbol,
                    r.fiscal_year,
                    r.fiscal_quarter,
                    r.period_end,
                    r.roe,
                    r.roa,
                    r.gross_margin,
                    r.operating_margin,
                    r.net_margin,
                    r.current_ratio,
                    r.debt_to_equity,
                    r.eps,
                    "sectors",
                ),
            )


async def main() -> None:
    """CLI entrypoint for testing."""
    import asyncio
    import sys

    symbols = sys.argv[1:] if len(sys.argv) > 1 else None
    scraper = FinancialsScraper(symbols=symbols)
    await scraper.run()


if __name__ == "__main__":
    import asyncio

    asyncio.run(main())
