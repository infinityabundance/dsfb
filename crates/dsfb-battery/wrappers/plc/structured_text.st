(*
    Deterministic Structured Text translation of the current DSFB grammar-state
    logic in src/detection.rs::evaluate_grammar_state.

    This wrapper is advisory only. It does not implement actuation, protection,
    or host control policy.
*)

FUNCTION_BLOCK DSFB_GrammarState
VAR_INPUT
    residual : LREAL;
    envelope_rho : LREAL;
    drift : LREAL;
    slew : LREAL;
    drift_counter : UDINT;
    slew_counter : UDINT;
    boundary_fraction : LREAL;
    drift_threshold : LREAL;
    slew_threshold : LREAL;
    drift_persistence : UDINT;
    slew_persistence : UDINT;
END_VAR
VAR_OUTPUT
    state_code : INT; (* 0 = Admissible, 1 = Boundary, 2 = Violation *)
    tri_state_color : STRING[8];
END_VAR
VAR
    abs_residual : LREAL;
    near_boundary : BOOL;
    persistent_drift : BOOL;
    persistent_slew_and_drift : BOOL;
END_VAR

abs_residual := ABS(residual);
near_boundary := abs_residual > (boundary_fraction * envelope_rho);
persistent_drift := ABS(drift) > drift_threshold AND (drift_counter >= drift_persistence);

(* The current Rust implementation gates this condition on persistence counters.
   The counters are expected to be maintained upstream using the same policy. *)
persistent_slew_and_drift :=
    (slew_counter >= slew_persistence) AND
    (drift_counter >= drift_persistence);

IF abs_residual > envelope_rho THEN
    state_code := 2;
ELSIF near_boundary OR persistent_drift OR persistent_slew_and_drift THEN
    state_code := 1;
ELSE
    state_code := 0;
END_IF;

CASE state_code OF
    0: tri_state_color := 'Green';
    1: tri_state_color := 'Yellow';
    2: tri_state_color := 'Red';
ELSE
    tri_state_color := 'Green';
END_CASE;
