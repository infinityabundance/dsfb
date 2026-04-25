#!/usr/bin/env python3
"""Build a self-contained interactive reviewer dashboard at
`paper/dashboard.html`.

The dashboard embeds, per dataset:
- the per-timestep residual stream (from `data/processed/<slug>.csv`,
  preferring `<slug>_published.csv` when present)
- the grammar-state timeline computed by the parametric Python FSM
  (validated against the canonical Rust binary on canonical
  parameters) coloured by Admissible / Boundary / Violation
- the bootstrap CI summary (mean and 95 % interval) from
  `audit/bootstrap/<slug>_ci.json`

The output is a single HTML file with no backend, using plotly.js
loaded from cdnjs. A reviewer opens it in any browser and scrubs
through the residual streams.

Generated assets are gitignored under `paper/figures/dashboard_data/`
because they are regenerated from the canonical CSVs and audit
bundles each time this script runs.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

CRATE_ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(CRATE_ROOT / "scripts"))

from dsfb_fsm_parametric import FsmParams, load_residual_stream, run_fsm  # noqa: E402

PROCESSED_ROOT = CRATE_ROOT / "data" / "processed"
BOOTSTRAP_ROOT = CRATE_ROOT / "audit" / "bootstrap"
OUT_HTML = CRATE_ROOT / "paper" / "dashboard.html"

ALL_SLUGS = [
    "cwru", "ims", "kuka_lwr", "femto_st", "panda_gaz", "dlr_justin",
    "ur10_kufieta", "cheetah3", "icub_pushrecovery", "droid", "openx",
    "anymal_parkour", "unitree_g1", "aloha_static", "icub3_sorrentino",
    "mobile_aloha", "so100", "aloha_static_tape", "aloha_static_screw_driver",
    "aloha_static_pingpong_test",
]

MAX_POINTS = 4096  # downsample longer streams for fast in-browser rendering


def downsample(values, max_points):
    n = len(values)
    if n <= max_points:
        return list(range(n)), list(values)
    step = max(1, n // max_points)
    return list(range(0, n, step)), [values[i] for i in range(0, n, step)]


def build_one_dataset(slug: str) -> dict | None:
    pub = PROCESSED_ROOT / f"{slug}_published.csv"
    base = PROCESSED_ROOT / f"{slug}.csv"
    target = pub if pub.is_file() else base
    if not target.is_file():
        return None
    source = "published-theta" if pub.is_file() else "early-window-nominal"
    stream = load_residual_stream(str(target))
    if len(stream) < 2:
        return None
    # Run the canonical FSM
    params = FsmParams()

    # We need the per-sample grammar (not just the aggregate). Use
    # figures_real.run_dsfb which returns episodes.
    sys.path.insert(0, str(CRATE_ROOT / "scripts"))
    from figures_real import run_dsfb  # noqa: E402
    eps, rho = run_dsfb(stream)

    indices, norms = downsample([e.norm for e in eps], MAX_POINTS)
    _, grammar = downsample([e.grammar for e in eps], MAX_POINTS)
    _, drift = downsample([e.drift for e in eps], MAX_POINTS)

    # Bootstrap summary
    ci_path = BOOTSTRAP_ROOT / f"{slug}_ci.json"
    ci = None
    if ci_path.is_file():
        with ci_path.open() as fh:
            ci = json.load(fh)

    return {
        "slug": slug,
        "residual_source": source,
        "n_total": len(eps),
        "rho": rho,
        "indices": indices,
        "norms": norms,
        "grammar": grammar,
        "drift": drift,
        "ci": ci,
    }


def render_html(payloads: dict[str, dict]) -> str:
    payload_json = json.dumps(payloads, indent=None, separators=(",", ":"))
    return r"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>dsfb-robotics — interactive reviewer dashboard</title>
<style>
  body { font-family: 'Helvetica Neue', sans-serif; margin: 0; padding: 0;
         background: #fafafa; color: #222; }
  header { background: #1e2a3a; color: white; padding: 0.8em 1.5em;
           display: flex; justify-content: space-between; align-items: center; }
  header h1 { font-size: 1.3em; margin: 0; font-weight: 500; }
  header .blurb { font-size: 0.9em; color: #c7d3e2; }
  .container { display: flex; gap: 1em; padding: 1em; }
  .sidebar { width: 240px; min-width: 240px; background: white; border-radius: 4px;
             padding: 0.6em; box-shadow: 0 1px 3px rgba(0,0,0,0.06);
             max-height: calc(100vh - 80px); overflow-y: auto; }
  .sidebar h2 { font-size: 0.95em; margin: 0 0 0.5em 0; }
  .slug-button { display: block; width: 100%; text-align: left;
                 padding: 0.45em 0.6em; margin: 2px 0; border: none;
                 background: #f3f5f8; border-radius: 3px; cursor: pointer;
                 font-family: 'Menlo', monospace; font-size: 0.85em; color: #333; }
  .slug-button:hover { background: #e3e9f0; }
  .slug-button.active { background: #1e2a3a; color: white; }
  .source-tag { font-size: 0.7em; color: #888; margin-left: 0.4em; }
  .source-tag.literal { color: #057642; }
  main { flex: 1; background: white; border-radius: 4px;
         padding: 1em; box-shadow: 0 1px 3px rgba(0,0,0,0.06); }
  .summary { font-size: 0.9em; line-height: 1.5; margin: 0 0 1em 0;
             padding: 0.6em 0.8em; background: #f3f5f8; border-radius: 3px; }
  .summary code { background: #e3e9f0; padding: 1px 4px; border-radius: 2px;
                  font-size: 0.85em; }
  #plot-grammar, #plot-residual { width: 100%; height: 320px; }
  footer { text-align: center; font-size: 0.78em; color: #777; padding: 1em;
           margin-top: 1em; }
  footer code { background: #e3e9f0; padding: 1px 4px; border-radius: 2px; }
</style>
</head>
<body>
<header>
  <h1>dsfb-robotics &middot; interactive reviewer dashboard</h1>
  <span class="blurb">grammar emergence over real residual streams &middot; no backend &middot; static HTML</span>
</header>
<div class="container">
  <aside class="sidebar">
    <h2>Datasets (20)</h2>
    <div id="slug-list"></div>
  </aside>
  <main>
    <div class="summary" id="summary">Select a dataset on the left to load the residual + grammar timeline.</div>
    <div id="plot-grammar"></div>
    <div id="plot-residual"></div>
  </main>
</div>
<footer>
  Reproduce: <code>python3 scripts/build_dashboard.py</code> &middot;
  validated against <code>cargo test --features std,paper_lock</code>
  &middot; bit-exact across architectures via <code>.github/workflows/determinism.yml</code>.
</footer>
<script src="https://cdn.plot.ly/plotly-2.35.0.min.js"></script>
<script>
const PAYLOADS = """ + payload_json + r""";
const COLOURS = { Admissible: '#3a8f3a', Boundary: '#d6a700', Violation: '#c43a3a' };

function render(slug) {
  const d = PAYLOADS[slug];
  if (!d) return;
  const summary = document.getElementById('summary');
  const ciAdm = d.ci ? d.ci.ci.admissible : null;
  const ciBnd = d.ci ? d.ci.ci.boundary : null;
  const ciVio = d.ci ? d.ci.ci.violation : null;
  const ciCmp = d.ci ? d.ci.ci.compression_ratio : null;
  const fmt = (v, p=0) => v.toFixed(p);
  const fmtR = (v) => v.toFixed(3);
  summary.innerHTML =
    `<strong>${slug}</strong> &middot; ` +
    `residual = <code>${d.residual_source}</code> &middot; ` +
    `total samples = <code>${d.n_total.toLocaleString()}</code> &middot; ` +
    `envelope &rho; = <code>${fmtR(d.rho)}</code><br>` +
    (d.ci
      ? `bootstrap mean &middot; ` +
        `Admissible <code>${fmt(ciAdm.mean)}</code> [${fmt(ciAdm.ci_lo_2_5)},${fmt(ciAdm.ci_hi_97_5)}], ` +
        `Boundary <code>${fmt(ciBnd.mean)}</code> [${fmt(ciBnd.ci_lo_2_5)},${fmt(ciBnd.ci_hi_97_5)}], ` +
        `Violation <code>${fmt(ciVio.mean)}</code> [${fmt(ciVio.ci_lo_2_5)},${fmt(ciVio.ci_hi_97_5)}], ` +
        `compression <code>${fmtR(ciCmp.mean)}</code> [${fmtR(ciCmp.ci_lo_2_5)},${fmtR(ciCmp.ci_hi_97_5)}]`
      : '<em>(bootstrap CI not yet computed for this dataset)</em>');
  // Grammar timeline as filled area, colour-coded
  const colour = d.grammar.map(g => COLOURS[g] || '#888');
  Plotly.newPlot('plot-grammar', [{
    x: d.indices,
    y: d.norms,
    type: 'bar',
    marker: { color: colour, line: { width: 0 } },
    hovertemplate: 'k=%{x}<br>‖r‖=%{y:.4f}<br>grammar=%{customdata}<extra></extra>',
    customdata: d.grammar,
  }], {
    title: 'Grammar timeline — bar colour = committed state',
    margin: { l: 50, r: 20, t: 40, b: 30 },
    xaxis: { title: 'sample index k' },
    yaxis: { title: '‖r(k)‖' },
    bargap: 0,
  }, { responsive: true });
  // Residual + envelope overlay
  Plotly.newPlot('plot-residual', [
    {
      x: d.indices, y: d.norms, mode: 'lines', type: 'scatter', name: '‖r(k)‖',
      line: { color: '#1e2a3a', width: 1 },
      hovertemplate: 'k=%{x}<br>‖r‖=%{y:.4f}<extra></extra>',
    },
    {
      x: [d.indices[0], d.indices[d.indices.length-1]],
      y: [d.rho, d.rho], mode: 'lines', name: 'ρ envelope',
      line: { color: '#c43a3a', width: 1, dash: 'dash' },
    },
    {
      x: [d.indices[0], d.indices[d.indices.length-1]],
      y: [0.5*d.rho, 0.5*d.rho], mode: 'lines', name: 'βρ boundary',
      line: { color: '#d6a700', width: 1, dash: 'dot' },
    },
  ], {
    title: 'Residual stream with calibration envelope',
    margin: { l: 50, r: 20, t: 40, b: 30 },
    xaxis: { title: 'sample index k' },
    yaxis: { title: '‖r(k)‖' },
  }, { responsive: true });
}

// Build sidebar
const list = document.getElementById('slug-list');
Object.keys(PAYLOADS).forEach(slug => {
  const btn = document.createElement('button');
  btn.className = 'slug-button';
  btn.dataset.slug = slug;
  const tag = PAYLOADS[slug].residual_source === 'published-theta'
    ? '<span class="source-tag literal">literal</span>'
    : '<span class="source-tag">proxy</span>';
  btn.innerHTML = slug + tag;
  btn.addEventListener('click', () => {
    document.querySelectorAll('.slug-button').forEach(b => b.classList.remove('active'));
    btn.classList.add('active');
    render(slug);
  });
  list.appendChild(btn);
});
// Auto-load the panda_gaz exemplar on first paint
const first = document.querySelector('.slug-button[data-slug=panda_gaz]') ||
              document.querySelector('.slug-button');
if (first) first.click();
</script>
</body>
</html>
"""


def main() -> int:
    payloads = {}
    for slug in ALL_SLUGS:
        print(f"  building {slug}...", flush=True)
        d = build_one_dataset(slug)
        if d is not None:
            payloads[slug] = d
    OUT_HTML.parent.mkdir(parents=True, exist_ok=True)
    html = render_html(payloads)
    OUT_HTML.write_text(html)
    size_kb = OUT_HTML.stat().st_size / 1024
    print(f"\nemitted {OUT_HTML.relative_to(CRATE_ROOT)} ({size_kb:.0f} KB, {len(payloads)} datasets)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
