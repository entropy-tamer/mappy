"""Example: Run a single task evaluation.

This script demonstrates how to run a single ML task evaluation.
"""

import sys
from pathlib import Path

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from evaluation.benchmark_runner import run_single_task
from evaluation.reporting import generate_report


def main():
    """Run a single task evaluation."""
    import argparse

    parser = argparse.ArgumentParser(description="Run single task evaluation")
    parser.add_argument(
        "--task",
        type=str,
        default="image_classification",
        choices=[
            "image_classification",
            "nlp_classification",
            "nlp_language_modeling",
            "recommendation",
            "image_diffusion",
            "image_captioning",
        ],
        help="Task to run",
    )
    parser.add_argument(
        "--use-mappy",
        action="store_true",
        default=True,
        help="Use mappy DataLoader",
    )
    parser.add_argument(
        "--no-cache",
        action="store_true",
        help="Disable data caching",
    )

    args = parser.parse_args()


    # Run task
    result = run_single_task(
        task_name=args.task,
        use_mappy=args.use_mappy,
        cache_data=not args.no_cache,
    )

    # Print results

    # Generate report
    report_paths = generate_report(
        [result],
        output_dir="reports",
        format="both",
    )

    if "html" in report_paths:
        pass
    if "json" in report_paths:
        pass


if __name__ == "__main__":
    main()

