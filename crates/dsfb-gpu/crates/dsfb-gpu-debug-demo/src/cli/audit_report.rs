//! S-REAL.1 — Deterministic, no-JavaScript HTML audit report renderer.
//!
//! WHY: The S-REAL.1 audit gauntlet's central credibility deliverable is
//! that a human reader can open `reports/s_real_1/<dataset>/audit_report.html`
//! in any browser, see exactly what DSFB-GPU saw on the upstream fixture,
//! cross-check every hash against the receipt files, and confirm that two
//! consecutive dispatches produced byte-identical output. The renderer
//! must therefore be:
//!
//! - **Byte-stable**: same inputs → same byte stream across two calls and
//!   across two machines. We enforce this by sorting all map iteration
//!   (BTreeMap), pre-sorting the episode list by `(entity_id,
//!   start_window, end_window)`, and never emitting wall-clock timestamps,
//!   process IDs, or any other non-pinned dynamic content.
//! - **No JavaScript**: the report renders entirely server-side; the
//!   HTML carries inline CSS for layout. A reader on an air-gapped audit
//!   workstation can open the file without any network or runtime
//!   dependency.
//! - **No charts (v1)**: charts would require either an external library
//!   (network dependency) or a hand-rolled SVG renderer whose visual
//!   conventions would themselves need provenance. Tables of admitted
//!   episodes + stage hashes are the honest v1 surface; charts/heatmaps
//!   are S-REAL.1.1 scope.
//!
//! Section order (panel-locked):
//!
//! 1. Input provenance — fixture identity + SHA-256 byte pin.
//! 2. Residual-projection lowering law — exact rule used to map cells
//!    into TraceEvents.
//! 3. Run configuration — contract grid, scale parameters, episode count.
//! 4. Admitted episodes — table of bank-admitted episodes with motif +
//!    reason + peak fields.
//! 5. Stage digest / hash chain — every per-stage hash + final
//!    case-file hash.
//! 6. Replay verification — byte-identity between two consecutive runs.
//! 7. Limitations and non-claims — verbatim panel-locked text.
//!
//! Non-claims (rendered verbatim in section 7 of every report):
//! - Does NOT claim DSFB has identified the "real" anomaly in the dataset.
//! - Does NOT claim DSFB outperforms any other anomaly detector.
//! - Does NOT claim DSFB has discovered causality.
//! - Does NOT claim fitness-for-purpose on regulated / safety-critical use.
//! - Does NOT claim the dataset is "correctly labeled" or "ground truth".
//! - Does NOT claim the corpus or registry is exhaustive.
//! - Does NOT claim replay determinism across driver / CUDA versions; the
//!   replay receipt records the toolchain explicitly.
//!
//! License: Apache-2.0. Background IP: Invariant Forge LLC.

use std::collections::BTreeMap;
use std::fmt::Write;

use dsfb_gpu_debug_core::bank::Episode;
use dsfb_gpu_debug_core::casefile::CaseFile;

use super::ingest::{IngestReport, LoweringConfig};

/// Provenance + identity for the dataset under audit.
///
/// WHY: Every receipt the audit emits must cite the upstream source by
/// DOI/URL, the license under which the fixture is reused, the vendored
/// path the audit actually read, and the SHA-256 of the bytes consumed.
/// Carrying these together in one struct prevents partial citations.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DatasetManifest {
    /// Short identifier used as the directory name (e.g.
    /// `"tadbench_f11"`). Lowercase, no spaces.
    pub dataset_id: String,
    /// Human-readable display name (e.g.
    /// `"TADBench TrainTicket F11"`).
    pub display_name: String,
    /// Upstream DOI, URL, or repository reference. Honest non-claim:
    /// the audit cites this for provenance; it does not endorse the
    /// upstream's labels or methodology.
    pub upstream_doi_or_url: String,
    /// License under which the audit reuses the fixture bytes
    /// (`"Apache-2.0"`, `"CC0-1.0"`, etc.).
    pub license: String,
    /// DSFB source-class tag for the dataset (e.g.
    /// `"DebuggingSoftwareTelemetry"`,
    /// `"ObservabilityTraces"`,
    /// `"TimeSeriesAnomaly"`).
    pub source_class: String,
    /// Path the audit actually read from (the vendored fixture path).
    pub vendored_path: String,
    /// SHA-256 of the fixture file bytes, lowercase hex, 64 chars.
    pub fixture_sha256_hex: String,
    /// Size of the fixture file in bytes.
    pub fixture_byte_size: u64,
}

/// Schema map describing how upstream cells project into DSFB-GPU events.
///
/// WHY: The audit's transparency story rests on the reader being able to
/// reconstruct the entire ingest path from this struct. Every parameter
/// that affects the emitted `Vec<TraceEvent>` is recorded here.
#[derive(Clone, PartialEq, Debug)]
pub struct SchemaMap {
    /// Declared `num_windows` from the upstream header.
    pub declared_num_windows: u32,
    /// Declared `num_signals` from the upstream header.
    pub declared_num_signals: u32,
    /// Declared `healthy_window_end` from the upstream header. Carried for
    /// reader transparency; the lowering does NOT pre-split by it.
    pub declared_healthy_window_end: u32,
    /// Observed windows in the actual data rows.
    pub observed_num_windows: u32,
    /// Observed columns per row.
    pub observed_num_signals: u32,
    /// NaN-cell count (skipped by the lowering rule).
    pub nan_cell_count: u32,
    /// Finite-cell count (each becomes one event).
    pub finite_cell_count: u32,
    /// Number of events emitted into the dispatcher.
    pub emitted_event_count: u32,
    /// Lowering-rule parameters used.
    pub lowering_config: LoweringConfig,
}

impl From<&IngestReport> for SchemaMap {
    fn from(r: &IngestReport) -> Self {
        Self {
            declared_num_windows: r.declared_num_windows,
            declared_num_signals: r.declared_num_signals,
            declared_healthy_window_end: 0,
            observed_num_windows: r.observed_num_windows,
            observed_num_signals: r.observed_num_signals,
            nan_cell_count: r.nan_cell_count,
            finite_cell_count: r.finite_cell_count,
            emitted_event_count: r.emitted_event_count,
            lowering_config: LoweringConfig::default(),
        }
    }
}

/// Replay-verification metadata.
///
/// WHY: The audit's load-bearing replay claim is "two consecutive
/// dispatches on the same input bytes produce byte-identical CaseFile +
/// episodes + report". This struct carries the per-run SHA-256 of each
/// load-bearing artifact so the audit_report.html can publish a side-by-
/// side comparison.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ReplayVerification {
    pub run_count: u32,
    /// Lowercase hex SHA-256 of the canonical casefile.json bytes from
    /// run 1 and run 2 (or `"unmeasured"` if only one run was performed).
    pub casefile_json_sha256_run1: String,
    pub casefile_json_sha256_run2: String,
    /// Lowercase hex SHA-256 of the episodes.jsonl bytes.
    pub episodes_jsonl_sha256_run1: String,
    pub episodes_jsonl_sha256_run2: String,
    /// Hex of the `CaseFile::final_case_file_hash` field from each run.
    pub final_case_file_hash_run1_hex: String,
    pub final_case_file_hash_run2_hex: String,
    /// Episode count from each run (must agree for the audit to admit).
    pub episode_count_run1: u32,
    pub episode_count_run2: u32,
    /// Toolchain identity (compiler version, CUDA driver, hardware).
    /// Sorted BTreeMap so the rendered output is deterministic.
    pub toolchain: BTreeMap<String, String>,
}

impl ReplayVerification {
    /// True when run1 and run2 produced byte-identical CaseFile JSON,
    /// episodes JSONL, final case-file hash, and episode count.
    #[must_use]
    pub fn admits(&self) -> bool {
        self.casefile_json_sha256_run1 == self.casefile_json_sha256_run2
            && self.episodes_jsonl_sha256_run1 == self.episodes_jsonl_sha256_run2
            && self.final_case_file_hash_run1_hex == self.final_case_file_hash_run2_hex
            && self.episode_count_run1 == self.episode_count_run2
    }
}

/// HTML-escape a free-form string. Only the four characters that affect
/// HTML parsing (`<`, `>`, `&`, `"`) are escaped; everything else is
/// passed through verbatim so that hash hex / dataset metadata renders
/// without surprise.
fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '&' => out.push_str("&amp;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

/// Convert 32 raw bytes (e.g. a stage digest from `CaseFile::hashes`)
/// to a lowercase hex string.
fn hex(bytes: &[u8; 32]) -> String {
    super::ingest::sha256_to_hex_lower(bytes)
}

/// Render the complete audit report as deterministic, no-JavaScript HTML.
#[allow(
    clippy::too_many_lines,
    reason = "Renderer is intentionally one long byte-stable string \
              builder; splitting into helpers risks accidental ordering \
              divergence between two builds."
)]
///
/// WHY: One function consumes every input the audit cares about and
/// returns a single byte-stable string. Callers can write the result
/// directly to `audit_report.html`. Two consecutive calls with identical
/// inputs return identical strings (verified by the S-REAL.1 replay
/// acceptance test).
///
/// # Determinism guarantees
///
/// - Episode iteration uses a pre-sorted view by `(entity_id,
///   start_window, end_window)`; the input slice is not modified.
/// - Toolchain metadata renders in BTreeMap key order.
/// - No wall-clock timestamps, process IDs, or environment-derived
///   dynamic content appears in the output.
/// - Hash bytes always render as lowercase hex.
#[must_use]
pub fn render_audit_report_html(
    manifest: &DatasetManifest,
    schema: &SchemaMap,
    case: &CaseFile,
    replay: &ReplayVerification,
) -> String {
    let mut h = String::with_capacity(8192);
    h.push_str("<!DOCTYPE html>\n");
    h.push_str("<html lang=\"en\">\n");
    h.push_str("<head>\n");
    h.push_str("<meta charset=\"utf-8\">\n");
    let _ = writeln!(
        h,
        "<title>S-REAL.1 audit — {}</title>",
        escape(&manifest.display_name)
    );
    h.push_str("<style>\n");
    h.push_str(STYLE);
    h.push_str("</style>\n");
    h.push_str("</head>\n");
    h.push_str("<body>\n");

    // Header
    let _ = writeln!(
        h,
        "<h1>DSFB-GPU S-REAL.1 audit — {}</h1>",
        escape(&manifest.display_name)
    );
    h.push_str(
        "<p class=\"subhead\">Deterministic residual-densor audit on real public dataset bytes. ",
    );
    h.push_str("Apache-2.0 reference implementation. Background IP: Invariant Forge LLC.</p>\n");

    // S-REAL.1.1: Top-of-report summary card. 30-second operator
    // comprehension target: shape, finite cells, skipped NaNs,
    // events, episodes, replay verdict, truncated final hash.
    section_summary_card(&mut h, manifest, schema, case, replay);

    // S-REAL.1.1.1: Replay proof card immediately below the summary
    // card so an operator sees the byte-identical-replay verdict +
    // per-artifact SHA-256s at a glance, without scrolling to
    // section 6.
    section_replay_proof_card(&mut h, case, replay);

    section_input_provenance(&mut h, manifest);
    section_lowering_law(&mut h, schema);
    section_run_configuration(&mut h, schema, case);
    section_admitted_episodes(&mut h, case);

    // S-REAL.1.1 aggregations: motif histogram + reason-code
    // histogram with prose + entity summary + episode timeline +
    // top structural spans (3 sub-tables) + plain-English motif
    // glossary. All rendered deterministically; all derived from
    // case.episodes only (no extra dispatcher state).
    section_motif_histogram(&mut h, case);
    section_reason_code_histogram(&mut h, case);
    section_entity_summary(&mut h, case);
    section_episode_timeline(&mut h, case);
    section_top_structural_spans(&mut h, case);
    section_motif_glossary(&mut h);

    section_stage_hash_chain(&mut h, case);
    section_replay_verification(&mut h, replay);
    section_limitations(&mut h);

    h.push_str("</body>\n</html>\n");
    h
}

fn section_input_provenance(h: &mut String, m: &DatasetManifest) {
    h.push_str("<h2>1. Input provenance</h2>\n");
    h.push_str("<table class=\"kv\">\n");
    kv(h, "dataset_id", &m.dataset_id);
    kv(h, "display_name", &m.display_name);
    kv(h, "upstream_doi_or_url", &m.upstream_doi_or_url);
    kv(h, "license", &m.license);
    kv(h, "source_class", &m.source_class);
    kv(h, "vendored_path", &m.vendored_path);
    kv(h, "fixture_sha256", &m.fixture_sha256_hex);
    kv(h, "fixture_byte_size", &m.fixture_byte_size.to_string());
    h.push_str("</table>\n");
    h.push_str("<p class=\"note\">The fixture bytes were SHA-256-verified before parsing. ");
    h.push_str("Any divergence from the pinned hash would have aborted the audit before any event was emitted.</p>\n");
}

fn section_lowering_law(h: &mut String, s: &SchemaMap) {
    h.push_str("<h2>2. Residual-projection lowering law</h2>\n");
    h.push_str("<p>The upstream fixture is in <code>residual-projection v2</code> form ");
    h.push_str("(window-major × signal-minor TSV with NaN cells). The audit deterministically ");
    h.push_str(
        "lowers each finite cell into one <code>TraceEvent</code> via the rule below:</p>\n",
    );
    h.push_str("<pre class=\"law\">");
    h.push_str("For each (window_idx, signal_idx, value) in fixture.rows.iter().enumerate()\n");
    h.push_str(
        "                                            .flat_map(|(w, row)| row.iter().enumerate()\n",
    );
    h.push_str("                                                                  .map(move |(s, v)| (w, s, v))):\n");
    h.push_str("  if value is None (nan): skip; no event emitted for this cell\n");
    h.push_str("  else:\n");
    h.push_str("    ts_ns         = window_idx * window_size_ns\n");
    h.push_str("    entity_id     = signal_idx\n");
    h.push_str("    route_id      = 0\n");
    h.push_str("    span_id       = window_idx * 65536 + signal_idx\n");
    h.push_str("    parent_span_id = 0\n");
    h.push_str(
        "    latency_us    = clamp(value * value_to_microsecond_scale, 0, latency_clamp_us)\n",
    );
    h.push_str("    status_code   = 200\n");
    h.push_str("    error_code    = 0\n");
    h.push_str("    event_kind    = 0\n");
    h.push_str("    flags         = 0\n");
    h.push_str("</pre>\n");
    h.push_str("<table class=\"kv\">\n");
    kv(
        h,
        "declared_num_windows",
        &s.declared_num_windows.to_string(),
    );
    kv(
        h,
        "declared_num_signals",
        &s.declared_num_signals.to_string(),
    );
    kv(
        h,
        "declared_healthy_window_end",
        &s.declared_healthy_window_end.to_string(),
    );
    kv(
        h,
        "observed_num_windows",
        &s.observed_num_windows.to_string(),
    );
    kv(
        h,
        "observed_num_signals",
        &s.observed_num_signals.to_string(),
    );
    kv(h, "nan_cell_count", &s.nan_cell_count.to_string());
    kv(h, "finite_cell_count", &s.finite_cell_count.to_string());
    kv(h, "emitted_event_count", &s.emitted_event_count.to_string());
    kv(
        h,
        "value_to_microsecond_scale",
        &s.lowering_config.value_to_microsecond_scale.to_string(),
    );
    kv(
        h,
        "latency_clamp_us",
        &s.lowering_config.latency_clamp_us.to_string(),
    );
    kv(
        h,
        "window_size_ns",
        &s.lowering_config.window_size_ns.to_string(),
    );
    h.push_str("</table>\n");
    h.push_str("<p class=\"note\">NaN cells produce no event. The audit ");
    h.push_str("does not claim DSFB-GPU saw the upstream's original trace ");
    h.push_str("events; it claims DSFB-GPU saw exactly the events the rule ");
    h.push_str("above produces from these bytes.</p>\n");
}

fn section_run_configuration(h: &mut String, s: &SchemaMap, c: &CaseFile) {
    h.push_str("<h2>3. Run configuration</h2>\n");
    h.push_str("<table class=\"kv\">\n");
    kv(h, "casefile_version", c.version);
    kv(h, "backend", c.backend);
    kv(
        h,
        "n_entities (= observed_num_signals)",
        &s.observed_num_signals.to_string(),
    );
    kv(
        h,
        "n_windows (= observed_num_windows)",
        &s.observed_num_windows.to_string(),
    );
    kv(h, "events_dispatched", &s.emitted_event_count.to_string());
    kv(h, "episodes_admitted", &c.episodes.len().to_string());
    kv(h, "final_verdict", c.final_verdict.name());
    h.push_str("</table>\n");
}

fn section_admitted_episodes(h: &mut String, case: &CaseFile) {
    h.push_str("<h2>4. Admitted episodes</h2>\n");
    if case.episodes.is_empty() {
        h.push_str("<p class=\"note\">No episodes were admitted on this fixture. ");
        h.push_str("Per the Semantic Non-Bypass Axiom the bank stage admitted zero. ");
        h.push_str("This is a valid honest outcome — DSFB-GPU saw the fixture and ");
        h.push_str("found no admissible motif under the canonical bank + detector registry.</p>\n");
        return;
    }
    h.push_str("<p class=\"note\">Episodes are listed in canonical order by ");
    h.push_str("<code>(entity_id, start_window, end_window)</code>. Each row reports the ");
    h.push_str("bank motif, reason code, and peak Q16.16 magnitudes the bank used to admit.</p>\n");

    let mut sorted: Vec<&Episode> = case.episodes.iter().collect();
    sorted.sort_by_key(|e| (e.entity_id, e.start_window, e.end_window));

    h.push_str("<table class=\"episodes\">\n");
    h.push_str("<thead><tr>");
    h.push_str("<th>idx</th><th>entity_id</th><th>start_window</th><th>end_window</th>");
    h.push_str("<th>motif</th><th>reason</th><th>peak_state</th>");
    h.push_str("<th>peak_residual_q</th><th>peak_drift_q</th><th>peak_slew_q</th>");
    h.push_str("<th>detector_bit_count</th>");
    h.push_str("</tr></thead>\n");
    h.push_str("<tbody>\n");
    for (idx, e) in sorted.iter().enumerate() {
        let _ = writeln!(
            h,
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            idx,
            e.entity_id,
            e.start_window,
            e.end_window,
            escape(motif_name(e.motif)),
            escape(reason_name(e.reason)),
            escape(grammar_name(e.peak_state)),
            e.peak_residual_q.0,
            e.peak_drift_q.0,
            e.peak_slew_q.0,
            e.detector_bit_count,
        );
    }
    h.push_str("</tbody>\n</table>\n");
}

fn section_stage_hash_chain(h: &mut String, c: &CaseFile) {
    h.push_str("<h2>5. Stage digest / hash chain</h2>\n");
    h.push_str("<p class=\"note\">Every per-stage hash links into the next ");
    h.push_str("via the canonical DSFB-GPU chain. A reader can replay the ");
    h.push_str("dispatch and confirm every hex value below byte-for-byte.</p>\n");
    h.push_str("<table class=\"hashes\">\n");
    h.push_str("<thead><tr><th>chain link</th><th>sha256 (hex)</th></tr></thead>\n");
    h.push_str("<tbody>\n");
    hash_row(h, "h_input_catalog", &c.hashes.input_catalog);
    hash_row(h, "h_contract", &c.hashes.contract);
    hash_row(h, "h_bank", &c.hashes.bank);
    hash_row(h, "h_detector_registry", &c.hashes.detector_registry);
    hash_row(h, "h_kernel_sequence", &c.hashes.kernel_sequence);
    hash_row(h, "h_window_feature", &c.hashes.window_feature);
    hash_row(h, "h_residual_field", &c.hashes.residual_field);
    hash_row(h, "h_sign_field", &c.hashes.sign_field);
    hash_row(h, "h_detector_cell", &c.hashes.detector_cell);
    hash_row(h, "h_consensus_grid", &c.hashes.consensus_grid);
    hash_row(h, "h_candidate_interval", &c.hashes.candidate_interval);
    hash_row(h, "h_episode", &c.hashes.episode);
    hash_row(h, "final_case_file_hash", &c.final_case_file_hash);
    h.push_str("</tbody>\n</table>\n");
}

fn section_replay_verification(h: &mut String, r: &ReplayVerification) {
    h.push_str("<h2>6. Replay verification</h2>\n");
    h.push_str("<table class=\"kv\">\n");
    kv(h, "run_count", &r.run_count.to_string());
    kv(h, "casefile_json_sha256_run1", &r.casefile_json_sha256_run1);
    kv(h, "casefile_json_sha256_run2", &r.casefile_json_sha256_run2);
    kv(
        h,
        "episodes_jsonl_sha256_run1",
        &r.episodes_jsonl_sha256_run1,
    );
    kv(
        h,
        "episodes_jsonl_sha256_run2",
        &r.episodes_jsonl_sha256_run2,
    );
    kv(
        h,
        "final_case_file_hash_run1",
        &r.final_case_file_hash_run1_hex,
    );
    kv(
        h,
        "final_case_file_hash_run2",
        &r.final_case_file_hash_run2_hex,
    );
    kv(h, "episode_count_run1", &r.episode_count_run1.to_string());
    kv(h, "episode_count_run2", &r.episode_count_run2.to_string());
    kv(
        h,
        "byte_identical_replay",
        if r.admits() { "true" } else { "false" },
    );
    h.push_str("</table>\n");
    h.push_str("<h3>Toolchain identity</h3>\n");
    h.push_str("<table class=\"kv\">\n");
    for (k, v) in &r.toolchain {
        kv(h, k, v);
    }
    h.push_str("</table>\n");
    h.push_str("<p class=\"note\">Replay determinism is asserted only for ");
    h.push_str("the recorded toolchain. The audit does not claim replay ");
    h.push_str("byte-identity across different driver, CUDA, or hardware versions.</p>\n");
}

fn section_limitations(h: &mut String) {
    h.push_str("<h2>7. Limitations and non-claims</h2>\n");
    h.push_str("<ul class=\"nonclaims\">\n");
    for nc in NON_CLAIMS {
        let _ = writeln!(h, "<li>{}</li>", escape(nc));
    }
    h.push_str("</ul>\n");
}

/// Convenience: emit one `<tr><th>key</th><td>value</td></tr>` row.
fn kv(h: &mut String, key: &str, value: &str) {
    let _ = writeln!(
        h,
        "<tr><th>{}</th><td>{}</td></tr>",
        escape(key),
        escape(value)
    );
}

fn hash_row(h: &mut String, label: &str, bytes: &[u8; 32]) {
    let _ = writeln!(
        h,
        "<tr><td class=\"label\">{}</td><td class=\"hex\">{}</td></tr>",
        escape(label),
        hex(bytes)
    );
}

// =====================================================================
// S-REAL.1.1 renderer additions (operator-facing richness).
// =====================================================================
//
// Panel-locked goal: a human reading the report should understand what
// DSFB-GPU saw in 30 seconds. The seven additions below trade no
// hash-chain semantics or replay byte-identity to get there — every
// section is a deterministic projection over `case.episodes` already
// pinned by the chain.

/// 30-second operator summary card rendered at the top of the report.
///
/// WHY: An operator opening the audit report should see the
/// load-bearing facts (shape, finite cells, NaN skip count, events,
/// episodes admitted, replay verdict) WITHOUT scrolling. Every field
/// is also re-rendered later in its own dedicated section, but this
/// card pins the punch-line for the impatient reader.
fn section_summary_card(
    h: &mut String,
    m: &DatasetManifest,
    s: &SchemaMap,
    case: &CaseFile,
    replay: &ReplayVerification,
) {
    h.push_str("<div class=\"summary-card\">\n");
    h.push_str("<h2 class=\"summary-title\">Dataset summary</h2>\n");
    h.push_str("<table class=\"kv summary-kv\">\n");
    kv(h, "dataset", &m.display_name);
    kv(h, "source class", &m.source_class);
    kv(
        h,
        "shape (entities × windows)",
        &format!("{} × {}", s.observed_num_signals, s.observed_num_windows),
    );
    kv(
        h,
        "finite cells (events emitted)",
        &s.finite_cell_count.to_string(),
    );
    kv(h, "NaN cells skipped", &s.nan_cell_count.to_string());
    kv(h, "episodes admitted", &case.episodes.len().to_string());
    kv(
        h,
        "byte-identical replay",
        if replay.admits() { "YES" } else { "NO" },
    );
    // Truncated final_case_file_hash for at-a-glance chain-head pinning;
    // the full hex appears in the Replay Proof card below + section 5.
    kv(
        h,
        "final_case_file_hash (first 16 hex)",
        &hex_truncated_16(&case.final_case_file_hash),
    );
    kv(h, "final_verdict", case.final_verdict.name());
    h.push_str("</table>\n");
    h.push_str("</div>\n");
}

/// Replay Proof card rendered immediately below the summary card.
///
/// WHY: An operator reading the audit should see the replay chain
/// confidence at a glance — without scrolling to section 6. The card
/// surfaces the byte-identical-replay verdict + per-artifact SHA-256s
/// for casefile.json and episodes.jsonl + the final_case_file_hash.
///
/// `audit_report.html` cannot carry its own SHA-256 by construction
/// (the rendered HTML would have to contain its own hash, which is
/// computationally infeasible — equivalent to finding a SHA-256
/// pre-image of the rendered string within the rendered string). The
/// audit_report.html's authoritative SHA-256 is computed AFTER
/// rendering and pinned in `replay_verification.txt`; the card
/// explicitly cites that pin location instead of fabricating a
/// self-referential value.
fn section_replay_proof_card(h: &mut String, case: &CaseFile, replay: &ReplayVerification) {
    h.push_str("<div class=\"replay-proof-card\">\n");
    h.push_str("<h2 class=\"summary-title\">Replay proof</h2>\n");
    h.push_str("<table class=\"kv summary-kv\">\n");
    kv(
        h,
        "byte-identical replay",
        if replay.admits() { "YES" } else { "NO" },
    );
    kv(
        h,
        "casefile.json SHA-256",
        &replay.casefile_json_sha256_run1,
    );
    kv(
        h,
        "episodes.jsonl SHA-256",
        &replay.episodes_jsonl_sha256_run1,
    );
    kv(
        h,
        "audit_report.html SHA-256",
        "(externally pinned in replay_verification.txt — not embedded here because a self-referential hash is computationally infeasible)",
    );
    kv(
        h,
        "final_case_file_hash (full)",
        &super::ingest::sha256_to_hex_lower(&case.final_case_file_hash),
    );
    kv(
        h,
        "episode count (run 1)",
        &replay.episode_count_run1.to_string(),
    );
    kv(
        h,
        "episode count (run 2)",
        &replay.episode_count_run2.to_string(),
    );
    h.push_str("</table>\n");
    h.push_str("</div>\n");
}

/// Truncate a 32-byte hash to the first 16 hex characters (8 bytes).
/// Used in the dataset summary card for at-a-glance chain-head
/// pinning. The full hex appears in the Replay Proof card and
/// section 5.
fn hex_truncated_16(bytes: &[u8; 32]) -> String {
    let full = super::ingest::sha256_to_hex_lower(bytes);
    full.chars().take(16).collect()
}

/// Motif histogram sorted ascending by motif wire name.
///
/// WHY: After section 4 lists every admitted episode in canonical order,
/// the histogram answers "which motifs dominated this dataset?" at a
/// glance. Counts + percentages are exact integers (no f64) so the
/// rendered output is byte-stable across two calls.
fn section_motif_histogram(h: &mut String, case: &CaseFile) {
    h.push_str("<h3>4a. Motif histogram</h3>\n");
    if case.episodes.is_empty() {
        h.push_str("<p class=\"note\">No motifs fired on this dataset.</p>\n");
        return;
    }
    let mut counts: BTreeMap<&'static str, u32> = BTreeMap::new();
    for e in &case.episodes {
        *counts.entry(motif_name(e.motif)).or_insert(0) += 1;
    }
    let total = case.episodes.len() as u32;
    h.push_str("<table class=\"histogram\">\n");
    h.push_str("<thead><tr><th>motif</th><th>count</th><th>percent</th></tr></thead>\n<tbody>\n");
    for (name, count) in &counts {
        let pct_bp = (*count * 10_000) / total;
        let _ = writeln!(
            h,
            "<tr><td>{}</td><td>{}</td><td>{}.{:02} %</td></tr>",
            escape(name),
            count,
            pct_bp / 100,
            pct_bp % 100
        );
    }
    h.push_str("</tbody>\n</table>\n");
}

/// Reason-code histogram with plain-English meaning per row.
///
/// WHY: Reason codes are pinned wire-name enums; without the prose
/// column, an operator must look up what each code means. Folding the
/// prose into the histogram row makes the table self-documenting and
/// removes the need for a separate glossary section for reason codes.
/// Sorted descending by `ReasonCode::severity()` (canonical tie-break
/// from `dsfb_gpu_debug_core::grammar`), then ascending by wire name
/// for ties so the rendered output is deterministic.
fn section_reason_code_histogram(h: &mut String, case: &CaseFile) {
    use dsfb_gpu_debug_core::grammar::ReasonCode;
    h.push_str("<h3>4b. Reason-code histogram</h3>\n");
    if case.episodes.is_empty() {
        h.push_str("<p class=\"note\">No reason codes fired on this dataset.</p>\n");
        return;
    }
    let mut counts: BTreeMap<u8, (ReasonCode, u32)> = BTreeMap::new();
    for e in &case.episodes {
        counts.entry(e.reason as u8).or_insert((e.reason, 0)).1 += 1;
    }
    let total = case.episodes.len() as u32;

    // Sort by severity desc, then wire name asc.
    let mut rows: Vec<(ReasonCode, u32)> = counts.values().copied().collect();
    rows.sort_by(|a, b| {
        b.0.severity()
            .cmp(&a.0.severity())
            .then_with(|| reason_name(a.0).cmp(reason_name(b.0)))
    });

    h.push_str("<table class=\"histogram\">\n");
    h.push_str("<thead><tr><th>reason_code</th><th>count</th><th>percent</th><th>plain-English meaning</th></tr></thead>\n<tbody>\n");
    for (reason, count) in &rows {
        let pct_bp = (*count * 10_000) / total;
        let _ = writeln!(
            h,
            "<tr><td>{}</td><td>{}</td><td>{}.{:02} %</td><td>{}</td></tr>",
            escape(reason_name(*reason)),
            count,
            pct_bp / 100,
            pct_bp % 100,
            escape(reason_prose(*reason)),
        );
    }
    h.push_str("</tbody>\n</table>\n");
}

/// Per-entity summary table: one row per entity that admitted at least
/// one episode.
///
/// WHY: Entity-level aggregation answers "which entities are most
/// affected?" in one glance. Dominant motif (motif with the most
/// episodes on this entity; tie-broken by motif wire name ascending)
/// and first/last window give the operator a quick footprint.
fn section_entity_summary(h: &mut String, case: &CaseFile) {
    h.push_str("<h3>4c. Entity summary</h3>\n");
    if case.episodes.is_empty() {
        h.push_str("<p class=\"note\">No entities admitted episodes on this dataset.</p>\n");
        return;
    }
    // Aggregate by entity. Seed first/last window from the first
    // observation to avoid the u32::default() == 0 trap (min(0, x)
    // would always pick 0 and corrupt first_window).
    let mut per_entity: BTreeMap<u32, EntityAggregation> = BTreeMap::new();
    for e in &case.episodes {
        per_entity
            .entry(e.entity_id)
            .and_modify(|agg| {
                agg.episode_count += 1;
                agg.first_window = agg.first_window.min(e.start_window);
                agg.last_window = agg.last_window.max(e.end_window);
                agg.max_detector_bit_count = agg.max_detector_bit_count.max(e.detector_bit_count);
                *agg.motif_counts.entry(motif_name(e.motif)).or_insert(0) += 1;
            })
            .or_insert_with(|| {
                let mut m: BTreeMap<&'static str, u32> = BTreeMap::new();
                m.insert(motif_name(e.motif), 1);
                EntityAggregation {
                    episode_count: 1,
                    first_window: e.start_window,
                    last_window: e.end_window,
                    max_detector_bit_count: e.detector_bit_count,
                    motif_counts: m,
                }
            });
    }
    h.push_str("<table class=\"summary\">\n");
    h.push_str("<thead><tr><th>entity_id</th><th>episode_count</th><th>first_window</th><th>last_window</th><th>max_detector_bit_count</th><th>dominant_motif</th></tr></thead>\n<tbody>\n");
    for (entity_id, agg) in &per_entity {
        // Dominant motif: highest count; ties broken by name ascending
        // (BTreeMap iterates ascending so first match in the max-fold wins).
        let (dominant_motif, _) = agg
            .motif_counts
            .iter()
            .max_by(|a, b| a.1.cmp(b.1).then_with(|| b.0.cmp(a.0)))
            .map_or((&"-", &0u32), |(k, v)| (k, v));
        let _ = writeln!(
            h,
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            entity_id,
            agg.episode_count,
            agg.first_window,
            agg.last_window,
            agg.max_detector_bit_count,
            escape(dominant_motif),
        );
    }
    h.push_str("</tbody>\n</table>\n");
}

struct EntityAggregation {
    episode_count: u32,
    first_window: u32,
    last_window: u32,
    max_detector_bit_count: u32,
    motif_counts: BTreeMap<&'static str, u32>,
}

/// Episode timeline grouped by entity, sorted ascending by
/// `(start_window, end_window, motif as u8)` within each entity.
///
/// WHY: Section 4 lists every admitted episode in canonical order;
/// the timeline re-arranges them per-entity in time order so the
/// operator can scan "what happened on each entity in order" without
/// having to filter the master list mentally.
fn section_episode_timeline(h: &mut String, case: &CaseFile) {
    h.push_str("<h3>4d. Episode timeline (per-entity, time-ordered)</h3>\n");
    if case.episodes.is_empty() {
        h.push_str("<p class=\"note\">No episodes to plot.</p>\n");
        return;
    }
    // Bucket episodes by entity in BTreeMap (deterministic).
    let mut per_entity: BTreeMap<u32, Vec<&dsfb_gpu_debug_core::bank::Episode>> = BTreeMap::new();
    for e in &case.episodes {
        per_entity.entry(e.entity_id).or_default().push(e);
    }
    h.push_str("<table class=\"timeline\">\n");
    h.push_str("<thead><tr><th>entity_id</th><th>start_window</th><th>end_window</th><th>motif</th><th>reason</th></tr></thead>\n<tbody>\n");
    for (entity_id, episodes) in &mut per_entity {
        episodes.sort_by_key(|e| (e.start_window, e.end_window, e.motif as u8));
        for e in episodes {
            let _ = writeln!(
                h,
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                entity_id,
                e.start_window,
                e.end_window,
                escape(motif_name(e.motif)),
                escape(reason_name(e.reason)),
            );
        }
    }
    h.push_str("</tbody>\n</table>\n");
}

/// Per-entity aggregation for the highest-repeated-entities sub-table
/// of section 4e(b). `(episode_count, motif_counts, reason_counts)`.
/// Hoisted to a type alias because the inline tuple was clippy-noisy
/// without buying readability.
type EntityRankAggregation = (
    u32,
    BTreeMap<&'static str, u32>,
    BTreeMap<&'static str, u32>,
);

/// Top structural spans rendered as three deterministic sub-tables:
/// longest spans / highest-repeated entities / most-recurrent reason
/// codes.
///
/// WHY: An operator scanning the audit wants three different "top of
/// the heap" views: which episodes lasted longest, which entities
/// produced the most episodes, and which reason codes recurred most.
/// Each sub-table is small (top-5 or top-10) and deterministic; ties
/// are broken by stable secondary keys so two renders agree byte-for-
/// byte.
fn section_top_structural_spans(h: &mut String, case: &CaseFile) {
    use dsfb_gpu_debug_core::grammar::ReasonCode;
    h.push_str("<h3>4e. Top structural spans</h3>\n");
    if case.episodes.is_empty() {
        h.push_str("<p class=\"note\">No episodes to rank.</p>\n");
        return;
    }

    // (a) Longest spans — top-10 by (end-start) desc, ties broken by
    //     (detector_bit_count desc, entity asc, start asc).
    h.push_str("<h4>4e(a). Longest spans</h4>\n");
    let mut by_length: Vec<&dsfb_gpu_debug_core::bank::Episode> = case.episodes.iter().collect();
    by_length.sort_by(|a, b| {
        let la = a.end_window.saturating_sub(a.start_window);
        let lb = b.end_window.saturating_sub(b.start_window);
        lb.cmp(&la)
            .then_with(|| b.detector_bit_count.cmp(&a.detector_bit_count))
            .then_with(|| a.entity_id.cmp(&b.entity_id))
            .then_with(|| a.start_window.cmp(&b.start_window))
    });
    h.push_str("<table class=\"summary\">\n");
    h.push_str("<thead><tr><th>rank</th><th>length_windows</th><th>entity_id</th><th>start</th><th>end</th><th>motif</th><th>detector_bit_count</th></tr></thead>\n<tbody>\n");
    for (rank, e) in by_length.iter().take(10).enumerate() {
        let _ = writeln!(
            h,
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            rank + 1,
            e.end_window.saturating_sub(e.start_window),
            e.entity_id,
            e.start_window,
            e.end_window,
            escape(motif_name(e.motif)),
            e.detector_bit_count,
        );
    }
    h.push_str("</tbody>\n</table>\n");

    // (b) Highest-repeated entities — top-5 entities by episode_count
    //     desc, ties broken by entity_id asc.
    h.push_str("<h4>4e(b). Highest-repeated entities</h4>\n");
    let mut per_entity: BTreeMap<u32, EntityRankAggregation> = BTreeMap::new();
    for e in &case.episodes {
        let agg = per_entity.entry(e.entity_id).or_default();
        agg.0 += 1;
        *agg.1.entry(motif_name(e.motif)).or_insert(0) += 1;
        *agg.2.entry(reason_name(e.reason)).or_insert(0) += 1;
    }
    let mut entity_ranks: Vec<(u32, u32, &'static str, &'static str)> = per_entity
        .iter()
        .map(|(entity_id, (count, motifs, reasons))| {
            let dom_motif = motifs
                .iter()
                .max_by(|a, b| a.1.cmp(b.1).then_with(|| b.0.cmp(a.0)))
                .map_or("-", |(k, _)| *k);
            let dom_reason = reasons
                .iter()
                .max_by(|a, b| a.1.cmp(b.1).then_with(|| b.0.cmp(a.0)))
                .map_or("-", |(k, _)| *k);
            (*entity_id, *count, dom_motif, dom_reason)
        })
        .collect();
    entity_ranks.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    h.push_str("<table class=\"summary\">\n");
    h.push_str("<thead><tr><th>rank</th><th>entity_id</th><th>episode_count</th><th>dominant_motif</th><th>dominant_reason_code</th></tr></thead>\n<tbody>\n");
    for (rank, (entity_id, count, motif, reason)) in entity_ranks.iter().take(5).enumerate() {
        let _ = writeln!(
            h,
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            rank + 1,
            entity_id,
            count,
            escape(motif),
            escape(reason),
        );
    }
    h.push_str("</tbody>\n</table>\n");

    // (c) Most-recurrent reason codes — top-5 by count desc, ties broken
    //     by severity desc then wire name asc.
    h.push_str("<h4>4e(c). Most-recurrent reason codes</h4>\n");
    let mut counts: BTreeMap<u8, (ReasonCode, u32)> = BTreeMap::new();
    for e in &case.episodes {
        counts.entry(e.reason as u8).or_insert((e.reason, 0)).1 += 1;
    }
    let mut rows: Vec<(ReasonCode, u32)> = counts.values().copied().collect();
    rows.sort_by(|a, b| {
        b.1.cmp(&a.1)
            .then_with(|| b.0.severity().cmp(&a.0.severity()))
            .then_with(|| reason_name(a.0).cmp(reason_name(b.0)))
    });
    h.push_str("<table class=\"summary\">\n");
    h.push_str("<thead><tr><th>rank</th><th>reason_code</th><th>count</th><th>severity</th></tr></thead>\n<tbody>\n");
    for (rank, (reason, count)) in rows.iter().take(5).enumerate() {
        let _ = writeln!(
            h,
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            rank + 1,
            escape(reason_name(*reason)),
            count,
            reason.severity(),
        );
    }
    h.push_str("</tbody>\n</table>\n");
}

/// Plain-English motif glossary, rendered as a table.
///
/// WHY: An operator encountering DSFB motifs for the first time needs
/// a structural-vocabulary key. The glossary lists ALL 8 motifs (even
/// ones that did not fire in this dataset) so the report is
/// self-documenting. Each prose line ends with the standardised tail
/// *"DSFB interprets this structurally, not as a ground-truth causal
/// diagnosis."* — the panel-locked non-overclaim boundary.
fn section_motif_glossary(h: &mut String) {
    h.push_str("<h3>4f. Plain-English motif glossary</h3>\n");
    h.push_str("<p class=\"note\">Every DSFB motif describes a STRUCTURAL ");
    h.push_str("residual shape. DSFB interprets each motif structurally, ");
    h.push_str("not as a ground-truth causal diagnosis. The glossary covers ");
    h.push_str("all eight motifs even if some did not fire on this dataset.</p>\n");
    h.push_str("<table class=\"summary glossary\">\n");
    h.push_str(
        "<thead><tr><th>motif</th><th>structural interpretation</th></tr></thead>\n<tbody>\n",
    );
    for (motif, prose) in MOTIF_PROSE {
        let _ = writeln!(
            h,
            "<tr><td class=\"label\">{}</td><td>{}</td></tr>",
            escape(motif_name(*motif)),
            escape(prose)
        );
    }
    h.push_str("</tbody>\n</table>\n");
}

/// Look up a reason code's plain-English meaning.
///
/// WHY: Reason codes are pinned wire-name enums; the prose makes the
/// reason-code histogram self-documenting. Each line ends with the
/// standardised structural-not-causal tail so the renderer cannot
/// accidentally claim ground-truth causality.
fn reason_prose(r: dsfb_gpu_debug_core::grammar::ReasonCode) -> &'static str {
    use dsfb_gpu_debug_core::grammar::ReasonCode;
    match r {
        ReasonCode::Admissible => {
            "Cell admitted as within the admissibility envelope; DSFB interprets this structurally, not as a ground-truth causal diagnosis."
        }
        ReasonCode::BoundaryApproach => {
            "Residual or drift entered the boundary band but did not cross the violation threshold; DSFB interprets this structurally, not as a ground-truth causal diagnosis."
        }
        ReasonCode::SustainedOutwardDrift => {
            "Drift remained above the violation threshold for multiple windows; DSFB interprets this structurally, not as a ground-truth causal diagnosis."
        }
        ReasonCode::AbruptSlewViolation => {
            "Single-window slew shock crossed the violation threshold; DSFB interprets this structurally, not as a ground-truth causal diagnosis."
        }
        ReasonCode::RecurrentBoundaryGrazing => {
            "Multiple boundary cells with no clear violation — repeated graze without commitment; DSFB interprets this structurally, not as a ground-truth causal diagnosis."
        }
        ReasonCode::EnvelopeViolation => {
            "Envelope-magnitude violation (norm itself crossed the high band); DSFB interprets this structurally, not as a ground-truth causal diagnosis."
        }
        ReasonCode::DriftWithRecovery => {
            "Drift descended after a peak — recovery edge; DSFB interprets this structurally, not as a ground-truth causal diagnosis."
        }
        ReasonCode::SingleCrossing => {
            "One-shot boundary crossing that did not re-enter on the next cell; DSFB interprets this structurally, not as a ground-truth causal diagnosis."
        }
    }
}

/// Pinned plain-English glossary for all 8 BankMotif variants.
///
/// WHY: Self-documenting glossary so an operator opening the audit
/// for the first time doesn't need an external DSFB reference. Each
/// line ends with the standardised non-overclaim tail so the
/// rendered report cannot accidentally claim causality.
const MOTIF_PROSE: &[(dsfb_gpu_debug_core::bank::BankMotif, &str)] = {
    use dsfb_gpu_debug_core::bank::BankMotif;
    &[
        (
            BankMotif::LatencyRamp,
            "Sustained directional increase in residual latency-projection cells across a contiguous window span; DSFB interprets this as recurrent directional latency structure, not as a ground-truth causal diagnosis.",
        ),
        (
            BankMotif::ErrorBurst,
            "Concentrated burst of error-projection cells in a short window range; DSFB interprets this as locally concentrated error structure, not as a ground-truth causal diagnosis.",
        ),
        (
            BankMotif::SlewShockRecovery,
            "Abrupt slew shock followed by a recovery edge in the same entity; DSFB interprets this as transient slew + recovery structure, not as a ground-truth causal diagnosis.",
        ),
        (
            BankMotif::SustainedDegradation,
            "Persistent elevation of residual magnitude over many windows without recovery; DSFB interprets this as sustained structural degradation, not as a ground-truth causal diagnosis.",
        ),
        (
            BankMotif::OscillationInstability,
            "Repeated alternation across the boundary band without sustained commitment; DSFB interprets this as oscillatory structural pattern, not as a ground-truth causal diagnosis.",
        ),
        (
            BankMotif::LocalizedRouteFault,
            "Episode bounded to a specific entity/route locality with neighbouring entities admissible; DSFB interprets this as locality-confined structure, not as a ground-truth causal diagnosis.",
        ),
        (
            BankMotif::FanoutCascadeCandidate,
            "Co-occurrence pattern across multiple entities consistent with fan-out cascade structure; DSFB interprets this as multi-entity structural co-firing, not as a ground-truth causal diagnosis.",
        ),
        (
            BankMotif::ConfuserTransient,
            "Confuser-like transient that fired but did not sustain into a full motif; DSFB interprets this structurally as transient near-violation, not as a ground-truth causal diagnosis.",
        ),
    ]
};

fn motif_name(m: dsfb_gpu_debug_core::bank::BankMotif) -> &'static str {
    // Keep this lookup deterministic: avoid Debug derives in the
    // rendered output to prevent format drift between compiler versions.
    use dsfb_gpu_debug_core::bank::BankMotif;
    match m {
        BankMotif::LatencyRamp => "LatencyRamp",
        BankMotif::ErrorBurst => "ErrorBurst",
        BankMotif::SlewShockRecovery => "SlewShockRecovery",
        BankMotif::SustainedDegradation => "SustainedDegradation",
        BankMotif::OscillationInstability => "OscillationInstability",
        BankMotif::LocalizedRouteFault => "LocalizedRouteFault",
        BankMotif::FanoutCascadeCandidate => "FanoutCascadeCandidate",
        BankMotif::ConfuserTransient => "ConfuserTransient",
    }
}

fn reason_name(r: dsfb_gpu_debug_core::grammar::ReasonCode) -> &'static str {
    use dsfb_gpu_debug_core::grammar::ReasonCode;
    match r {
        ReasonCode::Admissible => "Admissible",
        ReasonCode::BoundaryApproach => "BoundaryApproach",
        ReasonCode::SustainedOutwardDrift => "SustainedOutwardDrift",
        ReasonCode::AbruptSlewViolation => "AbruptSlewViolation",
        ReasonCode::RecurrentBoundaryGrazing => "RecurrentBoundaryGrazing",
        ReasonCode::EnvelopeViolation => "EnvelopeViolation",
        ReasonCode::DriftWithRecovery => "DriftWithRecovery",
        ReasonCode::SingleCrossing => "SingleCrossing",
    }
}

fn grammar_name(g: dsfb_gpu_debug_core::grammar::GrammarState) -> &'static str {
    use dsfb_gpu_debug_core::grammar::GrammarState;
    match g {
        GrammarState::Admissible => "Admissible",
        GrammarState::Boundary => "Boundary",
        GrammarState::Violation => "Violation",
        GrammarState::Recovery => "Recovery",
    }
}

const NON_CLAIMS: &[&str] = &[
    "Does NOT claim DSFB has identified the \"real\" anomaly in the dataset.",
    "Does NOT claim DSFB outperforms any other anomaly detector.",
    "Does NOT claim DSFB has discovered causality.",
    "Does NOT claim DSFB has measured remediation effectiveness.",
    "Does NOT claim fitness-for-purpose on regulated or safety-critical use.",
    "Does NOT claim the dataset is \"correctly labeled\" or \"ground truth\"; the audit report describes deterministic structure DSFB-GPU saw, not labels.",
    "Does NOT claim the corpus or registry is exhaustive.",
    "Does NOT claim replay determinism across different driver / CUDA / hardware versions; the replay receipt records the toolchain explicitly.",
];

const STYLE: &str = r#"
body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", "Helvetica Neue", Arial, sans-serif;
       margin: 2rem; max-width: 1100px; color: #222; }
h1 { border-bottom: 2px solid #444; padding-bottom: .25rem; }
h2 { margin-top: 2rem; border-bottom: 1px solid #aaa; padding-bottom: .15rem; }
h3 { margin-top: 1.25rem; color: #444; }
p.subhead { color: #555; font-size: .95rem; margin-top: -.25rem; }
p.note { color: #555; font-size: .9rem; max-width: 75ch; }
table { border-collapse: collapse; margin: .5rem 0 1rem 0; }
table.kv th { text-align: left; padding: .25rem .75rem .25rem 0; vertical-align: top; font-weight: 600; color: #333; }
table.kv td { padding: .25rem 0; font-family: ui-monospace, SFMono-Regular, Consolas, monospace; font-size: .9rem; }
table.episodes, table.hashes { width: 100%; font-size: .85rem; }
table.episodes th, table.episodes td,
table.hashes th, table.hashes td { border-bottom: 1px solid #ddd; padding: .35rem .5rem; text-align: left; }
table.episodes th, table.hashes th { background: #f4f4f4; font-weight: 600; }
table.hashes td.hex { font-family: ui-monospace, SFMono-Regular, Consolas, monospace; font-size: .82rem; word-break: break-all; }
table.hashes td.label { font-weight: 600; white-space: nowrap; }
pre.law { background: #f8f8f8; border: 1px solid #ddd; padding: .75rem; font-size: .85rem;
          overflow-x: auto; max-width: 100%; }
ul.nonclaims li { margin: .25rem 0; color: #333; }
code { background: #eee; padding: 1px 4px; border-radius: 3px; font-size: .9em; }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> DatasetManifest {
        DatasetManifest {
            dataset_id: "aiops_kpi".to_string(),
            display_name: "AIOps Challenge 2018 KPI".to_string(),
            upstream_doi_or_url: "Su et al., IPCCC 2018; github.com/NetManAIOps/Bagel".to_string(),
            license: "Apache-2.0".to_string(),
            source_class: "TimeSeriesAnomaly".to_string(),
            vendored_path: "/home/one/dsfb/crates/dsfb-debug/data/fixtures/aiops_challenge.tsv"
                .to_string(),
            fixture_sha256_hex: "29961b8b66d941c19c065cfa974a62f098ebd63ef8c9017d8219e9f228135642"
                .to_string(),
            fixture_byte_size: 2015,
        }
    }

    fn schema() -> SchemaMap {
        SchemaMap {
            declared_num_windows: 32,
            declared_num_signals: 4,
            declared_healthy_window_end: 12,
            observed_num_windows: 38,
            observed_num_signals: 4,
            nan_cell_count: 0,
            finite_cell_count: 152,
            emitted_event_count: 152,
            lowering_config: LoweringConfig::default(),
        }
    }

    fn empty_case() -> CaseFile {
        use dsfb_gpu_debug_core::casefile::{CaseFile, EmissionMode, IntermediateHashes};
        use dsfb_gpu_debug_core::verdict::FinalVerdict;
        let z = [0u8; 32];
        CaseFile {
            version: "dsfb-gpu-debug-case-0.1",
            backend: "cuda",
            mode: EmissionMode::Throughput,
            hashes: IntermediateHashes {
                input_catalog: z,
                contract: z,
                bank: z,
                detector_registry: z,
                kernel_sequence: z,
                window_feature: z,
                residual_field: z,
                sign_field: z,
                detector_cell: z,
                consensus_grid: z,
                candidate_interval: z,
                episode: z,
            },
            episodes: Vec::new(),
            final_case_file_hash: [0u8; 32],
            final_verdict: FinalVerdict::ReplayAdmissible,
        }
    }

    fn replay() -> ReplayVerification {
        let mut tc = BTreeMap::new();
        tc.insert("rustc".to_string(), "1.84.0 stable".to_string());
        tc.insert("cuda".to_string(), "13.2".to_string());
        tc.insert("driver".to_string(), "test-stub".to_string());
        tc.insert("gpu".to_string(), "RTX 4080 SUPER".to_string());
        ReplayVerification {
            run_count: 2,
            casefile_json_sha256_run1: "0".repeat(64),
            casefile_json_sha256_run2: "0".repeat(64),
            episodes_jsonl_sha256_run1: "0".repeat(64),
            episodes_jsonl_sha256_run2: "0".repeat(64),
            final_case_file_hash_run1_hex: "0".repeat(64),
            final_case_file_hash_run2_hex: "0".repeat(64),
            episode_count_run1: 0,
            episode_count_run2: 0,
            toolchain: tc,
        }
    }

    #[test]
    fn render_is_byte_stable_across_two_calls() {
        let mani = manifest();
        let sch = schema();
        let case = empty_case();
        let rep = replay();
        let render_a = render_audit_report_html(&mani, &sch, &case, &rep);
        let render_b = render_audit_report_html(&mani, &sch, &case, &rep);
        assert_eq!(render_a, render_b);
        assert!(render_a.starts_with("<!DOCTYPE html>"));
        assert!(render_a.contains("AIOps Challenge 2018 KPI"));
    }

    #[test]
    fn render_contains_all_seven_sections() {
        let html = render_audit_report_html(&manifest(), &schema(), &empty_case(), &replay());
        for section in [
            "1. Input provenance",
            "2. Residual-projection lowering law",
            "3. Run configuration",
            "4. Admitted episodes",
            "5. Stage digest / hash chain",
            "6. Replay verification",
            "7. Limitations and non-claims",
        ] {
            assert!(html.contains(section), "missing section: {section}");
        }
    }

    #[test]
    fn render_carries_every_non_claim() {
        let html = render_audit_report_html(&manifest(), &schema(), &empty_case(), &replay());
        for nc in NON_CLAIMS {
            assert!(html.contains(&escape(nc)), "missing non-claim: {nc}");
        }
    }

    #[test]
    fn empty_episodes_render_honestly() {
        let html = render_audit_report_html(&manifest(), &schema(), &empty_case(), &replay());
        assert!(html.contains("No episodes were admitted"));
    }

    #[test]
    fn replay_admits_when_hashes_agree() {
        let r = replay();
        assert!(r.admits());
    }

    #[test]
    fn replay_rejects_when_casefile_hashes_diverge() {
        let mut r = replay();
        r.casefile_json_sha256_run2 = "1".repeat(64);
        assert!(!r.admits());
    }

    // ----- S-REAL.1.1 — renderer richness acceptance tests -----

    use dsfb_gpu_debug_core::bank::{BankMotif, Episode};
    use dsfb_gpu_debug_core::fixed::Q16;
    use dsfb_gpu_debug_core::grammar::{GrammarState, ReasonCode};

    fn mk_ep(
        entity: u32,
        start: u32,
        end: u32,
        motif: BankMotif,
        reason: ReasonCode,
        bits: u32,
    ) -> Episode {
        Episode {
            entity_id: entity,
            start_window: start,
            end_window: end,
            motif,
            reason,
            peak_state: GrammarState::Boundary,
            peak_residual_q: Q16(100),
            peak_drift_q: Q16(200),
            peak_slew_q: Q16(50),
            detector_bit_count: bits,
            admission: None,
        }
    }

    fn case_with_episodes(episodes: Vec<Episode>) -> CaseFile {
        let mut c = empty_case();
        c.episodes = episodes;
        c
    }

    fn nonempty_case() -> CaseFile {
        case_with_episodes(vec![
            mk_ep(
                0,
                5,
                12,
                BankMotif::LatencyRamp,
                ReasonCode::SustainedOutwardDrift,
                8,
            ),
            mk_ep(
                0,
                20,
                25,
                BankMotif::ErrorBurst,
                ReasonCode::AbruptSlewViolation,
                6,
            ),
            mk_ep(
                3,
                1,
                4,
                BankMotif::OscillationInstability,
                ReasonCode::RecurrentBoundaryGrazing,
                2,
            ),
            mk_ep(
                3,
                10,
                30,
                BankMotif::SustainedDegradation,
                ReasonCode::EnvelopeViolation,
                10,
            ),
            mk_ep(
                7,
                8,
                9,
                BankMotif::ConfuserTransient,
                ReasonCode::SingleCrossing,
                1,
            ),
        ])
    }

    #[test]
    fn render_carries_summary_card() {
        let html = render_audit_report_html(&manifest(), &schema(), &nonempty_case(), &replay());
        assert!(html.contains("class=\"summary-card\""));
        assert!(html.contains("Dataset summary"));
        assert!(html.contains("shape (entities × windows)"));
        assert!(html.contains("episodes admitted"));
        assert!(html.contains("byte-identical replay"));
    }

    #[test]
    fn render_summary_card_includes_source_class_and_truncated_hash() {
        let html = render_audit_report_html(&manifest(), &schema(), &nonempty_case(), &replay());
        // S-REAL.1.1.1: source_class is in the summary card.
        assert!(html.contains("source class"));
        assert!(html.contains("TimeSeriesAnomaly"));
        // S-REAL.1.1.1: truncated final_case_file_hash (16 hex chars) in summary.
        assert!(html.contains("final_case_file_hash (first 16 hex)"));
    }

    #[test]
    fn render_carries_replay_proof_card_with_artifact_hashes() {
        let html = render_audit_report_html(&manifest(), &schema(), &nonempty_case(), &replay());
        assert!(html.contains("class=\"replay-proof-card\""));
        assert!(html.contains("Replay proof"));
        assert!(html.contains("casefile.json SHA-256"));
        assert!(html.contains("episodes.jsonl SHA-256"));
        assert!(html.contains("audit_report.html SHA-256"));
        // Honest disclosure of the self-referential-hash impossibility:
        assert!(html.contains("externally pinned in replay_verification.txt"));
        assert!(html.contains("final_case_file_hash (full)"));
    }

    #[test]
    fn render_carries_motif_histogram() {
        let html = render_audit_report_html(&manifest(), &schema(), &nonempty_case(), &replay());
        assert!(html.contains("4a. Motif histogram"));
        // All 5 distinct motifs appear by wire name (sorted ascending).
        for name in [
            "ConfuserTransient",
            "ErrorBurst",
            "LatencyRamp",
            "OscillationInstability",
            "SustainedDegradation",
        ] {
            assert!(html.contains(name), "motif missing: {name}");
        }
    }

    #[test]
    fn render_carries_reason_code_histogram_with_prose() {
        let html = render_audit_report_html(&manifest(), &schema(), &nonempty_case(), &replay());
        assert!(html.contains("4b. Reason-code histogram"));
        // The standardised structural-not-causal tail appears in every prose entry.
        let count = html
            .matches("DSFB interprets this structurally, not as a ground-truth causal diagnosis.")
            .count();
        assert!(
            count >= 5,
            "expected at least 5 reason-prose tails, got {count}"
        );
    }

    #[test]
    fn render_carries_entity_summary_with_dominant_motif() {
        let html = render_audit_report_html(&manifest(), &schema(), &nonempty_case(), &replay());
        assert!(html.contains("4c. Entity summary"));
        // Three distinct entities (0, 3, 7) so 3 rows in the table.
        assert!(html.contains("first_window"));
        assert!(html.contains("dominant_motif"));
    }

    #[test]
    fn render_carries_episode_timeline() {
        let html = render_audit_report_html(&manifest(), &schema(), &nonempty_case(), &replay());
        assert!(html.contains("4d. Episode timeline"));
    }

    #[test]
    fn render_carries_top_structural_spans_with_three_subtables() {
        let html = render_audit_report_html(&manifest(), &schema(), &nonempty_case(), &replay());
        assert!(html.contains("4e. Top structural spans"));
        assert!(html.contains("4e(a). Longest spans"));
        assert!(html.contains("4e(b). Highest-repeated entities"));
        assert!(html.contains("4e(c). Most-recurrent reason codes"));
    }

    #[test]
    fn render_carries_motif_glossary_with_all_eight_motifs() {
        let html = render_audit_report_html(&manifest(), &schema(), &empty_case(), &replay());
        // Glossary renders ALL 8 motifs even when no episodes fired,
        // so the report is self-documenting for first-time readers.
        assert!(html.contains("4f. Plain-English motif glossary"));
        for name in [
            "LatencyRamp",
            "ErrorBurst",
            "SlewShockRecovery",
            "SustainedDegradation",
            "OscillationInstability",
            "LocalizedRouteFault",
            "FanoutCascadeCandidate",
            "ConfuserTransient",
        ] {
            assert!(html.contains(name), "motif glossary missing: {name}");
        }
    }

    #[test]
    fn render_is_byte_stable_after_s_real_1_1_additions() {
        // Two renders of the SAME nonempty case must produce
        // byte-identical bytes — the S-REAL.1.1 additions must not
        // introduce any HashMap or wall-clock ordering.
        let case = nonempty_case();
        let a = render_audit_report_html(&manifest(), &schema(), &case, &replay());
        let b = render_audit_report_html(&manifest(), &schema(), &case, &replay());
        assert_eq!(a, b);
    }

    /// Causal-diagnosis-language regression scanner.
    ///
    /// WHY: The S-REAL.1.1 plan-locked non-overclaim boundary forbids
    /// causal-correctness or ground-truth-anomaly claims in the new
    /// operator-facing surfaces (summary card, motif histogram,
    /// reason-code histogram, entity summary, episode timeline, top
    /// structural spans, motif glossary). The pre-existing section 7
    /// "Limitations and non-claims" is panel-locked verbatim and is
    /// allowed to NEGATE forbidden phrases (e.g. *"Does NOT claim
    /// DSFB outperforms ..."*); the scanner explicitly scopes itself
    /// to the new S-REAL.1.1 sub-sections to avoid false-positive
    /// matches on legitimate disclaimers.
    ///
    /// The standardised structural-not-causal tail (*"DSFB interprets
    /// this structurally, not as a ground-truth causal diagnosis"*)
    /// is allowed inside the scoped region because it is itself a
    /// disclaimer; the scanner subtracts those occurrences before
    /// asserting zero bare hits.
    #[test]
    fn rejects_causal_diagnosis_language() {
        const FORBIDDEN: &[&str] = &[
            "real root cause",
            "true anomaly",
            "ground-truth anomaly",
            "outperforms",
            "outperformed",
            "the real cause",
        ];

        let html = render_audit_report_html(&manifest(), &schema(), &nonempty_case(), &replay());
        // Scope: from the start of the summary card to the start of
        // section 5 (Stage digest / hash chain). This covers every
        // new S-REAL.1.1 surface; the panel-locked non-claims block
        // (section 7) is outside the scope by construction.
        let start = html
            .find("class=\"summary-card\"")
            .expect("summary card present");
        let end = html.find("5. Stage digest").expect("section 5 present");
        let scoped = &html[start..end].to_ascii_lowercase();
        for needle in FORBIDDEN {
            assert!(
                !scoped.contains(needle),
                "S-REAL.1.1 operator-facing sub-sections contain forbidden \
                 causal-diagnosis phrase: {needle}"
            );
        }

        // "causal diagnosis" is admissible ONLY inside the standardised
        // negation tail; bare occurrences (positive claims) fail.
        let bare_causal = scoped.matches("causal diagnosis").count()
            - scoped
                .matches("not as a ground-truth causal diagnosis")
                .count();
        assert_eq!(
            bare_causal, 0,
            "bare 'causal diagnosis' (outside the standardised disclaimer) \
             found in S-REAL.1.1 operator-facing sub-sections"
        );
    }
}
