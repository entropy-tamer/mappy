"""Data preprocessing utilities for evaluation suite."""

from typing import Any

import numpy as np
import torch
from PIL import Image
from torchvision import transforms


def preprocess_image(
    image: Image.Image | np.ndarray | torch.Tensor,
    size: tuple[int, int] = (224, 224),
    normalize: bool = True,
) -> torch.Tensor:
    """Preprocess image for model input.

    Args:
        image: Input image (PIL, numpy, or tensor)
        size: Target size (height, width)
        normalize: Whether to normalize to ImageNet stats

    Returns:
        Preprocessed tensor [C, H, W]

    """
    # Try to use Rust-native image decoder if available
    try:
        import python_mappy_ml
        use_rust = True
    except ImportError:
        use_rust = False
    
    if use_rust and isinstance(image, (Image.Image, np.ndarray)):
        # Use Rust image decoder
        try:
            if isinstance(image, Image.Image):
                # Convert PIL to bytes
                import io
                buf = io.BytesIO()
                image.save(buf, format='PNG')
                image_bytes = buf.getvalue()
            elif isinstance(image, np.ndarray):
                # Convert numpy to bytes (PNG)
                from PIL import Image as PILImage
                import io
                if image.dtype != np.uint8:
                    image = (image * 255).astype(np.uint8)
                img_pil = PILImage.fromarray(image)
                buf = io.BytesIO()
                img_pil.save(buf, format='PNG')
                image_bytes = buf.getvalue()
            else:
                image_bytes = None
            
            if image_bytes:
                # Decode and preprocess using Rust
                decoded = python_mappy_ml.decode_image(image_bytes)
                preprocessed = python_mappy_ml.preprocess_image(decoded, size, normalize)
                # Convert to torch tensor
                import torch
                return torch.from_numpy(preprocessed)
        except Exception as e:
            # Fallback to PIL if Rust decoder fails
            import logging
            logger = logging.getLogger(__name__)
            logger.debug(f"Rust image decoder failed, falling back to PIL: {e}")
            use_rust = False
    
    # Fallback to PIL-based processing
    # Convert to PIL if needed
    if isinstance(image, np.ndarray):
        if image.dtype != np.uint8:
            image = (image * 255).astype(np.uint8)
        image = Image.fromarray(image)
    elif isinstance(image, torch.Tensor):
        if image.dim() == 3:
            image = image.permute(1, 2, 0).numpy()
            image = (image * 255).astype(np.uint8)
            image = Image.fromarray(image)
        else:
            return image  # Already processed

    # Apply transforms
    transform_list = [
        transforms.Resize(size),
        transforms.ToTensor(),
    ]

    if normalize:
        transform_list.append(
            transforms.Normalize(
                mean=[0.485, 0.456, 0.406],
                std=[0.229, 0.224, 0.225],
            ),
        )

    transform = transforms.Compose(transform_list)
    return transform(image)


def normalize_image(image: torch.Tensor) -> torch.Tensor:
    """Normalize image tensor to ImageNet stats.

    Args:
        image: Image tensor [C, H, W] or [B, C, H, W]

    Returns:
        Normalized tensor

    """
    mean = torch.tensor([0.485, 0.456, 0.406]).view(-1, 1, 1)
    std = torch.tensor([0.229, 0.224, 0.225]).view(-1, 1, 1)

    if image.dim() == 4:
        mean = mean.unsqueeze(0)
        std = std.unsqueeze(0)

    return (image - mean) / std


def preprocess_text(
    text: str,
    tokenizer: Any = None,
    max_length: int = 512,
    padding: bool = True,
    truncation: bool = True,
) -> dict[str, torch.Tensor]:
    """Preprocess text for model input.

    Args:
        text: Input text string
        tokenizer: Optional tokenizer (from transformers)
        max_length: Maximum sequence length
        padding: Whether to pad sequences
        truncation: Whether to truncate long sequences

    Returns:
        Dictionary with input_ids, attention_mask, etc.

    """
    if tokenizer is None:
        # Simple tokenization fallback
        tokens = text.split()[:max_length]
        input_ids = torch.tensor([hash(t) % 10000 for t in tokens])
        attention_mask = torch.ones_like(input_ids)

        if padding and len(input_ids) < max_length:
            pad_length = max_length - len(input_ids)
            input_ids = torch.cat([input_ids, torch.zeros(pad_length, dtype=torch.long)])
            attention_mask = torch.cat([attention_mask, torch.zeros(pad_length, dtype=torch.long)])

        return {
            "input_ids": input_ids,
            "attention_mask": attention_mask,
        }

    # Use tokenizer
    encoded = tokenizer(
        text,
        max_length=max_length,
        padding="max_length" if padding else False,
        truncation=truncation,
        return_tensors="pt",
    )

    return {k: v.squeeze(0) for k, v in encoded.items()}


def collate_fn_images(batch: list[tuple[torch.Tensor, int]]) -> tuple[torch.Tensor, torch.Tensor]:
    """Collate function for image classification batches.

    Args:
        batch: List of (image, label) tuples

    Returns:
        (images, labels) tensors

    """
    images, labels = zip(*batch, strict=False)
    images = torch.stack(images)
    labels = torch.tensor(labels, dtype=torch.long)
    return images, labels


def collate_fn_text(batch: list[dict[str, torch.Tensor]]) -> dict[str, torch.Tensor]:
    """Collate function for text batches.

    Args:
        batch: List of tokenized text dictionaries

    Returns:
        Batched tokenized text dictionary

    """
    keys = batch[0].keys()
    batched = {}
    for key in keys:
        batched[key] = torch.stack([item[key] for item in batch])
    return batched

