"""LSTM model training pipeline."""

import json
from datetime import datetime
from pathlib import Path

import numpy as np
import torch
import torch.nn as nn
from torch.utils.data import DataLoader, TensorDataset

from ..models.lstm import StockLSTM


class LSTMTrainer:
    """Trainer for LSTM stock prediction model."""

    def __init__(
        self,
        input_size: int = 10,
        hidden_size: int = 64,
        num_layers: int = 2,
        sequence_length: int = 30,
        learning_rate: float = 0.001,
        batch_size: int = 32,
        epochs: int = 100,
        early_stopping_patience: int = 10,
    ):
        self.input_size = input_size
        self.hidden_size = hidden_size
        self.num_layers = num_layers
        self.sequence_length = sequence_length
        self.learning_rate = learning_rate
        self.batch_size = batch_size
        self.epochs = epochs
        self.early_stopping_patience = early_stopping_patience

        self.device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
        self.model: StockLSTM | None = None

        # Normalization stats
        self.feature_means: np.ndarray | None = None
        self.feature_stds: np.ndarray | None = None

    def create_sequences(
        self,
        features: np.ndarray,
        labels: np.ndarray,
    ) -> tuple[np.ndarray, np.ndarray]:
        """Create sequences for LSTM training.

        Args:
            features: Shape (num_samples, num_features)
            labels: Shape (num_samples,) - class labels (0, 1, 2)

        Returns:
            x_seq: Shape (num_sequences, seq_len, num_features)
            y_seq: Shape (num_sequences,)
        """
        x_seq, y_seq = [], []

        for i in range(len(features) - self.sequence_length):
            x_seq.append(features[i : i + self.sequence_length])
            y_seq.append(labels[i + self.sequence_length])

        return np.array(x_seq), np.array(y_seq)

    def normalize_features(self, features: np.ndarray, fit: bool = False) -> np.ndarray:
        """Normalize features.

        Args:
            features: Input features
            fit: If True, compute and store normalization stats

        Returns:
            Normalized features
        """
        if fit:
            self.feature_means = features.mean(axis=0)
            self.feature_stds = features.std(axis=0)

        if self.feature_means is None or self.feature_stds is None:
            return features

        return (features - self.feature_means) / (self.feature_stds + 1e-8)

    def train(
        self,
        x_train: np.ndarray,
        y_train: np.ndarray,
        x_val: np.ndarray | None = None,
        y_val: np.ndarray | None = None,
    ) -> dict:
        """Train the LSTM model.

        Args:
            x_train: Training features (num_samples, seq_len, num_features)
            y_train: Training labels (num_samples,)
            x_val: Validation features (optional)
            y_val: Validation labels (optional)

        Returns:
            Training history dict
        """
        # Create model
        self.model = StockLSTM(
            input_size=self.input_size,
            hidden_size=self.hidden_size,
            num_layers=self.num_layers,
            num_classes=3,
        ).to(self.device)

        # Loss and optimizer
        # Use class weights for imbalanced data
        class_counts = np.bincount(y_train.astype(int), minlength=3)
        class_weights = 1.0 / (class_counts + 1)
        class_weights = class_weights / class_weights.sum()
        weights = torch.FloatTensor(class_weights).to(self.device)

        criterion = nn.CrossEntropyLoss(weight=weights)
        optimizer = torch.optim.Adam(self.model.parameters(), lr=self.learning_rate)
        scheduler = torch.optim.lr_scheduler.ReduceLROnPlateau(
            optimizer, mode="min", factor=0.5, patience=5
        )

        # Create data loaders
        train_dataset = TensorDataset(
            torch.FloatTensor(x_train),
            torch.LongTensor(y_train),
        )
        train_loader = DataLoader(train_dataset, batch_size=self.batch_size, shuffle=True)

        val_loader = None
        if x_val is not None and y_val is not None:
            val_dataset = TensorDataset(
                torch.FloatTensor(x_val),
                torch.LongTensor(y_val),
            )
            val_loader = DataLoader(val_dataset, batch_size=self.batch_size, shuffle=False)

        # Training loop
        history: dict[str, list[float]] = {"train_loss": [], "val_loss": [], "val_acc": []}
        best_val_loss = float("inf")
        patience_counter = 0
        best_state = None

        for epoch in range(self.epochs):
            # Training phase
            self.model.train()
            train_losses = []

            for x_batch, y_batch in train_loader:
                x_batch = x_batch.to(self.device)
                y_batch = y_batch.to(self.device)

                optimizer.zero_grad()
                outputs = self.model(x_batch)
                loss = criterion(outputs, y_batch)
                loss.backward()

                # Gradient clipping
                torch.nn.utils.clip_grad_norm_(self.model.parameters(), max_norm=1.0)

                optimizer.step()
                train_losses.append(loss.item())

            avg_train_loss = float(np.mean(train_losses))
            history["train_loss"].append(avg_train_loss)

            # Validation phase
            if val_loader is not None:
                self.model.eval()
                val_losses = []
                correct = 0
                total = 0

                with torch.no_grad():
                    for x_batch, y_batch in val_loader:
                        x_batch = x_batch.to(self.device)
                        y_batch = y_batch.to(self.device)

                        outputs = self.model(x_batch)
                        loss = criterion(outputs, y_batch)
                        val_losses.append(loss.item())

                        _, predicted = torch.max(outputs.data, 1)
                        total += y_batch.size(0)
                        correct += (predicted == y_batch).sum().item()

                avg_val_loss = float(np.mean(val_losses))
                val_acc = correct / total
                history["val_loss"].append(avg_val_loss)
                history["val_acc"].append(val_acc)

                scheduler.step(avg_val_loss)

                # Early stopping
                if avg_val_loss < best_val_loss:
                    best_val_loss = avg_val_loss
                    patience_counter = 0
                    best_state = {k: v.cpu().clone() for k, v in self.model.state_dict().items()}
                else:
                    patience_counter += 1

                if patience_counter >= self.early_stopping_patience:
                    print(f"Early stopping at epoch {epoch + 1}")
                    break

                if (epoch + 1) % 10 == 0:
                    print(
                        f"Epoch {epoch + 1}/{self.epochs} - "
                        f"Train Loss: {avg_train_loss:.4f} - "
                        f"Val Loss: {avg_val_loss:.4f} - "
                        f"Val Acc: {val_acc:.4f}"
                    )

        # Restore best model
        if best_state is not None:
            self.model.load_state_dict(best_state)

        return history

    def save(self, model_path: str) -> None:
        """Save trained model."""
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

        # Save metadata
        metadata = {
            "input_size": self.input_size,
            "hidden_size": self.hidden_size,
            "num_layers": self.num_layers,
            "sequence_length": self.sequence_length,
            "trained_at": datetime.utcnow().isoformat(),
        }

        with open(path / "metadata.json", "w") as f:
            json.dump(metadata, f, indent=2)

    def evaluate(self, x_test: np.ndarray, y_test: np.ndarray) -> dict:
        """Evaluate model on test set."""
        if self.model is None:
            raise RuntimeError("No model to evaluate")

        self.model.eval()

        with torch.no_grad():
            x_tensor = torch.FloatTensor(x_test).to(self.device)
            y_tensor = torch.LongTensor(y_test).to(self.device)

            outputs = self.model(x_tensor)
            _, predicted = torch.max(outputs.data, 1)

            accuracy = (predicted == y_tensor).sum().item() / len(y_tensor)

            # Per-class accuracy
            class_names = ["DOWN", "SIDEWAYS", "UP"]
            class_acc = {}
            for i, name in enumerate(class_names):
                mask = y_tensor == i
                if mask.sum() > 0:
                    class_acc[name] = (
                        (predicted[mask] == y_tensor[mask]).sum().item() / mask.sum().item()
                    )
                else:
                    class_acc[name] = 0.0

        return {
            "accuracy": accuracy,
            "class_accuracy": class_acc,
        }
