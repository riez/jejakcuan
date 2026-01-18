"""News scraper for Indonesian financial news with optional browser automation."""

import re
from dataclasses import dataclass
from datetime import datetime
from typing import Any

from bs4 import BeautifulSoup
from loguru import logger

from .base import BaseScraper, ScraperConfig
from .database import DatabaseClient


@dataclass
class NewsItem:
    """News item data."""

    symbol: str
    title: str
    summary: str | None
    source: str
    url: str
    published_at: datetime
    keywords: list[str]


class NewsScraper(BaseScraper):
    """Scraper for stock news from Indonesian financial news sites."""

    SOURCES = {
        "kontan": "https://investasi.kontan.co.id",
        "bisnis": "https://market.bisnis.com",
    }

    KEYWORD_PATTERNS = [
        ("acquisition", ["akuisisi", "acquire", "acquisition"]),
        ("dividend", ["dividen", "dividend"]),
        ("rights_issue", ["right issue", "rights issue", "HMETD"]),
        ("earnings", ["laba", "profit", "earnings", "rugi", "loss"]),
        ("expansion", ["ekspansi", "expansion", "investasi"]),
        ("debt", ["utang", "debt", "obligasi", "bond"]),
    ]

    def __init__(
        self,
        config: ScraperConfig | None = None,
        db_client: DatabaseClient | None = None,
        symbols: list[str] | None = None,
        use_browser: bool = False,
    ) -> None:
        super().__init__(config, db_client)
        self._symbols = symbols
        self._use_browser = use_browser

    def get_name(self) -> str:
        return "News"

    async def scrape(self) -> int:
        """Scrape news for all symbols."""
        count = 0

        if self._symbols:
            symbols = self._symbols
        else:
            symbols = self.db.get_all_symbols()[:20]

        logger.info(f"Scraping news for {len(symbols)} stocks")

        for symbol in symbols:
            try:
                news_items = await self.fetch_news_for_stock(symbol)
                for item in news_items:
                    self._save_news(item)
                    count += 1
            except Exception as e:
                logger.warning(f"Failed to scrape news for {symbol}: {e}")

        return count

    async def fetch_news_for_stock(self, symbol: str) -> list[NewsItem]:
        """Fetch news for a specific stock."""
        news: list[NewsItem] = []

        if self._use_browser:
            news.extend(await self._fetch_with_browser(symbol))
        else:
            news.extend(await self._fetch_kontan(symbol))
            news.extend(await self._fetch_bisnis(symbol))

        return sorted(news, key=lambda n: n.published_at, reverse=True)[:10]

    async def _fetch_kontan(self, symbol: str) -> list[NewsItem]:
        """Fetch news from Kontan."""
        news: list[NewsItem] = []
        url = f"{self.SOURCES['kontan']}/search/?q={symbol}"

        response = await self._fetch_url(url)
        if not response:
            return news

        try:
            soup = BeautifulSoup(response.text, "html.parser")
            articles = soup.select(".list-news .news-item, .list-berita article")[:5]

            for article in articles:
                title_el = article.select_one("h3 a, .title a")
                date_el = article.select_one(".date, .time")

                if title_el:
                    title = title_el.get_text(strip=True)
                    href = str(title_el.get("href", ""))
                    pub_date = self._parse_indo_date(
                        date_el.get_text(strip=True) if date_el else None
                    )

                    news.append(
                        NewsItem(
                            symbol=symbol,
                            title=title,
                            summary=None,
                            source="kontan",
                            url=href if href.startswith("http") else f"https://kontan.co.id{href}",
                            published_at=pub_date,
                            keywords=self._extract_keywords(title),
                        )
                    )
        except Exception as e:
            logger.debug(f"Failed to parse Kontan news for {symbol}: {e}")

        return news

    async def _fetch_bisnis(self, symbol: str) -> list[NewsItem]:
        """Fetch news from Bisnis Indonesia."""
        news: list[NewsItem] = []
        url = f"{self.SOURCES['bisnis']}/search?q={symbol}"

        response = await self._fetch_url(url)
        if not response:
            return news

        try:
            soup = BeautifulSoup(response.text, "html.parser")
            articles = soup.select(".list-news article, .search-result-item")[:5]

            for article in articles:
                title_el = article.select_one("h2 a, .title a")
                date_el = article.select_one(".date, time")

                if title_el:
                    title = title_el.get_text(strip=True)
                    href = str(title_el.get("href", ""))
                    pub_date = self._parse_indo_date(
                        date_el.get_text(strip=True) if date_el else None
                    )

                    news.append(
                        NewsItem(
                            symbol=symbol,
                            title=title,
                            summary=None,
                            source="bisnis",
                            url=href if href.startswith("http") else f"https://bisnis.com{href}",
                            published_at=pub_date,
                            keywords=self._extract_keywords(title),
                        )
                    )
        except Exception as e:
            logger.debug(f"Failed to parse Bisnis news for {symbol}: {e}")

        return news

    async def _fetch_with_browser(self, symbol: str) -> list[NewsItem]:
        """Fetch news using Playwright browser automation."""
        news: list[NewsItem] = []

        try:
            from playwright.async_api import async_playwright

            async with async_playwright() as p:
                browser = await p.chromium.launch(headless=True)
                page = await browser.new_page()

                search_url = f"https://investasi.kontan.co.id/search/?q={symbol}"
                await page.goto(search_url)

                try:
                    await page.wait_for_selector(".list-news, .list-berita", timeout=10000)
                except Exception:
                    logger.debug(f"Timeout waiting for news list for {symbol}")
                    await browser.close()
                    return news

                articles = await page.query_selector_all(
                    ".list-news .news-item, .list-berita article"
                )

                for article in articles[:5]:
                    title_el = await article.query_selector("h3 a, .title a")
                    date_el = await article.query_selector(".date, .time")

                    if title_el:
                        title = await title_el.inner_text()
                        url = (await title_el.get_attribute("href")) or ""
                        pub_date_str = await date_el.inner_text() if date_el else None

                        news.append(
                            NewsItem(
                                symbol=symbol,
                                title=title,
                                summary=None,
                                source="kontan",
                                url=url if url.startswith("http") else f"https://kontan.co.id{url}",
                                published_at=self._parse_indo_date(pub_date_str),
                                keywords=self._extract_keywords(title),
                            )
                        )

                await browser.close()

        except ImportError:
            logger.warning(
                "Playwright not installed. Run: pip install playwright && playwright install"
            )
        except Exception as e:
            logger.warning(f"Browser automation failed for {symbol}: {e}")

        return news

    def _extract_keywords(self, text: str) -> list[str]:
        """Extract relevant keywords from news title."""
        keywords = []
        text_lower = text.lower()

        for keyword, triggers in self.KEYWORD_PATTERNS:
            if any(t.lower() in text_lower for t in triggers):
                keywords.append(keyword)

        return keywords

    def _parse_indo_date(self, date_str: str | None) -> datetime:
        """Parse Indonesian date string."""
        if not date_str:
            return datetime.now()

        try:
            date_str = date_str.strip().lower()

            if "hari" in date_str or "jam" in date_str or "menit" in date_str:
                return datetime.now()

            months: dict[str, int] = {
                "januari": 1,
                "februari": 2,
                "maret": 3,
                "april": 4,
                "mei": 5,
                "juni": 6,
                "juli": 7,
                "agustus": 8,
                "september": 9,
                "oktober": 10,
                "november": 11,
                "desember": 12,
                "jan": 1,
                "feb": 2,
                "mar": 3,
                "apr": 4,
                "may": 5,
                "jun": 6,
                "jul": 7,
                "aug": 8,
                "sep": 9,
                "oct": 10,
                "nov": 11,
                "dec": 12,
            }

            for month_name, month_num in months.items():
                if month_name in date_str:
                    match = re.search(r"(\d{1,2})\s*" + month_name + r"\s*(\d{4})?", date_str)
                    if match:
                        day = int(match.group(1))
                        year = int(match.group(2)) if match.group(2) else datetime.now().year
                        return datetime(year, month_num, day)

        except Exception:
            pass

        return datetime.now()

    def _save_news(self, item: NewsItem) -> None:
        """Save news item to database."""
        self.db.execute(
            """
            INSERT INTO stock_news 
            (symbol, title, summary, source, url, published_at, keywords)
            VALUES (%s, %s, %s, %s, %s, %s, %s)
            ON CONFLICT DO NOTHING
            """,
            (
                item.symbol,
                item.title,
                item.summary,
                item.source,
                item.url,
                item.published_at,
                item.keywords,
            ),
        )
