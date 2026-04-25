# Standards alignment

DSFB's robotics observer is deliberately **out of scope** for
safety-rated certification (see [`SAFETY.md`](../SAFETY.md)
§"Functional safety"). This document enumerates the standards DSFB
aligns with or interoperates with — **without claiming certification
under any of them**.

## Controller interface standards

### ROS 2 `diagnostic_msgs`

DSFB episode output maps cleanly to the ROS 2 diagnostic-message
format (`diagnostic_msgs/DiagnosticArray` /
`diagnostic_msgs/DiagnosticStatus`):

| DSFB field | Maps to |
|---|---|
| `Episode.grammar` | `DiagnosticStatus.level` (`OK` / `WARN` / `ERROR`) |
| `Episode.decision` | `DiagnosticStatus.message` (`Silent` / `Review` / `Escalate`) |
| `Episode.index` | `DiagnosticStatus.values[].key = "sample_index"` |
| `Episode.residual_norm_sq`, `drift` | `DiagnosticStatus.values[]` as name / value pairs |

The mapping is **publish-only**: DSFB writes diagnostics, never
reads robot control commands. This matches the non-intrusion
contract in [`non_intrusion_contract.md`](non_intrusion_contract.md).

### OPC UA Robotics (IEC 62541, Companion Specification 40010)

DSFB can populate OPC UA Robotics `MotionDevice/HealthIndicator`
nodes with:

- Grammar state via a `StructuralHealthIndicator` object.
- Episode traces via a `HistoricalHealthRecord` mirror.

Again, read-side only — DSFB does not subscribe to
`MotionControlCommand` nodes or any other write path into the
motion-device state.

## Functional-safety standards — **interop, not certification**

### ISO 10218-1:2025 / ISO 10218-2:2025 (industrial robots)

- **Not certified.** DSFB is not a safety-related part of the
  control system.
- **Interop:** DSFB outputs can be routed to the operator
  review-surface mandated by ISO 10218-2 §5.11 (Information for
  use). DSFB never participates in the protective-stop functions
  required by ISO 10218-1 §5.5.
- **Integrator obligation:** any automation built on DSFB outputs
  (e.g. an alert rule that triggers a protective stop based on
  `Violation` episodes) is safety-rated by the integrator under the
  standard's rules, not by DSFB.

### IEC 61508 (functional safety of E/E/PE safety-related systems)

- **Not certified.** DSFB is an SC-1 / SIL-1-adjacent advisory
  layer at best; no IEC 61508 claim is made.
- **Interop:** DSFB is compatible with an IEC 61508 mixed-
  criticality architecture where the observer runs on a non-
  safety-rated partition / processor, producing advisory outputs
  over a read-only interface to the safety-rated controller.

### ISO 13849 (safety of machinery)

- **Not certified.** DSFB is outside the scope of ISO 13849's
  performance-level categorisation.
- **Interop:** DSFB does not affect the PLr determination for any
  upstream safety function.

### ISO 13482 (personal-care robot safety)

- **Not applicable** — DSFB is not designed for personal-care
  robotics. Deployments on humanoid platforms (e.g. iCub) are in
  research settings, not personal-care deployments.

## Metrology standards

### GUM JCGM 100:2008 (Guide to the Expression of Uncertainty in Measurement)

- DSFB's uncertainty budget is documented in
  [`uncertainty_budget_gum.md`](uncertainty_budget_gum.md).
- The [`uncertainty`](../src/uncertainty.rs) module implements Type
  A / Type B component accumulation and expanded-uncertainty
  coverage factors per JCGM 100:2008 §5 and §6.

## Interoperability test coverage

- Non-intrusion contract: covered by
  [`non_intrusion_contract.md`](non_intrusion_contract.md) and
  enforced at compile time.
- Determinism / reproducibility: covered by `paper-lock`'s bit-exact
  tolerance gate (`tests/paper_lock_binary.rs`).
- Memory safety: covered by `#![forbid(unsafe_code)]` + Miri ×3
  clean + Kani proofs (see `audit/`).

## What DSFB is NOT

To prevent any reviewer confusion:

- **Not a safety controller.** Not ISO 10218-rated, not IEC 61508-rated.
- **Not a fault-detection and diagnosis (FDD) classifier.** DSFB does
  not classify fault types or identify root cause.
- **Not a replacement** for any incumbent controller, observer, or
  diagnostic system. Augmentative only.

See [`README.md`](../README.md) §"Non-Claims" for the authoritative
non-claim table.
