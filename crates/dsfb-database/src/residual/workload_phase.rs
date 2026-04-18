//! Workload phase residuals.
//!
//! The mix of query digests (statement classes) shifts over time as ETL
//! windows, reporting bursts, and ad-hoc spikes overlap. We track this with
//! one residual per time bucket: the *Jensen–Shannon divergence* between the
//! current digest-mix histogram and the rolling reference histogram. JS is
//! bounded in `[0, 1]`, symmetric, and zero when distributions agree —
//! exactly the properties the motif state machine needs to call a phase
//! transition.
//!
//! MySQL Performance Schema digests (fact #19), SQL Server `sys.dm_exec_query_stats`
//! grouped by `query_hash`, PostgreSQL `pg_stat_statements.queryid` all
//! provide statement-class identifiers compatible with this construction.

use super::{ResidualClass, ResidualSample, ResidualStream};
use std::collections::HashMap;

/// Compute Jensen–Shannon divergence (base 2, in `[0, 1]`) between two
/// digest-mix histograms `p` and `q`. Inputs do not need to be normalised;
/// they are renormalised internally.
pub fn js_divergence(p: &HashMap<String, u64>, q: &HashMap<String, u64>) -> f64 {
    let sp: f64 = p.values().sum::<u64>() as f64;
    let sq: f64 = q.values().sum::<u64>() as f64;
    if sp == 0.0 || sq == 0.0 {
        return 0.0;
    }
    let mut keys: Vec<&String> = p.keys().chain(q.keys()).collect();
    keys.sort();
    keys.dedup();
    let mut acc = 0.0;
    for k in keys {
        let pi = *p.get(k).unwrap_or(&0) as f64 / sp;
        let qi = *q.get(k).unwrap_or(&0) as f64 / sq;
        let mi = 0.5 * (pi + qi);
        if pi > 0.0 {
            acc += 0.5 * pi * (pi / mi).log2();
        }
        if qi > 0.0 {
            acc += 0.5 * qi * (qi / mi).log2();
        }
    }
    acc.clamp(0.0, 1.0)
}

/// Push a phase residual at `t`. `bucket_id` identifies the time bucket
/// (e.g. an ISO-week or a 5-minute window).
pub fn push_jsd(stream: &mut ResidualStream, t: f64, bucket_id: &str, jsd: f64) {
    stream.push(ResidualSample::new(t, ResidualClass::WorkloadPhase, jsd).with_channel(bucket_id));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn js_zero_when_equal() {
        let mut p = HashMap::new();
        p.insert("a".into(), 3);
        p.insert("b".into(), 7);
        assert!(js_divergence(&p, &p).abs() < 1e-12);
    }
    #[test]
    fn js_positive_when_disjoint() {
        let mut p = HashMap::new();
        p.insert("a".into(), 10);
        let mut q = HashMap::new();
        q.insert("b".into(), 10);
        let d = js_divergence(&p, &q);
        assert!(d > 0.99);
    }
}
