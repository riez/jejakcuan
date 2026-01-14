"""Stockbit data models."""

from dataclasses import dataclass
from datetime import datetime
from enum import Enum


class PostType(str, Enum):
    """Stockbit post types."""

    STREAM = "stream"
    ANALYSIS = "analysis"
    NEWS = "news"
    IDEA = "idea"
    COMMENT = "comment"


@dataclass
class StockbitUser:
    """Stockbit user info."""

    user_id: str
    username: str
    display_name: str
    followers_count: int = 0
    is_verified: bool = False
    is_premium: bool = False


@dataclass
class StockbitPost:
    """Stockbit post/stream item."""

    post_id: str
    post_type: PostType
    user: StockbitUser
    content: str
    symbol: str | None
    created_at: datetime
    likes_count: int = 0
    comments_count: int = 0
    shares_count: int = 0
    sentiment_label: str | None = None  # From Stockbit's own analysis
    mentioned_symbols: list[str] | None = None


@dataclass
class StreamItem:
    """Aggregated stream item with enriched data."""

    post: StockbitPost
    engagement_score: float  # Normalized engagement metric
    virality_score: float  # How quickly it's spreading
    credibility_score: float  # Based on user reputation
    is_potential_pump: bool = False  # Flag for suspicious activity


@dataclass
class SymbolSentiment:
    """Aggregated sentiment for a symbol."""

    symbol: str
    total_posts: int
    positive_posts: int
    negative_posts: int
    neutral_posts: int
    sentiment_score: float  # -1 to +1
    trending_score: float  # How much discussion increased
    top_influencers: list[str]  # Users driving discussion
    sample_posts: list[str]  # Representative post excerpts
