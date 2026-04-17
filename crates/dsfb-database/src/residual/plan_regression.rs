//! Plan-regression residuals.
//!
//! Two flavours, both standard in operator practice:
//!  * **latency vs rolling baseline** — `actual_latency − ema_latency` for
//!    the same query class. This is the canonical Query-Store style signal
//!    and is what `pg_stat_statements` `mean_exec_time` lets you derive.
//!  * **plan-hash transition** — emitted as a residual sample with value `1.0`
//!    at the timestamp where the plan hash of a query class changes. This is
//!    fact #11 from the paperstack: *plan change alone is not the event;
//!    plan change plus changed behaviour plus changed waits is*.
//!
//! The grammar layer fuses both signals via a [`dsfb::DsfbObserver`] so that
//! a plan change with no latency reaction does not produce an episode.

use super::{ResidualClass, ResidualSample, ResidualStream};

/// Append a latency residual at `t` for query class `qclass`. The stored
/// residual is `(latency − baseline) / max(baseline, ε)` — a dimensionless
/// fraction so that thresholds in `spec/motifs.yaml` (`drift_threshold:
/// 0.20` etc.) read as "20 % latency drift" regardless of whether the
/// adapter measured in seconds, milliseconds or microseconds.
pub fn push_latency(
    stream: &mut ResidualStream,
    t: f64,
    qclass: &str,
    latency: f64,
    baseline: f64,
) {
    let denom = baseline.abs().max(1e-9);
    let r = (latency - baseline) / denom;
    stream.push(
        ResidualSample::new(t, ResidualClass::PlanRegression, r).with_channel(qclass),
    );
}

/// Mark a plan-hash transition. This is encoded as a unit-impulse residual on
/// the same channel as the latency residual so that the motif state machine
/// can co-locate the two signals.
pub fn push_plan_change(stream: &mut ResidualStream, t: f64, qclass: &str) {
    stream.push(
        ResidualSample::new(t, ResidualClass::PlanRegression, 1.0)
            .with_channel(format!("{qclass}#plan_change")),
    );
}
