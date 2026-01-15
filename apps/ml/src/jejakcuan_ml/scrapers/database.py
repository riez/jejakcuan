"""Database client for scrapers."""

import os
from collections.abc import Generator
from contextlib import contextmanager
from datetime import date, datetime
from decimal import Decimal
from typing import Any

import psycopg2
import psycopg2.extras
from loguru import logger


class DatabaseClient:
    """PostgreSQL database client for scrapers."""

    def __init__(
        self,
        host: str | None = None,
        port: int | None = None,
        database: str | None = None,
        user: str | None = None,
        password: str | None = None,
    ) -> None:
        """Initialize database client.

        Args:
            host: Database host
            port: Database port
            database: Database name
            user: Database user
            password: Database password
        """
        self.host = host or os.environ.get("JEJAKCUAN_DB_HOST", "localhost")
        self.port = port or int(os.environ.get("JEJAKCUAN_DB_PORT", "5432"))
        self.database = database or os.environ.get("JEJAKCUAN_DB_NAME", "jejakcuan")
        self.user = user or os.environ.get("JEJAKCUAN_DB_USER", "jejakcuan")
        self.password = password or os.environ.get("JEJAKCUAN_DB_PASSWORD", "jejakcuan_dev")
        self._conn: psycopg2.extensions.connection | None = None

    def connect(self) -> None:
        """Establish database connection."""
        if self._conn is None or self._conn.closed:
            logger.info(f"Connecting to database {self.database}@{self.host}:{self.port}")
            self._conn = psycopg2.connect(
                host=self.host,
                port=self.port,
                database=self.database,
                user=self.user,
                password=self.password,
            )

    def close(self) -> None:
        """Close database connection."""
        if self._conn and not self._conn.closed:
            self._conn.close()
            self._conn = None
            logger.info("Database connection closed")

    @contextmanager
    def cursor(self) -> Generator[psycopg2.extensions.cursor, None, None]:
        """Get a database cursor with auto-commit.

        Yields:
            Database cursor
        """
        self.connect()
        assert self._conn is not None
        cursor = self._conn.cursor(cursor_factory=psycopg2.extras.RealDictCursor)
        try:
            yield cursor
            self._conn.commit()
        except Exception:
            self._conn.rollback()
            raise
        finally:
            cursor.close()

    def get_all_symbols(self) -> list[str]:
        """Get all active stock symbols from database.

        Returns:
            List of stock symbols
        """
        with self.cursor() as cur:
            cur.execute("SELECT symbol FROM stocks WHERE is_active = true ORDER BY symbol")
            rows = cur.fetchall()
            return [str(row[0]) for row in rows]

    def get_stock_by_symbol(self, symbol: str) -> dict[str, Any] | None:
        """Get stock info by symbol.

        Args:
            symbol: Stock symbol

        Returns:
            Stock data or None
        """
        with self.cursor() as cur:
            cur.execute("SELECT * FROM stocks WHERE symbol = %s", (symbol,))
            return cur.fetchone()  # type: ignore[return-value]

    def upsert_stock(
        self,
        symbol: str,
        name: str,
        sector: str | None = None,
        subsector: str | None = None,
        listing_date: date | None = None,
        market_cap: int | None = None,
        is_active: bool = True,
    ) -> None:
        """Insert or update stock.

        Args:
            symbol: Stock symbol
            name: Company name
            sector: Sector
            subsector: Subsector
            listing_date: IPO listing date
            market_cap: Market capitalization
            is_active: Whether stock is active
        """
        with self.cursor() as cur:
            cur.execute(
                """
                INSERT INTO stocks (
                    symbol, name, sector, subsector, listing_date, market_cap, is_active
                ) VALUES (%s, %s, %s, %s, %s, %s, %s)
                ON CONFLICT (symbol) DO UPDATE SET
                    name = EXCLUDED.name,
                    sector = COALESCE(EXCLUDED.sector, stocks.sector),
                    subsector = COALESCE(EXCLUDED.subsector, stocks.subsector),
                    listing_date = COALESCE(EXCLUDED.listing_date, stocks.listing_date),
                    market_cap = COALESCE(EXCLUDED.market_cap, stocks.market_cap),
                    is_active = EXCLUDED.is_active,
                    updated_at = NOW()
                """,
                (symbol, name, sector, subsector, listing_date, market_cap, is_active),
            )
            logger.debug(f"Upserted stock: {symbol}")

    def insert_price(
        self,
        symbol: str,
        time: datetime,
        open_: Decimal,
        high: Decimal,
        low: Decimal,
        close: Decimal,
        volume: int,
        value: Decimal | None = None,
        frequency: int | None = None,
    ) -> None:
        """Insert stock price data.

        Args:
            symbol: Stock symbol
            time: Price timestamp
            open_: Open price
            high: High price
            low: Low price
            close: Close price
            volume: Trading volume
            value: Trading value
            frequency: Trade frequency
        """
        with self.cursor() as cur:
            cur.execute(
                """
                INSERT INTO stock_prices (
                    time, symbol, open, high, low, close, volume, value, frequency
                ) VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s)
                ON CONFLICT DO NOTHING
                """,
                (time, symbol, open_, high, low, close, volume, value, frequency),
            )

    def insert_prices_batch(self, prices: list[dict[str, Any]]) -> int:
        """Insert multiple price records efficiently.

        Args:
            prices: List of price dictionaries

        Returns:
            Number of records inserted
        """
        if not prices:
            return 0

        with self.cursor() as cur:
            # Use execute_values for efficient batch insert
            psycopg2.extras.execute_values(
                cur,
                """
                INSERT INTO stock_prices (
                    time, symbol, open, high, low, close, volume, value, frequency
                ) VALUES %s
                ON CONFLICT DO NOTHING
                """,
                [
                    (
                        p["time"],
                        p["symbol"],
                        p["open"],
                        p["high"],
                        p["low"],
                        p["close"],
                        p["volume"],
                        p.get("value"),
                        p.get("frequency"),
                    )
                    for p in prices
                ],
            )
            return cur.rowcount

    def insert_broker_summary(
        self,
        symbol: str,
        time: datetime,
        broker_code: str,
        buy_volume: int,
        sell_volume: int,
        buy_value: Decimal,
        sell_value: Decimal,
    ) -> None:
        """Insert broker summary data.

        Args:
            symbol: Stock symbol
            time: Summary timestamp
            broker_code: Broker code
            buy_volume: Buy volume
            sell_volume: Sell volume
            buy_value: Buy value
            sell_value: Sell value
        """
        with self.cursor() as cur:
            cur.execute(
                """
                INSERT INTO broker_summary (
                    time, symbol, broker_code, buy_volume, sell_volume,
                    buy_value, sell_value
                ) VALUES (%s, %s, %s, %s, %s, %s, %s)
                ON CONFLICT DO NOTHING
                """,
                (time, symbol, broker_code, buy_volume, sell_volume, buy_value, sell_value),
            )

    def insert_broker_summary_batch(self, summaries: list[dict[str, Any]]) -> int:
        """Insert multiple broker summary records efficiently.

        Args:
            summaries: List of broker summary dictionaries

        Returns:
            Number of records inserted
        """
        if not summaries:
            return 0

        with self.cursor() as cur:
            psycopg2.extras.execute_values(
                cur,
                """
                INSERT INTO broker_summary (
                    time, symbol, broker_code, buy_volume, sell_volume,
                    buy_value, sell_value
                ) VALUES %s
                ON CONFLICT DO NOTHING
                """,
                [
                    (
                        s["time"],
                        s["symbol"],
                        s["broker_code"],
                        s["buy_volume"],
                        s["sell_volume"],
                        s["buy_value"],
                        s["sell_value"],
                    )
                    for s in summaries
                ],
            )
            return cur.rowcount

    def upsert_financials(
        self,
        symbol: str,
        period_end: date,
        **kwargs: Any,
    ) -> None:
        """Insert or update financial data.

        Args:
            symbol: Stock symbol
            period_end: Period end date
            **kwargs: Financial metrics (revenue, net_income, etc.)
        """
        columns = ["symbol", "period_end"] + list(kwargs.keys())
        values = [symbol, period_end] + list(kwargs.values())
        placeholders = ", ".join(["%s"] * len(values))
        column_names = ", ".join(columns)

        # Build update clause excluding primary keys
        update_cols = [k for k in kwargs.keys()]
        update_clause = ", ".join([f"{col} = EXCLUDED.{col}" for col in update_cols])

        with self.cursor() as cur:
            cur.execute(
                f"""
                INSERT INTO financials ({column_names})
                VALUES ({placeholders})
                ON CONFLICT (symbol, period_end) DO UPDATE SET
                    {update_clause}
                """,
                values,
            )
            logger.debug(f"Upserted financials: {symbol} - {period_end}")

    def get_latest_price_date(self, symbol: str) -> date | None:
        """Get the latest price date for a symbol.

        Args:
            symbol: Stock symbol

        Returns:
            Latest price date or None
        """
        with self.cursor() as cur:
            cur.execute(
                "SELECT MAX(time)::date as latest FROM stock_prices WHERE symbol = %s",
                (symbol,),
            )
            result = cur.fetchone()
            if result and result[0] is not None:
                return result[0]  # type: ignore[no-any-return]
            return None

    def get_stock_count(self) -> int:
        """Get total number of stocks.

        Returns:
            Stock count
        """
        with self.cursor() as cur:
            cur.execute("SELECT COUNT(*) as count FROM stocks")
            result = cur.fetchone()
            if result and result[0] is not None:
                return int(result[0])
            return 0
