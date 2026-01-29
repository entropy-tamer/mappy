"""Comprehensive metrics collection for mappy DataLoader evaluation.

Tracks performance metrics (throughput, latency, memory, GPU) and
accuracy metrics for ML tasks.
"""

import logging
import threading
import time
from dataclasses import dataclass, field
from typing import Any

import numpy as np
import psutil

try:
    import pynvml
    PYNVML_AVAILABLE = True
except ImportError:
    PYNVML_AVAILABLE = False

logger = logging.getLogger(__name__)


@dataclass
class PerformanceMetrics:
    """Performance metrics for data loading."""

    # Throughput metrics
    samples_per_second: float = 0.0
    batches_per_second: float = 0.0
    total_samples: int = 0
    total_batches: int = 0

    # Latency metrics
    first_batch_time_ms: float = 0.0
    avg_batch_time_ms: float = 0.0
    min_batch_time_ms: float = float("inf")
    max_batch_time_ms: float = 0.0
    batch_times_ms: list[float] = field(default_factory=list)

    # Memory metrics
    peak_memory_mb: float = 0.0
    avg_memory_mb: float = 0.0
    memory_per_sample_mb: float = 0.0
    cache_size_mb: float = 0.0
    memory_samples: list[float] = field(default_factory=list)

    # GPU metrics
    gpu_utilization_pct: float = 0.0  # Average utilization
    gpu_memory_mb: float = 0.0  # Average memory
    gpu_idle_time_ms: float = 0.0
    gpu_samples: int = 0
    _gpu_utilization_samples: list[float] = field(default_factory=list)  # Internal tracking
    _gpu_memory_samples: list[float] = field(default_factory=list)  # Internal tracking

    # DataLoader-specific metrics
    worker_efficiency: float = 0.0
    queue_utilization: float = 0.0
    prefetch_effectiveness: float = 0.0
    cache_hit_rate: float = 0.0

    # Timing
    total_time_seconds: float = 0.0
    data_loading_time_seconds: float = 0.0
    preprocessing_time_seconds: float = 0.0


@dataclass
class AccuracyMetrics:
    """Accuracy metrics for ML tasks."""

    task_name: str = ""
    accuracy: float = 0.0
    precision: float = 0.0
    recall: float = 0.0
    f1_score: float = 0.0

    # Task-specific metrics
    perplexity: float | None = None  # For language modeling
    bleu_score: float | None = None  # For captioning
    meteor_score: float | None = None  # For captioning
    cider_score: float | None = None  # For captioning
    fid_score: float | None = None  # For diffusion
    rmse: float | None = None  # For recommendation
    precision_at_k: float | None = None  # For recommendation
    recall_at_k: float | None = None  # For recommendation

    # Comparison metrics
    accuracy_difference: float = 0.0  # vs baseline
    is_statistically_significant: bool = False


@dataclass
class EvaluationResults:
    """Complete evaluation results for a task."""

    task_name: str
    performance: PerformanceMetrics
    accuracy: AccuracyMetrics
    config: dict[str, Any]
    timestamp: float = field(default_factory=time.time)


class MetricsCollector:
    """Collects comprehensive metrics during evaluation."""

    def __init__(self, track_gpu: bool = True):
        """Initialize metrics collector.

        Args:
            track_gpu: Whether to track GPU metrics (requires pynvml)

        """
        self.track_gpu = track_gpu and PYNVML_AVAILABLE
        self.performance = PerformanceMetrics()
        self.accuracy = AccuracyMetrics()

        # Threading for background monitoring
        self._monitoring = False
        self._monitor_thread: threading.Thread | None = None
        self._memory_thread: threading.Thread | None = None
        self._start_time: float | None = None
        self._batch_times: list[tuple[int, float]] = []

        # Initialize GPU monitoring if available
        if self.track_gpu:
            try:
                logger.debug("Initializing GPU monitoring (pynvml)...")
                pynvml.nvmlInit()
                self.gpu_handle = pynvml.nvmlDeviceGetHandleByIndex(0)
                logger.debug("GPU monitoring initialized successfully")
            except Exception as e:
                logger.debug(f"GPU monitoring not available: {e}")
                self.track_gpu = False
        else:
            logger.debug("GPU monitoring disabled (track_gpu=False or pynvml not available)")

    def start_monitoring(self):
        """Start background monitoring of system metrics."""
        logger.debug("Starting metrics monitoring...")
        self._monitoring = True
        self._start_time = time.time()
        self.performance.memory_samples = []
        self.performance._gpu_utilization_samples = []
        self.performance._gpu_memory_samples = []

        if self.track_gpu:
            logger.debug("Starting GPU monitoring thread...")
            self._monitor_thread = threading.Thread(target=self._monitor_gpu, daemon=True)
            self._monitor_thread.start()
            logger.debug("GPU monitoring thread started")

        # Start memory monitoring in background thread
        logger.debug("Starting memory monitoring thread...")
        self._memory_thread = threading.Thread(target=self._monitor_memory, daemon=True)
        self._memory_thread.start()
        logger.debug("Memory monitoring thread started")
        logger.debug("Metrics monitoring started successfully")

    def stop_monitoring(self):
        """Stop background monitoring."""
        logger.debug("Stopping metrics monitoring...")
        self._monitoring = False
        
        # Wait for threads to finish
        if self._monitor_thread is not None:
            logger.debug("Waiting for GPU monitoring thread to finish...")
            self._monitor_thread.join(timeout=1.0)
        if self._memory_thread is not None:
            logger.debug("Waiting for memory monitoring thread to finish...")
            self._memory_thread.join(timeout=1.0)

        # Calculate final metrics
        logger.debug("Calculating final metrics...")
        self._calculate_metrics()
        logger.debug("Metrics monitoring stopped")

    def _monitor_gpu(self):
        """Background thread to monitor GPU metrics."""
        while self._monitoring:
            try:
                if self.track_gpu:
                    # GPU utilization
                    util = pynvml.nvmlDeviceGetUtilizationRates(self.gpu_handle)
                    self.performance._gpu_utilization_samples.append(float(util.gpu))

                    # GPU memory
                    mem_info = pynvml.nvmlDeviceGetMemoryInfo(self.gpu_handle)
                    self.performance._gpu_memory_samples.append(mem_info.used / 1024**2)

                time.sleep(0.1)  # Sample every 100ms
            except Exception:
                break

    def _monitor_memory(self):
        """Monitor system memory."""
        process = psutil.Process()
        while self._monitoring:
            try:
                mem_info = process.memory_info()
                self.performance.memory_samples.append(mem_info.rss / 1024**2)  # MB
                time.sleep(0.5)  # Sample every 500ms
            except Exception:
                break

    def record_batch(
        self,
        batch_idx: int,
        batch_size: int,
        batch_time_ms: float,
        data_loading_time_ms: float | None = None,
    ):
        """Record a batch loading event.

        Args:
            batch_idx: Batch index
            batch_size: Number of samples in batch
            batch_time_ms: Time to load batch in milliseconds
            data_loading_time_ms: Optional time spent in data loading (vs preprocessing)

        """
        self.performance.batch_times_ms.append(batch_time_ms)
        self.performance.total_batches += 1
        self.performance.total_samples += batch_size

        if batch_idx == 0:
            self.performance.first_batch_time_ms = batch_time_ms

        if data_loading_time_ms is not None:
            self.performance.data_loading_time_seconds += data_loading_time_ms / 1000.0

    def record_training_step(
        self,
        step_time_ms: float,
        gpu_idle_time_ms: float | None = None,
    ):
        """Record a training step (for GPU idle time calculation).

        Args:
            step_time_ms: Time for training step
            gpu_idle_time_ms: Optional GPU idle time during step

        """
        if gpu_idle_time_ms is not None:
            self.performance.gpu_idle_time_ms += gpu_idle_time_ms
            self.performance.gpu_samples += 1

    def _calculate_metrics(self):
        """Calculate final performance metrics."""
        if not self.performance.batch_times_ms:
            return

        # Latency metrics
        self.performance.avg_batch_time_ms = np.mean(self.performance.batch_times_ms)
        self.performance.min_batch_time_ms = np.min(self.performance.batch_times_ms)
        self.performance.max_batch_time_ms = np.max(self.performance.batch_times_ms)

        # Throughput metrics
        if self._start_time is not None:
            total_time = time.time() - self._start_time
            self.performance.total_time_seconds = total_time

            if total_time > 0:
                self.performance.samples_per_second = self.performance.total_samples / total_time
                self.performance.batches_per_second = self.performance.total_batches / total_time

        # Memory metrics
        if self.performance.memory_samples:
            self.performance.peak_memory_mb = np.max(self.performance.memory_samples)
            self.performance.avg_memory_mb = np.mean(self.performance.memory_samples)

            if self.performance.total_samples > 0:
                self.performance.memory_per_sample_mb = (
                    self.performance.peak_memory_mb / self.performance.total_samples
                )

        # GPU metrics - calculate averages
        if self.performance._gpu_utilization_samples:
            self.performance.gpu_utilization_pct = float(np.mean(self.performance._gpu_utilization_samples))

        if self.performance._gpu_memory_samples:
            self.performance.gpu_memory_mb = float(np.mean(self.performance._gpu_memory_samples))

    def set_accuracy_metrics(
        self,
        task_name: str,
        accuracy: float,
        precision: float | None = None,
        recall: float | None = None,
        f1: float | None = None,
        **kwargs,
    ):
        """Set accuracy metrics for the task.

        Args:
            task_name: Name of the task
            accuracy: Accuracy score
            precision: Optional precision score
            recall: Optional recall score
            f1: Optional F1 score
            **kwargs: Additional task-specific metrics

        """
        self.accuracy.task_name = task_name
        self.accuracy.accuracy = accuracy
        if precision is not None:
            self.accuracy.precision = precision
        if recall is not None:
            self.accuracy.recall = recall
        if f1 is not None:
            self.accuracy.f1_score = f1

        # Set task-specific metrics
        for key, value in kwargs.items():
            if hasattr(self.accuracy, key):
                setattr(self.accuracy, key, value)

    def compare_with_baseline(self, baseline_accuracy: float, baseline_performance: PerformanceMetrics):
        """Compare metrics with baseline (exact storage).

        Args:
            baseline_accuracy: Baseline accuracy
            baseline_performance: Baseline performance metrics

        """
        self.accuracy.accuracy_difference = abs(self.accuracy.accuracy - baseline_accuracy)

        # Simple statistical significance test (can be improved)
        if self.accuracy.accuracy_difference < 0.01:  # 1% threshold
            self.accuracy.is_statistically_significant = False
        else:
            self.accuracy.is_statistically_significant = True

    def get_results(self, task_name: str, config: dict[str, Any]) -> EvaluationResults:
        """Get complete evaluation results.

        Args:
            task_name: Name of the task
            config: Configuration used for evaluation

        Returns:
            EvaluationResults object

        """
        return EvaluationResults(
            task_name=task_name,
            performance=self.performance,
            accuracy=self.accuracy,
            config=config,
        )

    def reset(self):
        """Reset all metrics."""
        self.performance = PerformanceMetrics()
        self.accuracy = AccuracyMetrics()
        self._batch_times = []
        self._start_time = None


def calculate_accuracy_metrics(
    predictions: np.ndarray,
    targets: np.ndarray,
    task_type: str = "classification",
) -> dict[str, float]:
    """Calculate accuracy metrics for predictions.

    Args:
        predictions: Model predictions
        targets: Ground truth targets
        task_type: Type of task (classification, regression, etc.)

    Returns:
        Dictionary of metric names to values

    """
    from sklearn.metrics import (
        accuracy_score,
        f1_score,
        mean_squared_error,
        precision_score,
        recall_score,
    )

    metrics = {}

    if task_type == "classification":
        metrics["accuracy"] = accuracy_score(targets, predictions)
        metrics["precision"] = precision_score(targets, predictions, average="weighted", zero_division=0)
        metrics["recall"] = recall_score(targets, predictions, average="weighted", zero_division=0)
        metrics["f1_score"] = f1_score(targets, predictions, average="weighted", zero_division=0)
    elif task_type == "regression":
        metrics["rmse"] = np.sqrt(mean_squared_error(targets, predictions))
        metrics["mae"] = np.mean(np.abs(predictions - targets))

    return metrics

