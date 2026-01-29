"""Recommendation system task for mappy DataLoader evaluation.

Supports collaborative filtering with user-item interaction matrices.
"""

import pickle
import time
from typing import Any

import mappy_python
import numpy as np
import torch
from torch import nn, optim
from torch.utils.data import DataLoader

from evaluation.dataloader import MappyDataset, store_data_in_mappy
from evaluation.metrics import MetricsCollector


class RecommendationTask:
    """Recommendation system task (collaborative filtering)."""

    def __init__(
        self,
        num_users: int = 1000,
        num_items: int = 100,
        batch_size: int = 32,
        num_epochs: int = 1,
        use_mappy: bool = True,
        cache_data: bool = True,
    ):
        """Initialize recommendation task.

        Args:
            num_users: Number of users
            num_items: Number of items
            batch_size: Batch size
            num_epochs: Number of epochs
            use_mappy: Whether to use mappy
            cache_data: Whether to cache data

        """
        self.num_users = num_users
        self.num_items = num_items
        self.batch_size = batch_size
        self.num_epochs = num_epochs
        self.use_mappy = use_mappy
        self.cache_data = cache_data

        self.maplet = None
        if self.use_mappy:
            self.maplet = mappy_python.Maplet(
                capacity=100000,
                false_positive_rate=0.01,
                operator=mappy_python.VectorOperator(),
            )

    def generate_interaction_matrix(self, sparsity: float = 0.9) -> np.ndarray:
        """Generate synthetic user-item interaction matrix.

        Args:
            sparsity: Fraction of zero interactions

        Returns:
            Interaction matrix [num_users, num_items]

        """
        matrix = np.random.rand(self.num_users, self.num_items)
        # Apply sparsity
        mask = np.random.rand(self.num_users, self.num_items) > sparsity
        return matrix * mask

    def create_training_data(self, matrix: np.ndarray) -> list[tuple[int, int, float]]:
        """Create training data from interaction matrix.

        Args:
            matrix: Interaction matrix

        Returns:
            List of (user_id, item_id, rating) tuples

        """
        data = []
        for user_id in range(self.num_users):
            for item_id in range(self.num_items):
                rating = matrix[user_id, item_id]
                if rating > 0:  # Only include non-zero interactions
                    data.append((user_id, item_id, rating))
        return data

    def store_in_mappy(self, data: list[tuple[int, int, float]]) -> list[str]:
        """Store data in mappy.

        Args:
            data: List of (user_id, item_id, rating) tuples

        Returns:
            List of keys

        """
        data_dict = {}
        for idx, (user_id, item_id, rating) in enumerate(data):
            key = f"interaction_{idx}"
            serialized = pickle.dumps((user_id, item_id, rating))
            data_dict[key] = serialized

        return store_data_in_mappy(self.maplet, data_dict, compress=False)

    def create_dataloader(
        self, data: list[tuple[int, int, float]], keys: list[str] | None = None, shuffle: bool = True,
    ) -> DataLoader:
        """Create DataLoader.

        Args:
            data: Training data
            keys: Optional mappy keys
            shuffle: Whether to shuffle

        Returns:
            DataLoader

        """
        if self.use_mappy and keys is not None:
            dataset = MappyDataset(
                maplet=self.maplet,
                keys=keys,
                cache_data=self.cache_data,
                decompress_fn=None,  # Data is auto-deserialized in MappyDataset
            )
            return DataLoader(
                dataset,
                batch_size=self.batch_size,
                shuffle=shuffle,
                collate_fn=self._collate_fn,
                num_workers=0,
            )
        from torch.utils.data import Dataset

        class SimpleDataset(Dataset):
            def __init__(self, data):
                self.data = data

            def __len__(self):
                return len(self.data)

            def __getitem__(self, idx):
                return self.data[idx]

        dataset = SimpleDataset(data)
        return DataLoader(
            dataset,
            batch_size=self.batch_size,
            shuffle=shuffle,
            collate_fn=self._collate_fn,
            num_workers=0,
        )

    def _collate_fn(self, batch: list[tuple[int, int, float]]) -> tuple[torch.Tensor, torch.Tensor, torch.Tensor]:
        """Collate function for recommendation batches."""
        users, items, ratings = zip(*batch, strict=False)
        return (
            torch.tensor(users, dtype=torch.long),
            torch.tensor(items, dtype=torch.long),
            torch.tensor(ratings, dtype=torch.float32),
        )

    def create_model(self, embedding_dim: int = 64) -> nn.Module:
        """Create matrix factorization model.

        Args:
            embedding_dim: Embedding dimension

        Returns:
            PyTorch model

        """
        class MatrixFactorization(nn.Module):
            def __init__(self, num_users, num_items, embedding_dim):
                super().__init__()
                self.user_embedding = nn.Embedding(num_users, embedding_dim)
                self.item_embedding = nn.Embedding(num_items, embedding_dim)
                self.bias_user = nn.Embedding(num_users, 1)
                self.bias_item = nn.Embedding(num_items, 1)
                self.global_bias = nn.Parameter(torch.zeros(1))

            def forward(self, user_ids, item_ids):
                user_emb = self.user_embedding(user_ids)
                item_emb = self.item_embedding(item_ids)
                pred = (user_emb * item_emb).sum(dim=1)
                pred += self.bias_user(user_ids).squeeze()
                pred += self.bias_item(item_ids).squeeze()
                pred += self.global_bias
                return pred

        return MatrixFactorization(self.num_users, self.num_items, embedding_dim)

    def evaluate(
        self, model: nn.Module, dataloader: DataLoader, device: torch.device,
    ) -> dict[str, float]:
        """Evaluate model.

        Args:
            model: PyTorch model
            dataloader: DataLoader
            device: Device

        Returns:
            Metrics dictionary

        """
        model.eval()
        all_predictions = []
        all_targets = []

        with torch.no_grad():
            for users, items, ratings in dataloader:
                users = users.to(device)
                items = items.to(device)
                ratings = ratings.to(device)

                predictions = model(users, items)

                all_predictions.extend(predictions.cpu().numpy())
                all_targets.extend(ratings.cpu().numpy())

        predictions = np.array(all_predictions)
        targets = np.array(all_targets)

        # Calculate metrics
        rmse = np.sqrt(np.mean((predictions - targets) ** 2))
        mae = np.mean(np.abs(predictions - targets))

        # Precision@k and Recall@k (simplified)
        # Sort predictions and calculate top-k metrics
        k = min(10, len(predictions))
        top_k_indices = np.argsort(predictions)[-k:]
        precision_at_k = np.mean(targets[top_k_indices] > 0.5) if k > 0 else 0.0

        return {
            "rmse": float(rmse),
            "mae": float(mae),
            "precision_at_k": float(precision_at_k),
        }

    def run_evaluation(self, device: torch.device | None = None) -> dict[str, Any]:
        """Run complete evaluation.

        Args:
            device: Optional device

        Returns:
            Results dictionary

        """
        if device is None:
            device = torch.device("cuda" if torch.cuda.is_available() else "cpu")

        metrics_collector = MetricsCollector(track_gpu=torch.cuda.is_available())
        metrics_collector.start_monitoring()

        # Generate data
        matrix = self.generate_interaction_matrix()
        train_data = self.create_training_data(matrix)

        # Store in mappy
        train_keys = None
        if self.use_mappy:
            train_keys = self.store_in_mappy(train_data)

        # Split train/test
        split_idx = int(len(train_data) * 0.8)
        train_split = train_data[:split_idx]
        test_split = train_data[split_idx:]

        # Create dataloaders
        train_loader = self.create_dataloader(train_split, train_keys[:split_idx] if train_keys else None, shuffle=True)
        test_loader = self.create_dataloader(test_split, train_keys[split_idx:] if train_keys else None, shuffle=False)

        # Create model
        model = self.create_model().to(device)
        criterion = nn.MSELoss()
        optimizer = optim.Adam(model.parameters(), lr=0.01)

        # Training
        model.train()
        for _epoch in range(self.num_epochs):
            for batch_idx, (users, items, ratings) in enumerate(train_loader):
                batch_start = time.time()

                users = users.to(device)
                items = items.to(device)
                ratings = ratings.to(device)

                optimizer.zero_grad()
                predictions = model(users, items)
                loss = criterion(predictions, ratings)
                loss.backward()
                optimizer.step()

                batch_time = (time.time() - batch_start) * 1000
                metrics_collector.record_batch(batch_idx, len(users), batch_time)

        # Evaluation
        eval_metrics = self.evaluate(model, test_loader, device)
        metrics_collector.set_accuracy_metrics(
            task_name="recommendation",
            accuracy=1.0 - eval_metrics["rmse"],  # Use inverse RMSE as accuracy proxy
            **eval_metrics,
        )

        metrics_collector.stop_monitoring()

        config = {
            "num_users": self.num_users,
            "num_items": self.num_items,
            "batch_size": self.batch_size,
            "num_epochs": self.num_epochs,
            "use_mappy": self.use_mappy,
            "cache_data": self.cache_data,
        }

        results = metrics_collector.get_results("recommendation", config)

        return {
            "results": results,
            "eval_metrics": eval_metrics,
        }

