"""Pydantic schemas for text data sources (news, social media)."""

from datetime import datetime
from enum import Enum
from typing import Any

from pydantic import BaseModel, Field, HttpUrl


class TextSource(str, Enum):
    """Source type for text data."""

    NEWS = "news"
    TWITTER = "twitter"
    STOCKBIT = "stockbit"
    IDX_ANNOUNCEMENT = "idx_announcement"
    PRESS_RELEASE = "press_release"
    FORUM = "forum"
    OTHER = "other"


class NewsArticle(BaseModel):
    """News article for sentiment analysis."""

    id: str | None = None
    title: str = Field(..., min_length=1, max_length=500)
    content: str = Field(..., min_length=1)
    summary: str | None = Field(None, max_length=1000)
    source: str = Field(..., description="News source name (e.g., Kontan, Bisnis)")
    url: HttpUrl | None = None
    published_at: datetime
    author: str | None = None
    mentioned_symbols: list[str] = Field(default_factory=list)
    tags: list[str] = Field(default_factory=list)


class SocialPost(BaseModel):
    """Social media post for sentiment analysis."""

    id: str
    platform: TextSource
    username: str
    content: str = Field(..., min_length=1, max_length=5000)
    posted_at: datetime
    likes: int = Field(default=0, ge=0)
    replies: int = Field(default=0, ge=0)
    reposts: int = Field(default=0, ge=0)
    mentioned_symbols: list[str] = Field(default_factory=list)
    hashtags: list[str] = Field(default_factory=list)
    is_verified: bool = False
    follower_count: int | None = None


class TextBatchRequest(BaseModel):
    """Request to analyze batch of texts from various sources."""

    news_articles: list[NewsArticle] = Field(default_factory=list)
    social_posts: list[SocialPost] = Field(default_factory=list)


class TextSentimentResult(BaseModel):
    """Sentiment result for a single text item."""

    source_id: str
    source_type: TextSource
    sentiment: str  # positive, negative, neutral
    confidence: float = Field(..., ge=0, le=1)
    mentioned_symbols: list[str]
    timestamp: datetime
    weight: float = Field(
        default=1.0,
        ge=0,
        description="Weight for aggregation (based on source credibility)",
    )


class AggregatedSentiment(BaseModel):
    """Aggregated sentiment for a stock symbol."""

    symbol: str
    timestamp: datetime

    # Overall sentiment
    sentiment: str  # positive, negative, neutral
    score: float = Field(..., ge=-1, le=1, description="-1 to 1 scale")
    confidence: float = Field(..., ge=0, le=1)

    # Breakdown by source
    news_sentiment: float | None = Field(None, ge=-1, le=1)
    news_count: int = 0
    social_sentiment: float | None = Field(None, ge=-1, le=1)
    social_count: int = 0

    # Time window
    window_hours: int = 24

    # Individual results
    results: list[TextSentimentResult] = Field(default_factory=list)


class SymbolMention(BaseModel):
    """Track symbol mentions across text sources."""

    symbol: str
    mention_count: int = 0
    positive_count: int = 0
    negative_count: int = 0
    neutral_count: int = 0
    avg_confidence: float = 0.0
    sources: list[str] = Field(default_factory=list)
    last_mentioned: datetime | None = None


class SentimentTrend(BaseModel):
    """Sentiment trend over time for a symbol."""

    symbol: str
    data_points: list[dict[str, Any]] = Field(
        default_factory=list,
        description="List of {timestamp, score, count} dicts",
    )
    trend_direction: str = "neutral"  # improving, declining, neutral
    volatility: float = 0.0  # Sentiment volatility measure
