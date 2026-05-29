(*
  DSFB grammar — Coq / Rocq formalisation of the proof obligations.

  A second, independent machine-checked formalisation alongside the Lean 4 development in
  `formal/lean/DsfbGrammar.lean`, mirroring `crates/dsfb-chemical-engineering-core` (the no_std fixed-point
  grammar). The integer `classifyAxis` is formalised over `Z` (unbounded — strictly more general than the
  core's i64/i128). Verified with the Rocq Prover 9.1.1 (`coqc DsfbGrammar.v`); needs `rocq-stdlib`
  (`List` / `ZArith` / `Lia`).

  Each theorem discharges a row of `proof_obligations::ProofObligationLedgerV1`, cross-checking the Lean 4
  results in a second prover kernel.

  Bounded (non-claim): formalises the grammar/fusion logic, not the floating-point edge pipeline or any
  physical-process claim. Replay determinism stays empirical (the verify-replay gate).
*)

From Stdlib Require Import List ZArith Lia.
Import ListNotations.
(* ZArith opens Z_scope; close it so the `nat` theorems below read `<=` as `Nat.le`. classifyAxis annotates
   its arithmetic with %Z explicitly, so it is unaffected by the ambient scope. *)
Close Scope Z_scope.

(* ── Per-axis classification ───────────────────────────────────────────────────────────────────── *)
Inductive CoordClass := interior | grazing | outside.

(* Exact integer axis classification (doubled coordinates), mirroring the core's `classify_axis` over Z. *)
Definition classifyAxis (v lo hi band scale : Z) : CoordClass :=
  let d2 := (2 * v - (hi + lo))%Z in
  let width := (hi - lo)%Z in
  let ad2 := (if Z.ltb d2 0 then - d2 else d2)%Z in
  if Z.gtb ad2 width then outside
  else if Z.geb (ad2 * scale)%Z (width * (scale - band))%Z then grazing
  else interior.

(* ── Grammar state machine ─────────────────────────────────────────────────────────────────────── *)
Inductive GrammarState :=
  | nominal | driftAccum | slewSpike | envViolation
  | boundaryGrazing | recovery | compound | sensorFault.

Definition isOutside (c : CoordClass) : bool :=
  match c with outside => true | _ => false end.

Definition isGrazing (c : CoordClass) : bool :=
  match c with grazing => true | _ => false end.

Record Eval := { r : CoordClass; delta : CoordClass; sigma : CoordClass }.

Definition anyGrazing (e : Eval) : bool :=
  andb (andb (andb (negb (isOutside (r e))) (negb (isOutside (delta e))))
             (negb (isOutside (sigma e))))
       (orb (orb (isGrazing (r e)) (isGrazing (delta e))) (isGrazing (sigma e))).

(* Mirrors the core's `GrammarClassifier::classify` selection order exactly. *)
Definition classify (e : Eval) (prevNominal valid : bool) : GrammarState :=
  if negb valid then sensorFault
  else if andb (isOutside (delta e)) (isOutside (sigma e)) then compound
  else if isOutside (r e) then envViolation
  else if isOutside (sigma e) then slewSpike
  else if isOutside (delta e) then driftAccum
  else if anyGrazing e then boundaryGrazing
  else if negb prevNominal then recovery
  else nominal.

(* ── Obligation: grammar_totality ──────────────────────────────────────────────────────────────── *)
Theorem classify_total : forall e p v, exists s, classify e p v = s.
Proof. intros. exists (classify e p v). reflexivity. Qed.

(* ── Obligation: interior_not_sensorfault (a valid reading is never SensorFault) ────────────────── *)
Theorem valid_not_sensorFault : forall e p, classify e p true <> sensorFault.
Proof. intros [rr dd ss] p. destruct rr, dd, ss, p; simpl; discriminate. Qed.

(* ── Obligation: beyond_bound_not_interior (out-of-bound r is never nominal) ────────────────────── *)
Theorem outside_r_not_nominal : forall e p v, r e = outside -> classify e p v <> nominal.
Proof. intros [rr dd ss] p v H. simpl in H. subst rr. destruct dd, ss, p, v; simpl; discriminate. Qed.

(* The compound rule: both δ and σ outside (and valid) ⇒ compound. *)
Theorem deltaSigma_outside_is_compound :
  forall e p, delta e = outside -> sigma e = outside -> classify e p true = compound.
Proof. intros [rr dd ss] p Hd Hs. simpl in *. subst dd ss. destruct rr, p; reflexivity. Qed.

(* ── Obligation: quorum_soundness ──────────────────────────────────────────────────────────────── *)
Definition fused (familiesFired quorum : nat) : bool := Nat.leb quorum familiesFired.

Theorem fused_sound : forall f q, fused f q = true -> q <= f.
Proof. intros f q H. apply Nat.leb_le. exact H. Qed.

Theorem not_fused_below_quorum : forall f q, f < q -> fused f q = false.
Proof. intros f q H. unfold fused. apply Nat.leb_gt. exact H. Qed.

(* ── Obligation: episode_compression_monotone ──────────────────────────────────────────────────── *)
Theorem compression_monotone :
  forall (A : Type) (keep : A -> bool) (xs : list A),
    length (filter keep xs) <= length xs.
Proof.
  intros A keep xs. induction xs as [| x xs IH]; simpl.
  - lia.
  - destruct (keep x); simpl; lia.
Qed.
