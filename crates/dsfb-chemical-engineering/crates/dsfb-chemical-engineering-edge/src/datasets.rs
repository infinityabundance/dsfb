//! Dataset access layer.
//!
//! ## Real datasets
//!
//! Public datasets that permit redistribution (subject to their individual licences) are committed
//! as processed CSV slices under `data/slices/`. Provenance and SHA-256 checksums are recorded in
//! `data/MANIFEST.toml`. Slices are loaded via [`load_csv_slice`].
//!
//! ## Synthetic datasets
//!
//! Where redistribution is not permitted, or as a reproducibility stand-in, [`synthetic_process`]
//! generates process-like multivariate data with injected, *labelled* faults. Synthetic data is
//! always tagged `kind = "synthetic"` in outputs and is never presented as measured data.
//!
//! ## RNG
//!
//! The only source of randomness in this crate is the [`Rng`] struct in this module, used
//! exclusively by `synthetic_process`. It is a deterministic seeded SplitMix64 generator — no
//! thread-local state, no system entropy — so the same seed always produces the same matrix.
//! This is the only place in the pipeline where pseudo-random numbers are generated; everything
//! else is deterministic arithmetic over the input data.

use crate::data::DataMatrix;
use std::path::Path;

/// Deterministic 64-bit SplitMix64 pseudo-random number generator.
///
/// SplitMix64 (Vigna 2015) is a fast, high-quality generator suitable for simulation.
/// No `rand` crate dependency; the implementation is ~10 lines so it is self-contained
/// and auditable. The only purpose in this crate is synthetic process data generation.
///
/// The initial state is `seed XOR φ` where φ = 0x9E3779B97F4A7C15 (the fractional part of
/// the golden ratio in 64-bit fixed point). This is the standard SplitMix64 initialisation.
pub struct Rng {
    state: u64,
}
impl Rng {
    pub fn new(seed: u64) -> Self {
        // XOR with golden-ratio constant to mix the seed before the first step.
        Rng {
            state: seed ^ 0x9E37_79B9_7F4A_7C15,
        }
    }

    /// Advance the state and return a uniformly distributed 64-bit integer.
    ///
    /// SplitMix64 state transition: add the golden-ratio constant, then apply three
    /// multiply-xorshift mixing rounds (Murmur-style finaliser). The sequence has full
    /// period 2^64 and passes statistical tests (SmallCrush, BigCrush).
    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform f64 in the half-open interval [0, 1).
    ///
    /// Uses the upper 53 bits of the 64-bit output (discarding the 11 lowest bits) to
    /// fill the 53-bit mantissa of an IEEE 754 double, giving uniform spacing in [0, 1).
    pub fn uniform(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Standard normal sample via Box–Muller transform (deterministic, consumes 2 uniform draws).
    ///
    /// `u1` is clamped to [1e-12, 1) to avoid `ln(0)` = -∞. The cosine form of Box–Muller is
    /// used; the sine form would give a second independent sample, which is discarded here to
    /// keep the sequence simple and auditable.
    pub fn normal(&mut self) -> f64 {
        let u1 = self.uniform().max(1e-12);
        let u2 = self.uniform();
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }
}

/// The four fault archetypes injected into synthetic process data.
///
/// These correspond to common fault classes in process chemometrics:
/// - Co-drift / step: a sudden shift in the mean of multiple correlated variables simultaneously
///   (e.g., a set-point change, feed-composition disturbance).
/// - Ramp drift: a slow, progressive departure from nominal (e.g., catalyst deactivation,
///   sensor fouling).
/// - Sensor bias: a fixed offset on a single measurement channel (single-variable fault).
/// - Oscillation: a periodic signal superimposed on one variable (valve hunting, control
///   instability, stiction).
#[derive(Debug, Clone, Copy)]
pub enum SyntheticFault {
    /// Sustained step in a subset of correlated variables (co-drift).
    StepDrift,
    /// Ramp drift in a subset of variables.
    RampDrift,
    /// Single-variable bias (sensor fault).
    SensorBias,
    /// Oscillatory limit cycle in one variable (valve hunting / stiction).
    Oscillation,
}

/// Generate a deterministic synthetic multivariate process dataset with a labelled injected fault.
///
/// ## Data-generating model
///
/// Samples are drawn from a latent-factor model:
/// ```text
/// x[t, j] = Σ_f  loadings[j][f] * factor[f][t]  +  noise[j][t]
/// ```
/// where:
/// - `loadings[j][f] = sin((j+1)*0.7 + (f+1)*1.3)` — deterministic, no RNG, produces
///   a dense cross-variable correlation structure.
/// - `factor[f][t]` follows AR(1): `factor[f][t] = 0.85 * factor[f][t-1] + 0.5 * N(0,1)` —
///   AR coefficient 0.85 gives realistic autocorrelation without explosive behaviour.
/// - `noise[j][t] ~ N(0, 0.09)` — i.i.d. sensor noise (std = 0.3).
///
/// ## Fault injection
///
/// The first `onset` samples are nominal (`label = 0`). From `onset` onwards (`label = 1`) the
/// fault signal is additively superimposed. Fault amplitudes are chosen to produce clearly
/// detectable but not trivially obvious faults:
/// - `StepDrift`: +3.0 on the first third of variables.
/// - `RampDrift`: +0.05 * (t - onset) on the first third of variables (grows by 1 unit per 20
///   samples).
/// - `SensorBias`: +6.0 on variable 0 only.
/// - `Oscillation`: +4.0 * sin(0.6 * (t - onset)) on variable 0 (period ≈ 10 samples).
///
/// ## Determinism contract
///
/// Given the same `(n_samples, n_vars, onset, fault, seed)`, this function always produces
/// the same `DataMatrix`. The only source of randomness is the seeded `Rng`; `name` is unused
/// in the computation and is included only for call-site readability.
pub fn synthetic_process(
    name: &str,
    n_samples: usize,
    n_vars: usize,
    onset: usize,
    fault: SyntheticFault,
    seed: u64,
) -> DataMatrix {
    let mut rng = Rng::new(seed);
    // Cap at 3 latent factors; more would add negligible benefit for monitoring benchmarks.
    let n_factors = 3.min(n_vars.max(1));
    // Deterministic loading matrix: no RNG, reproducible cross-variable correlation structure.
    let mut loadings = vec![vec![0.0; n_factors]; n_vars];
    for (j, row) in loadings.iter_mut().enumerate() {
        for (f, w) in row.iter_mut().enumerate() {
            *w = ((j as f64 + 1.0) * 0.7 + (f as f64 + 1.0) * 1.3).sin();
        }
    }
    let var_names: Vec<String> = (0..n_vars).map(|j| format!("v{:02}", j)).collect();
    let mut rows: Vec<Vec<f64>> = Vec::with_capacity(n_samples);
    let mut labels = vec![0u32; n_samples];

    // Factor state vector carries AR(1) memory across time steps.
    let mut factor_state = vec![0.0f64; n_factors];
    // `t` is the timestep: it drives the AR(1) update, the fault-onset test (`t >= onset`), the
    // elapsed-time progression (`t - onset`) and `labels[t]` — not merely a cursor into one slice — so
    // the range-loop form is the clearest expression here (clippy's needless_range_loop is a false
    // positive). This loop is also replay-critical: its RNG call order defines the golden synthetic hashes.
    #[allow(clippy::needless_range_loop)]
    for t in 0..n_samples {
        // AR(1) update: coefficient 0.85 gives moderate autocorrelation (typical of process data).
        for fs in factor_state.iter_mut() {
            *fs = 0.85 * *fs + 0.5 * rng.normal();
        }
        // Project latent factors onto observed variables.
        let mut row = vec![0.0; n_vars];
        for j in 0..n_vars {
            let mut x = 0.0;
            for f in 0..n_factors {
                x += loadings[j][f] * factor_state[f];
            }
            x += 0.3 * rng.normal(); // i.i.d. sensor noise, std = 0.3
            row[j] = x;
        }
        // Fault injection: additive superposition from onset onwards.
        if t >= onset {
            labels[t] = 1;
            let prog = (t - onset) as f64; // samples elapsed since fault onset
            match fault {
                // Step in the first third of variables; magnitude chosen to be detectable by PCA T².
                SyntheticFault::StepDrift => {
                    for cell in row.iter_mut().take((n_vars / 3).max(1)) {
                        *cell += 3.0;
                    }
                }
                // Ramp that grows by 1 unit per 20 samples; tests detection-delay sensitivity.
                SyntheticFault::RampDrift => {
                    for cell in row.iter_mut().take((n_vars / 3).max(1)) {
                        *cell += 0.05 * prog;
                    }
                }
                // Single-variable bias; isolated from other variables — tests univariate detectors.
                SyntheticFault::SensorBias => {
                    row[0] += 6.0;
                }
                // Oscillation at ~10-sample period; tests frequency-domain / SPE detectors.
                SyntheticFault::Oscillation => {
                    row[0] += 4.0 * (0.6 * prog).sin();
                }
            }
        }
        rows.push(row);
    }
    let mut m = DataMatrix::new(var_names, rows);
    m.labels = Some(labels);
    m.fault_onset = Some(onset);
    let _ = name; // unused in computation; kept for call-site readability
    m
}

/// Load a committed CSV slice and return the `DataMatrix` plus a resolved baseline length.
///
/// ## CSV format expected
/// - First row is a header.
/// - Numeric variable columns: any column not named `"label"`, `"fault"`, or `"phase"`.
/// - Optional `"label"` or `"fault"` column: non-zero values are treated as fault (1);
///   zero values are nominal (0). The first non-zero row becomes `fault_onset`.
/// - Optional `"phase"` column: stored as `DataMatrix::phase` (unused by the pipeline,
///   available for external analysis).
/// - Non-parseable cells are silently replaced with `f64::NAN`. The pipeline's standardise
///   step propagates NaN to detector outputs, which are typically clipped to zero margin.
///
/// ## Baseline length resolution
/// 1. If fault_onset is known and ≥ 4: use `fault_onset` as the baseline length (nominal window
///    before the fault). The minimum of 4 avoids degenerate statistics.
/// 2. Otherwise: use `floor(n * baseline_fraction)`, clamped to `[4, n]`.
///
/// Returns `Err(String)` for IO errors or empty files; individual parse failures are silent.
/// Result of the dataset-provenance gate: how many committed slices matched / are missing / changed.
#[derive(Debug, Clone)]
pub struct ManifestGate {
    pub verified: usize,
    pub missing: usize,
    pub mismatched: usize,
    /// Names of slices whose bytes no longer match their recorded SHA-256 (corruption or tampering).
    pub mismatched_names: Vec<String>,
}

/// Verify every committed slice against the SHA-256 recorded for it in `data/MANIFEST.toml` — the
/// dataset-provenance gate, enforced in Rust (previously only the Colab notebook checked it). For each
/// `[[dataset]]` with a `sha256`, hash `data/slices/<name>.csv` and compare. A *mismatch* means the
/// slice bytes changed since the manifest was written (corruption/tampering); a *missing* slice is not
/// an integrity failure (some datasets are fetched separately). Deterministic; reads files, no network.
pub fn verify_manifest_sha256(crate_dir: &Path) -> Result<ManifestGate, String> {
    #[derive(serde::Deserialize)]
    struct Entry {
        name: String,
        #[serde(default)]
        sha256: Option<String>,
    }
    #[derive(serde::Deserialize)]
    struct Manifest {
        #[serde(default)]
        dataset: Vec<Entry>,
    }
    let path = crate_dir.join("data").join("MANIFEST.toml");
    let text =
        std::fs::read_to_string(&path).map_err(|e| format!("read {}: {e}", path.display()))?;
    let man: Manifest =
        toml::from_str(&text).map_err(|e| format!("parse {}: {e}", path.display()))?;
    let slices = crate_dir.join("data").join("slices");
    let mut g = ManifestGate {
        verified: 0,
        missing: 0,
        mismatched: 0,
        mismatched_names: Vec::new(),
    };
    for e in &man.dataset {
        let Some(want) = e.sha256.as_deref() else {
            continue;
        };
        match std::fs::read(slices.join(format!("{}.csv", e.name))) {
            Ok(bytes) if crate::hashing::sha256_hex(&bytes) == want => g.verified += 1,
            Ok(_) => {
                g.mismatched += 1;
                g.mismatched_names.push(e.name.clone());
            }
            Err(_) => g.missing += 1,
        }
    }
    Ok(g)
}

/// Map each MANIFEST `[[dataset]]` `name` → its honest provenance `kind` (`measured` / `simulation` /
/// `agreement-gated`). This is the single source of truth the running tool uses to stamp `data_kind`
/// on every emitted artifact, so `metrics.csv`, each `casefile.json`, the operator report, and
/// `manifest.json` agree with MANIFEST.toml and the paper's dataset table — instead of a blanket
/// "measured/slice" that mislabels the simulation slices. Returns an empty map (not an error) if the
/// manifest is absent, so callers degrade gracefully (e.g. the synthetic-suite fallback).
pub fn manifest_kinds(crate_dir: &Path) -> std::collections::BTreeMap<String, String> {
    #[derive(serde::Deserialize)]
    struct Entry {
        name: String,
        #[serde(default)]
        kind: Option<String>,
    }
    #[derive(serde::Deserialize)]
    struct Manifest {
        #[serde(default)]
        dataset: Vec<Entry>,
    }
    let path = crate_dir.join("data").join("MANIFEST.toml");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return std::collections::BTreeMap::new();
    };
    let Ok(man) = toml::from_str::<Manifest>(&text) else {
        return std::collections::BTreeMap::new();
    };
    man.dataset
        .into_iter()
        .filter_map(|e| e.kind.map(|k| (e.name, k)))
        .collect()
}

/// Map a MANIFEST provenance `kind` to the `data_kind` tag stamped on artifacts. Faithful, not lossy:
/// `measured` → "measured/slice", `simulation` → "simulation/slice", `agreement-gated` → "gated-standin".
/// An unrecognised kind passes through verbatim, so a future MANIFEST kind is *disclosed*, never hidden.
pub fn data_kind_tag(manifest_kind: &str) -> String {
    match manifest_kind {
        "measured" => "measured/slice".to_string(),
        "simulation" => "simulation/slice".to_string(),
        "agreement-gated" => "gated-standin".to_string(),
        other => other.to_string(),
    }
}

pub fn load_csv_slice(path: &Path, baseline_fraction: f64) -> Result<(DataMatrix, usize), String> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true) // tolerate rows with varying field counts
        .from_path(path)
        .map_err(|e| format!("open {}: {e}", path.display()))?;
    let headers: Vec<String> = rdr
        .headers()
        .map_err(|e| e.to_string())?
        .iter()
        .map(|s| s.to_string())
        .collect();

    // Identify the optional label/fault and phase columns by case-insensitive name matching.
    let label_col = headers
        .iter()
        .position(|h| h.eq_ignore_ascii_case("label") || h.eq_ignore_ascii_case("fault"));
    let phase_col = headers.iter().position(|h| h.eq_ignore_ascii_case("phase"));
    // BTreeSet for O(log n) skip-check and to avoid a HashMap import.
    let skip: std::collections::BTreeSet<usize> =
        [label_col, phase_col].into_iter().flatten().collect();
    // Variable names = all header columns that are not in `skip`.
    let var_names: Vec<String> = headers
        .iter()
        .enumerate()
        .filter(|(i, _)| !skip.contains(i))
        .map(|(_, h)| h.clone())
        .collect();

    let mut rows: Vec<Vec<f64>> = Vec::new();
    let mut labels: Vec<u32> = Vec::new();
    let mut phases: Vec<u32> = Vec::new();
    for rec in rdr.records() {
        let rec = rec.map_err(|e| e.to_string())?;
        let mut row = Vec::with_capacity(var_names.len());
        for (i, field) in rec.iter().enumerate() {
            if skip.contains(&i) {
                continue;
            }
            // Silently substitute NaN for unparseable cells rather than failing the whole slice.
            row.push(field.trim().parse::<f64>().unwrap_or(f64::NAN));
        }
        // The reader is `flexible(true)`, so a malformed row may carry fewer (or more) fields than the
        // header. Reconcile to exactly `var_names.len()` — pad short rows with NaN (a gap the detectors
        // already tolerate) and truncate over-long rows — so a single ragged row degrades gracefully
        // instead of tripping `DataMatrix::new`'s rectangularity assert and panicking the whole run.
        row.resize(var_names.len(), f64::NAN);
        if let Some(lc) = label_col {
            // Treat any non-zero float as fault = 1, zero as nominal = 0.
            labels.push(
                rec.get(lc)
                    .and_then(|s| s.trim().parse::<f64>().ok())
                    .map(|v| if v != 0.0 { 1 } else { 0 })
                    .unwrap_or(0),
            );
        }
        if let Some(pc) = phase_col {
            phases.push(
                rec.get(pc)
                    .and_then(|s| s.trim().parse::<f64>().ok())
                    .map(|v| v as u32)
                    .unwrap_or(0),
            );
        }
        rows.push(row);
    }
    if rows.is_empty() {
        return Err(format!("{}: no data rows", path.display()));
    }
    let n = rows.len();
    let mut m = DataMatrix::new(var_names, rows);
    if !labels.is_empty() {
        // `fault_onset` = index of the first non-zero label; `None` if all labels are zero.
        let onset = labels.iter().position(|&l| l != 0);
        m.fault_onset = onset;
        m.labels = Some(labels);
    }
    if !phases.is_empty() {
        m.phase = Some(phases);
    }
    // Baseline = leading nominal window: up to fault onset, else a fraction of the record. The lower
    // bound is `4.min(n)` (not a bare 4) so a tiny slice with n < 4 does not trip `usize::clamp`'s
    // `min <= max` requirement — `.max(4).min(n)` yields the whole record when n < 4 and the 4-floored
    // fraction otherwise. (All committed slices have n >= 80, so this is replay-inert; it hardens the
    // loader for tiny/degenerate inputs, matching `load_historian_csv`'s `4.min(n)` floor.)
    let n_base = match m.fault_onset {
        Some(o) if o >= 4 => o,
        _ => ((n as f64 * baseline_fraction) as usize).max(4).min(n),
    };
    Ok((m, n_base))
}

/// Load a plant-historian export in long ("narrow") format and pivot it to the wide tag-per-column
/// [`DataMatrix`] the pipeline expects. This is the generic industrial-transition entry point: any
/// historian that can emit `timestamp,tag,value[,unit,quality,source,phase]` rows can be replayed
/// through DSFB-Chemical-Engineering without bespoke glue.
///
/// Required columns (case-insensitive): `timestamp`, `tag`, `value`. Optional: `quality` (a nonzero
/// numeric, or non-`good`/`ok` text, marks the sample bad → NaN, so detectors see a gap rather than a
/// spurious reading); `phase` or `phase_id` (→ [`DataMatrix::phase`], which enables regime-conditioned
/// envelopes — this is the "batch-record mode" path); and the control-context columns `setpoint`,
/// `manipulated_variable` (alias `mv`), and `controller_mode` (alias `mode`), which are injected as
/// derived per-tag variables `<tag>::setpoint`, `<tag>::mv`, `<tag>::mode` so loop intent becomes a
/// first-class witness. `unit`/`source` are provenance only (ignored). **All optional columns are
/// purely additive: a bare `timestamp,tag,value` historian is parsed byte-for-byte as before.**
///
/// Rows are the sorted distinct timestamps (string sort; ISO-8601 timestamps sort chronologically);
/// columns are the sorted distinct tags. Missing `(timestamp, tag)` cells become NaN. The baseline is
/// the leading `baseline_fraction` of timestamps. Deterministic: `BTreeMap`/`BTreeSet` ordering, no RNG.
/// Deterministic categorical → numeric code for an ISA/NAMUR controller-mode string, so controller
/// mode can enter the residual matrix as a witnessable step variable rather than free text:
/// auto = 0, manual = 1, cascade = 2, any other non-empty token = 9; empty → NaN (no derived cell).
fn controller_mode_code(s: &str) -> f64 {
    match s.to_ascii_lowercase().as_str() {
        "auto" | "automatic" | "a" => 0.0,
        "man" | "manual" | "m" => 1.0,
        "cas" | "cascade" => 2.0,
        "" => f64::NAN,
        _ => 9.0,
    }
}

pub fn load_historian_csv(
    path: &Path,
    baseline_fraction: f64,
) -> Result<(DataMatrix, usize), String> {
    use std::collections::{BTreeMap, BTreeSet};
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_path(path)
        .map_err(|e| format!("open {}: {e}", path.display()))?;
    let headers: Vec<String> = rdr
        .headers()
        .map_err(|e| e.to_string())?
        .iter()
        .map(|s| s.to_string())
        .collect();
    let find = |name: &str| headers.iter().position(|h| h.eq_ignore_ascii_case(name));
    let ts_c =
        find("timestamp").ok_or_else(|| "historian CSV missing 'timestamp' column".to_string())?;
    let tag_c = find("tag").ok_or_else(|| "historian CSV missing 'tag' column".to_string())?;
    let val_c = find("value").ok_or_else(|| "historian CSV missing 'value' column".to_string())?;
    let qual_c = find("quality");
    let phase_c = find("phase").or_else(|| find("phase_id"));
    // Optional control-context columns (a richer real-historian schema). Each, when present, becomes a
    // derived per-tag variable so loop context enters the residual matrix as a first-class witness.
    // `unit` is provenance only (ignored here). Absent columns => bare timestamp,tag,value behaviour.
    let sp_c = find("setpoint").or_else(|| find("sp"));
    let mv_c = find("manipulated_variable").or_else(|| find("mv"));
    let mode_c = find("controller_mode").or_else(|| find("mode"));

    // timestamp -> (tag -> value), with timestamps kept sorted by BTreeMap; plus per-timestamp phase.
    let mut cells: BTreeMap<String, BTreeMap<String, f64>> = BTreeMap::new();
    let mut ts_phase: BTreeMap<String, u32> = BTreeMap::new();
    let mut tags: BTreeSet<String> = BTreeSet::new();
    for rec in rdr.records() {
        let rec = rec.map_err(|e| e.to_string())?;
        let ts = rec.get(ts_c).unwrap_or("").trim().to_string();
        let tag = rec.get(tag_c).unwrap_or("").trim().to_string();
        if ts.is_empty() || tag.is_empty() {
            continue;
        }
        let mut value = rec
            .get(val_c)
            .and_then(|s| s.trim().parse::<f64>().ok())
            .unwrap_or(f64::NAN);
        if let Some(qc) = qual_c {
            if let Some(q) = rec.get(qc) {
                let q = q.trim();
                // A nonzero numeric quality, or non-empty non-"good"/"ok" text, marks the sample bad.
                let bad = match q.parse::<f64>() {
                    Ok(x) => x != 0.0,
                    Err(_) => {
                        !q.is_empty()
                            && !q.eq_ignore_ascii_case("good")
                            && !q.eq_ignore_ascii_case("ok")
                    }
                };
                if bad {
                    value = f64::NAN;
                }
            }
        }
        cells
            .entry(ts.clone())
            .or_default()
            .insert(tag.clone(), value);
        // Derived control-context variables: `<tag>::setpoint`, `<tag>::mv`, `<tag>::mode`. Injected
        // only when the corresponding column exists and parses, so a 5-column historian is byte-for-byte
        // unchanged. They let the control-action context and detectors see loop intent directly.
        if let Some(c) = sp_c {
            if let Some(x) = rec.get(c).and_then(|s| s.trim().parse::<f64>().ok()) {
                let d = format!("{tag}::setpoint");
                cells.entry(ts.clone()).or_default().insert(d.clone(), x);
                tags.insert(d);
            }
        }
        if let Some(c) = mv_c {
            if let Some(x) = rec.get(c).and_then(|s| s.trim().parse::<f64>().ok()) {
                let d = format!("{tag}::mv");
                cells.entry(ts.clone()).or_default().insert(d.clone(), x);
                tags.insert(d);
            }
        }
        if let Some(c) = mode_c {
            let code = controller_mode_code(rec.get(c).unwrap_or("").trim());
            if code.is_finite() {
                let d = format!("{tag}::mode");
                cells.entry(ts.clone()).or_default().insert(d.clone(), code);
                tags.insert(d);
            }
        }
        tags.insert(tag);
        if let Some(pc) = phase_c {
            if let Some(p) = rec.get(pc).and_then(|s| s.trim().parse::<f64>().ok()) {
                ts_phase.insert(ts, p as u32);
            }
        }
    }
    if cells.is_empty() || tags.is_empty() {
        return Err(format!("{}: no (timestamp,tag,value) rows", path.display()));
    }
    let tag_list: Vec<String> = tags.iter().cloned().collect();
    let have_phase = phase_c.is_some();
    let mut rows: Vec<Vec<f64>> = Vec::with_capacity(cells.len());
    let mut phases: Vec<u32> = Vec::with_capacity(cells.len());
    for (ts, row_map) in &cells {
        rows.push(
            tag_list
                .iter()
                .map(|tag| row_map.get(tag).copied().unwrap_or(f64::NAN))
                .collect(),
        );
        if have_phase {
            phases.push(ts_phase.get(ts).copied().unwrap_or(0));
        }
    }
    let n = rows.len();
    let mut m = DataMatrix::new(tag_list, rows);
    if have_phase && phases.len() == n {
        m.phase = Some(phases);
    }
    let n_base = ((n as f64 * baseline_fraction) as usize).clamp(4.min(n), n);
    Ok((m, n_base))
}

#[cfg(test)]
mod historian_tests {
    use super::*;

    #[test]
    fn historian_pivot_roundtrip() {
        // Two timestamps, two tags (T1, P1), with a phase column → should pivot to a 2×2 matrix with
        // tags sorted (P1, T1), per-timestamp phase captured, and baseline = leading half.
        let p = std::env::temp_dir().join("dsfb_historian_roundtrip_test.csv");
        std::fs::write(
            &p,
            "timestamp,tag,value,quality,phase\n\
             2026-01-01T00:00:00Z,T1,10.0,0,0\n\
             2026-01-01T00:00:00Z,P1,1.0,0,0\n\
             2026-01-01T00:00:01Z,T1,11.0,0,1\n\
             2026-01-01T00:00:01Z,P1,1.1,0,1\n",
        )
        .unwrap();
        let (m, nb) = load_historian_csv(&p, 0.5).unwrap();
        assert_eq!(m.n_samples, 2);
        assert_eq!(m.n_vars, 2);
        assert_eq!(m.var_names, vec!["P1".to_string(), "T1".to_string()]);
        assert_eq!(m.row(0), &[1.0, 10.0]); // sorted tags: P1 then T1
        assert_eq!(m.row(1), &[1.1, 11.0]);
        assert_eq!(m.phase.as_deref(), Some(&[0u32, 1u32][..]));
        // n < 4: the baseline floor (4.min(n)) covers the whole tiny record.
        assert_eq!(nb, 2);
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn historian_quality_gate_nans_bad_samples() {
        let p = std::env::temp_dir().join("dsfb_historian_quality_test.csv");
        std::fs::write(
            &p,
            "timestamp,tag,value,quality\n\
             t0,A,5.0,0\n\
             t1,A,9.9,1\n", // quality=1 → bad → NaN
        )
        .unwrap();
        let (m, _) = load_historian_csv(&p, 0.5).unwrap();
        assert_eq!(m.row(0), &[5.0]);
        assert!(m.row(1)[0].is_nan());
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn historian_control_context_columns_become_derived_variables() {
        // setpoint / manipulated_variable / controller_mode become derived per-tag columns
        // (`<tag>::setpoint`, `<tag>::mv`, `<tag>::mode`); rows without those columns are unaffected.
        let p = std::env::temp_dir().join("dsfb_historian_control_test.csv");
        std::fs::write(
            &p,
            "timestamp,tag,value,unit,controller_mode,setpoint,manipulated_variable\n\
             t0,LIT,700.0,mm,auto,700.0,50.0\n\
             t0,FIT,10.0,m3/h,,,\n\
             t1,LIT,701.0,mm,manual,700.0,55.0\n\
             t1,FIT,11.0,m3/h,,,\n",
        )
        .unwrap();
        let (m, _) = load_historian_csv(&p, 0.5).unwrap();
        // 2 base tags (FIT, LIT) + 3 derived for LIT (mode/mv/setpoint) = 5 columns; FIT has none.
        assert_eq!(
            m.n_vars, 5,
            "derived control columns must be injected for LIT only"
        );
        for v in ["FIT", "LIT", "LIT::mode", "LIT::mv", "LIT::setpoint"] {
            assert!(m.var_names.iter().any(|n| n == v), "missing column {v}");
        }
        // controller_mode auto→0, manual→1 (deterministic categorical code).
        let mode_col = m.var_names.iter().position(|n| n == "LIT::mode").unwrap();
        assert_eq!(m.row(0)[mode_col], 0.0);
        assert_eq!(m.row(1)[mode_col], 1.0);
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn median_ignores_non_finite() {
        // A quality-gated NaN must not poison the robust statistic, and must not panic the sort.
        assert_eq!(crate::linalg::median(&[1.0, f64::NAN, 3.0, 2.0]), 2.0);
        assert!(crate::linalg::mad(&[1.0, f64::NAN, 1.0, 1.0]).is_finite());
    }

    #[test]
    fn ragged_csv_row_degrades_gracefully_instead_of_panicking() {
        // A malformed slice row with fewer fields than the header must NOT panic `DataMatrix::new`'s
        // rectangularity assert (P46): the short row is NaN-padded to the column count. Row 2 here has
        // only 2 of 3 numeric fields (a third column is absent), and an unparseable cell becomes NaN.
        let p = std::env::temp_dir().join("dsfb_ragged_row_test.csv");
        std::fs::write(&p, "a,b,c\n1.0,2.0,3.0\n4.0,5.0\n7.0,x,9.0\n").unwrap();
        let (m, _) = load_csv_slice(&p, 0.5).expect("ragged slice still loads");
        assert_eq!(m.n_vars, 3);
        assert_eq!(m.n_samples, 3);
        assert_eq!(m.row(0), &[1.0, 2.0, 3.0]);
        assert_eq!(m.row(1)[0], 4.0);
        assert_eq!(m.row(1)[1], 5.0);
        assert!(m.row(1)[2].is_nan(), "missing trailing field is NaN-padded");
        assert!(m.row(2)[1].is_nan(), "unparseable cell is NaN");
        std::fs::remove_file(&p).ok();
    }
}
