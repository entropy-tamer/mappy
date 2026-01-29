"""GPU monitoring utilities for evaluation suite."""


import torch

try:
    import pynvml
    PYNVML_AVAILABLE = True
except ImportError:
    PYNVML_AVAILABLE = False


def get_gpu_utilization() -> float:
    """Get current GPU utilization percentage.

    Returns:
        GPU utilization (0-100) or 0.0 if unavailable

    """
    if not PYNVML_AVAILABLE:
        return 0.0

    try:
        pynvml.nvmlInit()
        handle = pynvml.nvmlDeviceGetHandleByIndex(0)
        util = pynvml.nvmlDeviceGetUtilizationRates(handle)
        return float(util.gpu)
    except Exception:
        return 0.0


def get_gpu_memory() -> tuple[float, float]:
    """Get current GPU memory usage.

    Returns:
        (used_mb, total_mb) or (0.0, 0.0) if unavailable

    """
    if not PYNVML_AVAILABLE:
        return 0.0, 0.0

    try:
        pynvml.nvmlInit()
        handle = pynvml.nvmlDeviceGetHandleByIndex(0)
        mem_info = pynvml.nvmlDeviceGetMemoryInfo(handle)
        used_mb = mem_info.used / 1024**2
        total_mb = mem_info.total / 1024**2
        return float(used_mb), float(total_mb)
    except Exception:
        return 0.0, 0.0


def is_gpu_available() -> bool:
    """Check if GPU is available.

    Returns:
        True if GPU is available

    """
    return torch.cuda.is_available()


def get_device() -> torch.device:
    """Get the best available device.

    Returns:
        torch.device (cuda or cpu)

    """
    return torch.device("cuda" if torch.cuda.is_available() else "cpu")

