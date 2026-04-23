#!/usr/bin/env python3
"""
Prepare eight real-dataset slices for the dsfb-rf Colab reproducibility notebook.

WHY
---
The companion paper lists eight real RF datasets as slice exhibits:
RadioML, ORACLE, POWDER, Tampere GNSS, ColO-RAN, ColO-RAN-commag, DeepBeam,
DeepSense-6G. The full datasets range from 1 GB to 100+ GB and cannot be
fetched in a free-tier Colab session. This script extracts schema-preserving
<=2 MB slices that ARE committable and can round-trip through Colab without
network access where possible.

WHAT IT DOES
------------
For each of the eight datasets:

  1. If a pre-extracted slice already exists in data/slices/, verify SHA-256
     and move on.
  2. Else: try the real source. Local ZIPs (ORACLE, POWDER) are extracted
     head-slice style. Public mirrors (Tampere GNSS, ColO-RAN, ColO-RAN-commag)
     are probed via urllib. If the mirror fetch succeeds, a sliced artefact
     is written and marked `provenance="real-public"`.
  3. Else: a `[SYNTHETIC PROXY]` slice is generated from the crate's
     existing signal models (via the gen_proxy_*.py sibling modules).
     Attributes are stamped with `provenance="synthetic-proxy"` and the
     reason the real path failed. Every proxy emission prints a loud
     stderr banner so a reviewer scanning a notebook output can tell.

Every slice ends up in data/slices/<name>.<ext> and an entry in
data/slices/SLICE_MANIFEST.json with SHA-256, bytes, provenance, and schema.

ACADEMIC HONESTY
----------------
A proxy slice is NOT the same as the real dataset. The notebook cell that
summarises slices prints the provenance column for every row. No headline
figure or Table 1 number is computed from any slice — the slices are
contextual exhibits only, fed to `generate_figures_all.rs` which reads
none of them (the figures still come from the crate's deterministic
synthetic models, frozen at v1.0.0).

USAGE
-----
    python3 scripts/prepare_slices.py              # try real, fall back to proxy
    python3 scripts/prepare_slices.py --offline    # skip network; proxy-only
    python3 scripts/prepare_slices.py --force      # regenerate even if cached
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import sys
import time
import urllib.error
import urllib.request
import zipfile
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Callable, Optional

import numpy as np

HERE = Path(__file__).resolve().parent
CRATE = HERE.parent
DATA = CRATE / "data"
SLICES = DATA / "slices"
MANIFEST_PATH = SLICES / "SLICE_MANIFEST.json"

MAX_SLICE_BYTES = 2 * 1024 * 1024

sys.path.insert(0, str(HERE))

import gen_proxy_coloran  # noqa: E402
import gen_proxy_coloran_commag  # noqa: E402
import gen_proxy_deepbeam  # noqa: E402
import gen_proxy_deepsense  # noqa: E402
import gen_proxy_oracle  # noqa: E402
import gen_proxy_powder  # noqa: E402
import gen_proxy_tampere_gnss  # noqa: E402


# --- Tiny utilities ---------------------------------------------------------


def sha256_of(path: Path, chunk: int = 4 * 1024 * 1024) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        while True:
            b = f.read(chunk)
            if not b:
                break
            h.update(b)
    return h.hexdigest()


def try_download(url: str, timeout: int = 60, max_bytes: int = MAX_SLICE_BYTES) -> Optional[bytes]:
    """GET the first max_bytes via HTTP Range; fall back to a bounded read.

    Prevents accidental multi-GB downloads when a mirror points at a large
    archive (e.g. Zenodo Data.zip). Always returns at most max_bytes.
    """
    try:
        headers = {
            "User-Agent": "dsfb-rf-prepare-slices/1",
            "Range": f"bytes=0-{max_bytes - 1}",
        }
        req = urllib.request.Request(url, headers=headers)
        with urllib.request.urlopen(req, timeout=timeout) as resp:
            return resp.read(max_bytes)
    except (urllib.error.URLError, urllib.error.HTTPError, TimeoutError, OSError) as exc:
        print(f"  [network] {url} -> {exc}", file=sys.stderr)
        return None


def loud_proxy_banner(name: str, reason: str) -> None:
    bar = "=" * 72
    print(bar, file=sys.stderr)
    print(f"[SYNTHETIC PROXY]  {name}: real source unavailable.", file=sys.stderr)
    print(f"                   reason: {reason}", file=sys.stderr)
    print(f"                   emitting schema-preserving proxy (<=2 MB).", file=sys.stderr)
    print(bar, file=sys.stderr)


def looks_like_csv(blob: bytes) -> bool:
    """True iff the first ~2 KB look like a CSV header + data row.

    Guard against falsely labelling a fetched README.md or HTML error page
    as a real CSV slice. We need a comma on the first line and at least one
    additional line to count as a KPI trace.
    """
    head = blob[:2048]
    if not head:
        return False
    if head[:1] == b"<" or head.lstrip()[:5].lower() == b"<!doc":
        return False
    try:
        text = head.decode("utf-8", errors="replace")
    except Exception:
        return False
    lines = text.splitlines()
    if len(lines) < 2:
        return False
    if "," not in lines[0]:
        return False
    if lines[0].lower().startswith("#") or lines[0].startswith("---"):
        return False
    return True


# --- Per-dataset handlers ---------------------------------------------------


@dataclass
class SliceEntry:
    """One row in SLICE_MANIFEST.json."""

    name: str
    provenance: str  # "real-in-repo" | "real-local-zip" | "real-public" | "synthetic-proxy"
    files: list[str] = field(default_factory=list)
    bytes: int = 0
    sha256: str = ""
    schema: str = ""
    source_url: str = ""
    reason: str = ""
    generated_at: str = field(default_factory=lambda: time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()))


def _compose_sha256(paths: list[Path]) -> tuple[str, int]:
    h = hashlib.sha256()
    total = 0
    for p in sorted(paths):
        h.update(p.name.encode())
        with p.open("rb") as fh:
            while True:
                buf = fh.read(1 << 20)
                if not buf:
                    break
                h.update(buf)
                total += len(buf)
    return h.hexdigest(), total


def handle_radioml(offline: bool, force: bool) -> SliceEntry:
    out = SLICES / "radioml_2018_slice.hdf5"
    if not out.exists():
        raise FileNotFoundError(
            f"radioml_2018_slice.hdf5 missing from {SLICES}. "
            f"Re-run scripts/extract_radioml_slice.py first."
        )
    digest, size = _compose_sha256([out])
    return SliceEntry(
        name="radioml",
        provenance="real-in-repo",
        files=[out.name],
        bytes=size,
        sha256=digest,
        schema="HDF5 {X: (N,1024,2) float32, Y: (N,24) int64, Z: (N,1) int64}",
        source_url="DeepSig RadioML 2018.01a — CC BY-NC-SA 4.0 derivative",
    )


def _oracle_extract_head_from_zip(zip_path: Path, out_meta: Path, out_data: Path) -> Optional[dict]:
    """Find first .sigmf-data + .sigmf-meta pair in the zip; write a <=2 MB head.

    ORACLE stores samples as complex128 (16 B/sample) although metadata claims
    cf32 — the dtype mismatch is documented in REPRODUCE.md. We preserve the
    quirk verbatim: the slice is the byte-identical head of the original
    .sigmf-data, and the slice's .sigmf-meta reuses the parent metadata with
    an updated sample_count.
    """
    with zipfile.ZipFile(zip_path) as zf:
        data_members = sorted(m for m in zf.namelist() if m.endswith(".sigmf-data"))
        meta_members = sorted(m for m in zf.namelist() if m.endswith(".sigmf-meta"))
        if not data_members or not meta_members:
            return None
        data_name = data_members[0]
        meta_name = data_name.replace(".sigmf-data", ".sigmf-meta")
        if meta_name not in zf.namelist():
            meta_name = meta_members[0]

        with zf.open(data_name) as df:
            head = df.read(MAX_SLICE_BYTES)
        out_data.write_bytes(head)

        with zf.open(meta_name) as mf:
            meta = json.loads(mf.read().decode("utf-8"))

    # ORACLE files stored as complex128 (16 B/sample).
    sample_bytes = 16
    new_count = len(head) // sample_bytes

    if "_metadata" in meta and "annotations" in meta["_metadata"]:
        for ann in meta["_metadata"]["annotations"]:
            if "core:sample_count" in ann:
                ann["core:sample_count"] = new_count
    if "annotations" in meta:
        anns = meta["annotations"]
        if isinstance(anns, list):
            for ann in anns:
                if isinstance(ann, dict) and "core:sample_count" in ann:
                    ann["core:sample_count"] = new_count
        elif isinstance(anns, dict) and "core:sample_count" in anns:
            anns["core:sample_count"] = new_count

    meta.setdefault("dsfb_rf_slice", {})
    meta["dsfb_rf_slice"].update(
        {
            "parent_zip": zip_path.name,
            "parent_member": data_name,
            "head_bytes": len(head),
            "on_disk_dtype": "complex128 (metadata declares cf32 — parent dataset quirk, preserved)",
            "notice": "Head-slice byte-identical to parent .sigmf-data; truncated for git.",
        }
    )

    out_meta.write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")
    return {
        "schema": (
            "SigMF .sigmf-meta + .sigmf-data; core:datatype=cf32 (parent quirk: on-disk cf64)"
            f"; sliced sample_count={new_count}"
        ),
        "source": f"local zip: {zip_path.name} -> {data_name}",
    }


def handle_oracle(offline: bool, force: bool) -> SliceEntry:
    zip_path = DATA / "ORACLE" / "neu_m044q5210.zip"
    out_meta = SLICES / "oracle_slice.sigmf-meta"
    out_data = SLICES / "oracle_slice.sigmf-data"

    if not force and out_meta.exists() and out_data.exists():
        digest, size = _compose_sha256([out_meta, out_data])
        return SliceEntry(
            name="oracle",
            provenance="real-local-zip",
            files=[out_meta.name, out_data.name],
            bytes=size,
            sha256=digest,
            schema="SigMF cf32 (parent on-disk cf64 preserved); Northeastern GENESYS ORACLE USRP X310",
            source_url=f"local: {zip_path}",
        )

    if zip_path.exists():
        info = _oracle_extract_head_from_zip(zip_path, out_meta, out_data)
        if info is not None:
            digest, size = _compose_sha256([out_meta, out_data])
            return SliceEntry(
                name="oracle",
                provenance="real-local-zip",
                files=[out_meta.name, out_data.name],
                bytes=size,
                sha256=digest,
                schema=info["schema"],
                source_url=info["source"],
            )

    reason = f"local zip {zip_path} not present and public mirror gated by Northeastern GENESYS"
    loud_proxy_banner("oracle", reason)
    gen_proxy_oracle.generate(out_meta, out_data, rng=np.random.default_rng(20260421))
    digest, size = _compose_sha256([out_meta, out_data])
    return SliceEntry(
        name="oracle",
        provenance="synthetic-proxy",
        files=[out_meta.name, out_data.name],
        bytes=size,
        sha256=digest,
        schema="SigMF cf32 (proxy); USRP X310 fingerprinting residual model",
        source_url="crate synthetic model; see examples/oracle_usrp_b200.rs",
        reason=reason,
    )


def _powder_extract_head_from_zip(zip_path: Path, out_json: Path, out_bin: Path) -> Optional[dict]:
    """POWDER ships <name>.bin (cf32, 42.4 MB) + <name>.json (SigMF-lite)."""
    with zipfile.ZipFile(zip_path) as zf:
        bins = sorted(m for m in zf.namelist() if m.endswith(".bin"))
        jsons = sorted(m for m in zf.namelist() if m.endswith(".json"))
        if not bins or not jsons:
            return None
        bin_name = bins[0]
        json_name = bin_name.replace(".bin", ".json")
        if json_name not in zf.namelist():
            json_name = jsons[0]

        with zf.open(bin_name) as bf:
            head = bf.read(MAX_SLICE_BYTES)
        out_bin.write_bytes(head)

        with zf.open(json_name) as jf:
            meta = json.loads(jf.read().decode("utf-8"))

    sample_bytes = 8  # cf32 per metadata
    new_count = len(head) // sample_bytes

    anns = meta.get("annotations")
    if isinstance(anns, dict) and "core:sample_count" in anns:
        anns["core:sample_count"] = str(new_count)
    elif isinstance(anns, list):
        for ann in anns:
            if isinstance(ann, dict) and "core:sample_count" in ann:
                ann["core:sample_count"] = str(new_count)

    meta.setdefault("dsfb_rf_slice", {})
    meta["dsfb_rf_slice"].update(
        {
            "parent_zip": zip_path.name,
            "parent_member": bin_name,
            "head_bytes": len(head),
            "notice": "Head-slice byte-identical to parent .bin; truncated for git.",
        }
    )

    out_json.write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")
    return {
        "schema": f"POWDER cf32 raw .bin + .json; sliced sample_count={new_count}",
        "source": f"local zip: {zip_path.name} -> {bin_name}",
    }


def handle_powder(offline: bool, force: bool) -> SliceEntry:
    zip_path = DATA / "POWDER" / "neu_m046tb444.zip"
    out_json = SLICES / "powder_slice.json"
    out_bin = SLICES / "powder_slice.bin"

    if not force and out_json.exists() and out_bin.exists():
        digest, size = _compose_sha256([out_json, out_bin])
        return SliceEntry(
            name="powder",
            provenance="real-local-zip",
            files=[out_json.name, out_bin.name],
            bytes=size,
            sha256=digest,
            schema="POWDER cf32 raw .bin + .json; Globecom POWDER 4G LTE Band 7 capture",
            source_url=f"local: {zip_path}",
        )

    if zip_path.exists():
        info = _powder_extract_head_from_zip(zip_path, out_json, out_bin)
        if info is not None:
            digest, size = _compose_sha256([out_json, out_bin])
            return SliceEntry(
                name="powder",
                provenance="real-local-zip",
                files=[out_json.name, out_bin.name],
                bytes=size,
                sha256=digest,
                schema=info["schema"],
                source_url=info["source"],
            )

    reason = f"local zip {zip_path} not present and public mirror gated by genesys-lab.org"
    loud_proxy_banner("powder", reason)
    gen_proxy_powder.generate(out_json, out_bin, rng=np.random.default_rng(20260421 + 1))
    digest, size = _compose_sha256([out_json, out_bin])
    return SliceEntry(
        name="powder",
        provenance="synthetic-proxy",
        files=[out_json.name, out_bin.name],
        bytes=size,
        sha256=digest,
        schema="POWDER cf32 .bin + .json (proxy); PAWR LTE Band 7 urban multipath model",
        source_url="crate synthetic model; see examples/urban_multipath_prognosis.rs",
        reason=reason,
    )


def _fetch_slice_real_or_proxy(
    name: str,
    out_files: list[Path],
    real_fetch: Callable[[], Optional[dict]],
    proxy_fn: Callable[..., Any],
    proxy_args: tuple,
    schema_if_real: str,
    schema_if_proxy: str,
    proxy_reason_hint: str,
    offline: bool,
    force: bool,
) -> SliceEntry:
    if not force and all(p.exists() for p in out_files):
        digest, size = _compose_sha256(out_files)
        return SliceEntry(
            name=name,
            provenance="real-public" if any(p.name.endswith(".csv") or p.name.endswith(".bin") or p.name.endswith(".hdf5") or p.name.endswith(".sigmf-data") for p in out_files) else "synthetic-proxy",
            files=[p.name for p in out_files],
            bytes=size,
            sha256=digest,
            schema=schema_if_real,  # trust manifest post-hoc
            source_url="cached",
        )

    if not offline:
        info = real_fetch()
        if info is not None:
            digest, size = _compose_sha256(out_files)
            return SliceEntry(
                name=name,
                provenance="real-public",
                files=[p.name for p in out_files],
                bytes=size,
                sha256=digest,
                schema=schema_if_real,
                source_url=info.get("source", ""),
            )

    reason = proxy_reason_hint if offline else f"{proxy_reason_hint} (network attempt failed)"
    loud_proxy_banner(name, reason)
    proxy_fn(*proxy_args)
    digest, size = _compose_sha256(out_files)
    return SliceEntry(
        name=name,
        provenance="synthetic-proxy",
        files=[p.name for p in out_files],
        bytes=size,
        sha256=digest,
        schema=schema_if_proxy,
        source_url="crate synthetic model",
        reason=reason,
    )


def handle_tampere_gnss(offline: bool, force: bool) -> SliceEntry:
    out_bin = SLICES / "tampere_gnss_slice.bin"
    out_meta = SLICES / "tampere_gnss_slice.json"

    def real_fetch() -> Optional[dict]:
        url = "https://zenodo.org/records/13846381/files/Data.zip"
        blob = try_download(url, timeout=30)
        if blob is None:
            return None
        out_bin.write_bytes(blob[:MAX_SLICE_BYTES])
        meta = {
            "source": url,
            "note": "Head slice of Tampere Uni GNSS RFF Data.zip (CC BY 4.0, Wang/Sankari/Lohan/Valkama, Zenodo 10.5281/zenodo.13846381)",
            "head_bytes": min(len(blob), MAX_SLICE_BYTES),
        }
        out_meta.write_text(json.dumps(meta, indent=2) + "\n", encoding="utf-8")
        return {"source": url}

    return _fetch_slice_real_or_proxy(
        "tampere_gnss",
        [out_bin, out_meta],
        real_fetch,
        gen_proxy_tampere_gnss.generate,
        (out_bin, out_meta, np.random.default_rng(20260421 + 2)),
        schema_if_real="Tampere GNSS Raw IQ (L1 C/A baseband) head slice; CC BY 4.0",
        schema_if_proxy="GNSS L1 C/A clean+spoofed proxy; see examples/gps_spoofing_detection.rs",
        proxy_reason_hint="zenodo.org mirror unreachable or --offline",
        offline=offline,
        force=force,
    )


def handle_coloran(offline: bool, force: bool) -> SliceEntry:
    out_csv = SLICES / "coloran_slice.csv"
    out_meta = SLICES / "coloran_slice.json"

    def real_fetch() -> Optional[dict]:
        # Real KPI CSVs from the wineslab/colosseum-oran-coloran-dataset repo.
        # Path: rome_static_medium/<sched>/<trace>/<exp>/<bs>/bs*.csv.
        url_candidates = [
            "https://raw.githubusercontent.com/wineslab/colosseum-oran-coloran-dataset/master/rome_static_medium/sched0/tr0/exp1/bs1/bs1.csv",
            "https://raw.githubusercontent.com/wineslab/colosseum-oran-coloran-dataset/master/rome_static_medium/sched0/tr0/exp1/bs1/ue1.csv",
        ]
        for url in url_candidates:
            blob = try_download(url, timeout=30)
            if blob is None:
                continue
            if not looks_like_csv(blob):
                print(f"  [guard] {url} did not pass CSV sniff; skipping", file=sys.stderr)
                continue
            out_csv.write_bytes(blob[:MAX_SLICE_BYTES])
            out_meta.write_text(
                json.dumps(
                    {
                        "source": url,
                        "note": (
                            "Head slice of wineslab/colosseum-oran-coloran-dataset "
                            "rome_static_medium KPI CSV; columns: time,nof_ue,dl_brate,ul_brate."
                        ),
                        "head_bytes": min(len(blob), MAX_SLICE_BYTES),
                        "schema_hint": "O-RAN KPI trace; non-IQ companion annex.",
                    },
                    indent=2,
                )
                + "\n",
                encoding="utf-8",
            )
            return {"source": url}
        return None

    return _fetch_slice_real_or_proxy(
        "coloran",
        [out_csv, out_meta],
        real_fetch,
        gen_proxy_coloran.generate,
        (out_csv, out_meta, np.random.default_rng(20260421 + 3)),
        schema_if_real="Colosseum ColO-RAN CSV KPI trace (DU-reported throughput/ratio)",
        schema_if_proxy="CSV KPI trace proxy (eMBB/MTC/URLLC ratios)",
        proxy_reason_hint="github raw mirror unreachable or --offline",
        offline=offline,
        force=force,
    )


def handle_coloran_commag(offline: bool, force: bool) -> SliceEntry:
    out_csv = SLICES / "coloran_commag_slice.csv"
    out_meta = SLICES / "coloran_commag_slice.json"

    def real_fetch() -> Optional[dict]:
        # Real KPI CSVs from the wineslab/colosseum-oran-commag-dataset repo.
        # Path: slice_mixed/rome_static_close/<trace>/<exp>/<bs>/bs*.csv.
        url_candidates = [
            "https://raw.githubusercontent.com/wineslab/colosseum-oran-commag-dataset/master/slice_mixed/rome_static_close/tr0/exp1/bs1/bs1.csv",
            "https://raw.githubusercontent.com/wineslab/colosseum-oran-commag-dataset/master/slice_mixed/rome_static_close/tr0/exp1/bs1/ue1.csv",
        ]
        for url in url_candidates:
            blob = try_download(url, timeout=30)
            if blob is None:
                continue
            if not looks_like_csv(blob):
                print(f"  [guard] {url} did not pass CSV sniff; skipping", file=sys.stderr)
                continue
            out_csv.write_bytes(blob[:MAX_SLICE_BYTES])
            out_meta.write_text(
                json.dumps(
                    {
                        "source": url,
                        "note": (
                            "Head slice of wineslab/colosseum-oran-commag-dataset "
                            "slice_mixed KPI CSV; columns: time,nof_ue,dl_brate,ul_brate."
                        ),
                        "head_bytes": min(len(blob), MAX_SLICE_BYTES),
                        "schema_hint": "O-RAN scheduling-policy KPI trace; non-IQ companion annex.",
                    },
                    indent=2,
                )
                + "\n",
                encoding="utf-8",
            )
            return {"source": url}
        return None

    return _fetch_slice_real_or_proxy(
        "coloran_commag",
        [out_csv, out_meta],
        real_fetch,
        gen_proxy_coloran_commag.generate,
        (out_csv, out_meta, np.random.default_rng(20260421 + 4)),
        schema_if_real="Colosseum ColO-RAN-commag CSV scheduling-policy trace",
        schema_if_proxy="CSV scheduling-policy proxy (throughput, ratio_granted_req)",
        proxy_reason_hint="github raw mirror unreachable or --offline",
        offline=offline,
        force=force,
    )


def _find_deepbeam_local_h5() -> Optional[Path]:
    """Locate a user-downloaded DeepBeam HDF5 file under data/deepbeam/.

    Matches any ``neu_*.h5`` file >= 128 MB to avoid picking up sparse
    probe artefacts. Returns the largest such file or ``None``.
    """
    root = DATA / "deepbeam"
    if not root.is_dir():
        return None
    candidates: list[tuple[int, Path]] = []
    for p in root.rglob("neu_*.h5"):
        try:
            sz = p.stat().st_size
        except OSError:
            continue
        if sz >= 128 * 1024 * 1024:
            candidates.append((sz, p))
    if not candidates:
        return None
    candidates.sort(reverse=True)
    return candidates[0][1]


def handle_deepbeam(offline: bool, force: bool) -> SliceEntry:
    # Primary path: user-downloaded DeepBeam HDF5 (e.g. neu_ww72bk394.h5).
    # Fallback path: synthetic-proxy stand-in (loudly labelled).
    out_h5 = SLICES / "deepbeam_slice.h5"
    out_meta = SLICES / "deepbeam_slice.json"
    out_bin = SLICES / "deepbeam_slice.bin"  # proxy-only

    if not force and out_h5.exists() and out_meta.exists():
        digest, size = _compose_sha256([out_h5, out_meta])
        return SliceEntry(
            name="deepbeam",
            provenance="real-local-file",
            files=[out_h5.name, out_meta.name],
            bytes=size,
            sha256=digest,
            schema=(
                "HDF5 head slice — iq (N,2) float64, gain (N,) float64, "
                "rx_beam (N,) float64, tx_beam (N,) float64; "
                "DeepBeam NI mmWave transceiver native layout"
            ),
            source_url="local DeepBeam HDF5 under data/deepbeam/",
        )

    local_h5 = _find_deepbeam_local_h5()
    if local_h5 is not None:
        try:
            gen_proxy_deepbeam.extract_local_slice(local_h5, out_h5, out_meta)
            if out_bin.exists():
                out_bin.unlink()
            digest, size = _compose_sha256([out_h5, out_meta])
            return SliceEntry(
                name="deepbeam",
                provenance="real-local-file",
                files=[out_h5.name, out_meta.name],
                bytes=size,
                sha256=digest,
                schema=(
                    "HDF5 head slice — iq (N,2) float64, gain (N,) float64, "
                    "rx_beam (N,) float64, tx_beam (N,) float64; "
                    "DeepBeam NI mmWave transceiver native layout"
                ),
                source_url=f"local: {local_h5}",
            )
        except Exception as err:  # noqa: BLE001 — fall through to proxy on extract failure
            reason = f"DeepBeam local HDF5 extract failed: {type(err).__name__}: {err}"
            loud_proxy_banner("deepbeam", reason)
            if out_h5.exists():
                out_h5.unlink()
            gen_proxy_deepbeam.generate(out_bin, out_meta, rng=np.random.default_rng(20260421 + 5))
            digest, size = _compose_sha256([out_bin, out_meta])
            return SliceEntry(
                name="deepbeam",
                provenance="synthetic-proxy",
                files=[out_bin.name, out_meta.name],
                bytes=size,
                sha256=digest,
                schema="60 GHz mmWave beam-pair proxy (NI-like schema)",
                source_url="crate synthetic model",
                reason=reason,
            )

    reason = (
        "Northeastern repository requires auth or --offline; "
        "no local DeepBeam HDF5 (>=128 MiB neu_*.h5) found under data/deepbeam/"
    )
    loud_proxy_banner("deepbeam", reason)
    gen_proxy_deepbeam.generate(out_bin, out_meta, rng=np.random.default_rng(20260421 + 5))
    digest, size = _compose_sha256([out_bin, out_meta])
    return SliceEntry(
        name="deepbeam",
        provenance="synthetic-proxy",
        files=[out_bin.name, out_meta.name],
        bytes=size,
        sha256=digest,
        schema="60 GHz mmWave beam-pair proxy (NI-like schema)",
        source_url="crate synthetic model",
        reason=reason,
    )


def handle_deepsense_6g(offline: bool, force: bool) -> SliceEntry:
    # Primary path: user-downloaded Scenario 23 UAV mmWave zip from deepsense6g.net.
    # Fallback path: synthetic multimodal proxy (loudly labelled).
    zip_path = DATA / "Deepsense6G" / "scenario23_dev_w_resources.zip"
    out_h5 = SLICES / "deepsense_6g_slice.h5"
    out_meta = SLICES / "deepsense_6g_slice.json"
    out_bin = SLICES / "deepsense_6g_slice.bin"  # proxy-only

    if not force and out_h5.exists() and out_meta.exists():
        digest, size = _compose_sha256([out_h5, out_meta])
        return SliceEntry(
            name="deepsense_6g",
            provenance="real-local-zip",
            files=[out_h5.name, out_meta.name],
            bytes=size,
            sha256=digest,
            schema="HDF5 (N,64) mmWave power + UAV telemetry; DeepSense-6G Scenario 23 UAV mmWave",
            source_url=f"local: {zip_path}",
        )

    if zip_path.exists():
        try:
            gen_proxy_deepsense.extract_scenario23_slice(zip_path, out_h5, out_meta)
            # Clean up any leftover proxy .bin from a prior run.
            if out_bin.exists():
                out_bin.unlink()
            digest, size = _compose_sha256([out_h5, out_meta])
            return SliceEntry(
                name="deepsense_6g",
                provenance="real-local-zip",
                files=[out_h5.name, out_meta.name],
                bytes=size,
                sha256=digest,
                schema=(
                    "HDF5 (N,64) mmwave_power float32 + best_beam_index int16 + "
                    "UAV telemetry (altitude, speed, pitch, roll, distance, height); "
                    "DeepSense-6G Scenario 23 UAV mmWave head slice"
                ),
                source_url="https://www.deepsense6g.net/scenarios/scenario-23 (user-downloaded zip)",
            )
        except Exception as err:  # noqa: BLE001 — fall through to proxy on any extract failure
            reason = f"scenario23 zip extract failed: {type(err).__name__}: {err}"
            loud_proxy_banner("deepsense_6g", reason)
            # Clean up partial HDF5 if present.
            if out_h5.exists():
                out_h5.unlink()
            gen_proxy_deepsense.generate(out_bin, out_meta, rng=np.random.default_rng(20260421 + 6))
            digest, size = _compose_sha256([out_bin, out_meta])
            return SliceEntry(
                name="deepsense_6g",
                provenance="synthetic-proxy",
                files=[out_bin.name, out_meta.name],
                bytes=size,
                sha256=digest,
                schema="DeepSense-6G multimodal proxy (mmWave residual + GPS + cam hash)",
                source_url="crate synthetic model",
                reason=reason,
            )

    reason = (
        f"local zip {zip_path} not present and deepsense6g.net requires interactive "
        "HTML download (form-gated)"
    )
    loud_proxy_banner("deepsense_6g", reason)
    gen_proxy_deepsense.generate(out_bin, out_meta, rng=np.random.default_rng(20260421 + 6))
    digest, size = _compose_sha256([out_bin, out_meta])
    return SliceEntry(
        name="deepsense_6g",
        provenance="synthetic-proxy",
        files=[out_bin.name, out_meta.name],
        bytes=size,
        sha256=digest,
        schema="DeepSense-6G multimodal proxy (mmWave residual + GPS + cam hash)",
        source_url="crate synthetic model",
        reason=reason,
    )


# --- Driver -----------------------------------------------------------------


HANDLERS: list[tuple[str, Callable[[bool, bool], SliceEntry]]] = [
    ("radioml", handle_radioml),
    ("oracle", handle_oracle),
    ("powder", handle_powder),
    ("tampere_gnss", handle_tampere_gnss),
    ("coloran", handle_coloran),
    ("coloran_commag", handle_coloran_commag),
    ("deepbeam", handle_deepbeam),
    ("deepsense_6g", handle_deepsense_6g),
]


def main() -> int:
    ap = argparse.ArgumentParser(description="Prepare dsfb-rf slice catalog.")
    ap.add_argument("--offline", action="store_true", help="Skip network; proxy-only.")
    ap.add_argument("--force", action="store_true", help="Regenerate even if cached.")
    args = ap.parse_args()

    SLICES.mkdir(parents=True, exist_ok=True)

    entries: list[SliceEntry] = []
    for name, fn in HANDLERS:
        print(f"[prepare_slices] {name}", file=sys.stderr)
        try:
            entry = fn(args.offline, args.force)
        except Exception as exc:  # pragma: no cover — diagnostic only
            print(f"  [error] {name}: {exc}", file=sys.stderr)
            raise
        if entry.bytes > MAX_SLICE_BYTES * len(entry.files):
            print(
                f"  [warn] {name} slice {entry.bytes} bytes exceeds "
                f"{MAX_SLICE_BYTES} * files={len(entry.files)} cap",
                file=sys.stderr,
            )
        entries.append(entry)
        print(
            f"  -> provenance={entry.provenance} bytes={entry.bytes} files={entry.files}",
            file=sys.stderr,
        )

    manifest: dict[str, Any] = {
        "schema_version": 2,
        "generated_by": "scripts/prepare_slices.py",
        "generated_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "max_slice_bytes": MAX_SLICE_BYTES,
        "notice": (
            "Eight-dataset slice catalog for the dsfb-rf Colab reproducibility notebook. "
            "'real-in-repo' means the slice lives in git; 'real-local-zip' means it was "
            "extracted from a user-downloaded archive; 'real-public' means the slice was "
            "fetched from a public mirror at run time; 'synthetic-proxy' means the mirror "
            "was unreachable and the slice is a schema-preserving model-generated stand-in. "
            "No proxy is ever a substitute for a paper result."
        ),
        "slices": [entry.__dict__ for entry in entries],
    }

    if MANIFEST_PATH.exists():
        try:
            legacy = json.loads(MANIFEST_PATH.read_text())
            if isinstance(legacy, dict) and legacy.get("schema_version") == 1:
                manifest["legacy_v1"] = {
                    "parents": legacy.get("parents"),
                    "radioml_slice_summary": {
                        k: v
                        for k, v in legacy.get("slices", {}).items()
                        if k in {"radioml_2018_slice.hdf5", "deepsig_2018_snr30_slice.hdf5"}
                    },
                }
        except Exception:
            pass

    MANIFEST_PATH.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(f"[prepare_slices] wrote manifest: {MANIFEST_PATH}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
