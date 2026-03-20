//! Minimal C ABI surface for bounded DSFB semiotics integration.
//!
//! Unsafe code is required here because C ABI boundaries cannot be expressed without raw pointer
//! handling. The exposed surface stays intentionally small and delegates all science to the safe
//! bounded online engine in the parent crate.

#![deny(unsafe_op_in_unsafe_fn)]

use dsfb_semiotics_engine::engine::settings::EngineSettings;
use dsfb_semiotics_engine::engine::types::{EnvelopeMode, GrammarReasonCode, GrammarState};
use dsfb_semiotics_engine::live::{LiveEngineStatus, OnlineStructuralEngine, Real};
use dsfb_semiotics_engine::math::envelope::EnvelopeSpec;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DsfbFfiResult {
    Ok = 0,
    NullHandle = 1,
    NullOutput = 2,
    InvalidArgument = 3,
    EngineError = 4,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DsfbGrammarState {
    Admissible = 0,
    Boundary = 1,
    Violation = 2,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DsfbGrammarReason {
    Admissible = 0,
    Boundary = 1,
    RecurrentBoundaryGrazing = 2,
    SustainedOutwardDrift = 3,
    AbruptSlewViolation = 4,
    EnvelopeViolation = 5,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DsfbSemanticDisposition {
    Match = 0,
    CompatibleSet = 1,
    Ambiguous = 2,
    Unknown = 3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct DsfbCurrentStatus {
    pub step: u64,
    pub time: f64,
    pub residual_norm: f64,
    pub drift_norm: f64,
    pub slew_norm: f64,
    pub history_buffer_capacity: u64,
    pub current_history_len: u64,
    pub offline_history_len: u64,
    pub grammar_state: DsfbGrammarState,
    pub grammar_reason: DsfbGrammarReason,
    pub semantic_disposition: DsfbSemanticDisposition,
}

pub struct EngineHandle {
    history_buffer_capacity: usize,
    envelope_radius: f64,
    dt: f64,
    engine: OnlineStructuralEngine,
    latest_status: Option<LiveEngineStatus>,
}

impl EngineHandle {
    fn new(history_buffer_capacity: usize, envelope_radius: f64, dt: f64) -> Result<Self, String> {
        if !envelope_radius.is_finite() || envelope_radius <= 0.0 {
            return Err("envelope_radius must be positive and finite".to_string());
        }
        if !dt.is_finite() || dt <= 0.0 {
            return Err("dt must be positive and finite".to_string());
        }
        let mut settings = EngineSettings::default();
        settings.online.history_buffer_capacity = history_buffer_capacity;
        let engine = OnlineStructuralEngine::with_builtin_bank(
            "ffi_live_engine",
            vec!["residual".to_string()],
            dt,
            EnvelopeSpec {
                name: "ffi_fixed_envelope".to_string(),
                mode: EnvelopeMode::Fixed,
                base_radius: envelope_radius,
                slope: 0.0,
                switch_step: None,
                secondary_slope: None,
                secondary_base: None,
            },
            settings,
        )
        .map_err(|error| error.to_string())?;
        Ok(Self {
            history_buffer_capacity,
            envelope_radius,
            dt,
            engine,
            latest_status: None,
        })
    }

    fn reset(&mut self) -> Result<(), String> {
        let replacement = Self::new(self.history_buffer_capacity, self.envelope_radius, self.dt)?;
        self.engine = replacement.engine;
        self.latest_status = None;
        Ok(())
    }
}

#[no_mangle]
pub extern "C" fn dsfb_semiotics_engine_create(
    history_buffer_capacity: usize,
    envelope_radius: f64,
    dt: f64,
) -> *mut EngineHandle {
    match EngineHandle::new(history_buffer_capacity, envelope_radius, dt) {
        Ok(handle) => Box::into_raw(Box::new(handle)),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Releases an engine handle created by `dsfb_semiotics_engine_create`.
///
/// # Safety
///
/// `handle` must be either null or a pointer returned by
/// `dsfb_semiotics_engine_create` that has not already been destroyed.
#[no_mangle]
pub unsafe extern "C" fn dsfb_semiotics_engine_destroy(handle: *mut EngineHandle) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

/// Pushes one residual sample into the bounded live engine.
///
/// # Safety
///
/// `handle` must be a valid mutable pointer returned by
/// `dsfb_semiotics_engine_create` and not yet destroyed.
#[no_mangle]
pub unsafe extern "C" fn dsfb_semiotics_engine_push_sample(
    handle: *mut EngineHandle,
    time: f64,
    residual_value: f64,
) -> DsfbFfiResult {
    let Some(handle) = (unsafe { handle.as_mut() }) else {
        return DsfbFfiResult::NullHandle;
    };
    match handle
        .engine
        .push_residual_sample(time, &[residual_value as Real])
    {
        Ok(status) => {
            handle.latest_status = Some(status);
            DsfbFfiResult::Ok
        }
        Err(_) => DsfbFfiResult::EngineError,
    }
}

/// Writes the latest live-engine status into `out_status`.
///
/// # Safety
///
/// `handle` must be a valid pointer returned by `dsfb_semiotics_engine_create`,
/// and `out_status` must be a valid writable pointer.
#[no_mangle]
pub unsafe extern "C" fn dsfb_semiotics_engine_current_status(
    handle: *const EngineHandle,
    out_status: *mut DsfbCurrentStatus,
) -> DsfbFfiResult {
    let Some(handle) = (unsafe { handle.as_ref() }) else {
        return DsfbFfiResult::NullHandle;
    };
    let Some(out_status) = (unsafe { out_status.as_mut() }) else {
        return DsfbFfiResult::NullOutput;
    };
    let Some(status) = &handle.latest_status else {
        return DsfbFfiResult::InvalidArgument;
    };
    *out_status = DsfbCurrentStatus {
        step: status.step as u64,
        time: status.time,
        residual_norm: status.residual_norm,
        drift_norm: status.drift_norm,
        slew_norm: status.slew_norm,
        history_buffer_capacity: status.history_buffer_capacity as u64,
        current_history_len: status.current_history_len as u64,
        offline_history_len: status.offline_history_len.unwrap_or(0) as u64,
        grammar_state: match status.grammar_state {
            GrammarState::Admissible => DsfbGrammarState::Admissible,
            GrammarState::Boundary => DsfbGrammarState::Boundary,
            GrammarState::Violation => DsfbGrammarState::Violation,
        },
        grammar_reason: match status.grammar_reason_code {
            GrammarReasonCode::Admissible => DsfbGrammarReason::Admissible,
            GrammarReasonCode::Boundary => DsfbGrammarReason::Boundary,
            GrammarReasonCode::RecurrentBoundaryGrazing => {
                DsfbGrammarReason::RecurrentBoundaryGrazing
            }
            GrammarReasonCode::SustainedOutwardDrift => DsfbGrammarReason::SustainedOutwardDrift,
            GrammarReasonCode::AbruptSlewViolation => DsfbGrammarReason::AbruptSlewViolation,
            GrammarReasonCode::EnvelopeViolation => DsfbGrammarReason::EnvelopeViolation,
        },
        semantic_disposition: match status.semantic_disposition.as_str() {
            "Match" => DsfbSemanticDisposition::Match,
            "CompatibleSet" => DsfbSemanticDisposition::CompatibleSet,
            "Ambiguous" => DsfbSemanticDisposition::Ambiguous,
            _ => DsfbSemanticDisposition::Unknown,
        },
    };
    DsfbFfiResult::Ok
}

/// Resets an existing engine handle to its initial bounded state.
///
/// # Safety
///
/// `handle` must be a valid mutable pointer returned by
/// `dsfb_semiotics_engine_create` and not yet destroyed.
#[no_mangle]
pub unsafe extern "C" fn dsfb_semiotics_engine_reset(
    handle: *mut EngineHandle,
) -> DsfbFfiResult {
    let Some(handle) = (unsafe { handle.as_mut() }) else {
        return DsfbFfiResult::NullHandle;
    };
    match handle.reset() {
        Ok(()) => DsfbFfiResult::Ok,
        Err(_) => DsfbFfiResult::EngineError,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_push_measurement_flow_via_ffi() {
        let handle = dsfb_semiotics_engine_create(8, 1.0, 1.0);
        assert!(!handle.is_null());
        unsafe {
            assert_eq!(
                dsfb_semiotics_engine_push_sample(handle, 0.0, 0.1),
                DsfbFfiResult::Ok
            );
            assert_eq!(
                dsfb_semiotics_engine_push_sample(handle, 1.0, 0.2),
                DsfbFfiResult::Ok
            );
            dsfb_semiotics_engine_destroy(handle);
        }
    }

    #[test]
    fn ffi_current_status_query() {
        let handle = dsfb_semiotics_engine_create(4, 1.0, 1.0);
        assert!(!handle.is_null());
        let mut status = DsfbCurrentStatus {
            step: 0,
            time: 0.0,
            residual_norm: 0.0,
            drift_norm: 0.0,
            slew_norm: 0.0,
            history_buffer_capacity: 0,
            current_history_len: 0,
            offline_history_len: 0,
            grammar_state: DsfbGrammarState::Admissible,
            grammar_reason: DsfbGrammarReason::Admissible,
            semantic_disposition: DsfbSemanticDisposition::Unknown,
        };
        unsafe {
            assert_eq!(
                dsfb_semiotics_engine_push_sample(handle, 0.0, 0.1),
                DsfbFfiResult::Ok
            );
            assert_eq!(
                dsfb_semiotics_engine_current_status(handle, &mut status),
                DsfbFfiResult::Ok
            );
            assert_eq!(status.history_buffer_capacity, 4);
            assert!(status.current_history_len >= 1);
            assert_eq!(dsfb_semiotics_engine_reset(handle), DsfbFfiResult::Ok);
            dsfb_semiotics_engine_destroy(handle);
        }
    }
}
