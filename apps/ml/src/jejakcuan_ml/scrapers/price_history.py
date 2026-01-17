"""Historical stock price scraper.

Sources:
- Yahoo Finance (primary)
- IDX API
- Investing.com (backup)
"""

import asyncio
from dataclasses import dataclass
from datetime import UTC, date, datetime, timedelta
from decimal import Decimal

from loguru import logger

from .base import BaseScraper, ScraperConfig
from .database import DatabaseClient


@dataclass
class PriceBar:
    """OHLCV price bar."""

    symbol: str
    time: datetime
    open: Decimal
    high: Decimal
    low: Decimal
    close: Decimal
    volume: int
    value: Decimal | None = None
    frequency: int | None = None


class PriceHistoryScraper(BaseScraper):
    """Scraper for historical stock prices."""

    YAHOO_FINANCE_API = "https://query1.finance.yahoo.com/v8/finance/chart"
    IDX_API = "https://www.idx.co.id/primary"

    def __init__(
        self,
        config: ScraperConfig | None = None,
        db_client: DatabaseClient | None = None,
        symbols: list[str] | None = None,
        days: int = 365,
    ) -> None:
        """Initialize price history scraper.

        Args:
            config: Scraper configuration
            db_client: Database client
            symbols: Specific symbols to scrape (None for all)
            days: Number of days of history to fetch (default 365)
        """
        super().__init__(config, db_client)
        self._symbols = symbols
        self._days = days

    def get_name(self) -> str:
        """Get scraper name."""
        return "Price History"

    async def scrape(self) -> int:
        """Scrape historical price data.

        Returns:
            Number of records scraped
        """
        count = 0

        # Get symbols to scrape
        if self._symbols:
            symbols = self._symbols
        else:
            symbols = self.db.get_all_symbols()

        logger.info(f"Scraping price history for {len(symbols)} stocks ({self._days} days)")

        # Scrape each stock
        for i, symbol in enumerate(symbols):
            logger.info(f"[{i+1}/{len(symbols)}] Scraping prices for {symbol}")
            try:
                # Check latest price date in database
                latest_date = self.db.get_latest_price_date(symbol)
                desired_start = date.today() - timedelta(days=self._days)

                if latest_date:
                    earliest_date = self.db.get_earliest_price_date(symbol)
                    needs_backfill = earliest_date is None or earliest_date > desired_start

                    if needs_backfill:
                        start_date = desired_start
                        if earliest_date is not None:
                            logger.info(
                                f"Backfilling {symbol} price history from {desired_start} "
                                f"(earliest existing: {earliest_date})"
                            )
                    else:
                        # Only fetch missing days
                        start_date = latest_date + timedelta(days=1)
                        if start_date > date.today():
                            logger.debug(f"{symbol} already up to date")
                            continue
                else:
                    # Fetch full history
                    start_date = desired_start

                end_date = date.today()

                # Fetch prices
                prices = await self._fetch_prices(symbol, start_date, end_date)
                if prices:
                    batch = [
                        {
                            "time": p.time,
                            "symbol": p.symbol,
                            "open": p.open,
                            "high": p.high,
                            "low": p.low,
                            "close": p.close,
                            "volume": p.volume,
                            "value": p.value,
                            "frequency": p.frequency,
                        }
                        for p in prices
                    ]
                    inserted = self.db.insert_prices_batch(batch)
                    count += inserted
                    logger.info(f"Inserted {inserted} price records for {symbol}")

            except Exception as e:
                logger.warning(f"Failed to scrape prices for {symbol}: {e}")

        return count

    async def _fetch_prices(
        self,
        symbol: str,
        start_date: date,
        end_date: date,
    ) -> list[PriceBar]:
        """Fetch price history from available sources.

        Args:
            symbol: Stock symbol
            start_date: Start date
            end_date: End date

        Returns:
            List of price bars
        """
        # Try Yahoo Finance first (most reliable for IDX stocks)
        prices = await self._fetch_yahoo_finance(symbol, start_date, end_date)

        # Fallback to IDX API
        if not prices:
            prices = await self._fetch_idx_api(symbol, start_date, end_date)

        return prices

    async def _fetch_yahoo_finance(
        self,
        symbol: str,
        start_date: date,
        end_date: date,
    ) -> list[PriceBar]:
        """Fetch prices from Yahoo Finance using yfinance library.

        Args:
            symbol: Stock symbol
            start_date: Start date
            end_date: End date

        Returns:
            List of price bars
        """
        import yfinance as yf

        prices: list[PriceBar] = []

        # Yahoo Finance uses .JK suffix for Indonesian stocks
        yf_symbol = f"{symbol}.JK"

        try:
            ticker = yf.Ticker(yf_symbol)
            df = ticker.history(start=start_date, end=end_date)

            if df.empty:
                logger.debug(f"No data from Yahoo Finance for {symbol}")
                return prices

            for idx, row in df.iterrows():
                ts = idx.to_pydatetime().replace(tzinfo=UTC)
                prices.append(
                    PriceBar(
                        symbol=symbol,
                        time=ts,
                        open=Decimal(str(round(row["Open"], 2))),
                        high=Decimal(str(round(row["High"], 2))),
                        low=Decimal(str(round(row["Low"], 2))),
                        close=Decimal(str(round(row["Close"], 2))),
                        volume=int(row["Volume"]),
                    )
                )

            logger.debug(f"Fetched {len(prices)} prices for {symbol} from Yahoo Finance")

        except Exception as e:
            logger.warning(f"Yahoo Finance error for {symbol}: {e}")

        return prices

    async def _fetch_yahoo_finance_old(
        self,
        symbol: str,
        start_date: date,
        end_date: date,
    ) -> list[PriceBar]:
        """Fetch prices from Yahoo Finance API (fallback).

        Args:
            symbol: Stock symbol
            start_date: Start date
            end_date: End date

        Returns:
            List of price bars
        """
        prices: list[PriceBar] = []

        # Yahoo Finance uses .JK suffix for Indonesian stocks
        yf_symbol = f"{symbol}.JK"

        # Convert dates to Unix timestamps
        period1 = int(datetime.combine(start_date, datetime.min.time()).timestamp())
        period2 = int(datetime.combine(end_date, datetime.max.time()).timestamp())

        url = f"{self.YAHOO_FINANCE_API}/{yf_symbol}"
        params = {
            "period1": period1,
            "period2": period2,
            "interval": "1d",
            "includePrePost": "false",
            "events": "history",
        }

        data = await self._fetch_json(url, params=params)
        if not data:
            return prices

        try:
            chart = data.get("chart", {})
            result = chart.get("result", [])
            if not result:
                return prices

            result = result[0]
            timestamps = result.get("timestamp", [])
            indicators = result.get("indicators", {})
            quote = indicators.get("quote", [{}])[0]

            opens = quote.get("open", [])
            highs = quote.get("high", [])
            lows = quote.get("low", [])
            closes = quote.get("close", [])
            volumes = quote.get("volume", [])

            for i, ts in enumerate(timestamps):
                # Skip if any value is None
                if any(
                    v is None or (i < len(v) and v[i] is None)
                    for v in [opens, highs, lows, closes, volumes]
                ):
                    continue

                if i >= len(opens) or i >= len(closes):
                    continue

                dt = datetime.fromtimestamp(ts, tz=UTC)

                price = PriceBar(
                    symbol=symbol,
                    time=dt,
                    open=Decimal(str(opens[i])),
                    high=Decimal(str(highs[i])),
                    low=Decimal(str(lows[i])),
                    close=Decimal(str(closes[i])),
                    volume=int(volumes[i]) if volumes[i] else 0,
                )
                prices.append(price)

            logger.debug(f"Fetched {len(prices)} prices from Yahoo Finance for {symbol}")

        except Exception as e:
            logger.debug(f"Failed to parse Yahoo Finance data for {symbol}: {e}")

        return prices

    async def _fetch_idx_api(
        self,
        symbol: str,
        start_date: date,
        end_date: date,
    ) -> list[PriceBar]:
        """Fetch prices from IDX API.

        Args:
            symbol: Stock symbol
            start_date: Start date
            end_date: End date

        Returns:
            List of price bars
        """
        prices: list[PriceBar] = []

        url = f"{self.IDX_API}/StockData/GetStockData"
        params = {
            "code": symbol,
            "start": start_date.strftime("%Y%m%d"),
            "end": end_date.strftime("%Y%m%d"),
        }

        data = await self._fetch_json(url, params=params)
        if not data or "Results" not in data:
            return prices

        try:
            for item in data["Results"]:
                dt = datetime.strptime(item["Date"], "%Y-%m-%d")

                price = PriceBar(
                    symbol=symbol,
                    time=dt,
                    open=Decimal(str(item.get("OpenPrice", item.get("Open", 0)))),
                    high=Decimal(str(item.get("High", 0))),
                    low=Decimal(str(item.get("Low", 0))),
                    close=Decimal(str(item.get("ClosePrice", item.get("Close", 0)))),
                    volume=int(item.get("Volume", 0)),
                    value=Decimal(str(item.get("Value", 0))) if item.get("Value") else None,
                    frequency=int(item.get("Frequency", 0)) if item.get("Frequency") else None,
                )
                prices.append(price)

            logger.debug(f"Fetched {len(prices)} prices from IDX API for {symbol}")

        except Exception as e:
            logger.debug(f"Failed to parse IDX API data for {symbol}: {e}")

        return prices


async def main() -> None:
    """Run price history scraper as CLI command."""
    import sys

    logger.remove()
    logger.add(sys.stderr, level="INFO")

    # Parse command line arguments
    symbols = None
    days = 365

    args = sys.argv[1:]
    i = 0
    while i < len(args):
        arg = args[i]
        if arg == "--days" and i + 1 < len(args):
            days = int(args[i + 1])
            i += 2
        elif not arg.startswith("--"):
            if symbols is None:
                symbols = []
            symbols.append(arg)
            i += 1
        else:
            i += 1

    if symbols:
        logger.info(f"Scraping price history for: {symbols}")

    scraper = PriceHistoryScraper(symbols=symbols, days=days)
    try:
        count = await scraper.run()
        logger.info(f"Completed: {count} price records scraped")
    except Exception as e:
        logger.error(f"Scraper failed: {e}")
        sys.exit(1)


if __name__ == "__main__":
    asyncio.run(main())
