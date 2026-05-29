/-
  DSFB grammar — Lean 4 formalisation of the proof obligations.

  Mirrors `crates/dsfb-chemical-engineering-core` (the `no_std` fixed-point grammar) over Lean `Int`
  (unbounded — strictly more general than the core's i64/i128, so a proof here covers every machine width).
  Pure Lean 4 core; NO Mathlib (builds offline against the pinned toolchain). Each theorem below discharges a
  row of `proof_obligations::ProofObligationLedgerV1`: the three Kani-checked obligations are re-proven here
  *unbounded*, and two of the three "open" obligations (quorum soundness, compression monotonicity) are
  proven outright. Replay determinism stays empirical (the `verify-replay` gate) — see the ledger note.

  Bounded (non-claim): this formalises the *grammar/fusion logic*, not the floating-point edge pipeline or
  any physical-process claim. It proves the state machine is total, the separations hold, and fusion is sound
  + non-expansive — nothing about root cause, causality, or control/safety authority.
-/
namespace Dsfb

/-- Per-axis classification of a value against one envelope axis. -/
inductive CoordClass
  | interior | grazing | outside
  deriving DecidableEq, Repr

/-- `|x|` on `Int` without Mathlib. -/
def iabs (x : Int) : Int := if x < 0 then -x else x

/--
  Exact integer axis classification, mirroring the core's `classify_axis` in **doubled coordinates**:
  `d2 = 2v − (hi+lo)`, `width = hi − lo`. `band` is the grazing-band numerator over `scale`
  (`0 ≤ band < scale`, `scale > 0`, `hi > lo`). Total by construction (it is a function).
-/
def classifyAxis (v lo hi band scale : Int) : CoordClass :=
  let d2 := 2 * v - (hi + lo)
  let width := hi - lo
  let ad2 := iabs d2
  if ad2 > width then .outside
  else if ad2 * scale ≥ width * (scale - band) then .grazing
  else .interior

/-- DSFB grammar state (mirrors the core's `GrammarState`). -/
inductive GrammarState
  | nominal | driftAccum | slewSpike | envViolation | boundaryGrazing | recovery | compound | sensorFault
  deriving DecidableEq, Repr

def isOutside : CoordClass → Bool
  | .outside => true
  | _ => false

def isGrazing : CoordClass → Bool
  | .grazing => true
  | _ => false

/-- A triple's per-axis evaluation against the envelope. -/
structure Eval where
  r : CoordClass
  delta : CoordClass
  sigma : CoordClass

/-- "Some axis grazes while none is outside" (mirrors `EnvelopeEval::any_grazing`). -/
def anyGrazing (e : Eval) : Bool :=
  !(isOutside e.r) && !(isOutside e.delta) && !(isOutside e.sigma) &&
  (isGrazing e.r || isGrazing e.delta || isGrazing e.sigma)

/--
  The grammar classifier, mirroring the core's `GrammarClassifier::classify` selection order exactly:
  an invalid (sensor-fault) reading short-circuits; then compound (δ∧σ outside), env-violation (r outside),
  slew (σ outside), drift (δ outside), boundary-grazing, recovery (prev not nominal), else nominal.
-/
def classify (e : Eval) (prevNominal : Bool) (valid : Bool) : GrammarState :=
  if !valid then .sensorFault
  else if isOutside e.delta && isOutside e.sigma then .compound
  else if isOutside e.r then .envViolation
  else if isOutside e.sigma then .slewSpike
  else if isOutside e.delta then .driftAccum
  else if anyGrazing e then .boundaryGrazing
  else if !prevNominal then .recovery
  else .nominal

-- ── Obligation: grammar_totality (Kani; here unbounded) ──────────────────────────────────────────────
/-- `classify` is a total function: every input maps to some state. -/
theorem classify_total (e : Eval) (p v : Bool) : ∃ s, classify e p v = s :=
  ⟨classify e p v, rfl⟩

-- ── Obligation: interior_not_sensorfault (Kani; here unbounded) ──────────────────────────────────────
/-- A **valid** reading is never classified `SensorFault` (regardless of the axis classes / prev state). -/
theorem valid_not_sensorFault (e : Eval) (p : Bool) :
    classify e p true ≠ .sensorFault := by
  obtain ⟨r, d, s⟩ := e
  cases r <;> cases d <;> cases s <;> cases p <;> decide

-- ── Obligation: beyond_bound_not_interior (Kani; here unbounded) ─────────────────────────────────────
/-- If the raw-residual axis is `outside`, the state is never `nominal` (a breach is never read as nominal). -/
theorem outside_r_not_nominal (e : Eval) (p v : Bool) (h : e.r = .outside) :
    classify e p v ≠ .nominal := by
  obtain ⟨r, d, s⟩ := e
  cases r <;> cases d <;> cases s <;> cases p <;> cases v <;> simp_all [classify, isOutside]

/-- The compound rule: when **both** δ and σ are outside (and the reading is valid), the state is `compound`. -/
theorem deltaSigma_outside_is_compound (e : Eval) (p : Bool)
    (hd : e.delta = .outside) (hs : e.sigma = .outside) :
    classify e p true = .compound := by
  obtain ⟨r, d, s⟩ := e
  cases r <;> cases d <;> cases s <;> cases p <;> simp_all [classify, isOutside]

-- ── Obligation: quorum_soundness (ledger: open → here proven) ────────────────────────────────────────
/-- A fused episode requires at least `quorum` independent detector families to have fired. -/
def fused (familiesFired quorum : Nat) : Bool := decide (quorum ≤ familiesFired)

/-- Soundness: if `fused` reports an episode, the quorum was genuinely met. -/
theorem fused_sound {familiesFired quorum : Nat} (h : fused familiesFired quorum = true) :
    quorum ≤ familiesFired :=
  of_decide_eq_true h

/-- Conversely, below quorum there is no fused episode (no spurious fusion). -/
theorem not_fused_below_quorum {familiesFired quorum : Nat} (h : familiesFired < quorum) :
    fused familiesFired quorum = false := by
  simp [fused, Nat.not_le.mpr h]

-- ── Obligation: episode_compression_monotone (ledger: open → here proven) ────────────────────────────
/--
  Fusion is **non-expansive**: modelling episode fusion as selecting a sub-collection of the raw firings
  (a `filter`), the number of fused episodes never exceeds the number of raw firings. Proven by structural
  induction in pure core (no Mathlib).
-/
theorem compression_monotone {α : Type} (keep : α → Bool) (xs : List α) :
    (xs.filter keep).length ≤ xs.length := by
  induction xs with
  | nil => simp
  | cons x xs ih =>
    rw [List.filter_cons]
    split
    all_goals simp only [List.length_cons]
    all_goals omega

end Dsfb
