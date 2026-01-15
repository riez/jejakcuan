"""e-IPO scraper for upcoming Indonesian IPO listings.

Source: https://e-ipo.co.id
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
class IPOListing:
    """Represents an IPO listing."""

    symbol: str
    company_name: str
    sector: str | None = None
    subsector: str | None = None
    ipo_price_low: Decimal | None = None
    ipo_price_high: Decimal | None = None
    shares_offered: int | None = None
    book_building_start: date | None = None
    book_building_end: date | None = None
    offering_start: date | None = None
    offering_end: date | None = None
    listing_date: date | None = None
    status: str = "upcoming"  # upcoming, ongoing, completed


class EIPOScraper(BaseScraper):
    """Scraper for e-ipo.co.id IPO listings."""

    BASE_URL = "https://e-ipo.co.id"

    def __init__(
        self,
        config: ScraperConfig | None = None,
        db_client: DatabaseClient | None = None,
    ) -> None:
        """Initialize e-IPO scraper."""
        super().__init__(config, db_client)

    def get_name(self) -> str:
        """Get scraper name."""
        return "e-IPO"

    async def scrape(self) -> int:
        """Scrape e-IPO listings.

        Returns:
            Number of records scraped
        """
        count = 0

        # Scrape upcoming IPOs
        upcoming = await self._scrape_upcoming()
        count += len(upcoming)

        # Scrape completed IPOs for historical data
        completed = await self._scrape_completed()
        count += len(completed)

        # Save to database
        all_listings = upcoming + completed
        for listing in all_listings:
            try:
                self._save_listing(listing)
            except Exception as e:
                logger.warning(f"Failed to save listing {listing.symbol}: {e}")

        return count

    async def _scrape_upcoming(self) -> list[IPOListing]:
        """Scrape upcoming IPO listings.

        Returns:
            List of upcoming IPO listings
        """
        listings: list[IPOListing] = []

        # Main IPO page
        response = await self._fetch_url(f"{self.BASE_URL}/ipo")
        if response is None:
            logger.warning("Failed to fetch e-IPO main page")
            return listings

        soup = BeautifulSoup(response.text, "html.parser")

        # Parse IPO cards/listings from the page
        ipo_cards = soup.select(".ipo-card, .card, [data-ipo]")
        if not ipo_cards:
            # Try alternative selectors
            ipo_cards = soup.select("article, .listing-item, .ipo-item")

        for card in ipo_cards:
            try:
                listing = self._parse_ipo_card(card)
                if listing:
                    listings.append(listing)
                    logger.info(f"Found upcoming IPO: {listing.symbol} - {listing.company_name}")
            except Exception as e:
                logger.debug(f"Failed to parse IPO card: {e}")

        # Also try to fetch from API endpoint if available
        api_listings = await self._scrape_api()
        listings.extend(api_listings)

        return listings

    async def _scrape_api(self) -> list[IPOListing]:
        """Try to fetch from e-IPO API endpoints.

        Returns:
            List of IPO listings from API
        """
        listings: list[IPOListing] = []

        # Common API endpoints to try
        api_endpoints = [
            f"{self.BASE_URL}/api/ipo/list",
            f"{self.BASE_URL}/api/v1/ipo",
            f"{self.BASE_URL}/ajax/ipo-list",
        ]

        for endpoint in api_endpoints:
            try:
                data = await self._fetch_json(endpoint)
                if data and isinstance(data, dict):
                    items = data.get("data") or data.get("ipos") or data.get("items") or []
                    for item in items:
                        listing = self._parse_api_item(item)
                        if listing:
                            listings.append(listing)
                    if listings:
                        logger.info(f"Found {len(listings)} listings from API")
                        break
            except Exception as e:
                logger.debug(f"API endpoint {endpoint} failed: {e}")

        return listings

    async def _scrape_completed(self) -> list[IPOListing]:
        """Scrape completed IPO listings.

        Returns:
            List of completed IPO listings
        """
        listings: list[IPOListing] = []

        # Try to fetch completed IPOs page
        response = await self._fetch_url(f"{self.BASE_URL}/ipo/completed")
        if response is None:
            response = await self._fetch_url(f"{self.BASE_URL}/ipo?status=completed")

        if response is None:
            return listings

        soup = BeautifulSoup(response.text, "html.parser")

        # Parse completed listings
        completed_items = soup.select(".completed-ipo, .ipo-completed, table tbody tr")
        for item in completed_items:
            try:
                listing = self._parse_completed_item(item)
                if listing:
                    listing.status = "completed"
                    listings.append(listing)
            except Exception as e:
                logger.debug(f"Failed to parse completed IPO: {e}")

        return listings

    def _parse_ipo_card(self, card: Any) -> IPOListing | None:
        """Parse an IPO card element.

        Args:
            card: BeautifulSoup element

        Returns:
            IPO listing or None
        """
        # Try to extract company name and symbol
        title_elem = card.select_one("h3, h4, .title, .company-name, [data-symbol]")
        if not title_elem:
            return None

        text = title_elem.get_text(strip=True)

        # Extract symbol (usually in format "ABCD" or "(ABCD)")
        symbol_match = re.search(r"\(([A-Z]{4})\)|^([A-Z]{4})$|([A-Z]{4})\s*-", text)
        if not symbol_match:
            # Try data attribute
            symbol = card.get("data-symbol", "")
            if not symbol:
                return None
        else:
            symbol = next(g for g in symbol_match.groups() if g)

        # Clean company name
        company_name = re.sub(r"\([A-Z]{4}\)", "", text).strip()
        if not company_name:
            company_name = symbol

        listing = IPOListing(
            symbol=symbol.upper(),
            company_name=company_name,
        )

        # Extract dates
        date_elems = card.select(".date, [data-date], time")
        for elem in date_elems:
            date_text = elem.get_text(strip=True)
            date_type = elem.get("data-type", elem.get("class", [""])[0])

            parsed_date = self._parse_date(date_text)
            if parsed_date:
                if "listing" in str(date_type).lower():
                    listing.listing_date = parsed_date
                elif "offer" in str(date_type).lower():
                    listing.offering_start = parsed_date

        # Extract price
        price_elem = card.select_one(".price, [data-price]")
        if price_elem:
            price_text = price_elem.get_text(strip=True)
            prices = self._parse_price_range(price_text)
            if prices:
                listing.ipo_price_low, listing.ipo_price_high = prices

        # Extract sector
        sector_elem = card.select_one(".sector, [data-sector]")
        if sector_elem:
            listing.sector = sector_elem.get_text(strip=True)

        return listing

    def _parse_api_item(self, item: dict[str, Any]) -> IPOListing | None:
        """Parse an API response item.

        Args:
            item: API response dictionary

        Returns:
            IPO listing or None
        """
        symbol = item.get("symbol", item.get("ticker", item.get("code")))
        if not symbol:
            return None

        listing = IPOListing(
            symbol=symbol.upper(),
            company_name=item.get("name", item.get("company_name", symbol)),
            sector=item.get("sector"),
            subsector=item.get("subsector", item.get("industry")),
        )

        # Parse prices
        if "price_low" in item:
            listing.ipo_price_low = Decimal(str(item["price_low"]))
        if "price_high" in item:
            listing.ipo_price_high = Decimal(str(item["price_high"]))
        if "price" in item and not listing.ipo_price_low:
            listing.ipo_price_low = Decimal(str(item["price"]))
            listing.ipo_price_high = listing.ipo_price_low

        # Parse dates
        for date_field, attr in [
            ("listing_date", "listing_date"),
            ("book_building_start", "book_building_start"),
            ("book_building_end", "book_building_end"),
            ("offering_start", "offering_start"),
            ("offering_end", "offering_end"),
        ]:
            if date_field in item:
                parsed = self._parse_date(item[date_field])
                if parsed:
                    setattr(listing, attr, parsed)

        # Parse shares
        if "shares_offered" in item:
            listing.shares_offered = int(item["shares_offered"])

        listing.status = item.get("status", "upcoming")

        return listing

    def _parse_completed_item(self, item: Any) -> IPOListing | None:
        """Parse a completed IPO table row.

        Args:
            item: BeautifulSoup table row element

        Returns:
            IPO listing or None
        """
        cells = item.select("td")
        if len(cells) < 3:
            return None

        # Typically: Symbol, Company Name, Listing Date, Price
        symbol_text = cells[0].get_text(strip=True)
        symbol_match = re.search(r"([A-Z]{4})", symbol_text.upper())
        if not symbol_match:
            return None

        symbol = symbol_match.group(1)
        company_name = cells[1].get_text(strip=True) if len(cells) > 1 else symbol

        listing = IPOListing(
            symbol=symbol,
            company_name=company_name,
            status="completed",
        )

        # Parse listing date
        if len(cells) > 2:
            listing.listing_date = self._parse_date(cells[2].get_text(strip=True))

        # Parse price
        if len(cells) > 3:
            prices = self._parse_price_range(cells[3].get_text(strip=True))
            if prices:
                listing.ipo_price_low, listing.ipo_price_high = prices

        return listing

    def _parse_date(self, text: str) -> date | None:
        """Parse date from various formats.

        Args:
            text: Date string

        Returns:
            Parsed date or None
        """
        if not text:
            return None

        # Common date formats
        formats = [
            "%Y-%m-%d",
            "%d/%m/%Y",
            "%d-%m-%Y",
            "%d %b %Y",
            "%d %B %Y",
            "%B %d, %Y",
        ]

        text = text.strip()
        for fmt in formats:
            try:
                return datetime.strptime(text, fmt).date()
            except ValueError:
                continue

        return None

    def _parse_price_range(self, text: str) -> tuple[Decimal, Decimal] | None:
        """Parse price range from text.

        Args:
            text: Price text (e.g., "Rp 100 - 150" or "100-150")

        Returns:
            Tuple of (low, high) prices or None
        """
        if not text:
            return None

        # Remove currency symbols and whitespace
        text = re.sub(r"[Rp.,\s]", "", text)

        # Look for range pattern
        range_match = re.search(r"(\d+)-(\d+)", text)
        if range_match:
            low = Decimal(range_match.group(1))
            high = Decimal(range_match.group(2))
            return (low, high)

        # Single price
        single_match = re.search(r"(\d+)", text)
        if single_match:
            price = Decimal(single_match.group(1))
            return (price, price)

        return None

    def _save_listing(self, listing: IPOListing) -> None:
        """Save IPO listing to database.

        Args:
            listing: IPO listing to save
        """
        # Insert/update stock record
        self.db.upsert_stock(
            symbol=listing.symbol,
            name=listing.company_name,
            sector=listing.sector,
            subsector=listing.subsector,
            listing_date=listing.listing_date,
            is_active=listing.status == "completed",
        )

        logger.info(f"Saved IPO listing: {listing.symbol}")


async def main() -> None:
    """Run e-IPO scraper as CLI command."""
    import sys

    logger.remove()
    logger.add(sys.stderr, level="INFO")

    scraper = EIPOScraper()
    try:
        count = await scraper.run()
        logger.info(f"Completed: {count} IPO listings scraped")
    except Exception as e:
        logger.error(f"Scraper failed: {e}")
        sys.exit(1)


if __name__ == "__main__":
    asyncio.run(main())
