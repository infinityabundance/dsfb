//! Telemetry adapters bridge real telemetry shapes into [`ResidualSample`].
//!
//! DSFB's core observer accepts immutable [`ResidualSample`] values. In real
//! systems, telemetry often arrives as tracing events, metrics snapshots, or
//! protocol-specific records. [`TelemetryAdapter`] lets library users keep
//! those native types while projecting them into the DSFB residual model.

use crate::residual::ResidualSample;

/// Adapter from an application-specific telemetry record into a DSFB sample.
///
/// # Example
///
/// ```rust
/// use dsfb_gray::{ResidualSample, ResidualSource, TelemetryAdapter};
///
/// struct LatencyPoint {
///     p95_ns: u64,
///     baseline_ns: u64,
///     ts_ns: u64,
/// }
///
/// struct LatencyAdapter;
///
/// impl TelemetryAdapter<LatencyPoint> for LatencyAdapter {
///     fn adapt(&self, input: &LatencyPoint) -> ResidualSample {
///         ResidualSample {
///             value: input.p95_ns as f64,
///             baseline: input.baseline_ns as f64,
///             timestamp_ns: input.ts_ns,
///             source: ResidualSource::Latency,
///         }
///     }
/// }
/// ```
pub trait TelemetryAdapter<T> {
    /// Convert one application-specific telemetry record into a DSFB sample.
    fn adapt(&self, input: &T) -> ResidualSample;
}

/// Identity adapter for callers that already hold [`ResidualSample`] values.
#[derive(Debug, Clone, Copy, Default)]
pub struct IdentityTelemetryAdapter;

impl TelemetryAdapter<ResidualSample> for IdentityTelemetryAdapter {
    fn adapt(&self, input: &ResidualSample) -> ResidualSample {
        *input
    }
}
