// DSFB Chemical Court "what-if" simulator — browser glue for the raw-export WASM module.
//
// Flow (all synchronous, single-threaded — matching the module's static-buffer contract):
//   1. fetch + instantiate dsfb_court_sim.wasm  (no imports → instantiate with {})
//   2. write the residual stream (f64) into the module's IN_BUF via dsfb_sim_in_ptr
//   3. call dsfb_sim_run(n, k, band, window) → episode count; tokens land in OUT_BUF
//   4. read the per-sample grammar tokens from OUT_BUF via dsfb_sim_out_ptr and render
//
// The residual stream is the IMMUTABLE EVIDENCE: its SHA-256 (computed once, in JS) is displayed and never
// changes as the operator drags the envelope sliders. This is a what-if/training tool — it never mutates a
// sealed Court Record. See index.html for the full non-claims banner.

// Grammar token codes (must match dsfb_chemical_engineering_core::GrammarState order / the wasm token_code).
const TOKENS = [
  { code: 0, tag: "NOM", name: "Nominal",          css: "--nom" },
  { code: 1, tag: "DA",  name: "DriftAccum",        css: "--da"  },
  { code: 2, tag: "SS",  name: "SlewSpike",         css: "--ss"  },
  { code: 3, tag: "EV",  name: "EnvViolation",      css: "--ev"  },
  { code: 4, tag: "BG",  name: "BoundaryGrazing",   css: "--bg"  },
  { code: 5, tag: "RC",  name: "Recovery",          css: "--rc"  },
  { code: 6, tag: "CP",  name: "Compound",          css: "--cp"  },
  { code: 7, tag: "SF",  name: "SensorFault",       css: "--sf"  },
];
const WINDOWS = [8, 16, 32]; // the core's monomorphised drift-window sizes

function cssColor(varName) {
  return getComputedStyle(document.documentElement).getPropertyValue(varName).trim() || "#888";
}

async function main() {
  const colors = TOKENS.map((t) => cssColor(t.css));

  // ── load the sample residual stream + the wasm module ──────────────────────────────────────────────
  let sample, wasm;
  try {
    sample = await (await fetch("sample_residuals.json")).json();
    const bytes = await (await fetch("dsfb_court_sim.wasm")).arrayBuffer();
    wasm = (await WebAssembly.instantiate(bytes, {})).instance;
  } catch (e) {
    document.getElementById("digest").innerHTML =
      '<span class="err">Failed to load wasm/sample. Serve this folder over HTTP ' +
      "(e.g. <code>python3 -m http.server</code>) — file:// fetch is blocked by the browser.</span>";
    return;
  }
  const ex = wasm.exports;
  const residuals = Float64Array.from(sample.residuals);
  const n = Math.min(residuals.length, ex.dsfb_sim_max_samples());

  // Write the residuals into the module's input buffer ONCE (the evidence does not change between runs).
  const inPtr = ex.dsfb_sim_in_ptr();
  new Float64Array(ex.memory.buffer, inPtr, n).set(residuals.subarray(0, n));
  const outPtr = ex.dsfb_sim_out_ptr();

  // Immutable-evidence digest: SHA-256 over the exact residual bytes fed to the module. Computed once.
  const hash = await crypto.subtle.digest("SHA-256", residuals.buffer.slice(0, n * 8));
  const hex = [...new Uint8Array(hash)].map((b) => b.toString(16).padStart(2, "0")).join("");
  document.getElementById("digest").innerHTML =
    "<b>Immutable evidence digest (SHA-256 of the residual stream):</b><br><code>" + hex +
    "</code><br>This is constant across every what-if below — the envelope changes, the evidence does not.";

  // ── controls ───────────────────────────────────────────────────────────────────────────────────────
  const kEl = document.getElementById("k");
  const bandEl = document.getElementById("band");
  const winEl = document.getElementById("win");
  kEl.value = sample.k_default ?? 3.0;
  bandEl.value = sample.band_default ?? 0.1;
  winEl.value = String(WINDOWS.indexOf(sample.window_default ?? 16) >= 0 ? WINDOWS.indexOf(sample.window_default) : 1);

  const canvas = document.getElementById("strip");
  const ctx = canvas.getContext("2d");

  // Baseline (default-envelope) episode count, shown as the reference the what-if is compared against.
  const baselineEpisodes = ex.dsfb_sim_run(n, sample.k_default ?? 3.0, sample.band_default ?? 0.1, sample.window_default ?? 16);

  function render() {
    const k = parseFloat(kEl.value);
    const band = parseFloat(bandEl.value);
    const window = WINDOWS[parseInt(winEl.value, 10)];
    document.getElementById("kv").textContent = "= " + k.toFixed(1);
    document.getElementById("bv").textContent = "= " + band.toFixed(2);
    document.getElementById("wv").textContent = "= " + window;

    const episodes = ex.dsfb_sim_run(n, k, band, window);
    const tokens = new Uint8Array(ex.memory.buffer, outPtr, n);

    // Per-state tallies + derived counts (the wasm returns episodes; JS derives the rest from the stream).
    const tally = new Array(TOKENS.length).fill(0);
    let nonNominal = 0, breaches = 0;
    for (let i = 0; i < n; i++) {
      tally[tokens[i]]++;
      if (tokens[i] !== 0) nonNominal++;
      if (tokens[i] === 3 || tokens[i] === 6 || tokens[i] === 7) breaches++;
    }

    // ── draw: residual line + ±k envelope over a per-sample grammar-token strip ──────────────────────
    const W = canvas.width, H = canvas.height, stripH = 26, plotH = H - stripH;
    ctx.clearRect(0, 0, W, H);
    const colW = W / n;

    // token strip along the bottom
    for (let i = 0; i < n; i++) {
      ctx.fillStyle = colors[tokens[i]];
      ctx.fillRect(i * colW, plotH, Math.max(1, colW + 0.5), stripH);
    }

    // residual plot (auto-scaled so ±k and the data both fit)
    let amax = k * 1.2;
    for (let i = 0; i < n; i++) amax = Math.max(amax, Math.abs(residuals[i]));
    const mid = plotH / 2, scale = (plotH / 2 - 6) / amax;
    // envelope band ±k
    ctx.strokeStyle = "#e0a0a0"; ctx.setLineDash([4, 3]); ctx.lineWidth = 1;
    for (const s of [+1, -1]) {
      ctx.beginPath(); ctx.moveTo(0, mid - s * k * scale); ctx.lineTo(W, mid - s * k * scale); ctx.stroke();
    }
    ctx.setLineDash([]); ctx.strokeStyle = "#bbb"; ctx.beginPath(); ctx.moveTo(0, mid); ctx.lineTo(W, mid); ctx.stroke();
    // residual trace
    ctx.strokeStyle = "#222"; ctx.lineWidth = 1; ctx.beginPath();
    for (let i = 0; i < n; i++) {
      const x = i * colW, y = mid - residuals[i] * scale;
      i === 0 ? ctx.moveTo(x, y) : ctx.lineTo(x, y);
    }
    ctx.stroke();

    // ── counts panel ─────────────────────────────────────────────────────────────────────────────────
    document.getElementById("counts").innerHTML =
      "<b>" + episodes + "</b> fused episode(s) &nbsp;·&nbsp; <b>" + breaches + "</b> hard breach sample(s) " +
      "(EV/CP/SF) &nbsp;·&nbsp; <b>" + nonNominal + "</b> non-nominal of " + n + "<br>" +
      '<span style="color:#666">default-envelope baseline: ' + baselineEpisodes + " episode(s) — " +
      (episodes > baselineEpisodes ? "tighter envelope flags more"
        : episodes < baselineEpisodes ? "looser envelope flags fewer" : "same as baseline") + "</span>";

    document.getElementById("legend").innerHTML = TOKENS.map((t, i) =>
      '<span><span class="sw" style="background:' + colors[i] + '"></span>' + t.tag + " " + tally[i] + "</span>"
    ).join("");
  }

  for (const el of [kEl, bandEl, winEl]) el.addEventListener("input", render);
  render();

  document.getElementById("meta").textContent =
    "Sample: " + sample.label + " · " + n + " samples · module dsfb_court_sim.wasm (raw wasm32 exports, " +
    "no wasm-bindgen). Grammar = dsfb-chemical-engineering-core (fixed-point, SCALE=1e6).";
}

document.addEventListener("DOMContentLoaded", main);
