"""Pydantic schemas."""

from .prediction import (
    BatchPredictionRequest,
    BatchPredictionResponse,
    BatchSentimentRequest,
    BatchSentimentResponse,
    Direction,
    ModelStatus,
    PredictionWithFeatures,
    PricePredictionRequest,
    PricePredictionResponse,
    Sentiment,
    SentimentRequest,
    SentimentResponse,
    TrainingJobStatus,
    TrainingRequest,
    TrainingResponse,
    TrainingStatus,
)
from .text_sources import (
    AggregatedSentiment,
    NewsArticle,
    SentimentTrend,
    SocialPost,
    SymbolMention,
    TextBatchRequest,
    TextSentimentResult,
    TextSource,
)

__all__ = [
    # Prediction schemas
    "BatchPredictionRequest",
    "BatchPredictionResponse",
    "BatchSentimentRequest",
    "BatchSentimentResponse",
    "Direction",
    "ModelStatus",
    "PredictionWithFeatures",
    "PricePredictionRequest",
    "PricePredictionResponse",
    "Sentiment",
    "SentimentRequest",
    "SentimentResponse",
    "TrainingJobStatus",
    "TrainingRequest",
    "TrainingResponse",
    "TrainingStatus",
    # Text source schemas
    "AggregatedSentiment",
    "NewsArticle",
    "SentimentTrend",
    "SocialPost",
    "SymbolMention",
    "TextBatchRequest",
    "TextSentimentResult",
    "TextSource",
]
