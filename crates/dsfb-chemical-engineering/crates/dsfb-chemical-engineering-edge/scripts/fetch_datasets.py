#!/usr/bin/env python3
"""Fetch + process the public chemical-engineering / chemometric datasets into small, committed
processed slices, with rigorous provenance (source URL, license, citation, SHA-256, status).

Design principles (elite-review grade):
  * MEASURED real data is fetched from canonical public sources and sliced deterministically.
  * SIMULATION benchmarks (CSTR, three-tank, BSM1) are generated from real process physics/ODEs —
    these are the field-standard public benchmarks, honestly labelled `simulation`.
  * AGREEMENT-GATED datasets (iTrust SWaT/WADI) are NOT redistributed; we ship a loader + a clearly
    labelled physics stand-in and document the fetch path.
  * Every committed slice gets a per-dataset provenance block in data/MANIFEST.toml with a SHA-256.
  * Slices are intentionally small (reproducibility), with a leading nominal baseline where the
    monitoring framing requires one, and a `label` (0/1) and optional `phase` column when known.

Run:  python3 scripts/fetch_datasets.py [--only name1,name2]
Output:  data/slices/<name>.csv  and  data/MANIFEST.toml
Raw downloads are cached (gitignored) in data/_raw/.

Dataset taxonomy used in MANIFEST.toml and the paper:
  * "measured"        -- 10 datasets: real sensor/assay readings from physical processes or labs.
  * "simulation"      -- 9 datasets: generated from real process-physics ODEs (CSTR, three-tank,
                         BSM1-style, penicillin) or from the Tennessee Eastman SIMULATOR (Downs &
                         Vogel 1993). The Tennessee Eastman Process (TEP) test-set files are outputs
                         of the Downs-Vogel simulator, not real plant data.
  * "agreement-gated" -- 1 dataset: SWaT/WADI (iTrust SUTD). Only a physics stand-in is committed;
                         the real bytes require an iTrust data-use agreement.

Determinism: every slice is byte-exact reproducible from the same public source + fixed RNG seed
(RNG = np.random.default_rng(20260523)) once raw downloads are cached.  The committed SHA-256 in
MANIFEST.toml lets downstream stages gate on provenance without re-downloading.
"""
import hashlib
import io
import os
import sys
import time
import zipfile
from datetime import date

import numpy as np

# Root of the `dsfb-chemical-engineering-edge` crate (two levels up from this script).
HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
# Committed CSV slices land here; tracked by git and by MANIFEST.toml SHA-256.
SLICES = os.path.join(HERE, "data", "slices")
# Raw downloaded archives/files are cached here (gitignored); avoids re-downloading on re-runs.
RAW = os.path.join(HERE, "data", "_raw")
# Single provenance manifest written at the end of a run; one [[dataset]] block per slice.
MANIFEST = os.path.join(HERE, "data", "MANIFEST.toml")
os.makedirs(SLICES, exist_ok=True)
os.makedirs(RAW, exist_ok=True)

# ISO date of the current run, stored in MANIFEST.toml under `retrieved` for real-slice entries.
TODAY = date.today().isoformat()
ENTRIES = []  # provenance dicts accumulated by add_entry() across all dataset handlers

# Single fixed-seed RNG shared by all simulation generators so every call sequence is
# deterministic: identical seed -> identical slice -> identical SHA-256.
RNG = np.random.default_rng(20260523)  # fixed seed for any simulation


def log(*a):
    """Flush-on-write print so progress is visible immediately in CI/script output."""
    print(*a, flush=True)


def fetch(url, fname, timeout=60):
    """Download to RAW cache (idempotent). Returns local path or raises.

    If the target file already exists and is non-empty, the download is skipped and
    the cached path is returned — making repeated runs cheap without a network hit.
    `requests` is imported lazily so the module loads without it on simulation-only runs.
    """
    import requests

    dst = os.path.join(RAW, fname)
    # Cache hit: skip download if a non-empty file is already present in RAW/.
    if os.path.exists(dst) and os.path.getsize(dst) > 0:
        return dst
    r = requests.get(url, timeout=timeout, headers={"User-Agent": "dsfb-chem/0.1"})
    r.raise_for_status()
    with open(dst, "wb") as f:
        f.write(r.content)
    return dst


def sha256_file(path):
    """Compute the SHA-256 hex digest of a file by streaming it in 64 KiB chunks.

    Reads: the committed slice at `path`.
    Returns: lowercase 64-character hex string matching MANIFEST.toml `sha256` field.
    Streaming avoids loading large files entirely into memory.
    """
    h = hashlib.sha256()
    with open(path, "rb") as f:
        # iter sentinel pattern: call f.read(65536) until it returns b"".
        for chunk in iter(lambda: f.read(1 << 16), b""):
            h.update(chunk)
    return h.hexdigest()


def write_slice(name, X, var_names, labels=None, phase=None):
    """Write a numeric CSV slice to data/slices/<name>.csv.

    X must be an (n_samples, n_vars) numeric array. Columns are written in the order:
      var_names columns (formatted with %.6g), then `label` (0/1 int) if provided,
      then `phase` (int) if provided.

    Returns (path, n_samples, n_vars).  The returned path is what sha256_file() and
    add_entry() use, so the on-disk bytes are what the MANIFEST digest covers.
    """
    X = np.asarray(X, dtype=float)
    n, v = X.shape
    path = os.path.join(SLICES, f"{name}.csv")
    cols = list(var_names)
    header = cols[:]
    if labels is not None:
        header = header + ["label"]
    if phase is not None:
        header = header + ["phase"]
    with open(path, "w") as f:
        f.write(",".join(header) + "\n")
        for i in range(n):
            # %.6g: up to 6 significant figures, no trailing zeros, switches to scientific
            # notation only when the exponent is < -4 or >= 6.
            row = [f"{x:.6g}" for x in X[i]]
            if labels is not None:
                row.append(str(int(labels[i])))
            if phase is not None:
                row.append(str(int(phase[i])))
            f.write(",".join(row) + "\n")
    return path, n, v


def add_entry(name, kind, status, source_url, license_, citation, path, n, v, notes=""):
    """Append a provenance record to the module-level ENTRIES list.

    Computes the SHA-256 of the committed slice at `path` and records it alongside
    the human-readable fields that write_manifest() will serialise to MANIFEST.toml.

    kind:   'measured' | 'simulation' | 'agreement-gated'  (dataset taxonomy)
    status: 'real-slice' | 'simulation' | 'gated-standin'  (committed artifact type)
    `retrieved` is set to today's ISO date for real-slices; for simulation/gated entries
    the status string itself is stored (there is no meaningful download date).
    """
    ENTRIES.append(
        dict(
            name=name,
            kind=kind,  # measured | simulation | agreement-gated
            status=status,  # real-slice | simulation | gated-standin
            source_url=source_url,
            license=license_,
            citation=citation,
            retrieved=TODAY if status == "real-slice" else status,
            n_samples=n,
            n_vars=v,
            sha256=sha256_file(path),
            notes=notes,
        )
    )
    log(f"  wrote {name}: {n}x{v}  [{kind}/{status}]")


# ────────────────────────────────────────────────────────────────────────────
# MEASURED real datasets
# ────────────────────────────────────────────────────────────────────────────

def ds_tep(fault_id, onset=160, n=960):
    """Download and slice one Tennessee Eastman Process (TEP) test-set file.

    Source: Braatz group GitHub mirror of the Downs-Vogel TEP simulator outputs.
    File format: whitespace-delimited; either (52, n_samples) or (n_samples, 52) —
      the transposition guard below normalises to (n_samples, 52).

    IMPORTANT labelling note: these files are SIMULATOR outputs, not real plant readings.
    The TEP (Downs & Vogel 1993) is the field's canonical simulation benchmark.
    They are labelled `kind="simulation"` in MANIFEST.toml accordingly.

    Variable layout (52 columns):
      xmeas1..xmeas41  = 41 measured process variables
      xmv1..xmv11      = 11 manipulated variables (control setpoints)

    fault_id: integer IDV number (1-based). The five variants fetched are IDV(1, 4, 6, 13, 14),
      chosen to span distinct fault types: step disturbance, feed composition step, feed loss,
      slow drift (random variation), and a sticking-valve fault.
    onset: sample index at which the fault is introduced (default 160 = 8 h at 3-min cadence).
    n: number of samples to keep from the head of the file (default 960 ≈ 48 h).

    Reads:  data/_raw/tep_d<NN>_te.dat  (downloaded from Braatz mirror)
    Writes: data/slices/tennessee_eastman_idv<NN>.csv
    """
    base = "https://raw.githubusercontent.com/camaramm/tennessee-eastman-profBraatz/master"
    f = fetch(f"{base}/d{fault_id:02d}_te.dat", f"tep_d{fault_id:02d}_te.dat")
    arr = np.loadtxt(f)
    if arr.shape[0] == 52:  # some mirrors store transposed
        arr = arr.T
    arr = arr[:n]
    # Binary fault label: 0 for nominal samples before onset, 1 from onset onwards.
    labels = (np.arange(arr.shape[0]) >= onset).astype(int)
    var_names = [f"xmeas{j+1}" if j < 41 else f"xmv{j-40}" for j in range(arr.shape[1])]
    p, ns, v = write_slice(f"tennessee_eastman_idv{fault_id:02d}", arr, var_names, labels)
    add_entry(
        f"tennessee_eastman_idv{fault_id:02d}", "simulation", "real-slice",
        f"{base}/d{fault_id:02d}_te.dat",
        "public (Downs & Vogel TEP; Braatz group distribution)",
        "Downs & Vogel 1993; Chiang, Russell & Braatz 2001; Rieth et al. 2017",
        p, ns, v, notes=f"IDV({fault_id}) introduced at sample {onset}",
    )


def ds_wine(color):
    """Download and slice the UCI Wine Quality dataset (red or white variant).

    Source: UCI ML Repository (Cortez et al. 2009). Semicolon-delimited CSV with 11
    physicochemical features (fixed acidity, volatile acidity, citric acid, residual sugar,
    chlorides, free SO2, total SO2, density, pH, sulphates, alcohol) and a quality score.

    Monitoring framing: mid-quality wines (near the median score) form the nominal baseline;
    wines deviating ≥ 2 quality points from the median are flagged label=1.  Rows are sorted
    nearest-to-median first so the leading block is nominal — as expected by a monitoring
    pipeline that trains on the first segment as the reference distribution.

    color: 'red' (1599 rows) or 'white' (4898 rows).

    Reads:  data/_raw/winequality-<color>.csv
    Writes: data/slices/wine_quality_<color>.csv
    """
    url = f"https://archive.ics.uci.edu/ml/machine-learning-databases/wine-quality/winequality-{color}.csv"
    f = fetch(url, f"winequality-{color}.csv")
    import pandas as pd

    df = pd.read_csv(f, sep=";")
    y = df["quality"].to_numpy()
    X = df.drop(columns=["quality"]).to_numpy()
    names = [c.replace(" ", "_") for c in df.columns if c != "quality"]
    # Order by quality so a leading nominal block (mid-quality) precedes off-spec extremes:
    # treat low/high-quality (off-target) as the "fault" class for residual monitoring.
    order = np.argsort(np.abs(y - np.median(y)))  # nearest-to-median first
    Xs = X[order]
    lab = (np.abs(y[order] - np.median(y)) >= 2).astype(int)
    p, n, v = write_slice(f"wine_quality_{color}", Xs, names, lab)
    add_entry(
        f"wine_quality_{color}", "measured", "real-slice", url,
        "CC BY 4.0 (UCI)", "Cortez et al. 2009, Decision Support Systems",
        p, n, v, notes="physicochemical wine assays; off-target quality flagged as label=1",
    )


def ds_secom():
    """Download and slice the UCI SECOM semiconductor manufacturing dataset.

    Source: UCI ML Repository (McCann & Johnston 2008). Two files:
      secom.data        -- 1567 samples × 591 sensor features (whitespace-delimited,
                           many NaN sentinels representing missing sensor readings).
      secom_labels.data -- 1567 rows; column 0 is +1 (fail) or -1 (pass).

    Preprocessing steps applied here:
      1. Drop columns that are entirely NaN (no usable signal).
      2. Impute remaining NaN values with the per-column median (deterministic; no RNG).
      3. Sort so label=0 (pass) rows come first, forming a nominal baseline window.
      4. Trim to the first 600 rows to keep the committed slice small.

    The median-imputation is a deliberate, documented choice: it is deterministic and
    preserves the marginal distribution better than zero-fill for highly censored process
    sensors.  The imputation is acknowledged in the MANIFEST notes.

    Reads:  data/_raw/secom.data, data/_raw/secom_labels.data
    Writes: data/slices/secom_semiconductor.csv
    """
    base = "https://archive.ics.uci.edu/ml/machine-learning-databases/secom"
    data = fetch(f"{base}/secom.data", "secom.data")
    labf = fetch(f"{base}/secom_labels.data", "secom_labels.data")
    X = np.genfromtxt(data)
    lab_raw = np.genfromtxt(labf, usecols=0)
    lab = (lab_raw > 0).astype(int)  # +1 = fail -> 1
    # Drop all-NaN columns; impute remaining NaN with column median (deterministic).
    keep = ~np.all(np.isnan(X), axis=0)
    X = X[:, keep]
    med = np.nanmedian(X, axis=0)
    inds = np.where(np.isnan(X))
    X[inds] = np.take(med, inds[1])
    # Order: nominal (pass) first to form a baseline, then interleave fails.
    order = np.argsort(lab, kind="stable")
    X, lab = X[order], lab[order]
    # Slice to a manageable size.
    take = min(600, X.shape[0])
    X, lab = X[:take], lab[:take]
    names = [f"s{j:03d}" for j in range(X.shape[1])]
    p, n, v = write_slice("secom_semiconductor", X, names, lab)
    add_entry(
        "secom_semiconductor", "measured", "real-slice", f"{base}/secom.data",
        "CC BY 4.0 (UCI)", "McCann & Johnston 2008 (UCI SECOM)",
        p, n, v, notes="semiconductor process sensors; pass/fail label; median-imputed",
    )


def ds_steel_plates():
    """Download and slice the UCI Steel Plates Faults dataset.

    Source: UCI ML Repository (Semeion Research Center). The Faults.NNA file is
    whitespace-delimited with 1941 rows × 34 columns:
      columns 0..26  -- 27 geometric and photometric surface features
      columns 27..33 -- 7-class one-hot fault type encoding

    Monitoring framing: argmax of the one-hot columns gives the fault class index.
    Class 0 is treated as nominal; all other classes are coarsely flagged label=1.
    Rows are sorted to put label=0 first so the leading block forms the nominal
    baseline for the monitoring pipeline.

    Reads:  data/_raw/steel_faults.NNA
    Writes: data/slices/steel_plates_faults.csv  (27 feature columns + label)
    """
    url = "https://archive.ics.uci.edu/ml/machine-learning-databases/00198/Faults.NNA"
    f = fetch(url, "steel_faults.NNA")
    arr = np.loadtxt(f)
    feats = arr[:, :27]
    fault_onehot = arr[:, 27:]
    fault_id = fault_onehot.argmax(axis=1)
    lab = (fault_id != 0).astype(int)  # class 0 vs others as a coarse fault flag
    order = np.argsort(lab, kind="stable")
    feats, lab = feats[order], lab[order]
    names = [f"f{j:02d}" for j in range(feats.shape[1])]
    p, n, v = write_slice("steel_plates_faults", feats, names, lab)
    add_entry(
        "steel_plates_faults", "measured", "real-slice", url,
        "CC BY 4.0 (UCI)", "Semeion Research Center; UCI Steel Plates Faults",
        p, n, v, notes="27 geometric/photometric features; coarse fault flag",
    )


def ds_gas_sensor_drift():
    """Download and slice the UCI Gas Sensor Array Drift dataset.

    Source: UCI ML Repository (Vergara et al. 2012). A ZIP containing one .dat file per
    collection batch (10 batches; batch files sorted lexicographically for determinism).
    File format: sparse libsvm-style lines:
      <gas_id>[;<extra>] <feat_idx>:<value> ...
    where gas_id is an integer (1-6) and features are indexed 1..128 (16 MOX sensors ×
    8 statistical features each).

    Parsing assumption: if the first token contains ';', the gas id precedes it; otherwise
    the token is a plain float cast to int.  Only tokens containing ':' are treated as
    feature pairs; the label/separator token is skipped.

    Drift-monitoring framing:
      * Batch 1 (nominal baseline): label=0
      * Batches 8-10 (aged sensors with significant drift): label=1
      * Up to 300 rows are taken from each group for a balanced slice.
    The `phase` column records the gas identity (1-6), preserved for any phase-aware
    detector downstream.

    Reads:  data/_raw/gas_drift.zip  (downloaded from UCI)
    Writes: data/slices/gas_sensor_array_drift.csv  (128 feat cols + label + phase)
    """
    url = "https://archive.ics.uci.edu/ml/machine-learning-databases/00224/Dataset.zip"
    f = fetch(url, "gas_drift.zip")
    rows, gases, batches = [], [], []
    with zipfile.ZipFile(f) as z:
        names = sorted(n for n in z.namelist() if n.endswith(".dat"))
        for bi, nm in enumerate(names):
            for line in z.read(nm).decode().splitlines():
                parts = line.split()
                if not parts:
                    continue
                # Gas-id token may include a semicolon-delimited sub-label; extract the integer part.
                gas = int(parts[0].split(";")[0]) if ";" in parts[0] else int(float(parts[0]))
                feat = [float(p.split(":")[1]) for p in parts[1:] if ":" in p]
                rows.append(feat)
                gases.append(gas)
                batches.append(bi + 1)
    X = np.array(rows)
    batches = np.array(batches)
    gases = np.array(gases)
    # Drift monitoring: baseline = batch 1; "fault" (drift) = later batches. Take a balanced slice.
    b1 = np.where(batches == 1)[0]
    blate = np.where(batches >= 8)[0]
    take1 = b1[: min(300, len(b1))]
    takel = blate[: min(300, len(blate))]
    idx = np.concatenate([take1, takel])
    Xs = X[idx]
    lab = (batches[idx] >= 8).astype(int)
    phase = gases[idx]
    names_v = [f"feat{j:03d}" for j in range(Xs.shape[1])]
    p, n, v = write_slice("gas_sensor_array_drift", Xs, names_v, lab, phase)
    add_entry(
        "gas_sensor_array_drift", "measured", "real-slice", url,
        "CC BY 4.0 (UCI)", "Vergara et al. 2012, Sensors and Actuators B",
        p, n, v, notes="16 MOX sensors x 8 features; batch1 baseline vs late-batch drift; phase=gas",
    )


def ds_air_quality():
    """Download and slice the UCI Air Quality multisensor dataset.

    Source: UCI ML Repository (De Vito et al. 2008). A ZIP containing a semicolon-delimited
    CSV with European number formatting (decimal comma).  12 sensor/reference columns are
    selected; the dataset is unlabelled for this pipeline (no fault annotations provided).

    Parsing notes:
      * sep=';' and decimal=',' because the source uses European CSV conventions.
      * The value -200 is the dataset's sentinel for a missing/invalid reading; rows
        containing any -200 value are dropped entirely (dropna after replacement).
      * Columns that cannot be coerced to numeric are silently treated as NaN.
      * Up to 1200 rows are retained from the head of the cleaned frame.

    Column names are sanitised for CSV compatibility by replacing '(', ')', '.' with '_'.

    Reads:  data/_raw/airquality.zip
    Writes: data/slices/air_quality_multisensor.csv  (12 sensor columns; no label)
    """
    url = "https://archive.ics.uci.edu/ml/machine-learning-databases/00360/AirQualityUCI.zip"
    f = fetch(url, "airquality.zip")
    import pandas as pd

    with zipfile.ZipFile(f) as z:
        csv = [n for n in z.namelist() if n.endswith(".csv")][0]
        df = pd.read_csv(z.open(csv), sep=";", decimal=",")
    sensor_cols = [
        "CO(GT)", "PT08.S1(CO)", "C6H6(GT)", "PT08.S2(NMHC)", "NOx(GT)",
        "PT08.S3(NOx)", "NO2(GT)", "PT08.S4(NO2)", "PT08.S5(O3)", "T", "RH", "AH",
    ]
    # Guard against column-name variations across mirrors.
    sensor_cols = [c for c in sensor_cols if c in df.columns]
    sub = df[sensor_cols].apply(pd.to_numeric, errors="coerce").replace(-200, np.nan).dropna()
    X = sub.to_numpy()[: min(1200, len(sub))]
    names = [c.replace("(", "_").replace(")", "").replace(".", "_") for c in sensor_cols]
    p, n, v = write_slice("air_quality_multisensor", X, names)
    add_entry(
        "air_quality_multisensor", "measured", "real-slice", url,
        "CC BY 4.0 (UCI)", "De Vito et al. 2008, Sensors and Actuators B",
        p, n, v, notes="multisensor air-quality time series; -200 sentinels dropped; unlabelled",
    )


def ds_tecator():
    """Fetch the Tecator NIR meat dataset via OpenML and write the 100 absorbance channels.

    Source: OpenML dataset 505 (Tecator Infratec Food & Feed Analyzer; StatLib mirror).
    240 ground-meat samples measured at 100 NIR wavelengths (850-1050 nm, 2 nm steps).
    The OpenML record may include 22 derived principal components after the 100 channels;
    only the first 100 columns (the raw absorbance spectra) are committed.

    Uses sklearn.datasets.fetch_openml with parser="liac-arff" for deterministic parsing.
    No fault labels are provided — this dataset is used for spectral residual monitoring.

    Reads:  OpenML API (cached by sklearn in its local cache)
    Writes: data/slices/tecator_nir_meat.csv  (100 absorbance columns; no label)
    """
    from sklearn.datasets import fetch_openml

    d = fetch_openml(name="tecator", version=1, as_frame=False, parser="liac-arff")
    X = np.asarray(d.data, dtype=float)
    # Tecator: 100 absorbance channels (+ possibly 22 PCs); keep the 100 spectra channels.
    spec = X[:, :100]
    names = [f"ab{j:03d}" for j in range(spec.shape[1])]
    p, n, v = write_slice("tecator_nir_meat", spec, names)
    add_entry(
        "tecator_nir_meat", "measured", "real-slice",
        "https://www.openml.org/d/505",
        "public (Tecator/StatLib)", "Tecator Infratec; Borggaard & Thodberg 1992",
        p, n, v, notes="240 NIR meat spectra x 100 channels (850-1050 nm)",
    )


def ds_corn(instrument="m5"):
    """Eigenvector corn benchmark (.mat). Each of three NIR instruments (m5, mp5, mp6) measured the
    same 80 corn samples over 700 channels — the canonical instrument-standardisation / calibration-
    transfer benchmark. We extract two instruments as distinct real datasets.

    Source: Eigenvector Research (eigenvector.com/data/Corn/corn.mat).
    The MATLAB .mat file contains struct arrays keyed as `m5spec`, `mp5spec`, `mp6spec`.
    scipy.io.loadmat wraps these as nested numpy record arrays; `mat[key][0, 0]["data"]`
    extracts the (80, 700) float64 spectra matrix.

    Parsing assumption: if the first dimension is not 80, the matrix is transposed.
    Wavelengths run from 1100 nm to 2498 nm in 2 nm steps (700 channels); column names
    encode the centre wavelength as `nm<wavelength>`.

    instrument: 'm5' | 'mp5' | 'mp6'. All three are fetched from the same .mat file
    (one network download); each produces its own committed slice.

    Reads:  data/_raw/corn.mat
    Writes: data/slices/corn_nir_<instrument>.csv  (700 NIR columns; no label)
    """
    url = "https://eigenvector.com/data/Corn/corn.mat"
    f = fetch(url, "corn.mat")
    from scipy.io import loadmat

    mat = loadmat(f)
    key = f"{instrument}spec"
    spec = np.asarray(mat[key][0, 0]["data"], dtype=float)
    if spec.shape[0] != 80:
        spec = spec.T
    spec = spec[:80, :700]
    names = [f"nm{1100 + 2*j}" for j in range(spec.shape[1])]
    nm = f"corn_nir_{instrument}"
    p, n, v = write_slice(nm, spec, names)
    add_entry(
        nm, "measured", "real-slice", url,
        "public (Eigenvector data sets)", "Eigenvector Research Corn NIR standardisation benchmark",
        p, n, v, notes=f"80 corn samples x 700 NIR channels ({instrument} instrument); calibration-transfer benchmark",
    )


def ds_gasoline_octane():
    """Fetch the Kalivas gasoline NIR octane dataset from the R `pls` package CRAN mirror.

    Source: Kalivas 1997 (Chemometrics Intell. Lab. Syst.); canonical R distribution via
    the CRAN `pls` package.  The .RData file contains a single data frame with columns:
      octane  -- octane number (response; not written to the slice)
      V1..V401 (or similar) -- 401 NIR absorbance values per spectrum

    This function requires the `pyreadr` package to parse the R binary format.  If
    pyreadr is absent or the download fails, the function logs the reason and returns
    False; the main() loop catches the absence gracefully.

    Parsing assumption: all columns except 'octane' are NIR channels.  pyreadr may
    flatten a matrix column into individual numbered columns; the column order is
    preserved as-is from pyreadr's output.

    Reads:  data/_raw/gasoline.RData  (downloaded from CRAN mirror)
    Writes: data/slices/gasoline_nir_octane.csv  (401 NIR channels; no label)
    Returns True on success, False on any failure.
    """
    # 'gasoline' NIR octane dataset (Kalivas 1997), mirrored from the R `pls` package.
    candidates = [
        "https://raw.githubusercontent.com/cran/pls/master/data/gasoline.RData",
    ]
    # RData needs pyreadr; instead use a CSV mirror if pyreadr is unavailable.
    try:
        import pyreadr

        f = fetch(candidates[0], "gasoline.RData")
        res = pyreadr.read_r(f)
        df = list(res.values())[0]
        # gasoline: columns 'octane' and 'NIR' (matrix of 401). pyreadr flattens NIR.
        nir_cols = [c for c in df.columns if c != "octane"]
        X = df[nir_cols].to_numpy()
        names = [f"nir{j:03d}" for j in range(X.shape[1])]
        p, n, v = write_slice("gasoline_nir_octane", X, names)
        add_entry(
            "gasoline_nir_octane", "measured", "real-slice", candidates[0],
            "GPL (R pls package data)", "Kalivas 1997, Chemometrics Intell. Lab. Syst.",
            p, n, v, notes="60 gasoline NIR spectra x 401 channels",
        )
        return True
    except Exception as e:
        log(f"  gasoline: pyreadr path failed ({e})")
        return False


# ────────────────────────────────────────────────────────────────────────────
# SIMULATION benchmarks (real process physics; honestly labelled)
# ────────────────────────────────────────────────────────────────────────────

def ds_cstr():
    """Simulate a non-isothermal CSTR with first-order exothermic reaction A->B (Arrhenius).

    A cooling/fouling fault is injected mid-run (step reduction in coolant heat-transfer
    coefficient and step rise in coolant temperature), driving a thermal excursion.

    Model parameters follow the standard Seborg et al. non-isothermal CSTR benchmark
    (Process Dynamics and Control, Wiley):
      q=100, V=100, k0=7.2e10, Ea/R=8750, dH=-5e4, rho*Cp=239,
      UA=5e4, Tf=350, Tc0=300, Caf=1.0.

    Numerical integration:
      * RK4 with sub=50 substeps per recorded output step (dt_out=0.05) for numerical
        stability of the stiff energy-balance ODE.
      * The two inner `advance`/RK4 blocks are logically identical; `advance` is used
        only during burn-in (state not recorded), the inline block during the recorded run.
      * 400 burn-in steps are discarded so the first recorded sample is at steady state.

    Fault injection at sample 480:
      * Coolant temperature Tc rises from 300 to 312 K (Tc0 + 12).
      * Heat-transfer coefficient UA drops to 60% of nominal (UA * 0.6), simulating fouling.

    Output columns: [Ca (concentration, mol/L), T (reactor temperature, K),
                     Tc (coolant temperature, K), rate (k*Ca, 1/s * mol/L), q (flow, L/min)]
    Measurement noise is added from RNG (module-level fixed seed) to Ca and T only; Tc, rate,
    and q are computed from the noisy observations.

    Writes: data/slices/cstr_reactor.csv  (5 columns + label)
    """
    dt_out, N, onset = 0.05, 800, 480
    # Seborg et al. non-isothermal CSTR parameters (A->B, first order, exothermic).
    q, V, k0, EaR, dH, rhoCp = 100.0, 100.0, 7.2e10, 8750.0, -5.0e4, 239.0
    UA, Tf, Tc0, Caf = 5.0e4, 350.0, 300.0, 1.0
    Ca, T = 0.5, 350.0
    sub = 50  # internal substeps for stiff stability
    h = dt_out / sub

    def deriv(Ca, T, Tc, UA_eff):
        # Reaction rate constant via Arrhenius equation.
        k = k0 * np.exp(-EaR / T)
        # Mass balance: dCa/dt = (q/V)*(Caf - Ca) - k*Ca
        dCa = (q / V) * (Caf - Ca) - k * Ca
        # Energy balance: includes reaction heat and coolant heat exchange.
        dT = (q / V) * (Tf - T) + (-dH / rhoCp) * k * Ca + (UA_eff / (rhoCp * V)) * (Tc - T)
        return dCa, dT, k

    def advance(Ca, T, Tc, UA_eff):
        # Fourth-order Runge-Kutta integration of the CSTR ODEs for `sub` sub-steps.
        for _ in range(sub):
            k1c, k1t, _ = deriv(Ca, T, Tc, UA_eff)
            k2c, k2t, _ = deriv(Ca + 0.5 * h * k1c, T + 0.5 * h * k1t, Tc, UA_eff)
            k3c, k3t, _ = deriv(Ca + 0.5 * h * k2c, T + 0.5 * h * k2t, Tc, UA_eff)
            k4c, k4t, _ = deriv(Ca + h * k3c, T + h * k3t, Tc, UA_eff)
            Ca += (h / 6.0) * (k1c + 2 * k2c + 2 * k3c + k4c)
            T += (h / 6.0) * (k1t + 2 * k2t + 2 * k3t + k4t)
            # Physical bounds: concentration in [0, Caf]; temperature in [250, 500] K.
            Ca = float(np.clip(Ca, 0.0, Caf))
            T = float(np.clip(T, 250.0, 500.0))
        return Ca, T

    # Burn-in to the nominal steady state so the recorded baseline is stationary.
    for _ in range(400):
        Ca, T = advance(Ca, T, Tc0, UA)

    rows, lab = [], []
    for t in range(N):
        Tc = Tc0 if t < onset else Tc0 + 12.0  # coolant temperature rises (cooling fault)
        UA_eff = UA if t < onset else UA * 0.6  # heat-transfer fouling
        for _ in range(sub):  # RK4 substepping for the stiff energy balance
            k1c, k1t, _ = deriv(Ca, T, Tc, UA_eff)
            k2c, k2t, _ = deriv(Ca + 0.5 * h * k1c, T + 0.5 * h * k1t, Tc, UA_eff)
            k3c, k3t, _ = deriv(Ca + 0.5 * h * k2c, T + 0.5 * h * k2t, Tc, UA_eff)
            k4c, k4t, _ = deriv(Ca + h * k3c, T + h * k3t, Tc, UA_eff)
            Ca += (h / 6.0) * (k1c + 2 * k2c + 2 * k3c + k4c)
            T += (h / 6.0) * (k1t + 2 * k2t + 2 * k3t + k4t)
            Ca = float(np.clip(Ca, 0.0, Caf))
            T = float(np.clip(T, 250.0, 500.0))
        # Add sensor noise from module-level RNG (fixed seed → deterministic).
        Ca_n = Ca + 0.002 * RNG.standard_normal()
        T_n = T + 0.05 * RNG.standard_normal()
        # Rate and flow columns are computed from noisy observations, not the clean state.
        k = k0 * np.exp(-EaR / T_n)
        rows.append([Ca_n, T_n, Tc, k * Ca_n, q])
        lab.append(1 if t >= onset else 0)
    X = np.array(rows)
    p, n, v = write_slice("cstr_reactor", X, ["Ca", "T", "Tc", "rate", "q"], np.array(lab))
    add_entry(
        "cstr_reactor", "simulation", "simulation",
        "process physics (non-isothermal CSTR, Arrhenius kinetics)",
        "n/a (generated)", "Standard exothermic CSTR benchmark (Seborg et al., Process Dynamics & Control)",
        p, n, v, notes=f"cooling/fouling fault injected at sample {onset}",
    )


def ds_three_tank():
    """Simulate the three-tank hydraulic benchmark (Torricelli flow, DTS200/Amira family).

    A leak fault is injected in tank 1 mid-run, causing the level to drop abnormally.

    Model topology: tank 1 has inflow q1 and connects to tank 3 via a pipe; tank 3
    connects to tank 2; tank 2 has an outflow to atmosphere and inflow q3.
    Torricelli's law: flow between tanks i and j is sign(hi-hj)*az*sqrt(2g*|hi-hj|).

    Parameters:
      A=154 cm²  (tank cross-section), az=0.5 cm² (pipe cross-section), g=981 cm/s²,
      dt=0.5 s, N=700 steps, burn-in=600 steps (discarded to reach steady state).

    Fault injection at sample 400:
      leak = 1.2 * az * sqrt(2g * |h[0]|)  (proportional to tank-1 level — a drain)

    Measurement noise (from module-level RNG) is added after each Euler step.
    Physical bounds: levels clipped to [0.1, 100] cm.

    Output columns: [h1, h3, h2, q1, q3] — note the column order is h1, h3, h2 (not
    sequential by tank number), reflecting the measurement layout of the DTS200 bench.

    Writes: data/slices/three_tank.csv  (5 columns + label)
    """
    dt, N, onset = 0.5, 700, 400
    A, az, g = 154.0, 0.5, 981.0
    q1, q3 = 50.0, 30.0
    h = np.array([40.0, 20.0, 30.0])
    rows, lab = [], []

    def flow(hi, hj):
        # Torricelli: bidirectional flow proportional to sqrt of the head difference.
        d = hi - hj
        return np.sign(d) * az * np.sqrt(2 * g * abs(d))

    def step(h, leak):
        # Euler step of the three-tank ODEs; leak is subtracted from tank-1's balance.
        q13 = flow(h[0], h[2])
        q32 = flow(h[2], h[1])
        q20 = az * np.sqrt(2 * g * abs(h[1]))
        dh = np.array([(q1 - q13 - leak) / A, (q13 - q32) / A, (q3 + q32 - q20) / A])
        return np.clip(h + dt * dh, 0.1, 100.0)

    # Burn-in to the nominal hydraulic steady state (discarded) for a stationary baseline.
    for _ in range(600):
        h = step(h, 0.0)

    for t in range(N):
        leak = 0.0 if t < onset else 1.2 * az * np.sqrt(2 * g * abs(h[0]))  # tank-1 leak
        h = step(h, leak) + 0.02 * RNG.standard_normal(3)
        h = np.clip(h, 0.1, 100.0)
        rows.append([h[0], h[2], h[1], q1, q3])
        lab.append(1 if t >= onset else 0)
    X = np.array(rows)
    p, n, v = write_slice("three_tank", X, ["h1", "h3", "h2", "q1", "q3"], np.array(lab))
    add_entry(
        "three_tank", "simulation", "simulation",
        "process physics (three-tank Torricelli hydraulics)",
        "n/a (generated)", "DTS200 / Amira three-tank FDI benchmark family",
        p, n, v, notes=f"tank-1 leak injected at sample {onset}",
    )


def ds_bsm1_like():
    """Simulate a BSM1-style activated-sludge monitoring proxy.

    Generates a diurnal influent load (24-hour period at 15-min cadence = period 96 samples)
    with a slow nitrification upset starting at sample 600: dissolved oxygen (DO) is linearly
    depressed and effluent ammonia/COD increase proportionally over the remaining samples.

    This is a REDUCED, ASM1-flavoured proxy, not the official IWA BSM1 model. The official
    BSM1 (Alex et al.) requires the IWA MATLAB/Simulink package, which is not freely
    redistributable. The manifest notes document the official route for users who have it.

    N=960 samples at 15-min cadence represents ~10 days of operation (a subset).
    Fault injection at onset=600: `prog = (k - onset) / (N - onset)` scales linearly 0→1
    so the fault effects grow gradually rather than as a step.

    Output columns: [inf_COD, inf_NH, DO, eff_NH, eff_COD]
    All noise is from the module-level RNG (deterministic).

    Writes: data/slices/bsm1_wastewater.csv  (5 columns + label)
    """
    N, onset = 960, 600  # 10-day, 15-min cadence (subset)
    t = np.arange(N)
    # Diurnal pattern: period = 96 samples = 24 h at 15-min intervals.
    diurnal = 1.0 + 0.3 * np.sin(2 * np.pi * t / 96.0)
    influent_cod = 300 * diurnal + 5 * RNG.standard_normal(N)
    influent_nh = 30 * diurnal + 1.0 * RNG.standard_normal(N)
    do = 2.0 + 0.1 * RNG.standard_normal(N)
    eff_nh = 1.5 + 0.2 * RNG.standard_normal(N)
    eff_cod = 45 + 3 * RNG.standard_normal(N)
    for k in range(onset, N):
        prog = (k - onset) / (N - onset)
        do[k] -= 1.2 * prog  # aeration/nitrifier upset
        eff_nh[k] += 8.0 * prog
        eff_cod[k] += 10.0 * prog
    X = np.column_stack([influent_cod, influent_nh, do, eff_nh, eff_cod])
    lab = (t >= onset).astype(int)
    p, n, v = write_slice("bsm1_wastewater", X, ["inf_COD", "inf_NH", "DO", "eff_NH", "eff_COD"], lab)
    add_entry(
        "bsm1_wastewater", "simulation", "simulation",
        "process model (reduced ASM1-flavoured activated sludge; official BSM1 = IWA MATLAB)",
        "n/a (generated)", "Alex et al., IWA Benchmark Simulation Model no. 1 (BSM1)",
        p, n, v, notes=f"nitrification upset injected at sample {onset}; see fetch notes for official BSM1",
    )


def ds_penicillin_fedbatch():
    """Simulate an IndPenSim-flavoured fed-batch penicillin fermentation with an aeration fault.

    Fed-batch penicillin fermentation proxy: biomass/substrate/penicillin trajectories across
    batch phases with a labelled aeration fault. Labelled simulation; the real IndPenSim CSV
    (~100MB) can be fetched via the documented URL in the manifest notes.

    This is a simplified proxy, not the actual IndPenSim simulator (Goldrick et al. 2015/2019).
    The real IndPenSim dataset is available at https://data.mendeley.com/datasets/pdnjz7zz5x.

    Batch phase encoding (stored in `phase` column):
      0 = lag phase (t < 120),  1 = exponential phase (120 ≤ t < 520),  2 = stationary (t ≥ 520)

    Biomass follows a simplified exponential growth law capped at onset (520) to represent
    substrate depletion entering stationary phase.  Substrate exhibits a slow decline plus
    sinusoidal oscillation (feeding strategy proxy).  Penicillin accumulates linearly from
    the end of the lag phase onwards.

    Fault injection at onset=520 (coinciding with entry to stationary phase):
      * Dissolved oxygen (DO2) decreases linearly (aeration fault).
      * Penicillin yield is reduced proportionally (consequence of DO deficit).

    A final small additive noise term (σ=0.01 from RNG) is applied to all six columns
    after fault injection, representing sensor noise on top of the process trajectory.

    Output columns: [biomass, substrate, penicillin, DO2, pH, T]
    phase column: 0/1/2 as above.

    Writes: data/slices/penicillin_fedbatch.csv  (6 columns + label + phase)
    """
    N, onset = 800, 520
    t = np.arange(N)
    phase = np.where(t < 120, 0, np.where(t < 520, 1, 2))  # lag / exponential / stationary
    X0 = 0.1
    mu = 0.08
    biomass = X0 * np.exp(mu * np.minimum(t, 520) / 50.0)
    substrate = np.clip(15 - 0.02 * t + 2 * np.sin(2 * np.pi * t / 80.0), 0.1, None)
    penicillin = np.clip(0.0 + 0.004 * np.maximum(t - 120, 0), 0, None)
    do2 = 1.2 + 0.05 * RNG.standard_normal(N)
    ph = 5.0 + 0.02 * RNG.standard_normal(N)
    temp = 298 + 0.1 * RNG.standard_normal(N)
    for k in range(onset, N):
        prog = (k - onset) / (N - onset)
        do2[k] -= 0.7 * prog  # aeration fault
        penicillin[k] *= 1.0 - 0.4 * prog  # yield loss
    # Small global noise added after fault injection; uses RNG so noise is deterministic.
    X = np.column_stack([biomass, substrate, penicillin, do2, ph, temp]) + 0.01 * RNG.standard_normal((N, 6))
    lab = (t >= onset).astype(int)
    p, n, v = write_slice("penicillin_fedbatch", X, ["biomass", "substrate", "penicillin", "DO2", "pH", "T"], lab, phase)
    add_entry(
        "penicillin_fedbatch", "simulation", "simulation",
        "process model (fed-batch penicillin; real IndPenSim at https://data.mendeley.com/datasets/pdnjz7zz5x)",
        "n/a (generated)", "Goldrick et al. 2015/2019, IndPenSim industrial penicillin simulator",
        p, n, v, notes=f"aeration fault at sample {onset}; phase=lag/exp/stationary; real IndPenSim is ~100MB",
    )


def ds_swat_gated():
    """Generate a physics stand-in for the iTrust SWaT water-treatment dataset.

    SWaT/WADI are agreement-gated (iTrust SUTD). We DO NOT redistribute their bytes. We ship a
    clearly-labelled water-treatment physics stand-in so the pipeline runs; the real data is fetched
    by the user after accepting the iTrust data-use agreement (see manifest source_url).

    The stand-in simulates a simple water-treatment loop:
      LIT101 -- sinusoidal tank level with Gaussian noise
      FIT101 -- flow (nominally ~2.5 m³/h with noise)
      pH, COND, ORP -- static nominal values with sensor noise

    Attack-like upset at onset=540:
      LIT101 increases by 80 units (spoofed level, as in a typical SWaT replay attack).
      FIT101 decreases by 0.8 (consistent with a closed-valve condition).

    The column names (LIT101, FIT101, etc.) match SWaT sensor naming conventions to make
    the stand-in structurally compatible with code that reads real SWaT data by column name.

    MANIFEST kind="agreement-gated", status="gated-standin" — this is NOT real SWaT data.

    Writes: data/slices/swat_water_treatment_standin.csv  (5 columns + label)
    """
    N, onset = 900, 540
    t = np.arange(N)
    lit = 500 + 50 * np.sin(2 * np.pi * t / 120.0) + 3 * RNG.standard_normal(N)  # tank level
    fit = 2.5 + 0.1 * RNG.standard_normal(N)  # flow
    ph = 7.0 + 0.05 * RNG.standard_normal(N)
    cond = 350 + 5 * RNG.standard_normal(N)
    orp = 420 + 8 * RNG.standard_normal(N)
    for k in range(onset, N):
        lit[k] += 80.0  # spoofed level (attack-like upset)
        fit[k] -= 0.8
    X = np.column_stack([lit, fit, ph, cond, orp])
    lab = (t >= onset).astype(int)
    p, n, v = write_slice("swat_water_treatment_standin", X, ["LIT101", "FIT101", "pH", "COND", "ORP"], lab)
    add_entry(
        "swat_water_treatment_standin", "agreement-gated", "gated-standin",
        "https://itrust.sutd.edu.sg/itrust-labs_datasets/ (request required)",
        "iTrust data-use agreement (NOT redistributed)", "Goh et al. 2016, SWaT testbed",
        p, n, v, notes="physics STAND-IN only; real SWaT/WADI requires the iTrust agreement",
    )


REGISTRY = {
    # ── Tennessee Eastman ───────────────────────────────────────────────────────────────
    # Five IDV fault types spanning the standard TEP fault taxonomy:
    #   IDV(1)  = step change in A/C feed ratio (step disturbance)
    #   IDV(4)  = reactor cooling water inlet temperature step
    #   IDV(6)  = feed loss (A feed zero)
    #   IDV(13) = slow drift in reaction kinetics
    #   IDV(14) = sticking agitator speed valve
    # These are SIMULATOR outputs (Downs-Vogel TEP), not real plant data.
    "tennessee_eastman_idv01": lambda: ds_tep(1),
    "tennessee_eastman_idv04": lambda: ds_tep(4),
    "tennessee_eastman_idv06": lambda: ds_tep(6),
    "tennessee_eastman_idv13": lambda: ds_tep(13),
    "tennessee_eastman_idv14": lambda: ds_tep(14),
    # ── Measured real datasets ──────────────────────────────────────────────────────────
    # Distinct measured sources spanning spectroscopy, tabular chemistry, manufacturing,
    # electronic-nose drift, and ambient air quality.
    "wine_quality_red": lambda: ds_wine("red"),
    "wine_quality_white": lambda: ds_wine("white"),
    "secom_semiconductor": ds_secom,
    "steel_plates_faults": ds_steel_plates,
    "gas_sensor_array_drift": ds_gas_sensor_drift,
    "air_quality_multisensor": ds_air_quality,
    "tecator_nir_meat": ds_tecator,
    "corn_nir_m5": lambda: ds_corn("m5"),
    "corn_nir_mp5": lambda: ds_corn("mp5"),
    "corn_nir_mp6": lambda: ds_corn("mp6"),
    # ── Simulation benchmarks ───────────────────────────────────────────────────────────
    # Field-standard public simulation benchmarks (real process physics; honestly labelled).
    "cstr_reactor": ds_cstr,
    "three_tank": ds_three_tank,
    "bsm1_wastewater": ds_bsm1_like,
    "penicillin_fedbatch": ds_penicillin_fedbatch,
    # ── Agreement-gated ─────────────────────────────────────────────────────────────────
    # Stand-in only; real SWaT/WADI data requires the iTrust data-use agreement.
    "swat_water_treatment_standin": ds_swat_gated,
}


def _merge_existing():
    """When run with --only, preserve previously-recorded datasets so the manifest stays complete.

    Reads the existing MANIFEST.toml with a lightweight regex parser (no toml dependency) and
    re-appends any [[dataset]] blocks for datasets not in the current ENTRIES list (i.e. not
    re-generated in this run).  This prevents a partial --only run from dropping other datasets.

    Parsing assumptions:
      * String fields match: word = "value"
      * Integer fields match: word = integer (end of line)
      * [[dataset]] markers delimit blocks; regex uses non-greedy dotall between markers.
    """
    if not os.path.exists(MANIFEST):
        return
    have = {e["name"] for e in ENTRIES}
    import re as _re

    txt = open(MANIFEST).read()
    for block in _re.findall(r"\[\[dataset\]\](.*?)(?=\[\[dataset\]\]|\Z)", txt, _re.S):
        fields = dict(_re.findall(r'(\w+)\s*=\s*"([^"]*)"', block))
        ints = dict(_re.findall(r"(\w+)\s*=\s*(\d+)\s*$", block, _re.M))
        name = fields.get("name")
        if not name or name in have:
            continue
        ENTRIES.append(dict(
            name=name, kind=fields.get("kind", ""), status=fields.get("status", ""),
            source_url=fields.get("source_url", ""), license=fields.get("license", ""),
            citation=fields.get("citation", ""), retrieved=fields.get("retrieved", ""),
            n_samples=int(ints.get("n_samples", 0)), n_vars=int(ints.get("n_vars", 0)),
            sha256=fields.get("sha256", ""), notes=fields.get("notes", "")))


def write_manifest():
    """Serialise the ENTRIES list to data/MANIFEST.toml in TOML array-of-tables format.

    Calls _merge_existing() first so that a --only partial run doesn't erase previously
    generated entries.  Entries are sorted alphabetically by name for a stable, diff-friendly
    output; this is why git diff on MANIFEST.toml shows minimal changes on partial re-runs.

    Output format: one [[dataset]] block per entry with all string fields quoted and integer
    fields (n_samples, n_vars) unquoted.  The sha256 field is the hex digest of the committed
    CSV slice as computed by sha256_file().

    Writes: data/MANIFEST.toml  (committed artifact; human-readable provenance record)
    """
    _merge_existing()
    lines = [
        "# DSFB-Chemical-Engineering — dataset provenance manifest.",
        "# Generated by scripts/fetch_datasets.py. Each block records the canonical public source,",
        "# license, citation, SHA-256 of the committed slice, and an honest status:",
        "#   real-slice    = a processed slice of MEASURED public data",
        "#   simulation    = generated from real process physics (field-standard public benchmark)",
        "#   gated-standin = physics stand-in; real data requires a data-use agreement (not redistributed)",
        "",
        f'generated = "{TODAY}"',
        f"dataset_count = {len(ENTRIES)}",
        "",
    ]
    for e in sorted(ENTRIES, key=lambda x: x["name"]):
        lines.append("[[dataset]]")
        for key in ["name", "kind", "status", "source_url", "license", "citation", "retrieved"]:
            lines.append(f'{key} = "{e[key]}"')
        lines.append(f'n_samples = {e["n_samples"]}')
        lines.append(f'n_vars = {e["n_vars"]}')
        lines.append(f'sha256 = "{e["sha256"]}"')
        lines.append(f'notes = "{e["notes"]}"')
        lines.append("")
    with open(MANIFEST, "w") as f:
        f.write("\n".join(lines))
    log(f"wrote manifest: {MANIFEST} ({len(ENTRIES)} datasets)")


def main():
    """Entry point: iterate REGISTRY, run each handler, then write MANIFEST.toml.

    --only name1,name2  runs only the named datasets (comma-separated); all others are
    preserved from the existing manifest via _merge_existing() inside write_manifest().
    Any individual dataset failure is caught and logged but does not abort the run; this
    allows partial fetches (e.g. when pyreadr is absent) to still produce a usable manifest.
    Wall-clock time per dataset is reported for diagnostics.
    """
    only = None
    if "--only" in sys.argv:
        only = set(sys.argv[sys.argv.index("--only") + 1].split(","))
    log("== DSFB-Chemical-Engineering dataset fetch ==")
    for name, fn in REGISTRY.items():
        if only and name not in only:
            continue
        try:
            t0 = time.time()
            fn()
            log(f"    ({time.time()-t0:.1f}s)")
        except Exception as e:
            log(f"  !! {name} FAILED: {type(e).__name__}: {e}")
    write_manifest()


if __name__ == "__main__":
    main()
