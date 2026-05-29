module Dsfb_chemical_engineering_core
#set-options "--fuel 0 --ifuel 1 --z3rlimit 15"
open FStar.Mul
open Core_models

/// Fixed-point scale: every engineering value is represented as `round(value × SCALE)` in an `i64`. Chosen
/// to match the CUDA evidence kernel's `SCALE = 1e6` determinism convention so the *numbers* are comparable
/// across the stack even though the embedded core is not claimed bit-identical to the float pipeline.
let v_SCALE: i64 = mk_i64 1000000

/// A fixed-point residual triple: raw residual `r`, drift `δ`, slew `σ`, each scaled by [`SCALE`].
type t_FixedTriple = {
  f_r:i64;
  f_delta:i64;
  f_sigma:i64
}

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_8': Core_models.Fmt.t_Debug t_FixedTriple

unfold
let impl_8 = impl_8'

let impl_9: Core_models.Clone.t_Clone t_FixedTriple =
  { f_clone = (fun x -> x); f_clone_pre = (fun _ -> True); f_clone_post = (fun _ _ -> True) }

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_10': Core_models.Marker.t_Copy t_FixedTriple

unfold
let impl_10 = impl_10'

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_11': Core_models.Marker.t_StructuralPartialEq t_FixedTriple

unfold
let impl_11 = impl_11'

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_12': Core_models.Cmp.t_PartialEq t_FixedTriple t_FixedTriple

unfold
let impl_12 = impl_12'

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_13': Core_models.Cmp.t_Eq t_FixedTriple

unfold
let impl_13 = impl_13'

/// Per-axis classification of a value against an envelope axis.
type t_CoordClass =
  | CoordClass_Interior : t_CoordClass
  | CoordClass_Grazing : t_CoordClass
  | CoordClass_Outside : t_CoordClass

let t_CoordClass_cast_to_repr (x: t_CoordClass) : isize =
  match x <: t_CoordClass with
  | CoordClass_Interior  -> mk_isize 0
  | CoordClass_Grazing  -> mk_isize 1
  | CoordClass_Outside  -> mk_isize 2

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_14': Core_models.Fmt.t_Debug t_CoordClass

unfold
let impl_14 = impl_14'

let impl_15: Core_models.Clone.t_Clone t_CoordClass =
  { f_clone = (fun x -> x); f_clone_pre = (fun _ -> True); f_clone_post = (fun _ _ -> True) }

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_16': Core_models.Marker.t_Copy t_CoordClass

unfold
let impl_16 = impl_16'

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_17': Core_models.Marker.t_StructuralPartialEq t_CoordClass

unfold
let impl_17 = impl_17'

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_18': Core_models.Cmp.t_PartialEq t_CoordClass t_CoordClass

unfold
let impl_18 = impl_18'

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_19': Core_models.Cmp.t_Eq t_CoordClass

unfold
let impl_19 = impl_19'

/// Classify a scaled value `v` against the scaled axis bounds `[lo, hi]` with a grazing band `band_scaled`
/// (a fraction of the half-width, itself scaled by [`SCALE`], so `band_scaled ∈ [0, SCALE)`).
/// Exact integer logic, no division: working in doubled coordinates `d2 = 2v − (hi+lo)` and `width = hi − lo`,
/// - `Outside`  ⇔ `|d2| > width`
/// - `Grazing`  ⇔ `|d2|·SCALE ≥ width·(SCALE − band_scaled)` (promoted to `i128` so the product never overflows)
/// - `Interior` otherwise.
let classify_axis (v lo hi band_scaled: i64) : t_CoordClass =
  let d2:i128 =
    (mk_i128 2 *! (cast (v <: i64) <: i128) <: i128) -!
    ((cast (hi <: i64) <: i128) +! (cast (lo <: i64) <: i128) <: i128)
  in
  let width:i128 = (cast (hi <: i64) <: i128) -! (cast (lo <: i64) <: i128) in
  let ad2:i128 = Core_models.Num.impl_i128__abs d2 in
  if ad2 >. width
  then CoordClass_Outside <: t_CoordClass
  else
    if
      (ad2 *! (cast (v_SCALE <: i64) <: i128) <: i128) >=.
      (width *! ((cast (v_SCALE <: i64) <: i128) -! (cast (band_scaled <: i64) <: i128) <: i128)
        <:
        i128)
    then CoordClass_Grazing <: t_CoordClass
    else CoordClass_Interior <: t_CoordClass

/// A fixed-point admissibility envelope: scaled bounds on each axis plus a scaled grazing-band fraction.
type t_FixedEnvelope = {
  f_r_min:i64;
  f_r_max:i64;
  f_delta_min:i64;
  f_delta_max:i64;
  f_sigma_min:i64;
  f_sigma_max:i64;
  f_band_scaled:i64
}

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_20': Core_models.Fmt.t_Debug t_FixedEnvelope

unfold
let impl_20 = impl_20'

let impl_21: Core_models.Clone.t_Clone t_FixedEnvelope =
  { f_clone = (fun x -> x); f_clone_pre = (fun _ -> True); f_clone_post = (fun _ _ -> True) }

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_22': Core_models.Marker.t_Copy t_FixedEnvelope

unfold
let impl_22 = impl_22'

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_23': Core_models.Marker.t_StructuralPartialEq t_FixedEnvelope

unfold
let impl_23 = impl_23'

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_24': Core_models.Cmp.t_PartialEq t_FixedEnvelope t_FixedEnvelope

unfold
let impl_24 = impl_24'

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_25': Core_models.Cmp.t_Eq t_FixedEnvelope

unfold
let impl_25 = impl_25'

/// Symmetric envelope for a zero-centred residual of scale `k` (scaled), mirroring the edge crate's
/// ratios (r: ±k, δ: ±0.6k, σ: ±2k): drift is time-averaged (tighter), slew is a first difference (wider).
/// These ratios are heuristic, not physics-derived — calibrate per channel for engineering-unit residuals.
let impl_FixedEnvelope__symmetric (k_scaled band_scaled: i64) : t_FixedEnvelope =
  {
    f_r_min = Rust_primitives.Arithmetic.neg k_scaled;
    f_r_max = k_scaled;
    f_delta_min = Rust_primitives.Arithmetic.neg ((k_scaled *! mk_i64 3 <: i64) /! mk_i64 5 <: i64);
    f_delta_max = (k_scaled *! mk_i64 3 <: i64) /! mk_i64 5;
    f_sigma_min = Rust_primitives.Arithmetic.neg (k_scaled *! mk_i64 2 <: i64);
    f_sigma_max = k_scaled *! mk_i64 2;
    f_band_scaled = band_scaled
  }
  <:
  t_FixedEnvelope

/// Per-axis classification result of evaluating a triple against an envelope.
type t_EnvelopeEval = {
  f_r_class:t_CoordClass;
  f_delta_class:t_CoordClass;
  f_sigma_class:t_CoordClass
}

/// Classify a triple against the envelope.
let impl_FixedEnvelope__eval (self: t_FixedEnvelope) (t: t_FixedTriple) : t_EnvelopeEval =
  {
    f_r_class = classify_axis t.f_r self.f_r_min self.f_r_max self.f_band_scaled;
    f_delta_class = classify_axis t.f_delta self.f_delta_min self.f_delta_max self.f_band_scaled;
    f_sigma_class = classify_axis t.f_sigma self.f_sigma_min self.f_sigma_max self.f_band_scaled
  }
  <:
  t_EnvelopeEval

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_26': Core_models.Fmt.t_Debug t_EnvelopeEval

unfold
let impl_26 = impl_26'

let impl_27: Core_models.Clone.t_Clone t_EnvelopeEval =
  { f_clone = (fun x -> x); f_clone_pre = (fun _ -> True); f_clone_post = (fun _ _ -> True) }

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_28': Core_models.Marker.t_Copy t_EnvelopeEval

unfold
let impl_28 = impl_28'

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_29': Core_models.Marker.t_StructuralPartialEq t_EnvelopeEval

unfold
let impl_29 = impl_29'

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_30': Core_models.Cmp.t_PartialEq t_EnvelopeEval t_EnvelopeEval

unfold
let impl_30 = impl_30'

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_31': Core_models.Cmp.t_Eq t_EnvelopeEval

unfold
let impl_31 = impl_31'

let impl_EnvelopeEval__r_violated (self: t_EnvelopeEval) : bool =
  self.f_r_class =. (CoordClass_Outside <: t_CoordClass)

let impl_EnvelopeEval__delta_violated (self: t_EnvelopeEval) : bool =
  self.f_delta_class =. (CoordClass_Outside <: t_CoordClass)

let impl_EnvelopeEval__sigma_violated (self: t_EnvelopeEval) : bool =
  self.f_sigma_class =. (CoordClass_Outside <: t_CoordClass)

/// No axis is `Outside` but at least one is `Grazing` (mirrors the edge `any_grazing` guard).
let impl_EnvelopeEval__any_grazing (self: t_EnvelopeEval) : bool =
  ~.(impl_EnvelopeEval__r_violated self <: bool) &&
  ~.(impl_EnvelopeEval__delta_violated self <: bool) &&
  ~.(impl_EnvelopeEval__sigma_violated self <: bool) &&
  (self.f_r_class =. (CoordClass_Grazing <: t_CoordClass) ||
  self.f_delta_class =. (CoordClass_Grazing <: t_CoordClass) ||
  self.f_sigma_class =. (CoordClass_Grazing <: t_CoordClass))

/// DSFB grammar state (mirrors the edge crate's `GrammarState`). One token is emitted per sample; the
/// selection order in [`GrammarClassifier::classify`] is what disambiguates overlapping breaches.
type t_GrammarState =
  | GrammarState_Nominal : t_GrammarState
  | GrammarState_DriftAccum : t_GrammarState
  | GrammarState_SlewSpike : t_GrammarState
  | GrammarState_EnvViolation : t_GrammarState
  | GrammarState_BoundaryGrazing : t_GrammarState
  | GrammarState_Recovery : t_GrammarState
  | GrammarState_Compound : t_GrammarState
  | GrammarState_SensorFault : t_GrammarState

let t_GrammarState_cast_to_repr (x: t_GrammarState) : isize =
  match x <: t_GrammarState with
  | GrammarState_Nominal  -> mk_isize 0
  | GrammarState_DriftAccum  -> mk_isize 1
  | GrammarState_SlewSpike  -> mk_isize 2
  | GrammarState_EnvViolation  -> mk_isize 3
  | GrammarState_BoundaryGrazing  -> mk_isize 4
  | GrammarState_Recovery  -> mk_isize 5
  | GrammarState_Compound  -> mk_isize 6
  | GrammarState_SensorFault  -> mk_isize 7

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_32': Core_models.Fmt.t_Debug t_GrammarState

unfold
let impl_32 = impl_32'

let impl_33: Core_models.Clone.t_Clone t_GrammarState =
  { f_clone = (fun x -> x); f_clone_pre = (fun _ -> True); f_clone_post = (fun _ _ -> True) }

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_34': Core_models.Marker.t_Copy t_GrammarState

unfold
let impl_34 = impl_34'

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_35': Core_models.Marker.t_StructuralPartialEq t_GrammarState

unfold
let impl_35 = impl_35'

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_36': Core_models.Cmp.t_PartialEq t_GrammarState t_GrammarState

unfold
let impl_36 = impl_36'

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_37': Core_models.Cmp.t_Eq t_GrammarState

unfold
let impl_37 = impl_37'

/// Stable 2-letter token (identical to the edge crate's tokens).
let impl_GrammarState__token (self: t_GrammarState) : string =
  match self <: t_GrammarState with
  | GrammarState_Nominal  -> "NOM"
  | GrammarState_DriftAccum  -> "DA"
  | GrammarState_SlewSpike  -> "SS"
  | GrammarState_EnvViolation  -> "EV"
  | GrammarState_BoundaryGrazing  -> "BG"
  | GrammarState_Recovery  -> "RC"
  | GrammarState_Compound  -> "CP"
  | GrammarState_SensorFault  -> "SF"

let impl_GrammarState__is_nominal (self: t_GrammarState) : bool =
  match self <: t_GrammarState with
  | GrammarState_Nominal  -> true
  | _ -> false

/// Reason code attached to a classification (a subset of the edge crate's, carrying drift/slew *direction*
/// so a downstream reader can tell an upward drift from a downward one without re-deriving it from the triple).
type t_ReasonCode =
  | ReasonCode_Nominal : t_ReasonCode
  | ReasonCode_DriftPositive : t_ReasonCode
  | ReasonCode_DriftNegative : t_ReasonCode
  | ReasonCode_SlewPositive : t_ReasonCode
  | ReasonCode_SlewNegative : t_ReasonCode
  | ReasonCode_Violation : t_ReasonCode
  | ReasonCode_Grazing : t_ReasonCode
  | ReasonCode_Recovery : t_ReasonCode
  | ReasonCode_Compound : t_ReasonCode
  | ReasonCode_OobSensor : t_ReasonCode

let t_ReasonCode_cast_to_repr (x: t_ReasonCode) : isize =
  match x <: t_ReasonCode with
  | ReasonCode_Nominal  -> mk_isize 0
  | ReasonCode_DriftPositive  -> mk_isize 1
  | ReasonCode_DriftNegative  -> mk_isize 2
  | ReasonCode_SlewPositive  -> mk_isize 3
  | ReasonCode_SlewNegative  -> mk_isize 4
  | ReasonCode_Violation  -> mk_isize 5
  | ReasonCode_Grazing  -> mk_isize 6
  | ReasonCode_Recovery  -> mk_isize 7
  | ReasonCode_Compound  -> mk_isize 8
  | ReasonCode_OobSensor  -> mk_isize 9

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_38': Core_models.Fmt.t_Debug t_ReasonCode

unfold
let impl_38 = impl_38'

let impl_39: Core_models.Clone.t_Clone t_ReasonCode =
  { f_clone = (fun x -> x); f_clone_pre = (fun _ -> True); f_clone_post = (fun _ _ -> True) }

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_40': Core_models.Marker.t_Copy t_ReasonCode

unfold
let impl_40 = impl_40'

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_41': Core_models.Marker.t_StructuralPartialEq t_ReasonCode

unfold
let impl_41 = impl_41'

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_42': Core_models.Cmp.t_PartialEq t_ReasonCode t_ReasonCode

unfold
let impl_42 = impl_42'

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_43': Core_models.Cmp.t_Eq t_ReasonCode

unfold
let impl_43 = impl_43'

/// One-step-memory grammar classifier (mirrors the edge `GrammarClassifier`).
type t_GrammarClassifier = { f_prev_state:t_GrammarState }

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_44': Core_models.Fmt.t_Debug t_GrammarClassifier

unfold
let impl_44 = impl_44'

let impl_45: Core_models.Clone.t_Clone t_GrammarClassifier =
  { f_clone = (fun x -> x); f_clone_pre = (fun _ -> True); f_clone_post = (fun _ _ -> True) }

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_46': Core_models.Marker.t_Copy t_GrammarClassifier

unfold
let impl_46 = impl_46'

[@@ FStar.Tactics.Typeclasses.tcinstance]
let impl_3: Core_models.Default.t_Default t_GrammarClassifier =
  {
    f_default_pre = (fun (_: Prims.unit) -> true);
    f_default_post = (fun (_: Prims.unit) (out: t_GrammarClassifier) -> true);
    f_default
    =
    fun (_: Prims.unit) ->
      { f_prev_state = GrammarState_Nominal <: t_GrammarState } <: t_GrammarClassifier
  }

let impl_GrammarClassifier__new (_: Prims.unit) : t_GrammarClassifier =
  Core_models.Default.f_default #t_GrammarClassifier #FStar.Tactics.Typeclasses.solve ()

/// Classify an evaluated triple. `valid = false` signals a bad sensor reading (the fixed-point analogue
/// of a non-finite float): emit `SensorFault` without touching the one-step memory. The state-selection
/// order is identical to the edge crate's.
let impl_GrammarClassifier__classify
      (self: t_GrammarClassifier)
      (eval: t_EnvelopeEval)
      (t: t_FixedTriple)
      (valid: bool)
    : (t_GrammarClassifier & (t_GrammarState & t_ReasonCode)) =
  if ~.valid
  then
    self,
    ((GrammarState_SensorFault <: t_GrammarState), (ReasonCode_OobSensor <: t_ReasonCode)
      <:
      (t_GrammarState & t_ReasonCode))
    <:
    (t_GrammarClassifier & (t_GrammarState & t_ReasonCode))
  else
    let new_state:t_GrammarState =
      if impl_EnvelopeEval__delta_violated eval && impl_EnvelopeEval__sigma_violated eval
      then GrammarState_Compound <: t_GrammarState
      else
        if impl_EnvelopeEval__r_violated eval
        then GrammarState_EnvViolation <: t_GrammarState
        else
          if impl_EnvelopeEval__sigma_violated eval
          then GrammarState_SlewSpike <: t_GrammarState
          else
            if impl_EnvelopeEval__delta_violated eval
            then GrammarState_DriftAccum <: t_GrammarState
            else
              if impl_EnvelopeEval__any_grazing eval
              then GrammarState_BoundaryGrazing <: t_GrammarState
              else
                if self.f_prev_state <>. (GrammarState_Nominal <: t_GrammarState)
                then GrammarState_Recovery <: t_GrammarState
                else GrammarState_Nominal <: t_GrammarState
    in
    let reason:t_ReasonCode =
      match new_state <: t_GrammarState with
      | GrammarState_Nominal  -> ReasonCode_Nominal <: t_ReasonCode
      | GrammarState_DriftAccum  ->
        if t.f_delta >=. mk_i64 0
        then ReasonCode_DriftPositive <: t_ReasonCode
        else ReasonCode_DriftNegative <: t_ReasonCode
      | GrammarState_SlewSpike  ->
        if t.f_sigma >=. mk_i64 0
        then ReasonCode_SlewPositive <: t_ReasonCode
        else ReasonCode_SlewNegative <: t_ReasonCode
      | GrammarState_EnvViolation  -> ReasonCode_Violation <: t_ReasonCode
      | GrammarState_BoundaryGrazing  -> ReasonCode_Grazing <: t_ReasonCode
      | GrammarState_Recovery  -> ReasonCode_Recovery <: t_ReasonCode
      | GrammarState_Compound  -> ReasonCode_Compound <: t_ReasonCode
      | GrammarState_SensorFault  -> ReasonCode_OobSensor <: t_ReasonCode
    in
    let self:t_GrammarClassifier =
      {
        self with
        f_prev_state
        =
        if new_state =. (GrammarState_Recovery <: t_GrammarState)
        then GrammarState_Nominal <: t_GrammarState
        else new_state
      }
      <:
      t_GrammarClassifier
    in
    let hax_temp_output:(t_GrammarState & t_ReasonCode) =
      new_state, reason <: (t_GrammarState & t_ReasonCode)
    in
    self, hax_temp_output <: (t_GrammarClassifier & (t_GrammarState & t_ReasonCode))

let impl_GrammarClassifier__reset (self: t_GrammarClassifier) : t_GrammarClassifier =
  let self:t_GrammarClassifier =
    { self with f_prev_state = GrammarState_Nominal <: t_GrammarState } <: t_GrammarClassifier
  in
  self

/// A fixed-capacity ring buffer of scaled residuals — the only "memory" the core keeps, with statically
/// known size `N` (no heap). Tracks a running sum so the windowed mean is O(1).
type t_RingBuffer (v_N: usize) = {
  f_buf:t_Array i64 v_N;
  f_len:usize;
  f_head:usize;
  f_sum:i64
}

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_47': v_N: usize -> Core_models.Fmt.t_Debug (t_RingBuffer v_N)

unfold
let impl_47 (v_N: usize) = impl_47' v_N

let impl_48 (v_N: usize) : Core_models.Clone.t_Clone (t_RingBuffer v_N) =
  { f_clone = (fun x -> x); f_clone_pre = (fun _ -> True); f_clone_post = (fun _ _ -> True) }

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_49': v_N: usize -> Core_models.Marker.t_Copy (t_RingBuffer v_N)

unfold
let impl_49 (v_N: usize) = impl_49' v_N

[@@ FStar.Tactics.Typeclasses.tcinstance]
let impl_5 (v_N: usize) : Core_models.Default.t_Default (t_RingBuffer v_N) =
  {
    f_default_pre = (fun (_: Prims.unit) -> true);
    f_default_post = (fun (_: Prims.unit) (out: t_RingBuffer v_N) -> true);
    f_default
    =
    fun (_: Prims.unit) ->
      {
        f_buf = Rust_primitives.Hax.repeat (mk_i64 0) v_N;
        f_len = mk_usize 0;
        f_head = mk_usize 0;
        f_sum = mk_i64 0
      }
      <:
      t_RingBuffer v_N
  }

let impl_6__new (v_N: usize) (_: Prims.unit) : t_RingBuffer v_N =
  Core_models.Default.f_default #(t_RingBuffer v_N) #FStar.Tactics.Typeclasses.solve ()

/// Push a value, evicting the oldest once full. Maintains the running `sum`.
let impl_6__push (v_N: usize) (self: t_RingBuffer v_N) (v: i64) : t_RingBuffer v_N =
  let self:t_RingBuffer v_N =
    if self.f_len =. v_N
    then
      let self:t_RingBuffer v_N =
        { self with f_sum = self.f_sum -! (self.f_buf.[ self.f_head ] <: i64) } <: t_RingBuffer v_N
      in
      self
    else
      let self:t_RingBuffer v_N =
        { self with f_len = self.f_len +! mk_usize 1 } <: t_RingBuffer v_N
      in
      self
  in
  let self:t_RingBuffer v_N =
    {
      self with
      f_buf = Rust_primitives.Hax.Monomorphized_update_at.update_at_usize self.f_buf self.f_head v
    }
    <:
    t_RingBuffer v_N
  in
  let self:t_RingBuffer v_N = { self with f_sum = self.f_sum +! v } <: t_RingBuffer v_N in
  let self:t_RingBuffer v_N =
    {
      self with
      f_head
      =
      (self.f_head +! mk_usize 1 <: usize) %!
      (Core_models.Cmp.f_max #usize #FStar.Tactics.Typeclasses.solve v_N (mk_usize 1) <: usize)
    }
    <:
    t_RingBuffer v_N
  in
  self

/// Windowed mean (scaled), via integer division of the running sum by the current length. 0 when empty.
let impl_6__mean (v_N: usize) (self: t_RingBuffer v_N) : i64 =
  if self.f_len =. mk_usize 0 then mk_i64 0 else self.f_sum /! (cast (self.f_len <: usize) <: i64)

let impl_6__len (v_N: usize) (self: t_RingBuffer v_N) : usize = self.f_len

let impl_6__is_empty (v_N: usize) (self: t_RingBuffer v_N) : bool = self.f_len =. mk_usize 0

/// The full single-channel embedded engine: ring buffer (drift) + previous sample (slew) + envelope +
/// classifier, all stack-allocated with statically-known size `N`. One instance per monitored channel.
type t_DsfbCore (v_N: usize) = {
  f_env:t_FixedEnvelope;
  f_ring:t_RingBuffer v_N;
  f_prev_r:Core_models.Option.t_Option i64;
  f_classifier:t_GrammarClassifier
}

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_50': v_N: usize -> Core_models.Fmt.t_Debug (t_DsfbCore v_N)

unfold
let impl_50 (v_N: usize) = impl_50' v_N

let impl_51 (v_N: usize) : Core_models.Clone.t_Clone (t_DsfbCore v_N) =
  { f_clone = (fun x -> x); f_clone_pre = (fun _ -> True); f_clone_post = (fun _ _ -> True) }

[@@ FStar.Tactics.Typeclasses.tcinstance]
assume
val impl_52': v_N: usize -> Core_models.Marker.t_Copy (t_DsfbCore v_N)

unfold
let impl_52 (v_N: usize) = impl_52' v_N

/// Create a channel engine with the given fixed-point envelope and a window of `N` samples.
let impl_7__new (v_N: usize) (env: t_FixedEnvelope) : t_DsfbCore v_N =
  {
    f_env = env;
    f_ring = impl_6__new v_N ();
    f_prev_r = Core_models.Option.Option_None <: Core_models.Option.t_Option i64;
    f_classifier = impl_GrammarClassifier__new ()
  }
  <:
  t_DsfbCore v_N

/// Process one scaled residual sample and return its grammar state + reason. `valid = false` signals a
/// bad reading (the fixed-point analogue of a non-finite float): emits `SensorFault` and leaves the ring,
/// the previous sample, and the classifier memory untouched. Otherwise:
/// - the drift δ is the windowed mean of `r` (sustained accumulation),
/// - the slew σ is `r − previous r` (rapid transient; 0 on the first sample).
let impl_7__step (v_N: usize) (self: t_DsfbCore v_N) (r_scaled: i64) (valid: bool)
    : (t_DsfbCore v_N & (t_GrammarState & t_ReasonCode)) =
  if ~.valid
  then
    let dummy:t_FixedTriple =
      { f_r = mk_i64 0; f_delta = mk_i64 0; f_sigma = mk_i64 0 } <: t_FixedTriple
    in
    let bad:t_EnvelopeEval =
      {
        f_r_class = CoordClass_Interior <: t_CoordClass;
        f_delta_class = CoordClass_Interior <: t_CoordClass;
        f_sigma_class = CoordClass_Interior <: t_CoordClass
      }
      <:
      t_EnvelopeEval
    in
    let (tmp0: t_GrammarClassifier), (out: (t_GrammarState & t_ReasonCode)) =
      impl_GrammarClassifier__classify self.f_classifier bad dummy false
    in
    let self:t_DsfbCore v_N = { self with f_classifier = tmp0 } <: t_DsfbCore v_N in
    self, out <: (t_DsfbCore v_N & (t_GrammarState & t_ReasonCode))
  else
    let self:t_DsfbCore v_N =
      { self with f_ring = impl_6__push v_N self.f_ring r_scaled } <: t_DsfbCore v_N
    in
    let delta:i64 = impl_6__mean v_N self.f_ring in
    let sigma:i64 =
      match self.f_prev_r <: Core_models.Option.t_Option i64 with
      | Core_models.Option.Option_Some p -> r_scaled -! p
      | Core_models.Option.Option_None  -> mk_i64 0
    in
    let self:t_DsfbCore v_N =
      {
        self with
        f_prev_r = Core_models.Option.Option_Some r_scaled <: Core_models.Option.t_Option i64
      }
      <:
      t_DsfbCore v_N
    in
    let t:t_FixedTriple = { f_r = r_scaled; f_delta = delta; f_sigma = sigma } <: t_FixedTriple in
    let eval:t_EnvelopeEval = impl_FixedEnvelope__eval self.f_env t in
    let (tmp0: t_GrammarClassifier), (out: (t_GrammarState & t_ReasonCode)) =
      impl_GrammarClassifier__classify self.f_classifier eval t true
    in
    let self:t_DsfbCore v_N = { self with f_classifier = tmp0 } <: t_DsfbCore v_N in
    let hax_temp_output:(t_GrammarState & t_ReasonCode) = out in
    self, hax_temp_output <: (t_DsfbCore v_N & (t_GrammarState & t_ReasonCode))

/// Reset to the initial state (empty window, no previous sample, Nominal memory) for a fresh replay.
let impl_7__reset (v_N: usize) (self: t_DsfbCore v_N) : t_DsfbCore v_N =
  let self:t_DsfbCore v_N = { self with f_ring = impl_6__new v_N () } <: t_DsfbCore v_N in
  let self:t_DsfbCore v_N =
    { self with f_prev_r = Core_models.Option.Option_None <: Core_models.Option.t_Option i64 }
    <:
    t_DsfbCore v_N
  in
  let self:t_DsfbCore v_N =
    { self with f_classifier = impl_GrammarClassifier__reset self.f_classifier } <: t_DsfbCore v_N
  in
  self
