# AGENTS.md — Standing Rules for AI Agents in This Workspace

**Read this file completely before taking any action in this workspace.**
**Every rule here is a hard constraint, not a suggestion.**

---

## 1. Output Folder — Non-Negotiable Naming Convention

All generated output goes to `/home/one/dsfb-rf/dsfb-rf-output/`.

Every run creates a **new** timestamped subfolder named:
```
dsfb-rf-<YYYY-MM-DD_HH-MM-SS>/
```

**Example:** `dsfb-rf-2026-04-09_05-01-49/`

This is hardcoded in `dsfb-rf/examples/generate_figures_all.rs`. Do not override
it. Do not create folders with any other naming scheme. Do not reuse an existing
folder. Every invocation of `cargo run --example generate_figures_all` creates
a new `dsfb-rf-<timestamp>/` folder automatically.

### Contents of every output folder (all required, no exceptions):
```
dsfb-rf-<timestamp>/
  figs/                              — 50 individual figure PDFs + PNGs
  dsfb-rf-all-figures.pdf            — all figures merged into one PDF
  figure_data.json                   — Phase-1 engine data (85 KB)
  figure_data_all.json               — all-phases engine data (~1 MB)
  dsfb-rf-<timestamp>-artifacts.zip  — complete zip of above
```

The single command that does all of this:
```sh
cd /home/one/dsfb-rf/dsfb-rf
cargo run --example generate_figures_all --features std,serde
```

---

## 2. The `paper/` Folder — ABSOLUTE OFF-LIMITS

**NEVER write to `/home/one/dsfb-rf/paper/` or any path containing `/paper/`.**

This folder contains the author's LaTeX source, PDFs, and paper stack. It is
read-only from the agent's perspective. Writing anything to it — including
`figure_data.json`, PNGs, or any other file — is a critical violation.

- Do NOT run `cargo run --example generate_figures` (writes to `paper/` by default)
- Do NOT ever `create`, `write`, or `copy` any file into any path matching `*/paper/*`
- If a script or example attempts to write there, fix the path first, then run it

The correct pipeline never touches `paper/`:
```
generate_figures     → dsfb-rf-output/figure_data.json
generate_figures_all → dsfb-rf-output/figure_data_all.json
figures_all.py       → dsfb-rf-output/dsfb-rf-<timestamp>/figs/
```

---

## 3. Workspace Layout

```
/home/one/dsfb-rf/
  dsfb-rf/           ← Rust crate root (Cargo.toml lives here)
  dsfb-rf-output/    ← ALL generated output. Never anything else.
  paper/             ← READ-ONLY. LaTeX source. Never touch.
  gr-dsfb/           ← GNU Radio OOT module
  .github/           ← CI workflows
  AGENTS.md          ← this file
  CONVENTIONS.md     ← engineering standards
```

When running `cargo` commands, `cd` to `/home/one/dsfb-rf/dsfb-rf` first.

---

## 4. No Stray Directories

Do not create any directory that was not explicitly requested. In particular:
- No `paper/` anywhere under `dsfb-rf/`
- No new top-level folders under `/home/one/dsfb-rf/` without explicit instruction
- No `tmp/`, `out/`, `test_output/`, or any ad-hoc folder outside `dsfb-rf-output/`

---

## 5. Zero Warnings Policy

`cargo check --examples --features std` must produce zero warnings at all times.
Before completing any task that touches `.rs` files, verify:
```sh
cargo check --examples --features std 2>&1 | grep warning
```
If warnings appear, fix them before declaring the task done.

---

## 6. Never Touch the Paper's LaTeX

`/home/one/dsfb-rf/paper/dsfb_rf_v2.tex` is only modified when explicitly asked.
When modifying it:
- Match the existing IEEEtran style exactly
- Do not change any theorem, lemma, or proof statement without instruction
- Do not remove limitations disclosures (L1–L12) or bounded-claims language
- Do not add overclaiming language (no "always", "guarantees", "optimal")

---

## 7. Code Integrity Rules

- `#![forbid(unsafe_code)]` is set in `lib.rs` — never remove it
- `#![no_std]` compatibility must be preserved at all times
- Never add heap allocation in the hot path (`observe()` call chain)
- No new dependencies without explicit instruction — the crate must remain `no_std`/`no_alloc` compatible
- All new public functions must have doc comments with `# Examples`

---

## 8. Before Acting on Any Multi-Step Task

1. Read this file (`AGENTS.md`) — you are doing this now
2. Check which directory to `cd` to before running commands
3. Verify the output path before any file write
4. Run `cargo check` after any `.rs` edit
5. Never reuse an existing `dsfb-rf-output/` subfolder

---

## 9. Sequential Tool Use

Do not run multiple terminal commands in parallel. Run one, wait for output,
then run the next. This avoids race conditions on output files.

---

## 10. When in Doubt

Stop and ask. Do not guess at folder names, paths, or whether something should
be written to `paper/`. The cost of asking is zero. The cost of another
`paper/` violation is not.
