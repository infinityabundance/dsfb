#!/usr/bin/env python3

from __future__ import annotations

import argparse
import json
import textwrap
import zipfile
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Build the Unreal-native executive sheet, PDF, and ZIP bundle."
    )
    parser.add_argument("--run-dir", required=True, help="Run directory produced by run-unreal-native")
    return parser.parse_args()


def load_json(path: Path) -> dict:
    with path.open("r", encoding="utf-8") as handle:
        return json.load(handle)


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
        if Path(candidate).exists():
            return ImageFont.truetype(candidate, size=size)
    return ImageFont.load_default()


TITLE_FONT = font(42, bold=True)
HEADING_FONT = font(26, bold=True)
BODY_FONT = font(20)
SMALL_FONT = font(17)


def wrap(draw: ImageDraw.ImageDraw, text: str, active_font: ImageFont.ImageFont, width: int) -> list[str]:
    words = text.split()
    if not words:
        return [""]
    lines: list[str] = []
    current = words[0]
    for word in words[1:]:
        candidate = f"{current} {word}"
        if draw.textlength(candidate, font=active_font) <= width:
            current = candidate
        else:
            lines.append(current)
            current = word
    lines.append(current)
    return lines


def load_required_image(run_dir: Path, relative: str) -> Image.Image:
    path = run_dir / relative
    if not path.exists():
        raise FileNotFoundError(f"required image missing: {path}")
    return Image.open(path).convert("RGB")


def draw_wrapped(
    draw: ImageDraw.ImageDraw,
    text: str,
    *,
    x: int,
    y: int,
    width: int,
    active_font: ImageFont.ImageFont,
    fill: str = "black",
    spacing: int = 8,
) -> int:
    cursor_y = y
    for line in wrap(draw, text, active_font, width):
        draw.text((x, cursor_y), line, fill=fill, font=active_font)
        cursor_y += active_font.getbbox("Ag")[3] - active_font.getbbox("Ag")[1] + spacing
    return cursor_y


def panel_tile(image: Image.Image, size: tuple[int, int]) -> Image.Image:
    tile = image.copy()
    tile.thumbnail(size, Image.Resampling.LANCZOS)
    canvas = Image.new("RGB", size, "#f4f5f7")
    offset_x = (size[0] - tile.width) // 2
    offset_y = (size[1] - tile.height) // 2
    canvas.paste(tile, (offset_x, offset_y))
    return canvas


def build_boardroom_panel(run_dir: Path, frame: dict) -> Path:
    current = load_required_image(run_dir, frame["current_frame_path"])
    baseline = load_required_image(run_dir, frame["baseline_frame_path"])
    trust = load_required_image(run_dir, frame["trust_map_path"])
    alpha = load_required_image(run_dir, frame["alpha_map_path"])
    intervention = load_required_image(run_dir, frame["intervention_map_path"])
    instability = load_required_image(run_dir, frame["instability_overlay_path"])

    canvas = Image.new("RGB", (1800, 1350), "#ffffff")
    draw = ImageDraw.Draw(canvas)
    draw.text((60, 40), f"DSFB Unreal-Native Panel: {frame['label']}", fill="black", font=TITLE_FONT)
    subtitle = (
        f"{frame['scene_name']} / {frame['shot_name']} / frame {frame['frame_index']} "
        f" / classification = {frame['classification']}"
    )
    draw.text((60, 95), subtitle, fill="#374151", font=BODY_FONT)

    tiles = [
        ("Current frame", current),
        ("Fixed-alpha baseline", baseline),
        ("DSFB trust", trust),
        ("DSFB alpha", alpha),
        ("DSFB intervention", intervention),
        ("Instability overlay", instability),
    ]

    tile_w, tile_h = 520, 300
    start_x = 60
    start_y = 150
    gap_x = 40
    gap_y = 60

    for index, (label, image) in enumerate(tiles):
        row = index // 3
        col = index % 3
        x = start_x + col * (tile_w + gap_x)
        y = start_y + row * (tile_h + gap_y)
        draw.rectangle([x - 2, y - 34, x + tile_w + 2, y + tile_h + 2], outline="#d1d5db", width=2)
        draw.text((x, y - 28), label, fill="black", font=HEADING_FONT)
        canvas.paste(panel_tile(image, (tile_w, tile_h)), (x, y))

    metric_x = 60
    metric_y = 860
    draw.text((metric_x, metric_y), "Key metrics", fill="black", font=HEADING_FONT)
    metric_y += 42
    for metric in frame["key_metrics"]:
        draw.text(
            (metric_x, metric_y),
            f"{metric['label']}: {metric['value']}",
            fill="#111827",
            font=BODY_FONT,
        )
        metric_y += 34

    explanation_x = 760
    explanation_y = 860
    draw.text((explanation_x, explanation_y), "Interpretation", fill="black", font=HEADING_FONT)
    explanation_y += 46
    for heading, text in [
        ("What went wrong", frame["explanation"]["what_went_wrong"]),
        ("What DSFB detected", frame["explanation"]["what_dsfb_detected"]),
        ("What DSFB changed", frame["explanation"]["what_dsfb_changed"]),
        ("Overhead / caveat", frame["explanation"]["overhead_and_caveat"]),
    ]:
        draw.text((explanation_x, explanation_y), f"{heading}:", fill="black", font=BODY_FONT)
        explanation_y += 28
        explanation_y = draw_wrapped(
            draw,
            text,
            x=explanation_x,
            y=explanation_y,
            width=900,
            active_font=SMALL_FONT,
            fill="#374151",
            spacing=6,
        )
        explanation_y += 12

    output_path = run_dir / frame["output_panel_path"]
    output_path.parent.mkdir(parents=True, exist_ok=True)
    canvas.save(output_path)
    return output_path


def build_executive_sheet(run_dir: Path, frame: dict) -> Path:
    panel = load_required_image(run_dir, frame["output_panel_path"])
    canvas = Image.new("RGB", (1700, 1100), "#ffffff")
    draw = ImageDraw.Draw(canvas)
    draw.text((50, 40), "Executive Evidence Sheet", fill="black", font=TITLE_FONT)
    draw.text(
        (50, 95),
        f"{frame['scene_name']} / {frame['shot_name']} / frame {frame['frame_index']}",
        fill="#374151",
        font=BODY_FONT,
    )

    panel.thumbnail((980, 760), Image.Resampling.LANCZOS)
    canvas.paste(panel, (50, 170))

    right_x = 1070
    cursor_y = 170
    draw.text((right_x, cursor_y), "Decision posture", fill="black", font=HEADING_FONT)
    cursor_y += 42
    cursor_y = draw_wrapped(
        draw,
        "This page summarizes one Unreal-native frame where DSFB is acting as a bounded supervisory layer over a temporal reuse path.",
        x=right_x,
        y=cursor_y,
        width=560,
        active_font=BODY_FONT,
        fill="#111827",
    )
    cursor_y += 20

    for heading, text in [
        ("1. What went wrong", frame["explanation"]["what_went_wrong"]),
        ("2. What DSFB detected", frame["explanation"]["what_dsfb_detected"]),
        ("3. What DSFB changed", frame["explanation"]["what_dsfb_changed"]),
        ("4. Overhead / caveat", frame["explanation"]["overhead_and_caveat"]),
    ]:
        draw.text((right_x, cursor_y), heading, fill="black", font=HEADING_FONT)
        cursor_y += 34
        cursor_y = draw_wrapped(
            draw,
            text,
            x=right_x,
            y=cursor_y,
            width=560,
            active_font=SMALL_FONT,
            fill="#374151",
            spacing=5,
        )
        cursor_y += 14

    draw.text((right_x, cursor_y), "Core metrics", fill="black", font=HEADING_FONT)
    cursor_y += 40
    for metric in frame["key_metrics"][:5]:
        draw.text((right_x, cursor_y), f"{metric['label']}: {metric['value']}", fill="#111827", font=BODY_FONT)
        cursor_y += 32

    output_path = run_dir / "executive_evidence_sheet.png"
    canvas.save(output_path)
    return output_path


def build_pdf(run_dir: Path, summary: dict, manifest: dict) -> Path:
    pages: list[Image.Image] = []

    title = Image.new("RGB", (1600, 2100), "#ffffff")
    draw = ImageDraw.Draw(title)
    draw.text((90, 100), "DSFB Unreal-Native Evidence Bundle", fill="black", font=TITLE_FONT)
    cursor_y = 180
    for paragraph in [
        f"Dataset: {summary['dataset_id']}",
        f"Provenance: {summary['provenance_label']}",
        f"Captures: {summary['capture_count']}",
        "This PDF is generated from one strict Unreal-native replay run. It is an evidence package for technical diligence, not a claim of universal superiority.",
        "The crate is acting as a supervisory trust / admissibility / intervention layer over a real Unreal-exported temporal pipeline input package.",
    ]:
        cursor_y = draw_wrapped(
            draw,
            paragraph,
            x=90,
            y=cursor_y,
            width=1360,
            active_font=BODY_FONT,
            fill="#111827",
        )
        cursor_y += 18
    pages.append(title)

    executive = Image.open(run_dir / manifest["executive_sheet_file_name"]).convert("RGB")
    pages.append(executive)

    for frame in manifest["frames"]:
        pages.append(Image.open(run_dir / frame["output_panel_path"]).convert("RGB"))

    pdf_path = run_dir / manifest["pdf_file_name"]
    first, *rest = pages
    first.save(pdf_path, save_all=True, append_images=rest)
    return pdf_path


def build_zip(run_dir: Path, manifest: dict) -> Path:
    zip_path = run_dir / manifest["zip_file_name"]
    if zip_path.exists():
        zip_path.unlink()

    with zipfile.ZipFile(zip_path, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for path in sorted(run_dir.rglob("*")):
            if not path.is_file():
                continue
            if path == zip_path:
                continue
            archive.write(path, path.relative_to(run_dir))
    return zip_path


def main() -> None:
    args = parse_args()
    run_dir = Path(args.run_dir).resolve()
    manifest = load_json(run_dir / "evidence_bundle_manifest.json")
    summary = load_json(run_dir / "summary.json")

    for frame in manifest["frames"]:
        build_boardroom_panel(run_dir, frame)

    executive_frame_label = summary["executive_capture_label"]
    executive_frame = next(frame for frame in manifest["frames"] if frame["label"] == executive_frame_label)
    build_executive_sheet(run_dir, executive_frame)
    build_pdf(run_dir, summary, manifest)
    build_zip(run_dir, manifest)


if __name__ == "__main__":
    main()
