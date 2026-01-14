"""Anomaly detection for stock market data."""

from dataclasses import dataclass
from datetime import datetime
from enum import Enum
from typing import Any

import numpy as np
from numpy.typing import NDArray


class AnomalyType(str, Enum):
    """Types of market anomalies."""

    PRICE_SPIKE = "price_spike"
    VOLUME_SPIKE = "volume_spike"
    VOLATILITY_EXPLOSION = "volatility_explosion"
    GAP_ANOMALY = "gap_anomaly"
    CORRELATION_BREAK = "correlation_break"
    PATTERN_DEVIATION = "pattern_deviation"


@dataclass
class Anomaly:
    """Detected market anomaly."""

    anomaly_type: AnomalyType
    symbol: str
    timestamp: datetime
    severity: float  # 0-1 scale
    z_score: float
    description: str
    value: float  # The anomalous value
    expected_range: tuple[float, float]


@dataclass
class AnomalyConfig:
    """Configuration for anomaly detection."""

    price_z_threshold: float = 3.0
    volume_z_threshold: float = 4.0
    volatility_z_threshold: float = 3.5
    gap_z_threshold: float = 3.0
    lookback_period: int = 60  # Days for computing baselines
    min_severity: float = 0.5  # Minimum severity to report


class AnomalyDetector:
    """Detect market anomalies using statistical methods."""

    def __init__(self, config: AnomalyConfig | None = None) -> None:
        """Initialize detector.

        Args:
            config: Detection configuration
        """
        self.config = config or AnomalyConfig()

    def detect_all(
        self,
        symbol: str,
        prices: NDArray[Any],
        volumes: NDArray[Any],
        highs: NDArray[Any],
        lows: NDArray[Any],
        opens: NDArray[Any],
    ) -> list[Anomaly]:
        """Run all anomaly detection methods.

        Args:
            symbol: Stock symbol
            prices: Close prices array
            volumes: Volume array
            highs: High prices array
            lows: Low prices array
            opens: Open prices array

        Returns:
            List of detected anomalies
        """
        anomalies = []

        # Price anomalies
        price_anomalies = self.detect_price_anomalies(symbol, prices)
        anomalies.extend(price_anomalies)

        # Volume anomalies
        volume_anomalies = self.detect_volume_anomalies(symbol, volumes)
        anomalies.extend(volume_anomalies)

        # Volatility anomalies
        volatility_anomalies = self.detect_volatility_anomalies(
            symbol, highs, lows, prices
        )
        anomalies.extend(volatility_anomalies)

        # Gap anomalies
        gap_anomalies = self.detect_gap_anomalies(symbol, opens, prices)
        anomalies.extend(gap_anomalies)

        # Filter by minimum severity
        return [a for a in anomalies if a.severity >= self.config.min_severity]

    def detect_price_anomalies(
        self,
        symbol: str,
        prices: NDArray[Any],
    ) -> list[Anomaly]:
        """Detect abnormal price movements.

        Uses rolling z-score to identify price spikes.
        """
        if len(prices) < self.config.lookback_period + 1:
            return []

        anomalies = []
        returns = np.diff(prices) / prices[:-1]

        for i in range(self.config.lookback_period, len(returns)):
            lookback = returns[i - self.config.lookback_period : i]
            mean_ret = np.mean(lookback)
            std_ret = np.std(lookback)

            if std_ret < 1e-10:
                continue

            z_score = (returns[i] - mean_ret) / std_ret

            if abs(z_score) >= self.config.price_z_threshold:
                severity = min(abs(z_score) / 5.0, 1.0)
                anomalies.append(
                    Anomaly(
                        anomaly_type=AnomalyType.PRICE_SPIKE,
                        symbol=symbol,
                        timestamp=datetime.now(),  # In production, use actual date
                        severity=severity,
                        z_score=float(z_score),
                        description=f"Price moved {z_score:.1f} std from normal",
                        value=float(returns[i]),
                        expected_range=(
                            float(mean_ret - 2 * std_ret),
                            float(mean_ret + 2 * std_ret),
                        ),
                    )
                )

        return anomalies

    def detect_volume_anomalies(
        self,
        symbol: str,
        volumes: NDArray[Any],
    ) -> list[Anomaly]:
        """Detect abnormal trading volumes."""
        if len(volumes) < self.config.lookback_period + 1:
            return []

        anomalies = []

        for i in range(self.config.lookback_period, len(volumes)):
            lookback = volumes[i - self.config.lookback_period : i]
            mean_vol = np.mean(lookback)
            std_vol = np.std(lookback)

            if std_vol < 1e-10 or mean_vol < 1e-10:
                continue

            z_score = (volumes[i] - mean_vol) / std_vol

            if z_score >= self.config.volume_z_threshold:
                severity = min(z_score / 8.0, 1.0)
                anomalies.append(
                    Anomaly(
                        anomaly_type=AnomalyType.VOLUME_SPIKE,
                        symbol=symbol,
                        timestamp=datetime.now(),
                        severity=severity,
                        z_score=float(z_score),
                        description=f"Volume {volumes[i]/mean_vol:.1f}x normal",
                        value=float(volumes[i]),
                        expected_range=(
                            float(mean_vol - 2 * std_vol),
                            float(mean_vol + 2 * std_vol),
                        ),
                    )
                )

        return anomalies

    def detect_volatility_anomalies(
        self,
        symbol: str,
        highs: NDArray[Any],
        lows: NDArray[Any],
        closes: NDArray[Any],
    ) -> list[Anomaly]:
        """Detect abnormal volatility (ATR-based)."""
        if len(highs) < self.config.lookback_period + 1:
            return []

        # Calculate True Range
        tr = np.maximum(
            highs[1:] - lows[1:],
            np.maximum(
                np.abs(highs[1:] - closes[:-1]),
                np.abs(lows[1:] - closes[:-1]),
            ),
        )

        anomalies = []

        for i in range(self.config.lookback_period, len(tr)):
            lookback = tr[i - self.config.lookback_period : i]
            mean_tr = np.mean(lookback)
            std_tr = np.std(lookback)

            if std_tr < 1e-10:
                continue

            z_score = (tr[i] - mean_tr) / std_tr

            if z_score >= self.config.volatility_z_threshold:
                severity = min(z_score / 6.0, 1.0)
                anomalies.append(
                    Anomaly(
                        anomaly_type=AnomalyType.VOLATILITY_EXPLOSION,
                        symbol=symbol,
                        timestamp=datetime.now(),
                        severity=severity,
                        z_score=float(z_score),
                        description=f"Volatility {tr[i]/mean_tr:.1f}x normal",
                        value=float(tr[i]),
                        expected_range=(
                            float(mean_tr - 2 * std_tr),
                            float(mean_tr + 2 * std_tr),
                        ),
                    )
                )

        return anomalies

    def detect_gap_anomalies(
        self,
        symbol: str,
        opens: NDArray[Any],
        closes: NDArray[Any],
    ) -> list[Anomaly]:
        """Detect abnormal overnight gaps."""
        if len(opens) < self.config.lookback_period + 2:
            return []

        # Calculate gaps (open vs previous close)
        gaps = (opens[1:] - closes[:-1]) / closes[:-1]

        anomalies = []

        for i in range(self.config.lookback_period, len(gaps)):
            lookback = gaps[i - self.config.lookback_period : i]
            mean_gap = np.mean(lookback)
            std_gap = np.std(lookback)

            if std_gap < 1e-10:
                continue

            z_score = (gaps[i] - mean_gap) / std_gap

            if abs(z_score) >= self.config.gap_z_threshold:
                severity = min(abs(z_score) / 5.0, 1.0)
                direction = "up" if gaps[i] > 0 else "down"
                anomalies.append(
                    Anomaly(
                        anomaly_type=AnomalyType.GAP_ANOMALY,
                        symbol=symbol,
                        timestamp=datetime.now(),
                        severity=severity,
                        z_score=float(z_score),
                        description=f"Gap {direction} {abs(gaps[i])*100:.1f}%",
                        value=float(gaps[i]),
                        expected_range=(
                            float(mean_gap - 2 * std_gap),
                            float(mean_gap + 2 * std_gap),
                        ),
                    )
                )

        return anomalies

    def compute_anomaly_score(
        self,
        symbol: str,
        prices: NDArray[Any],
        volumes: NDArray[Any],
    ) -> float:
        """Compute overall anomaly score for recent data.

        Args:
            symbol: Stock symbol
            prices: Close prices
            volumes: Volumes

        Returns:
            Anomaly score 0-1 (higher = more anomalous)
        """
        if len(prices) < self.config.lookback_period + 5:
            return 0.0

        scores = []

        # Price deviation score
        recent_returns = np.diff(prices[-6:]) / prices[-6:-1]
        lookback_returns = np.diff(prices[:-5]) / prices[1:-5]
        if len(lookback_returns) > 0 and np.std(lookback_returns) > 0:
            price_z = (np.mean(recent_returns) - np.mean(lookback_returns)) / np.std(
                lookback_returns
            )
            scores.append(min(abs(price_z) / 3.0, 1.0))

        # Volume deviation score
        recent_vol = np.mean(volumes[-5:])
        lookback_vol = np.mean(volumes[:-5])
        if lookback_vol > 0:
            vol_ratio = recent_vol / lookback_vol
            vol_score = min(max(vol_ratio - 1, 0) / 3.0, 1.0)
            scores.append(vol_score)

        return np.mean(scores) if scores else 0.0


# Isolation Forest based anomaly detection (optional, requires sklearn)
class IsolationForestDetector:
    """Isolation Forest based anomaly detection."""

    def __init__(self, contamination: float = 0.05) -> None:
        """Initialize detector.

        Args:
            contamination: Expected proportion of anomalies
        """
        self.contamination = contamination
        self._model = None
        self._fitted = False

    def fit(self, features: NDArray[Any]) -> None:
        """Fit the model on historical data.

        Args:
            features: Shape (n_samples, n_features)
        """
        try:
            from sklearn.ensemble import IsolationForest

            self._model = IsolationForest(
                contamination=self.contamination,
                random_state=42,
                n_estimators=100,
            )
            self._model.fit(features)
            self._fitted = True
        except ImportError:
            print("sklearn not installed. Install with: pip install scikit-learn")
            self._fitted = False

    def predict(self, features: NDArray[Any]) -> NDArray[Any]:
        """Predict anomaly scores.

        Args:
            features: Shape (n_samples, n_features)

        Returns:
            Array of -1 (anomaly) or 1 (normal)
        """
        if not self._fitted or self._model is None:
            return np.ones(len(features))
        return self._model.predict(features)

    def score_samples(self, features: NDArray[Any]) -> NDArray[Any]:
        """Get anomaly scores (lower = more anomalous).

        Args:
            features: Shape (n_samples, n_features)

        Returns:
            Array of anomaly scores
        """
        if not self._fitted or self._model is None:
            return np.zeros(len(features))
        return self._model.score_samples(features)
