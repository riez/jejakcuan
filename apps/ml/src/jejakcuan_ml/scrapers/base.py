"""Base scraper class with common functionality."""

import asyncio
import random
import time
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from typing import Any

import httpx
from loguru import logger

from .database import DatabaseClient


@dataclass
class ScraperConfig:
    """Configuration for scrapers."""

    # Rate limiting
    requests_per_minute: int = 30
    min_delay: float = 1.0
    max_delay: float = 3.0

    # Retry settings
    max_retries: int = 3
    retry_delay: float = 5.0

    # HTTP settings
    timeout: float = 30.0
    user_agent: str = (
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 "
        "(KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
    )

    # Extra headers
    extra_headers: dict[str, str] = field(default_factory=dict)


class BaseScraper(ABC):
    """Base class for all scrapers."""

    def __init__(
        self,
        config: ScraperConfig | None = None,
        db_client: DatabaseClient | None = None,
    ) -> None:
        """Initialize scraper.

        Args:
            config: Scraper configuration
            db_client: Database client instance
        """
        self.config = config or ScraperConfig()
        self.db = db_client or DatabaseClient()
        self._last_request_time: float = 0
        self._request_count: int = 0
        self._client: httpx.AsyncClient | None = None

    def _build_headers(self) -> dict[str, str]:
        """Build HTTP request headers.

        Returns:
            Headers dictionary
        """
        headers = {
            "User-Agent": self.config.user_agent,
            "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            "Accept-Language": "en-US,en;q=0.5",
            "Accept-Encoding": "gzip, deflate",
            "Connection": "keep-alive",
        }
        headers.update(self.config.extra_headers)
        return headers

    async def _get_client(self) -> httpx.AsyncClient:
        """Get or create HTTP client.

        Returns:
            Async HTTP client
        """
        if self._client is None or self._client.is_closed:
            self._client = httpx.AsyncClient(
                timeout=self.config.timeout,
                headers=self._build_headers(),
                follow_redirects=True,
            )
        return self._client

    async def close(self) -> None:
        """Close HTTP client and database connection."""
        if self._client and not self._client.is_closed:
            await self._client.aclose()
        self.db.close()

    async def _rate_limit(self) -> None:
        """Apply rate limiting between requests."""
        now = time.time()
        elapsed = now - self._last_request_time

        # Calculate required delay
        min_interval = 60.0 / self.config.requests_per_minute
        if elapsed < min_interval:
            await asyncio.sleep(min_interval - elapsed)

        # Add random jitter
        jitter = random.uniform(self.config.min_delay, self.config.max_delay)
        await asyncio.sleep(jitter)

        self._last_request_time = time.time()
        self._request_count += 1

    async def _fetch_url(
        self,
        url: str,
        method: str = "GET",
        **kwargs: Any,
    ) -> httpx.Response | None:
        """Fetch URL with rate limiting and retry logic.

        Args:
            url: URL to fetch
            method: HTTP method
            **kwargs: Additional request arguments

        Returns:
            Response or None on failure
        """
        client = await self._get_client()

        for attempt in range(self.config.max_retries):
            try:
                await self._rate_limit()

                logger.debug(f"Fetching: {url} (attempt {attempt + 1})")
                response = await client.request(method, url, **kwargs)
                response.raise_for_status()

                return response

            except httpx.HTTPStatusError as e:
                logger.warning(f"HTTP error {e.response.status_code} for {url}")
                if e.response.status_code == 429:  # Rate limited
                    await asyncio.sleep(self.config.retry_delay * (attempt + 1))
                elif e.response.status_code >= 500:  # Server error
                    await asyncio.sleep(self.config.retry_delay)
                else:
                    break  # Client error, don't retry

            except httpx.RequestError as e:
                logger.warning(f"Request error for {url}: {e}")
                await asyncio.sleep(self.config.retry_delay)

        logger.error(f"Failed to fetch {url} after {self.config.max_retries} attempts")
        return None

    async def _fetch_json(self, url: str, **kwargs: Any) -> dict[str, Any] | None:
        """Fetch URL and parse JSON response.

        Args:
            url: URL to fetch
            **kwargs: Additional request arguments

        Returns:
            Parsed JSON or None
        """
        response = await self._fetch_url(url, **kwargs)
        if response is None:
            return None
        try:
            result: dict[str, Any] = response.json()
            return result
        except Exception as e:
            logger.error(f"Failed to parse JSON from {url}: {e}")
            return None

    @abstractmethod
    async def scrape(self) -> int:
        """Run the scraper.

        Returns:
            Number of records scraped
        """
        pass

    @abstractmethod
    def get_name(self) -> str:
        """Get scraper name.

        Returns:
            Scraper name
        """
        pass

    async def run(self) -> int:
        """Run scraper with proper setup and cleanup.

        Returns:
            Number of records scraped
        """
        logger.info(f"Starting {self.get_name()} scraper...")
        start_time = time.time()

        try:
            self.db.connect()
            count = await self.scrape()
            elapsed = time.time() - start_time
            logger.success(f"{self.get_name()}: Scraped {count} records in {elapsed:.1f}s")
            return count
        except Exception as e:
            logger.exception(f"{self.get_name()} scraper failed: {e}")
            raise
        finally:
            await self.close()
