"""Pydantic schemas for prediction endpoints."""

from datetime import datetime
from enum import Enum
from typing import Any

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

    # Optional detailed predictions
    probabilities: dict[str, float] | None = None


class BatchPredictionRequest(BaseModel):
    """Request for batch price prediction."""

    symbols: list[str] = Field(..., min_length=1, max_length=50)
    horizon_days: int = Field(default=5, ge=1, le=30)


class BatchPredictionResponse(BaseModel):
    """Response for batch price prediction."""

    predictions: list[PricePredictionResponse]
    timestamp: datetime
    model_version: str


class PredictionWithFeatures(BaseModel):
    """Request prediction with pre-computed features."""

    symbol: str
    features: list[list[float]] = Field(..., description="Feature matrix (seq_len x num_features)")
    horizon_days: int = Field(default=5)


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
    version: str | None = None
    last_trained: datetime | None = None
    metrics: dict[str, Any] | None = None


class TrainingStatus(str, Enum):
    """Training job status."""

    PENDING = "pending"
    RUNNING = "running"
    COMPLETED = "completed"
    FAILED = "failed"


class TrainingRequest(BaseModel):
    """Request to trigger model training."""

    symbols: list[str] = Field(
        default_factory=list, description="Symbols to train on (empty = all)"
    )
    epochs: int = Field(default=100, ge=10, le=500)
    force_retrain: bool = Field(default=False)


class TrainingResponse(BaseModel):
    """Response for training request."""

    job_id: str
    status: TrainingStatus
    message: str
    started_at: datetime | None = None


class TrainingJobStatus(BaseModel):
    """Status of a training job."""

    job_id: str
    status: TrainingStatus
    progress: float = Field(ge=0, le=1)
    started_at: datetime | None = None
    completed_at: datetime | None = None
    metrics: dict[str, Any] | None = None
    error: str | None = None
