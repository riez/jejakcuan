"""Stockbit API client for social sentiment data."""

import re
from dataclasses import dataclass
from datetime import datetime, timedelta

import httpx

from .models import PostType, StockbitPost, StockbitUser, StreamItem, SymbolSentiment


@dataclass
class StockbitConfig:
    """Stockbit client configuration."""

    base_url: str = "https://stockbit.com/api"
    timeout: float = 30.0
    # Auth tokens if available (improves rate limits)
    access_token: str | None = None


class StockbitClient:
    """Client for Stockbit social data.

    Note: This client uses Stockbit's web API endpoints.
    Ensure compliance with their ToS and rate limits.
    """

    def __init__(self, config: StockbitConfig | None = None) -> None:
        """Initialize client.

        Args:
            config: Client configuration
        """
        self.config = config or StockbitConfig()
        self._client = httpx.AsyncClient(
            base_url=self.config.base_url,
            timeout=self.config.timeout,
            headers=self._build_headers(),
        )

    def _build_headers(self) -> dict[str, str]:
        """Build request headers."""
        headers = {
            "User-Agent": "Mozilla/5.0 (compatible; JejakCuan/1.0)",
            "Accept": "application/json",
        }
        if self.config.access_token:
            headers["Authorization"] = f"Bearer {self.config.access_token}"
        return headers

    async def close(self) -> None:
        """Close the HTTP client."""
        await self._client.aclose()

    async def get_symbol_stream(
        self,
        symbol: str,
        limit: int = 50,
        offset: int = 0,
    ) -> list[StockbitPost]:
        """Get stream posts for a specific symbol.

        Args:
            symbol: Stock symbol (e.g., "BBCA")
            limit: Max posts to return
            offset: Pagination offset

        Returns:
            List of posts mentioning the symbol
        """
        try:
            response = await self._client.get(
                f"/stream/symbol/{symbol}",
                params={"limit": limit, "offset": offset},
            )
            response.raise_for_status()
            data = response.json()

            posts = []
            for item in data.get("data", []):
                post = self._parse_post(item)
                if post:
                    posts.append(post)

            return posts

        except httpx.HTTPError as e:
            print(f"Error fetching stream for {symbol}: {e}")
            return []

    async def get_trending_stream(self, limit: int = 50) -> list[StockbitPost]:
        """Get trending posts across all symbols.

        Args:
            limit: Max posts to return

        Returns:
            List of trending posts
        """
        try:
            response = await self._client.get(
                "/stream/trending",
                params={"limit": limit},
            )
            response.raise_for_status()
            data = response.json()

            posts = []
            for item in data.get("data", []):
                post = self._parse_post(item)
                if post:
                    posts.append(post)

            return posts

        except httpx.HTTPError as e:
            print(f"Error fetching trending stream: {e}")
            return []

    async def get_symbol_sentiment(self, symbol: str) -> SymbolSentiment | None:
        """Get aggregated sentiment for a symbol.

        Args:
            symbol: Stock symbol

        Returns:
            Aggregated sentiment data or None
        """
        posts = await self.get_symbol_stream(symbol, limit=100)
        if not posts:
            return None

        positive = 0
        negative = 0
        neutral = 0
        influencers: dict[str, int] = {}
        sample_posts: list[str] = []

        for post in posts:
            # Count by sentiment label
            label = post.sentiment_label or "neutral"
            if label == "positive":
                positive += 1
            elif label == "negative":
                negative += 1
            else:
                neutral += 1

            # Track influencers by engagement
            if post.user.followers_count > 1000:
                influencers[post.user.username] = influencers.get(
                    post.user.username, 0
                ) + post.likes_count

            # Collect sample posts
            if len(sample_posts) < 5 and len(post.content) > 20:
                sample_posts.append(post.content[:200])

        total = positive + negative + neutral
        if total == 0:
            return None

        # Calculate sentiment score (-1 to +1)
        sentiment_score = (positive - negative) / total

        # Sort influencers by engagement
        top_influencers = sorted(
            influencers.keys(), key=lambda k: influencers[k], reverse=True
        )[:5]

        return SymbolSentiment(
            symbol=symbol,
            total_posts=total,
            positive_posts=positive,
            negative_posts=negative,
            neutral_posts=neutral,
            sentiment_score=sentiment_score,
            trending_score=0.0,  # Would need historical comparison
            top_influencers=top_influencers,
            sample_posts=sample_posts,
        )

    async def get_enriched_stream(
        self,
        symbol: str | None = None,
        limit: int = 50,
    ) -> list[StreamItem]:
        """Get enriched stream items with scores.

        Args:
            symbol: Optional symbol filter
            limit: Max items to return

        Returns:
            List of enriched stream items
        """
        if symbol:
            posts = await self.get_symbol_stream(symbol, limit)
        else:
            posts = await self.get_trending_stream(limit)

        items = []
        for post in posts:
            item = self._enrich_post(post)
            items.append(item)

        return items

    def _parse_post(self, data: dict) -> StockbitPost | None:
        """Parse API response into StockbitPost."""
        try:
            user_data = data.get("user", {})
            user = StockbitUser(
                user_id=str(user_data.get("id", "")),
                username=user_data.get("username", ""),
                display_name=user_data.get("display_name", ""),
                followers_count=user_data.get("followers_count", 0),
                is_verified=user_data.get("is_verified", False),
                is_premium=user_data.get("is_premium", False),
            )

            # Parse timestamp
            created_str = data.get("created_at", "")
            try:
                created_at = datetime.fromisoformat(created_str.replace("Z", "+00:00"))
            except (ValueError, AttributeError):
                created_at = datetime.now()

            # Extract mentioned symbols from content
            content = data.get("content", "") or data.get("body", "")
            mentioned = self._extract_symbols(content)

            post_type_str = data.get("type", "stream")
            try:
                post_type = PostType(post_type_str)
            except ValueError:
                post_type = PostType.STREAM

            return StockbitPost(
                post_id=str(data.get("id", "")),
                post_type=post_type,
                user=user,
                content=content,
                symbol=data.get("symbol"),
                created_at=created_at,
                likes_count=data.get("likes_count", 0),
                comments_count=data.get("comments_count", 0),
                shares_count=data.get("shares_count", 0),
                sentiment_label=data.get("sentiment"),
                mentioned_symbols=mentioned if mentioned else None,
            )

        except Exception as e:
            print(f"Error parsing post: {e}")
            return None

    def _extract_symbols(self, text: str) -> list[str]:
        """Extract stock symbols from text."""
        symbols: set[str] = set()

        # $SYMBOL pattern
        dollar_pattern = r"\$([A-Za-z]{4})"
        symbols.update(m.upper() for m in re.findall(dollar_pattern, text))

        # SYMBOL.JK pattern
        jk_pattern = r"([A-Z]{4})\.JK"
        symbols.update(re.findall(jk_pattern, text.upper()))

        return list(symbols)[:10]

    def _enrich_post(self, post: StockbitPost) -> StreamItem:
        """Calculate enrichment scores for a post."""
        # Engagement score (normalized)
        engagement = (
            post.likes_count * 1.0 + post.comments_count * 2.0 + post.shares_count * 3.0
        )
        engagement_score = min(engagement / 100, 1.0)

        # Virality score (based on recency + engagement)
        hours_old = (datetime.now() - post.created_at).total_seconds() / 3600
        recency_factor = max(0, 1 - hours_old / 24)  # Decays over 24h
        virality_score = engagement_score * recency_factor

        # Credibility score (based on user reputation)
        credibility_score = min(
            (post.user.followers_count / 10000)
            + (0.2 if post.user.is_verified else 0)
            + (0.1 if post.user.is_premium else 0),
            1.0,
        )

        # Pump detection heuristics
        is_potential_pump = self._detect_pump_signals(post)

        return StreamItem(
            post=post,
            engagement_score=engagement_score,
            virality_score=virality_score,
            credibility_score=credibility_score,
            is_potential_pump=is_potential_pump,
        )

    def _detect_pump_signals(self, post: StockbitPost) -> bool:
        """Detect potential pump-and-dump signals."""
        content_lower = post.content.lower()

        # Red flags
        pump_keywords = [
            "to the moon",
            "🚀🚀🚀",
            "guaranteed",
            "pasti naik",
            "buruan beli",
            "jangan sampai ketinggalan",
            "1000%",
            "insider",
            "rahasia",
            "only today",
            "hari ini saja",
        ]

        keyword_count = sum(1 for kw in pump_keywords if kw in content_lower)

        # New/low-follower account pushing aggressively
        low_cred_high_engagement = (
            post.user.followers_count < 100 and post.likes_count > 50
        )

        # Multiple rocket emojis
        rocket_spam = post.content.count("🚀") >= 3

        return keyword_count >= 2 or low_cred_high_engagement or rocket_spam
