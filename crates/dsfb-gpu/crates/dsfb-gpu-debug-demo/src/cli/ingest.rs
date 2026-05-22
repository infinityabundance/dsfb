//! S-REAL.1 — Deterministic ingest of `# residual-projection v2` TSV fixtures.
//!
//! WHY: The S-REAL.1 audit gauntlet runs DSFB-GPU on real public datasets
//! whose canonical released form is **pre-projected residuals** in a
//! TAB-delimited text file with `# key=value` comment headers. DSFB-GPU's
//! production pipeline normally takes a `Vec<TraceEvent>` (trace-event
//! stream) and projects events into residuals via the window-feature kernel.
//! The TSV fixtures are already past that projection stage. To run the
//! deterministic engine on this form without modifying the dispatcher, we
//! deterministically **lower** each (window, signal) cell into a synthetic
//! `TraceEvent` whose latency value carries the residual magnitude. The
//! lowering rule is byte-replayable: same TSV bytes → same `Vec<TraceEvent>`
//! → same `CaseFile`.
//!
//! Discipline:
//! - **No probabilistic ingest**: every cell maps to one event by a fixed rule.
//! - **No silent NaN handling**: NaN cells are skipped (no event emitted for
//!   that cell), and the count of skipped cells is reported in the
//!   `IngestReport` so the audit report can disclose the projection loss.
//! - **SHA-256 byte-pin**: the loader returns an error if the file bytes
//!   do not hash to the expected pin. The audit's `dataset_manifest.toml`
//!   records the pin alongside the upstream DOI/URL.
//! - **No domain-truth claim**: the lowering interprets cell values as
//!   "milliseconds-scaled signal magnitude" and reports that convention in
//!   the audit; we make no claim about what the cell value "is" in the
//!   upstream domain.
//!
//! Non-claims (preserved into the audit's `limitations.md`):
//! - This loader does NOT recover the upstream's original trace events. It
//!   produces a deterministic event sequence whose post-projection residual
//!   matches the fixture's residual at each cell, under the documented
//!   lowering rule.
//! - This loader does NOT validate the upstream dataset's labels, semantics,
//!   ground truth, or fitness for any downstream use.
//!
//! License: Apache-2.0. Background IP: Invariant Forge LLC.

use std::collections::BTreeMap;
use std::fmt;

use dsfb_gpu_debug_core::event::TraceEvent;
use dsfb_gpu_debug_core::hash::sha256;

/// Parsed `# residual-projection v2` fixture.
///
/// WHY: The TSV header carries provenance metadata (DOI, URL, archive
/// SHA-256, num_windows, num_signals, healthy_window_end, license) that the
/// audit's `dataset_manifest.toml` must cite verbatim, and the data rows
/// are the residual projection that the event-lowering rule consumes.
/// Carrying both in one struct keeps the metadata + data co-located so the
/// audit report cannot accidentally claim metadata from one fixture against
/// data from another.
///
/// `Eq` is intentionally not derived because cell values are `f64` (NaN is
/// already collapsed to `None` by the parser, so the remaining floats are
/// finite, but f64 still rejects `Eq` at the type level). `PartialEq` is
/// sufficient for the byte-replay determinism we test.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct ResidualProjectionFixture {
    /// SHA-256 of the input file bytes (hex lowercase, 64 chars). This is
    /// the load-bearing provenance pin: the loader rejects mismatched
    /// bytes before parsing, so a corrupted fixture cannot silently
    /// produce a valid-looking CaseFile.
    pub fixture_sha256_hex: String,
    /// Every `# key=value` comment line collected in encounter order then
    /// re-sorted into a BTreeMap for deterministic iteration. Keys without
    /// `=` (e.g. raw comment text) are NOT recorded; the audit's
    /// `dataset_manifest.toml` mirrors only key=value entries.
    pub metadata: BTreeMap<String, String>,
    /// Declared row count from the `# num_windows=` header. May not equal
    /// `rows.len()`: some fixtures publish a num_windows < observed rows
    /// (extra metadata-noise rows past the declared count). The loader
    /// reports both; the lowering uses the observed `rows.len()`.
    pub declared_num_windows: u32,
    /// Declared column count from the `# num_signals=` header. Must equal
    /// every data row's column count; the parser rejects rows that diverge.
    pub declared_num_signals: u32,
    /// Declared healthy-window boundary from `# healthy_window_end=`.
    /// Recorded for the audit report; the loader does NOT use it to split
    /// the data (the dispatcher's bank stage performs admission, not
    /// pre-classification by the loader).
    pub declared_healthy_window_end: u32,
    /// Window-major × signal-minor matrix of cell values. `None` represents
    /// a `nan` token in the TSV — the lowering rule skips these cells.
    pub rows: Vec<Vec<Option<f64>>>,
}

/// What the loader did. Surfaced into the audit report so the human reader
/// can see exactly how many cells were skipped (NaN), how many events were
/// emitted, and what the input/output shapes were.
///
/// WHY: The audit's central honesty claim is "DSFB-GPU saw exactly this".
/// A summary that hides cell-skip counts would let a future engineer
/// over-interpret the case file; making the projection loss explicit
/// prevents that.
#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct IngestReport {
    pub fixture_sha256_hex: String,
    pub fixture_byte_size: u64,
    pub declared_num_windows: u32,
    pub declared_num_signals: u32,
    pub observed_num_windows: u32,
    pub observed_num_signals: u32,
    pub nan_cell_count: u32,
    pub finite_cell_count: u32,
    pub emitted_event_count: u32,
}

/// Loader errors. Every variant has an explicit message so the CLI can
/// surface a human-actionable diagnosis without inventing free-form text.
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum IngestError {
    EmptyFile,
    NotUtf8 {
        byte_offset: usize,
    },
    MissingHeader {
        key: &'static str,
    },
    MalformedNumericHeader {
        key: String,
        value: String,
    },
    RowColumnCountMismatch {
        line: usize,
        expected: usize,
        found: usize,
    },
    BadCell {
        line: usize,
        column: usize,
        token: String,
    },
    NoDataRows,
    Sha256Mismatch {
        expected: String,
        actual: String,
    },
    Sha256NotLowerHex64 {
        provided: String,
    },
}

impl fmt::Display for IngestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(f, "fixture file is empty"),
            Self::NotUtf8 { byte_offset } => {
                write!(
                    f,
                    "fixture file is not UTF-8 (invalid at byte {byte_offset})"
                )
            }
            Self::MissingHeader { key } => {
                write!(f, "fixture missing required `# {key}=...` header")
            }
            Self::MalformedNumericHeader { key, value } => write!(
                f,
                "fixture header `{key}` is not a non-negative integer: {value:?}"
            ),
            Self::RowColumnCountMismatch {
                line,
                expected,
                found,
            } => write!(
                f,
                "fixture data row on line {line} has {found} columns; \
                 declared num_signals = {expected}"
            ),
            Self::BadCell {
                line,
                column,
                token,
            } => write!(
                f,
                "fixture cell on line {line} column {column} is neither a \
                 finite float nor `nan`: {token:?}"
            ),
            Self::NoDataRows => write!(
                f,
                "fixture file contains comment headers but no TAB-delimited \
                 data rows"
            ),
            Self::Sha256Mismatch { expected, actual } => write!(
                f,
                "fixture SHA-256 mismatch (expected {expected}, computed {actual})"
            ),
            Self::Sha256NotLowerHex64 { provided } => write!(
                f,
                "expected fixture SHA-256 must be 64 lowercase hex characters; \
                 got {provided:?}"
            ),
        }
    }
}

/// Parameters for the deterministic event-lowering rule.
///
/// WHY: Each panel-locked S-REAL.1 dataset uses a slightly different cell-
/// value scale (TADBench latency_p50_ms ≈ 50.0; AIOps KPI ≈ 0.04). We
/// expose `value_to_microsecond_scale` so the audit report can record the
/// scale-factor per dataset rather than baking a single magic number into
/// the lowering. The default `value_to_microsecond_scale = 1000.0` treats
/// cell values as milliseconds and produces microseconds for the
/// `TraceEvent.latency_us` field.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct LoweringConfig {
    /// `latency_us = clamp(value * scale, 0, latency_clamp_us)`. The
    /// audit's `schema_map.toml` records this value.
    pub value_to_microsecond_scale: u32,
    /// Upper bound for latency_us. Matches the contract's
    /// `latency_clamp_ms * 1000`. Default 32,767,000 us (32.767 seconds).
    pub latency_clamp_us: u32,
    /// Window size in nanoseconds. Default 1_000_000_000 ns = 1 s, which
    /// matches `Contract::canonical().window_size_ms = 1000`. The lowering
    /// emits `ts_ns = window_index as u64 * window_size_ns`.
    pub window_size_ns: u64,
}

impl Default for LoweringConfig {
    fn default() -> Self {
        Self {
            value_to_microsecond_scale: 1000,
            latency_clamp_us: 32_767_000,
            window_size_ns: 1_000_000_000,
        }
    }
}

/// Convert a SHA-256 byte array to lowercase hex.
///
/// WHY: The MANIFEST pin format and every audit receipt uses lowercase hex;
/// returning hex from one place keeps the conversion canonical and lets
/// the caller compare directly against the pinned constant without
/// case-folding round-trips.
#[must_use]
pub fn sha256_to_hex_lower(bytes: &[u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(64);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

/// Verify file bytes hash to the expected pin.
///
/// WHY: The S-REAL.1 audit's load-bearing provenance claim is "DSFB
/// processed these specific bytes". The loader rejects mismatched bytes
/// before any parsing happens, so a corrupted fixture cannot produce a
/// valid-looking but actually-wrong CaseFile that later replays cleanly.
///
/// # Errors
///
/// Returns `Sha256NotLowerHex64` if `expected_hex` isn't exactly 64
/// lowercase hex characters; `Sha256Mismatch` if the bytes don't match.
pub fn verify_fixture_sha256(bytes: &[u8], expected_hex: &str) -> Result<String, IngestError> {
    if expected_hex.len() != 64 || !expected_hex.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err(IngestError::Sha256NotLowerHex64 {
            provided: expected_hex.to_string(),
        });
    }
    if !expected_hex.bytes().all(|c| !c.is_ascii_uppercase()) {
        return Err(IngestError::Sha256NotLowerHex64 {
            provided: expected_hex.to_string(),
        });
    }
    let actual = sha256_to_hex_lower(&sha256(bytes));
    if actual != expected_hex {
        return Err(IngestError::Sha256Mismatch {
            expected: expected_hex.to_string(),
            actual,
        });
    }
    Ok(actual)
}

/// Parse a `# residual-projection v2` TSV fixture.
///
/// WHY: This is the single entry point for ingesting an S-REAL.1 dataset.
/// It performs the SHA-256 pin check first (no surprises after bytes are
/// trusted), then walks the file once collecting metadata and data rows.
/// The loader is intentionally pedantic: it rejects malformed headers,
/// row-column mismatches, and non-finite/non-nan cell tokens by structural
/// error variants so the audit report can attribute any failure to a
/// specific (line, column) pair.
///
/// # Errors
///
/// Returns the matching `IngestError` variant if the file is empty, not
/// UTF-8, missing a required header, has malformed numeric headers,
/// has rows whose column count diverges from `num_signals`, or has cell
/// tokens that are neither finite floats nor `nan`.
pub fn load_residual_projection_tsv(
    bytes: &[u8],
    expected_sha256_hex: &str,
) -> Result<ResidualProjectionFixture, IngestError> {
    if bytes.is_empty() {
        return Err(IngestError::EmptyFile);
    }
    let actual_hex = verify_fixture_sha256(bytes, expected_sha256_hex)?;

    let text = match std::str::from_utf8(bytes) {
        Ok(s) => s,
        Err(e) => {
            return Err(IngestError::NotUtf8 {
                byte_offset: e.valid_up_to(),
            });
        }
    };

    let mut metadata: BTreeMap<String, String> = BTreeMap::new();
    let mut rows: Vec<Vec<Option<f64>>> = Vec::new();
    let mut expected_columns: Option<usize> = None;

    for (idx, raw_line) in text.lines().enumerate() {
        let line_no = idx + 1;
        let line = raw_line.trim_end_matches('\r');
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix('#') {
            // Metadata line. Format: `# key=value` (extra whitespace
            // after `#` is tolerated). Lines without `=` are silently
            // skipped — they are commentary text, not metadata.
            let trimmed = rest.trim_start();
            if let Some((k, v)) = trimmed.split_once('=') {
                metadata.insert(k.trim().to_string(), v.trim().to_string());
            } else if trimmed.is_empty() {
                // bare `#` is fine; ignore
            } else if trimmed.starts_with("residual-projection") {
                // bare format-marker line, no `=` expected
                metadata.insert("format".to_string(), trimmed.to_string());
            }
            // Other bare comment text is ignored without error.
            continue;
        }

        // Data row. TAB-split, parse each cell as f64 or `nan`.
        let cols: Vec<&str> = line.split('\t').collect();
        let cell_count = cols.len();
        if let Some(prev) = expected_columns {
            if cell_count != prev {
                return Err(IngestError::RowColumnCountMismatch {
                    line: line_no,
                    expected: prev,
                    found: cell_count,
                });
            }
        } else {
            expected_columns = Some(cell_count);
        }

        let mut row: Vec<Option<f64>> = Vec::with_capacity(cell_count);
        for (col_idx, tok) in cols.iter().enumerate() {
            let token = tok.trim();
            if token.eq_ignore_ascii_case("nan") {
                row.push(None);
            } else {
                match token.parse::<f64>() {
                    Ok(v) if v.is_finite() => row.push(Some(v)),
                    _ => {
                        return Err(IngestError::BadCell {
                            line: line_no,
                            column: col_idx + 1,
                            token: token.to_string(),
                        });
                    }
                }
            }
        }
        rows.push(row);
    }

    if rows.is_empty() {
        return Err(IngestError::NoDataRows);
    }

    let declared_num_windows = require_u32_header(&metadata, "num_windows")?;
    let declared_num_signals = require_u32_header(&metadata, "num_signals")?;
    let declared_healthy_window_end = require_u32_header(&metadata, "healthy_window_end")?;

    // Cross-check: every row must have `declared_num_signals` columns.
    let observed_cols = expected_columns.unwrap_or(0);
    if observed_cols != declared_num_signals as usize {
        return Err(IngestError::RowColumnCountMismatch {
            line: 0,
            expected: declared_num_signals as usize,
            found: observed_cols,
        });
    }

    Ok(ResidualProjectionFixture {
        fixture_sha256_hex: actual_hex,
        metadata,
        declared_num_windows,
        declared_num_signals,
        declared_healthy_window_end,
        rows,
    })
}

fn require_u32_header(
    meta: &BTreeMap<String, String>,
    key: &'static str,
) -> Result<u32, IngestError> {
    let v = meta
        .get(key)
        .ok_or(IngestError::MissingHeader { key })?
        .trim();
    v.parse::<u32>()
        .map_err(|_| IngestError::MalformedNumericHeader {
            key: key.to_string(),
            value: v.to_string(),
        })
}

/// Apply the panel-locked deterministic lowering rule.
///
/// WHY: One cell → at most one event. NaN cells produce no event. The rule
/// is intentionally simple and replayable: future engineers (including the
/// user months later) can read this function and know exactly what events
/// the dispatcher saw for any given TSV cell. The audit's `schema_map.toml`
/// records the lowering parameters so the route from cell value to event
/// bytes is fully cited.
///
/// Lowering law:
///
/// ```text
/// For each (window_idx, signal_idx, value) in rows.iter().enumerate()
///                                              .flat_map(|(w, row)| row.iter().enumerate()
///                                                                   .map(move |(s, v)| (w, s, v))):
///   if value is None (nan): skip; no event emitted
///   else:
///     ts_ns         = window_idx * config.window_size_ns
///     entity_id     = signal_idx
///     route_id      = 0
///     span_id       = window_idx * 65536 + signal_idx           (deterministic, unique per cell)
///     parent_span_id = 0
///     latency_us    = clamp(value * config.value_to_microsecond_scale,
///                           0, config.latency_clamp_us)
///     status_code   = 200                                       (canonical "ok")
///     error_code    = 0                                         (no error flag from projection)
///     event_kind    = 0
///     flags         = 0
/// ```
///
/// The returned events are in row-major × column-major canonical order
/// (window ascending, signal ascending within window). The dispatcher's
/// window-feature kernel groups by `entity_id` × `window_index`, so the
/// emit order does not affect the resulting case file; we still pin the
/// order for byte-level replay determinism.
#[must_use]
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "Cell-value lowering is deterministic-by-construction and recorded \
              in audit_report.html section 2; precision / sign / truncation \
              losses are intentional under the documented rule."
)]
pub fn lower_to_trace_events(
    fixture: &ResidualProjectionFixture,
    config: &LoweringConfig,
) -> Vec<TraceEvent> {
    let mut events: Vec<TraceEvent> = Vec::new();
    let scale = config.value_to_microsecond_scale as u64;
    let clamp = config.latency_clamp_us;
    let win_ns = config.window_size_ns;

    for (w_idx, row) in fixture.rows.iter().enumerate() {
        let ts_ns = (w_idx as u64).saturating_mul(win_ns);
        for (s_idx, cell) in row.iter().enumerate() {
            let Some(v) = cell else {
                continue;
            };
            // Deterministic clamp via integer math: floor(v * scale)
            // with non-negative saturation to [0, clamp].
            let latency_us = if !v.is_finite() || *v <= 0.0 {
                0_u32
            } else {
                // Multiply in f64 then cast; the audit report records this
                // explicitly so the conversion is non-mysterious. The
                // `scale` constant is well under 2^52 (default 1000) so
                // the f64 mantissa losses are not a concern in practice.
                let scaled = *v * scale as f64;
                if scaled >= clamp as f64 {
                    clamp
                } else {
                    scaled as u32
                }
            };
            let span_id = (w_idx as u64).saturating_mul(65_536) + s_idx as u64;
            events.push(TraceEvent::new(
                ts_ns,
                s_idx as u32, // entity_id
                0,            // route_id
                span_id,
                0, // parent_span_id
                latency_us,
                200, // status_code (ok)
                0,   // error_code
                0,   // event_kind
                0,   // flags
            ));
        }
    }
    events
}

/// Build the ingest report from a fixture + emitted events.
///
/// WHY: Audit transparency. The HTML report shows the operator exactly how
/// many cells were skipped and how many events were emitted so they can
/// see the projection's information loss before reading any episode list.
#[must_use]
pub fn build_ingest_report(
    fixture: &ResidualProjectionFixture,
    events: &[TraceEvent],
    fixture_byte_size: u64,
) -> IngestReport {
    let observed_windows = fixture.rows.len() as u32;
    let observed_signals = fixture.declared_num_signals;
    let total_cells = fixture.rows.iter().map(Vec::len).sum::<usize>() as u32;
    let nan_cell_count = fixture
        .rows
        .iter()
        .flat_map(|r| r.iter())
        .filter(|c| c.is_none())
        .count() as u32;
    IngestReport {
        fixture_sha256_hex: fixture.fixture_sha256_hex.clone(),
        fixture_byte_size,
        declared_num_windows: fixture.declared_num_windows,
        declared_num_signals: fixture.declared_num_signals,
        observed_num_windows: observed_windows,
        observed_num_signals: observed_signals,
        nan_cell_count,
        finite_cell_count: total_cells.saturating_sub(nan_cell_count),
        emitted_event_count: events.len() as u32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // SHA-256 pin for the vendored `data/fixtures/aiops_challenge.tsv`.
    // Mirrors the `aiops_kpi` entry in `s_real_audit::AUDIT_DATASETS`.
    // Refreshed at S-REAL.3.1.2 — the prior hard-coded value
    // (29961b8b6...) had become stale relative to the on-disk
    // fixture, breaking 4 ingest unit tests in the post-PAPER.1c
    // baseline. The mirror is now load-bearing: the
    // `audit_dataset_tier_dir_matches_bundle_manifest` test in
    // `s_real_audit::tests` cross-validates the audit driver's table
    // against `reports/s_real_3/bundle_manifest.toml`; a future SHA
    // change to the AIOps fixture requires refreshing both pins
    // (this constant AND `AUDIT_DATASETS[aiops_kpi].fixture_sha256_hex`)
    // in the same atomic commit.
    const PIN_AIOPS: &str = "be17110ebe6647d00fad79dc1ca69b1b01b22788773202bad6e3322e97b0602e";

    fn aiops_path() -> std::path::PathBuf {
        // Resolve the vendored AIOps fixture relative to this crate's
        // manifest dir so unit tests do not depend on any external
        // repository being mounted at a fixed absolute path. The
        // workspace root is two levels up from the crate manifest dir;
        // the fixture lives at `<workspace>/data/fixtures/...`.
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
        std::path::PathBuf::from(manifest_dir)
            .parent()
            .and_then(std::path::Path::parent)
            .expect("workspace root")
            .join("data/fixtures/aiops_challenge.tsv")
    }

    #[test]
    fn sha256_to_hex_lower_roundtrip() {
        let bytes = sha256(b"hello world");
        let hex = sha256_to_hex_lower(&bytes);
        assert_eq!(hex.len(), 64);
        assert!(hex
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }

    #[test]
    fn verify_fixture_sha256_admits_correct_pin() {
        let bytes = std::fs::read(aiops_path()).expect("read aiops fixture");
        let actual = verify_fixture_sha256(&bytes, PIN_AIOPS).expect("admit");
        assert_eq!(actual, PIN_AIOPS);
    }

    #[test]
    fn verify_fixture_sha256_rejects_uppercase_hex() {
        let bytes = std::fs::read(aiops_path()).expect("read aiops fixture");
        let upper: String = PIN_AIOPS.chars().map(|c| c.to_ascii_uppercase()).collect();
        match verify_fixture_sha256(&bytes, &upper) {
            Err(IngestError::Sha256NotLowerHex64 { .. }) => {}
            other => panic!("expected Sha256NotLowerHex64, got {other:?}"),
        }
    }

    #[test]
    fn verify_fixture_sha256_rejects_corrupted_bytes() {
        let mut bytes = std::fs::read(aiops_path()).expect("read aiops fixture");
        // flip one byte to simulate corruption (avoid the SHA-256 header
        // bytes — we want the change to affect the hash deterministically)
        if !bytes.is_empty() {
            let mid = bytes.len() / 2;
            bytes[mid] ^= 0x01;
        }
        match verify_fixture_sha256(&bytes, PIN_AIOPS) {
            Err(IngestError::Sha256Mismatch { .. }) => {}
            other => panic!("expected Sha256Mismatch, got {other:?}"),
        }
    }

    #[test]
    fn load_aiops_admits_canonical_shape() {
        let bytes = std::fs::read(aiops_path()).expect("read aiops fixture");
        let fixture = load_residual_projection_tsv(&bytes, PIN_AIOPS).expect("load");
        assert_eq!(fixture.declared_num_signals, 4);
        assert_eq!(fixture.declared_num_windows, 32);
        // AIOps file has no NaN; every cell parses.
        for row in &fixture.rows {
            assert_eq!(row.len(), 4);
            assert!(row.iter().all(Option::is_some));
        }
        assert!(fixture.metadata.contains_key("license"));
    }

    #[test]
    fn lower_to_trace_events_canonical_ordering() {
        let bytes = std::fs::read(aiops_path()).expect("read aiops fixture");
        let fixture = load_residual_projection_tsv(&bytes, PIN_AIOPS).expect("load");
        let cfg = LoweringConfig::default();
        let events = lower_to_trace_events(&fixture, &cfg);
        assert_eq!(
            events.len(),
            fixture
                .rows
                .iter()
                .map(|r| r.iter().filter(|c| c.is_some()).count())
                .sum::<usize>()
        );
        // First event maps to window 0 / entity 0
        assert_eq!(events[0].ts_ns, 0);
        assert_eq!(events[0].entity_id, 0);
        // Within a window, entity_id ascends.
        let signals = fixture.declared_num_signals as usize;
        for i in 1..signals.min(events.len()) {
            assert!(events[i].entity_id >= events[i - 1].entity_id);
        }
    }

    #[test]
    fn lowering_is_deterministic_across_two_runs() {
        let bytes = std::fs::read(aiops_path()).expect("read aiops fixture");
        let a = load_residual_projection_tsv(&bytes, PIN_AIOPS).expect("load");
        let b = load_residual_projection_tsv(&bytes, PIN_AIOPS).expect("load");
        let cfg = LoweringConfig::default();
        let ea = lower_to_trace_events(&a, &cfg);
        let eb = lower_to_trace_events(&b, &cfg);
        assert_eq!(ea, eb);
    }

    #[test]
    fn empty_file_rejected() {
        match load_residual_projection_tsv(b"", "0".repeat(64).as_str()) {
            Err(IngestError::EmptyFile) => {}
            other => panic!("expected EmptyFile, got {other:?}"),
        }
    }
}
