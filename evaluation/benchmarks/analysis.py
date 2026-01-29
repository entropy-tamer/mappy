"""Empirical analysis of benchmark results.

Statistical analysis, visualization, and insights from benchmark comparisons.
"""

import json
from pathlib import Path
from typing import Any

import numpy as np

try:
    import plotly.graph_objects as go

    PLOTLY_AVAILABLE = True
except ImportError:
    PLOTLY_AVAILABLE = False


class BenchmarkAnalyzer:
    """Analyzes benchmark results and generates insights."""

    def __init__(self, results: dict[str, Any]):
        """Initialize analyzer with benchmark results.

        Args:
            results: Benchmark results dictionary

        """
        self.results = results
        self.analysis = results.get("analysis", {})

    def generate_statistical_summary(self) -> dict[str, Any]:
        """Generate statistical summary of results.

        Returns:
            Dictionary with statistical metrics

        """
        summary: dict[str, Any] = {
            "approaches": {},
            "comparisons": {},
            "statistical_tests": {},
        }

        # Extract data for each approach
        for approach, result_list in self.results.get("results", {}).items():
            if not result_list:
                continue

            throughputs = [r.throughput_samples_per_sec for r in result_list]
            memories = [r.peak_memory_mb for r in result_list]
            latencies = [r.avg_batch_time_ms for r in result_list]

            summary["approaches"][approach] = {
                "throughput": {
                    "mean": float(np.mean(throughputs)),
                    "std": float(np.std(throughputs)),
                    "min": float(np.min(throughputs)),
                    "max": float(np.max(throughputs)),
                    "median": float(np.median(throughputs)),
                },
                "memory": {
                    "mean": float(np.mean(memories)),
                    "std": float(np.std(memories)),
                    "min": float(np.min(memories)),
                    "max": float(np.max(memories)),
                    "median": float(np.median(memories)),
                },
                "latency": {
                    "mean": float(np.mean(latencies)),
                    "std": float(np.std(latencies)),
                    "min": float(np.min(latencies)),
                    "max": float(np.max(latencies)),
                    "median": float(np.median(latencies)),
                },
            }

        # Compare mappy vs standard
        if "mappy" in summary["approaches"] and "standard" in summary["approaches"]:
            mappy_tp = summary["approaches"]["mappy"]["throughput"]["mean"]
            standard_tp = summary["approaches"]["standard"]["throughput"]["mean"]

            mappy_mem = summary["approaches"]["mappy"]["memory"]["mean"]
            standard_mem = summary["approaches"]["standard"]["memory"]["mean"]

            mappy_lat = summary["approaches"]["mappy"]["latency"]["mean"]
            standard_lat = summary["approaches"]["standard"]["latency"]["mean"]

            summary["comparisons"] = {
                "throughput_ratio": float(
                    mappy_tp / standard_tp if standard_tp > 0 else 0,
                ),
                "memory_ratio": float(
                    mappy_mem / standard_mem if standard_mem > 0 else 0,
                ),
                "latency_ratio": float(
                    mappy_lat / standard_lat if standard_lat > 0 else 0,
                ),
                "throughput_improvement_pct": float(
                    ((mappy_tp - standard_tp) / standard_tp * 100)
                    if standard_tp > 0
                    else 0,
                ),
                "memory_savings_pct": float(
                    ((standard_mem - mappy_mem) / standard_mem * 100)
                    if standard_mem > 0
                    else 0,
                ),
                "latency_improvement_pct": float(
                    ((standard_lat - mappy_lat) / standard_lat * 100)
                    if standard_lat > 0
                    else 0,
                ),
            }

            # Statistical significance test (t-test approximation)
            mappy_tp_data = [
                r.throughput_samples_per_sec
                for r in self.results.get("results", {}).get("mappy", [])
            ]
            standard_tp_data = [
                r.throughput_samples_per_sec
                for r in self.results.get("results", {}).get("standard", [])
            ]

            if len(mappy_tp_data) > 1 and len(standard_tp_data) > 1:
                # Simple t-test approximation
                mappy_mean = np.mean(mappy_tp_data)
                standard_mean = np.mean(standard_tp_data)
                mappy_std = np.std(mappy_tp_data)
                standard_std = np.std(standard_tp_data)

                # Pooled standard error
                n1, n2 = len(mappy_tp_data), len(standard_tp_data)
                pooled_se = np.sqrt(
                    (mappy_std**2 / n1) + (standard_std**2 / n2),
                )

                if pooled_se > 0:
                    t_stat = (mappy_mean - standard_mean) / pooled_se
                    # Simplified: if |t| > 2, likely significant (p < 0.05)
                    is_significant = abs(t_stat) > 2.0

                    summary["statistical_tests"] = {
                        "throughput_t_statistic": float(t_stat),
                        "throughput_significant": is_significant,
                        "interpretation": (
                            "Mappy is significantly different from standard"
                            if is_significant
                            else "Difference not statistically significant"
                        ),
                    }

        return summary

    def generate_insights(self) -> list[str]:
        """Generate human-readable insights from results.

        Returns:
            List of insight strings

        """
        insights: list[str] = []

        if "analysis" not in self.results:
            return insights

        analysis = self.results["analysis"]

        # Throughput insights
        if "throughput_comparison" in analysis:
            tp_comp = analysis["throughput_comparison"]
            ratio = tp_comp.get("mappy_vs_standard_ratio", 0)

            if ratio >= 1.0:
                insights.append(
                    f"✓ Mappy is {ratio:.2f}x faster than standard DataLoader "
                    f"({ratio * 100:.1f}% of standard throughput)",
                )
            elif ratio >= 0.9:
                insights.append(
                    f"✓ Mappy performance is competitive: {ratio:.2f}x "
                    f"({ratio * 100:.1f}% of standard throughput)",
                )
            elif ratio >= 0.7:
                insights.append(
                    f"⚠ Mappy is {ratio:.2f}x slower than standard "
                    f"({ratio * 100:.1f}% of standard throughput)",
                )
            else:
                insights.append(
                    f"⚠ Mappy is significantly slower: {ratio:.2f}x "
                    f"({ratio * 100:.1f}% of standard throughput)",
                )

        # Memory insights
        if "memory_comparison" in analysis:
            mem_comp = analysis["memory_comparison"]
            savings = mem_comp.get("memory_savings_pct", 0)

            if savings > 0:
                insights.append(
                    f"✓ Mappy uses {savings:.1f}% less memory than standard DataLoader",
                )
            elif savings < -10:
                insights.append(
                    f"⚠ Mappy uses {abs(savings):.1f}% more memory than standard",
                )

        # Latency insights
        if "latency_comparison" in analysis:
            lat_comp = analysis["latency_comparison"]
            improvement = lat_comp.get("latency_improvement_pct", 0)

            if improvement > 0:
                insights.append(
                    f"✓ Mappy has {improvement:.1f}% lower latency than standard",
                )
            elif improvement < -10:
                insights.append(
                    f"⚠ Mappy has {abs(improvement):.1f}% higher latency than standard",
                )

        # Recommendations
        if "recommendations" in analysis:
            insights.extend(analysis["recommendations"])

        return insights

    def generate_report(
        self, output_path: str | Path | None = None,
    ) -> dict[str, Any]:
        """Generate comprehensive analysis report.

        Args:
            output_path: Optional path to save report

        Returns:
            Report dictionary

        """
        statistical_summary = self.generate_statistical_summary()
        insights = self.generate_insights()

        report = {
            "config": self.results.get("config", {}),
            "statistical_summary": statistical_summary,
            "insights": insights,
            "raw_analysis": self.analysis,
        }

        if output_path:
            output_path = Path(output_path)
            output_path.parent.mkdir(parents=True, exist_ok=True)

            with open(output_path, "w") as f:
                json.dump(report, f, indent=2, default=str)

        return report

    def create_visualizations(
        self, output_dir: str | Path = "reports",
    ) -> dict[str, str]:
        """Create visualization charts.

        Args:
            output_dir: Output directory for charts

        Returns:
            Dictionary of chart file paths

        """
        if not PLOTLY_AVAILABLE:
            return {}

        output_dir = Path(output_dir)
        output_dir.mkdir(parents=True, exist_ok=True)

        charts: dict[str, str] = {}

        # Throughput comparison chart
        if "summary" in self.analysis:
            summary = self.analysis["summary"]
            approaches = list(summary.keys())
            throughputs = [
                summary[app]["avg_throughput_samples_per_sec"] for app in approaches
            ]

            fig = go.Figure()
            fig.add_trace(
                go.Bar(
                    x=approaches,
                    y=throughputs,
                    name="Throughput",
                    marker_color="blue",
                ),
            )
            fig.update_layout(
                title="Throughput Comparison (samples/sec)",
                xaxis_title="Approach",
                yaxis_title="Samples per Second",
                height=400,
            )

            chart_path = output_dir / "throughput_comparison.html"
            fig.write_html(str(chart_path))
            charts["throughput"] = str(chart_path)

            # Memory comparison
            memories = [summary[app]["avg_peak_memory_mb"] for app in approaches]

            fig = go.Figure()
            fig.add_trace(
                go.Bar(
                    x=approaches,
                    y=memories,
                    name="Peak Memory",
                    marker_color="orange",
                ),
            )
            fig.update_layout(
                title="Memory Usage Comparison (MB)",
                xaxis_title="Approach",
                yaxis_title="Peak Memory (MB)",
                height=400,
            )

            chart_path = output_dir / "memory_comparison.html"
            fig.write_html(str(chart_path))
            charts["memory"] = str(chart_path)

        return charts


def analyze_benchmark_results(
    results: dict[str, Any], output_dir: str | Path = "reports",
) -> dict[str, Any]:
    """Analyze benchmark results and generate report.

    Args:
        results: Benchmark results dictionary
        output_dir: Output directory for reports

    Returns:
        Analysis report

    """
    analyzer = BenchmarkAnalyzer(results)
    report = analyzer.generate_report(output_dir / "benchmark_analysis.json")
    charts = analyzer.create_visualizations(output_dir)

    return {
        "report": report,
        "charts": charts,
        "insights": analyzer.generate_insights(),
    }

