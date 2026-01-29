"""Example: Run MNIST evaluation with mappy DataLoader.

This script runs a real MNIST training evaluation using PyTorch
with the mappy DataLoader to demonstrate real-world performance.
"""

import logging
import sys
from pathlib import Path

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

import torch
from evaluation.benchmark_runner import run_single_task
from evaluation.reporting import generate_report

# Configure logging
logging.basicConfig(
    level=logging.DEBUG,
    format='%(asctime)s [%(levelname)s] %(name)s: %(message)s',
    datefmt='%Y-%m-%d %H:%M:%S'
)
logger = logging.getLogger(__name__)


def main():
    """Run MNIST evaluation with mappy DataLoader."""
    import argparse

    parser = argparse.ArgumentParser(description="Run MNIST evaluation with mappy")
    parser.add_argument(
        "--use-mappy",
        action="store_true",
        default=True,
        help="Use mappy DataLoader (default: True)",
    )
    parser.add_argument(
        "--no-mappy",
        action="store_true",
        help="Disable mappy DataLoader (use standard PyTorch DataLoader)",
    )
    parser.add_argument(
        "--no-cache",
        action="store_true",
        help="Disable data caching",
    )
    parser.add_argument(
        "--batch-size",
        type=int,
        default=64,
        help="Batch size for training (default: 64)",
    )
    parser.add_argument(
        "--epochs",
        type=int,
        default=3,
        help="Number of training epochs (default: 3)",
    )
    parser.add_argument(
        "--device",
        type=str,
        default=None,
        help="Device to use (cuda/cpu, default: auto-detect)",
    )
    parser.add_argument(
        "--log-level",
        type=str,
        default="INFO",
        choices=["DEBUG", "INFO", "WARNING", "ERROR"],
        help="Logging level (default: INFO)",
    )

    args = parser.parse_args()

    # Set logging level
    log_level = getattr(logging, args.log_level.upper())
    logging.getLogger().setLevel(log_level)
    logger.setLevel(log_level)

    # Determine device
    if args.device:
        device = torch.device(args.device)
        logger.debug(f"Using specified device: {args.device}")
    else:
        device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
        logger.debug(f"Auto-detected device: {device}")

    logger.info("=" * 70)
    logger.info("MNIST Evaluation with Mappy DataLoader")
    logger.info("=" * 70)
    logger.info(f"Device: {device}")
    logger.info(f"Use Mappy: {args.use_mappy and not args.no_mappy}")
    logger.info(f"Cache Data: {not args.no_cache}")
    logger.info(f"Batch Size: {args.batch_size}")
    logger.info(f"Epochs: {args.epochs}")
    logger.info(f"Log Level: {args.log_level}")
    logger.info("=" * 70)
    logger.info("")

    # Run task with MNIST configuration
    task_config = {
        "dataset_name": "mnist",
        "batch_size": args.batch_size,
        "num_epochs": args.epochs,
    }

    use_mappy = args.use_mappy and not args.no_mappy

    logger.info("Starting MNIST training evaluation...")
    logger.info("This will download MNIST dataset if not already present.")
    logger.info("")
    logger.debug(f"Task config: {task_config}")
    logger.debug(f"Using mappy: {use_mappy}")
    logger.debug(f"Caching data: {not args.no_cache}")

    try:
        logger.info("Calling run_single_task...")
        result = run_single_task(
            task_name="image_classification",
            use_mappy=use_mappy,
            cache_data=not args.no_cache,
            task_config=task_config,
        )
        logger.debug(f"Task completed, received result: {type(result)}")

        # Print results summary
        logger.info("")
        logger.info("=" * 70)
        logger.info("Evaluation Results")
        logger.info("=" * 70)
        logger.info(f"Task: {result.task_name}")
        logger.info(f"Accuracy: {result.accuracy.accuracy:.4f}")
        logger.info(f"Throughput: {result.performance.samples_per_second:.2f} samples/sec")
        logger.info(f"Total Time: {result.performance.total_time_seconds:.2f} seconds")
        logger.info(f"Peak Memory: {result.performance.peak_memory_mb:.2f} MB")
        if result.performance.avg_batch_time_ms:
            logger.info(f"Avg Batch Time: {result.performance.avg_batch_time_ms:.2f} ms")
        logger.info("=" * 70)

        # Generate report
        logger.info("")
        logger.info("Generating report...")
        logger.debug(f"Report output directory: reports")
        logger.debug(f"Report format: both (HTML + JSON)")
        report_paths = generate_report(
            [result],
            output_dir="reports",
            format="both",
        )
        logger.debug(f"Report paths generated: {report_paths}")

        if "html" in report_paths:
            logger.info(f"HTML report: {report_paths['html']}")
        if "json" in report_paths:
            logger.info(f"JSON report: {report_paths['json']}")

        logger.info("")
        logger.info("✅ Evaluation complete!")

    except Exception as e:
        logger.error(f"❌ Error during evaluation: {e}", exc_info=True)
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()

