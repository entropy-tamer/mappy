#!/usr/bin/env python3
"""Run dataset comparison benchmark comparing mappy vs standard DataLoader.

This script runs comprehensive benchmarks comparing mappy DataLoader
performance against standard PyTorch DataLoader and HuggingFace datasets.
"""

import argparse
import json
import sys
from pathlib import Path

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent.parent))

from benchmarks.analysis import analyze_benchmark_results
from benchmarks.dataset_comparison import run_dataset_comparison_benchmark


def main():
    """Run dataset comparison benchmark."""
    parser = argparse.ArgumentParser(
        description="Compare mappy DataLoader vs standard DataLoader"
    )
    parser.add_argument(
        "--dataset-size",
        type=int,
        default=1000,
        help="Number of samples in dataset (default: 1000)",
    )
    parser.add_argument(
        "--batch-size",
        type=int,
        default=32,
        help="Batch size (default: 32)",
    )
    parser.add_argument(
        "--num-workers",
        type=int,
        default=0,
        help="Number of worker processes (default: 0)",
    )
    parser.add_argument(
        "--iterations",
        type=int,
        default=3,
        help="Number of benchmark iterations (default: 3)",
    )
    parser.add_argument(
        "--output-dir",
        type=str,
        default="reports",
        help="Output directory for reports (default: reports)",
    )

    args = parser.parse_args()

    print("=" * 60)
    print("Dataset Comparison Benchmark")
    print("=" * 60)
    print(f"Dataset size: {args.dataset_size}")
    print(f"Batch size: {args.batch_size}")
    print(f"Workers: {args.num_workers}")
    print(f"Iterations: {args.iterations}")
    print()

    # Run benchmark
    try:
        results = run_dataset_comparison_benchmark(
            dataset_size=args.dataset_size,
            batch_size=args.batch_size,
            num_workers=args.num_workers,
            num_iterations=args.iterations,
        )

        # Analyze results
        print("\n" + "=" * 60)
        print("Analysis")
        print("=" * 60)

        analysis = analyze_benchmark_results(results, output_dir=args.output_dir)

        # Print insights
        print("\nKey Insights:")
        for insight in analysis["insights"]:
            print(f"  {insight}")

        # Print summary
        if "statistical_summary" in analysis["report"]:
            summary = analysis["report"]["statistical_summary"]
            if "comparisons" in summary:
                comp = summary["comparisons"]
                print("\nPerformance Comparison:")
                print(
                    f"  Throughput ratio: {comp.get('throughput_ratio', 0):.3f} "
                    f"({comp.get('throughput_improvement_pct', 0):+.1f}%)"
                )
                print(
                    f"  Memory ratio: {comp.get('memory_ratio', 0):.3f} "
                    f"({comp.get('memory_savings_pct', 0):+.1f}% savings)"
                )
                print(
                    f"  Latency ratio: {comp.get('latency_ratio', 0):.3f} "
                    f"({comp.get('latency_improvement_pct', 0):+.1f}% improvement)"
                )

        # Save results
        output_path = Path(args.output_dir)
        output_path.mkdir(parents=True, exist_ok=True)

        results_file = output_path / "comparison_results.json"
        with open(results_file, "w") as f:
            json.dump(results, f, indent=2, default=str)

        print(f"\n✓ Results saved to {results_file}")
        print(f"✓ Analysis saved to {output_path / 'benchmark_analysis.json'}")
        if analysis["charts"]:
            print(f"✓ Charts saved to {output_path}")

        return 0

    except Exception as e:
        print(f"\n✗ Benchmark failed: {e}", file=sys.stderr)
        import traceback

        traceback.print_exc()
        return 1


if __name__ == "__main__":
    sys.exit(main())

