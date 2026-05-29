# SBIR risk register — DSFB-Chemical-Engineering (honest)

> A candid risk register for a Phase-I-style evaluation/transition. Generic (no agency/program). Risks are
> stated plainly with mitigations and a residual level; nothing here is spun. It complements the paper's
> limitations section and the v2 panel review (`docs/legendary_panel_review.md`).

| # | Risk | Mitigation (in the artifact today) | Residual |
|---|---|---|---|
| R1 | **Public benchmarks ≠ a real plant.** 9/20 datasets are simulations; the strongest real-data win (SWaT) is agreement-gated. | Honest provenance tiers in MANIFEST; the readiness court + coverage map grade *your* data; drop-in path for a real ungated historian. | **Medium** — closes only with a real plant historian run. |
| R2 | **Execution coverage is a fraction of the catalogue.** 18/57 detectors and 7/12 fault signatures are executed; the rest are catalogued prior-art surface. | Marked honestly everywhere (executed vs catalogued); coverage is widened by *adding* executions, never trimming the catalogue. | **Medium** — narrows as more are executed. |
| R3 | **Determinism scope.** Byte-exact replay is achieved by fixed-point quantisation; arbitrary floating-point pipelines are not claimed bit-reproducible. | Stated as a Tier-1 fact with its exact mechanism (SCALE=1e6, `--fmad=false`); digest-equivalence harness gates every kernel. | **Low** — the boundary is disclosed and tested. |
| R4 | **GPU value is auditability, not throughput.** The evidence kernel runs far below the memory roofline at realistic sizes. | Framed as auditability/determinism in the paper + figures; CPU-vs-GPU end-to-end timing disclosed. | **Low** — honestly framed; not a throughput claim. |
| R5 | **Operator over-reads a CANDIDATE label as a diagnosis.** | Prominent claim-boundary banner + claim-strength legend at the top of every operator report; whole-report claim audit. | **Low** — every statement is tier-tagged. |
| R6 | **Numerical degeneracy on low-variance / dead channels.** | Baseline-constant channels fall back to raw deviation (the CSTR ~1.4e35 SPE explosion is fixed) + a regression test + Kani grammar-totality proof. | **Low** — guarded + machine-checked. |
| R7 | **Real plant data is proprietary / export-controlled.** | The confidential-evaluation path: operator runs locally, shares only a redacted hash-linked evidence bundle; raw data never egresses. | **Low** — the adoption blocker is removed by design. |
| R8 | **Embedded / edge real-time claim is aspirational.** | Disclosed as a `no_std` fixed-point design + a QEMU Cortex-M smoke run; explicitly *not* a real-time certification. | **Medium** — smoke run, not certified WCET. |
| R9 | **No SIS / safety authority.** | Stated as a hard boundary (IEC 61511): DSFB is advisory, read-only, independent of any safety layer. | **Low** — by design, never crossed. |

## What kills the program (stated honestly)
If, on a real ungated plant historian, the read-only court neither reduces operator triage load nor produces a
forensic record an engineer trusts more than the incumbent's, the value thesis fails — that is the honest
go/no-go. The mitigations above widen the evidence surface; they do not guarantee the plant-specific outcome,
which is exactly what a Phase-I evaluation on real data is for.
