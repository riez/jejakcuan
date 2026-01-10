"""Technical feature extraction from OHLCV data."""

from typing import Any

import numpy as np
import pandas as pd
from numpy.typing import NDArray
from pandas import Series


class TechnicalFeatureExtractor:
    """Extract technical features from OHLCV data for ML models."""

    def __init__(self) -> None:
        """Initialize feature extractor."""
        self.feature_names = [
            "returns",  # Daily returns
            "volume_change",  # Volume change %
            "rsi",  # RSI (14)
            "macd_signal",  # MACD signal (normalized)
            "bb_position",  # Position in Bollinger Bands
            "ema_ratio",  # Close / EMA20 ratio
            "volume_ratio",  # Volume / Avg volume ratio
            "high_low_ratio",  # (High - Low) / Close
            "gap",  # (Open - Prev Close) / Prev Close
            "trend",  # EMA20 slope (normalized)
        ]

    def extract(
        self,
        df: pd.DataFrame,
        dropna: bool = True,
    ) -> pd.DataFrame:
        """Extract features from OHLCV DataFrame.

        Args:
            df: DataFrame with columns ['open', 'high', 'low', 'close', 'volume']
            dropna: Whether to drop rows with NaN values

        Returns:
            DataFrame with feature columns
        """
        # Ensure lowercase columns
        df = df.copy()
        df.columns = df.columns.str.lower()

        features = pd.DataFrame(index=df.index)

        # Daily returns
        features["returns"] = df["close"].pct_change()

        # Volume change
        features["volume_change"] = df["volume"].pct_change()

        # RSI (14 period)
        features["rsi"] = self._calculate_rsi(df["close"], period=14) / 100  # Normalize to 0-1

        # MACD signal (normalized)
        macd, signal = self._calculate_macd(df["close"])
        features["macd_signal"] = (macd - signal) / df["close"]  # Normalize by price

        # Bollinger Bands position
        features["bb_position"] = self._calculate_bb_position(df["close"])

        # EMA20 ratio
        ema20 = df["close"].ewm(span=20, adjust=False).mean()
        features["ema_ratio"] = (df["close"] / ema20) - 1

        # Volume ratio (vs 20-day average)
        avg_volume = df["volume"].rolling(window=20).mean()
        features["volume_ratio"] = df["volume"] / avg_volume - 1

        # High-Low ratio (volatility measure)
        features["high_low_ratio"] = (df["high"] - df["low"]) / df["close"]

        # Gap (overnight return)
        features["gap"] = (df["open"] - df["close"].shift(1)) / df["close"].shift(1)

        # Trend (EMA20 slope, normalized)
        features["trend"] = ema20.diff(5) / ema20

        # Handle infinities
        features = features.replace([np.inf, -np.inf], np.nan)

        if dropna:
            features = features.dropna()

        return features

    def _calculate_rsi(self, prices: "Series[Any]", period: int = 14) -> "Series[Any]":
        """Calculate Relative Strength Index."""
        delta = prices.diff()
        gain = (delta.where(delta > 0, 0)).rolling(window=period).mean()  # type: ignore[operator]
        loss = (-delta.where(delta < 0, 0)).rolling(window=period).mean()  # type: ignore[operator]

        rs = gain / (loss + 1e-10)
        rsi = 100 - (100 / (1 + rs))
        return rsi

    def _calculate_macd(
        self,
        prices: "Series[Any]",
        fast: int = 12,
        slow: int = 26,
        signal: int = 9,
    ) -> tuple["Series[Any]", "Series[Any]"]:
        """Calculate MACD and signal line."""
        ema_fast = prices.ewm(span=fast, adjust=False).mean()
        ema_slow = prices.ewm(span=slow, adjust=False).mean()
        macd = ema_fast - ema_slow
        signal_line = macd.ewm(span=signal, adjust=False).mean()
        return macd, signal_line

    def _calculate_bb_position(
        self,
        prices: "Series[Any]",
        period: int = 20,
        std_dev: float = 2.0,
    ) -> "Series[Any]":
        """Calculate position within Bollinger Bands (0 to 1)."""
        sma = prices.rolling(window=period).mean()
        std = prices.rolling(window=period).std()

        upper = sma + (std_dev * std)
        lower = sma - (std_dev * std)

        # Position: 0 = at lower band, 1 = at upper band
        position = (prices - lower) / (upper - lower + 1e-10)
        return position.clip(0, 1)

    def create_labels(
        self,
        df: pd.DataFrame,
        horizon: int = 5,
        threshold: float = 0.02,
    ) -> "Series[int]":
        """Create classification labels based on future returns.

        Args:
            df: DataFrame with 'close' column
            horizon: Days ahead to predict
            threshold: Return threshold for UP/DOWN classification

        Returns:
            Series with labels: 0=DOWN, 1=SIDEWAYS, 2=UP
        """
        df = df.copy()
        df.columns = df.columns.str.lower()

        # Future returns
        future_returns: Series[float] = df["close"].shift(-horizon) / df["close"] - 1

        # Classify
        labels: Series[int] = pd.Series(1, index=df.index)  # Default: SIDEWAYS
        labels = labels.mask(future_returns > threshold, 2)  # UP
        labels = labels.mask(future_returns < -threshold, 0)  # DOWN

        return labels


def extract_features_for_symbol(
    ohlcv_data: list[dict[str, Any]],
    horizon: int = 5,
) -> tuple[NDArray[Any], NDArray[Any], list[str]]:
    """Convenience function to extract features from OHLCV data.

    Args:
        ohlcv_data: List of dicts with keys: date, open, high, low, close, volume
        horizon: Prediction horizon for labels

    Returns:
        features: (num_samples, num_features) array
        labels: (num_samples,) array
        dates: List of date strings
    """
    # Convert to DataFrame
    df = pd.DataFrame(ohlcv_data)
    df["date"] = pd.to_datetime(df["date"])
    df = df.set_index("date").sort_index()

    # Extract features
    extractor = TechnicalFeatureExtractor()
    features_df = extractor.extract(df, dropna=False)
    labels = extractor.create_labels(df, horizon=horizon)

    # Align indices
    common_idx = features_df.dropna().index.intersection(labels.dropna().index)
    # Remove last `horizon` samples (no labels)
    common_idx = common_idx[:-horizon] if len(common_idx) > horizon else common_idx[:0]

    features_df = features_df.loc[common_idx]
    labels = labels.loc[common_idx]

    return (
        features_df.values,
        labels.values.astype(int),
        [str(d) for d in common_idx],
    )
