## Claim Ledger

- DSFB detects 4/4 primary deterministic scenarios under the recommended configuration.
  Evidence: `data/evaluation_results.txt`, Section 1; generated from 4 primary scenarios.
- The recommended configuration is not zero-false-alarm in all clean windows.
  Evidence: `data/evaluation_results.txt`, Section 3; async starvation clean control produces a 3.0% false rate.
- Sensitivity behavior is configuration-dependent rather than universally robust.
  Evidence: `data/evaluation_results.txt`, Section 2; 19/42 sweep points show pre-injection alarms.
- Reproducibility is deterministic for the current clock-drift harness.
  Evidence: `data/evaluation_results.txt`, Section 4; 10/10 runs identical.
- DSFB provides structurally distinct detection-point signatures across the primary scenarios.
  Evidence: `data/evaluation_results.txt`, Section 5.
- The companion crate now emits one canonical broad audit rather than primary profile-specific reports.
  Evidence: `docs/generated/AUDIT_CONTRACT.md`; regenerated from `cargo run --bin dsfb-regenerate-public-artifacts`.
- The audit score method is `dsfb-assurance-score-v1` and is treated as a broad improvement/readiness guide rather than certification.
  Evidence: `docs/generated/AUDIT_CONTRACT.md` and `docs/AUDIT_SCORING_LOCKED.md`.
- The audit report includes conclusion lenses over one shared evidence set rather than separate primary scan modes.
  Evidence: `docs/generated/AUDIT_CONTRACT.md`; mirrored in the current scan report contract.
- The scanner emits SARIF, in-toto, and DSSE artifacts as part of the established public contract.
  Evidence: `docs/generated/AUDIT_CONTRACT.md` and the generated scanner outputs in `output-dsfb-gray/`.
