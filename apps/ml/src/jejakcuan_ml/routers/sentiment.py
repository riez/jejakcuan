"""Sentiment analysis endpoints."""

from fastapi import APIRouter, HTTPException

from ..models import sentiment_analyzer
from ..schemas import (
    BatchSentimentRequest,
    BatchSentimentResponse,
    ModelStatus,
    SentimentRequest,
    SentimentResponse,
)

router = APIRouter(prefix="/sentiment", tags=["sentiment"])


@router.post("/text", response_model=SentimentResponse)
async def analyze_text(request: SentimentRequest) -> SentimentResponse:
    """Analyze sentiment of a single text."""
    if not sentiment_analyzer.loaded:
        raise HTTPException(status_code=503, detail="Model not loaded")

    return sentiment_analyzer.analyze(request.text)


@router.post("/batch", response_model=BatchSentimentResponse)
async def analyze_batch(request: BatchSentimentRequest) -> BatchSentimentResponse:
    """Analyze sentiment of multiple texts."""
    if not sentiment_analyzer.loaded:
        raise HTTPException(status_code=503, detail="Model not loaded")

    results = [sentiment_analyzer.analyze(text) for text in request.texts]
    return BatchSentimentResponse(results=results)


@router.get("/status", response_model=ModelStatus)
async def get_model_status() -> ModelStatus:
    """Get sentiment model status."""
    return ModelStatus(
        name="IndoBERT Sentiment Analyzer",
        loaded=sentiment_analyzer.loaded,
        version=sentiment_analyzer.version if sentiment_analyzer.loaded else None,
        last_trained=None,
    )
