//! Minimal C ABI surface for bounded DSFB semiotics integration.
//!
//! Unsafe code is required here because C ABI boundaries cannot be expressed without raw pointer
//! handling. The exposed surface stays intentionally small and delegates all science to the safe
//! bounded online engine in the parent crate.

#![deny(unsafe_op_in_unsafe_fn)]

use std::ffi::c_char;
use std::sync::{Mutex, OnceLock};

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
    BufferTooSmall = 5,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DsfbSyntaxCode {
    WeaklyStructuredBaselineLike = 0,
    MixedStructured = 1,
    PersistentOutwardDrift = 2,
    CoordinatedOutwardRise = 3,
    DiscreteEventLike = 4,
    CurvatureRichTransition = 5,
    InwardCompatibleContainment = 6,
    NearBoundaryRecurrent = 7,
    BoundedOscillatoryStructured = 8,
    StructuredNoisyAdmissible = 9,
    Other = 255,
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
    pub trust_scalar: f64,
    pub history_buffer_capacity: u64,
    pub current_history_len: u64,
    pub offline_history_len: u64,
    pub syntax_code: DsfbSyntaxCode,
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

fn last_error_slot() -> &'static Mutex<String> {
    static LAST_ERROR: OnceLock<Mutex<String>> = OnceLock::new();
    LAST_ERROR.get_or_init(|| Mutex::new(String::new()))
}

fn set_last_error(message: impl Into<String>) {
    let mut slot = last_error_slot().lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    *slot = message.into();
}

fn clear_last_error() {
    set_last_error("");
}

fn last_error_message() -> String {
    last_error_slot()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

fn syntax_code_from_label(label: &str) -> DsfbSyntaxCode {
    match label {
        "weakly-structured-baseline-like" => DsfbSyntaxCode::WeaklyStructuredBaselineLike,
        "mixed-structured" => DsfbSyntaxCode::MixedStructured,
        "persistent-outward-drift" => DsfbSyntaxCode::PersistentOutwardDrift,
        "coordinated-outward-rise" => DsfbSyntaxCode::CoordinatedOutwardRise,
        "discrete-event-like" => DsfbSyntaxCode::DiscreteEventLike,
        "curvature-rich-transition" => DsfbSyntaxCode::CurvatureRichTransition,
        "inward-compatible-containment" => DsfbSyntaxCode::InwardCompatibleContainment,
        "near-boundary-recurrent" => DsfbSyntaxCode::NearBoundaryRecurrent,
        "bounded-oscillatory-structured" => DsfbSyntaxCode::BoundedOscillatoryStructured,
        "structured-noisy-admissible" => DsfbSyntaxCode::StructuredNoisyAdmissible,
        _ => DsfbSyntaxCode::Other,
    }
}

fn grammar_reason_from_code(code: GrammarReasonCode) -> DsfbGrammarReason {
    match code {
        GrammarReasonCode::Admissible => DsfbGrammarReason::Admissible,
        GrammarReasonCode::Boundary => DsfbGrammarReason::Boundary,
        GrammarReasonCode::RecurrentBoundaryGrazing => DsfbGrammarReason::RecurrentBoundaryGrazing,
        GrammarReasonCode::SustainedOutwardDrift => DsfbGrammarReason::SustainedOutwardDrift,
        GrammarReasonCode::AbruptSlewViolation => DsfbGrammarReason::AbruptSlewViolation,
        GrammarReasonCode::EnvelopeViolation => DsfbGrammarReason::EnvelopeViolation,
    }
}

fn semantic_disposition_from_code(code: u8) -> DsfbSemanticDisposition {
    match code {
        0 => DsfbSemanticDisposition::Match,
        1 => DsfbSemanticDisposition::CompatibleSet,
        2 => DsfbSemanticDisposition::Ambiguous,
        _ => DsfbSemanticDisposition::Unknown,
    }
}

fn grammar_reason_label(reason: DsfbGrammarReason) -> &'static str {
    match reason {
        DsfbGrammarReason::Admissible => "Admissible",
        DsfbGrammarReason::Boundary => "Boundary",
        DsfbGrammarReason::RecurrentBoundaryGrazing => "RecurrentBoundaryGrazing",
        DsfbGrammarReason::SustainedOutwardDrift => "SustainedOutwardDrift",
        DsfbGrammarReason::AbruptSlewViolation => "AbruptSlewViolation",
        DsfbGrammarReason::EnvelopeViolation => "EnvelopeViolation",
    }
}

fn semantic_disposition_label(disposition: DsfbSemanticDisposition) -> &'static str {
    match disposition {
        DsfbSemanticDisposition::Match => "Match",
        DsfbSemanticDisposition::CompatibleSet => "CompatibleSet",
        DsfbSemanticDisposition::Ambiguous => "Ambiguous",
        DsfbSemanticDisposition::Unknown => "Unknown",
    }
}

unsafe fn copy_string_to_buffer(
    value: &str,
    out_buffer: *mut c_char,
    buffer_len: usize,
) -> DsfbFfiResult {
    if out_buffer.is_null() || buffer_len == 0 {
        set_last_error("output buffer must be non-null and buffer_len must be greater than zero");
        return DsfbFfiResult::InvalidArgument;
    }
    let bytes = value.as_bytes();
    let write_len = bytes.len().min(buffer_len.saturating_sub(1));
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), out_buffer.cast::<u8>(), write_len);
        *out_buffer.add(write_len) = 0;
    }
    if write_len < bytes.len() {
        set_last_error("output buffer was too small; string result was truncated");
        DsfbFfiResult::BufferTooSmall
    } else {
        clear_last_error();
        DsfbFfiResult::Ok
    }
}

fn latest_status(handle: &EngineHandle) -> Result<&LiveEngineStatus, DsfbFfiResult> {
    handle.latest_status.as_ref().ok_or_else(|| {
        set_last_error("no live status is available yet; push at least one residual sample first");
        DsfbFfiResult::InvalidArgument
    })
}

fn map_status(status: &LiveEngineStatus) -> DsfbCurrentStatus {
    DsfbCurrentStatus {
        step: status.step as u64,
        time: status.time,
        residual_norm: status.residual_norm,
        drift_norm: status.drift_norm,
        slew_norm: status.slew_norm,
        trust_scalar: status.trust_scalar,
        history_buffer_capacity: status.history_buffer_capacity as u64,
        current_history_len: status.current_history_len as u64,
        offline_history_len: status.offline_history_len.unwrap_or(0) as u64,
        syntax_code: syntax_code_from_label(&status.syntax_label),
        grammar_state: match status.grammar_state {
            GrammarState::Admissible => DsfbGrammarState::Admissible,
            GrammarState::Boundary => DsfbGrammarState::Boundary,
            GrammarState::Violation => DsfbGrammarState::Violation,
        },
        grammar_reason: grammar_reason_from_code(status.grammar_reason_code),
        semantic_disposition: semantic_disposition_from_code(status.semantic_disposition_code),
    }
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
        Ok(handle) => {
            clear_last_error();
            Box::into_raw(Box::new(handle))
        }
        Err(error) => {
            set_last_error(error);
            std::ptr::null_mut()
        }
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
        set_last_error("null engine handle");
        return DsfbFfiResult::NullHandle;
    };
    match handle
        .engine
        .push_residual_sample(time, &[residual_value as Real])
    {
        Ok(status) => {
            handle.latest_status = Some(status);
            clear_last_error();
            DsfbFfiResult::Ok
        }
        Err(error) => {
            set_last_error(error.to_string());
            DsfbFfiResult::EngineError
        }
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
        set_last_error("null engine handle");
        return DsfbFfiResult::NullHandle;
    };
    let Some(out_status) = (unsafe { out_status.as_mut() }) else {
        set_last_error("null output pointer");
        return DsfbFfiResult::NullOutput;
    };
    let status = match latest_status(handle) {
        Ok(status) => status,
        Err(code) => return code,
    };
    *out_status = map_status(status);
    clear_last_error();
    DsfbFfiResult::Ok
}

/// Writes the latest trust scalar into `out_trust`.
///
/// # Safety
///
/// `handle` must be a valid pointer returned by `dsfb_semiotics_engine_create`,
/// and `out_trust` must be a valid writable pointer.
#[no_mangle]
pub unsafe extern "C" fn dsfb_semiotics_engine_current_trust_scalar(
    handle: *const EngineHandle,
    out_trust: *mut f64,
) -> DsfbFfiResult {
    let Some(handle) = (unsafe { handle.as_ref() }) else {
        set_last_error("null engine handle");
        return DsfbFfiResult::NullHandle;
    };
    let Some(out_trust) = (unsafe { out_trust.as_mut() }) else {
        set_last_error("null output pointer");
        return DsfbFfiResult::NullOutput;
    };
    let status = match latest_status(handle) {
        Ok(status) => status,
        Err(code) => return code,
    };
    *out_trust = status.trust_scalar;
    clear_last_error();
    DsfbFfiResult::Ok
}

/// Copies the current syntax label into a caller-owned buffer.
///
/// # Safety
///
/// `handle` must be a valid pointer returned by `dsfb_semiotics_engine_create`.
/// `out_buffer` must be writable for `buffer_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn dsfb_semiotics_engine_copy_current_syntax_label(
    handle: *const EngineHandle,
    out_buffer: *mut c_char,
    buffer_len: usize,
) -> DsfbFfiResult {
    let Some(handle) = (unsafe { handle.as_ref() }) else {
        set_last_error("null engine handle");
        return DsfbFfiResult::NullHandle;
    };
    let status = match latest_status(handle) {
        Ok(status) => status,
        Err(code) => return code,
    };
    unsafe { copy_string_to_buffer(&status.syntax_label, out_buffer, buffer_len) }
}

/// Copies the current grammar reason label into a caller-owned buffer.
///
/// # Safety
///
/// `handle` must be a valid pointer returned by `dsfb_semiotics_engine_create`.
/// `out_buffer` must be writable for `buffer_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn dsfb_semiotics_engine_copy_current_grammar_label(
    handle: *const EngineHandle,
    out_buffer: *mut c_char,
    buffer_len: usize,
) -> DsfbFfiResult {
    let Some(handle) = (unsafe { handle.as_ref() }) else {
        set_last_error("null engine handle");
        return DsfbFfiResult::NullHandle;
    };
    let status = match latest_status(handle) {
        Ok(status) => status,
        Err(code) => return code,
    };
    let label = grammar_reason_label(grammar_reason_from_code(status.grammar_reason_code));
    unsafe { copy_string_to_buffer(label, out_buffer, buffer_len) }
}

/// Copies the current grammar reason explanation text into a caller-owned buffer.
///
/// # Safety
///
/// `handle` must be a valid pointer returned by `dsfb_semiotics_engine_create`.
/// `out_buffer` must be writable for `buffer_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn dsfb_semiotics_engine_copy_current_grammar_reason_text(
    handle: *const EngineHandle,
    out_buffer: *mut c_char,
    buffer_len: usize,
) -> DsfbFfiResult {
    let Some(handle) = (unsafe { handle.as_ref() }) else {
        set_last_error("null engine handle");
        return DsfbFfiResult::NullHandle;
    };
    let status = match latest_status(handle) {
        Ok(status) => status,
        Err(code) => return code,
    };
    unsafe { copy_string_to_buffer(&status.grammar_reason_text, out_buffer, buffer_len) }
}

/// Copies the current semantic disposition label into a caller-owned buffer.
///
/// # Safety
///
/// `handle` must be a valid pointer returned by `dsfb_semiotics_engine_create`.
/// `out_buffer` must be writable for `buffer_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn dsfb_semiotics_engine_copy_current_semantic_label(
    handle: *const EngineHandle,
    out_buffer: *mut c_char,
    buffer_len: usize,
) -> DsfbFfiResult {
    let Some(handle) = (unsafe { handle.as_ref() }) else {
        set_last_error("null engine handle");
        return DsfbFfiResult::NullHandle;
    };
    let status = match latest_status(handle) {
        Ok(status) => status,
        Err(code) => return code,
    };
    let label = semantic_disposition_label(semantic_disposition_from_code(
        status.semantic_disposition_code,
    ));
    unsafe { copy_string_to_buffer(label, out_buffer, buffer_len) }
}

/// Returns the number of bytes required to store the current last-error string, including the
/// trailing NUL terminator.
#[no_mangle]
pub extern "C" fn dsfb_semiotics_engine_last_error_length() -> usize {
    last_error_message().len() + 1
}

/// Copies the last FFI error string into a caller-owned buffer.
///
/// # Safety
///
/// `out_buffer` must be writable for `buffer_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn dsfb_semiotics_engine_copy_last_error(
    out_buffer: *mut c_char,
    buffer_len: usize,
) -> DsfbFfiResult {
    let message = last_error_message();
    if out_buffer.is_null() || buffer_len == 0 {
        return DsfbFfiResult::InvalidArgument;
    }
    let bytes = message.as_bytes();
    let write_len = bytes.len().min(buffer_len.saturating_sub(1));
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), out_buffer.cast::<u8>(), write_len);
        *out_buffer.add(write_len) = 0;
    }
    if write_len < bytes.len() {
        DsfbFfiResult::BufferTooSmall
    } else {
        DsfbFfiResult::Ok
    }
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
        set_last_error("null engine handle");
        return DsfbFfiResult::NullHandle;
    };
    match handle.reset() {
        Ok(()) => {
            clear_last_error();
            DsfbFfiResult::Ok
        }
        Err(error) => {
            set_last_error(error);
            DsfbFfiResult::EngineError
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_push_measurement_flow_via_ffi() {
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
    fn test_ffi_current_status_query() {
        let handle = dsfb_semiotics_engine_create(4, 1.0, 1.0);
        assert!(!handle.is_null());
        let mut status = DsfbCurrentStatus {
            step: 0,
            time: 0.0,
            residual_norm: 0.0,
            drift_norm: 0.0,
            slew_norm: 0.0,
            trust_scalar: 0.0,
            history_buffer_capacity: 0,
            current_history_len: 0,
            offline_history_len: 0,
            syntax_code: DsfbSyntaxCode::Other,
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
            assert_ne!(status.syntax_code, DsfbSyntaxCode::Other);
            assert_eq!(dsfb_semiotics_engine_reset(handle), DsfbFfiResult::Ok);
            dsfb_semiotics_engine_destroy(handle);
        }
    }

    #[test]
    fn ffi_trust_and_label_queries_work() {
        let handle = dsfb_semiotics_engine_create(8, 1.0, 1.0);
        assert!(!handle.is_null());
        let mut trust = 0.0;
        let mut syntax_label = [0 as c_char; 64];
        let mut grammar_label = [0 as c_char; 64];
        let mut semantic_label = [0 as c_char; 64];
        let mut grammar_reason_text = [0 as c_char; 128];
        unsafe {
            assert_eq!(
                dsfb_semiotics_engine_push_sample(handle, 0.0, 0.12),
                DsfbFfiResult::Ok
            );
            assert_eq!(
                dsfb_semiotics_engine_current_trust_scalar(handle, &mut trust),
                DsfbFfiResult::Ok
            );
            assert!((0.0..=1.0).contains(&trust));
            assert_eq!(
                dsfb_semiotics_engine_copy_current_syntax_label(
                    handle,
                    syntax_label.as_mut_ptr(),
                    syntax_label.len(),
                ),
                DsfbFfiResult::Ok
            );
            assert_eq!(
                dsfb_semiotics_engine_copy_current_grammar_label(
                    handle,
                    grammar_label.as_mut_ptr(),
                    grammar_label.len(),
                ),
                DsfbFfiResult::Ok
            );
            assert_eq!(
                dsfb_semiotics_engine_copy_current_semantic_label(
                    handle,
                    semantic_label.as_mut_ptr(),
                    semantic_label.len(),
                ),
                DsfbFfiResult::Ok
            );
            assert_eq!(
                dsfb_semiotics_engine_copy_current_grammar_reason_text(
                    handle,
                    grammar_reason_text.as_mut_ptr(),
                    grammar_reason_text.len(),
                ),
                DsfbFfiResult::Ok
            );
            dsfb_semiotics_engine_destroy(handle);
        }
    }

    #[test]
    fn ffi_last_error_is_copyable() {
        let mut buffer = [0 as c_char; 128];
        unsafe {
            assert_eq!(
                dsfb_semiotics_engine_push_sample(std::ptr::null_mut(), 0.0, 0.1),
                DsfbFfiResult::NullHandle
            );
            assert!(dsfb_semiotics_engine_last_error_length() > 1);
            assert_eq!(
                dsfb_semiotics_engine_copy_last_error(buffer.as_mut_ptr(), buffer.len()),
                DsfbFfiResult::Ok
            );
        }
    }
}
