"""IDX (Indonesia Stock Exchange) fundamental data scraper.

Sources:
- IDX website: https://www.idx.co.id
- StockBit API for statistics
- Financial reports from IDX

References:
- https://github.com/noczero/idx-fundamental-analysis
- https://github.com/basnugroho/indonesia-stocks-scraper
"""

import asyncio
import re
from dataclasses import dataclass
from datetime import date, datetime
from decimal import Decimal
from typing import Any

from bs4 import BeautifulSoup
from loguru import logger

from .base import BaseScraper, ScraperConfig
from .database import DatabaseClient


@dataclass
class FinancialData:
    """Financial data for a stock."""

    symbol: str
    period_end: date

    # Income statement
    revenue: Decimal | None = None
    net_income: Decimal | None = None
    ebitda: Decimal | None = None

    # Balance sheet
    total_assets: Decimal | None = None
    total_equity: Decimal | None = None
    total_debt: Decimal | None = None

    # Cash flow
    free_cash_flow: Decimal | None = None

    # Per share
    eps: Decimal | None = None
    book_value_per_share: Decimal | None = None

    # Ratios
    pe_ratio: Decimal | None = None
    pb_ratio: Decimal | None = None
    ev_ebitda: Decimal | None = None
    roe: Decimal | None = None
    roa: Decimal | None = None


@dataclass
class StockInfo:
    """Basic stock information from IDX."""

    symbol: str
    name: str
    sector: str | None = None
    subsector: str | None = None
    listing_date: date | None = None
    market_cap: int | None = None
    shares_outstanding: int | None = None


class IDXScraper(BaseScraper):
    """Scraper for IDX fundamental data."""

    IDX_BASE = "https://www.idx.co.id"
    STOCKBIT_API = "https://api.stockbit.com"

    def __init__(
        self,
        config: ScraperConfig | None = None,
        db_client: DatabaseClient | None = None,
        symbols: list[str] | None = None,
    ) -> None:
        """Initialize IDX scraper.

        Args:
            config: Scraper configuration
            db_client: Database client
            symbols: Specific symbols to scrape (None for all)
        """
        super().__init__(config, db_client)
        self._symbols = symbols

    def get_name(self) -> str:
        """Get scraper name."""
        return "IDX Fundamental"

    async def scrape(self) -> int:
        """Scrape IDX fundamental data.

        Returns:
            Number of records scraped
        """
        count = 0

        # Get symbols to scrape
        if self._symbols:
            symbols = self._symbols
        else:
            # Get all Syariah stocks from IDX
            symbols = await self._fetch_issi_stocks()
            if not symbols:
                # Fallback to database symbols
                symbols = self.db.get_all_symbols()

        logger.info(f"Scraping fundamentals for {len(symbols)} stocks")

        # Scrape each stock
        for i, symbol in enumerate(symbols):
            logger.info(f"[{i+1}/{len(symbols)}] Scraping {symbol}")
            try:
                # Get stock info and financial data
                info = await self._fetch_stock_info(symbol)
                if info:
                    self._save_stock_info(info)

                # Get financial statements
                financials = await self._fetch_financials(symbol)
                if financials:
                    for fin in financials:
                        self._save_financials(fin)
                    count += len(financials)

                # Get key statistics
                stats = await self._fetch_statistics(symbol)
                if stats:
                    self._save_financials(stats)
                    count += 1

            except Exception as e:
                logger.warning(f"Failed to scrape {symbol}: {e}")

        return count

    async def _fetch_issi_stocks(self) -> list[str]:
        """Fetch ISSI (Indonesia Sharia Stock Index) constituent list.

        Returns:
            List of Syariah stock symbols
        """
        symbols: list[str] = []

        # Try IDX API for ISSI constituents
        url = f"{self.IDX_BASE}/primary/ListedCompany/GetListedCompany"
        params = {"sharia": "true", "page": 1, "pageSize": 500}

        response = await self._fetch_url(url, params=params)
        if response:
            try:
                data = response.json()
                for item in data.get("Results", []):
                    symbol = item.get("Code", "")
                    if symbol:
                        symbols.append(symbol)
            except Exception as e:
                logger.debug(f"Failed to parse ISSI list: {e}")

        # Alternative: Scrape ISSI page
        if not symbols:
            url = f"{self.IDX_BASE}/en/data/stock-index/ISSI"
            response = await self._fetch_url(url)
            if response:
                soup = BeautifulSoup(response.text, "html.parser")
                # Look for stock codes in the page
                for link in soup.select("a[href*='/en/company/']"):
                    symbol = link.get_text(strip=True)
                    if re.match(r"^[A-Z]{4}$", symbol):
                        symbols.append(symbol)

        # Fallback: Known major Syariah stocks
        if not symbols:
            symbols = [
                # Banking
                "BBCA",
                "BMRI",
                "BBRI",
                "BBNI",
                "BRIS",
                "BTPN",
                "ARTO",
                # Telecom
                "TLKM",
                "EXCL",
                "ISAT",
                "TOWR",
                "TBIG",
                # Mining
                "ADRO",
                "ITMG",
                "PTBA",
                "ANTM",
                "INCO",
                "MDKA",
                "MEDC",
                # Consumer
                "UNVR",
                "ICBP",
                "INDF",
                "MYOR",
                "KLBF",
                "SIDO",
                # Property
                "BSDE",
                "CTRA",
                "SMRA",
                "PWON",
                # Industrial
                "ASII",
                "UNTR",
                "SMSM",
                "SMGR",
                "INTP",
                # Tech
                "GOTO",
                "BUKA",
                "EMTK",
                "DCII",
                # Infrastructure
                "JSMR",
                "WIKA",
                "PTPP",
            ]

        logger.info(f"Found {len(symbols)} ISSI stocks")
        return symbols

    async def _fetch_stock_info(self, symbol: str) -> StockInfo | None:
        """Fetch basic stock info from IDX.

        Args:
            symbol: Stock symbol

        Returns:
            Stock info or None
        """
        # Try IDX company profile API
        url = f"{self.IDX_BASE}/primary/ListedCompany/GetCompanyProfile"
        params = {"code": symbol}

        response = await self._fetch_url(url, params=params)
        if response:
            try:
                data = response.json()
                return StockInfo(
                    symbol=symbol,
                    name=data.get("Name", symbol),
                    sector=data.get("Sector"),
                    subsector=data.get("SubSector"),
                    listing_date=self._parse_date(data.get("ListingDate")),
                    shares_outstanding=data.get("SharesOutstanding"),
                )
            except Exception as e:
                logger.debug(f"Failed to parse stock info for {symbol}: {e}")

        # Fallback: Scrape company page
        url = f"{self.IDX_BASE}/en/company/{symbol}"
        response = await self._fetch_url(url)
        if response:
            soup = BeautifulSoup(response.text, "html.parser")
            name = soup.select_one("h1, .company-name")
            if name:
                return StockInfo(symbol=symbol, name=name.get_text(strip=True))

        return None

    async def _fetch_financials(self, symbol: str) -> list[FinancialData]:
        """Fetch financial statements from IDX.

        Args:
            symbol: Stock symbol

        Returns:
            List of financial data for different periods
        """
        financials: list[FinancialData] = []

        # Try IDX financial statements API
        url = f"{self.IDX_BASE}/primary/ListedCompany/GetFinancialStatements"
        params = {"code": symbol}

        response = await self._fetch_url(url, params=params)
        if response:
            try:
                data = response.json()
                for item in data.get("Results", []):
                    fin = self._parse_financial_statement(symbol, item)
                    if fin:
                        financials.append(fin)
            except Exception as e:
                logger.debug(f"Failed to parse financials for {symbol}: {e}")

        # Try alternative: Download financial report
        if not financials:
            fin = await self._fetch_latest_financial_report(symbol)
            if fin:
                financials.append(fin)

        return financials

    async def _fetch_latest_financial_report(self, symbol: str) -> FinancialData | None:
        """Try to fetch the latest financial report.

        Args:
            symbol: Stock symbol

        Returns:
            Financial data or None
        """
        # Try IDX financial report page
        year = datetime.now().year
        quarters = ["Q4", "Q3", "Q2", "Q1"]

        for q in quarters:
            path = f"/StaticData/NewsAndAnnouncement/INDEXANNOUNCEMENT/{symbol}_{year}_{q}.pdf"
            url = f"{self.IDX_BASE}{path}"
            response = await self._fetch_url(url, method="HEAD")
            if response and response.status_code == 200:
                logger.info(f"Found financial report: {symbol} {year} {q}")
                # We would need PDF parsing here
                break

        return None

    async def _fetch_statistics(self, symbol: str) -> FinancialData | None:
        """Fetch key statistics from StockBit API.

        Args:
            symbol: Stock symbol

        Returns:
            Financial data with ratios
        """
        # StockBit fundamental API
        url = f"{self.STOCKBIT_API}/v1/companies/{symbol}/fundamental"

        # Add StockBit headers
        headers = {
            "Accept": "application/json",
            "Origin": "https://stockbit.com",
        }

        client = await self._get_client()
        try:
            await self._rate_limit()
            response = await client.get(url, headers=headers)
            if response.status_code == 200:
                data = response.json()
                return self._parse_stockbit_fundamental(symbol, data)
        except Exception as e:
            logger.debug(f"Failed to fetch StockBit data for {symbol}: {e}")

        # Alternative: Try Yahoo Finance for basic ratios
        return await self._fetch_yfinance_stats(symbol)

    async def _fetch_yfinance_stats(self, symbol: str) -> FinancialData | None:
        """Fetch statistics from Yahoo Finance.

        Args:
            symbol: Stock symbol

        Returns:
            Financial data or None
        """
        # Yahoo Finance uses .JK suffix for Indonesian stocks
        yf_symbol = f"{symbol}.JK"

        # Yahoo Finance modules API
        url = "https://query1.finance.yahoo.com/v10/finance/quoteSummary"
        params = {
            "symbol": yf_symbol,
            "modules": "defaultKeyStatistics,financialData,summaryDetail",
        }

        client = await self._get_client()
        try:
            await self._rate_limit()
            response = await client.get(url, params=params)
            if response.status_code == 200:
                data = response.json()
                result = data.get("quoteSummary", {}).get("result", [])
                if result:
                    return self._parse_yfinance_stats(symbol, result[0])
        except Exception as e:
            logger.debug(f"Failed to fetch Yahoo Finance data for {symbol}: {e}")

        return None

    def _parse_financial_statement(self, symbol: str, data: dict[str, Any]) -> FinancialData | None:
        """Parse IDX financial statement data.

        Args:
            symbol: Stock symbol
            data: Financial statement data

        Returns:
            Parsed financial data or None
        """
        period_end = self._parse_date(data.get("ReportDate"))
        if not period_end:
            return None

        return FinancialData(
            symbol=symbol,
            period_end=period_end,
            revenue=self._to_decimal(data.get("Revenue")),
            net_income=self._to_decimal(data.get("NetIncome")),
            total_assets=self._to_decimal(data.get("TotalAssets")),
            total_equity=self._to_decimal(data.get("TotalEquity")),
            total_debt=self._to_decimal(data.get("TotalDebt")),
            ebitda=self._to_decimal(data.get("EBITDA")),
            eps=self._to_decimal(data.get("EPS")),
            book_value_per_share=self._to_decimal(data.get("BookValuePerShare")),
        )

    def _parse_stockbit_fundamental(
        self, symbol: str, data: dict[str, Any]
    ) -> FinancialData | None:
        """Parse StockBit fundamental data.

        Args:
            symbol: Stock symbol
            data: StockBit API response

        Returns:
            Parsed financial data or None
        """
        fund_data = data.get("data", {})
        if not fund_data:
            return None

        # Use latest quarter
        today = date.today()
        quarter_end = date(today.year, ((today.month - 1) // 3) * 3 + 1, 1)

        return FinancialData(
            symbol=symbol,
            period_end=quarter_end,
            pe_ratio=self._to_decimal(fund_data.get("pe")),
            pb_ratio=self._to_decimal(fund_data.get("pbv")),
            ev_ebitda=self._to_decimal(fund_data.get("ev_ebitda")),
            roe=self._to_decimal(fund_data.get("roe")),
            roa=self._to_decimal(fund_data.get("roa")),
            eps=self._to_decimal(fund_data.get("eps")),
        )

    def _parse_yfinance_stats(self, symbol: str, data: dict[str, Any]) -> FinancialData | None:
        """Parse Yahoo Finance statistics.

        Args:
            symbol: Stock symbol
            data: Yahoo Finance response

        Returns:
            Parsed financial data or None
        """
        key_stats = data.get("defaultKeyStatistics", {})
        fin_data = data.get("financialData", {})
        summary = data.get("summaryDetail", {})

        today = date.today()
        quarter_end = date(today.year, ((today.month - 1) // 3) * 3 + 1, 1)

        return FinancialData(
            symbol=symbol,
            period_end=quarter_end,
            pe_ratio=self._to_decimal(self._get_raw(summary, "trailingPE")),
            pb_ratio=self._to_decimal(self._get_raw(key_stats, "priceToBook")),
            ev_ebitda=self._to_decimal(self._get_raw(key_stats, "enterpriseToEbitda")),
            roe=self._to_decimal(self._get_raw(fin_data, "returnOnEquity")),
            roa=self._to_decimal(self._get_raw(fin_data, "returnOnAssets")),
            eps=self._to_decimal(self._get_raw(key_stats, "trailingEps")),
            revenue=self._to_decimal(self._get_raw(fin_data, "totalRevenue")),
            ebitda=self._to_decimal(self._get_raw(fin_data, "ebitda")),
            total_debt=self._to_decimal(self._get_raw(fin_data, "totalDebt")),
            free_cash_flow=self._to_decimal(self._get_raw(fin_data, "freeCashflow")),
        )

    def _get_raw(self, data: dict[str, Any], key: str) -> Any:
        """Get raw value from Yahoo Finance nested structure.

        Args:
            data: Data dictionary
            key: Key to look for

        Returns:
            Raw value or None
        """
        item = data.get(key, {})
        if isinstance(item, dict):
            return item.get("raw")
        return item

    def _parse_date(self, text: Any) -> date | None:
        """Parse date from various formats.

        Args:
            text: Date string or timestamp

        Returns:
            Parsed date or None
        """
        if not text:
            return None

        if isinstance(text, int):
            # Unix timestamp
            return datetime.fromtimestamp(text / 1000).date()

        if isinstance(text, str):
            formats = ["%Y-%m-%d", "%d/%m/%Y", "%Y%m%d", "%d-%m-%Y"]
            for fmt in formats:
                try:
                    return datetime.strptime(text, fmt).date()
                except ValueError:
                    continue

        return None

    def _to_decimal(self, value: Any) -> Decimal | None:
        """Convert value to Decimal.

        Args:
            value: Value to convert

        Returns:
            Decimal or None
        """
        if value is None:
            return None
        try:
            return Decimal(str(value))
        except (ValueError, TypeError):
            return None

    def _save_stock_info(self, info: StockInfo) -> None:
        """Save stock info to database.

        Args:
            info: Stock info to save
        """
        self.db.upsert_stock(
            symbol=info.symbol,
            name=info.name,
            sector=info.sector,
            subsector=info.subsector,
            listing_date=info.listing_date,
            market_cap=info.market_cap,
        )

    def _save_financials(self, fin: FinancialData) -> None:
        """Save financial data to database.

        Args:
            fin: Financial data to save
        """
        # Build kwargs from non-None fields
        kwargs: dict[str, Any] = {}

        fields = [
            "revenue",
            "net_income",
            "ebitda",
            "total_assets",
            "total_equity",
            "total_debt",
            "free_cash_flow",
            "eps",
            "book_value_per_share",
            "pe_ratio",
            "pb_ratio",
            "ev_ebitda",
            "roe",
            "roa",
        ]

        for field in fields:
            value = getattr(fin, field)
            if value is not None:
                kwargs[field] = value

        if kwargs:
            self.db.upsert_financials(fin.symbol, fin.period_end, **kwargs)


async def main() -> None:
    """Run IDX scraper as CLI command."""
    import sys

    logger.remove()
    logger.add(sys.stderr, level="INFO")

    # Parse command line arguments
    symbols = None
    if len(sys.argv) > 1:
        symbols = sys.argv[1:]
        logger.info(f"Scraping specific symbols: {symbols}")

    scraper = IDXScraper(symbols=symbols)
    try:
        count = await scraper.run()
        logger.info(f"Completed: {count} financial records scraped")
    except Exception as e:
        logger.error(f"Scraper failed: {e}")
        sys.exit(1)


if __name__ == "__main__":
    asyncio.run(main())
