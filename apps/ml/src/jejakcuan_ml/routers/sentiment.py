"""Sentiment analysis endpoints."""

from datetime import datetime
from typing import Optional

from fastapi import APIRouter, HTTPException, Query

from ..models import sentiment_analyzer
from ..schemas import (
    BatchSentimentRequest,
    BatchSentimentResponse,
    ModelStatus,
    Sentiment,
    SentimentRequest,
    SentimentResponse,
)

router = APIRouter(prefix="/sentiment", tags=["sentiment"])


@router.post("/text", response_model=SentimentResponse)
async def analyze_text(request: SentimentRequest) -> SentimentResponse:
    """Analyze sentiment of a single Indonesian text.

    Returns sentiment (positive/negative/neutral), confidence score,
    and any stock symbols mentioned in the text.
    """
    if not sentiment_analyzer.loaded:
        raise HTTPException(
            status_code=503,
            detail="Model not loaded. Call /sentiment/load first.",
        )

    return sentiment_analyzer.analyze(request.text)


@router.post("/batch", response_model=BatchSentimentResponse)
async def analyze_batch(request: BatchSentimentRequest) -> BatchSentimentResponse:
    """Analyze sentiment of multiple texts in batch.

    More efficient than calling /text multiple times.
    Limited to 100 texts per request.
    """
    if not sentiment_analyzer.loaded:
        raise HTTPException(status_code=503, detail="Model not loaded")

    # Use batch inference if available
    if hasattr(sentiment_analyzer, "analyze_batch"):
        results = sentiment_analyzer.analyze_batch(request.texts)
    else:
        results = [sentiment_analyzer.analyze(text) for text in request.texts]

    return BatchSentimentResponse(results=results)


@router.get("/status", response_model=ModelStatus)
async def get_model_status() -> ModelStatus:
    """Get sentiment model status and information."""
    return ModelStatus(
        name="IndoBERT Sentiment Analyzer",
        loaded=sentiment_analyzer.loaded,
        version=sentiment_analyzer.version if sentiment_analyzer.loaded else None,
        last_trained=None,
        metrics={
            "model_type": "IndoBERT",
            "max_length": getattr(sentiment_analyzer, "max_length", 512),
        }
        if sentiment_analyzer.loaded
        else None,
    )


@router.post("/load")
async def load_model(model_path: Optional[str] = None) -> dict:
    """Load or reload the sentiment model.

    Args:
        model_path: Optional path to local model. Uses HuggingFace model if not specified.
    """
    try:
        success = sentiment_analyzer.load(model_path)
        if success:
            return {
                "status": "loaded",
                "version": sentiment_analyzer.version,
                "model": model_path or "mdhugol/indonesia-bert-sentiment-classification",
            }
        else:
            raise HTTPException(status_code=500, detail="Failed to load model")
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@router.post("/symbol/{symbol}")
async def analyze_for_symbol(
    symbol: str,
    request: BatchSentimentRequest,
) -> dict:
    """Analyze texts and aggregate sentiment for a specific symbol.

    Filters results to only include texts mentioning the given symbol
    and returns aggregated sentiment score.
    """
    if not sentiment_analyzer.loaded:
        raise HTTPException(status_code=503, detail="Model not loaded")

    # Analyze all texts
    if hasattr(sentiment_analyzer, "analyze_batch"):
        results = sentiment_analyzer.analyze_batch(request.texts)
    else:
        results = [sentiment_analyzer.analyze(text) for text in request.texts]

    # Filter for symbol mentions
    symbol_upper = symbol.upper()
    relevant = [r for r in results if symbol_upper in r.mentioned_symbols]

    if not relevant:
        return {
            "symbol": symbol_upper,
            "mention_count": 0,
            "sentiment": "neutral",
            "score": 0.0,
            "confidence": 0.0,
            "timestamp": datetime.utcnow().isoformat(),
        }

    # Calculate aggregate sentiment
    sentiment_scores = {
        Sentiment.POSITIVE: 1.0,
        Sentiment.NEUTRAL: 0.0,
        Sentiment.NEGATIVE: -1.0,
    }

    total_weight = 0.0
    weighted_score = 0.0

    for r in relevant:
        score = sentiment_scores.get(r.sentiment, 0.0)
        weight = r.confidence
        weighted_score += score * weight
        total_weight += weight

    avg_score = weighted_score / total_weight if total_weight > 0 else 0.0
    avg_confidence = total_weight / len(relevant)

    # Determine label
    if avg_score > 0.2:
        label = "positive"
    elif avg_score < -0.2:
        label = "negative"
    else:
        label = "neutral"

    return {
        "symbol": symbol_upper,
        "mention_count": len(relevant),
        "sentiment": label,
        "score": round(avg_score, 4),
        "confidence": round(avg_confidence, 4),
        "breakdown": {
            "positive": sum(1 for r in relevant if r.sentiment == Sentiment.POSITIVE),
            "negative": sum(1 for r in relevant if r.sentiment == Sentiment.NEGATIVE),
            "neutral": sum(1 for r in relevant if r.sentiment == Sentiment.NEUTRAL),
        },
        "timestamp": datetime.utcnow().isoformat(),
    }


@router.post("/trending")
async def get_trending_symbols(
    request: BatchSentimentRequest,
    min_mentions: int = Query(default=2, ge=1, le=100),
) -> dict:
    """Get trending stock symbols from texts based on mention frequency.

    Returns symbols sorted by mention count with sentiment breakdown.
    """
    if not sentiment_analyzer.loaded:
        raise HTTPException(status_code=503, detail="Model not loaded")

    # Analyze all texts
    if hasattr(sentiment_analyzer, "analyze_batch"):
        results = sentiment_analyzer.analyze_batch(request.texts)
    else:
        results = [sentiment_analyzer.analyze(text) for text in request.texts]

    # Aggregate by symbol
    symbol_stats: dict = {}

    for r in results:
        for symbol in r.mentioned_symbols:
            if symbol not in symbol_stats:
                symbol_stats[symbol] = {
                    "mention_count": 0,
                    "positive": 0,
                    "negative": 0,
                    "neutral": 0,
                    "total_confidence": 0.0,
                }

            stats = symbol_stats[symbol]
            stats["mention_count"] += 1
            stats["total_confidence"] += r.confidence

            if r.sentiment == Sentiment.POSITIVE:
                stats["positive"] += 1
            elif r.sentiment == Sentiment.NEGATIVE:
                stats["negative"] += 1
            else:
                stats["neutral"] += 1

    # Filter by minimum mentions and sort
    trending = []
    for symbol, stats in symbol_stats.items():
        if stats["mention_count"] >= min_mentions:
            # Calculate net sentiment
            net = stats["positive"] - stats["negative"]
            total = stats["mention_count"]

            trending.append(
                {
                    "symbol": symbol,
                    "mention_count": total,
                    "sentiment_breakdown": {
                        "positive": stats["positive"],
                        "negative": stats["negative"],
                        "neutral": stats["neutral"],
                    },
                    "net_sentiment": net,
                    "sentiment_ratio": round(net / total, 4) if total > 0 else 0,
                    "avg_confidence": round(stats["total_confidence"] / total, 4),
                }
            )

    # Sort by mention count
    trending.sort(key=lambda x: x["mention_count"], reverse=True)

    return {
        "trending_symbols": trending[:20],  # Top 20
        "total_texts_analyzed": len(request.texts),
        "total_symbols_found": len(symbol_stats),
        "timestamp": datetime.utcnow().isoformat(),
    }


@router.post("/extract-symbols")
async def extract_symbols(request: SentimentRequest) -> dict:
    """Extract stock symbols from text without full sentiment analysis.

    Faster than full analysis when only symbols are needed.
    """
    symbols = sentiment_analyzer._extract_symbols(request.text)

    return {
        "text": request.text[:100] + "..." if len(request.text) > 100 else request.text,
        "symbols": symbols,
        "count": len(symbols),
    }
