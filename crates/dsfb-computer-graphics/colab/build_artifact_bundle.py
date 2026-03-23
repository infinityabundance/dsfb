#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import math
import os
import subprocess
import textwrap
import zipfile
from pathlib import Path
from tempfile import TemporaryDirectory

from PIL import Image, ImageDraw, ImageFont


EXPERIMENT_SENTENCE = (
    "“The experiment is intended to demonstrate behavioral differences rather than "
    "establish optimal performance.”"
)
PDF_FILE_NAME = "artifacts_bundle.pdf"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Build a PDF and ZIP reviewer bundle for a dsfb-computer-graphics run directory."
    )
    parser.add_argument("--run-dir", required=True, help="Timestamped run directory to bundle.")
    return parser.parse_args()


def load_json(path: Path) -> dict:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


def resolve_manifest(run_dir: Path) -> dict:
    manifest_path = run_dir / "artifact_manifest.json"
    if not manifest_path.exists():
        raise FileNotFoundError(
            f"artifact manifest was not found at {manifest_path}; run `cargo run -- run-all` first"
        )
    return load_json(manifest_path)


def require_paths(run_dir: Path, relative_paths: list[str]) -> list[Path]:
    resolved = []
    missing = []
    for relative in relative_paths:
        path = run_dir / relative
        if not path.exists():
            missing.append(str(path))
        else:
            resolved.append(path)
    if missing:
        raise FileNotFoundError("missing required artifact(s):\n" + "\n".join(missing))
    return resolved


def font(size: int, bold: bool = False) -> ImageFont.FreeTypeFont | ImageFont.ImageFont:
    candidates = []
    if bold:
        candidates.extend(
            [
                "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
                "/usr/share/fonts/dejavu/DejaVuSans-Bold.ttf",
            ]
        )
    candidates.extend(
        [
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/dejavu/DejaVuSans.ttf",
        ]
    )
    for candidate in candidates:
        if os.path.exists(candidate):
            return ImageFont.truetype(candidate, size=size)
    return ImageFont.load_default()


class PageBuilder:
    def __init__(self, title: str):
        self.image = Image.new("RGB", (1500, 2000), "white")
        self.draw = ImageDraw.Draw(self.image)
        self.margin = 90
        self.cursor_y = 90
        self.title_font = font(44, bold=True)
        self.heading_font = font(30, bold=True)
        self.body_font = font(22)
        self.small_font = font(18)
        self.draw.text((self.margin, self.cursor_y), title, fill="black", font=self.title_font)
        self.cursor_y += 90

    def paragraph(self, text: str, *, indent: int = 0, spacing: int = 18) -> None:
        max_width = self.image.width - self.margin * 2 - indent
        wrapped = wrap_text(text, self.body_font, max_width)
        for line in wrapped:
            self.draw.text(
                (self.margin + indent, self.cursor_y), line, fill="black", font=self.body_font
            )
            self.cursor_y += line_height(self.body_font) + 6
        self.cursor_y += spacing

    def small_paragraph(self, text: str, *, indent: int = 0, spacing: int = 14) -> None:
        max_width = self.image.width - self.margin * 2 - indent
        wrapped = wrap_text(text, self.small_font, max_width)
        for line in wrapped:
            self.draw.text(
                (self.margin + indent, self.cursor_y), line, fill="black", font=self.small_font
            )
            self.cursor_y += line_height(self.small_font) + 4
        self.cursor_y += spacing

    def heading(self, text: str) -> None:
        self.draw.text((self.margin, self.cursor_y), text, fill="black", font=self.heading_font)
        self.cursor_y += 52

    def table(self, headers: list[str], rows: list[list[str]]) -> None:
        column_widths = [240, 320, 320, 220]
        row_height = 44
        x = self.margin
        y = self.cursor_y
        self.draw.rectangle(
            [x, y, x + sum(column_widths), y + row_height], outline="black", width=2, fill="#eef3f7"
        )
        current_x = x
        for header, width in zip(headers, column_widths):
            self.draw.line([(current_x, y), (current_x, y + row_height)], fill="black", width=2)
            self.draw.text((current_x + 10, y + 9), header, fill="black", font=self.small_font)
            current_x += width
        self.draw.line(
            [(x + sum(column_widths), y), (x + sum(column_widths), y + row_height)],
            fill="black",
            width=2,
        )
        y += row_height
        for row in rows:
            self.draw.rectangle(
                [x, y, x + sum(column_widths), y + row_height], outline="black", width=1
            )
            current_x = x
            for value, width in zip(row, column_widths):
                self.draw.line(
                    [(current_x, y), (current_x, y + row_height)], fill="black", width=1
                )
                self.draw.text((current_x + 10, y + 9), value, fill="black", font=self.small_font)
                current_x += width
            self.draw.line(
                [(x + sum(column_widths), y), (x + sum(column_widths), y + row_height)],
                fill="black",
                width=1,
            )
            y += row_height
        self.cursor_y = y + 24

    def image_with_caption(self, title: str, image_path: Path, caption: str) -> None:
        self.heading(title)
        image = Image.open(image_path).convert("RGB")
        available_width = self.image.width - self.margin * 2
        available_height = 980
        image.thumbnail((available_width, available_height), Image.Resampling.LANCZOS)
        x = (self.image.width - image.width) // 2
        self.image.paste(image, (x, self.cursor_y))
        self.cursor_y += image.height + 24
        self.small_paragraph(caption)


def line_height(active_font: ImageFont.ImageFont) -> int:
    bbox = active_font.getbbox("Ag")
    return bbox[3] - bbox[1]


def wrap_text(text: str, active_font: ImageFont.ImageFont, max_width: int) -> list[str]:
    words = text.split()
    if not words:
        return [""]
    lines: list[str] = []
    current = words[0]
    for word in words[1:]:
        candidate = f"{current} {word}"
        if active_font.getlength(candidate) <= max_width:
            current = candidate
        else:
            lines.append(current)
            current = word
    lines.append(current)
    return lines


def svg_to_png(svg_path: Path, png_path: Path) -> None:
    command = ["rsvg-convert", "-o", str(png_path), str(svg_path)]
    try:
        subprocess.run(command, check=True, capture_output=True)
    except FileNotFoundError as exc:
        raise RuntimeError(
            "rsvg-convert is required for PDF bundling; install `librsvg2-bin` before running"
        ) from exc
    except subprocess.CalledProcessError as exc:
        raise RuntimeError(
            f"failed to rasterize {svg_path} with rsvg-convert:\n{exc.stderr.decode('utf-8', errors='ignore')}"
        ) from exc


def format_metric(value: float) -> str:
    if math.isfinite(value):
        return f"{value:.5f}"
    return str(value)


def find_run(scenario: dict, run_id: str) -> dict:
    return next(run["summary"] for run in scenario["runs"] if run["summary"]["run_id"] == run_id)


def find_policy(scenario: dict, policy_id: str) -> dict:
    return next(policy for policy in scenario["policies"] if policy["policy_id"] == policy_id)


def canonical_demo_a_scenario(metrics: dict) -> dict:
    return next(
        (scenario for scenario in metrics["scenarios"] if scenario["scenario_id"] == "thin_reveal"),
        metrics["scenarios"][0],
    )


def canonical_demo_b_scenario(metrics: dict) -> dict:
    return next(
        (scenario for scenario in metrics["scenarios"] if scenario["scenario_id"] == "thin_reveal"),
        metrics["scenarios"][0],
    )


def build_pdf(run_dir: Path, manifest: dict, demo_a_metrics: dict, demo_b_metrics: dict | None) -> Path:
    pdf_path = run_dir / PDF_FILE_NAME
    demo_a_paths = manifest["demo_a"]
    figure_paths = require_paths(run_dir, demo_a_paths["figure_paths"])
    demo_b_figures = require_paths(run_dir, manifest["demo_b"]["figure_paths"])
    canonical_a = canonical_demo_a_scenario(demo_a_metrics)
    fixed = find_run(canonical_a, "fixed_alpha")
    strong = find_run(canonical_a, "strong_heuristic")
    host = find_run(canonical_a, "dsfb_host_realistic")

    with TemporaryDirectory(dir=run_dir) as temp_dir_name:
        temp_dir = Path(temp_dir_name)
        rasterized_figures = []
        for figure_path in figure_paths + demo_b_figures:
            raster_path = temp_dir / f"{figure_path.stem}.png"
            svg_to_png(figure_path, raster_path)
            rasterized_figures.append(raster_path)

        pages: list[Image.Image] = []

        title_page = PageBuilder("DSFB Computer Graphics Reviewer Bundle")
        title_page.paragraph(
            "This PDF is generated from a single timestamped run directory so reviewers can archive the bounded synthetic artifact without depending on the notebook session state."
        )
        title_page.paragraph(EXPERIMENT_SENTENCE)
        title_page.heading("What This Bundle Contains")
        title_page.paragraph(
            "Demo A figures, key numeric metrics, an interpretation grounded in the actual run outputs, and the main limitations of the artifact."
        )
        title_page.paragraph(
            "The bundle is intended to support technical review, transition conversations, and reproducible replay rather than optimality claims or deployment claims."
        )
        title_page.small_paragraph(f"Run directory: {run_dir.name}")
        pages.append(title_page.image)

        metrics_page = PageBuilder("Key Metrics")
        metrics_page.paragraph(demo_a_metrics["summary"]["primary_behavioral_result"])
        secondary = demo_a_metrics["summary"].get("secondary_behavioral_result")
        if secondary:
            metrics_page.paragraph(secondary)
        metrics_page.table(
            ["Metric", "Fixed-alpha", "Strong heuristic", "Host-realistic"],
            [
                [
                    "Ghost persistence frames",
                    str(fixed["ghost_persistence_frames"]),
                    str(strong["ghost_persistence_frames"]),
                    str(host["ghost_persistence_frames"]),
                ],
                [
                    "Peak ROI error",
                    format_metric(fixed["peak_roi_mae"]),
                    format_metric(strong["peak_roi_mae"]),
                    format_metric(host["peak_roi_mae"]),
                ],
                [
                    "Cumulative ROI error",
                    format_metric(fixed["cumulative_roi_mae"]),
                    format_metric(strong["cumulative_roi_mae"]),
                    format_metric(host["cumulative_roi_mae"]),
                ],
                [
                    "Average non-ROI MAE",
                    format_metric(fixed["average_non_roi_mae"]),
                    format_metric(strong["average_non_roi_mae"]),
                    format_metric(host["average_non_roi_mae"]),
                ],
            ],
        )
        metrics_page.heading("Event Timing")
        metrics_page.small_paragraph(f"Canonical onset frame: {canonical_a['onset_frame']}")
        metrics_page.small_paragraph(
            "Host-realistic onset response latency: "
            f"{host['onset_response_latency_frames']}"
        )
        metrics_page.small_paragraph(
            "Strong-heuristic onset response latency: "
            f"{strong['onset_response_latency_frames']}"
        )
        metrics_page.small_paragraph(
            "Host-realistic trust/error rank correlation: "
            f"{host['trust_error_rank_correlation']:.4f}"
        )
        pages.append(metrics_page.image)

        selected_demo_a_indices = [0, 1, 2, 3, 4, 8]
        captions = [
            (
                "Figure 1. System Diagram",
                "Inputs, residuals, proxies, simplified grammar, trust, and intervention as used in the current crate.",
            ),
            (
                "Figure 2. Trust Map",
                "Low trust concentrates around the reveal event and motion-edge supervision region on the actual demo frame.",
            ),
            (
                "Figure 3. Before / After",
                "Fixed alpha, strong heuristic, and host-realistic DSFB on the same canonical comparison frame and persistence ROI.",
            ),
            (
                "Figure 4. Trust vs Error",
                "Frame index on the x-axis, ROI error on the left y-axis, and DSFB ROI trust on the right y-axis.",
            ),
            (
                "Figure 5. Intervention and Alpha",
                "The intervention map and alpha field show where host-realistic DSFB actively suppresses stale history.",
            ),
            (
                "Figure 6. Aggregate Scenario View",
                "The expanded scenario suite makes the effect distribution and mixed outcomes visible instead of relying on one favorable case.",
            ),
        ]
        selected_demo_a_figures = [rasterized_figures[index] for index in selected_demo_a_indices if index < len(rasterized_figures)]
        for raster_path, (title, caption) in zip(selected_demo_a_figures, captions):
            page = PageBuilder(title)
            page.image_with_caption(title, raster_path, caption)
            pages.append(page.image)

        if demo_b_metrics is not None and len(rasterized_figures) > 4:
            canonical_b = canonical_demo_b_scenario(demo_b_metrics)
            uniform = find_policy(canonical_b, "uniform")
            combined = find_policy(canonical_b, "combined_heuristic")
            imported = find_policy(canonical_b, "imported_trust")
            demo_b_page = PageBuilder("Demo B. Fixed-Budget Adaptive Sampling")
            demo_b_page.image_with_caption(
                "Demo B. Fixed-Budget Adaptive Sampling",
                rasterized_figures[-2],
                (
                    "At equal total sample budget, the imported-trust policy concentrates work in low-trust regions while remaining comparable to or better than cheap heuristic reallocators on the hardest cases. "
                    f"Uniform ROI MAE is {uniform['roi_mae']:.5f}; combined-heuristic ROI MAE is {combined['roi_mae']:.5f}; imported-trust ROI MAE is {imported['roi_mae']:.5f}."
                ),
            )
            pages.append(demo_b_page.image)
            if len(rasterized_figures) >= 8:
                efficiency_page = PageBuilder("Demo B. Budget Efficiency")
                efficiency_page.image_with_caption(
                    "Demo B. Budget Efficiency",
                    rasterized_figures[-1],
                    "The budget-efficiency curve shows ROI MAE against mean spp for uniform, combined-heuristic, imported-trust, and hybrid trust-plus-variance allocation on the canonical sampling case.",
                )
                pages.append(efficiency_page.image)

        limits_page = PageBuilder("Limitations and Scope")
        limits_page.paragraph(
            "This remains a bounded synthetic artifact. It does not claim production-optimal TAA, measured GPU timing wins, field readiness, or universal superiority over commercial reconstruction systems."
        )
        limits_page.paragraph(
            "PDF and ZIP export exist to make external review easier: a reviewer can archive the exact bundle from one run directory and inspect the same figures and metrics later without rerunning the notebook."
        )
        limits_page.paragraph(EXPERIMENT_SENTENCE)
        pages.append(limits_page.image)

        pages[0].save(
            pdf_path,
            "PDF",
            resolution=150.0,
            save_all=True,
            append_images=pages[1:],
        )

    return pdf_path


def build_zip(run_dir: Path, zip_path: Path) -> Path:
    if zip_path.exists():
        zip_path.unlink()

    with zipfile.ZipFile(zip_path, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for path in sorted(run_dir.rglob("*")):
            if path.is_file():
                archive.write(path, path.relative_to(run_dir.parent))
    return zip_path


def main() -> None:
    args = parse_args()
    run_dir = Path(args.run_dir).resolve()
    if not run_dir.exists():
        raise FileNotFoundError(f"run directory {run_dir} does not exist")

    manifest = resolve_manifest(run_dir)
    metrics = load_json(run_dir / manifest["demo_a"]["metrics_path"])
    demo_b_metrics_path = run_dir / manifest["demo_b"]["metrics_path"]
    demo_b_metrics = load_json(demo_b_metrics_path) if demo_b_metrics_path.exists() else None

    pdf_path = build_pdf(run_dir, manifest, metrics, demo_b_metrics)
    zip_path = build_zip(run_dir, run_dir.parent / manifest["zip_bundle_file_name"])

    print(
        json.dumps(
            {
                "run_dir": str(run_dir),
                "pdf_path": str(pdf_path),
                "zip_path": str(zip_path),
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
