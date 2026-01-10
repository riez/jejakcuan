"""Sequence building for LSTM input."""

from typing import Any

import numpy as np
from numpy.typing import NDArray


class SequenceBuilder:
    """Build sequences from feature arrays for LSTM training."""

    def __init__(
        self,
        sequence_length: int = 30,
        stride: int = 1,
    ) -> None:
        """Initialize sequence builder.

        Args:
            sequence_length: Number of time steps in each sequence
            stride: Step size between sequences
        """
        self.sequence_length = sequence_length
        self.stride = stride

    def build_sequences(
        self,
        features: NDArray[Any],
        labels: NDArray[Any],
    ) -> tuple[NDArray[Any], NDArray[Any]]:
        """Build sequences from feature array.

        Args:
            features: Shape (num_samples, num_features)
            labels: Shape (num_samples,)

        Returns:
            x_seqs: Shape (num_sequences, sequence_length, num_features)
            y_seqs: Shape (num_sequences,)
        """
        x_seqs, y_seqs = [], []

        for i in range(0, len(features) - self.sequence_length, self.stride):
            x_seqs.append(features[i : i + self.sequence_length])
            y_seqs.append(labels[i + self.sequence_length - 1])

        return np.array(x_seqs), np.array(y_seqs)

    def build_inference_sequence(
        self,
        features: NDArray[Any],
    ) -> NDArray[Any]:
        """Build single sequence for inference.

        Args:
            features: Shape (>=sequence_length, num_features)

        Returns:
            Shape (1, sequence_length, num_features)
        """
        if len(features) < self.sequence_length:
            raise ValueError(f"Need at least {self.sequence_length} samples, got {len(features)}")

        # Take last sequence_length samples
        seq = features[-self.sequence_length :]
        return seq[np.newaxis, ...]  # Add batch dimension


class DataSplitter:
    """Split time series data for training/validation/testing."""

    @staticmethod
    def split_temporal(
        features: NDArray[Any],
        labels: NDArray[Any],
        train_ratio: float = 0.7,
        val_ratio: float = 0.15,
    ) -> tuple[
        tuple[NDArray[Any], NDArray[Any]],
        tuple[NDArray[Any], NDArray[Any]],
        tuple[NDArray[Any], NDArray[Any]],
    ]:
        """Split data temporally (no shuffling for time series).

        Args:
            features: Feature sequences
            labels: Labels
            train_ratio: Fraction for training
            val_ratio: Fraction for validation

        Returns:
            (train_features, train_labels), (val_features, val_labels), (test_features, test_labels)
        """
        n = len(features)
        train_end = int(n * train_ratio)
        val_end = int(n * (train_ratio + val_ratio))

        return (
            (features[:train_end], labels[:train_end]),
            (features[train_end:val_end], labels[train_end:val_end]),
            (features[val_end:], labels[val_end:]),
        )

    @staticmethod
    def split_by_date(
        features: NDArray[Any],
        labels: NDArray[Any],
        dates: list[str],
        train_end_date: str,
        val_end_date: str,
    ) -> tuple[
        tuple[NDArray[Any], NDArray[Any]],
        tuple[NDArray[Any], NDArray[Any]],
        tuple[NDArray[Any], NDArray[Any]],
    ]:
        """Split data by specific dates.

        Args:
            features: Feature sequences
            labels: Labels
            dates: List of date strings corresponding to sequences
            train_end_date: Last date for training
            val_end_date: Last date for validation

        Returns:
            (train_features, train_labels), (val_features, val_labels), (test_features, test_labels)
        """
        dates_arr = np.array(dates)

        train_mask = dates_arr <= train_end_date
        val_mask = (dates_arr > train_end_date) & (dates_arr <= val_end_date)
        test_mask = dates_arr > val_end_date

        return (
            (features[train_mask], labels[train_mask]),
            (features[val_mask], labels[val_mask]),
            (features[test_mask], labels[test_mask]),
        )
