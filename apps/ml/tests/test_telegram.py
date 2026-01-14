"""Tests for Telegram monitoring module."""

import pytest
from jejakcuan_ml.telegram.parser import TelegramMessageParser, ParsedStockMention


class TestTelegramParser:
    """Tests for TelegramMessageParser."""

    def setup_method(self):
        self.parser = TelegramMessageParser()

    def test_is_stock_related_with_keywords(self):
        """Test detection of stock-related messages via keywords."""
        assert self.parser.is_stock_related("IHSG menguat hari ini")
        assert self.parser.is_stock_related("Saham BBCA naik 2%")
        assert self.parser.is_stock_related("Emiten perbankan bullish")
        assert not self.parser.is_stock_related("Cuaca hari ini cerah")

    def test_is_stock_related_with_symbols(self):
        """Test detection via stock symbols."""
        assert self.parser.is_stock_related("$BBCA looks strong")
        assert self.parser.is_stock_related("Check out BBRI.JK")
        assert self.parser.is_stock_related("TLKM breaking out")

    def test_extract_symbols_dollar_format(self):
        """Test extraction of $SYMBOL format."""
        text = "I'm buying $BBCA and $BBRI today"
        symbols = self.parser.extract_symbols(text)
        assert "BBCA" in symbols
        assert "BBRI" in symbols

    def test_extract_symbols_jk_format(self):
        """Test extraction of SYMBOL.JK format."""
        text = "Check BMRI.JK and ASII.JK"
        symbols = self.parser.extract_symbols(text)
        assert "BMRI" in symbols
        assert "ASII" in symbols

    def test_extract_symbols_filters_non_tickers(self):
        """Test that common words are filtered out."""
        text = "YANG AKAN DARI BBCA saham"
        symbols = self.parser.extract_symbols(text)
        assert "BBCA" in symbols
        assert "YANG" not in symbols
        assert "AKAN" not in symbols

    def test_extract_mentions_with_context(self):
        """Test extraction of mentions with surrounding context."""
        text = "Saham BBCA sedang bullish dan breakout resistance"
        mentions = self.parser.extract_mentions(text)
        
        assert len(mentions) == 1
        assert mentions[0].symbol == "BBCA"
        assert mentions[0].sentiment_hint == "bullish"

    def test_detect_bearish_sentiment(self):
        """Test bearish sentiment detection."""
        text = "BRIS anjlok, cut loss segera"
        mentions = self.parser.extract_mentions(text)
        
        assert len(mentions) >= 1
        assert mentions[0].sentiment_hint == "bearish"

    def test_message_intensity_exclamations(self):
        """Test intensity scoring with exclamations."""
        low_intensity = "BBCA looks okay"
        high_intensity = "BBCA TO THE MOON!!! 🚀🚀🚀"
        
        assert self.parser.get_message_intensity(high_intensity) > \
               self.parser.get_message_intensity(low_intensity)

    def test_message_intensity_caps(self):
        """Test intensity scoring with caps."""
        normal = "bbca naik"
        shouting = "BBCA NAIK TERUS!!!"
        
        assert self.parser.get_message_intensity(shouting) > \
               self.parser.get_message_intensity(normal)

    def test_message_intensity_emojis(self):
        """Test intensity scoring with hype emojis."""
        no_emoji = "BBCA breaking out"
        with_emoji = "BBCA 🚀🔥💰 breaking out"
        
        assert self.parser.get_message_intensity(with_emoji) > \
               self.parser.get_message_intensity(no_emoji)

    def test_message_intensity_bounds(self):
        """Test that intensity is bounded 0-1."""
        extreme = "BUY BUY BUY!!!!!!! 🚀🚀🚀🚀🚀 NOWWWWWW"
        intensity = self.parser.get_message_intensity(extreme)
        
        assert 0 <= intensity <= 1
