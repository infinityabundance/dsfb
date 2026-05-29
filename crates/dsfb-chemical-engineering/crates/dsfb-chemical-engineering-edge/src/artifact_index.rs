//! `artifact_index` (P97) — a deterministic, self-contained index of the whole DSFB artifact graph.
//!
//! `generate-index` emits `reports/index.{html,json}` plus a sealed `index_root`, so a reviewer opens ONE file and
//! reaches the shipped artifact graph: optional paper/figure entries when present, the eight crates (the seven
//! chemical crates + the `dsfb-densor-runtime` substrate), the 20 evidence bundles, the governance courts, the
//! controlled-access policy, and the SBIR/operator docs — entering the (deliberately broad) artifact at whatever
//! layer they care about. It is a *navigation* layer, not a new authority: it only reads already-committed, sealed
//! sources and re-presents them.
//!
//! **Determinism.** The index is a pure function of the committed inputs — no timestamps, no run-specific paths,
//! every collection sorted by a stable key. `index.json` therefore has a re-runnable `index_root` (a
//! [`CanonicalHasher`] seal over the canonical inventory), so two runs on the same tree produce byte-identical
//! output. The transient paper-build numbers (page count, overfull) are NOT folded into the seal — only the
//! committed PDF's SHA-256 is, or an explicit absent-paper sentinel when no PDF ships — so the hash does not depend
//! on whether the paper was just rebuilt.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::hashing::{sha256_hex, CanonicalHasher};
use crate::release_scrub::ControlledAccessDatasetPolicy;

const PAPER_PDF_PATH: &str = "paper/dsfb_chemical_engineering.pdf";
const PAPER_NOT_BUNDLED: &str = "not bundled in this crate snapshot";

// ── Parsed-input shapes (subset of the committed files we read) ─────────────────────────────────────

/// One row of `data/EXPECTED_BUNDLE_ROOTS.toml` (a `[dataset]` table).
#[derive(Deserialize)]
struct BundleEntry {
    kind: String,
    bundle_root: String,
    evidence_root: String,
}

/// One figure of `paper/figures/figure_manifest.json`.
#[derive(Deserialize)]
struct FigureEntry {
    id: String,
    group: String,
    source: String,
    #[serde(default)]
    png_sha256: String,
}
#[derive(Deserialize)]
struct FigureManifest {
    #[serde(default)]
    n_figures: usize,
    #[serde(default)]
    figures: Vec<FigureEntry>,
}

/// One `[[dataset]]` of `data/MANIFEST.toml` (provenance).
#[derive(Deserialize)]
struct ManifestDataset {
    name: String,
    kind: String,
    #[serde(default)]
    status: String,
}
#[derive(Deserialize)]
struct ManifestDoc {
    #[serde(default)]
    dataset: Vec<ManifestDataset>,
}

// ── Index model (serialised verbatim to index.json) ─────────────────────────────────────────────────

#[derive(Serialize)]
struct PaperInfo {
    pdf_path: String,
    pdf_sha256: String,
    n_figures: usize,
}
#[derive(Serialize, Clone)]
struct CrateRow {
    name: &'static str,
    role: &'static str,
}
#[derive(Serialize)]
struct BundleRow {
    dataset: String,
    kind: String,
    bundle_root: String,
    evidence_root: String,
}
#[derive(Serialize)]
struct FigureRow {
    id: String,
    group: String,
    source: String,
    /// Derived: "tracked", or a controlled-access note for figures whose source data is research-quarantined
    /// (not redistributed). Display/clarity only — NOT part of the sealed inventory (`seal()` hashes id + png sha).
    source_status: String,
    png_sha256: String,
}
#[derive(Serialize)]
struct DatasetRow {
    name: String,
    kind: String,
    status: String,
}
#[derive(Serialize)]
struct CourtRow {
    name: &'static str,
    verdict: String,
    report_hash: String,
}
#[derive(Serialize)]
struct DocRow {
    path: String,
    title: String,
}
#[derive(Serialize)]
struct PolicyView {
    may_ship_raw: bool,
    may_ship_processed_rows: bool,
    may_ship_attack_list: bool,
    may_ship_reconstructable_windows: bool,
    may_ship_aggregate_metrics: bool,
    may_ship_scripts: bool,
}

/// The full artifact index — the structured payload of `reports/index.json`.
#[derive(Serialize)]
pub struct IndexModel {
    format: &'static str,
    format_version: u32,
    crate_version: &'static str,
    paper: PaperInfo,
    crates: Vec<CrateRow>,
    bundles: Vec<BundleRow>,
    figures: Vec<FigureRow>,
    datasets: Vec<DatasetRow>,
    courts: Vec<CourtRow>,
    controlled_access_policy: PolicyView,
    docs: Vec<DocRow>,
    /// SHA-256 (via [`CanonicalHasher`]) over the canonical inventory — re-runnable, deterministic.
    index_root: String,
}

pub const INDEX_FORMAT: &str = "dsfb_chemical_engineering_artifact_index_v1";

// ── Build ──────────────────────────────────────────────────────────────────────────────────────────

/// The crates and their roles (the workspace authority map). Static, so the index never drifts from the
/// architecture without a code change here. The seven DSFB-Chemical crates plus the domain-agnostic
/// `dsfb-densor-runtime` substrate (which carries no chemical claims — listed for completeness).
const CRATES: [CrateRow; 8] = [
    CrateRow {
        name: "dsfb-chemical-engineering-edge",
        role: "CPU execution over process residuals",
    },
    CrateRow {
        name: "dsfb-chemical-engineering-cuda",
        role: "GPU evidence factory + forensic court",
    },
    CrateRow {
        name: "dsfb-chemical-engineering-atlas",
        role: "detector / heuristic / fault-signature authority (no_std)",
    },
    CrateRow {
        name: "dsfb-chemical-engineering-corpus",
        role: "soft-sensor dataset authority catalogue (no_std)",
    },
    CrateRow {
        name: "dsfb-chemical-engineering-core",
        role: "no_std fixed-point embedded grammar",
    },
    CrateRow {
        name: "dsfb-chemical-engineering-py",
        role: "standalone Python binding surface",
    },
    CrateRow {
        name: "dsfb-chemical-engineering-wasm",
        role: "standalone browser Chemical Court simulator",
    },
    CrateRow {
        name: "dsfb-densor-runtime",
        role: "deterministic execution-substrate skeleton (no chemical/cross-domain claims)",
    },
];

/// Read + parse all committed inputs into the deterministic [`IndexModel`]. Missing inputs degrade gracefully
/// (empty section + a sentinel), never panic — the index reflects exactly what is present.
pub fn build_index(crate_dir: &Path, repo_root: &Path) -> IndexModel {
    // Paper: committed PDF sha256 (stable) + figure count.
    let pdf_path = PAPER_PDF_PATH;
    let pdf_sha256 = std::fs::read(repo_root.join(pdf_path))
        .map(|b| sha256_hex(&b))
        .unwrap_or_else(|_| PAPER_NOT_BUNDLED.to_string());

    // Figures (paper/figures/figure_manifest.json) — keep manifest order (deterministic).
    let fig_manifest: FigureManifest =
        std::fs::read_to_string(repo_root.join("paper/figures/figure_manifest.json"))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or(FigureManifest {
                n_figures: 0,
                figures: vec![],
            });
    let figures: Vec<FigureRow> = fig_manifest
        .figures
        .iter()
        .map(|f| {
            // Figures built from controlled-access (research-quarantined) data name that source for provenance, but
            // the rows are never shipped — annotate so a reviewer does not read a missing source file as a broken graph.
            let source_status = if f.source.contains("instrumented") || f.source.contains(".witness.csv") {
                "research-quarantined (controlled-access); aggregate figure only; rows not redistributed".to_string()
            } else {
                "tracked".to_string()
            };
            FigureRow { id: f.id.clone(), group: f.group.clone(), source: f.source.clone(), source_status, png_sha256: f.png_sha256.clone() }
        })
        .collect();

    // Evidence bundles (data/EXPECTED_BUNDLE_ROOTS.toml) — a table of `[dataset]` tables; BTreeMap → sorted.
    let bundles: Vec<BundleRow> =
        std::fs::read_to_string(crate_dir.join("data/EXPECTED_BUNDLE_ROOTS.toml"))
            .ok()
            .and_then(|s| toml::from_str::<BTreeMap<String, BundleEntry>>(&s).ok())
            .map(|m| {
                m.into_iter()
                    .map(|(dataset, e)| BundleRow {
                        dataset,
                        kind: e.kind,
                        bundle_root: e.bundle_root,
                        evidence_root: e.evidence_root,
                    })
                    .collect()
            })
            .unwrap_or_default();

    // Dataset provenance (data/MANIFEST.toml) — manifest order.
    let datasets: Vec<DatasetRow> = std::fs::read_to_string(crate_dir.join("data/MANIFEST.toml"))
        .ok()
        .and_then(|s| toml::from_str::<ManifestDoc>(&s).ok())
        .map(|m| {
            m.dataset
                .into_iter()
                .map(|d| DatasetRow {
                    name: d.name,
                    kind: d.kind,
                    status: d.status,
                })
                .collect()
        })
        .unwrap_or_default();

    // Governance courts (run in-process — deterministic verdicts + report hashes).
    let comp = crate::completeness::run_completeness_court(crate_dir);
    let scrub = crate::release_scrub::run_release_scrub(repo_root);
    let courts = vec![
        CourtRow {
            name: "ArtifactCompletenessCourtV1",
            verdict: format!(
                "{} ({}/{})",
                if comp.all_passed() {
                    "COMPLETE"
                } else {
                    "INCOMPLETE"
                },
                comp.n_pass,
                comp.n_fail
            ),
            report_hash: comp.report_hash.clone(),
        },
        CourtRow {
            name: "PublicReleaseScrubCourtV1",
            verdict: format!(
                "{} ({}/{})",
                if scrub.all_passed() {
                    "RELEASE-CLEAN"
                } else {
                    "RELEASE-DIRTY"
                },
                scrub.n_pass,
                scrub.n_fail
            ),
            report_hash: scrub.report_hash.clone(),
        },
    ];

    // Controlled-access policy — the single source of truth, surfaced as a view.
    let p = ControlledAccessDatasetPolicy::STANDING;
    let controlled_access_policy = PolicyView {
        may_ship_raw: p.may_ship_raw,
        may_ship_processed_rows: p.may_ship_processed_rows,
        may_ship_attack_list: p.may_ship_attack_list,
        may_ship_reconstructable_windows: p.may_ship_reconstructable_windows,
        may_ship_aggregate_metrics: p.may_ship_aggregate_metrics,
        may_ship_scripts: p.may_ship_scripts,
    };

    // Docs (docs/*.md) — sorted by path; title = first `# ` heading, else the file stem.
    let mut docs: Vec<DocRow> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(repo_root.join("docs")) {
        let mut paths: Vec<std::path::PathBuf> = rd
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map(|x| x == "md").unwrap_or(false))
            .collect();
        paths.sort();
        for path in paths {
            let rel = format!(
                "docs/{}",
                path.file_name().and_then(|n| n.to_str()).unwrap_or("")
            );
            let title = std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| {
                    s.lines()
                        .find(|l| l.starts_with("# "))
                        .map(|l| l.trim_start_matches("# ").trim().to_string())
                })
                .unwrap_or_else(|| rel.clone());
            docs.push(DocRow { path: rel, title });
        }
    }

    let mut model = IndexModel {
        format: INDEX_FORMAT,
        format_version: 1,
        crate_version: crate::VERSION,
        paper: PaperInfo {
            pdf_path: pdf_path.to_string(),
            pdf_sha256,
            n_figures: fig_manifest.n_figures,
        },
        crates: CRATES.to_vec(),
        bundles,
        figures,
        datasets,
        courts,
        controlled_access_policy,
        docs,
        index_root: String::new(),
    };
    model.index_root = seal(&model);
    model
}

/// Seal the canonical inventory (NOT the transient paper build numbers) — the deterministic `index_root`. Field
/// order is fixed and every collection is already in a stable order, so this is re-runnable byte-for-byte.
fn seal(m: &IndexModel) -> String {
    let mut h = CanonicalHasher::new();
    h.field("schema", INDEX_FORMAT.as_bytes());
    h.u64("format_version", m.format_version as u64);
    h.field("paper_pdf_sha256", m.paper.pdf_sha256.as_bytes());
    h.u64("n_figures", m.paper.n_figures as u64);
    for c in &m.crates {
        h.field("crate", c.name.as_bytes());
    }
    for b in &m.bundles {
        h.field("bundle_dataset", b.dataset.as_bytes());
        h.field("bundle_root", b.bundle_root.as_bytes());
        h.field("evidence_root", b.evidence_root.as_bytes());
    }
    for f in &m.figures {
        h.field("figure_id", f.id.as_bytes());
        h.field("figure_sha256", f.png_sha256.as_bytes());
    }
    for d in &m.datasets {
        h.field("dataset", d.name.as_bytes());
        h.field("dataset_kind", d.kind.as_bytes());
    }
    for c in &m.courts {
        h.field("court", c.name.as_bytes());
        h.field("court_verdict", c.verdict.as_bytes());
        h.field("court_hash", c.report_hash.as_bytes());
    }
    let p = &m.controlled_access_policy;
    for (k, v) in [
        ("may_ship_raw", p.may_ship_raw),
        ("may_ship_processed_rows", p.may_ship_processed_rows),
        ("may_ship_attack_list", p.may_ship_attack_list),
        (
            "may_ship_reconstructable_windows",
            p.may_ship_reconstructable_windows,
        ),
        ("may_ship_aggregate_metrics", p.may_ship_aggregate_metrics),
        ("may_ship_scripts", p.may_ship_scripts),
    ] {
        h.u64(k, v as u64);
    }
    for d in &m.docs {
        h.field("doc", d.path.as_bytes());
    }
    h.finalize_hex()
}

// ── Render ─────────────────────────────────────────────────────────────────────────────────────────

/// Minimal HTML escape for text that lands inside element bodies.
fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
/// First 12 hex chars of a digest, for compact display (full value is in `index.json`).
fn short(h: &str) -> &str {
    if h.len() >= 12 {
        &h[..12]
    } else {
        h
    }
}

/// Render the self-contained `index.html` (no external assets). Links are relative to `reports/`.
pub fn render_html(m: &IndexModel) -> String {
    let mut s = String::new();
    s.push_str("<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\">\
<title>DSFB-Chemical-Engineering — artifact index</title>\
<style>body{font-family:system-ui,sans-serif;max-width:1000px;margin:2rem auto;color:#1d3557;line-height:1.5;padding:0 1rem}\
h1{margin-bottom:.2rem}h2{margin-top:2rem;border-bottom:2px solid #e0e6ef;padding-bottom:.2rem}\
code{background:#f0f3f8;padding:.1em .3em;border-radius:3px;font-size:.9em}\
table{border-collapse:collapse;width:100%;font-size:.88rem;margin:.5rem 0}\
th,td{border:1px solid #dde3ec;padding:.3rem .5rem;text-align:left;vertical-align:top}\
th{background:#f0f3f8}.muted{color:#5a6b80}.seal{font-family:monospace;font-size:.8rem;color:#5a6b80}\
.ok{color:#2a7d2a;font-weight:600}.note{color:#9a6b00}\
h3{margin:1rem 0 .2rem;font-size:1rem;color:#16324f}\
.routes{display:grid;grid-template-columns:repeat(auto-fit,minmax(280px,1fr));gap:.5rem 1.5rem}\
.route ul{margin:.2rem 0 .6rem;padding-left:1.1rem}.route li{margin:.1rem 0;font-size:.9rem}</style></head><body>");
    s.push_str(&format!(
        "<h1>DSFB-Chemical-Engineering — artifact index</h1>\
<p class=\"muted\">A deterministic, one-click map of the whole artifact graph. Crate version <code>{}</code>. \
This index is a navigation layer over already-sealed evidence — not a new authority.</p>\
<p class=\"seal\">index_root: {}</p>",
        esc(m.crate_version), esc(&m.index_root)
    ));

    // 0. Reviewer routes — role-targeted entry points so a reviewer of any background reaches the right
    // 5–8 committed artifacts in one click. Static HTML built from a fixed table; it is NOT part of the
    // sealed inventory, so `index_root` is unchanged by it (the seal covers the JSON model, not this prose).
    // Every href was confirmed to resolve on disk; paths are relative to `reports/`.
    type Route = (&'static str, &'static [(&'static str, &'static str)]);
    const ROUTES: &[Route] = &[
        (
            "Chemical / process engineer",
            &[
                (
                    "SBIR one-page evaluation",
                    "../docs/sbir_phase_i_one_page_eval.md",
                ),
                (
                    "Balance-witness applicability criterion",
                    "../docs/balance_witness_criterion.md",
                ),
                (
                    "Practitioner dossier (ISA / NAMUR / HAZOP)",
                    "../docs/chemical_engineering_practitioner_dossier.md",
                ),
                (
                    "TEP IDV(1) head-to-head",
                    "../docs/head_to_head_tep_idv1.md",
                ),
                (
                    "When regime-conditioning helps",
                    "../docs/regime_conditioning_applicability.md",
                ),
                (
                    "Detector / heuristic / fault-signature atlas",
                    "../crates/dsfb-chemical-engineering-atlas/README.md",
                ),
                ("Verification report", "../reports/verification_report.md"),
            ],
        ),
        (
            "Rust engineer",
            &[
                (
                    "edge crate — execution",
                    "../crates/dsfb-chemical-engineering-edge/README.md",
                ),
                (
                    "core crate — no_std embedded",
                    "../crates/dsfb-chemical-engineering-core/README.md",
                ),
                (
                    "dsfb-densor-runtime — execution substrate",
                    "../crates/dsfb-densor-runtime/README.md",
                ),
                ("Fusion theorems", "../docs/fusion_theorems.md"),
                (
                    "Constrained-narration contract",
                    "../docs/constrained_narration_extension.md",
                ),
                ("Audit suite dashboard", "../audit/index.html"),
            ],
        ),
        (
            "CUDA / GPU engineer",
            &[
                (
                    "cuda crate — evidence factory + forensic court",
                    "../crates/dsfb-chemical-engineering-cuda/README.md",
                ),
                (
                    "CUDA evidence-kernel V2 design (digest-equivalence law)",
                    "../docs/cuda_evidence_kernel_v2_design.md",
                ),
                (
                    "Realistic-lane + end-to-end timing capture",
                    "../docs/gpu_realistic_lane_timing.md",
                ),
                (
                    "Noise-floor preservation",
                    "../docs/noise_floor_preservation.md",
                ),
            ],
        ),
        (
            "Edge / embedded engineer",
            &[
                (
                    "Embedded core profile (no_std / no-heap / fixed-point)",
                    "../docs/edge_core_profile.md",
                ),
                (
                    "Memory budget + QEMU Cortex-M smoke",
                    "../docs/embedded_memory_budget.md",
                ),
                (
                    "core crate",
                    "../crates/dsfb-chemical-engineering-core/README.md",
                ),
            ],
        ),
        (
            "SBIR operator / evaluator",
            &[
                (
                    "One-page evaluation",
                    "../docs/sbir_phase_i_one_page_eval.md",
                ),
                (
                    "30 / 60 / 90 workplan",
                    "../docs/phase_i_workplan_30_60_90.md",
                ),
                ("Risk register", "../docs/sbir_risk_register.md"),
                (
                    "Operator data request",
                    "../docs/sbir_operator_data_request.md",
                ),
                ("Phase-I eval card", "../docs/sbir_phase_i_eval_card.md"),
                (
                    "Forensic incident walkthrough (Court Record example)",
                    "../docs/forensic_incident_walkthrough.md",
                ),
            ],
        ),
        (
            "Release auditor",
            &[
                ("Release checklist", "../docs/release_checklist.md"),
                (
                    "Public-archive proof method",
                    "../docs/public_archive_proof.md",
                ),
                ("Verification report", "verification_report.md"),
                (
                    "Narration-context sample (citable-anchor vocabulary + contract)",
                    "narration_context_sample.md",
                ),
                (
                    "Constrained-narration contract",
                    "../docs/constrained_narration_extension.md",
                ),
                ("Governance courts (audit dashboard)", "../audit/index.html"),
            ],
        ),
    ];
    s.push_str("<h2>0. Reviewer routes</h2>\
<p class=\"muted\">Start here by role — each route lists the 5–8 committed artifacts most worth reading first.</p>\
<div class=\"routes\">");
    for (title, items) in ROUTES {
        s.push_str(&format!("<div class=\"route\"><h3>{}</h3><ul>", esc(title)));
        for (label, href) in *items {
            s.push_str(&format!(
                "<li><a href=\"{}\">{}</a></li>",
                esc(href),
                esc(label)
            ));
        }
        s.push_str("</ul></div>");
    }
    s.push_str("</div>\
<p class=\"muted\" style=\"font-size:.85rem\">Copyable reproduce commands (run from the repo root): \
<code>dsfb-chem-edge verify-replay</code> · <code>dsfb-chem-edge completeness-court</code> · \
<code>dsfb-chem-edge verify-index</code> · <code>dsfb-chem-edge release-scrub</code> · \
<code>dsfb-chem-edge confidential-demo</code> · <code>bash scripts/build_public_archive.sh</code> · \
<code>python3 crates/dsfb-chemical-engineering-edge/scripts/verify_reproducibility.py --bundles</code>.</p>");

    // 1. Paper
    s.push_str("<h2>1. Paper</h2><table><tr><th>Artifact</th><th>Value</th></tr>");
    if m.paper.pdf_sha256 == PAPER_NOT_BUNDLED {
        s.push_str(&format!(
            "<tr><td>PDF</td><td>{}</td></tr>",
            esc(PAPER_NOT_BUNDLED)
        ));
    } else {
        s.push_str(&format!(
            "<tr><td>PDF</td><td><a href=\"../{0}\">{0}</a></td></tr>",
            esc(&m.paper.pdf_path)
        ));
        s.push_str(&format!(
            "<tr><td>PDF SHA-256</td><td class=\"seal\">{}</td></tr>",
            esc(&m.paper.pdf_sha256)
        ));
    }
    s.push_str(&format!(
        "<tr><td>Figures</td><td>{} (manifest below)</td></tr>",
        m.paper.n_figures
    ));
    s.push_str("<tr><td>Verification report</td><td><a href=\"verification_report.md\">reports/verification_report.md</a></td></tr></table>");

    // 2. Crates
    s.push_str("<h2>2. Crates</h2><table><tr><th>Crate</th><th>Role</th></tr>");
    for c in &m.crates {
        s.push_str(&format!(
            "<tr><td><code>{}</code></td><td>{}</td></tr>",
            esc(c.name),
            esc(c.role)
        ));
    }
    s.push_str("</table>");

    // 3. Evidence bundles
    s.push_str(&format!("<h2>3. Evidence bundles ({} datasets)</h2>\
<p class=\"muted\">Each Chemical Court Record's <code>bundle_root</code> + byte-exact CPU/GPU <code>evidence_root</code> \
(from <code>EXPECTED_BUNDLE_ROOTS.toml</code>). Short digests shown; full values in <code>index.json</code>.</p>\
<table><tr><th>Dataset</th><th>Kind</th><th>bundle_root</th><th>evidence_root</th></tr>", m.bundles.len()));
    for b in &m.bundles {
        s.push_str(&format!(
            "<tr><td>{}</td><td class=\"muted\">{}</td><td class=\"seal\">{}…</td><td class=\"seal\">{}…</td></tr>",
            esc(&b.dataset), esc(&b.kind), short(&b.bundle_root), short(&b.evidence_root)
        ));
    }
    s.push_str("</table>");

    // 4. Figures
    s.push_str(&format!("<h2>4. Figures ({})</h2><table><tr><th>ID</th><th>Group</th><th>Source</th><th>Status</th><th>PNG SHA-256</th></tr>", m.figures.len()));
    for f in &m.figures {
        s.push_str(&format!(
            "<tr><td><code>{}</code></td><td>{}</td><td class=\"muted\">{}</td><td class=\"muted\">{}</td><td class=\"seal\">{}…</td></tr>",
            esc(&f.id), esc(&f.group), esc(&f.source), esc(&f.source_status), short(&f.png_sha256)
        ));
    }
    s.push_str("</table>");

    // 5. Courts
    s.push_str("<h2>5. Governance courts</h2><table><tr><th>Court</th><th>Verdict</th><th>report_hash</th></tr>");
    for c in &m.courts {
        s.push_str(&format!(
            "<tr><td><code>{}</code></td><td class=\"ok\">{}</td><td class=\"seal\">{}…</td></tr>",
            esc(c.name),
            esc(&c.verdict),
            short(&c.report_hash)
        ));
    }
    s.push_str("</table><p class=\"muted\">Plus the formal layer (Kani / Lean / Coq), the digest-equivalence harness \
(GPU↔CPU), and the claim-strength / evidence-kind taxonomies — see the paper + <code>audit/index.html</code>.</p>");

    // 6. Controlled-access policy
    let p = &m.controlled_access_policy;
    s.push_str("<h2>6. Controlled-access dataset policy</h2>\
<p class=\"muted\">For iTrust-gated / similarly controlled testbeds (SWaT / WADI / BATADAL): scripts + aggregate \
metrics ship; rows do not. Real bytes stay in the untracked <code>research/</code> quarantine.</p><table>\
<tr><th>May ship</th><th>Allowed?</th></tr>");
    for (label, v) in [
        ("raw bytes", p.may_ship_raw),
        ("processed / instrumented rows", p.may_ship_processed_rows),
        ("attack list", p.may_ship_attack_list),
        (
            "reconstructable windows",
            p.may_ship_reconstructable_windows,
        ),
        ("aggregate metrics", p.may_ship_aggregate_metrics),
        ("reproducible scripts", p.may_ship_scripts),
    ] {
        s.push_str(&format!(
            "<tr><td>{}</td><td>{}</td></tr>",
            label,
            if v {
                "<span class=\"ok\">yes</span>"
            } else {
                "<span class=\"note\">no</span>"
            }
        ));
    }
    s.push_str(&format!(
        "</table><p class=\"muted\">{} datasets total; see the provenance manifest \
(<code>data/MANIFEST.toml</code>) for per-dataset license/access tiers.</p>",
        m.datasets.len()
    ));

    // 7. SBIR / operator docs
    s.push_str("<h2>7. SBIR / operator &amp; reference docs</h2><table><tr><th>Doc</th><th>Title</th></tr>");
    for d in &m.docs {
        s.push_str(&format!(
            "<tr><td><a href=\"../{0}\">{0}</a></td><td>{1}</td></tr>",
            esc(&d.path),
            esc(&d.title)
        ));
    }
    s.push_str("</table>");

    // 8. CUDA evidence-format versions — how a GPU optimisation is admitted without ever silently mutating
    // forensic identity. Static (NOT sealed), mirroring crates/.../cuda/README.md; the "court status" is a review
    // gate, not part of the sealed evidence_root.
    s.push_str("<h2>8. CUDA evidence-format versions</h2>\
<p class=\"muted\">A performance change must never silently mutate forensic identity: each candidate kernel is \
classified against the V1 sealed reference (<code>CudaOptimizationStatus</code> — see the \
<a href=\"../crates/dsfb-chemical-engineering-cuda/README.md\">cuda crate README</a> and the \
<a href=\"../docs/cuda_evidence_kernel_v2_design.md\">V2 design doc</a>). The court status is a review gate, \
<b>not</b> part of the sealed <code>evidence_root</code>.</p>\
<table><tr><th>Kernel variant</th><th><code>evidence_root</code></th><th>Per-lane evidence</th>\
<th>Digest-identical?</th><th>Court status</th></tr>\
<tr><td><b>V1</b> — sealed reference</td><td>canonical Merkle <code>evidence_root</code></td><td>baseline</td>\
<td class=\"muted\">— (the reference)</td><td>everything is measured against it</td></tr>\
<tr><td><b>V2-A</b> — throughput optimisation</td><td>identical to V1</td><td>byte-identical</td>\
<td class=\"ok\">yes</td><td class=\"ok\">digest-identical-optimization — admitted</td></tr>\
<tr><td><b>V2-B</b> — segmented re-seal</td><td><code>evidence_root_v2</code> (differs by design)</td>\
<td>byte-identical</td><td class=\"note\">no (root construction changed)</td>\
<td class=\"ok\">new-evidence-format-version — admitted as a declared format</td></tr>\
<tr><td>any kernel that perturbs lane data</td><td>differs</td><td class=\"note\">diverged</td>\
<td class=\"note\">no</td><td class=\"note\">rejected-performance-regression — must not ship</td></tr>\
</table>");

    // 9. Evidence taxonomy — the witness-strength ladder + the 12 evidence kinds, so the chemical evidence
    // hierarchy is visible in the index front door (it is also rendered into every operator report + court record).
    // The kind rows iterate EvidenceKind::ALL, so they can never drift from the enum.
    s.push_str("<h2>9. Evidence taxonomy — witness strength &amp; evidence kind</h2>\
<p class=\"muted\">Every admitted episode is tagged on both axes, so a reviewer sees at a glance whether a label \
rests on first-principles physics, detector structure, or an advisory heuristic.</p>\
<h3>Witness-strength ladder (weakest &rarr; strongest)</h3>\
<p>PrecedentSimilarityOnly &lt; HeuristicPatternOnly &lt; DetectorFamilyQuorum &lt; ControlActionConsistent &lt; \
<b>TopologyResidenceAligned</b> &lt; <b>BalanceClosure</b> — the last two (bold) are first-principles physics \
(balance closure / residence-time topology); the rest are statistical or structural agreement.</p>\
<h3>Evidence kinds (12) and the claim tier each can support</h3>\
<table><tr><th>Evidence kind</th><th>Claim tier</th></tr>");
    for k in crate::evidence_kind::EvidenceKind::ALL {
        s.push_str(&format!(
            "<tr><td><code>{}</code></td><td>{}</td></tr>",
            esc(k.tag()),
            esc(k.claim_strength().tag())
        ));
    }
    s.push_str("</table>");

    s.push_str("<p class=\"muted\" style=\"margin-top:2rem;border-top:1px solid #e0e6ef;padding-top:.5rem\">\
Generated deterministically by <code>dsfb-chem-edge generate-index</code>; <code>index.json</code> carries the same \
data + the sealed <code>index_root</code>. Self-contained — no external assets.</p></body></html>");
    s
}

/// CLI entry: build the index, seal it, and write `reports/index.{json,html}`. Returns 0 on success.
pub fn run_generate_index(crate_dir: &Path) -> i32 {
    let Some(repo_root) = crate_dir.parent().and_then(|p| p.parent()) else {
        eprintln!(
            "generate-index: could not resolve repo root from {}",
            crate_dir.display()
        );
        return 1;
    };
    let model = build_index(crate_dir, repo_root);
    let reports = repo_root.join("reports");
    if let Err(e) = std::fs::create_dir_all(&reports) {
        eprintln!("generate-index: cannot create reports/: {e}");
        return 1;
    }
    let json = match serde_json::to_string_pretty(&model) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("generate-index: JSON serialise failed: {e}");
            return 1;
        }
    };
    let html = render_html(&model);
    if let Err(e) = std::fs::write(reports.join("index.json"), json.as_bytes())
        .and_then(|_| std::fs::write(reports.join("index.html"), html.as_bytes()))
    {
        eprintln!("generate-index: write failed: {e}");
        return 1;
    }
    println!("generate-index: wrote reports/index.html + reports/index.json");
    println!("  index_root: {}", model.index_root);
    println!(
        "  {} crates · {} bundles · {} figures · {} datasets · {} courts · {} docs",
        model.crates.len(),
        model.bundles.len(),
        model.figures.len(),
        model.datasets.len(),
        model.courts.len(),
        model.docs.len()
    );
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf()
    }

    #[test]
    fn index_is_deterministic_and_populated() {
        let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let r = repo();
        let a = build_index(&crate_dir, &r);
        let b = build_index(&crate_dir, &r);
        // Deterministic: the sealed root is re-runnable byte-for-byte.
        assert_eq!(
            a.index_root, b.index_root,
            "index_root must be deterministic across runs"
        );
        assert_eq!(a.index_root.len(), 64);
        // Populated from the committed inputs.
        assert_eq!(a.crates.len(), 8);
        assert!(
            a.bundles.len() >= 20,
            "expected >=20 evidence bundles, got {}",
            a.bundles.len()
        );
        assert!(
            !a.docs.is_empty(),
            "docs section must list the docs/*.md files"
        );
        assert_eq!(a.courts.len(), 2);
        // The HTML is self-contained (no external asset references).
        let html = render_html(&a);
        assert!(html.contains("artifact index") && html.contains("index_root:"));
        assert!(
            !html.contains("http://") && !html.contains("src=\"http"),
            "must be self-contained"
        );
        // CUDA evidence-format table + evidence taxonomy legend render (static, unsealed sections).
        assert!(
            html.contains("CUDA evidence-format versions") && html.contains("evidence_root_v2")
        );
        assert!(html.contains("Witness-strength ladder") && html.contains("BalanceClosure"));
        assert!(
            html.contains("physical_balance") && html.contains("narrative_summary"),
            "evidence-kind legend must list the enum tags"
        );
    }

    #[test]
    fn reseal_detects_tampering() {
        let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let mut m = build_index(&crate_dir, &repo());
        let original = m.index_root.clone();
        m.crates.truncate(1); // drop crates from the inventory
        assert_ne!(
            seal(&m),
            original,
            "re-sealing a tampered inventory must differ"
        );
    }
}
