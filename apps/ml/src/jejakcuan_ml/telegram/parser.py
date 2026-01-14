"""Telegram message parser for stock-related content."""

import re
from dataclasses import dataclass


@dataclass
class ParsedStockMention:
    """Extracted stock mention from message."""

    symbol: str
    context: str  # Surrounding text
    sentiment_hint: str | None = None  # "bullish", "bearish", or None


class TelegramMessageParser:
    """Parse Telegram messages for stock-related content."""

    # Indonesian stock-related keywords
    STOCK_KEYWORDS = {
        "saham",
        "emiten",
        "ihsg",
        "idx",
        "bursa",
        "trading",
        "bullish",
        "bearish",
        "support",
        "resistance",
        "breakout",
        "breakdown",
        "trending",
        "akumulasi",
        "distribusi",
        "bandarmology",
        "foreign",
        "asing",
        "net buy",
        "net sell",
        "all time high",
        "ath",
        "cut loss",
        "take profit",
        "tp",
        "sl",
        "lot",
        "volume",
        "bid",
        "offer",
        "eps",
        "per",
        "pbv",
        "dividend",
        "dividen",
        "right issue",
        "stock split",
    }

    # Bullish indicators
    BULLISH_KEYWORDS = {
        "bullish",
        "naik",
        "up",
        "buy",
        "beli",
        "akumulasi",
        "breakout",
        "mantap",
        "cuan",
        "profit",
        "terbang",
        "rocket",
        "moon",
        "strong",
        "kuat",
        "net buy",
        "ath",
        "all time high",
    }

    # Bearish indicators
    BEARISH_KEYWORDS = {
        "bearish",
        "turun",
        "down",
        "sell",
        "jual",
        "distribusi",
        "breakdown",
        "cut loss",
        "rugi",
        "loss",
        "jebol",
        "anjlok",
        "weak",
        "lemah",
        "net sell",
        "koreksi",
    }

    # Non-ticker 4-letter Indonesian words
    NON_TICKERS = {
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
        "DARI",
        "SAYA",
        "KITA",
        "ANDA",
        "SINI",
        "SANA",
        "MASA",
        "WAKTU",
        "HARI",
        "PAGI",
        "SORE",
        "MALAM",
    }

    def is_stock_related(self, text: str) -> bool:
        """Check if message is stock-related.

        Args:
            text: Message text

        Returns:
            True if message appears to be about stocks
        """
        text_lower = text.lower()

        # Check for stock keywords
        for keyword in self.STOCK_KEYWORDS:
            if keyword in text_lower:
                return True

        # Check for stock ticker patterns
        symbols = self.extract_symbols(text)
        if symbols:
            return True

        # Check for $ mentions (common in stock discussions)
        if re.search(r"\$[A-Za-z]{4}", text):
            return True

        return False

    def extract_symbols(self, text: str) -> list[str]:
        """Extract stock symbols from text.

        Args:
            text: Message text

        Returns:
            List of extracted stock symbols (4 uppercase letters)
        """
        symbols: set[str] = set()

        # Pattern 1: $SYMBOL (common in social media)
        dollar_pattern = r"\$([A-Za-z]{4})"
        symbols.update(m.upper() for m in re.findall(dollar_pattern, text))

        # Pattern 2: SYMBOL.JK (Yahoo Finance format)
        jk_pattern = r"([A-Z]{4})\.JK"
        symbols.update(re.findall(jk_pattern, text.upper()))

        # Pattern 3: Context-based extraction
        context_pattern = r"(?:saham|emiten|kode|ticker)\s+([A-Z]{4})"
        symbols.update(re.findall(context_pattern, text.upper()))

        # Pattern 4: Standalone 4 uppercase letters
        standalone_pattern = r"\b([A-Z]{4})\b"
        potential = re.findall(standalone_pattern, text)
        symbols.update(s for s in potential if s not in self.NON_TICKERS)

        return list(symbols)[:10]

    def extract_mentions(self, text: str) -> list[ParsedStockMention]:
        """Extract detailed stock mentions with context.

        Args:
            text: Message text

        Returns:
            List of parsed stock mentions
        """
        mentions: list[ParsedStockMention] = []
        symbols = self.extract_symbols(text)
        text_lower = text.lower()

        for symbol in symbols:
            # Get surrounding context (30 chars before and after)
            pattern = rf"(.{{0,30}}{re.escape(symbol)}.{{0,30}})"
            match = re.search(pattern, text, re.IGNORECASE)
            context = match.group(1) if match else symbol

            # Detect sentiment hint from context
            sentiment_hint = self._detect_sentiment_hint(context.lower())

            mentions.append(
                ParsedStockMention(
                    symbol=symbol,
                    context=context.strip(),
                    sentiment_hint=sentiment_hint,
                )
            )

        return mentions

    def _detect_sentiment_hint(self, context: str) -> str | None:
        """Detect sentiment hint from context.

        Args:
            context: Text surrounding the stock symbol

        Returns:
            "bullish", "bearish", or None
        """
        bullish_count = sum(1 for kw in self.BULLISH_KEYWORDS if kw in context)
        bearish_count = sum(1 for kw in self.BEARISH_KEYWORDS if kw in context)

        if bullish_count > bearish_count:
            return "bullish"
        elif bearish_count > bullish_count:
            return "bearish"
        return None

    def get_message_intensity(self, text: str) -> float:
        """Calculate message intensity/urgency score.

        Higher score indicates more emphatic/urgent message.

        Args:
            text: Message text

        Returns:
            Intensity score 0.0-1.0
        """
        score = 0.0

        # Exclamation marks add intensity
        exclamations = text.count("!")
        score += min(exclamations * 0.1, 0.3)

        # CAPS indicate emphasis
        caps_ratio = sum(1 for c in text if c.isupper()) / max(len(text), 1)
        score += min(caps_ratio, 0.3)

        # Emojis/rockets/fire indicate hype
        hype_emojis = len(re.findall(r"[🚀🔥💰💎📈📉]", text))
        score += min(hype_emojis * 0.05, 0.2)

        # Repetition (e.g., "BUYYYY")
        if re.search(r"(.)\1{3,}", text):
            score += 0.1

        # Urgent words
        urgent_words = ["urgent", "now", "segera", "cepat", "buruan", "alert"]
        for word in urgent_words:
            if word in text.lower():
                score += 0.1
                break

        return min(score, 1.0)
