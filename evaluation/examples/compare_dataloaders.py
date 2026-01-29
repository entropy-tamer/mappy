"""Example: Compare mappy DataLoader vs standard DataLoader.

This script demonstrates how to run a comparison benchmark
between mappy and standard PyTorch DataLoader.
"""

import sys
from pathlib import Path

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from evaluation.benchmark_runner import BenchmarkRunner
from evaluation.reporting import generate_report


def main():
    """Run comparison benchmark."""
    import argparse

    parser = argparse.ArgumentParser(description="Compare mappy vs standard DataLoader")
    parser.add_argument(
        "--tasks",
        type=str,
        nargs="+",
        default=None,
        help="Specific tasks to run (default: all)",
    )
    parser.add_argument(
        "--output-dir",
        type=str,
        default="reports",
        help="Output directory for reports",
    )

    args = parser.parse_args()


    # Create benchmark runner
    runner = BenchmarkRunner(use_mappy=True, cache_data=True)

    # Run comparison
    comparison_results = runner.run_comparison_benchmark(task_filter=args.tasks)

    # Print comparison summary

    comparison = comparison_results["comparison"]
    if comparison["tasks"]:
        for _task, _diff in zip(comparison["tasks"], comparison["accuracy_differences"], strict=False):
            pass

        for _task, _ratio in zip(comparison["tasks"], comparison["speed_ratios"], strict=False):
            pass

        for _task, _ratio in zip(comparison["tasks"], comparison["memory_ratios"], strict=False):
            pass

    # Generate reports
    report_paths = generate_report(
        comparison_results["mappy_results"],
        comparison=comparison,
        output_dir=args.output_dir,
        format="both",
    )

    if "html" in report_paths:
        pass
    if "json" in report_paths:
        pass



if __name__ == "__main__":
    main()

