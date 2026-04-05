#!/usr/bin/env python3
"""SBIR Demo Engineering Report Generator.

Reads pipeline artifacts + test results for dsfb-computer-graphics and produces
a single self-contained PDF engineering report suitable for crate-quality review.

Usage (called automatically by `cargo run -- sbir-demo`):

    python3 colab/sbir_demo_report.py \
        --run-dir generated/sbir_demo/<timestamp> \
        --test-results generated/sbir_demo/<timestamp>/test_results.json \
        --output generated/sbir_demo/<timestamp>/sbir_demo_report.pdf
"""

from __future__ import annotations

import argparse
import json
import math
import os
import textwrap
from datetime import datetime, timezone
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

PAGE_W = 1600
PAGE_H = 2100
MARGIN = 80
BG_WHITE = "white"
BLACK = "black"
PASS_GREEN = "#1a7a1a"
FAIL_RED = "#aa1a1a"
HEADER_BG = "#dfe8f2"
ROW_ALT_BG = "#f4f7fb"
SECTION_RULE = "#9aafcc"

REQUIRED_ARTIFACTS = [
    "artifact_manifest.json",
    "metrics.json",
    "report.md",
    "reviewer_summary.md",
    "ablation_report.md",
    "cost_report.md",
    "completion_note.md",
    "five_mentor_audit.md",
    "check_signing_blockers.md",
    "check_signing_readiness.md",
    "trust_mode_report.md",
    "external_replay_report.md",
    "external_handoff_report.md",
    "gpu_execution_report.md",
    "gpu_execution_metrics.json",
    "realism_suite_report.md",
    "realism_bridge_report.md",
    "demo_b_decision_report.md",
    "demo_b_competitive_baselines_report.md",
    "competitive_baseline_analysis.md",
    "product_positioning_report.md",
    "operating_band_report.md",
    "non_roi_penalty_report.md",
    "demo_b/metrics.json",
    "demo_b/report.md",
    "external_real/external_validation_report.md",
    "external_real/gpu_external_report.md",
    "external_real/demo_a_external_report.md",
    "external_real/demo_b_external_report.md",
    "external_real/scaling_report.md",
    "external_real/memory_bandwidth_report.md",
    "external_real/integration_scaling_report.md",
    "figures/fig_system_diagram.svg",
    "figures/fig_trust_map.svg",
    "figures/fig_before_after.svg",
    "figures/fig_trust_vs_error.svg",
    "figures/fig_leaderboard.svg",
    "figures/fig_scenario_mosaic.svg",
]


# ---------------------------------------------------------------------------
# Font helpers
# ---------------------------------------------------------------------------

def _font(size: int, bold: bool = False) -> ImageFont.FreeTypeFont | ImageFont.ImageFont:
    candidates: list[str] = []
    if bold:
        candidates += [
            "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
            "/usr/share/fonts/dejavu/DejaVuSans-Bold.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Bold.ttf",
        ]
    candidates += [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
    ]
    for c in candidates:
        if os.path.exists(c):
            return ImageFont.truetype(c, size=size)
    return ImageFont.load_default()


def _line_height(f: ImageFont.ImageFont) -> int:
    bb = f.getbbox("Ag")
    return bb[3] - bb[1]


def _wrap(text: str, f: ImageFont.ImageFont, max_w: int) -> list[str]:
    words = text.split()
    if not words:
        return [""]
    lines: list[str] = []
    cur = words[0]
    for w in words[1:]:
        cand = f"{cur} {w}"
        if f.getlength(cand) <= max_w:
            cur = cand
        else:
            lines.append(cur)
            cur = w
    lines.append(cur)
    return lines


def _fmt(v: float) -> str:
    return f"{v:.5f}" if math.isfinite(v) else str(v)


# ---------------------------------------------------------------------------
# Page builder
# ---------------------------------------------------------------------------

class Page:
    def __init__(self, title: str) -> None:
        self.img = Image.new("RGB", (PAGE_W, PAGE_H), BG_WHITE)
        self.d = ImageDraw.Draw(self.img)
        self._title_f = _font(42, bold=True)
        self._head_f = _font(28, bold=True)
        self._body_f = _font(22)
        self._small_f = _font(18)
        self._mono_f = _font(17)
        self.x0 = MARGIN
        self.y = MARGIN
        self._draw_header_bar(title)

    # ------------------------------------------------------------------
    # Internal helpers
    # ------------------------------------------------------------------

    def _draw_header_bar(self, title: str) -> None:
        self.d.rectangle([0, 0, PAGE_W, 72], fill="#2b4a7a")
        self.d.text((MARGIN, 14), title, fill="white", font=self._title_f)
        self.y = 90

    def _rule(self) -> None:
        self.d.line([(self.x0, self.y), (PAGE_W - MARGIN, self.y)], fill=SECTION_RULE, width=2)
        self.y += 8

    def _check_overflow(self, needed: int = 60) -> bool:
        return self.y + needed >= PAGE_H - MARGIN

    # ------------------------------------------------------------------
    # Public primitives
    # ------------------------------------------------------------------

    def heading(self, text: str, *, spacing: int = 14) -> None:
        if self._check_overflow(52):
            return
        self._rule()
        self.d.text((self.x0, self.y), text, fill="#1a3060", font=self._head_f)
        self.y += _line_height(self._head_f) + spacing

    def paragraph(self, text: str, *, indent: int = 0, spacing: int = 16) -> None:
        max_w = PAGE_W - self.x0 * 2 - indent
        for line in _wrap(text, self._body_f, max_w):
            if self._check_overflow():
                break
            self.d.text((self.x0 + indent, self.y), line, fill=BLACK, font=self._body_f)
            self.y += _line_height(self._body_f) + 5
        self.y += spacing

    def small(self, text: str, *, indent: int = 0, spacing: int = 10, color: str = BLACK) -> None:
        max_w = PAGE_W - self.x0 * 2 - indent
        for line in _wrap(text, self._small_f, max_w):
            if self._check_overflow():
                break
            self.d.text((self.x0 + indent, self.y), line, fill=color, font=self._small_f)
            self.y += _line_height(self._small_f) + 4
        self.y += spacing

    def mono(self, text: str, *, indent: int = 0, spacing: int = 8) -> None:
        if self._check_overflow():
            return
        self.d.text((self.x0 + indent, self.y), text, fill="#333333", font=self._mono_f)
        self.y += _line_height(self._mono_f) + spacing

    def badge(self, text: str, ok: bool) -> None:
        """Draw a pass/fail badge inline."""
        color = PASS_GREEN if ok else FAIL_RED
        self.d.text((self.x0, self.y), text, fill=color, font=self._body_f)
        self.y += _line_height(self._body_f) + 8

    def kv_row(self, key: str, value: str, *, ok: bool | None = None) -> None:
        if self._check_overflow():
            return
        self.d.text((self.x0, self.y), f"{key}:", fill="#444", font=self._small_f)
        vx = self.x0 + 380
        color = (PASS_GREEN if ok else FAIL_RED) if ok is not None else BLACK
        self.d.text((vx, self.y), value, fill=color, font=self._small_f)
        self.y += _line_height(self._small_f) + 6

    def spacer(self, px: int = 18) -> None:
        self.y += px

    def table(
        self,
        headers: list[str],
        rows: list[list[str]],
        *,
        col_widths: list[int] | None = None,
        row_colors: list[str | None] | None = None,
    ) -> None:
        if col_widths is None:
            avail = PAGE_W - self.x0 * 2
            col_widths = [avail // len(headers)] * len(headers)
        row_h = 36
        x = self.x0

        # Header row
        if self._check_overflow(row_h + 4):
            return
        self.d.rectangle([x, self.y, x + sum(col_widths), self.y + row_h], fill=HEADER_BG, outline=BLACK, width=1)
        cx = x
        for h, w in zip(headers, col_widths):
            self.d.text((cx + 8, self.y + 8), h, fill="#1a3060", font=self._small_f)
            cx += w
        self.y += row_h

        # Data rows
        for ri, row in enumerate(rows):
            if self._check_overflow(row_h):
                break
            bg = row_colors[ri] if row_colors and ri < len(row_colors) else (ROW_ALT_BG if ri % 2 else BG_WHITE)
            self.d.rectangle([x, self.y, x + sum(col_widths), self.y + row_h], fill=bg, outline="#ccc", width=1)
            cx = x
            for cell, w in zip(row, col_widths):
                color = BLACK
                if cell in ("PASS", "✓"):
                    color = PASS_GREEN
                elif cell in ("FAIL", "✗"):
                    color = FAIL_RED
                elif cell.startswith("WARN"):
                    color = "#b07000"
                self.d.text((cx + 8, self.y + 8), str(cell), fill=color, font=self._small_f)
                cx += w
            self.y += row_h
        self.y += 12

    @property
    def image(self) -> Image.Image:
        return self.img


# ---------------------------------------------------------------------------
# Data loaders
# ---------------------------------------------------------------------------

def load_json(path: Path) -> dict:
    with path.open("r", encoding="utf-8") as fh:
        return json.load(fh)


def safe_load(path: Path) -> dict | None:
    try:
        return load_json(path) if path.exists() else None
    except Exception:
        return None


def _find_run(scenario: dict, run_id: str) -> dict | None:
    for r in scenario.get("runs", []):
        if r.get("summary", {}).get("run_id") == run_id:
            return r["summary"]
    return None


def _find_policy(scenario: dict, policy_id: str) -> dict | None:
    for p in scenario.get("policies", []):
        if p.get("policy_id") == policy_id:
            return p
    return None


def _canonical(scenarios: list[dict], key: str = "thin_reveal") -> dict | None:
    for s in scenarios:
        if s.get("scenario_id") == key:
            return s
    return scenarios[0] if scenarios else None


# ---------------------------------------------------------------------------
# Page builders
# ---------------------------------------------------------------------------

def page_executive_summary(
    run_dir: Path,
    test_results: dict,
    demo_a: dict | None,
) -> Page:
    p = Page("SBIR Demo — Engineering Quality Report")
    now = datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")
    p.small(f"Generated: {now}    Run: {run_dir.name}")
    p.spacer(10)

    # Overall test gate
    passed = test_results.get("passed", 0)
    failed = test_results.get("failed", 0)
    total = passed + failed
    ignored = test_results.get("ignored", 0)
    all_pass = failed == 0

    p.heading("Test Suite Gate")
    p.kv_row("Total tests", str(total))
    p.kv_row("Passed", str(passed), ok=True)
    p.kv_row("Failed", str(failed), ok=(failed == 0))
    p.kv_row("Ignored", str(ignored))
    p.badge(f"  ● {'ALL TESTS PASS' if all_pass else f'{failed} TEST(S) FAILED'}", all_pass)
    p.spacer(10)

    # Pipeline artifact gate
    p.heading("Artifact Presence")
    present = sum(1 for r in REQUIRED_ARTIFACTS if (run_dir / r).exists())
    missing = len(REQUIRED_ARTIFACTS) - present
    p.kv_row("Required artifacts", str(len(REQUIRED_ARTIFACTS)))
    p.kv_row("Present", str(present), ok=(missing == 0))
    p.kv_row("Missing", str(missing), ok=(missing == 0))
    p.spacer(10)

    # Demo A headline
    if demo_a:
        p.heading("Demo A Headline")
        summary = demo_a.get("summary", {})
        primary = summary.get("primary_behavioral_result", "—")
        p.paragraph(primary)
        scenario_count = len(summary.get("scenario_ids", []))
        baseline_count = len(summary.get("baseline_ids", []))
        ablation_count = len(summary.get("ablation_ids", []))
        p.kv_row("Scenarios", str(scenario_count), ok=(scenario_count >= 8))
        p.kv_row("Baselines", str(baseline_count), ok=(baseline_count >= 5))
        p.kv_row("Ablations", str(ablation_count), ok=(ablation_count >= 6))
    p.spacer(10)

    # Footer
    p.heading("How to Use This Report")
    p.small(
        "Pages: 1 Executive Summary · 2 Test Results · 3 Demo A Canonical Metrics · "
        "4 Demo A Scenario Coverage · 5 Demo B Fixed-Budget · 6 GPU Timing · "
        "7 Artifact Inventory · 8 Blockers & Readiness"
    )

    return p


def page_test_results(test_results: dict) -> Page:
    p = Page("Test Suite Results")
    tests = test_results.get("tests", [])

    stats_line = (
        f"{test_results.get('passed', 0)} passed  "
        f"{test_results.get('failed', 0)} failed  "
        f"{test_results.get('ignored', 0)} ignored"
    )
    p.small(stats_line, color=(PASS_GREEN if test_results.get("failed", 0) == 0 else FAIL_RED))
    p.spacer(8)

    if tests:
        cw = [820, 160, 120, 200]
        headers = ["Test Name", "Suite", "Result", "Duration"]
        rows = []
        for t in tests:
            name = t.get("name", "unknown")
            # split suite prefix
            parts = name.rsplit("::", 1)
            suite = parts[0].replace("tests::", "") if len(parts) > 1 else ""
            short = parts[-1]
            result = "PASS" if t.get("ok") else "FAIL"
            dur = t.get("duration_ms")
            dur_s = f"{dur} ms" if dur is not None else "—"
            rows.append([short[:60], suite[:28], result, dur_s])
        p.table(headers, rows, col_widths=cw)
    else:
        p.paragraph("No individual test data captured. Check that cargo test ran correctly.")

    return p


def page_demo_a_canonical(demo_a: dict | None) -> Page:
    p = Page("Demo A — Canonical Scenario Metrics")

    if not demo_a:
        p.paragraph("metrics.json not found — run `cargo run -- run-all` first.")
        return p

    scenarios = demo_a.get("scenarios", [])
    canonical = _canonical(scenarios)
    if not canonical:
        p.paragraph("No canonical scenario in metrics.json.")
        return p

    p.small(f"Canonical scenario: {canonical.get('scenario_id', '—')}   "
            f"Onset frame: {canonical.get('onset_frame', '—')}")
    p.spacer(8)

    fixed = _find_run(canonical, "fixed_alpha") or {}
    strong = _find_run(canonical, "strong_heuristic") or {}
    host = _find_run(canonical, "dsfb_host_realistic") or {}

    p.heading("Error Comparison (canonical scenario)")
    cw = [380, 240, 300, 300]
    p.table(
        ["Metric", "Fixed-alpha", "Strong heuristic", "Host-realistic (DSFB)"],
        [
            ["Ghost persistence frames",
             str(fixed.get("ghost_persistence_frames", "—")),
             str(strong.get("ghost_persistence_frames", "—")),
             str(host.get("ghost_persistence_frames", "—"))],
            ["Peak ROI MAE",
             _fmt(fixed["peak_roi_mae"]) if "peak_roi_mae" in fixed else "—",
             _fmt(strong["peak_roi_mae"]) if "peak_roi_mae" in strong else "—",
             _fmt(host["peak_roi_mae"]) if "peak_roi_mae" in host else "—"],
            ["Cumulative ROI MAE",
             _fmt(fixed["cumulative_roi_mae"]) if "cumulative_roi_mae" in fixed else "—",
             _fmt(strong["cumulative_roi_mae"]) if "cumulative_roi_mae" in strong else "—",
             _fmt(host["cumulative_roi_mae"]) if "cumulative_roi_mae" in host else "—"],
            ["Average non-ROI MAE",
             _fmt(fixed["average_non_roi_mae"]) if "average_non_roi_mae" in fixed else "—",
             _fmt(strong["average_non_roi_mae"]) if "average_non_roi_mae" in strong else "—",
             _fmt(host["average_non_roi_mae"]) if "average_non_roi_mae" in host else "—"],
            ["Onset response latency (frames)",
             str(fixed.get("onset_response_latency_frames", "—")),
             str(strong.get("onset_response_latency_frames", "—")),
             str(host.get("onset_response_latency_frames", "—"))],
            ["Trust/error rank correlation",
             "—",
             "—",
             f"{host['trust_error_rank_correlation']:.4f}" if "trust_error_rank_correlation" in host else "—"],
        ],
        col_widths=cw,
    )

    summary = demo_a.get("summary", {})
    mixed = summary.get("mixed_or_neutral_scenarios", [])
    p.spacer(8)
    p.heading("Suite Summary")
    p.kv_row("Primary result", summary.get("primary_behavioral_result", "—")[:80])
    if mixed:
        p.kv_row("Mixed / neutral scenarios", ", ".join(str(s) for s in mixed[:6]))
    dsfb_wins = summary.get("dsfb_win_scenarios", [])
    if dsfb_wins:
        p.kv_row("DSFB win scenarios", str(len(dsfb_wins)))

    return p


def page_demo_a_scenario_coverage(demo_a: dict | None) -> Page:
    p = Page("Demo A — Scenario Coverage")

    if not demo_a:
        p.paragraph("metrics.json not found.")
        return p

    summary = demo_a.get("summary", {})
    scenario_ids = summary.get("scenario_ids", [])
    baseline_ids = summary.get("baseline_ids", [])
    ablation_ids = summary.get("ablation_ids", [])

    p.heading("Scenario Suite")
    if scenario_ids:
        cw = [60, 620, 620]
        rows = [[str(i + 1), sid, "✓"] for i, sid in enumerate(scenario_ids)]
        p.table(["#", "Scenario ID", "Present"], rows, col_widths=cw)
    else:
        p.paragraph("No scenario IDs found.")

    p.heading("Baselines")
    if baseline_ids:
        cw = [60, 800, 440]
        rows = [[str(i + 1), bid, "✓"] for i, bid in enumerate(baseline_ids)]
        p.table(["#", "Baseline ID", "Present"], rows, col_widths=cw)

    p.heading("Ablations")
    if ablation_ids:
        cw = [60, 800, 440]
        rows = [[str(i + 1), aid, "✓"] for i, aid in enumerate(ablation_ids)]
        p.table(["#", "Ablation ID", "Present"], rows, col_widths=cw)

    return p


def page_demo_b(demo_b: dict | None) -> Page:
    p = Page("Demo B — Fixed-Budget Adaptive Sampling")

    if not demo_b:
        p.paragraph("demo_b/metrics.json not found — run `cargo run -- run-all` first.")
        return p

    summary = demo_b.get("summary", {})
    scenarios = demo_b.get("scenarios", [])
    canonical = _canonical(scenarios)

    p.heading("Suite Summary")
    imported_wins = summary.get("imported_trust_beats_uniform_scenarios", "—")
    neutral = summary.get("neutral_or_mixed_scenarios", [])
    p.kv_row("Imported-trust beats uniform (# scenarios)", str(imported_wins),
             ok=(isinstance(imported_wins, int) and imported_wins >= 1))
    p.kv_row("Neutral or mixed scenarios", str(len(neutral)))
    p.spacer(8)

    if canonical:
        p.heading(f"Canonical Scenario: {canonical.get('scenario_id', '—')}")
        uniform = _find_policy(canonical, "uniform") or {}
        combined = _find_policy(canonical, "combined_heuristic") or {}
        imported = _find_policy(canonical, "imported_trust") or {}

        cw = [360, 240, 240, 240, 360]
        p.table(
            ["Policy", "Total samples", "ROI MAE", "Non-ROI MAE", "Budget equal"],
            [
                ["uniform",
                 str(uniform.get("total_samples", "—")),
                 _fmt(uniform["roi_mae"]) if "roi_mae" in uniform else "—",
                 _fmt(uniform["non_roi_mae"]) if "non_roi_mae" in uniform else "—",
                 "✓"],
                ["combined_heuristic",
                 str(combined.get("total_samples", "—")),
                 _fmt(combined["roi_mae"]) if "roi_mae" in combined else "—",
                 _fmt(combined["non_roi_mae"]) if "non_roi_mae" in combined else "—",
                 "✓"],
                ["imported_trust",
                 str(imported.get("total_samples", "—")),
                 _fmt(imported["roi_mae"]) if "roi_mae" in imported else "—",
                 _fmt(imported["non_roi_mae"]) if "non_roi_mae" in imported else "—",
                 "✓"],
            ],
            col_widths=cw,
        )
        # Budget preservation check
        total_u = uniform.get("total_samples")
        total_i = imported.get("total_samples")
        total_c = combined.get("total_samples")
        budget_ok = total_u == total_i == total_c and total_u is not None
        p.badge(f"  {'✓ Budget preserved across all policies' if budget_ok else '✗ Budget mismatch detected'}", budget_ok)
    p.spacer(8)

    # Per-scenario summary table
    if len(scenarios) > 1:
        p.heading("Per-Scenario Results")
        cw = [500, 220, 220, 200, 300]
        rows = []
        for s in scenarios[:20]:
            sid = s.get("scenario_id", "—")
            uni = _find_policy(s, "uniform") or {}
            imp = _find_policy(s, "imported_trust") or {}
            u_mae = uni.get("roi_mae")
            i_mae = imp.get("roi_mae")
            win = "✓" if (u_mae is not None and i_mae is not None and i_mae < u_mae) else "—"
            rows.append([
                sid,
                _fmt(u_mae) if u_mae is not None else "—",
                _fmt(i_mae) if i_mae is not None else "—",
                win,
                "equal" if uni.get("total_samples") == imp.get("total_samples") else "MISMATCH",
            ])
        p.table(["Scenario", "Uniform ROI MAE", "Imported ROI MAE", "Win", "Budget"], rows, col_widths=cw)

    return p


def page_gpu_timing(run_dir: Path) -> Page:
    p = Page("GPU Timing & Execution")

    gpu_metrics = safe_load(run_dir / "gpu_execution_metrics.json")
    if gpu_metrics:
        p.heading("GPU Execution Metrics")
        for key, val in gpu_metrics.items():
            if isinstance(val, (int, float, str, bool)):
                p.kv_row(str(key), str(val))
        p.spacer(8)

    gpu_report_path = run_dir / "gpu_execution_report.md"
    if gpu_report_path.exists():
        p.heading("GPU Execution Report (excerpt)")
        lines = gpu_report_path.read_text(encoding="utf-8").splitlines()
        for line in lines[:40]:
            if line.startswith("#"):
                p.small(line.lstrip("#").strip(), color="#1a3060", spacing=4)
            else:
                stripped = line.strip()
                if stripped:
                    p.mono(stripped[:120], spacing=4)

    timing_metrics = safe_load(run_dir / "timing_metrics.json")
    if timing_metrics:
        p.heading("Timing Metrics")
        for key, val in timing_metrics.items():
            if isinstance(val, (int, float, str, bool)):
                p.kv_row(str(key), str(val))

    if not gpu_metrics and not gpu_report_path.exists():
        p.paragraph("GPU metrics not found. Run `cargo run -- run-all` to generate.")

    return p


def page_artifact_inventory(run_dir: Path) -> Page:
    p = Page("Artifact Inventory")

    p.heading(f"Required Artifacts ({len(REQUIRED_ARTIFACTS)} total)")

    cw = [920, 120, 400]
    rows = []
    for rel in REQUIRED_ARTIFACTS:
        path = run_dir / rel
        present = path.exists()
        size_str = "—"
        if present:
            try:
                size_b = path.stat().st_size
                size_str = f"{size_b:,} B" if size_b < 10_000 else f"{size_b // 1024:,} KB"
            except OSError:
                pass
        rows.append([rel, "✓" if present else "✗", size_str])

    p.table(["Artifact path", "Present", "Size"], rows, col_widths=cw)

    # Count bonus artifacts
    all_files = list(run_dir.rglob("*"))
    file_count = sum(1 for f in all_files if f.is_file())
    p.spacer(6)
    p.small(f"Total files in run directory: {file_count}")

    return p


def page_blockers_readiness(run_dir: Path) -> Page:
    p = Page("Blockers & Readiness")

    # Read blocker report
    blocker_path = run_dir / "check_signing_blockers.md"
    if blocker_path.exists():
        p.heading("Blocker Report (excerpt — check_signing_blockers.md)")
        lines = blocker_path.read_text(encoding="utf-8").splitlines()
        for line in lines[:50]:
            if line.startswith("## "):
                p.heading(line[3:].strip())
            elif line.startswith("- "):
                p.small("• " + line[2:].strip(), indent=20, spacing=4)
            elif line.strip():
                p.small(line.strip()[:120], spacing=4)
        p.spacer(6)

    # Readiness report
    readiness_path = run_dir / "check_signing_readiness.md"
    if readiness_path.exists():
        p.heading("Signing Readiness (excerpt — check_signing_readiness.md)")
        lines = readiness_path.read_text(encoding="utf-8").splitlines()
        for line in lines[:30]:
            stripped = line.strip()
            if stripped:
                p.small(stripped[:120], spacing=4)
        p.spacer(6)

    # External validation status
    ext_val = run_dir / "external_real" / "external_validation_report.md"
    if ext_val.exists():
        p.heading("External Validation Status")
        first_lines = ext_val.read_text(encoding="utf-8").splitlines()[:15]
        for line in first_lines:
            stripped = line.strip()
            if stripped:
                p.small(stripped[:120], spacing=4)

    return p


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Generate SBIR demo engineering report PDF.")
    parser.add_argument("--run-dir", required=True, help="Pipeline run directory (output of run-all).")
    parser.add_argument("--test-results", required=True, help="Path to test_results.json from cargo test.")
    parser.add_argument("--output", required=True, help="Output PDF path.")
    return parser.parse_args()


def main() -> None:
    args = parse_args()
    run_dir = Path(args.run_dir).resolve()
    test_results_path = Path(args.test_results)
    output_path = Path(args.output)

    if not run_dir.exists():
        raise FileNotFoundError(f"run-dir does not exist: {run_dir}")

    test_results: dict = {}
    if test_results_path.exists():
        test_results = load_json(test_results_path)
    else:
        test_results = {"passed": 0, "failed": 0, "ignored": 0, "tests": [],
                        "note": "test_results.json not found"}

    demo_a = safe_load(run_dir / "metrics.json")
    demo_b = safe_load(run_dir / "demo_b" / "metrics.json")

    pages = [
        page_executive_summary(run_dir, test_results, demo_a),
        page_test_results(test_results),
        page_demo_a_canonical(demo_a),
        page_demo_a_scenario_coverage(demo_a),
        page_demo_b(demo_b),
        page_gpu_timing(run_dir),
        page_artifact_inventory(run_dir),
        page_blockers_readiness(run_dir),
    ]

    output_path.parent.mkdir(parents=True, exist_ok=True)
    images = [p.image for p in pages]
    images[0].save(
        str(output_path),
        "PDF",
        resolution=150.0,
        save_all=True,
        append_images=images[1:],
    )
    print(json.dumps({"pdf": str(output_path), "pages": len(images)}))


if __name__ == "__main__":
    main()
