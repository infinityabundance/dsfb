//! Process-fault SIGNATURE bank — residual-motif fingerprints over *cheap* sensors.
//!
//! This is the heuristics-bank thesis made concrete: each known chemical-process fault is recorded as
//! a deterministic **signature** — the residual / grammar motif it leaves on cheap, ubiquitous sensors
//! (temperature, pressure, flow, level, vibration RMS, motor current, valve position) — captured once
//! and inferred idempotently from residuals and usually-discarded noise. It is the *fault* counterpart
//! of the H1-H6 process heuristics: H1-H6 label fused episodes; these records pin the per-fault
//! fingerprint and the public datasets that exhibit it.
//!
//! Honesty: `implementation_status` distinguishes signatures the project has actually *executed*
//! (**six**: F6 mass-balance leak witness on the three-/quadruple-tank, BATADAL T1 and SWaT T101; F7
//! sensor-drift isolation on the CSTR thermocouple; and F1 valve stiction, F3 heat-transfer fouling, F8
//! controller-compensation masking, F9 valve hunting on synthetic instrumented demonstrators, all gated
//! by edge `tests/fault_demonstrators.rs`) from those *catalogued* from the literature and their datasets
//! but not yet run here. Every record cites the datasets that exhibit the mechanism.

use crate::detector::ImplementationStatus;

/// Canonical fault mechanism class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum FaultMechanism {
    ValveStiction,
    Cavitation,
    HeatTransferFouling,
    HeatExchangerBypass,
    PumpBearingDegradation,
    ProcessLeak,
    SensorDriftBias,
    ControllerMasking,
    ValveHunting,
    Blockage,
    RotorImbalance,
    RefrigerantCharge,
}

/// One process-fault signature record: the cheap-sensor residual fingerprint of a known fault.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct FaultSignatureRecordV1 {
    /// Stable signature key (`"F1"`..`"F12"`); the identity/sort key folded into `atlas_hash_v1`.
    pub fault_id: &'static str,
    /// Human-readable fault name for reports and figures (e.g. `"Control-valve stiction"`).
    pub name: &'static str,
    /// The physical fault class this signature belongs to (`FaultMechanism`).
    pub mechanism: FaultMechanism,
    /// Unit-operation / equipment contexts where this fault arises (e.g. `"control loop"`, `"pump"`).
    pub process_context: &'static [&'static str],
    /// Cheap, ubiquitous sensors the signature reads (no spectrometer required).
    pub cheap_sensors: &'static [&'static str],
    /// The residual / grammar motif fingerprint (the deterministic signature).
    pub residual_motif: &'static str,
    /// What distinguishes this signature from its confusers.
    pub discriminating_signature: &'static str,
    /// Faults / conditions that can be mistaken for this one.
    pub confuser_faults: &'static [&'static str],
    /// Atlas detector ids whose residuals carry the signature.
    pub detector_inputs: &'static [&'static str],
    /// Public datasets that exhibit this mechanism (provenance for the signature).
    pub exhibiting_datasets: &'static [&'static str],
    /// `Executed` (exercised by a project witness) vs `Catalogued` (literature/dataset-grounded, not yet run here).
    pub implementation_status: ImplementationStatus,
    /// Literature / standard references grounding the fault mechanism and its signature.
    pub source_refs: &'static [&'static str],
}

use FaultMechanism as M;
use ImplementationStatus::{Catalogued, Executed};

/// The process-fault signature bank. `Executed` records are exercised by the project's witnesses;
/// `Catalogued` records are grounded in the literature + named public datasets but not yet run here.
pub const FAULT_SIGNATURES: &[FaultSignatureRecordV1] = &[
    FaultSignatureRecordV1 {
        fault_id: "F1",
        name: "Control-valve stiction",
        mechanism: M::ValveStiction,
        process_context: &["control loops", "actuated valves"],
        cheap_sensors: &["process value (PV)", "controller output (OP)", "setpoint (SP)"],
        residual_motif: "OP ramps while PV stays flat (stick), then PV jumps (slip): sawtooth/limit-cycle in the PV residual; sharp slew after sustained drift",
        discriminating_signature: "PV-vs-OP traces a parallelogram (stick-slip), not a line; slip jump coincident with accumulated OP movement",
        confuser_faults: &["aggressive controller tuning", "external/upstream oscillation", "setpoint changes"],
        detector_inputs: &["page_hinkley_spe", "cusum_spe", "spectral_entropy_spe"],
        exhibiting_datasets: &["valve-stiction instrumented demonstrator (executed)", "DAMADICS f1-f4", "ISDB stiction loops", "UCI Hydraulic valve-condition", "TEP IDV14"],
        implementation_status: Executed,
        source_refs: &["Choudhury et al. stiction quantification", "Jelali & Huang, stiction in control loops", "DSFB-Chemical-Engineering valve-stiction demonstrator (this project)"],
    },
    FaultSignatureRecordV1 {
        fault_id: "F2",
        name: "Pump cavitation",
        mechanism: M::Cavitation,
        process_context: &["centrifugal pumps", "suction-limited flow"],
        cheap_sensors: &["vibration RMS", "discharge pressure", "flow", "motor current"],
        residual_motif: "broadband high-frequency vibration-energy rise with erratic pressure/flow fluctuation; spectral entropy increases",
        discriminating_signature: "broadband (not narrowband) vibration energy + suction-pressure dependence, distinct from a discrete bearing-defect tone",
        confuser_faults: &["flow turbulence", "entrained-air aeration", "rotor imbalance (narrowband)"],
        detector_inputs: &["spectral_entropy_spe", "knn_spe", "ewma_spe"],
        exhibiting_datasets: &["cavitation instrumented demonstrator (executed)", "NLN-EMP centrifugal pump", "SKAB pump loop"],
        implementation_status: Executed,
        source_refs: &["centrifugal-pump cavitation vibration signatures", "DSFB-Chemical-Engineering pump-cavitation demonstrator (this project)"],
    },
    FaultSignatureRecordV1 {
        fault_id: "F3",
        name: "Heat-transfer fouling",
        mechanism: M::HeatTransferFouling,
        process_context: &["heat exchangers", "coolers", "condensers"],
        cheap_sensors: &["inlet/outlet temperatures", "coolant flow"],
        residual_motif: "slow monotonic drift of heat-transfer effectiveness / approach temperature for the same duty; sustained DriftAccum, no sharp slew",
        discriminating_signature: "monotone trend over long horizon at steady load (Mann-Kendall positive), not a step or an ambient transient",
        confuser_faults: &["ambient/utility temperature drift", "planned load ramp"],
        detector_inputs: &["ewma_spe", "mann_kendall_spe", "co_drift"],
        exhibiting_datasets: &["heat-fouling instrumented demonstrator (executed)", "UCI Hydraulic cooler-condition", "ASHRAE RP-1043 condenser fouling", "LBNL fouled-coil"],
        implementation_status: Executed,
        source_refs: &["heat-exchanger fouling FDD literature", "DSFB-Chemical-Engineering heat-fouling demonstrator (this project)"],
    },
    FaultSignatureRecordV1 {
        fault_id: "F4",
        name: "Heat-exchanger bypass / reduced flow",
        mechanism: M::HeatExchangerBypass,
        process_context: &["heat exchangers", "cooling/heating loops"],
        cheap_sensors: &["coolant flow", "inlet/outlet temperature difference"],
        residual_motif: "step drop in flow with a co-moving temperature-difference change; flow and temperature residuals co-drift at onset",
        discriminating_signature: "flow step is the leading edge; the temperature response follows with the loop time-constant (distinct from a temperature-sensor fault, where flow is unchanged)",
        confuser_faults: &["coolant-supply disturbance", "flow-sensor fault"],
        detector_inputs: &["co_drift", "pca_t2", "cusum_spe"],
        exhibiting_datasets: &["ASHRAE RP-1043 reduced condenser/evaporator flow", "DAMADICS bypass-valve faults"],
        implementation_status: Catalogued,
        source_refs: &["chiller reduced-flow FDD (RP-1043)"],
    },
    FaultSignatureRecordV1 {
        fault_id: "F5",
        name: "Pump / bearing degradation",
        mechanism: M::PumpBearingDegradation,
        process_context: &["rotating equipment", "pumps", "motors"],
        cheap_sensors: &["vibration RMS", "motor current", "shaft speed"],
        residual_motif: "rising vibration energy concentrated at characteristic defect frequencies (BPFO/BPFI/BSF and harmonics); progressive DriftAccum with spectral concentration",
        discriminating_signature: "narrowband energy at bearing defect orders (not broadband cavitation, not 1x imbalance); grows monotonically toward failure",
        confuser_faults: &["cavitation (broadband)", "imbalance (1x only)", "looseness"],
        detector_inputs: &["spectral_entropy_spe", "ewma_spe", "knn_spe"],
        exhibiting_datasets: &["CWRU", "MFPT", "IMS/NASA", "FEMTO/PRONOSTIA", "Paderborn KAt", "NLN-EMP", "SKAB"],
        implementation_status: Catalogued,
        source_refs: &["rolling-element bearing defect-frequency diagnostics"],
    },
    FaultSignatureRecordV1 {
        fault_id: "F6",
        name: "Process leak / unmetered sink",
        mechanism: M::ProcessLeak,
        process_context: &["tanks", "pipelines", "water networks", "reactors"],
        cheap_sensors: &["level", "metered inflow/outflow"],
        residual_motif: "mass-balance closure shifts: area*dL/dt no longer matches the net metered flow; the closure residual breaks sustainedly at onset",
        discriminating_signature: "the level/flow inconsistency persists (a true loss), distinct from a transient; for a spoofed level the level reads flat while flow says it should move",
        confuser_faults: &["demand transient", "level-sensor fault (vs a real leak)"],
        detector_inputs: &["mass_energy_balance_witness", "co_drift", "cusum_spe"],
        exhibiting_datasets: &["three-tank/quadruple-tank (executed)", "BATADAL T1 PU2 (executed)", "SWaT T101 (executed)", "BattLeDIM", "DAMADICS leakage"],
        implementation_status: Executed,
        source_refs: &["DSFB-Chemical-Engineering mass-balance witnesses (this project)"],
    },
    FaultSignatureRecordV1 {
        fault_id: "F7",
        name: "Sensor drift / bias",
        mechanism: M::SensorDriftBias,
        process_context: &["any monitored channel"],
        cheap_sensors: &["single sensor channel vs structurally-related neighbours"],
        residual_motif: "one variable drifts while correlated neighbours do not follow; single-variable contribution dominates; balance residuals stay stable (the H1 isolation motif)",
        discriminating_signature: "isolation: the deviation is confined to one channel; a real process change recruits neighbours / breaks a balance",
        confuser_faults: &["a genuinely local process change", "intermittent comms dropouts"],
        detector_inputs: &["sensor_bias", "shewhart_max_z", "robust_z_mad"],
        exhibiting_datasets: &["CSTR thermocouple drift (executed)", "SWaT LIT101 spoof (executed)", "synthetic sensor-bias", "LBNL sensor-bias/drift"],
        implementation_status: Executed,
        source_refs: &["MSPC single-variable contribution isolation", "DSFB-Chemical-Engineering H1 + balance/control-action witnesses (this project)"],
    },
    FaultSignatureRecordV1 {
        fault_id: "F8",
        name: "Controller-compensation masking",
        mechanism: M::ControllerMasking,
        process_context: &["closed-loop controlled processes"],
        cheap_sensors: &["controlled variable", "manipulated variable / valve position"],
        residual_motif: "controlled variable stays in-band while the manipulated-variable residual grows; latent (score-space) residual changes before any raw limit breach (the H6 motif)",
        discriminating_signature: "the loop is absorbing growing stress: structural/latent change precedes a raw alarm; manipulated-variable headroom shrinks",
        confuser_faults: &["normal load-following", "disturbance rejection within design envelope"],
        detector_inputs: &["pca_t2", "ewma_spe", "co_drift"],
        exhibiting_datasets: &["controller-masking instrumented demonstrator (executed)", "TEP closed-loop faults", "DAMADICS positioner faults", "ISDB (controller chasing a sticky valve)"],
        implementation_status: Executed,
        source_refs: &["score-space vs raw-space residual divergence under closed-loop control", "DSFB-Chemical-Engineering controller-masking demonstrator (this project)"],
    },
    FaultSignatureRecordV1 {
        fault_id: "F9",
        name: "Valve hunting / limit-cycle oscillation",
        mechanism: M::ValveHunting,
        process_context: &["control loops", "actuated valves"],
        cheap_sensors: &["process value (PV)", "controller output (OP)"],
        residual_motif: "sustained limit-cycle oscillation in PV/OP: a dominant spectral peak and recurring slew spikes at a near-constant period",
        discriminating_signature: "near-constant oscillation period (limit cycle), distinct from random noise or a one-off transient",
        confuser_faults: &["external periodic disturbance", "interacting loops"],
        detector_inputs: &["spectral_entropy_spe", "page_hinkley_spe"],
        exhibiting_datasets: &["valve-hunting instrumented demonstrator (executed)", "ISDB", "DAMADICS positioner faults", "LBNL control faults"],
        implementation_status: Executed,
        source_refs: &["control-loop oscillation detection literature", "DSFB-Chemical-Engineering valve-hunting demonstrator (this project)"],
    },
    FaultSignatureRecordV1 {
        fault_id: "F10",
        name: "Blockage / clogging",
        mechanism: M::Blockage,
        process_context: &["pipelines", "valves", "inter-unit lines"],
        cheap_sensors: &["flow", "upstream pressure"],
        residual_motif: "flow drop accompanied by an upstream pressure rise; a coupled step in the flow/pressure residual pair",
        discriminating_signature: "flow down AND upstream pressure up together (a restriction), distinct from a leak (flow/level loss) or a pump fault",
        confuser_faults: &["leak (opposite pressure sign)", "demand change"],
        detector_inputs: &["co_drift", "cusum_spe", "pca_t2"],
        exhibiting_datasets: &["PRONTO air/water-line blockage", "DAMADICS valve clogging/sedimentation", "three-tank pipe clogging"],
        implementation_status: Catalogued,
        source_refs: &["restriction/blockage FDD"],
    },
    FaultSignatureRecordV1 {
        fault_id: "F11",
        name: "Rotor imbalance / misalignment",
        mechanism: M::RotorImbalance,
        process_context: &["rotating equipment"],
        cheap_sensors: &["vibration RMS"],
        residual_motif: "vibration energy concentrated at 1x running speed (imbalance) or 1x/2x (misalignment); narrowband spectral peak",
        discriminating_signature: "1x dominance (imbalance) or 1x+2x (misalignment), distinct from bearing defect orders and broadband cavitation",
        confuser_faults: &["bearing degradation (defect orders)", "looseness"],
        detector_inputs: &["spectral_entropy_spe", "ewma_spe"],
        exhibiting_datasets: &["SKAB rotor imbalance", "NLN-EMP unbalance/misalignment", "Paderborn KAt"],
        implementation_status: Catalogued,
        source_refs: &["rotating-machinery vibration order analysis"],
    },
    FaultSignatureRecordV1 {
        fault_id: "F12",
        name: "Refrigerant charge / non-condensables",
        mechanism: M::RefrigerantCharge,
        process_context: &["refrigeration cycles", "chillers"],
        cheap_sensors: &["evaporator/condenser approach temperatures", "subcooling/superheat proxies"],
        residual_motif: "sustained shift in subcooling/superheat and approach-temperature residuals at constant duty; DriftAccum on the thermal-state proxies",
        discriminating_signature: "approach-temperature / subcooling shift without a water-side flow or fouling trend (refrigerant-side, not water-side)",
        confuser_faults: &["fouling (water-side)", "reduced flow", "load change"],
        detector_inputs: &["ewma_spe", "co_drift"],
        exhibiting_datasets: &["ASHRAE RP-1043 refrigerant leak/overcharge, non-condensables"],
        implementation_status: Catalogued,
        source_refs: &["chiller refrigerant-charge FDD (RP-1043)"],
    },
];
