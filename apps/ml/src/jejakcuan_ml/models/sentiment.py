"""IndoBERT sentiment analysis model."""

import re

from ..schemas import Sentiment, SentimentResponse


class SentimentAnalyzer:
    """IndoBERT-based sentiment analyzer for Indonesian text."""

    def __init__(self) -> None:
        """Initialize analyzer."""
        self.model = None
        self.tokenizer = None
        self.version = "0.1.0"
        self.loaded = False

    def load(self, model_path: str) -> None:
        """Load model from path."""
        # TODO: Implement actual model loading
        self.loaded = True

    def analyze(self, text: str) -> SentimentResponse:
        """Analyze sentiment of Indonesian text."""
        # TODO: Implement actual sentiment analysis
        # For now, return placeholder
        mentioned_symbols = self._extract_symbols(text)
        return SentimentResponse(
            text=text[:100] + "..." if len(text) > 100 else text,
            sentiment=Sentiment.NEUTRAL,
            confidence=0.5,
            mentioned_symbols=mentioned_symbols,
        )

    def _extract_symbols(self, text: str) -> list[str]:
        """Extract stock symbols from text."""
        # Simple pattern: 4 uppercase letters that might be stock codes
        pattern = r"\b([A-Z]{4})\b"
        matches = re.findall(pattern, text)
        # Filter to likely IDX symbols (basic heuristic)
        return list(set(matches))[:5]


# Global instance
sentiment_analyzer = SentimentAnalyzer()
