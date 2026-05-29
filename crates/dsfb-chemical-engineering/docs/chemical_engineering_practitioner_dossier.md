# Chemical-engineering practitioner dossier — what DSFB is (and isn't) for a working engineer

**Purpose.** A grounded, cited account of what practicing chemical and process engineers actually need from a
fault-monitoring layer, how DSFB-Chemical-Engineering maps onto that reality, and — stated honestly — what it does
*not* do. This is the "homework": the domain knowledge, standards, unit operations, and operational realities the
artifact is built against, written so a reviewer sees it on the page rather than inferring it from source.

This is a **prior-art / defensive-publication** disclosure. Nothing here is a product promise or a capability claim
beyond what the sealed artifacts demonstrate. DSFB is **advisory**: it asserts no root cause, closes no loop, and
carries no control or safety-instrumented-function authority.

---

## 1. The day-to-day reality this is built for

A process engineer or board operator on a continuous or batch plant lives with:
- **Alarm overload.** Upsets generate alarm floods; the operator must triage hundreds of activations into a handful
  of actionable events. Alarm-management practice (below) exists precisely because flat alarm lists do not scale.
- **Sensor distrust.** A "high temperature" reading may be a real excursion or a failed/biased transmitter. The
  first question in any incident review is *was the instrument telling the truth?*
- **Forensics after the fact.** When something goes wrong, the engineer must reconstruct *what the data showed, when,
  and why a conclusion was drawn* — for the incident report, the management-of-change file, and sometimes the
  regulator. Reproducibility is not academic; it is the difference between a defensible record and a guess.
- **Conservatism about anything that can act.** Anything wired to a valve or an interlock is a safety and
  availability risk. Monitoring that *only reads* is deployable in weeks; anything that *writes* is a multi-year
  qualification.

DSFB is designed around the last point: it is read-only by construction. Removing it restores the pre-deployment
baseline exactly. That is the property that makes a monitoring layer adoptable at all.

---

## 2. Standards and frameworks the artifact is aligned to (cited)

| Standard / framework | What it governs | How DSFB engages it |
|---|---|---|
| **ISA-18.2** / **IEC 62682** | Management of alarm systems; alarm-flood reduction, rationalization, performance metrics. | The alarm-rationalization artifact compresses raw breach activations into a small set of fused episodes with `lost_evidence = 0` and `recoverable = true` — a rationalization that *preserves* the underlying evidence rather than suppressing it. |
| **NAMUR NE 107** | Self-monitoring & diagnosis of field devices; four status signals: Good, Maintenance required, Out of specification, Failure. | DSFB emits a per-sample, plant-wide NE107 status trace, mapping structural episodes onto the NE107 vocabulary an operator already reads on the DCS. |
| **NAMUR NE 131** | Requirements for field devices in standard process-automation applications. | Informs the "cheap, ubiquitous sensing" target: temperature, pressure, flow, level, current, vibration — not specialty in-line spectroscopy. |
| **IEC 61511** (SIS) | Safety-instrumented systems for the process industries. | A hard boundary, disclosed explicitly: DSFB is **not** an SIS, carries no SIF, and must be independent of any safety layer. It is advisory monitoring, period. |
| **Management of Change (MOC)** | Documented, auditable change control (e.g. OSHA PSM 1910.119 practice). | The evidence-amendment chain anchors corrections to an immutable original evidence hash — an append-only, tamper-evident audit trail in the MOC spirit. |
| **Plant historian / OPC-UA / OSIsoft-PI** | How time-series process data is actually stored and retrieved. | The historian batch-replay path consumes historian-style CSV exports; the realistic integration point is a historian/OPC-UA read connector (disclosed as direction, not built). |

---

## 3. First-principles chemical engineering encoded

- **Conservation laws as witnesses.** Mass and energy balances are closed over fully-metered control volumes
  (three-tank, CSTR, CSTH, quadruple-tank, and real storage-tank volume balances on BATADAL/SWaT). The closure
  residual is the witness: when a conserved quantity *appears* non-conserved, either a boundary flux is unmetered
  (a leak) or an instrument is lying (a spoof/drift).
- **The applicability criterion (stated honestly).** A balance witness fires only with (1) a closed, fully-metered
  control volume **and** (2) a fault that makes a conserved quantity appear non-conserved. Datasets that fail the
  criterion are *correctly rejected* (PRONTO recirculates inside the boundary; UCI-WWTP/BattLeDIM lack consumer
  metering; RP-1043 chiller's refrigerant leak respects energy conservation; HAI's tank outflow is unmetered). This
  is documented in [`balance_witness_criterion.md`](balance_witness_criterion.md).
- **Residence time and unit-ops topology.** A feed→reactor→separator topology with declared residence times lets the
  framework align an upstream onset to a downstream onset *as a candidate* — temporal precedence plus topological
  adjacency, with an explicit non-causal disclaimer. Residence time is the physical reason a downstream effect lags.
- **Regime / batch-phase structure.** Batch and fed-batch processes are non-stationary by design (growth vs
  production phases in penicillin; recipe transitions). Regime-conditioned admissibility envelopes acknowledge that
  "normal" is phase-dependent — a single global envelope mis-flags phase changes as faults.
- **Soft sensing on cheap signals.** The target is deterministic inference of process structure from cheaply-sensed
  variables, not probabilistic estimation requiring specialty instrumentation. The differentiator is determinism and
  auditability, **not** accuracy superiority.

---

## 4. What this gives a working chemical engineer — honestly

**It does give you:**
- A **read-only** layer you can deploy beside existing systems without touching control, with the guarantee that
  removing it restores the prior baseline exactly.
- A **replayable, byte-exact case file** per incident: the residual evidence, which detectors fired and which stayed
  silent (and why), the rejected near-misses, an NE107 status trace, an alarm rationalization, and a sealed
  `evidence_root` you can re-verify on any machine — CPU or GPU, identical bytes.
- **Honest uncertainty:** when the structure does not match a catalogued signature, it returns *unknown with
  preserved evidence* rather than a confident wrong label.
- **Physics-grounded sensor-integrity checks** where a closeable balance exists, validated to fire on real labelled
  manipulations and stay quiet otherwise.
- **Operator-shaped outputs:** a one-page incident report, alarm-flood compression, NE107 status — in the vocabulary
  already on the DCS.

**It does not give you (stated plainly):**
- It does **not** prove physical root cause. Structural motifs suggest *candidate* mechanisms only; confounders are
  not excluded.
- It does **not** close the loop, move a valve, or carry any SIS/SIF authority. It is advisory unless separately
  certified — which it is not.
- It does **not** replace your estimator, controller, historian, alarm system, or safety layer. It sits beside them.
- It does **not** claim higher accuracy or faster detection than established chemometrics. The claim is determinism,
  auditability, and treating noise structure as signal — not a leaderboard win.
- The public benchmarks (TEP, BATADAL, SWaT, BSM1, CSTR, penicillin) are valuable and labelled, but **cannot
  substitute** for validation on your own proprietary plant-historian data with real roles, units, and control logs.

---

## 5. Foresight — where this goes next (disclosed as prior art, not promised as product)

- **Historian / OPC-UA read connectors** so the batch-replay path ingests live historian exports directly.
- **A unit-ops / residence-time topology library** beyond the demonstrator, so propagation candidates are drawn from
  a plant's real P&ID adjacency.
- **A batch-phase / recipe model library** so regime envelopes track ISA-88-style recipe states automatically.
- **An MOC / e-records integration** (append-only amendment chains feeding a change-control system).
- **An embedded `no_std` core** (see [`edge_core_profile.md`](edge_core_profile.md)) for at-the-skid deployment.
- **A molecular / spectroscopic companion corpus** (see [`molecular_corpus_companion.md`](molecular_corpus_companion.md))
  for plants that *do* have in-line analytics.

Each is a disclosure of direction that widens the prior-art surface. None is claimed as a present capability.

---

*Committed deliberately: a visible, cited record of the domain homework is itself part of the prior-art disclosure
and the basis for the practitioner-facing figures (figure group I) and the paper's operator-value subsection.*
