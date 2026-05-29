# Real-data drop-in — run DSFB on your own historian export

> How to point DSFB at a **real, ungated plant CSV** and what you get back. The framework runs unchanged on
> your data; nothing here redistributes plant data (only recipes + hashes are committed publicly), and the
> whole flow runs **locally on your machine**. Advisory, read-only.

## The one-line story
A role-labelled or plain wide CSV runs through the existing edge binary **unchanged** — there is no separate
"real-data mode". The same pipeline that produces the 20 public-dataset results produces your Court Record.

## Input formats DSFB accepts
| Format | Shape | Loader |
|---|---|---|
| **Wide CSV** | one column per tag, one row per sample (optional trailing `label` column) | `analyze` / `data-readiness` / `demo` |
| **Role-labelled CSV** + `<stem>.roles.json` | wide CSV + a sidecar declaring variable roles/units + a balance equation | `balance-witness` / `control-action` / `data-readiness` |
| **Timestamped historian CSV** | a `timestamp` column + tag columns (+ optional `batch`/`phase`) | `historian` |

The optional `<stem>.roles.json` sidecar (see `data/instrumented/*.roles.json` for worked examples) unlocks the
physical balances, the unit-consistency court, and control-loop context. Without it the bare values still run.

## The recommended flow on a real export
```bash
# 0. Grade the data BEFORE analysis — honest go/no-go + a punch-list.
cargo run --release -p dsfb-chemical-engineering-edge -- data-readiness path/to/your_export.csv
#    => Ready / ReadyWithCaveats / NotReadyMissingCriticalWitnesses, per-dimension findings.

# 1. (if a roles sidecar is present) prove the documented balance combines like-united channels.
cargo run --release -p dsfb-chemical-engineering-edge -- unit-consistency      # over the shipped balances
#    (a sidecar for your file enables the same check on your balance)

# 2. Analyse + write the sealed Court Record bundle.
cargo run --release -p dsfb-chemical-engineering-edge -- historian path/to/your_export.csv   # timestamped
cargo run --release -p dsfb-chemical-engineering-edge -- casefile <dataset-name>             # wide/role-labelled
```
`data-readiness` derives what a values file can show (tag/row counts, missingness) and reads a sibling
`<stem>.roles.json` for unit coverage + controller context; fields a bare values file **cannot** show
(timestamp span, duplicate timestamps) are reported as *not assessed* — never optimistically assumed — and
licence clearance is taken as `true` because you are running it locally on your own data (printed explicitly).

## Your data never has to leave your control
DSFB is a local read-only binary. You run the court on your machine, keep the raw bytes, and (when a reviewer
is involved) share **only** a redacted, hash-linked evidence bundle — no raw time-series, no real tag names.
See `docs/sbir_operator_data_request.md` for the input contract and the confidential-evaluation path (the
machine `PlantDataContractV1` / `ConfidentialEvaluationBundleV1` / `PartnerDataEscrowProtocolV1` objects land
in Wave 6).

## Honest maturity dependency (TRL)
The 20 committed datasets are 15 measured / 4 simulated / 1 agreement-gated (SWaT). The strongest real-data
evidence (SWaT) is **agreement-gated** — its raw data is not redistributed; only recipes + digests are. A full
real-plant validation (**TRL-4 → TRL-5**) requires a **user-supplied real ungated historian export** run
through this exact path. The framework is ready for it today; the evidence tier advances when an operator
provides the data. That dependency is the honest go/no-go in `docs/sbir_risk_register.md` (R1).

## What DSFB will not do on your data
No connection to control; no setpoint or valve moves; no root-cause claim; no requirement for your raw bytes to
leave your control. A `data-readiness` "Ready" is advisory data-quality guidance, **not** a guarantee of
analysis success, and not a data-validation certificate.
