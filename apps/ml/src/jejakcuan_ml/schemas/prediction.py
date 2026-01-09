"""Pydantic schemas for prediction endpoints."""

from datetime import datetime
from enum import Enum

from pydantic import BaseModel, Field


class Direction(str, Enum):
    """Price direction prediction."""

    UP = "up"
    DOWN = "down"
    SIDEWAYS = "sideways"


class Sentiment(str, Enum):
    """Sentiment classification."""

    POSITIVE = "positive"
    NEGATIVE = "negative"
    NEUTRAL = "neutral"


class PricePredictionRequest(BaseModel):
    """Request for price prediction."""

    symbol: str = Field(..., description="Stock symbol (e.g., BBCA)")
    horizon_days: int = Field(default=5, ge=1, le=30, description="Prediction horizon in days")


class PricePredictionResponse(BaseModel):
    """Response for price prediction."""

    symbol: str
    timestamp: datetime
    direction: Direction
    confidence: float = Field(..., ge=0, le=1)
    horizon_days: int
    model_version: str


class SentimentRequest(BaseModel):
    """Request for sentiment analysis."""

    text: str = Field(..., min_length=1, max_length=5000)


class SentimentResponse(BaseModel):
    """Response for sentiment analysis."""

    text: str
    sentiment: Sentiment
    confidence: float = Field(..., ge=0, le=1)
    mentioned_symbols: list[str] = Field(default_factory=list)


class BatchSentimentRequest(BaseModel):
    """Request for batch sentiment analysis."""

    texts: list[str] = Field(..., min_length=1, max_length=100)


class BatchSentimentResponse(BaseModel):
    """Response for batch sentiment analysis."""

    results: list[SentimentResponse]


class ModelStatus(BaseModel):
    """Model status information."""

    name: str
    loaded: bool
    version: str | None
    last_trained: datetime | None
