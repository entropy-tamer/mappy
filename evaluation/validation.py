"""Configuration validation for evaluation suite.

Comprehensive validation of task configurations, maplet settings,
and data format compatibility.
"""

from dataclasses import dataclass
from typing import Any


@dataclass
class ValidationResult:
    """Result of configuration validation."""

    is_valid: bool
    errors: list[str]
    warnings: list[str]

    def __str__(self) -> str:
        """String representation of validation result."""
        if self.is_valid:
            return "✓ Configuration is valid"
        error_msg = "\n".join(f"  ✗ {e}" for e in self.errors)
        warning_msg = "\n".join(f"  ⚠ {w}" for w in self.warnings)
        parts = ["✗ Configuration validation failed:"]
        if self.errors:
            parts.append("Errors:")
            parts.append(error_msg)
        if self.warnings:
            parts.append("Warnings:")
            parts.append(warning_msg)
        return "\n".join(parts)


class ConfigValidator:
    """Validates evaluation suite configurations."""

    @staticmethod
    def validate_task_config(
        task_name: str,
        config: dict[str, Any],
        dataset_size: int | None = None,
    ) -> ValidationResult:
        """Validate task-specific configuration.

        Args:
            task_name: Name of the task
            config: Task configuration dictionary
            dataset_size: Optional dataset size for capacity validation

        Returns:
            ValidationResult with validation status

        """
        errors: list[str] = []
        warnings: list[str] = []

        # Common validations
        batch_size = config.get("batch_size", 32)
        if not isinstance(batch_size, int) or batch_size <= 0:
            errors.append(f"batch_size must be a positive integer, got {batch_size}")

        num_epochs = config.get("num_epochs", 1)
        if not isinstance(num_epochs, int) or num_epochs <= 0:
            errors.append(f"num_epochs must be a positive integer, got {num_epochs}")

        # Task-specific validations
        if task_name == "image_classification":
            dataset_name = config.get("dataset_name", "cifar10")
            valid_datasets = ["cifar10", "synthetic", "imagenet"]
            if dataset_name not in valid_datasets:
                errors.append(
                    f"dataset_name must be one of {valid_datasets}, got {dataset_name}",
                )

        elif task_name in ["nlp_classification", "nlp_language_modeling"]:
            vocab_size = config.get("vocab_size", 10000)
            if not isinstance(vocab_size, int) or vocab_size <= 0:
                errors.append(
                    f"vocab_size must be a positive integer, got {vocab_size}",
                )

            max_length = config.get("max_length", 512)
            if not isinstance(max_length, int) or max_length <= 0:
                errors.append(
                    f"max_length must be a positive integer, got {max_length}",
                )

        elif task_name == "recommendation":
            num_users = config.get("num_users", 1000)
            if not isinstance(num_users, int) or num_users <= 0:
                errors.append(f"num_users must be a positive integer, got {num_users}")

            num_items = config.get("num_items", 100)
            if not isinstance(num_items, int) or num_items <= 0:
                errors.append(f"num_items must be a positive integer, got {num_items}")

        elif task_name == "image_diffusion":
            image_size = config.get("image_size", (64, 64))
            if not isinstance(image_size, tuple) or len(image_size) != 2:
                errors.append(
                    f"image_size must be a tuple of 2 integers, got {image_size}",
                )
            elif not all(isinstance(x, int) and x > 0 for x in image_size):
                errors.append(
                    f"image_size values must be positive integers, got {image_size}",
                )

            num_timesteps = config.get("num_timesteps", 1000)
            if not isinstance(num_timesteps, int) or num_timesteps <= 0:
                errors.append(
                    f"num_timesteps must be a positive integer, got {num_timesteps}",
                )

        elif task_name == "image_captioning":
            image_size = config.get("image_size", (224, 224))
            if not isinstance(image_size, tuple) or len(image_size) != 2:
                errors.append(
                    f"image_size must be a tuple of 2 integers, got {image_size}",
                )
            elif not all(isinstance(x, int) and x > 0 for x in image_size):
                errors.append(
                    f"image_size values must be positive integers, got {image_size}",
                )

            vocab_size = config.get("vocab_size", 10000)
            if not isinstance(vocab_size, int) or vocab_size <= 0:
                errors.append(
                    f"vocab_size must be a positive integer, got {vocab_size}",
                )

            max_caption_length = config.get("max_caption_length", 20)
            if not isinstance(max_caption_length, int) or max_caption_length <= 0:
                errors.append(
                    f"max_caption_length must be a positive integer, "
                    f"got {max_caption_length}",
                )

        # Capacity validation if dataset size provided
        if dataset_size is not None:
            capacity = config.get("maplet_capacity", 100000)
            if capacity < dataset_size:
                warnings.append(
                    f"maplet_capacity ({capacity}) is less than dataset size "
                    f"({dataset_size}), may cause overflow",
                )
            elif capacity < dataset_size * 1.2:
                warnings.append(
                    f"maplet_capacity ({capacity}) is close to dataset size "
                    f"({dataset_size}), recommend at least 1.2x for safety",
                )

        return ValidationResult(
            is_valid=len(errors) == 0,
            errors=errors,
            warnings=warnings,
        )

    @staticmethod
    def validate_maplet_config(
        capacity: int,
        false_positive_rate: float,
        dataset_size: int | None = None,
    ) -> ValidationResult:
        """Validate maplet configuration.

        Args:
            capacity: Maplet capacity
            false_positive_rate: False positive rate (0.0 to 1.0)
            dataset_size: Optional dataset size for validation

        Returns:
            ValidationResult

        """
        errors: list[str] = []
        warnings: list[str] = []

        if not isinstance(capacity, int) or capacity <= 0:
            errors.append(f"capacity must be a positive integer, got {capacity}")

        if not isinstance(false_positive_rate, float):
            errors.append(
                f"false_positive_rate must be a float, got {type(false_positive_rate)}",
            )
        elif not 0.0 < false_positive_rate < 1.0:
            errors.append(
                f"false_positive_rate must be between 0.0 and 1.0, "
                f"got {false_positive_rate}",
            )

        if dataset_size is not None:
            if capacity < dataset_size:
                errors.append(
                    f"capacity ({capacity}) must be >= dataset size ({dataset_size})",
                )
            elif capacity < dataset_size * 1.1:
                warnings.append(
                    f"capacity ({capacity}) is close to dataset size ({dataset_size}), "
                    f"recommend at least 1.1x for optimal performance",
                )

        return ValidationResult(
            is_valid=len(errors) == 0,
            errors=errors,
            warnings=warnings,
        )

    @staticmethod
    def validate_dataloader_config(
        batch_size: int,
        num_workers: int,
        dataset_size: int | None = None,
    ) -> ValidationResult:
        """Validate DataLoader configuration.

        Args:
            batch_size: Batch size
            num_workers: Number of worker processes
            dataset_size: Optional dataset size

        Returns:
            ValidationResult

        """
        errors: list[str] = []
        warnings: list[str] = []

        if not isinstance(batch_size, int) or batch_size <= 0:
            errors.append(f"batch_size must be a positive integer, got {batch_size}")

        if not isinstance(num_workers, int) or num_workers < 0:
            errors.append(
                f"num_workers must be a non-negative integer, got {num_workers}",
            )

        if dataset_size is not None and batch_size > dataset_size:
            warnings.append(
                f"batch_size ({batch_size}) is larger than dataset size "
                f"({dataset_size}), will result in single batch",
            )

        if num_workers > 8:
            warnings.append(
                f"num_workers ({num_workers}) is high, may cause overhead. "
                f"Recommended: 0-4 for most cases",
            )

        return ValidationResult(
            is_valid=len(errors) == 0,
            errors=errors,
            warnings=warnings,
        )

    @staticmethod
    def validate_data_format(
        data: Any,
        expected_type: type | tuple[type, ...],
        task_name: str,
    ) -> ValidationResult:
        """Validate data format compatibility.

        Args:
            data: Data to validate
            expected_type: Expected data type(s)
            task_name: Name of task for context

        Returns:
            ValidationResult

        """
        errors: list[str] = []
        warnings: list[str] = []

        if not isinstance(data, expected_type):
            errors.append(
                f"Data type mismatch for {task_name}: expected {expected_type}, "
                f"got {type(data)}",
            )

        return ValidationResult(
            is_valid=len(errors) == 0,
            errors=errors,
            warnings=warnings,
        )

    @staticmethod
    def validate_benchmark_config(
        use_mappy: bool,
        cache_data: bool,
        task_configs: dict[str, dict[str, Any]],
    ) -> ValidationResult:
        """Validate complete benchmark configuration.

        Args:
            use_mappy: Whether to use mappy
            cache_data: Whether to cache data
            task_configs: Task-specific configurations

        Returns:
            ValidationResult

        """
        errors: list[str] = []
        warnings: list[str] = []

        if not isinstance(use_mappy, bool):
            errors.append(f"use_mappy must be a boolean, got {type(use_mappy)}")

        if not isinstance(cache_data, bool):
            errors.append(f"cache_data must be a boolean, got {type(cache_data)}")

        if cache_data and not use_mappy:
            warnings.append(
                "cache_data=True with use_mappy=False has no effect, "
                "consider setting cache_data=False",
            )

        # Validate each task config
        valid_tasks = [
            "image_classification",
            "nlp_classification",
            "nlp_language_modeling",
            "recommendation",
            "image_diffusion",
            "image_captioning",
        ]

        for task_name, task_config in task_configs.items():
            if task_name not in valid_tasks:
                errors.append(f"Unknown task: {task_name}")
                continue

            task_result = ConfigValidator.validate_task_config(
                task_name, task_config,
            )
            errors.extend(task_result.errors)
            warnings.extend(task_result.warnings)

        return ValidationResult(
            is_valid=len(errors) == 0,
            errors=errors,
            warnings=warnings,
        )

