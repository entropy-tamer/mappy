"""Benchmark runner for orchestrating all ML task evaluations.

Runs all tasks, collects metrics, and generates comprehensive reports.
"""

from typing import Any

import torch

from .metrics import EvaluationResults
from .tasks import (
    ImageCaptioningTask,
    ImageClassificationTask,
    ImageDiffusionTask,
    NLPTask,
    RecommendationTask,
)


class BenchmarkRunner:
    """Orchestrates evaluation of all ML tasks."""

    def __init__(
        self,
        use_mappy: bool = True,
        cache_data: bool = True,
        device: torch.device | None = None,
    ):
        """Initialize benchmark runner.

        Args:
            use_mappy: Whether to use mappy DataLoader
            cache_data: Whether to cache data
            device: Optional device (defaults to cuda if available)

        """
        self.use_mappy = use_mappy
        self.cache_data = cache_data
        self.device = device or torch.device(
            "cuda" if torch.cuda.is_available() else "cpu",
        )

        self.results: list[EvaluationResults] = []

    def run_task(
        self,
        task_name: str,
        task_config: dict[str, Any] | None = None,
    ) -> EvaluationResults:
        """Run a single task.

        Args:
            task_name: Name of task to run
            task_config: Optional task-specific configuration

        Returns:
            EvaluationResults

        """
        task_config = task_config or {}

        # Create task instance
        if task_name == "image_classification":
            task = ImageClassificationTask(
                dataset_name=task_config.get("dataset_name", "cifar10"),
                batch_size=task_config.get("batch_size", 32),
                num_epochs=task_config.get("num_epochs", 1),
                use_mappy=self.use_mappy,
                cache_data=self.cache_data,
            )
        elif task_name == "nlp_classification":
            task = NLPTask(
                task_type="classification",
                batch_size=task_config.get("batch_size", 32),
                num_epochs=task_config.get("num_epochs", 1),
                use_mappy=self.use_mappy,
                cache_data=self.cache_data,
            )
        elif task_name == "nlp_language_modeling":
            task = NLPTask(
                task_type="language_modeling",
                batch_size=task_config.get("batch_size", 32),
                num_epochs=task_config.get("num_epochs", 1),
                use_mappy=self.use_mappy,
                cache_data=self.cache_data,
            )
        elif task_name == "recommendation":
            task = RecommendationTask(
                num_users=task_config.get("num_users", 1000),
                num_items=task_config.get("num_items", 100),
                batch_size=task_config.get("batch_size", 32),
                num_epochs=task_config.get("num_epochs", 1),
                use_mappy=self.use_mappy,
                cache_data=self.cache_data,
            )
        elif task_name == "image_diffusion":
            task = ImageDiffusionTask(
                image_size=task_config.get("image_size", (64, 64)),
                batch_size=task_config.get("batch_size", 8),
                num_epochs=task_config.get("num_epochs", 1),
                use_mappy=self.use_mappy,
                cache_data=self.cache_data,
            )
        elif task_name == "image_captioning":
            task = ImageCaptioningTask(
                image_size=task_config.get("image_size", (224, 224)),
                batch_size=task_config.get("batch_size", 16),
                num_epochs=task_config.get("num_epochs", 1),
                use_mappy=self.use_mappy,
                cache_data=self.cache_data,
            )
        else:
            msg = f"Unknown task: {task_name}"
            raise ValueError(msg)

        # Run evaluation
        eval_result = task.run_evaluation(device=self.device)
        return eval_result["results"]

    def run_all_tasks(
        self,
        task_configs: dict[str, dict[str, Any]] | None = None,
        task_filter: list[str] | None = None,
    ) -> list[EvaluationResults]:
        """Run all available tasks.

        Args:
            task_configs: Optional task-specific configurations
            task_filter: Optional list of task names to run (None = all)

        Returns:
            List of EvaluationResults

        """
        task_configs = task_configs or {}
        task_filter = task_filter or []

        # All available tasks
        all_tasks = [
            "image_classification",
            "nlp_classification",
            "nlp_language_modeling",
            "recommendation",
            "image_diffusion",
            "image_captioning",
        ]

        # Filter tasks if specified
        tasks_to_run = task_filter if task_filter else all_tasks

        self.results = []

        for task_name in tasks_to_run:
            if task_name not in all_tasks:
                continue

            try:
                task_config = task_configs.get(task_name, {})
                result = self.run_task(task_name, task_config)
                self.results.append(result)
            except Exception as e:
                import logging

                logging.warning(
                    f"Failed to run task {task_name}: {e}", exc_info=True,
                )
                continue

        return self.results

    def compare_with_baseline(
        self,
        baseline_results: list[EvaluationResults],
    ) -> dict[str, Any]:
        """Compare mappy results with baseline (exact storage).

        Args:
            baseline_results: List of baseline EvaluationResults

        Returns:
            Comparison dictionary

        """
        comparison = {
            "tasks": [],
            "accuracy_differences": [],
            "speed_ratios": [],
            "memory_ratios": [],
        }

        # Create baseline lookup
        baseline_lookup = {r.task_name: r for r in baseline_results}

        for result in self.results:
            task_name = result.task_name
            if task_name not in baseline_lookup:
                continue

            baseline = baseline_lookup[task_name]

            # Compare accuracy
            accuracy_diff = abs(
                result.accuracy.accuracy - baseline.accuracy.accuracy,
            )

            # Compare speed (throughput)
            speed_ratio = (
                result.performance.samples_per_second
                / baseline.performance.samples_per_second
                if baseline.performance.samples_per_second > 0
                else 0.0
            )

            # Compare memory
            memory_ratio = (
                result.performance.peak_memory_mb
                / baseline.performance.peak_memory_mb
                if baseline.performance.peak_memory_mb > 0
                else 0.0
            )

            comparison["tasks"].append(task_name)
            comparison["accuracy_differences"].append(accuracy_diff)
            comparison["speed_ratios"].append(speed_ratio)
            comparison["memory_ratios"].append(memory_ratio)

        return comparison

    def get_summary(self) -> dict[str, Any]:
        """Get summary statistics across all tasks.

        Returns:
            Summary dictionary

        """
        if not self.results:
            return {}

        num_tasks = len(self.results)
        return {
            "num_tasks": num_tasks,
            "tasks": [r.task_name for r in self.results],
            "avg_accuracy": (
                sum(r.accuracy.accuracy for r in self.results) / num_tasks
            ),
            "avg_throughput": (
                sum(r.performance.samples_per_second for r in self.results) / num_tasks
            ),
            "avg_memory_mb": (
                sum(r.performance.peak_memory_mb for r in self.results) / num_tasks
            ),
            "total_time_seconds": (
                sum(r.performance.total_time_seconds for r in self.results)
            ),
        }


    def run_comparison_benchmark(
        self,
        task_configs: dict[str, dict[str, Any]] | None = None,
        task_filter: list[str] | None = None,
    ) -> dict[str, Any]:
        """Run benchmark comparing mappy vs baseline.

        Args:
            task_configs: Optional task configurations
            task_filter: Optional task filter

        Returns:
            Comparison results dictionary

        """
        # Run with mappy
        self.use_mappy = True
        mappy_results = self.run_all_tasks(task_configs, task_filter)

        # Run baseline (without mappy)
        self.use_mappy = False
        baseline_results = self.run_all_tasks(task_configs, task_filter)

        # Compare
        comparison = self.compare_with_baseline(baseline_results)

        return {
            "mappy_results": mappy_results,
            "baseline_results": baseline_results,
            "comparison": comparison,
            "summary": {
                "mappy": self.get_summary(),
            },
        }


def run_single_task(
    task_name: str,
    use_mappy: bool = True,
    cache_data: bool = True,
    task_config: dict[str, Any] | None = None,
) -> EvaluationResults:
    """Run a single task evaluation.

    Args:
        task_name: Name of task
        use_mappy: Whether to use mappy
        cache_data: Whether to cache data
        task_config: Optional task configuration

    Returns:
        EvaluationResults

    """
    runner = BenchmarkRunner(use_mappy=use_mappy, cache_data=cache_data)
    return runner.run_task(task_name, task_config)


def run_all_benchmarks(
    use_mappy: bool = True,
    cache_data: bool = True,
    task_configs: dict[str, dict[str, Any]] | None = None,
    task_filter: list[str] | None = None,
) -> list[EvaluationResults]:
    """Run all benchmark evaluations.

    Args:
        use_mappy: Whether to use mappy
        cache_data: Whether to cache data
        task_configs: Optional task configurations
        task_filter: Optional task filter

    Returns:
        List of EvaluationResults

    """
    runner = BenchmarkRunner(use_mappy=use_mappy, cache_data=cache_data)
    return runner.run_all_tasks(task_configs, task_filter)

