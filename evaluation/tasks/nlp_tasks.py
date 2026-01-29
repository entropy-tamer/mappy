"""NLP tasks for mappy DataLoader evaluation.

Supports text classification and language modeling tasks.
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
from evaluation.metrics import MetricsCollector, calculate_accuracy_metrics


class NLPTask:
    """NLP task implementation (text classification and language modeling)."""

    def __init__(
        self,
        task_type: str = "classification",
        batch_size: int = 32,
        num_epochs: int = 1,
        use_mappy: bool = True,
        cache_data: bool = True,
        vocab_size: int = 10000,
        max_length: int = 512,
    ):
        """Initialize NLP task.

        Args:
            task_type: Type of task (classification, language_modeling)
            batch_size: Batch size
            num_epochs: Number of epochs
            use_mappy: Whether to use mappy
            cache_data: Whether to cache data
            vocab_size: Vocabulary size
            max_length: Maximum sequence length

        """
        self.task_type = task_type
        self.batch_size = batch_size
        self.num_epochs = num_epochs
        self.use_mappy = use_mappy
        self.cache_data = cache_data
        self.vocab_size = vocab_size
        self.max_length = max_length

        self.maplet = None
        if self.use_mappy:
            self.maplet = mappy_python.Maplet(
                capacity=100000,
                false_positive_rate=0.01,
                operator=mappy_python.VectorOperator(),
            )

    def generate_synthetic_data(self, num_samples: int, num_classes: int = 2) -> list[dict[str, Any]]:
        """Generate synthetic text data.

        Args:
            num_samples: Number of samples
            num_classes: Number of classes (for classification)

        Returns:
            List of tokenized text dictionaries

        """
        data = []
        for i in range(num_samples):
            # Generate random token sequence
            seq_length = np.random.randint(10, self.max_length)
            input_ids = torch.randint(0, self.vocab_size, (seq_length,))
            attention_mask = torch.ones(seq_length)

            # Pad to max_length
            if seq_length < self.max_length:
                pad_length = self.max_length - seq_length
                input_ids = torch.cat([input_ids, torch.zeros(pad_length, dtype=torch.long)])
                attention_mask = torch.cat([attention_mask, torch.zeros(pad_length, dtype=torch.long)])

            item = {
                "input_ids": input_ids,
                "attention_mask": attention_mask,
            }

            if self.task_type == "classification":
                item["label"] = i % num_classes

            data.append(item)

        return data

    def store_in_mappy(self, data: list[dict[str, Any]]) -> list[str]:
        """Store data in mappy.

        Args:
            data: List of tokenized text dictionaries

        Returns:
            List of keys

        """
        data_dict = {}
        for idx, item in enumerate(data):
            key = f"text_{idx}"
            serialized = pickle.dumps(item)
            data_dict[key] = serialized

        return store_data_in_mappy(self.maplet, data_dict, compress=False)

    def create_dataloader(
        self, data: list[dict[str, Any]], keys: list[str] | None = None, shuffle: bool = True,
    ) -> DataLoader:
        """Create DataLoader.

        Args:
            data: Dataset data
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

    def _collate_fn(self, batch: list[dict[str, Any]]) -> dict[str, torch.Tensor]:
        """Collate function for NLP batches."""
        keys = batch[0].keys()
        batched = {}
        for key in keys:
            if key == "label":
                batched[key] = torch.tensor([item[key] for item in batch], dtype=torch.long)
            else:
                batched[key] = torch.stack([item[key] for item in batch])
        return batched

    def create_simple_model(self, num_classes: int = 2) -> nn.Module:
        """Create simple model for NLP task.

        Args:
            num_classes: Number of classes

        Returns:
            PyTorch model

        """
        if self.task_type == "classification":
            return nn.Sequential(
                nn.Embedding(self.vocab_size, 128),
                nn.LSTM(128, 64, batch_first=True),
                nn.AdaptiveAvgPool1d(1),
                nn.Flatten(),
                nn.Linear(64, num_classes),
            )
        # language_modeling
        return nn.Sequential(
            nn.Embedding(self.vocab_size, 256),
            nn.LSTM(256, 512, batch_first=True),
            nn.Linear(512, self.vocab_size),
        )

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
        total_loss = 0.0
        criterion = nn.CrossEntropyLoss()

        with torch.no_grad():
            for batch in dataloader:
                input_ids = batch["input_ids"].to(device)
                batch["attention_mask"].to(device)

                if self.task_type == "classification":
                    labels = batch["label"].to(device)
                    outputs = model(input_ids)
                    loss = criterion(outputs, labels)
                    total_loss += loss.item()

                    _, predicted = torch.max(outputs.data, 1)
                    all_predictions.extend(predicted.cpu().numpy())
                    all_targets.extend(labels.cpu().numpy())
                else:  # language_modeling
                    # Shift for language modeling
                    inputs = input_ids[:, :-1]
                    targets = input_ids[:, 1:]
                    outputs = model(inputs)
                    loss = criterion(outputs.view(-1, self.vocab_size), targets.view(-1))
                    total_loss += loss.item()

        metrics = {}
        if self.task_type == "classification":
            metrics = calculate_accuracy_metrics(
                np.array(all_predictions),
                np.array(all_targets),
                task_type="classification",
            )
        else:
            # Perplexity for language modeling
            metrics["perplexity"] = np.exp(total_loss / len(dataloader))

        metrics["loss"] = total_loss / len(dataloader)
        return metrics

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
        num_classes = 2 if self.task_type == "classification" else None
        train_data = self.generate_synthetic_data(1000, num_classes or 2)
        test_data = self.generate_synthetic_data(200, num_classes or 2)

        # Store in mappy
        train_keys = None
        test_keys = None
        if self.use_mappy:
            train_keys = self.store_in_mappy(train_data)
            test_keys = self.store_in_mappy(test_data)

        # Create dataloaders
        train_loader = self.create_dataloader(train_data, train_keys, shuffle=True)
        test_loader = self.create_dataloader(test_data, test_keys, shuffle=False)

        # Create model
        num_classes = 2 if self.task_type == "classification" else self.vocab_size
        model = self.create_simple_model(num_classes).to(device)
        criterion = nn.CrossEntropyLoss()
        optimizer = optim.Adam(model.parameters(), lr=0.001)

        # Training
        model.train()
        for _epoch in range(self.num_epochs):
            for batch_idx, batch in enumerate(train_loader):
                batch_start = time.time()

                input_ids = batch["input_ids"].to(device)
                batch["attention_mask"].to(device)

                optimizer.zero_grad()

                if self.task_type == "classification":
                    labels = batch["label"].to(device)
                    outputs = model(input_ids)
                    loss = criterion(outputs, labels)
                else:
                    inputs = input_ids[:, :-1]
                    targets = input_ids[:, 1:]
                    outputs = model(inputs)
                    loss = criterion(outputs.view(-1, self.vocab_size), targets.view(-1))

                loss.backward()
                optimizer.step()

                batch_time = (time.time() - batch_start) * 1000
                metrics_collector.record_batch(batch_idx, len(input_ids), batch_time)

        # Evaluation
        eval_metrics = self.evaluate(model, test_loader, device)
        metrics_collector.set_accuracy_metrics(
            task_name=f"nlp_{self.task_type}",
            **eval_metrics,
        )

        metrics_collector.stop_monitoring()

        config = {
            "task_type": self.task_type,
            "batch_size": self.batch_size,
            "num_epochs": self.num_epochs,
            "use_mappy": self.use_mappy,
            "cache_data": self.cache_data,
        }

        results = metrics_collector.get_results(f"nlp_{self.task_type}", config)

        return {
            "results": results,
            "eval_metrics": eval_metrics,
        }

