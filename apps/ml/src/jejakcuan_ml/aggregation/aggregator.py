"""Sentiment aggregation logic."""

from datetime import UTC, datetime, timedelta

from ..schemas import Sentiment
from ..schemas.text_sources import (
    AggregatedSentiment,
    SymbolMention,
    TextSentimentResult,
    TextSource,
)


class SentimentAggregator:
    """Aggregate sentiment from multiple sources for stock symbols."""

    # Source weights for aggregation
    SOURCE_WEIGHTS = {
        TextSource.NEWS: 1.5,
        TextSource.IDX_ANNOUNCEMENT: 2.0,
        TextSource.PRESS_RELEASE: 1.8,
        TextSource.STOCKBIT: 1.0,
        TextSource.TWITTER: 0.8,
        TextSource.FORUM: 0.6,
        TextSource.OTHER: 0.5,
    }

    # Sentiment to numeric mapping
    SENTIMENT_SCORES = {
        Sentiment.POSITIVE: 1.0,
        Sentiment.NEUTRAL: 0.0,
        Sentiment.NEGATIVE: -1.0,
        "positive": 1.0,
        "neutral": 0.0,
        "negative": -1.0,
    }

    def __init__(self) -> None:
        """Initialize aggregator."""
        pass

    def aggregate_for_symbol(
        self,
        symbol: str,
        results: list[TextSentimentResult],
        window_hours: int = 24,
    ) -> AggregatedSentiment:
        """Aggregate sentiment results for a specific symbol.

        Args:
            symbol: Stock symbol to aggregate for
            results: List of sentiment results mentioning the symbol
            window_hours: Time window for aggregation

        Returns:
            AggregatedSentiment with weighted score
        """
        if not results:
            return AggregatedSentiment(
                symbol=symbol,
                timestamp=datetime.now(UTC),
                sentiment="neutral",
                score=0.0,
                confidence=0.0,
                news_sentiment=None,
                social_sentiment=None,
                window_hours=window_hours,
            )

        # Filter results for this symbol and time window
        cutoff = datetime.now(UTC) - timedelta(hours=window_hours)
        relevant = [
            r
            for r in results
            if symbol in r.mentioned_symbols and r.timestamp >= cutoff
        ]

        if not relevant:
            return AggregatedSentiment(
                symbol=symbol,
                timestamp=datetime.now(UTC),
                sentiment="neutral",
                score=0.0,
                confidence=0.0,
                news_sentiment=None,
                social_sentiment=None,
                window_hours=window_hours,
            )

        # Calculate weighted sentiment score
        total_weight = 0.0
        weighted_score = 0.0
        weighted_confidence = 0.0

        news_scores: list[float] = []
        social_scores: list[float] = []

        for result in relevant:
            # Get source weight
            source_weight = self.SOURCE_WEIGHTS.get(result.source_type, 0.5)
            combined_weight = source_weight * result.weight * result.confidence

            # Get numeric score
            score = self.SENTIMENT_SCORES.get(result.sentiment.lower(), 0.0)

            weighted_score += score * combined_weight
            weighted_confidence += result.confidence * combined_weight
            total_weight += combined_weight

            # Track by source type
            if result.source_type in (
                TextSource.NEWS,
                TextSource.IDX_ANNOUNCEMENT,
                TextSource.PRESS_RELEASE,
            ):
                news_scores.append(score)
            else:
                social_scores.append(score)

        # Calculate final aggregated values
        if total_weight > 0:
            final_score = weighted_score / total_weight
            final_confidence = weighted_confidence / total_weight
        else:
            final_score = 0.0
            final_confidence = 0.0

        # Determine sentiment label
        if final_score > 0.2:
            sentiment_label = "positive"
        elif final_score < -0.2:
            sentiment_label = "negative"
        else:
            sentiment_label = "neutral"

        # Calculate source-specific averages
        news_sentiment = sum(news_scores) / len(news_scores) if news_scores else None
        social_sentiment = (
            sum(social_scores) / len(social_scores) if social_scores else None
        )

        return AggregatedSentiment(
            symbol=symbol,
            timestamp=datetime.now(UTC),
            sentiment=sentiment_label,
            score=round(final_score, 4),
            confidence=round(final_confidence, 4),
            news_sentiment=(
                round(news_sentiment, 4) if news_sentiment is not None else None
            ),
            news_count=len(news_scores),
            social_sentiment=(
                round(social_sentiment, 4) if social_sentiment is not None else None
            ),
            social_count=len(social_scores),
            window_hours=window_hours,
            results=relevant,
        )

    def get_symbol_mentions(
        self,
        results: list[TextSentimentResult],
    ) -> dict[str, SymbolMention]:
        """Get mention statistics for all symbols.

        Args:
            results: List of sentiment results

        Returns:
            Dict mapping symbol to SymbolMention
        """
        mentions: dict[str, SymbolMention] = {}

        for result in results:
            for symbol in result.mentioned_symbols:
                if symbol not in mentions:
                    mentions[symbol] = SymbolMention(symbol=symbol)

                m = mentions[symbol]
                m.mention_count += 1

                sentiment = (
                    result.sentiment.lower()
                    if isinstance(result.sentiment, str)
                    else result.sentiment.value
                )
                if sentiment == "positive":
                    m.positive_count += 1
                elif sentiment == "negative":
                    m.negative_count += 1
                else:
                    m.neutral_count += 1

                # Update average confidence
                m.avg_confidence = (
                    m.avg_confidence * (m.mention_count - 1) + result.confidence
                ) / m.mention_count

                # Track sources
                source_name = result.source_type.value
                if source_name not in m.sources:
                    m.sources.append(source_name)

                # Update last mentioned
                if m.last_mentioned is None or result.timestamp > m.last_mentioned:
                    m.last_mentioned = result.timestamp

        return mentions

    def calculate_sentiment_score(
        self,
        aggregated: AggregatedSentiment,
    ) -> float:
        """Calculate a 0-100 score for use in the scoring engine.

        Args:
            aggregated: Aggregated sentiment data

        Returns:
            Score from 0 to 100
        """
        # Base score from sentiment (convert -1 to 1 scale to 0 to 100)
        base_score = (aggregated.score + 1) * 50  # -1 -> 0, 0 -> 50, 1 -> 100

        # Adjust by confidence
        confidence_factor = 0.5 + (aggregated.confidence * 0.5)

        # Adjust by mention count (more mentions = more reliable)
        total_mentions = aggregated.news_count + aggregated.social_count
        mention_factor = min(1.0, total_mentions / 20)  # Normalize to 20 mentions
        reliability_factor = 0.7 + (mention_factor * 0.3)

        final_score = base_score * confidence_factor * reliability_factor

        return round(max(0, min(100, final_score)), 2)
