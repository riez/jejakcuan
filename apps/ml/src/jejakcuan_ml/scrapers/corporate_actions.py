"""Corporate actions scraper for IDX stocks."""

from dataclasses import dataclass
from datetime import date, datetime
from typing import Any

from loguru import logger

from .base import BaseScraper, ScraperConfig
from .database import DatabaseClient


@dataclass
class CorporateActionData:
    """Corporate action data."""

    symbol: str
    action_type: str
    announced_date: date
    effective_date: date | None
    ex_date: date | None
    description: str
    value: float | None
    status: str


class CorporateActionsScraper(BaseScraper):
    """Scraper for corporate actions from IDX."""

    IDX_CORPORATE_ACTIONS = "https://www.idx.co.id/primary/ListedCompany/GetCorporateAction"

    def __init__(
        self,
        config: ScraperConfig | None = None,
        db_client: DatabaseClient | None = None,
        symbols: list[str] | None = None,
    ) -> None:
        super().__init__(config, db_client)
        self._symbols = symbols

    def get_name(self) -> str:
        return "Corporate Actions"

    async def scrape(self) -> int:
        """Scrape corporate actions."""
        count = 0

        if self._symbols:
            symbols = self._symbols
        else:
            symbols = self.db.get_all_symbols()

        logger.info(f"Scraping corporate actions for {len(symbols)} stocks")

        for symbol in symbols:
            try:
                actions = await self.fetch_corporate_actions(symbol)
                for action in actions:
                    self._save_action(action)
                    count += 1
            except Exception as e:
                logger.warning(f"Failed to scrape corporate actions for {symbol}: {e}")

        return count

    async def fetch_corporate_actions(
        self,
        symbol: str | None = None,
        from_date: date | None = None,
    ) -> list[CorporateActionData]:
        """Fetch corporate actions for a stock."""
        actions: list[CorporateActionData] = []

        params: dict[str, str | int] = {"page": 1, "pageSize": 50}
        if symbol:
            params["code"] = symbol
        if from_date:
            params["fromDate"] = from_date.isoformat()

        response = await self._fetch_json(self.IDX_CORPORATE_ACTIONS, params=params)
        if response:
            for item in response.get("Results", []):
                action = self._parse_action(item)
                if action:
                    actions.append(action)

        return actions

    def _parse_action(self, item: dict[str, Any]) -> CorporateActionData | None:
        """Parse a corporate action from IDX API response."""
        symbol = item.get("Code", "")
        if not symbol:
            return None

        return CorporateActionData(
            symbol=symbol,
            action_type=self._normalize_action_type(item.get("Type", "")),
            announced_date=self._parse_date(item.get("AnnouncedDate")) or date.today(),
            effective_date=self._parse_date(item.get("EffectiveDate")),
            ex_date=self._parse_date(item.get("ExDate")),
            description=item.get("Description", ""),
            value=item.get("Value"),
            status=item.get("Status", "announced"),
        )

    def _normalize_action_type(self, raw_type: str) -> str:
        """Normalize corporate action type."""
        mapping = {
            "Cash Dividend": "dividend",
            "Stock Dividend": "stock_dividend",
            "Stock Split": "stock_split",
            "Rights Issue": "rights_issue",
            "Bonus Shares": "bonus_shares",
        }
        return mapping.get(raw_type, raw_type.lower().replace(" ", "_"))

    def _parse_date(self, date_str: str | None) -> date | None:
        """Parse date string to date object."""
        if not date_str:
            return None
        try:
            return datetime.fromisoformat(date_str.replace("Z", "+00:00")).date()
        except (ValueError, AttributeError):
            return None

    def _save_action(self, action: CorporateActionData) -> None:
        """Save corporate action to database."""
        self.db.execute(
            """
            INSERT INTO corporate_actions 
            (symbol, action_type, announced_date, effective_date, ex_date, description, value, status)
            VALUES (%s, %s, %s, %s, %s, %s, %s, %s)
            ON CONFLICT DO NOTHING
            """,
            (
                action.symbol,
                action.action_type,
                action.announced_date,
                action.effective_date,
                action.ex_date,
                action.description,
                action.value,
                action.status,
            ),
        )
