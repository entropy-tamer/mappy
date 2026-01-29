"""Report generation for evaluation results.

Generates HTML and JSON reports with visualizations.
"""

import json
from datetime import datetime
from pathlib import Path
from typing import Any

import plotly.graph_objects as go
from plotly.subplots import make_subplots

from .metrics import EvaluationResults


class ReportGenerator:
    """Generates comprehensive evaluation reports."""

    def __init__(self, output_dir: str = "reports"):
        """Initialize report generator.

        Args:
            output_dir: Output directory for reports

        """
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(parents=True, exist_ok=True)

    def generate_html_report(
        self,
        results: list[EvaluationResults],
        comparison: dict[str, Any] | None = None,
        filename: str | None = None,
    ) -> str:
        """Generate comprehensive HTML report.

        Args:
            results: List of evaluation results
            comparison: Optional comparison data (mappy vs baseline)
            filename: Optional output filename

        Returns:
            Path to generated report

        """
        if filename is None:
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            filename = f"mappy_evaluation_report_{timestamp}.html"

        filepath = self.output_dir / filename

        # Generate visualizations
        performance_fig = self._create_performance_chart(results)
        accuracy_fig = self._create_accuracy_chart(results)
        memory_fig = self._create_memory_chart(results)

        comparison_fig = None
        if comparison:
            comparison_fig = self._create_comparison_chart(comparison)

        # Generate HTML
        html_content = self._generate_html_content(
            results,
            performance_fig,
            accuracy_fig,
            memory_fig,
            comparison_fig,
            comparison,
        )

        # Write to file
        with open(filepath, "w") as f:
            f.write(html_content)

        return str(filepath)

    def generate_json_report(
        self,
        results: list[EvaluationResults],
        comparison: dict[str, Any] | None = None,
        filename: str | None = None,
    ) -> str:
        """Generate JSON report.

        Args:
            results: List of evaluation results
            comparison: Optional comparison data
            filename: Optional output filename

        Returns:
            Path to generated report

        """
        if filename is None:
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            filename = f"mappy_evaluation_report_{timestamp}.json"

        filepath = self.output_dir / filename

        # Serialize results
        report_data = {
            "timestamp": datetime.now().isoformat(),
            "results": [self._serialize_result(r) for r in results],
            "comparison": comparison,
        }

        # Write to file
        with open(filepath, "w") as f:
            json.dump(report_data, f, indent=2, default=str)

        return str(filepath)

    def _serialize_result(self, result: EvaluationResults) -> dict[str, Any]:
        """Serialize EvaluationResults to dictionary.

        Args:
            result: EvaluationResults object

        Returns:
            Dictionary representation

        """
        return {
            "task_name": result.task_name,
            "timestamp": result.timestamp,
            "config": result.config,
            "performance": {
                "samples_per_second": result.performance.samples_per_second,
                "batches_per_second": result.performance.batches_per_second,
                "total_samples": result.performance.total_samples,
                "total_batches": result.performance.total_batches,
                "first_batch_time_ms": result.performance.first_batch_time_ms,
                "avg_batch_time_ms": result.performance.avg_batch_time_ms,
                "min_batch_time_ms": result.performance.min_batch_time_ms,
                "max_batch_time_ms": result.performance.max_batch_time_ms,
                "peak_memory_mb": result.performance.peak_memory_mb,
                "avg_memory_mb": result.performance.avg_memory_mb,
                "memory_per_sample_mb": result.performance.memory_per_sample_mb,
                "gpu_utilization_pct": result.performance.gpu_utilization_pct,
                "gpu_memory_mb": result.performance.gpu_memory_mb,
                "total_time_seconds": result.performance.total_time_seconds,
            },
            "accuracy": {
                "task_name": result.accuracy.task_name,
                "accuracy": result.accuracy.accuracy,
                "precision": result.accuracy.precision,
                "recall": result.accuracy.recall,
                "f1_score": result.accuracy.f1_score,
                "perplexity": result.accuracy.perplexity,
                "bleu_score": result.accuracy.bleu_score,
                "meteor_score": result.accuracy.meteor_score,
                "cider_score": result.accuracy.cider_score,
                "fid_score": result.accuracy.fid_score,
                "rmse": result.accuracy.rmse,
                "precision_at_k": result.accuracy.precision_at_k,
                "recall_at_k": result.accuracy.recall_at_k,
            },
        }

    def _create_performance_chart(self, results: list[EvaluationResults]) -> go.Figure:
        """Create performance comparison chart.

        Args:
            results: List of evaluation results

        Returns:
            Plotly figure

        """
        tasks = [r.task_name for r in results]
        throughput = [r.performance.samples_per_second for r in results]
        latency = [r.performance.avg_batch_time_ms for r in results]

        fig = make_subplots(
            rows=2,
            cols=1,
            subplot_titles=("Throughput (samples/sec)", "Average Batch Latency (ms)"),
            vertical_spacing=0.15,
        )

        fig.add_trace(
            go.Bar(x=tasks, y=throughput, name="Throughput", marker_color="blue"),
            row=1,
            col=1,
        )

        fig.add_trace(
            go.Bar(x=tasks, y=latency, name="Latency", marker_color="red"),
            row=2,
            col=1,
        )

        fig.update_xaxes(title_text="Task", row=2, col=1)
        fig.update_yaxes(title_text="Samples/sec", row=1, col=1)
        fig.update_yaxes(title_text="Time (ms)", row=2, col=1)
        fig.update_layout(height=600, showlegend=False, title_text="Performance Metrics")

        return fig

    def _create_accuracy_chart(self, results: list[EvaluationResults]) -> go.Figure:
        """Create accuracy comparison chart.

        Args:
            results: List of evaluation results

        Returns:
            Plotly figure

        """
        tasks = [r.task_name for r in results]
        accuracy = [r.accuracy.accuracy for r in results]

        fig = go.Figure()
        fig.add_trace(
            go.Bar(
                x=tasks,
                y=accuracy,
                name="Accuracy",
                marker_color="green",
                text=[f"{a:.3f}" for a in accuracy],
                textposition="outside",
            ),
        )

        fig.update_layout(
            title="Accuracy by Task",
            xaxis_title="Task",
            yaxis_title="Accuracy",
            height=400,
        )

        return fig

    def _create_memory_chart(self, results: list[EvaluationResults]) -> go.Figure:
        """Create memory usage chart.

        Args:
            results: List of evaluation results

        Returns:
            Plotly figure

        """
        tasks = [r.task_name for r in results]
        peak_memory = [r.performance.peak_memory_mb for r in results]
        avg_memory = [r.performance.avg_memory_mb for r in results]

        fig = go.Figure()
        fig.add_trace(
            go.Bar(x=tasks, y=peak_memory, name="Peak Memory", marker_color="orange"),
        )
        fig.add_trace(
            go.Bar(x=tasks, y=avg_memory, name="Average Memory", marker_color="purple"),
        )

        fig.update_layout(
            title="Memory Usage by Task",
            xaxis_title="Task",
            yaxis_title="Memory (MB)",
            barmode="group",
            height=400,
        )

        return fig

    def _create_comparison_chart(self, comparison: dict[str, Any]) -> go.Figure:
        """Create mappy vs baseline comparison chart.

        Args:
            comparison: Comparison dictionary

        Returns:
            Plotly figure

        """
        tasks = comparison["tasks"]
        accuracy_diffs = comparison["accuracy_differences"]
        speed_ratios = comparison["speed_ratios"]
        memory_ratios = comparison["memory_ratios"]

        fig = make_subplots(
            rows=3,
            cols=1,
            subplot_titles=(
                "Accuracy Difference (Mappy vs Baseline)",
                "Speed Ratio (Mappy / Baseline)",
                "Memory Ratio (Mappy / Baseline)",
            ),
            vertical_spacing=0.15,
        )

        fig.add_trace(
            go.Bar(x=tasks, y=accuracy_diffs, name="Accuracy Diff", marker_color="blue"),
            row=1,
            col=1,
        )

        fig.add_trace(
            go.Bar(x=tasks, y=speed_ratios, name="Speed Ratio", marker_color="green"),
            row=2,
            col=1,
        )

        fig.add_trace(
            go.Bar(x=tasks, y=memory_ratios, name="Memory Ratio", marker_color="red"),
            row=3,
            col=1,
        )

        fig.update_xaxes(title_text="Task", row=3, col=1)
        fig.update_yaxes(title_text="Difference", row=1, col=1)
        fig.update_yaxes(title_text="Ratio", row=2, col=1)
        fig.update_yaxes(title_text="Ratio", row=3, col=1)
        fig.update_layout(height=800, showlegend=False, title_text="Mappy vs Baseline Comparison")

        return fig

    def _generate_html_content(
        self,
        results: list[EvaluationResults],
        performance_fig: go.Figure,
        accuracy_fig: go.Figure,
        memory_fig: go.Figure,
        comparison_fig: go.Figure | None,
        comparison: dict[str, Any] | None,
    ) -> str:
        """Generate HTML content.

        Args:
            results: List of results
            performance_fig: Performance chart
            accuracy_fig: Accuracy chart
            memory_fig: Memory chart
            comparison_fig: Optional comparison chart
            comparison: Optional comparison data

        Returns:
            HTML string

        """
        html = f"""
<!DOCTYPE html>
<html>
<head>
    <title>Mappy DataLoader Evaluation Report</title>
    <script src="https://cdn.plot.ly/plotly-latest.min.js"></script>
    <style>
        body {{
            font-family: Arial, sans-serif;
            margin: 20px;
            background-color: #f5f5f5;
        }}
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            background-color: white;
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        h1 {{
            color: #333;
            border-bottom: 3px solid #4CAF50;
            padding-bottom: 10px;
        }}
        h2 {{
            color: #555;
            margin-top: 30px;
        }}
        .summary {{
            background-color: #f9f9f9;
            padding: 15px;
            border-radius: 5px;
            margin: 20px 0;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin: 20px 0;
        }}
        th, td {{
            padding: 12px;
            text-align: left;
            border-bottom: 1px solid #ddd;
        }}
        th {{
            background-color: #4CAF50;
            color: white;
        }}
        tr:hover {{
            background-color: #f5f5f5;
        }}
        .chart-container {{
            margin: 30px 0;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>Mappy PyTorch DataLoader Evaluation Report</h1>
        <p><strong>Generated:</strong> {datetime.now().strftime("%Y-%m-%d %H:%M:%S")}</p>
        <p><strong>Number of Tasks:</strong> {len(results)}</p>

        <div class="summary">
            <h2>Summary</h2>
            <p>This report presents comprehensive evaluation results for mappy as a PyTorch DataLoader
            across multiple ML training tasks. Metrics include performance (throughput, latency),
            memory usage, GPU utilization, and task-specific accuracy metrics.</p>
        </div>

        <h2>Performance Metrics</h2>
        <div class="chart-container" id="performance-chart"></div>

        <h2>Accuracy Metrics</h2>
        <div class="chart-container" id="accuracy-chart"></div>

        <h2>Memory Usage</h2>
        <div class="chart-container" id="memory-chart"></div>
"""

        if comparison_fig:
            html += """
        <h2>Mappy vs Baseline Comparison</h2>
        <div class="chart-container" id="comparison-chart"></div>
"""

        html += """
        <h2>Detailed Results</h2>
        <table>
            <tr>
                <th>Task</th>
                <th>Accuracy</th>
                <th>Throughput (samples/sec)</th>
                <th>Avg Batch Time (ms)</th>
                <th>Peak Memory (MB)</th>
                <th>Total Time (s)</th>
            </tr>
"""

        for result in results:
            html += f"""
            <tr>
                <td>{result.task_name}</td>
                <td>{result.accuracy.accuracy:.4f}</td>
                <td>{result.performance.samples_per_second:.2f}</td>
                <td>{result.performance.avg_batch_time_ms:.2f}</td>
                <td>{result.performance.peak_memory_mb:.2f}</td>
                <td>{result.performance.total_time_seconds:.2f}</td>
            </tr>
"""

        html += """
        </table>
    </div>

    <script>
"""

        # Add chart JavaScript
        html += f"var performanceChart = {performance_fig.to_json()};"
        html += "Plotly.newPlot('performance-chart', performanceChart.data, performanceChart.layout);"

        html += f"var accuracyChart = {accuracy_fig.to_json()};"
        html += "Plotly.newPlot('accuracy-chart', accuracyChart.data, accuracyChart.layout);"

        html += f"var memoryChart = {memory_fig.to_json()};"
        html += "Plotly.newPlot('memory-chart', memoryChart.data, memoryChart.layout);"

        if comparison_fig:
            html += f"var comparisonChart = {comparison_fig.to_json()};"
            html += "Plotly.newPlot('comparison-chart', comparisonChart.data, comparisonChart.layout);"

        html += """
    </script>
</body>
</html>
"""

        return html


def generate_report(
    results: list[EvaluationResults],
    comparison: dict[str, Any] | None = None,
    output_dir: str = "reports",
    format: str = "both",  # "html", "json", or "both"
) -> dict[str, str]:
    """Convenience function to generate reports.

    Args:
        results: List of evaluation results
        comparison: Optional comparison data
        output_dir: Output directory
        format: Report format ("html", "json", or "both")

    Returns:
        Dictionary with report file paths

    """
    generator = ReportGenerator(output_dir=output_dir)

    report_paths = {}

    if format in ["html", "both"]:
        html_path = generator.generate_html_report(results, comparison)
        report_paths["html"] = html_path

    if format in ["json", "both"]:
        json_path = generator.generate_json_report(results, comparison)
        report_paths["json"] = json_path

    return report_paths

