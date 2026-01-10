"""LSTM model for price prediction."""

from datetime import datetime
from pathlib import Path
from typing import Any

import numpy as np
import numpy.typing as npt
import torch
import torch.nn as nn

from ..schemas import Direction, PricePredictionResponse


class StockLSTM(nn.Module):
    """LSTM network for stock price direction prediction."""

    def __init__(
        self,
        input_size: int = 10,  # Number of features
        hidden_size: int = 64,
        num_layers: int = 2,
        num_classes: int = 3,  # UP, DOWN, SIDEWAYS
        dropout: float = 0.2,
    ):
        super().__init__()
        self.hidden_size = hidden_size
        self.num_layers = num_layers

        self.lstm = nn.LSTM(
            input_size=input_size,
            hidden_size=hidden_size,
            num_layers=num_layers,
            batch_first=True,
            dropout=dropout if num_layers > 1 else 0,
        )

        self.fc = nn.Sequential(
            nn.Linear(hidden_size, hidden_size // 2),
            nn.ReLU(),
            nn.Dropout(dropout),
            nn.Linear(hidden_size // 2, num_classes),
        )

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        """Forward pass.

        Args:
            x: Input tensor of shape (batch, seq_len, features)

        Returns:
            Output tensor of shape (batch, num_classes)
        """
        # LSTM output
        lstm_out, (h_n, c_n) = self.lstm(x)

        # Use last hidden state
        out = self.fc(h_n[-1])
        return out


class LSTMPredictor:
    """LSTM-based price direction predictor."""

    def __init__(self) -> None:
        """Initialize predictor."""
        self.model: StockLSTM | None = None
        self.version = "1.0.0"
        self.loaded = False
        self.device = torch.device("cuda" if torch.cuda.is_available() else "cpu")

        # Feature configuration
        self.sequence_length = 30  # 30 days lookback
        self.feature_names = [
            "returns",  # Daily returns
            "volume_change",  # Volume change %
            "rsi",  # RSI (14)
            "macd_signal",  # MACD signal line
            "bb_position",  # Position in Bollinger Bands
            "ema_ratio",  # Close / EMA20 ratio
            "volume_ratio",  # Volume / Avg volume ratio
            "high_low_ratio",  # (High - Low) / Close
            "gap",  # (Open - Prev Close) / Prev Close
            "trend",  # EMA20 slope
        ]

        # Normalization stats (should be loaded with model)
        self.feature_means: np.ndarray | None = None
        self.feature_stds: np.ndarray | None = None

    def load(self, model_path: str) -> bool:
        """Load model from path.

        Args:
            model_path: Path to saved model directory

        Returns:
            True if loaded successfully
        """
        try:
            path = Path(model_path)

            # Load model architecture
            self.model = StockLSTM(
                input_size=len(self.feature_names),
                hidden_size=64,
                num_layers=2,
                num_classes=3,
            )

            # Load weights if available
            weights_path = path / "model.pt"
            if weights_path.exists():
                state_dict = torch.load(weights_path, map_location=self.device, weights_only=True)
                self.model.load_state_dict(state_dict)

            # Load normalization stats
            stats_path = path / "stats.npz"
            if stats_path.exists():
                stats = np.load(stats_path)
                self.feature_means = stats["means"]
                self.feature_stds = stats["stds"]
            else:
                # Default stats (will be overwritten during training)
                self.feature_means = np.zeros(len(self.feature_names))
                self.feature_stds = np.ones(len(self.feature_names))

            self.model.to(self.device)
            self.model.eval()
            self.loaded = True
            return True

        except Exception as e:
            print(f"Failed to load model: {e}")
            self.loaded = False
            return False

    def normalize_features(self, features: np.ndarray) -> np.ndarray:
        """Normalize features using saved statistics."""
        if self.feature_means is None or self.feature_stds is None:
            return features
        return (features - self.feature_means) / (self.feature_stds + 1e-8)

    def predict(self, symbol: str, horizon_days: int = 5) -> PricePredictionResponse:
        """Predict price direction for a stock.

        Note: In production, this would fetch actual price data.
        For now, returns placeholder with proper structure.
        """
        if not self.loaded or self.model is None:
            return PricePredictionResponse(
                symbol=symbol,
                timestamp=datetime.utcnow(),
                direction=Direction.SIDEWAYS,
                confidence=0.0,
                horizon_days=horizon_days,
                model_version=self.version,
            )

        # In production: fetch features from database
        # For now, return placeholder indicating model is ready
        return PricePredictionResponse(
            symbol=symbol,
            timestamp=datetime.utcnow(),
            direction=Direction.SIDEWAYS,
            confidence=0.5,
            horizon_days=horizon_days,
            model_version=self.version,
        )

    def predict_from_features(
        self,
        features: npt.NDArray[Any],
        symbol: str = "UNKNOWN",
        horizon_days: int = 5,
    ) -> PricePredictionResponse:
        """Predict from pre-computed feature sequence.

        Args:
            features: Shape (seq_len, num_features) or (batch, seq_len, num_features)
            symbol: Stock symbol
            horizon_days: Prediction horizon

        Returns:
            Prediction response with probabilities
        """
        if not self.loaded or self.model is None:
            raise RuntimeError("Model not loaded")

        # Ensure correct shape
        if features.ndim == 2:
            features = features[np.newaxis, ...]  # Add batch dimension

        # Normalize
        features = self.normalize_features(features)

        # Convert to tensor
        x = torch.FloatTensor(features).to(self.device)

        # Predict
        with torch.no_grad():
            logits = self.model(x)
            probs = torch.softmax(logits, dim=-1)
            pred_class = torch.argmax(probs, dim=-1).item()
            confidence = probs[0, pred_class].item()

        # Map class to direction
        direction_map = {0: Direction.DOWN, 1: Direction.SIDEWAYS, 2: Direction.UP}
        direction = direction_map.get(int(pred_class), Direction.SIDEWAYS)

        # Extract probabilities
        prob_values = probs[0].cpu().numpy()

        return PricePredictionResponse(
            symbol=symbol,
            timestamp=datetime.utcnow(),
            direction=direction,
            confidence=confidence,
            horizon_days=horizon_days,
            model_version=self.version,
            probabilities={
                "down": float(prob_values[0]),
                "sideways": float(prob_values[1]),
                "up": float(prob_values[2]),
            },
        )

    def save(self, model_path: str) -> None:
        """Save model to path."""
        if self.model is None:
            raise RuntimeError("No model to save")

        path = Path(model_path)
        path.mkdir(parents=True, exist_ok=True)

        # Save weights
        torch.save(self.model.state_dict(), path / "model.pt")

        # Save normalization stats
        if self.feature_means is not None and self.feature_stds is not None:
            np.savez(
                path / "stats.npz",
                means=self.feature_means,
                stds=self.feature_stds,
            )


# Global instance
lstm_predictor = LSTMPredictor()
