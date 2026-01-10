"""IndoBERT sentiment analysis model."""

import re
from pathlib import Path

import torch
from transformers import AutoModelForSequenceClassification, AutoTokenizer

from ..schemas import Sentiment, SentimentResponse


class SentimentAnalyzer:
    """IndoBERT-based sentiment analyzer for Indonesian financial text."""

    # Pre-trained model for Indonesian sentiment
    DEFAULT_MODEL = "mdhugol/indonesia-bert-sentiment-classification"

    # Mapping from model labels to our Sentiment enum
    LABEL_MAP = {
        "positive": Sentiment.POSITIVE,
        "negative": Sentiment.NEGATIVE,
        "neutral": Sentiment.NEUTRAL,
        # Handle numeric labels if model uses them
        "LABEL_0": Sentiment.NEGATIVE,
        "LABEL_1": Sentiment.NEUTRAL,
        "LABEL_2": Sentiment.POSITIVE,
    }

    def __init__(self) -> None:
        """Initialize analyzer."""
        self.model: AutoModelForSequenceClassification | None = None
        self.tokenizer: AutoTokenizer | None = None
        self.version = "1.0.0"
        self.loaded = False
        self.device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
        self.max_length = 512

    def load(self, model_path: str | None = None) -> bool:
        """Load model from path or download from HuggingFace.

        Args:
            model_path: Local path to saved model, or None to use default HF model

        Returns:
            True if loaded successfully
        """
        try:
            model_name = model_path if model_path else self.DEFAULT_MODEL

            # Check if local path exists
            if model_path and Path(model_path).exists():
                self.tokenizer = AutoTokenizer.from_pretrained(model_path)  # type: ignore[no-untyped-call]
                self.model = AutoModelForSequenceClassification.from_pretrained(model_path)
            else:
                # Load from HuggingFace
                self.tokenizer = AutoTokenizer.from_pretrained(model_name)  # type: ignore[no-untyped-call]
                self.model = AutoModelForSequenceClassification.from_pretrained(model_name)

            self.model.to(self.device)  # type: ignore[union-attr]
            self.model.eval()  # type: ignore[union-attr]
            self.loaded = True
            return True

        except Exception as e:
            print(f"Failed to load sentiment model: {e}")
            self.loaded = False
            return False

    def preprocess(self, text: str) -> str:
        """Preprocess Indonesian text for sentiment analysis."""
        # Clean text
        text = text.strip()

        # Remove URLs
        text = re.sub(r"http\S+|www\.\S+", "", text)

        # Remove HTML tags
        text = re.sub(r"<[^>]+>", "", text)

        # Remove extra whitespace
        text = re.sub(r"\s+", " ", text).strip()

        # Keep Indonesian characters, numbers, and common punctuation
        # Allow stock tickers (uppercase letters)
        text = re.sub(r"[^\w\s.,!?'\"-]", "", text)

        return text

    def analyze(self, text: str) -> SentimentResponse:
        """Analyze sentiment of Indonesian text.

        Args:
            text: Indonesian text to analyze

        Returns:
            SentimentResponse with sentiment, confidence, and mentioned symbols
        """
        if not self.loaded or self.model is None or self.tokenizer is None:
            # Return neutral with low confidence if model not loaded
            return SentimentResponse(
                text=text[:100] + "..." if len(text) > 100 else text,
                sentiment=Sentiment.NEUTRAL,
                confidence=0.0,
                mentioned_symbols=self._extract_symbols(text),
            )

        # Preprocess
        clean_text = self.preprocess(text)
        if not clean_text:
            return SentimentResponse(
                text=text[:100] + "..." if len(text) > 100 else text,
                sentiment=Sentiment.NEUTRAL,
                confidence=0.5,
                mentioned_symbols=[],
            )

        # Tokenize
        inputs = self.tokenizer(  # type: ignore[operator]
            clean_text,
            return_tensors="pt",
            truncation=True,
            max_length=self.max_length,
            padding=True,
        )
        inputs = {k: v.to(self.device) for k, v in inputs.items()}

        # Inference
        with torch.no_grad():
            outputs = self.model(**inputs)  # type: ignore[operator]
            probs = torch.softmax(outputs.logits, dim=-1)
            pred_class = torch.argmax(probs, dim=-1).item()
            confidence = probs[0, int(pred_class)].item()

        # Map to sentiment
        id2label = self.model.config.id2label  # type: ignore[attr-defined]
        label = id2label.get(pred_class, f"LABEL_{pred_class}")
        sentiment = self.LABEL_MAP.get(label.lower(), Sentiment.NEUTRAL)

        # Extract mentioned stock symbols
        mentioned_symbols = self._extract_symbols(text)

        return SentimentResponse(
            text=text[:100] + "..." if len(text) > 100 else text,
            sentiment=sentiment,
            confidence=confidence,
            mentioned_symbols=mentioned_symbols,
        )

    def analyze_batch(self, texts: list[str]) -> list[SentimentResponse]:
        """Analyze sentiment of multiple texts efficiently.

        Args:
            texts: List of Indonesian texts to analyze

        Returns:
            List of SentimentResponse objects
        """
        if not self.loaded or self.model is None or self.tokenizer is None:
            return [self.analyze(text) for text in texts]

        # Preprocess all texts
        clean_texts = [self.preprocess(text) for text in texts]

        # Tokenize batch
        inputs = self.tokenizer(  # type: ignore[operator]
            clean_texts,
            return_tensors="pt",
            truncation=True,
            max_length=self.max_length,
            padding=True,
        )
        inputs = {k: v.to(self.device) for k, v in inputs.items()}

        # Batch inference
        with torch.no_grad():
            outputs = self.model(**inputs)  # type: ignore[operator]
            probs = torch.softmax(outputs.logits, dim=-1)
            pred_classes = torch.argmax(probs, dim=-1).cpu().numpy()
            confidences = probs.max(dim=-1).values.cpu().numpy()

        # Build responses
        id2label = self.model.config.id2label  # type: ignore[attr-defined]
        results = []

        for i, (text, pred_class, conf) in enumerate(zip(texts, pred_classes, confidences)):
            label = id2label.get(int(pred_class), f"LABEL_{pred_class}")
            sentiment = self.LABEL_MAP.get(label.lower(), Sentiment.NEUTRAL)

            results.append(
                SentimentResponse(
                    text=text[:100] + "..." if len(text) > 100 else text,
                    sentiment=sentiment,
                    confidence=float(conf),
                    mentioned_symbols=self._extract_symbols(text),
                )
            )

        return results

    def _extract_symbols(self, text: str) -> list[str]:
        """Extract stock symbols from text.

        Looks for patterns like:
        - $BBCA or $bbca
        - BBCA.JK
        - saham BBCA
        - emiten BBRI
        """
        symbols: set[str] = set()

        # Pattern 1: $SYMBOL (common in social media)
        dollar_pattern = r"\$([A-Za-z]{4})"
        symbols.update(m.upper() for m in re.findall(dollar_pattern, text))

        # Pattern 2: SYMBOL.JK (Yahoo Finance format)
        jk_pattern = r"([A-Z]{4})\.JK"
        symbols.update(re.findall(jk_pattern, text.upper()))

        # Pattern 3: 4 uppercase letters that look like tickers
        # preceded by common Indonesian stock terms
        context_pattern = r"(?:saham|emiten|kode|ticker)\s+([A-Z]{4})"
        symbols.update(re.findall(context_pattern, text.upper()))

        # Pattern 4: Standalone 4 uppercase letters (be more conservative)
        # Only include if it looks like a real ticker (all caps, standalone)
        standalone_pattern = r"\b([A-Z]{4})\b"
        potential = re.findall(standalone_pattern, text)

        # Filter common non-ticker words
        non_tickers = {
            "YANG",
            "AKAN",
            "DARI",
            "PADA",
            "AGAR",
            "JIKA",
            "TAPI",
            "ATAU",
            "BISA",
            "KAMI",
            "MAKA",
            "JUGA",
            "SAMA",
            "LAIN",
        }
        symbols.update(s for s in potential if s not in non_tickers)

        return list(symbols)[:10]  # Limit to 10 symbols

    def save(self, model_path: str) -> None:
        """Save model to local path."""
        if self.model is None or self.tokenizer is None:
            raise RuntimeError("No model to save")

        path = Path(model_path)
        path.mkdir(parents=True, exist_ok=True)

        self.model.save_pretrained(path)  # type: ignore[attr-defined]
        self.tokenizer.save_pretrained(path)  # type: ignore[attr-defined]


# Global instance
sentiment_analyzer = SentimentAnalyzer()
