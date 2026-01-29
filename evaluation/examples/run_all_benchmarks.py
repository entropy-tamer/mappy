"""Example: Run all benchmarks and generate comprehensive report.

This script demonstrates how to run all ML tasks and generate
a comprehensive evaluation report.
"""

import sys
from pathlib import Path

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from evaluation.benchmark_runner import BenchmarkRunner
from evaluation.reporting import generate_report


def main():
    """Run all benchmarks and generate report."""
    # Create benchmark runner
    runner = BenchmarkRunner(use_mappy=True, cache_data=True)

    # Run all tasks
    results = runner.run_all_tasks()

    # Generate summary
    runner.get_summary()

    # Generate reports
    report_paths = generate_report(
        results,
        output_dir="reports",
        format="both",
    )

    if "html" in report_paths:
        pass
    if "json" in report_paths:
        pass



if __name__ == "__main__":
    main()

