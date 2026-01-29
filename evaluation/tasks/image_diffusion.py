"""Image diffusion task for mappy DataLoader evaluation.

Supports Stable Diffusion-style training with image-text pairs.
References stilts compression patterns for image metadata.
"""

import pickle
import time
from typing import Any

import mappy_python
import torch
from torch import nn, optim
from torch.utils.data import DataLoader

from evaluation.dataloader import MappyDataset, store_data_in_mappy
from evaluation.metrics import MetricsCollector


class ImageDiffusionTask:
    """Image diffusion task (Stable Diffusion-style)."""

    def __init__(
        self,
        image_size: tuple[int, int] = (64, 64),
        batch_size: int = 8,
        num_epochs: int = 1,
        use_mappy: bool = True,
        cache_data: bool = True,
        num_timesteps: int = 1000,
    ):
        """Initialize image diffusion task.

        Args:
            image_size: Image size (height, width)
            batch_size: Batch size
            num_epochs: Number of epochs
            use_mappy: Whether to use mappy
            cache_data: Whether to cache data
            num_timesteps: Number of diffusion timesteps

        """
        self.image_size = image_size
        self.batch_size = batch_size
        self.num_epochs = num_epochs
        self.use_mappy = use_mappy
        self.cache_data = cache_data
        self.num_timesteps = num_timesteps

        self.maplet = None
        if self.use_mappy:
            self.maplet = mappy_python.Maplet(
                capacity=100000,
                false_positive_rate=0.01,
                operator=mappy_python.VectorOperator(),
            )

    def generate_image_text_pairs(self, num_samples: int) -> list[tuple[torch.Tensor, str]]:
        """Generate synthetic image-text pairs.

        Args:
            num_samples: Number of samples

        Returns:
            List of (image_tensor, text_caption) tuples

        """
        data = []
        captions = [
            "a photo of a cat",
            "a photo of a dog",
            "a photo of a bird",
            "a photo of a car",
            "a photo of a house",
        ]

        for i in range(num_samples):
            # Generate random image
            image = torch.randn(3, self.image_size[0], self.image_size[1])
            caption = captions[i % len(captions)]
            data.append((image, caption))

        return data

    def store_in_mappy(self, data: list[tuple[torch.Tensor, str]]) -> list[str]:
        """Store image-text pairs in mappy.

        Args:
            data: List of (image, text) tuples

        Returns:
            List of keys

        """
        data_dict = {}
        for idx, (image, text) in enumerate(data):
            key = f"diffusion_{idx}"
            # Store as tuple (image, text)
            serialized = pickle.dumps((image, text))
            data_dict[key] = serialized

        return store_data_in_mappy(self.maplet, data_dict, compress=False)

    def create_dataloader(
        self, data: list[tuple[torch.Tensor, str]], keys: list[str] | None = None, shuffle: bool = True,
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

    def _collate_fn(self, batch: list[tuple[torch.Tensor, str]]) -> tuple[torch.Tensor, list[str]]:
        """Collate function for diffusion batches."""
        images, texts = zip(*batch, strict=False)
        return torch.stack(images), list(texts)

    def create_simple_unet(self, in_channels: int = 3, out_channels: int = 3) -> nn.Module:
        """Create simple U-Net for diffusion.

        Args:
            in_channels: Input channels
            out_channels: Output channels

        Returns:
            PyTorch model

        """
        # Simplified U-Net architecture
        return nn.Sequential(
            nn.Conv2d(in_channels + 1, 64, 3, padding=1),  # +1 for timestep embedding
            nn.ReLU(),
            nn.Conv2d(64, 64, 3, padding=1),
            nn.ReLU(),
            nn.Conv2d(64, 64, 3, padding=1),
            nn.ReLU(),
            nn.Conv2d(64, out_channels, 3, padding=1),
        )

    def add_timestep_embedding(self, x: torch.Tensor, t: torch.Tensor) -> torch.Tensor:
        """Add timestep embedding to input.

        Args:
            x: Input tensor [B, C, H, W]
            t: Timestep tensor [B]

        Returns:
            Tensor with timestep embedding

        """
        # Simple timestep embedding (expand to match spatial dimensions)
        t_emb = t.view(-1, 1, 1, 1).expand(-1, 1, x.shape[2], x.shape[3])
        return torch.cat([x, t_emb], dim=1)

    def sample_timesteps(self, batch_size: int) -> torch.Tensor:
        """Sample random timesteps for training.

        Args:
            batch_size: Batch size

        Returns:
            Timestep tensor

        """
        return torch.randint(0, self.num_timesteps, (batch_size,))

    def add_noise(self, images: torch.Tensor, timesteps: torch.Tensor) -> tuple[torch.Tensor, torch.Tensor]:
        """Add noise to images for diffusion training.

        Args:
            images: Clean images [B, C, H, W]
            timesteps: Timesteps [B]

        Returns:
            (noisy_images, noise) tuple

        """
        noise = torch.randn_like(images)
        # Simple linear noise schedule
        alpha = 1.0 - (timesteps.float() / self.num_timesteps)
        alpha = alpha.view(-1, 1, 1, 1)
        noisy_images = alpha * images + (1 - alpha) * noise
        return noisy_images, noise

    def evaluate(
        self, model: nn.Module, dataloader: DataLoader, device: torch.device,
    ) -> dict[str, float]:
        """Evaluate diffusion model (simplified).

        Args:
            model: PyTorch model
            dataloader: DataLoader
            device: Device

        Returns:
            Metrics dictionary

        """
        model.eval()
        total_loss = 0.0

        with torch.no_grad():
            for images, _ in dataloader:
                images = images.to(device)
                batch_size = images.shape[0]

                # Sample timesteps
                timesteps = self.sample_timesteps(batch_size).to(device)

                # Add noise
                noisy_images, noise = self.add_noise(images, timesteps)

                # Predict noise
                timestep_emb = self.add_timestep_embedding(noisy_images, timesteps)
                predicted_noise = model(timestep_emb)

                # Calculate loss
                loss = nn.functional.mse_loss(predicted_noise, noise)
                total_loss += loss.item()

        # Simplified FID score (using MSE as proxy)
        fid_proxy = total_loss / len(dataloader)

        return {
            "loss": total_loss / len(dataloader),
            "fid_score": float(fid_proxy),  # Proxy metric
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
        train_data = self.generate_image_text_pairs(100)
        test_data = self.generate_image_text_pairs(20)

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
        model = self.create_simple_unet().to(device)
        criterion = nn.MSELoss()
        optimizer = optim.Adam(model.parameters(), lr=0.001)

        # Training
        model.train()
        for _epoch in range(self.num_epochs):
            for batch_idx, (images, _) in enumerate(train_loader):
                batch_start = time.time()

                images = images.to(device)
                batch_size = images.shape[0]

                # Sample timesteps
                timesteps = self.sample_timesteps(batch_size).to(device)

                # Add noise
                noisy_images, noise = self.add_noise(images, timesteps)

                # Predict noise
                optimizer.zero_grad()
                timestep_emb = self.add_timestep_embedding(noisy_images, timesteps)
                predicted_noise = model(timestep_emb)
                loss = criterion(predicted_noise, noise)
                loss.backward()
                optimizer.step()

                batch_time = (time.time() - batch_start) * 1000
                metrics_collector.record_batch(batch_idx, batch_size, batch_time)

        # Evaluation
        eval_metrics = self.evaluate(model, test_loader, device)
        metrics_collector.set_accuracy_metrics(
            task_name="image_diffusion",
            accuracy=1.0 - eval_metrics["loss"],  # Use inverse loss as accuracy proxy
            **eval_metrics,
        )

        metrics_collector.stop_monitoring()

        config = {
            "image_size": self.image_size,
            "batch_size": self.batch_size,
            "num_epochs": self.num_epochs,
            "use_mappy": self.use_mappy,
            "cache_data": self.cache_data,
            "num_timesteps": self.num_timesteps,
        }

        results = metrics_collector.get_results("image_diffusion", config)

        return {
            "results": results,
            "eval_metrics": eval_metrics,
        }

