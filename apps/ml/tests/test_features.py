"""Tests for feature engineering."""

import numpy as np
import pandas as pd
import pytest

from jejakcuan_ml.features import SequenceBuilder, TechnicalFeatureExtractor
from jejakcuan_ml.features.sequence import DataSplitter


@pytest.fixture
def sample_ohlcv() -> pd.DataFrame:
    """Create sample OHLCV data."""
    np.random.seed(42)
    n = 100

    # Generate random walk price data
    returns = np.random.randn(n) * 0.02
    close = 1000 * np.exp(np.cumsum(returns))

    df = pd.DataFrame(
        {
            "open": close * (1 + np.random.randn(n) * 0.005),
            "high": close * (1 + np.abs(np.random.randn(n)) * 0.01),
            "low": close * (1 - np.abs(np.random.randn(n)) * 0.01),
            "close": close,
            "volume": np.random.randint(1000000, 10000000, n),
        }
    )

    return df


class TestTechnicalFeatureExtractor:
    """Tests for TechnicalFeatureExtractor."""

    def test_extract_features(self, sample_ohlcv: pd.DataFrame) -> None:
        extractor = TechnicalFeatureExtractor()
        features = extractor.extract(sample_ohlcv)

        # Check all features present
        assert len(extractor.feature_names) == features.shape[1]

        # Check no NaN after dropna
        assert not features.isna().any().any()

        # Check reasonable value ranges
        assert features["rsi"].between(0, 1).all()
        assert features["bb_position"].between(0, 1).all()

    def test_create_labels(self, sample_ohlcv: pd.DataFrame) -> None:
        extractor = TechnicalFeatureExtractor()
        labels = extractor.create_labels(sample_ohlcv, horizon=5, threshold=0.02)

        # Check label values
        assert set(labels.dropna().unique()).issubset({0, 1, 2})

        # Check length
        assert len(labels) == len(sample_ohlcv)


class TestSequenceBuilder:
    """Tests for SequenceBuilder."""

    def test_build_sequences(self, sample_ohlcv: pd.DataFrame) -> None:
        extractor = TechnicalFeatureExtractor()
        features = extractor.extract(sample_ohlcv).values
        labels = np.random.randint(0, 3, len(features))

        builder = SequenceBuilder(sequence_length=30)
        x_seqs, y_seqs = builder.build_sequences(features, labels)

        # Check shapes
        assert x_seqs.shape[1] == 30
        assert x_seqs.shape[2] == len(extractor.feature_names)
        assert len(x_seqs) == len(y_seqs)

    def test_build_inference_sequence(self, sample_ohlcv: pd.DataFrame) -> None:
        extractor = TechnicalFeatureExtractor()
        features = extractor.extract(sample_ohlcv).values

        builder = SequenceBuilder(sequence_length=30)
        seq = builder.build_inference_sequence(features)

        # Check shape
        assert seq.shape == (1, 30, features.shape[1])


class TestDataSplitter:
    """Tests for DataSplitter."""

    def test_split_temporal(self) -> None:
        features = np.random.randn(100, 30, 10)
        labels = np.random.randint(0, 3, 100)

        (train_feat, _), (val_feat, _), (test_feat, _) = DataSplitter.split_temporal(
            features, labels, train_ratio=0.7, val_ratio=0.15
        )

        assert len(train_feat) == 70
        assert len(val_feat) == 15
        assert len(test_feat) == 15
