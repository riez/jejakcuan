"""Tests for pump-and-dump detection."""

import pytest
from jejakcuan_ml.detection import PumpDetector, PumpSignal, PumpAlert
from jejakcuan_ml.detection.pump_detector import (
    SocialMetrics,
    MarketMetrics,
    PumpSignalType,
    AlertSeverity,
)


class TestPumpDetector:
    """Tests for PumpDetector."""

    def setup_method(self):
        self.detector = PumpDetector(
            social_spike_threshold=3.0,
            volume_spike_threshold=5.0,
            price_spike_threshold=0.15,
            min_confidence_threshold=0.3,
        )

    def test_no_alert_normal_activity(self):
        """Test that normal activity doesn't trigger alert."""
        social = SocialMetrics(
            symbol="BBCA",
            post_count=10,
            unique_authors=8,
            avg_author_age_days=365,
            sentiment_score=0.5,
            keyword_flags=[],
        )
        market = MarketMetrics(
            symbol="BBCA",
            current_price=10000,
            price_change_pct=0.02,
            volume=1000000,
            avg_volume=900000,
            volume_ratio=1.1,
        )

        alert = self.detector.analyze(social, market, historical_social=10)
        assert alert is None

    def test_social_spike_detection(self):
        """Test detection of social media spike."""
        social = SocialMetrics(
            symbol="TLKM",
            post_count=50,  # 5x baseline of 10
            unique_authors=20,
            avg_author_age_days=180,
            sentiment_score=0.8,
            keyword_flags=["moon", "rocket"],
        )
        market = MarketMetrics(
            symbol="TLKM",
            current_price=5000,
            price_change_pct=0.05,
            volume=2000000,
            avg_volume=1000000,
            volume_ratio=2.0,
        )

        alert = self.detector.analyze(social, market, historical_social=10)
        
        assert alert is not None
        signal_types = [s.signal_type for s in alert.signals]
        assert PumpSignalType.SOCIAL_SPIKE in signal_types

    def test_volume_spike_detection(self):
        """Test detection of volume spike."""
        social = SocialMetrics(
            symbol="BRIS",
            post_count=15,
            unique_authors=12,
            avg_author_age_days=200,
            sentiment_score=0.6,
            keyword_flags=[],
        )
        market = MarketMetrics(
            symbol="BRIS",
            current_price=1500,
            price_change_pct=0.10,
            volume=10000000,
            avg_volume=1000000,
            volume_ratio=10.0,  # 10x average
        )

        alert = self.detector.analyze(social, market, historical_social=15)
        
        assert alert is not None
        signal_types = [s.signal_type for s in alert.signals]
        assert PumpSignalType.VOLUME_SPIKE in signal_types

    def test_price_spike_detection(self):
        """Test detection of price spike."""
        social = SocialMetrics(
            symbol="GOTO",
            post_count=30,
            unique_authors=25,
            avg_author_age_days=150,
            sentiment_score=0.9,
            keyword_flags=["bullish"],
        )
        market = MarketMetrics(
            symbol="GOTO",
            current_price=150,
            price_change_pct=0.25,  # 25% up
            volume=50000000,
            avg_volume=10000000,
            volume_ratio=5.0,
        )

        alert = self.detector.analyze(social, market, historical_social=20)
        
        assert alert is not None
        signal_types = [s.signal_type for s in alert.signals]
        assert PumpSignalType.PRICE_SPIKE in signal_types

    def test_new_accounts_detection(self):
        """Test detection of activity from new accounts."""
        social = SocialMetrics(
            symbol="EMTK",
            post_count=40,
            unique_authors=30,
            avg_author_age_days=7,  # Very new accounts
            sentiment_score=0.7,
            keyword_flags=["pasti naik"],
        )
        market = MarketMetrics(
            symbol="EMTK",
            current_price=3000,
            price_change_pct=0.08,
            volume=5000000,
            avg_volume=3000000,
            volume_ratio=1.7,
        )

        alert = self.detector.analyze(social, market, historical_social=20)
        
        assert alert is not None
        signal_types = [s.signal_type for s in alert.signals]
        assert PumpSignalType.NEW_ACCOUNTS in signal_types

    def test_keyword_pattern_detection(self):
        """Test detection of pump keywords."""
        social = SocialMetrics(
            symbol="ARTO",
            post_count=25,
            unique_authors=20,
            avg_author_age_days=60,
            sentiment_score=0.85,
            keyword_flags=["to the moon", "buruan beli", "pasti naik"],
        )
        market = MarketMetrics(
            symbol="ARTO",
            current_price=2000,
            price_change_pct=0.05,
            volume=3000000,
            avg_volume=2500000,
            volume_ratio=1.2,
        )

        alert = self.detector.analyze(social, market, historical_social=15)
        
        assert alert is not None
        signal_types = [s.signal_type for s in alert.signals]
        assert PumpSignalType.KEYWORD_PATTERN in signal_types

    def test_coordinated_posts_detection(self):
        """Test detection of coordinated posting."""
        social = SocialMetrics(
            symbol="BUKA",
            post_count=50,
            unique_authors=5,  # 10 posts per author on average
            avg_author_age_days=100,
            sentiment_score=0.8,
            keyword_flags=[],
        )
        market = MarketMetrics(
            symbol="BUKA",
            current_price=300,
            price_change_pct=0.10,
            volume=8000000,
            avg_volume=4000000,
            volume_ratio=2.0,
        )

        alert = self.detector.analyze(social, market, historical_social=20)
        
        assert alert is not None
        signal_types = [s.signal_type for s in alert.signals]
        assert PumpSignalType.COORDINATED_POSTS in signal_types

    def test_severity_levels(self):
        """Test that severity increases with more signals."""
        # Multiple strong signals
        social = SocialMetrics(
            symbol="MCAS",
            post_count=100,  # 10x baseline
            unique_authors=10,  # Coordinated
            avg_author_age_days=5,  # New accounts
            sentiment_score=0.95,
            keyword_flags=["to the moon", "buruan beli", "100%", "pasti naik"],
        )
        market = MarketMetrics(
            symbol="MCAS",
            current_price=500,
            price_change_pct=0.30,  # 30% up
            volume=20000000,
            avg_volume=2000000,
            volume_ratio=10.0,  # 10x volume
        )

        alert = self.detector.analyze(social, market, historical_social=10)
        
        assert alert is not None
        assert alert.severity in [AlertSeverity.HIGH, AlertSeverity.CRITICAL]
        assert len(alert.signals) >= 4

    def test_check_text_for_pump(self):
        """Test individual text checking."""
        pump_text = "Buruan beli XXXX, pasti naik to the moon! 🚀🚀🚀"
        normal_text = "BBCA earnings report looks solid, good fundamentals"

        is_pump, keywords = self.detector.check_text_for_pump(pump_text)
        assert is_pump
        assert len(keywords) >= 2

        is_pump, keywords = self.detector.check_text_for_pump(normal_text)
        assert not is_pump

    def test_legitimate_keywords_reduce_confidence(self):
        """Test that legitimate keywords reduce alert confidence."""
        social_with_legit = SocialMetrics(
            symbol="BMRI",
            post_count=50,
            unique_authors=40,
            avg_author_age_days=180,
            sentiment_score=0.7,
            keyword_flags=["bullish", "laporan keuangan", "dividen"],  # Has legitimate keywords
        )
        social_without_legit = SocialMetrics(
            symbol="BMRI",
            post_count=50,
            unique_authors=40,
            avg_author_age_days=180,
            sentiment_score=0.7,
            keyword_flags=["to the moon", "rocket"],
        )
        market = MarketMetrics(
            symbol="BMRI",
            current_price=6000,
            price_change_pct=0.05,
            volume=5000000,
            avg_volume=3000000,
            volume_ratio=1.7,
        )

        alert_with = self.detector.analyze(social_with_legit, market, historical_social=10)
        alert_without = self.detector.analyze(social_without_legit, market, historical_social=10)

        # Alert with legitimate keywords should have lower confidence
        if alert_with and alert_without:
            assert alert_with.confidence < alert_without.confidence

    def test_update_baseline(self):
        """Test baseline updating."""
        self.detector.update_baseline("TEST", social_avg=50, volume_avg=1000000)
        
        assert "TEST" in self.detector._social_baselines
        assert self.detector._social_baselines["TEST"] == 50
