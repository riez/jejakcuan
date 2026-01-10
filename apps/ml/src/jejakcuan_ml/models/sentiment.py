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

    def load(self, model_path: str | None = None) -> bool:
        """Load model from path."""
        # TODO: Implement actual model loading
        self.loaded = True
        return True

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
        # Pattern 1: 4 uppercase letters that might be stock codes
        pattern1 = r"\b([A-Z]{4})\b"
        matches = set(re.findall(pattern1, text))
        
        # Pattern 2: $SYMBOL pattern (cashtag style)
        pattern2 = r"\$([A-Z]{4})"
        matches.update(re.findall(pattern2, text))
        
        # Filter to likely IDX symbols (basic heuristic)
        return list(matches)[:5]


# Global instance
sentiment_analyzer = SentimentAnalyzer()
