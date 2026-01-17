"""Broker flow data scraper.

Sources:
- Indopremier API
- StockBit broker summary
- IDX broker data

Broker codes reference:
- XL: Deutsche Bank
- AK: UBS
- YU: Merrill Lynch
- CC: Mandiri Sekuritas
- BK: JP Morgan
- ZP: Maybank
- KZ: CLSA
- CS: Credit Suisse
"""

import asyncio
from dataclasses import dataclass
from datetime import UTC, date, datetime, timedelta
from decimal import Decimal
from typing import Any

from bs4 import BeautifulSoup
from loguru import logger

from .base import BaseScraper, ScraperConfig
from .database import DatabaseClient


@dataclass
class BrokerTransaction:
    """Represents broker buy/sell activity for a stock."""

    symbol: str
    trade_date: datetime
    broker_code: str
    buy_volume: int = 0
    sell_volume: int = 0
    buy_value: Decimal = Decimal("0")
    sell_value: Decimal = Decimal("0")


class BrokerFlowScraper(BaseScraper):
    """Scraper for broker flow (buy/sell) data."""

    INDOPREMIER_BASE = "https://www.indopremier.com"
    STOCKBIT_API = "https://api.stockbit.com"
    IDX_BASE = "https://www.idx.co.id"

    def __init__(
        self,
        config: ScraperConfig | None = None,
        db_client: DatabaseClient | None = None,
        symbols: list[str] | None = None,
        days: int = 30,
    ) -> None:
        """Initialize broker flow scraper.

        Args:
            config: Scraper configuration
            db_client: Database client
            symbols: Specific symbols to scrape (None for all)
            days: Number of days to fetch (default 30)
        """
        super().__init__(config, db_client)
        self._symbols = symbols
        self._days = days

    def get_name(self) -> str:
        """Get scraper name."""
        return "Broker Flow"

    async def scrape(self) -> int:
        """Scrape broker flow data.

        Returns:
            Number of records scraped
        """
        count = 0

        # Get symbols to scrape
        if self._symbols:
            symbols = self._symbols
        else:
            symbols = self.db.get_all_symbols()

        logger.info(f"Scraping broker flow for {len(symbols)} stocks ({self._days} days)")

        # Calculate date range
        end_date = datetime.now()
        start_date = end_date - timedelta(days=self._days)

        # Scrape each stock
        for i, symbol in enumerate(symbols):
            logger.info(f"[{i+1}/{len(symbols)}] Scraping broker flow for {symbol}")
            try:
                transactions = await self._fetch_broker_flow(symbol, start_date, end_date)
                if transactions:
                    batch = [
                        {
                            "time": t.trade_date,
                            "symbol": t.symbol,
                            "broker_code": t.broker_code,
                            "buy_volume": t.buy_volume,
                            "sell_volume": t.sell_volume,
                            "buy_value": t.buy_value,
                            "sell_value": t.sell_value,
                        }
                        for t in transactions
                    ]
                    inserted = self.db.insert_broker_summary_batch(batch)
                    count += inserted
                    logger.debug(f"Inserted {inserted} broker records for {symbol}")

            except Exception as e:
                logger.warning(f"Failed to scrape broker flow for {symbol}: {e}")

        return count

    async def _fetch_broker_flow(
        self,
        symbol: str,
        start_date: datetime,
        end_date: datetime,
    ) -> list[BrokerTransaction]:
        """Fetch broker flow data from multiple sources.

        Args:
            symbol: Stock symbol
            start_date: Start date
            end_date: End date

        Returns:
            List of broker transactions
        """
        transactions: list[BrokerTransaction] = []

        # Try Indopremier broker summary (HTML) first
        indopremier_data = await self._fetch_indopremier(symbol, start_date, end_date)
        transactions.extend(indopremier_data)

        # Try StockBit API as backup
        if not transactions:
            stockbit_data = await self._fetch_stockbit(symbol, start_date, end_date)
            transactions.extend(stockbit_data)

        # Try IDX broker data as last resort
        if not transactions:
            idx_data = await self._fetch_idx_broker(symbol, start_date, end_date)
            transactions.extend(idx_data)

        return transactions

    async def _fetch_indopremier(
        self,
        symbol: str,
        start_date: datetime,
        end_date: datetime,
    ) -> list[BrokerTransaction]:
        """Fetch broker data from Indopremier broker summary (HTML).

        Args:
            symbol: Stock symbol
            start_date: Start date
            end_date: End date

        Returns:
            List of broker transactions
        """
        results: dict[tuple[date, str], BrokerTransaction] = {}

        current_day = start_date.date()
        end_day = end_date.date()

        while current_day <= end_day:
            day_txs = await self._fetch_indopremier_broker_summary_day(symbol, current_day)
            for tx in day_txs:
                key = (tx.trade_date.date(), tx.broker_code)
                existing = results.get(key)
                if existing is None:
                    results[key] = tx
                else:
                    existing.buy_volume += tx.buy_volume
                    existing.sell_volume += tx.sell_volume
                    existing.buy_value += tx.buy_value
                    existing.sell_value += tx.sell_value

            current_day += timedelta(days=1)

        return list(results.values())

    async def _fetch_indopremier_broker_summary_day(
        self,
        symbol: str,
        trade_day: date,
    ) -> list[BrokerTransaction]:
        """Fetch broker summary for a single day from Indopremier.

        The endpoint returns an HTML table with top buyers/sellers.
        We parse the table and emit broker transactions for that day.
        """
        url = f"{self.INDOPREMIER_BASE}/module/saham/include/data-brokersummary.php"
        date_str = trade_day.strftime("%m/%d/%Y")
        params = {
            "code": symbol,
            "start": date_str,
            "end": date_str,
            "fd": "all",
            "board": "RG",
        }

        response = await self._fetch_url(url, params=params)
        if response is None:
            return []

        soup = BeautifulSoup(response.text, "html.parser")
        table = soup.select_one("table.table-summary")
        if not table:
            return []

        def parse_int(text: str) -> int:
            cleaned = text.replace(",", "").replace(" ", "").strip()
            return int(cleaned) if cleaned else 0

        def parse_value(text: str) -> Decimal:
            cleaned = text.replace(",", "").strip()
            if not cleaned or cleaned == "-":
                return Decimal("0")

            parts = cleaned.split()
            num = Decimal(parts[0])
            mult = Decimal("1")
            if len(parts) > 1:
                suffix = parts[1].upper()
                if suffix == "T":
                    mult = Decimal("1000000000000")
                elif suffix == "B":
                    mult = Decimal("1000000000")
                elif suffix == "M":
                    mult = Decimal("1000000")
                elif suffix == "K":
                    mult = Decimal("1000")
            return num * mult

        trade_dt = datetime.combine(trade_day, datetime.min.time(), tzinfo=UTC)
        txs: dict[str, BrokerTransaction] = {}

        for row in table.select("tbody tr"):
            cells = row.find_all("td")
            if len(cells) < 9:
                continue

            buyer_code = cells[0].get_text(strip=True).upper()
            buy_lot = parse_int(cells[1].get_text(strip=True))
            buy_val = parse_value(cells[2].get_text(strip=True))

            seller_code = cells[5].get_text(strip=True).upper()
            sell_lot = parse_int(cells[6].get_text(strip=True))
            sell_val = parse_value(cells[7].get_text(strip=True))

            if buyer_code:
                tx = txs.get(buyer_code)
                if tx is None:
                    tx = BrokerTransaction(
                        symbol=symbol,
                        trade_date=trade_dt,
                        broker_code=buyer_code[:4],
                    )
                    txs[buyer_code] = tx
                tx.buy_volume += buy_lot * 100
                tx.buy_value += buy_val

            if seller_code:
                tx = txs.get(seller_code)
                if tx is None:
                    tx = BrokerTransaction(
                        symbol=symbol,
                        trade_date=trade_dt,
                        broker_code=seller_code[:4],
                    )
                    txs[seller_code] = tx
                tx.sell_volume += sell_lot * 100
                tx.sell_value += sell_val

        return [t for t in txs.values() if t.buy_volume or t.sell_volume]

    async def _fetch_stockbit(
        self,
        symbol: str,
        start_date: datetime,
        end_date: datetime,
    ) -> list[BrokerTransaction]:
        """Fetch broker data from StockBit.

        Args:
            symbol: Stock symbol
            start_date: Start date
            end_date: End date

        Returns:
            List of broker transactions
        """
        transactions: list[BrokerTransaction] = []

        # StockBit broker summary API
        headers = {
            "Accept": "application/json",
            "Origin": "https://stockbit.com",
            "Referer": f"https://stockbit.com/symbol/{symbol}",
        }

        # Fetch day by day within range
        current = start_date
        while current <= end_date:
            url = f"{self.STOCKBIT_API}/v1/companies/{symbol}/broker-summary"
            params = {"date": current.strftime("%Y-%m-%d")}

            client = await self._get_client()
            try:
                await self._rate_limit()
                response = await client.get(url, headers=headers, params=params)
                if response.status_code == 200:
                    data = response.json()
                    for item in data.get("data", {}).get("brokers", []):
                        tx = self._parse_stockbit_item(symbol, current, item)
                        if tx:
                            transactions.append(tx)
            except Exception as e:
                logger.debug(f"StockBit broker fetch failed for {symbol}: {e}")

            current += timedelta(days=1)

        return transactions

    async def _fetch_idx_broker(
        self,
        symbol: str,
        start_date: datetime,
        end_date: datetime,
    ) -> list[BrokerTransaction]:
        """Fetch broker data from IDX.

        Args:
            symbol: Stock symbol
            start_date: Start date
            end_date: End date

        Returns:
            List of broker transactions
        """
        transactions: list[BrokerTransaction] = []

        # IDX broker transaction API
        url = f"{self.IDX_BASE}/primary/TradingSummary/GetBrokerSummary"
        params = {"code": symbol}

        data = await self._fetch_json(url, params=params)
        if data and "Results" in data:
            for item in data["Results"]:
                tx = self._parse_idx_item(symbol, item)
                if tx and start_date <= tx.trade_date <= end_date:
                    transactions.append(tx)

        return transactions

    def _parse_indopremier_item(
        self, symbol: str, item: dict[str, Any]
    ) -> BrokerTransaction | None:
        """Parse Indopremier broker data item.

        Args:
            symbol: Stock symbol
            item: API response item

        Returns:
            Broker transaction or None
        """
        try:
            trade_date = datetime.strptime(item["date"], "%Y-%m-%d")
            broker_code = item.get("broker", "")
            if not broker_code:
                return None

            return BrokerTransaction(
                symbol=symbol,
                trade_date=trade_date,
                broker_code=broker_code[:4].upper(),
                buy_volume=int(item.get("buy_volume", 0)),
                sell_volume=int(item.get("sell_volume", 0)),
                buy_value=Decimal(str(item.get("buy_value", 0))),
                sell_value=Decimal(str(item.get("sell_value", 0))),
            )
        except (KeyError, ValueError) as e:
            logger.debug(f"Failed to parse Indopremier item: {e}")
            return None

    def _parse_stockbit_item(
        self, symbol: str, trade_date: datetime, item: dict[str, Any]
    ) -> BrokerTransaction | None:
        """Parse StockBit broker data item.

        Args:
            symbol: Stock symbol
            trade_date: Trade date
            item: API response item

        Returns:
            Broker transaction or None
        """
        try:
            broker_code = item.get("code", item.get("broker_code", ""))
            if not broker_code:
                return None

            return BrokerTransaction(
                symbol=symbol,
                trade_date=trade_date,
                broker_code=broker_code[:4].upper(),
                buy_volume=int(item.get("bvol", item.get("buy_vol", 0))),
                sell_volume=int(item.get("svol", item.get("sell_vol", 0))),
                buy_value=Decimal(str(item.get("bval", item.get("buy_val", 0)))),
                sell_value=Decimal(str(item.get("sval", item.get("sell_val", 0)))),
            )
        except (KeyError, ValueError) as e:
            logger.debug(f"Failed to parse StockBit item: {e}")
            return None

    def _parse_idx_item(self, symbol: str, item: dict[str, Any]) -> BrokerTransaction | None:
        """Parse IDX broker data item.

        Args:
            symbol: Stock symbol
            item: API response item

        Returns:
            Broker transaction or None
        """
        try:
            date_str = item.get("TradingDate", item.get("Date", ""))
            if not date_str:
                return None

            trade_date = datetime.strptime(date_str[:10], "%Y-%m-%d")
            broker_code = item.get("BrokerCode", "")
            if not broker_code:
                return None

            return BrokerTransaction(
                symbol=symbol,
                trade_date=trade_date,
                broker_code=broker_code[:4].upper(),
                buy_volume=int(item.get("BuyVolume", 0)),
                sell_volume=int(item.get("SellVolume", 0)),
                buy_value=Decimal(str(item.get("BuyValue", 0))),
                sell_value=Decimal(str(item.get("SellValue", 0))),
            )
        except (KeyError, ValueError) as e:
            logger.debug(f"Failed to parse IDX item: {e}")
            return None


async def main() -> None:
    """Run broker flow scraper as CLI command."""
    import sys

    logger.remove()
    logger.add(sys.stderr, level="INFO")

    # Parse command line arguments
    symbols = None
    days = 30

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
        logger.info(f"Scraping broker flow for: {symbols}")

    scraper = BrokerFlowScraper(symbols=symbols, days=days)
    try:
        count = await scraper.run()
        logger.info(f"Completed: {count} broker flow records scraped")
    except Exception as e:
        logger.error(f"Scraper failed: {e}")
        sys.exit(1)


if __name__ == "__main__":
    asyncio.run(main())
