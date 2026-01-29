"""Image classification task for mappy DataLoader evaluation.

Supports MNIST, CIFAR-10 and ImageNet-style classification tasks.
"""

import logging
import pickle
import time
from typing import Any

import mappy_python
import numpy as np
import torch
from torch import nn, optim
from torch.utils.data import DataLoader
from torchvision import datasets, transforms

from evaluation.dataloader import (
    ContiguousMappyDataset,
    MappyDataset,
    create_contiguous_collate_fn,
    store_data_in_mappy,
)
from evaluation.metrics import MetricsCollector, calculate_accuracy_metrics
from evaluation.utils.data_utils import collate_fn_images

logger = logging.getLogger(__name__)


class ImageClassificationTask:
    """Image classification task implementation."""

    def __init__(
        self,
        dataset_name: str = "cifar10",
        batch_size: int = 32,
        num_epochs: int = 1,
        use_mappy: bool = True,
        cache_data: bool = True,
    ):
        """Initialize image classification task.

        Args:
            dataset_name: Name of dataset (mnist, cifar10, imagenet, etc.)
            batch_size: Batch size for training
            num_epochs: Number of training epochs
            use_mappy: Whether to use mappy DataLoader
            cache_data: Whether to cache data (recommended)

        """
        self.dataset_name = dataset_name
        self.batch_size = batch_size
        self.num_epochs = num_epochs
        self.use_mappy = use_mappy
        self.cache_data = cache_data

        logger.info(f"Initializing ImageClassificationTask: dataset={dataset_name}, "
                   f"batch_size={batch_size}, epochs={num_epochs}, "
                   f"use_mappy={use_mappy}, cache_data={cache_data}")

        # Initialize maplet if using mappy
        self.maplet = None
        if self.use_mappy:
            # Use larger capacity to handle full datasets
            # MNIST: 60k train + 10k test = 70k items
            # CIFAR-10: 50k train + 10k test = 60k items
            # Use 200k capacity to provide plenty of headroom
            estimated_size = 200000
            logger.debug(f"Creating mappy Maplet with capacity={estimated_size}, fpr=0.01")
            self.maplet = mappy_python.Maplet(
                capacity=estimated_size,
                false_positive_rate=0.01,
                operator=mappy_python.VectorOperator(),
            )
            logger.debug("Maplet created successfully")

    def load_dataset(self) -> tuple[list[tuple[Any, int]], list[tuple[Any, int]]]:
        """Load dataset (MNIST, CIFAR-10 or synthetic ImageNet-style).

        Returns:
            (train_data, test_data) tuples of (image, label)

        """
        if self.dataset_name == "mnist":
            # Load MNIST
            logger.info("Loading MNIST dataset...")
            logger.debug("Creating MNIST transforms: ToTensor + Normalize(mean=0.1307, std=0.3081)")
            transform = transforms.Compose([
                transforms.ToTensor(),
                transforms.Normalize((0.1307,), (0.3081,)),
            ])

            logger.debug("Loading MNIST training set (download=True)...")
            train_dataset = datasets.MNIST(
                root="./data", train=True, download=True, transform=transform,
            )
            logger.info(f"MNIST training set loaded: {len(train_dataset)} samples")
            
            logger.debug("Loading MNIST test set (download=True)...")
            test_dataset = datasets.MNIST(
                root="./data", train=False, download=True, transform=transform,
            )
            logger.info(f"MNIST test set loaded: {len(test_dataset)} samples")

            logger.debug("Converting datasets to list of (image, label) tuples...")
            train_data = [(img, label) for img, label in train_dataset]
            test_data = [(img, label) for img, label in test_dataset]
            logger.debug(f"Train data: {len(train_data)} samples, "
                        f"Test data: {len(test_data)} samples")
            if train_data:
                logger.debug(f"Sample image shape: {train_data[0][0].shape}, "
                           f"Sample label: {train_data[0][1]}")

        elif self.dataset_name == "cifar10":
            # Load CIFAR-10
            logger.info("Loading CIFAR-10 dataset...")
            logger.debug("Creating CIFAR-10 transforms: ToTensor + Normalize")
            transform = transforms.Compose([
                transforms.ToTensor(),
                transforms.Normalize((0.5, 0.5, 0.5), (0.5, 0.5, 0.5)),
            ])

            logger.debug("Loading CIFAR-10 training set (download=True)...")
            train_dataset = datasets.CIFAR10(
                root="./data", train=True, download=True, transform=transform,
            )
            logger.info(f"CIFAR-10 training set loaded: {len(train_dataset)} samples")
            
            logger.debug("Loading CIFAR-10 test set (download=True)...")
            test_dataset = datasets.CIFAR10(
                root="./data", train=False, download=True, transform=transform,
            )
            logger.info(f"CIFAR-10 test set loaded: {len(test_dataset)} samples")

            logger.debug("Converting datasets to list of (image, label) tuples...")
            train_data = [(img, label) for img, label in train_dataset]
            test_data = [(img, label) for img, label in test_dataset]
            logger.debug(f"Train data: {len(train_data)} samples, "
                        f"Test data: {len(test_data)} samples")

        else:
            # Synthetic ImageNet-style data
            train_data = self._generate_synthetic_data(num_samples=1000, num_classes=10)
            test_data = self._generate_synthetic_data(num_samples=200, num_classes=10)

        return train_data, test_data

    def _generate_synthetic_data(
        self, num_samples: int, num_classes: int,
    ) -> list[tuple[torch.Tensor, int]]:
        """Generate synthetic image data for testing.

        Args:
            num_samples: Number of samples to generate
            num_classes: Number of classes

        Returns:
            List of (image_tensor, label) tuples

        """
        data = []
        for i in range(num_samples):
            # Generate random image (3, 32, 32) for CIFAR-10 style
            image = torch.randn(3, 32, 32)
            label = i % num_classes
            data.append((image, label))
        return data

    def store_in_mappy(self, data: list[tuple[Any, int]], prefix: str = "img_") -> list[str]:
        """Store dataset in mappy maplet.

        Args:
            data: List of (image, label) tuples
            prefix: Prefix for keys to avoid collisions (default: "img_")

        Returns:
            List of keys stored

        """
        logger.info(f"Storing {len(data)} samples in mappy maplet with prefix '{prefix}'...")
        
        # Try to use ByteArrayOperator for better performance
        try:
            import mappy_python
            # Check if maplet uses ByteArrayOperator
            # For now, we'll use the existing VectorOperator approach but prepare for ByteArrayOperator
            use_bytearray = False  # Will be enabled when ByteArrayOperator is fully integrated
        except ImportError:
            use_bytearray = False
        
        # Create list of (key, value) tuples directly - no intermediate dict
        # This is more memory efficient for large datasets
        data_items = []
        for idx, (image, label) in enumerate(data):
            key = f"{prefix}{idx}"
            if use_bytearray:
                # Store image as raw bytes (ByteArrayOperator)
                # Convert image to bytes (PNG format for now)
                from PIL import Image
                import io
                if isinstance(image, torch.Tensor):
                    # Convert tensor to PIL Image
                    img_np = image.permute(1, 2, 0).numpy()
                    if img_np.shape[2] == 1:
                        img_np = img_np.squeeze(2)
                    img_pil = Image.fromarray((img_np * 255).astype(np.uint8))
                else:
                    img_pil = image
                
                # Save as PNG bytes
                buf = io.BytesIO()
                img_pil.save(buf, format='PNG')
                image_bytes = buf.getvalue()
                data_items.append((key, image_bytes))
            else:
                # Legacy: Serialize image and label tuple (for compatibility)
                serialized = pickle.dumps((image, label))
                data_items.append((key, serialized))
            
            if (idx + 1) % 10000 == 0:
                logger.debug(f"Prepared {idx + 1}/{len(data)} samples for storage...")

        logger.debug(f"Prepared {len(data_items)} samples, storing in maplet...")
        keys = store_data_in_mappy(self.maplet, data_items, compress=False)
        logger.info(f"Stored {len(keys)} samples in mappy maplet")
        return keys

    def create_dataloader(
        self, data: list[tuple[Any, int]], keys: list[str] | None = None, shuffle: bool = True,
    ) -> DataLoader:
        """Create DataLoader (mappy or standard).

        Args:
            data: Dataset data
            keys: Optional keys for mappy (if using mappy)
            shuffle: Whether to shuffle

        Returns:
            DataLoader instance

        """
        if self.use_mappy and keys is not None:
            # Create contiguous mappy DataLoader for efficient data flow
            # Data flow: mappy -> contiguous memory -> tensor views -> batches
            logger.info(f"Creating contiguous mappy DataLoader: batch_size={self.batch_size}, "
                       f"shuffle={shuffle}")
            logger.debug(f"Using {len(keys)} keys from maplet")
            
            # Determine image shape based on dataset
            if self.dataset_name == "mnist":
                image_shape = (1, 28, 28)  # MNIST: 1 channel, 28x28
            elif self.dataset_name == "cifar10":
                image_shape = (3, 32, 32)  # CIFAR-10: 3 channels, 32x32
            else:
                # Default: try to infer from first sample
                if data:
                    sample_img, _ = data[0]
                    if isinstance(sample_img, torch.Tensor):
                        image_shape = tuple(sample_img.shape)
                    else:
                        image_shape = (3, 32, 32)  # Fallback
                else:
                    image_shape = (3, 32, 32)  # Fallback
            
            logger.debug(f"Image shape: {image_shape}")
            
            # Use new Rust-native MappyMLDataset
            try:
                import python_mappy_ml
                dataset = python_mappy_ml.MappyMLDataset(
                    maplet=self.maplet,
                    keys=keys,
                    image_shape=list(image_shape),  # Convert tuple to list for Rust
                    label_dtype="int64",
                    backend="auto",  # Auto-select DLPack or libtorch
                )
                logger.debug(f"MappyMLDataset created with {len(dataset)} samples")
                logger.info("Using Rust-native data pipeline: mappy -> Rust decode -> zero-copy tensor")
                
                # Standard collate function (MappyMLDataset returns proper tuples)
                collate_fn = None  # Use default collate
            except ImportError:
                # Fallback to ContiguousMappyDataset if python-mappy-ml not available
                logger.warning("python-mappy-ml not available, falling back to ContiguousMappyDataset")
                from evaluation.dataloader import ContiguousMappyDataset, create_contiguous_collate_fn
                dataset = ContiguousMappyDataset(
                    maplet=self.maplet,
                    keys=keys,
                    image_shape=image_shape,
                    label_dtype=torch.long,
                    transform=None,
                )
                logger.debug(f"ContiguousMappyDataset created with {len(dataset)} samples")
                collate_fn = create_contiguous_collate_fn(image_shape)
            
            # Professional ML DataLoader configuration
            # Based on research: num_workers, pin_memory, prefetch_factor for optimal performance
            use_gpu = torch.cuda.is_available()
            num_workers = 0  # For contiguous data, workers add overhead
            # But we can enable if needed: num_workers = min(4, os.cpu_count() or 1)
            
            dataloader = DataLoader(
                dataset,
                batch_size=self.batch_size,
                shuffle=shuffle,
                collate_fn=collate_fn,
                num_workers=num_workers,
                pin_memory=use_gpu,  # Pin memory for faster GPU transfer (professional best practice)
                prefetch_factor=2 if num_workers > 0 else None,  # Prefetch batches (async loading)
                persistent_workers=False,  # Don't keep workers alive between epochs
                drop_last=False,  # Keep all data
            )
            logger.debug(f"DataLoader created: {len(dataloader)} batches")
            logger.info("Using efficient data flow: mappy -> contiguous memory -> tensor views -> GPU batches")
            return dataloader
        # Standard DataLoader
        logger.info(f"Creating standard DataLoader: batch_size={self.batch_size}, "
                   f"shuffle={shuffle}")
        from torch.utils.data import TensorDataset

        logger.debug("Stacking images and converting labels to tensor...")
        images, labels = zip(*data, strict=False)
        images = torch.stack(images)
        labels = torch.tensor(labels, dtype=torch.long)
        logger.debug(f"Images shape: {images.shape}, Labels shape: {labels.shape}")

        dataset = TensorDataset(images, labels)
        dataloader = DataLoader(
            dataset,
            batch_size=self.batch_size,
            shuffle=shuffle,
            num_workers=0,
        )
        logger.debug(f"DataLoader created: {len(dataloader)} batches")
        return dataloader

    def create_simple_model(self, num_classes: int = 10, in_channels: int = 3) -> nn.Module:
        """Create a simple CNN model for classification.

        Args:
            num_classes: Number of classes
            in_channels: Number of input channels (1 for MNIST, 3 for CIFAR-10)

        Returns:
            PyTorch model

        """
        return nn.Sequential(
            nn.Conv2d(in_channels, 32, 3, padding=1),
            nn.ReLU(),
            nn.MaxPool2d(2),
            nn.Conv2d(32, 64, 3, padding=1),
            nn.ReLU(),
            nn.MaxPool2d(2),
            nn.AdaptiveAvgPool2d(1),
            nn.Flatten(),
            nn.Linear(64, num_classes),
        )

    def evaluate(
        self, model: nn.Module, dataloader: DataLoader, device: torch.device,
    ) -> dict[str, float]:
        """Evaluate model on dataset.

        Args:
            model: PyTorch model
            dataloader: DataLoader for evaluation
            device: Device to run on

        Returns:
            Dictionary of metrics

        """
        logger.debug(f"Evaluating model on {len(dataloader)} batches...")
        model.eval()
        all_predictions = []
        all_targets = []
        batch_count = 0

        with torch.no_grad():
            for batch_idx, (images, labels) in enumerate(dataloader):
                images = images.to(device)
                labels = labels.to(device)

                outputs = model(images)
                _, predicted = torch.max(outputs.data, 1)

                all_predictions.extend(predicted.cpu().numpy())
                all_targets.extend(labels.cpu().numpy())
                batch_count += 1
                
                if (batch_idx + 1) % 50 == 0:
                    logger.debug(f"  Evaluated {batch_idx + 1}/{len(dataloader)} batches...")

        logger.debug(f"Evaluation complete: {len(all_predictions)} predictions")
        logger.debug("Calculating accuracy metrics...")
        metrics = calculate_accuracy_metrics(
            np.array(all_predictions),
            np.array(all_targets),
            task_type="classification",
        )
        logger.debug(f"Metrics calculated: {metrics}")
        return metrics


    def run_evaluation(self, device: torch.device | None = None) -> dict[str, Any]:
        """Run complete evaluation.

        Args:
            device: Optional device (defaults to cuda if available)

        Returns:
            Dictionary with evaluation results

        """
        if device is None:
            device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
        
        logger.info(f"Starting evaluation on device: {device}")

        # Initialize metrics collector
        logger.debug("Initializing metrics collector...")
        metrics_collector = MetricsCollector(track_gpu=torch.cuda.is_available())
        metrics_collector.start_monitoring()
        logger.debug("Metrics monitoring started")

        # Load dataset
        logger.info("=" * 70)
        logger.info("Step 1: Loading dataset")
        logger.info("=" * 70)
        train_data, test_data = self.load_dataset()
        logger.info(f"Dataset loaded: {len(train_data)} train, {len(test_data)} test samples")

        # Store in mappy if using mappy
        train_keys = None
        test_keys = None
        if self.use_mappy:
            logger.info("=" * 70)
            logger.info("Step 2: Storing data in mappy")
            logger.info("=" * 70)
            # Store train data with "train_" prefix
            train_keys = self.store_in_mappy(train_data, prefix="train_")
            # Store test data with "test_" prefix to avoid key collisions
            test_keys = self.store_in_mappy(test_data, prefix="test_")
            logger.info(f"Data stored: {len(train_keys)} train keys, {len(test_keys)} test keys")
        else:
            logger.info("Skipping mappy storage (using standard DataLoader)")

        # Create dataloaders
        logger.info("=" * 70)
        logger.info("Step 3: Creating DataLoaders")
        logger.info("=" * 70)
        train_loader = self.create_dataloader(train_data, train_keys, shuffle=True)
        test_loader = self.create_dataloader(test_data, test_keys, shuffle=False)
        logger.info("DataLoaders created")

        # Create model
        logger.info("=" * 70)
        logger.info("Step 4: Creating model")
        logger.info("=" * 70)
        num_classes = 10
        # Determine input channels based on dataset
        in_channels = 1 if self.dataset_name == "mnist" else 3
        logger.info(f"Creating model: num_classes={num_classes}, in_channels={in_channels}")
        logger.debug(f"Model will be moved to device: {device}")
        model = self.create_simple_model(num_classes, in_channels=in_channels).to(device)
        logger.debug("Model created and moved to device")
        
        criterion = nn.CrossEntropyLoss()
        optimizer = optim.Adam(model.parameters(), lr=0.001)
        num_params = sum(p.numel() for p in model.parameters())
        logger.info(f"Optimizer: Adam with lr=0.001, {num_params:,} parameters")

        # Training loop with metrics
        logger.info("=" * 70)
        logger.info("Step 5: Training")
        logger.info("=" * 70)
        logger.info(f"Starting training: {self.num_epochs} epochs, "
                   f"{len(train_loader)} batches per epoch")
        model.train()
        batch_times = []

        for epoch in range(self.num_epochs):
            logger.info(f"Epoch {epoch + 1}/{self.num_epochs}")
            epoch_start = time.time()
            epoch_loss = 0.0
            num_batches = 0
            
            for batch_idx, (images, labels) in enumerate(train_loader):
                batch_start = time.time()

                images = images.to(device)
                labels = labels.to(device)

                optimizer.zero_grad()
                outputs = model(images)
                loss = criterion(outputs, labels)
                loss.backward()
                optimizer.step()

                batch_time = (time.time() - batch_start) * 1000  # ms
                batch_times.append(batch_time)
                metrics_collector.record_batch(batch_idx, len(labels), batch_time)
                
                epoch_loss += loss.item()
                num_batches += 1
                
                if (batch_idx + 1) % 100 == 0:
                    logger.debug(f"  Batch {batch_idx + 1}/{len(train_loader)}: "
                               f"loss={loss.item():.4f}, time={batch_time:.2f}ms")

            epoch_time = time.time() - epoch_start
            avg_loss = epoch_loss / num_batches if num_batches > 0 else 0.0
            logger.info(f"Epoch {epoch + 1} complete: avg_loss={avg_loss:.4f}, "
                       f"time={epoch_time:.2f}s")

        # Evaluation
        logger.info("=" * 70)
        logger.info("Step 6: Evaluation")
        logger.info("=" * 70)
        logger.info("Starting evaluation on test set...")
        logger.debug(f"Test set: {len(test_loader)} batches")
        eval_metrics = self.evaluate(model, test_loader, device)
        logger.info(f"Evaluation complete: accuracy={eval_metrics.get('accuracy', 0):.4f}")
        logger.debug(f"All eval metrics: {eval_metrics}")
        
        logger.debug("Setting accuracy metrics in collector...")
        metrics_collector.set_accuracy_metrics(
            task_name=f"image_classification_{self.dataset_name}",
            **eval_metrics,
        )

        logger.debug("Stopping metrics monitoring...")
        metrics_collector.stop_monitoring()
        logger.debug("Metrics monitoring stopped")

        # Get results
        logger.info("=" * 70)
        logger.info("Step 7: Collecting results")
        logger.info("=" * 70)
        config = {
            "dataset": self.dataset_name,
            "batch_size": self.batch_size,
            "num_epochs": self.num_epochs,
            "use_mappy": self.use_mappy,
            "cache_data": self.cache_data,
        }
        logger.debug(f"Config: {config}")

        results = metrics_collector.get_results(f"image_classification_{self.dataset_name}", config)
        logger.debug("Results collected")

        return {
            "results": results,
            "eval_metrics": eval_metrics,
            "batch_times": batch_times,
        }

