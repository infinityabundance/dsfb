# DSFB Phosphoric DSFB Gray Audit Report

## Run Summary

This report summarizes a local `dsfb-gray` scan of `crates/dsfb-phosphoric`.

- Command:
  `cargo run -p dsfb-gray --release --bin dsfb-scan-crate -- --out-dir crates/dsfb-phosphoric/output-dsfb-gray crates/dsfb-phosphoric`
- Scanner exit status: `0`
- Generated at UTC: `2026-05-03T23:48:09.302354712Z`
- Scan root: `/home/one/dsfb/crates/dsfb-phosphoric`
- Scanner crate: `dsfb-gray`
- Source SHA-256 reported by scanner: `2d211799c414c5da04e2a09337d51f277763f7abe8ea326b9006851308cf6a7e`
- VCS commit reported by scanner: not declared
- Path in VCS reported by scanner: not declared

No push was performed as part of this audit.

## Output Artifacts

Raw outputs were written under:

`crates/dsfb-phosphoric/output-dsfb-gray/dsfb-gray-2026-05-03T23-48-09Z/`

Generated artifact hashes:

| Artifact | SHA-256 |
| --- | --- |
| `dsfb_phosphoric_scan.txt` | `b6af18b4a1f95d85c270bef681f24c6bbfe9a734fe1bb964263f9c3791adcd95` |
| `dsfb_phosphoric_scan.sarif.json` | `5babddb9384cd743d7b765522e76f6b444abc8efd9df1f999d9cc84ab5d5c074` |
| `dsfb_phosphoric_scan.intoto.json` | `1f32dfa6e66f603925c78cd2fc0ae3af765301c2ca29539e4270981c7a46a69d` |
| `dsfb_phosphoric_scan.dsse.json` | `43f92f040156b02f7bdd24d0dfc587064ab6184cf0980a80477e56b285a93e3a` |

The DSSE envelope is unsigned unless `DSFB_SCAN_SIGNING_KEY` is set for a signed run.

## Scope And Method Limits

`dsfb-gray` is primarily a Rust crate static-audit tool. `dsfb-phosphoric` is an artifact-heavy reproducibility package rather than a normal Rust source crate. The scan therefore reported:

- Source files scanned: `0`
- Artifact files inspected: `26399`
- Matched heuristics: `0`

That means this audit is useful as a package-surface, governance, provenance, and review-readiness check. It is not a meaningful Rust function-body complexity audit for this folder, because no Rust function bodies were extracted. Function-level findings marked `indeterminate` should be read as scanner coverage limits, not as proof of defects in the phosphoric runtime or `.phos` artifacts.

The scanner also observed license and notice files under archived version folders because it inspected the full folder tree. This report did not edit `v0.1`, `v0.2`, or `v0.3`.

## Score Summary

Scanner overall score:

`73.3% (developing but substantial assurance posture)`

Advisory subscores:

| Area | Score |
| --- | ---: |
| Correctness | `82.2` |
| Maintainability | `79.5` |
| Concurrency / Async | `87.5` |
| Resource Discipline | `100.0` |
| Verification / Reviewability | `62.2` |
| Assurance / Provenance | `69.4` |

Rubric sections:

| Section | Score | Weight | Points | Checks |
| --- | ---: | ---: | ---: | ---: |
| Safety Surface | `80.0` | `15.0` | `12.0` | `5` |
| Verification Evidence | `20.0` | `15.0` | `3.0` | `5` |
| Build / Tooling Complexity | `100.0` | `10.0` | `10.0` | `6` |
| Lifecycle / Governance | `38.5` | `10.0` | `3.8` | `13` |
| NASA/JPL Power of Ten | `80.0` | `25.0` | `20.0` | `10` |
| Advanced Structural Checks | `97.8` | `25.0` | `24.5` | `23` |
| Overall | `73.3` | `100.0` | `73.3` | `62` |

This score is a broad source-visible review-readiness indicator from the scanner rubric. It is not a certification result, runtime correctness proof, or universal reproducibility claim.

## Findings

| Finding | Scanner State | Audit Interpretation |
| --- | --- | --- |
| `P10-10` Pedantic warnings and static analyzers are enforced | Not applied | The packaged scan surface did not expose warning-strictness or static-analyzer signals. For this folder, the practical gap is machine-readable evidence that the documented verification commands are enforced in CI or release practice. |
| `NASA-CC` Cyclomatic complexity hotspot audit | Indeterminate | No function summaries were extracted. This is a scanner applicability limit for `dsfb-phosphoric`, not evidence that complexity is high. |
| `P10-5` Assertion density averages at least two per function | Indeterminate | No function bodies were extracted. This cannot be evaluated by the current Rust-oriented scanner against the phosphoric artifact surface. |
| `P10-7` Return values are checked and parameters are validated | Indeterminate | No obvious unchecked-return motifs were observed, but full validation and propagation cannot be proven by this scanner on this package. |

No function hotspots were extracted.

## Positive Evidence

The scanner reported:

- No explicit unsafe sites.
- No panic-like sites.
- No unwrap/expect-like sites.
- No FFI boundary sites.
- No heap-allocation motifs observed.
- No build dependencies, direct dependencies, dev dependencies, `build.rs`, proc macro, or native-build signals.
- README present.
- Architecture/design documentation present.
- `docs/` content present.
- Tests directory present.
- NASA/JPL Power of Ten: `7` applied, `1` not applied, `2` indeterminate.
- Advanced structural checks: `22` clear, `0` elevated, `1` indeterminate.

These are source-visible scanner observations only. They do not replace the project-specific QEMU and reproducibility tests.

## Risk Assessment

The highest-confidence actionable gap is evidence packaging for verification and lifecycle governance, not a detected runtime defect.

The weak areas in the raw score are:

- Verification Evidence: `20.0%`
- Lifecycle / Governance: `38.5%`
- P10-10 analyzer/static-check visibility

For this folder, those low scores mostly mean `dsfb-gray` does not see enough conventional Rust crate and CI metadata in the package surface. The right remediation is to improve explicit, machine-readable audit evidence around the actual verification path, not to claim that the phosphoric artifacts failed Rust code checks.

## Recommended Follow-Up

1. Add or expose a machine-readable verification gate that `dsfb-gray` can recognize, or extend `dsfb-gray` so it understands this project's `make verify`, `make verify-court-active`, Colab, QEMU, and release-packaging evidence.
2. If `dsfb-gray` is intended to audit phosphoric semantics, extend it to parse `.phos` files and extract domain-relevant units, assertions, return/transition checks, and complexity measures. Without that, Rust function-level checks will remain indeterminate.
3. Consider adding folder-local `CHANGELOG.md`, `SECURITY.md`, and `SAFETY.md` or explicit links to inherited top-level policies. This would improve lifecycle/governance evidence without changing byte-matched runtime artifacts.
4. Consider adding path-exclusion support to `dsfb-gray` for archival folders when the desired scan scope is only the active package surface. The current run inspected `v0.1`, `v0.2`, and `v0.3` as artifacts.
5. Keep treating QEMU marker tests, release hashes, and Colab reproduction logs as the primary empirical evidence for `dsfb-phosphoric`; use `dsfb-gray` as an auxiliary static review-readiness signal.

## Non-Claims

This audit does not claim:

- compliance or certification against any external standard;
- bit-identical behavior across every host;
- absence of runtime bugs;
- that archived version folders were modified;
- that indeterminate Rust function-level checks represent concrete phosphoric defects.

