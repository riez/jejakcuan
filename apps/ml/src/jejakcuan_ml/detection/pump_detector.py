"""Pump-and-dump detection system."""

from dataclasses import dataclass, field
from datetime import datetime, timedelta
from enum import Enum
from typing import Any


class PumpSignalType(str, Enum):
    """Types of pump signals."""

    SOCIAL_SPIKE = "social_spike"  # Sudden increase in social mentions
    VOLUME_SPIKE = "volume_spike"  # Abnormal trading volume
    PRICE_SPIKE = "price_spike"  # Rapid price increase
    COORDINATED_POSTS = "coordinated_posts"  # Multiple similar posts
    NEW_ACCOUNTS = "new_accounts"  # Activity from new accounts
    KEYWORD_PATTERN = "keyword_pattern"  # Pump-related keywords
    TIME_PATTERN = "time_pattern"  # Suspicious timing patterns


class AlertSeverity(str, Enum):
    """Alert severity levels."""

    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    CRITICAL = "critical"


@dataclass
class PumpSignal:
    """Individual pump signal detection."""

    signal_type: PumpSignalType
    symbol: str
    confidence: float  # 0.0-1.0
    evidence: str
    timestamp: datetime = field(default_factory=datetime.now)
    metadata: dict[str, Any] = field(default_factory=dict)


@dataclass
class PumpAlert:
    """Aggregated pump-and-dump alert."""

    symbol: str
    severity: AlertSeverity
    confidence: float
    signals: list[PumpSignal]
    message: str
    timestamp: datetime = field(default_factory=datetime.now)
    recommended_action: str = "Monitor closely"


@dataclass
class SocialMetrics:
    """Social media metrics for a symbol."""

    symbol: str
    post_count: int
    unique_authors: int
    avg_author_age_days: float
    sentiment_score: float
    keyword_flags: list[str]
    time_window_hours: float = 24


@dataclass
class MarketMetrics:
    """Market metrics for a symbol."""

    symbol: str
    current_price: float
    price_change_pct: float
    volume: int
    avg_volume: int
    volume_ratio: float
    market_cap: float | None = None


class PumpDetector:
    """Detect pump-and-dump schemes using multiple signals."""

    # Pump-related keywords (Indonesian + English)
    PUMP_KEYWORDS = {
        # Urgent calls to action
        "buruan beli",
        "cepat beli",
        "jangan sampai ketinggalan",
        "last chance",
        "limited time",
        "hari ini saja",
        "sekarang atau tidak",
        # Guarantees
        "pasti naik",
        "guaranteed",
        "100% profit",
        "no risk",
        "tanpa risiko",
        # Insider claims
        "insider info",
        "info dalam",
        "rahasia",
        "secret",
        "belum tersebar",
        # Extreme predictions
        "to the moon",
        "moon",
        "1000%",
        "10x",
        "100x",
        "terbang",
        "rocket",
        # Group coordination
        "mari borong",
        "ayo beli bareng",
        "pompa",
        "pump it",
    }

    # Legitimate keywords that reduce pump probability
    LEGITIMATE_KEYWORDS = {
        "laporan keuangan",
        "financial report",
        "earning",
        "fundamental",
        "dividen",
        "dividend",
        "annual report",
        "quarterly",
        "analyst",
    }

    def __init__(
        self,
        social_spike_threshold: float = 3.0,  # 3x normal activity
        volume_spike_threshold: float = 5.0,  # 5x avg volume
        price_spike_threshold: float = 0.15,  # 15% price move
        min_confidence_threshold: float = 0.5,
        new_account_days: int = 30,
    ) -> None:
        """Initialize detector.

        Args:
            social_spike_threshold: Multiplier for social activity spike
            volume_spike_threshold: Multiplier for volume spike
            price_spike_threshold: Percentage for price spike
            min_confidence_threshold: Minimum confidence to trigger alert
            new_account_days: Days to consider an account "new"
        """
        self.social_spike_threshold = social_spike_threshold
        self.volume_spike_threshold = volume_spike_threshold
        self.price_spike_threshold = price_spike_threshold
        self.min_confidence_threshold = min_confidence_threshold
        self.new_account_days = new_account_days

        # Historical baselines (would be loaded from database)
        self._social_baselines: dict[str, float] = {}
        self._volume_baselines: dict[str, float] = {}

    def analyze(
        self,
        social: SocialMetrics,
        market: MarketMetrics,
        historical_social: float | None = None,
    ) -> PumpAlert | None:
        """Analyze symbol for pump-and-dump signals.

        Args:
            social: Current social media metrics
            market: Current market metrics
            historical_social: Historical average post count (optional)

        Returns:
            PumpAlert if suspicious activity detected, None otherwise
        """
        signals: list[PumpSignal] = []
        symbol = social.symbol

        # Signal 1: Social media spike
        baseline = historical_social or self._social_baselines.get(symbol, 10)
        if social.post_count > baseline * self.social_spike_threshold:
            signals.append(
                PumpSignal(
                    signal_type=PumpSignalType.SOCIAL_SPIKE,
                    symbol=symbol,
                    confidence=min((social.post_count / baseline - 1) / 5, 1.0),
                    evidence=f"Social mentions {social.post_count} vs baseline {baseline:.0f}",
                    metadata={"ratio": social.post_count / baseline},
                )
            )

        # Signal 2: Volume spike
        if market.volume_ratio >= self.volume_spike_threshold:
            signals.append(
                PumpSignal(
                    signal_type=PumpSignalType.VOLUME_SPIKE,
                    symbol=symbol,
                    confidence=min((market.volume_ratio - 1) / 10, 1.0),
                    evidence=f"Volume {market.volume_ratio:.1f}x average",
                    metadata={"volume_ratio": market.volume_ratio},
                )
            )

        # Signal 3: Price spike
        if market.price_change_pct >= self.price_spike_threshold:
            signals.append(
                PumpSignal(
                    signal_type=PumpSignalType.PRICE_SPIKE,
                    symbol=symbol,
                    confidence=min(market.price_change_pct / 0.3, 1.0),
                    evidence=f"Price up {market.price_change_pct*100:.1f}%",
                    metadata={"price_change": market.price_change_pct},
                )
            )

        # Signal 4: New accounts activity
        if social.avg_author_age_days < self.new_account_days:
            new_account_ratio = 1 - (
                social.avg_author_age_days / self.new_account_days
            )
            signals.append(
                PumpSignal(
                    signal_type=PumpSignalType.NEW_ACCOUNTS,
                    symbol=symbol,
                    confidence=new_account_ratio,
                    evidence=f"Avg account age {social.avg_author_age_days:.0f} days",
                    metadata={"avg_age_days": social.avg_author_age_days},
                )
            )

        # Signal 5: Pump keywords
        keyword_hits = [kw for kw in social.keyword_flags if kw in self.PUMP_KEYWORDS]
        if keyword_hits:
            signals.append(
                PumpSignal(
                    signal_type=PumpSignalType.KEYWORD_PATTERN,
                    symbol=symbol,
                    confidence=min(len(keyword_hits) / 5, 1.0),
                    evidence=f"Pump keywords: {', '.join(keyword_hits[:3])}",
                    metadata={"keywords": keyword_hits},
                )
            )

        # Signal 6: Low unique authors (coordinated)
        if social.unique_authors > 0:
            post_per_author = social.post_count / social.unique_authors
            if post_per_author > 3:  # More than 3 posts per author on average
                signals.append(
                    PumpSignal(
                        signal_type=PumpSignalType.COORDINATED_POSTS,
                        symbol=symbol,
                        confidence=min((post_per_author - 3) / 7, 1.0),
                        evidence=f"Avg {post_per_author:.1f} posts per author",
                        metadata={"posts_per_author": post_per_author},
                    )
                )

        # Calculate overall confidence
        if not signals:
            return None

        # Weighted average of signal confidences
        weights = {
            PumpSignalType.SOCIAL_SPIKE: 0.20,
            PumpSignalType.VOLUME_SPIKE: 0.25,
            PumpSignalType.PRICE_SPIKE: 0.20,
            PumpSignalType.NEW_ACCOUNTS: 0.10,
            PumpSignalType.KEYWORD_PATTERN: 0.15,
            PumpSignalType.COORDINATED_POSTS: 0.10,
        }

        total_weight = sum(weights.get(s.signal_type, 0.1) for s in signals)
        weighted_confidence = (
            sum(
                s.confidence * weights.get(s.signal_type, 0.1)
                for s in signals
            )
            / total_weight
        )

        # Adjust for legitimate signals
        legitimate_count = sum(
            1 for kw in social.keyword_flags if kw in self.LEGITIMATE_KEYWORDS
        )
        if legitimate_count > 0:
            weighted_confidence *= 0.7  # Reduce confidence if legitimate keywords present

        if weighted_confidence < self.min_confidence_threshold:
            return None

        # Determine severity
        severity = self._determine_severity(weighted_confidence, len(signals), market)

        return PumpAlert(
            symbol=symbol,
            severity=severity,
            confidence=weighted_confidence,
            signals=signals,
            message=self._generate_message(symbol, signals, severity),
            recommended_action=self._get_recommended_action(severity),
        )

    def _determine_severity(
        self,
        confidence: float,
        signal_count: int,
        market: MarketMetrics,
    ) -> AlertSeverity:
        """Determine alert severity based on factors."""
        # Higher confidence = higher severity
        if confidence >= 0.9:
            return AlertSeverity.CRITICAL
        elif confidence >= 0.75:
            return AlertSeverity.HIGH
        elif confidence >= 0.6:
            return AlertSeverity.MEDIUM
        return AlertSeverity.LOW

    def _generate_message(
        self,
        symbol: str,
        signals: list[PumpSignal],
        severity: AlertSeverity,
    ) -> str:
        """Generate human-readable alert message."""
        signal_summaries = [s.evidence for s in signals[:3]]
        summary = "; ".join(signal_summaries)

        return (
            f"Potential pump-and-dump detected for {symbol}. "
            f"Severity: {severity.value}. "
            f"Signals: {summary}"
        )

    def _get_recommended_action(self, severity: AlertSeverity) -> str:
        """Get recommended action based on severity."""
        actions = {
            AlertSeverity.CRITICAL: "Avoid trading. Report to authorities if applicable.",
            AlertSeverity.HIGH: "Avoid buying. Wait for activity to normalize.",
            AlertSeverity.MEDIUM: "Exercise caution. Do additional research.",
            AlertSeverity.LOW: "Monitor situation. Normal trading with awareness.",
        }
        return actions.get(severity, "Monitor closely")

    def check_text_for_pump(self, text: str) -> tuple[bool, list[str]]:
        """Check individual text for pump indicators.

        Args:
            text: Text to analyze

        Returns:
            Tuple of (is_suspicious, list of matched keywords)
        """
        text_lower = text.lower()
        matched = [kw for kw in self.PUMP_KEYWORDS if kw in text_lower]
        return len(matched) >= 2, matched

    def update_baseline(self, symbol: str, social_avg: float, volume_avg: float) -> None:
        """Update historical baselines for a symbol.

        Args:
            symbol: Stock symbol
            social_avg: Average social mentions
            volume_avg: Average trading volume
        """
        self._social_baselines[symbol] = social_avg
        self._volume_baselines[symbol] = volume_avg
