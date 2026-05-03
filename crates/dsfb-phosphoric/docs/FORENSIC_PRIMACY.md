# Forensic Primacy

## Apex statement

**Phosphoric is a drift-and-slew residual court, not a log analyzer.**

Phosphoric is not for writing software. Phosphoric is for writing evidence.

The system is judicial, not observational. The two systems ask
different questions:

> A log analyzer asks: *"What interesting things might have happened?"*
>
> A drift-and-slew residual court asks: *"Which declared invariant was
> violated, according to the evidence?"*

The compiler's job is to materialize residual structure such that the
court can render a single named verdict per incident. Every authority
boundary must produce a typed, deterministic residual; missing or
malformed residuals are themselves evidence of a specific drift class.

## Canonical role mapping

| Role        | Phosphoric / DSFB-gray artifact                       |
|-------------|-------------------------------------------------------|
| narrative   | logs (NOT used)                                       |
| evidence    | residuals (R1..R7, 32-byte fixed records)             |
| law         | manifest (declared task seal authority)               |
| court       | classifier (`tools/phosphoric-host/phosphoric_drift.phos`) |
| verdict     | drift class (`DriftClass` enum, 8 values)             |
| case file   | `.pfi` incident container                             |

There are NOT: logs, severity levels, free-text events, traces, anomaly
scores, or filterable observability streams.

## Implementation posture (2026-05-03)

Phosphoric's core value is **deterministic residual adjudication on
constrained industrial edge chips**, not compiler breadth. Active
implementation work is admitted in this priority order:

| Priority | Family | Court relevance |
|---|---|---|
| **A** | syscall / load32 evidence boundary | deterministic exit, evidence output, fixed byte reads, runtime boundary support |
| **B** | struct ABI / residual-record manipulation | residuals, PFI headers, manifest edges, task seals, verdicts are byte-structured records |
| **C** | bounded fixed-buffer scanning | bounded scanning over residual records, PFI bytes, manifest tables |
| **D** | minimal verdict match selection | deterministic classification from typed evidence |
| **E** | call / nested-call (only as needed) | useful substrate; admit only specific call shapes the court runtime invokes |
| **F** | source-parser helpers | only if source parsing is on-device; not necessarily a court-runtime requirement |
| **G** | quine / deep bootstrap polish | engineering retirement of the ASM stub |

The source↔ASM closure campaign (Sessions B–S, 2026-05-02 →
2026-05-03) reached 51 / 82 byte-equal; the remaining 31 fixtures
are scheduled named work in the campaign restart per `GOAL.md`
§"Bootstrap discipline" and tracked at
[`SELFHOST_BACKLOG.md`](SELFHOST_BACKLOG.md). Each fixture is
admitted under one of court need, x86 proving-ground need, or
edge-deployment need (per `FIXTURE_RAZOR.md`).

> Unused expressivity is not neutral on constrained edge chips.

### Court Requirements A1/B1 + D1 (2026-05-03) — host-reference closed loop

The first court-side end-to-end reproducibility unit. Given the
canonical declared-MMIO-violation vector (declared_lo=0x1000,
declared_hi=0x10FF, observed=0x1100), the entire chain from inputs
to canonical verdict is reproducible by a deterministic host
reference court:

```
input vector
  → R5 32-byte record   (tools/court/emit_r5_record.sh)
  → PFI0 192-byte case  (tools/court/emit_mmio_boundary_pfi.sh)
  → 6-line verdict      (tools/court/verdict_from_pfi.sh)
```

- **A1/B1** — `tools/court/emit_r5_record.sh` composes the 32-byte
  R5 record per `kernel/residual.phos`'s chain_step mixer (primes
  31, 131, 524287, 16777213); `tools/court/emit_mmio_boundary_pfi.sh`
  wraps the record with manifest_hash sentinel, image_hash sentinel,
  SHA-256 stream_hash, final_chain_hash, and reserved zero regions
  per `docs/PFI0.md`. `tools/verify/check_court_a1_b1.sh` (Make
  target `verify-court-a1-b1`) asserts `cmp` byte-equal between the
  emitter output and the anchor at
  `tools/verify/fixtures/pfi/mmio_boundary_violation.pfi`.
- **D1** — `tools/court/verdict_from_pfi.sh` reads a 192-byte PFI0
  containing one R5 record, parses record[0] (kind, seq,
  declared_lo/hi, observed), and writes the canonical 6 lines per
  §3 below. R5-only by scope (any other kind is a hard error).
  `tools/verify/check_court_d1_verdict.sh` (Make target
  `verify-court-d1-verdict`) runs `emit_mmio_boundary_pfi.sh` to
  produce the PFI0 (not the anchor), pipes the produced bytes
  through `verdict_from_pfi.sh`, and asserts the output is byte-
  identical to
  `tools/verify/fixtures/verdicts/mmio_boundary_violation.expect`.

The earned forensic claim is: **given the canonical MMIO violation
vector, the host reference court deterministically produces the R5
PFI0 case bytes and derives the canonical MMIO_BOUNDARY_PRESSURE
verdict bytes byte-identical to the locked expectation.** Every
stage is reproducible from inputs.

The not-yet-earned claim is: **bytes emitted / verdict bytes derived
by a Phosphoric-compiled binary**. All four steps remain host-side
bash + awk + od + sha256sum. The framing throughout this section is
"host reference emitter produced" / "host reference verdict path
produced" — never "Phosphoric runtime classifier executes" / "court
runtime adjudicates" / "compiled classifier emits verdict".
Replacing the host emitters with Phosphoric-compiled binaries that
emit the same 192 + 104 bytes is the next reserved court
requirement.

### Court Requirement B1 (narrow) (2026-05-03) — R5 payload semantic validity

In addition to the produce/verdict chain above, the host reference
court enforces the missing **semantic** invariant on R5 mmio_touch
payloads — namely that the recorded payload encodes an *actual*
boundary violation:

```
record[0].kind == 5
record[0].observed ∉ [record[0].declared_lo, record[0].declared_hi]
```

Layout / chain / hash / malformed-PFI validation are owned by the
existing gates (`verify-pfi-layout`, `verify-residual-r5-byte-layout`,
`verify-malformed-pfi`, `verify-court-a1-b1`,
`verify-court-d1-verdict`). B1 adds *only* the missing semantic check,
without re-implementing those gates' work.

The validator at `tools/court/validate_r5_case.sh` parses only the
four fields needed (`kind`, `declared_lo`, `declared_hi`,
`observed`); the gate at `tools/verify/check_court_b1_case_validity.sh`
(Make target `verify-court-b1-case-validity`) runs the A1/B1 emitter
and pipes the produced bytes through the validator. For the canonical
vector the invariant holds because `0x1100 > 0x10FF`. An R5 record
with `observed` inside `[declared_lo, declared_hi]` is semantically
invalid evidence (an in-range touch mislabeled R5) even if the layout
passes; B1 is what catches that.

The earned forensic claim is: **the host reference court not only
reproduces the R5/PFI0/verdict bytes, but also validates that the R5
payload semantically represents a real boundary violation.** The
not-yet-earned claim is: **general PFI validator / general R5
classifier / runtime enforcement / Phosphoric-compiled validation** —
B1 is a single-invariant host reference check, not a general parser.

### Single-case R5 host-reference court loop — closed and saturated (2026-05-03)

After A1/B1 + D1 + B1-narrow landed, an overlap analysis at the
proposed C1 (bounded fixed-buffer scan) admission point found that
the bounded-walk invariants — count-driven iteration, fixed 32-byte
record stride, no file-length / sentinel / unbounded scanning — are
**already structurally enforced** by `verify-pfi-layout`:

- `size == 192 + 32 * residual_count` is asserted, so count is the
  only free parameter; an "extra record" is structurally
  unrepresentable.
- The walker iterates `for i in 0..residual_count`, not by EOF or
  sentinel.
- Footer immediately abuts the last record; the existing gate
  rejects truncation and trailing slack.

C1 was therefore declined under the razor: a separate bounded-scan
gate would duplicate work the existing gate set already polices.
Together with the existing layout / R5-byte-layout / chain-hash /
stream-hash / malformed-PFI / verdict-replay / no-silent-authority
gates, the **single-case R5 host-reference court loop is closed and
saturated**. The repository proves the canonical MMIO violation
vector can be deterministically transformed into an R5 PFI0 case,
adjudicated into the canonical MMIO_BOUNDARY_PRESSURE verdict, and
semantically validated as an actual boundary violation.

This is the saturation point of host-side bash + awk + od +
sha256sum. Further progress on the court requires an **explicit
trigger**:

1. **Promotion trigger.** A Phosphoric-compiled binary emits the
   192-byte PFI0 case (or the 6-line verdict / B1 validation) and
   replaces the corresponding host-reference tool in the gate.
   Earns the strong claim "Phosphoric-toolchain-produced binary
   emits PFI bytes byte-identical to host emitter." This is named
   future work; the implementation path lands when scheduled in
   `SELFHOST_BACKLOG.md`.
2. **Breadth trigger.** A second residual kind (R6, R3, …) is
   admitted only if a specific court scenario requires it. Each
   new kind opens its own A1/B1+D1+B1-narrow chain.
3. **Replay trigger.** A multi-record PFI is admitted only if a
   bounded multi-record replay scenario matters. The layout gate
   already supports `count > 1` structurally; only a fixture +
   verdict expectation would be required.
4. **Edge trigger.** A syscall / load32 / struct-ABI fixture from
   the campaign backlog is promoted only if a specific edge target
   requires the court to emit / store / read evidence on-device.

None of these triggers has fired as of 2026-05-03. The court holds
position at the saturation point.

### ASM trust anchor (per GOAL.md)

Court semantics are defined by **source / doctrine / gates**, layered
on top of byte-equal-verified Phosphoric on top of an honest ASM
trust anchor. Per `GOAL.md` §"Bootstrap discipline":

- ASM is the trust anchor at the bottom of the bootstrap chain.
  This is a universal property of language bootstrapping (C from
  PDP-11, Rust from OCaml, Go from C); not a fantasy to escape.
- The audit floor is byte-equal Phosphoric → ASM closure proven by
  the 82-fixture campaign. 51 / 82 closed; 31 scheduled.
- HOST_REFERENCE emitters (bash + awk + sha256sum) are admitted
  only as transitional scaffolding while the Phosphoric-compiled
  emitters are still being grown. They are not a permanent state.
- Promoting a host-reference emitter to a Phosphoric-compiled
  emitter (the "promotion trigger" above) is named future work,
  on a path scheduled by `SELFHOST_BACKLOG.md`. Producer extensions
  to support that promotion are in scope under the campaign
  restart, analogous to Sessions B–S Exception A work.

## 1. Residual-first semantics

**Rule:** No observable effect occurs without a corresponding residual
emission.

For each authority boundary in source, the producer inserts a residual
emission of a specific kind:

| Boundary                  | Residual kind | Tag |
|---------------------------|---------------|-----|
| CAP ISSUE / CAP REVOKE    | cap_graph_delta   | R1 |
| IPC SEND / IPC RECV       | ipc_route_delta   | R2 |
| BUDGET USE / LOOP EXIT    | budget_pressure   | R3 |
| EFFECT ENTRY              | effect_trace      | R4 |
| MMIO TOUCH                | mmio_touch        | R5 |
| TASK TRANSITION           | task_transition   | R6 |
| BOOT / ATTEST             | boot_check        | R7 |

**Compiler obligations:**
- Insert emission sites deterministically (same source bytes → same
  insertion order, no scheduling variance).
- Preserve emission order across optimization.
- Forbid dead-code elimination that removes emissions.

## 2. Residual ABI shape

Every emission is a **fixed 32-byte record**:

```
struct Residual {
    kind: u8                  // 1..7 (R1..R7)
    arch_id: u8               // architecture identifier
    seq: u16                  // strictly monotonic, wrap allowed, no gaps
    cycle: u64                // hardware cycle counter (deterministic per run)
    payload: [u8; 14]         // kind-specific, byte-stable layout
    chain_hash: [u8; 4]       // H(chain_hash[n-1] || event_bytes)[0..4]
}
```

**Invariants (compiler-enforced):**
- `seq` strictly monotonic across the entire emission stream. Wrap is
  allowed; gaps are not.
- `chain_hash[n] = H(chain_hash[n-1] || event_bytes)`. Initial value
  is the boot attestation hash.
- Identical inputs → identical residual stream byte-for-byte.

**Compiler obligations:**
- Guarantee payload packing is byte-stable (no padding ambiguity, no
  host-dependent layout, no struct-reordering optimization).
- The 14-byte payload schema is per-kind and frozen at the ABI level —
  compiler must not rearrange fields.

## 3. Deterministic classification

Classification is a pure function:

```
classify(residual_stream, manifest) → DriftClass
```

The output type is a **closed enum**:

```
enum DriftClass {
    NO_DRIFT,
    AUTHORITY_EXPANSION,
    SILENT_NARROWING,
    IPC_ROUTE_DIVERGENCE,
    MMIO_BOUNDARY_PRESSURE,
    STACK_BUDGET_PRESSURE,
    TASK_STATE_SLEW,
    BOOT_ATTESTATION_MISMATCH,
}
```

**Invariants:**
- Exactly one class per incident. No "mostly NO_DRIFT" or compound
  classifications.
- No probabilistic scoring. No threshold tuning.
- Byte-identical replay → byte-identical class + report.

**Compiler obligations:**
- Emit all data required for classification at emission time.
- Do not require re-execution to classify (the residual stream is
  sufficient — the original program need not run again).

**Canonical verdict format.** The classifier emits exactly:

```
CLASS=AUTHORITY_EXPANSION
RESIDUAL=R1
SEQ=42
EXPECTED=cap_slot[0]
ACTUAL=cap_slot[1]
EXIT=2
```

Six lines. One verdict per incident. Every field load-bearing.

**Anti-verdicts the classifier MUST NOT emit:**

```
maybe suspicious                  ← probabilistic
probably abnormal                 ← heuristic
looks unusual                     ← observational
score = 0.83                      ← scoring
WARN: cap_slot mismatch           ← log severity
INFO: residual stream complete    ← observation, not judgment
```

A court does not say "looks unusual"; a court returns a named verdict
or it returns NO_DRIFT. The above are not produced regardless of how
ambiguous the evidence appears — ambiguous evidence still maps to
exactly one DriftClass under the rules.

**Exit-code convention** (per `tools/phosphoric-host/phosphoric_drift.phos`):

| Exit | Meaning                                                          |
|------|------------------------------------------------------------------|
| 0    | NO_DRIFT                                                         |
| 1    | classifier internal error (NOT a verdict)                        |
| 2    | AUTHORITY_EXPANSION / IPC_ROUTE_DIVERGENCE / TASK_STATE_SLEW     |
| 3    | BOOT_ATTESTATION_MISMATCH                                        |
| 4    | chain corruption (evidence chain_hash discontinuity)             |
| 5    | malformed `.pfi` container                                       |
| 6    | warning-class drift: SILENT_NARROWING / *_PRESSURE               |

## 4. Endoduction (residuals as grammar)

Residuals must be legible as **grammar**, not logs.

**Transform:**
```
(residual stream)
→ grouped by kind
→ reduced to deltas (declared vs observed)
→ mapped to named drift classes
→ emitted as minimal verdict
```

**Example verdict:**
```
INCIDENT 0xA3
CLASS=AUTHORITY_EXPANSION
RESIDUAL=R1
SEQ=42
EXPECTED=cap_slot[0]
ACTUAL=cap_slot[1]
```

**Rule:** No raw residual dump without a corresponding structured view.
The structured view is the verdict; the dump is forensic detail behind
it.

**Compiler/runtime obligations:**
- Payload fields map directly to "EXPECTED/ACTUAL" semantics. Avoid
  opaque encodings.
- Prefer small enums over free-form blobs.

## 5. Determinism requirements

Determinism MUST hold across:
- Recompile (same source, same producer → same residual stream).
- Replay (same input bytes → same residual stream).
- Reclassification (same residual stream + manifest → same verdict).
- Cross-machine execution (same arch).

**Forbidden:**
- Wall-clock timestamps (`time()`, `gettimeofday`, etc.).
- Heap allocation in the emission path.
- Unordered iteration (hash-map ordering, set traversal).
- Hash maps in emission logic (use fixed-size rings).

**Allowed:**
- Fixed-size rings (capacity declared at compile time).
- Compile-time constants.
- Monotonic counters.
- Hardware cycle counter (when deterministic per run on the target arch).

## 6. Minimality doctrine

Every added feature must answer:

1. Does this reduce post-failure ambiguity?
2. Does this produce a new residual class, or strengthen an existing one?
3. Is it required for deterministic classification?

If a feature does not satisfy at least one of these, it is rejected.

This applies to:
- Producer features (no new sub-passes that don't trace to a residual class).
- Runtime features (no instrumentation that isn't a residual emission).
- Tooling (no analyzer that isn't a classifier).

## 7. Court, not log analyzer

The judicial framing is not stylistic — it is structural. Treat the
system as a court at every design decision:

| Court term            | Phosphoric concept                                  |
|-----------------------|-----------------------------------------------------|
| Defendant             | Incident artifact (`.pfi` container)                |
| Evidence              | Residual stream (R1..R7 records, typed, monotonic)  |
| Closed registry of verdicts | `DriftClass` enum (eight values, no others)   |
| Judgment              | Output of `classify(stream, manifest)`              |
| Replay-determinism    | Same evidence → same verdict, byte-identical, every run |
| Defaulted verdict     | `NO_DRIFT` (returned only when evidence agrees)     |

A court does NOT do:
- **Sampling** — the court does not "decide based on representative
  records". All evidence is examined.
- **Severity grading** — there is no DEBUG/INFO/WARN/ERROR. Either
  evidence agrees with the manifest (`NO_DRIFT`) or it maps to exactly
  one named drift class.
- **Probabilistic scoring** — no "this is 73% likely to be drift".
  Either the evidence supports a named verdict or the system is broken.
- **Heuristic anomaly detection** — anomalies have no place. Drift
  is precisely the named taxonomy in §3; nothing outside that taxonomy
  can be reported.
- **Free-form observation** — there is no "info we kept in case it's
  useful". Every emission must be defensible as proof of a specific
  authority transition.

A log analyzer would offer all of the above. A court refuses them.

## 8. Non-goals

The following are explicitly NOT goals of Phosphoric:

- Logging frameworks. **Residuals are not logs.** They are evidence.
- Tracing layers (OpenTelemetry-style spans, etc.).
- Debug-only instrumentation that vanishes at release.
- Probabilistic anomaly detection.
- Heuristics without a closed classification mapping.
- Search / filter / aggregation UIs over residual streams. The court
  produces verdicts; it does not produce browseable observability data.
- Severity levels or log-level filtering of any kind.
- "Context" / "metadata" / "attributes" beyond the typed payload of an
  R1..R7 record. The 14-byte payload is the entire field surface; the
  court does not consider extra-record context.

The framing distinction is load-bearing: a log analyzer reads
free-form text for human eyeballs and tolerates ambiguity; a court
reads typed evidence and resolves to one of a closed set of verdicts
with zero ambiguity. Drifting toward the former dissolves
determinism; drifting toward the latter is the project's whole point.

## 8. Final principle

> The system is correct when every failure is small, named, and replayable.

Not when it is fast.
Not when it is expressive.
Not when it is feature-complete.

## Status as of 2026-04-30

**Producer-side runtime emission is still not implemented.** The
producer (`untracked/internaldocs/phase0_producer/phase0_stub.S`)
lowers a meaningful integer-with-conditionals subset (see
`docs/FIXTURE_RAZOR.md` for the inventory) but emits no R1..R7 records
on any input. The classifier (`tools/phosphoric-host/phosphoric_drift.phos`)
is a doctrine stub. `tools/verify/verify_residual_stream.sh` is a stub
that exits 0 informationally.

**Spec ABI + chain_hash determinism are now byte-pinned (Session 12,
2026-04-30).** The first residual-truth fixture
(`tools/verify/fixtures/residual_r1_byte_layout.phos`) and its gate
(`tools/verify/check_residual_byte_layout.sh`, wired into
`make verify-residual-byte-layout`) lock the 32-byte R1 record's field
declaration order/widths and the four `chain_step` prime constants
(31, 131, 524287, 16777213). On a fixed cap_issue test vector
(kind=1, arch_id=0, seq=1, cycle=0, payload=[1,5,0..0], prev=[0;4]),
the chain_hash output is byte-locked to [0xF8, 0x18, 0xF8, 0xE8]. Any
drift in the spec produces a different byte layout or chain_hash and
fails closed. This is the spec the producer must satisfy when runtime
emission lands; the byte-layout fixture is the anchor against which
runtime-emit fixtures (Session 14+) will be verified.

**PFI0 case-file format byte-stable (Session 13, 2026-04-30 — Stream C
Milestone A).** The `.pfi` evidentiary container layout is now closed
and byte-stable per [`docs/PFI0.md`](PFI0.md). The first case-file
fixture `tools/verify/fixtures/pfi/mmio_boundary_violation.pfi`
(192 bytes, sha256 `e689dbeb…`) encodes one R5 `mmio_touch` residual
for a task that touched address `0x1100` outside its declared MMIO
range `0x1000..0x10FF`. The gate `tools/verify/check_pfi_layout.sh`
(Make target `verify-pfi-layout`) verifies PFI0 magic, residual_count,
total file size, reserved-region zeroing, stream_hash =
sha256-of-records, kind closure, monotonic seq, chain_hash re-derived
per `kernel/residual.phos` `chain_step`, and `final_chain_hash`
anchored to the last record. R5 chain_hash on this fixture is
[0x8A, 0xA2, 0xCA, 0x5E].

**End-to-end one-case demonstration complete (Sessions 13–17, Stream C
Milestones A–E).** Following the panel reframe of 2026-05-01 ("Do not
start with ten residual types. Start with one fully sealed case."),
the R5 mmio_touch case is now sealed end-to-end at the spec level:

- **A. Container.** PFI0 layout byte-stable (Session 13, this section
  above).
- **B. R5 byte-layout pin.** `tools/verify/check_residual_r5_byte_layout.sh`
  (Make target `verify-residual-r5-byte-layout`) pins the R5 payload
  schema (declared_lo @0..2, declared_hi @2..4, observed_addr @4..8)
  and the chain_hash output [0x8A, 0xA2, 0xCA, 0x5E] on the canonical
  MMIO boundary test vector. Couples to mmio_boundary_violation.pfi
  by literal record-byte match. Session 14.
- **C. Verdict + replay.** `tools/verify/fixtures/verdicts/mmio_boundary_violation.expect`
  byte-locks the canonical 6-line verdict format (CLASS=, RESIDUAL=,
  SEQ=, EXPECTED=, ACTUAL=, EXIT=). `tools/verify/check_verdict_replay.sh`
  (Make target `verify-verdict-replay`) enforces the format, the
  closed DriftClass enum, the exit-code mapping, the cross-check
  against the .pfi's record[0], and rejects log-analyzer vocabulary.
  Same .pfi bytes → byte-identical verdict bytes. Session 15.
- **D. Adversarial rejection.** Seven malformed .pfi fixtures under
  `tools/verify/fixtures/pfi/malformed/` (bad_chain_hash, seq_gap,
  bad_kind, truncated_record, nonzero_reserved, bad_magic,
  stream_hash_mismatch) each rejected by check_pfi_layout.sh with a
  deterministic named violation. `tools/verify/check_malformed_pfi.sh`
  (Make target `verify-malformed-pfi`) enforces the closed
  expected-reason table — rejection for the wrong reason is itself
  a doctrine violation. The court refuses bad evidence with named
  verdicts, not "looks unusual". Session 16.
- **E. No silent authority.** [`docs/NO_SILENT_AUTHORITY.md`](NO_SILENT_AUTHORITY.md)
  locks the load-bearing sentence: "No authority transition may occur
  without either a declared manifest edge or a typed residual."
  `tools/verify/verify_no_silent_authority.sh` (Make target
  `verify-no-silent-authority`) enforces apex sentence presence,
  kernel `record()` fn intact with chain_step primes preserved, the
  closed R1..R7 + tail_marker taxonomy, and the boundary table listing
  each R<N> exactly once. This is the line that distinguishes
  Phosphoric from a tracing/observability system. Session 17.

The 60-second pitch is now demonstrable on this one case: declared
MMIO range, observed boundary touch, byte-stable typed residual, .pfi
case file, byte-identical verdict, replay idempotency, adversarial
rejection — all without a single timestamp, log line, or probabilistic
score. **Producer-side runtime emission is still pending** (Stream A
post-Session-17); when it lands, every gate above tightens to enforce
the runtime path matches the spec already pinned here.

This document is the contract that future producer / runtime / tooling
work must satisfy. Any change that violates determinism, ABI stability,
or the closed DriftClass set is doctrine-incompatible regardless of
local benefit.
