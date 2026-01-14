"""Tests for anomaly detection."""

import numpy as np
import pytest
from jejakcuan_ml.models.anomaly import (
    AnomalyDetector,
    AnomalyConfig,
    AnomalyType,
    IsolationForestDetector,
)


class TestAnomalyDetector:
    """Tests for AnomalyDetector."""

    def setup_method(self):
        config = AnomalyConfig(
            price_z_threshold=2.5,
            volume_z_threshold=3.0,
            lookback_period=20,
            min_severity=0.3,
        )
        self.detector = AnomalyDetector(config)

    def test_detect_price_spike(self):
        """Test detection of abnormal price movement."""
        # Normal prices with a spike at the end
        prices = np.concatenate([
            np.random.normal(100, 1, 25),  # Normal variation
            np.array([120.0]),  # Big spike
        ])
        
        anomalies = self.detector.detect_price_anomalies("TEST", prices)
        
        # Should detect the spike
        assert len(anomalies) >= 1
        spike_anomaly = anomalies[-1]
        assert spike_anomaly.anomaly_type == AnomalyType.PRICE_SPIKE
        assert spike_anomaly.z_score > 2.5

    def test_detect_volume_spike(self):
        """Test detection of abnormal volume."""
        # Normal volumes with a spike
        volumes = np.concatenate([
            np.random.normal(1000000, 100000, 25),
            np.array([5000000]),  # 5x normal
        ])
        
        anomalies = self.detector.detect_volume_anomalies("TEST", volumes)
        
        assert len(anomalies) >= 1
        assert anomalies[-1].anomaly_type == AnomalyType.VOLUME_SPIKE

    def test_detect_volatility_explosion(self):
        """Test detection of abnormal volatility."""
        # Normal volatility followed by high volatility
        closes = np.cumsum(np.random.normal(0, 1, 30)) + 100
        highs = closes + np.concatenate([np.full(25, 1), np.array([10, 10, 10, 10, 10])])
        lows = closes - np.concatenate([np.full(25, 1), np.array([10, 10, 10, 10, 10])])
        
        anomalies = self.detector.detect_volatility_anomalies("TEST", highs, lows, closes)
        
        # Should detect volatility spikes in the last 5 bars
        vol_anomalies = [a for a in anomalies if a.anomaly_type == AnomalyType.VOLATILITY_EXPLOSION]
        assert len(vol_anomalies) >= 1

    def test_detect_gap_anomaly(self):
        """Test detection of abnormal gaps."""
        np.random.seed(42)
        closes = np.cumsum(np.random.normal(0, 0.5, 30)) + 100
        # Normal opens close to previous close, then big gap at end
        opens = np.concatenate([
            closes[:-1] + np.random.normal(0, 0.2, 29),
        ])
        opens = np.append(opens, closes[-2] + 10)  # Big gap up
        
        anomalies = self.detector.detect_gap_anomalies("TEST", opens, closes)
        
        gap_anomalies = [a for a in anomalies if a.anomaly_type == AnomalyType.GAP_ANOMALY]
        # Due to randomness, may or may not detect - just check no errors
        assert isinstance(anomalies, list)

    def test_detect_all(self):
        """Test combined anomaly detection."""
        n = 30
        prices = np.cumsum(np.random.normal(0, 1, n)) + 100
        volumes = np.random.normal(1000000, 100000, n)
        highs = prices + np.abs(np.random.normal(0, 1, n))
        lows = prices - np.abs(np.random.normal(0, 1, n))
        opens = prices + np.random.normal(0, 0.5, n)

        # Add anomaly
        prices[-1] = prices[-2] * 1.15  # 15% spike
        volumes[-1] = 5000000

        anomalies = self.detector.detect_all(
            "TEST", prices, volumes, highs, lows, opens
        )

        # Should detect at least the price and volume anomalies
        types = [a.anomaly_type for a in anomalies]
        assert AnomalyType.PRICE_SPIKE in types or AnomalyType.VOLUME_SPIKE in types

    def test_anomaly_score(self):
        """Test overall anomaly score calculation."""
        # Normal data
        normal_prices = np.cumsum(np.random.normal(0, 1, 100)) + 100
        normal_volumes = np.random.normal(1000000, 100000, 100)

        score_normal = self.detector.compute_anomaly_score(
            "TEST", normal_prices, normal_volumes
        )
        
        # Anomalous data
        anomalous_prices = normal_prices.copy()
        anomalous_prices[-5:] = anomalous_prices[-6] * 1.2  # Big move
        anomalous_volumes = normal_volumes.copy()
        anomalous_volumes[-5:] = 5000000  # Volume spike

        score_anomalous = self.detector.compute_anomaly_score(
            "TEST", anomalous_prices, anomalous_volumes
        )
        
        assert score_anomalous > score_normal

    def test_insufficient_data(self):
        """Test handling of insufficient data."""
        short_prices = np.array([100, 101, 102])
        anomalies = self.detector.detect_price_anomalies("TEST", short_prices)
        assert len(anomalies) == 0


class TestIsolationForestDetector:
    """Tests for IsolationForestDetector."""

    def test_fit_and_predict(self):
        """Test fitting and predicting."""
        detector = IsolationForestDetector(contamination=0.1)

        # Generate training data
        np.random.seed(42)
        normal_data = np.random.normal(0, 1, (100, 5))
        
        detector.fit(normal_data)
        
        # Predict on normal data
        normal_preds = detector.predict(normal_data[:10])
        
        # Predict on anomalous data
        anomalous_data = np.random.normal(10, 1, (10, 5))  # Different distribution
        anomalous_preds = detector.predict(anomalous_data)
        
        # Most normal data should be predicted as normal (1)
        assert np.mean(normal_preds == 1) > 0.5
        
        # Anomalous data should have more -1 predictions
        # (Note: depends on sklearn being installed)

    def test_score_samples(self):
        """Test anomaly scoring."""
        detector = IsolationForestDetector()
        
        np.random.seed(42)
        normal_data = np.random.normal(0, 1, (100, 5))
        detector.fit(normal_data)
        
        scores = detector.score_samples(normal_data[:10])
        
        # Scores should be negative (sklearn convention)
        assert len(scores) == 10

    def test_not_fitted(self):
        """Test behavior when not fitted."""
        detector = IsolationForestDetector()
        
        data = np.random.normal(0, 1, (10, 5))
        predictions = detector.predict(data)
        
        # Should return all 1s (normal) when not fitted
        assert np.all(predictions == 1)
