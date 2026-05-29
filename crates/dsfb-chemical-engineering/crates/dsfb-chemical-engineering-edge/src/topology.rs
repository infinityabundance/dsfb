//! `ProcessTopologyGraphV1` + `ResidenceTimeAlignmentV1` (P64).
//!
//! Multi-unit plants have *structure*: a feed feeds a reactor, the reactor feeds a separator, and each
//! connection has a physical **residence time** (the transport/holdup delay between units). Two new
//! hash-sealed objects make that structure first-class:
//!
//! - [`ProcessTopologyGraphV1`] — the declared unit/flow topology, each flow carrying its residence
//!   time. Emittable as Graphviz DOT. This is the *declared* plant graph a propagation witness reads.
//! - [`ResidenceTimeAlignmentV1`] — aligns an upstream unit's residual stream to a downstream unit's by
//!   the declared residence-time lag and reports the at-lag correlation. **Advisory only**: a high
//!   at-lag correlation is consistent with propagation, it is *not* a causal proof (see
//!   `propagation::CausalNonClaimGraphV1`).
//!
//! Additive + off the replay path; the multi-unit demonstrator is exercised in the unit tests.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// A process unit (node).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessUnit {
    pub id: String,
    /// Unit kind (e.g. `"feed"`, `"reactor"`, `"separator"`).
    pub kind: String,
}

/// A directed material/energy flow between two units, carrying its declared residence time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessFlow {
    pub from: String,
    pub to: String,
    /// Declared residence/transport time from `from` to `to`.
    pub residence_time: f64,
    /// Time units of `residence_time` (e.g. `"min"`, `"s"`).
    pub time_units: String,
}

/// A hash-sealed process-topology graph (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessTopologyGraphV1 {
    pub plant: String,
    pub units: Vec<ProcessUnit>,
    pub flows: Vec<ProcessFlow>,
    pub topology_hash: String,
}

/// Builder for a process topology graph.
#[derive(Debug, Clone, Default)]
pub struct TopologyBuilder {
    plant: String,
    units: Vec<ProcessUnit>,
    flows: Vec<ProcessFlow>,
}

impl TopologyBuilder {
    pub fn new(plant: impl Into<String>) -> Self {
        TopologyBuilder {
            plant: plant.into(),
            units: Vec::new(),
            flows: Vec::new(),
        }
    }
    pub fn unit(&mut self, id: impl Into<String>, kind: impl Into<String>) -> &mut Self {
        self.units.push(ProcessUnit {
            id: id.into(),
            kind: kind.into(),
        });
        self
    }
    pub fn flow(
        &mut self,
        from: impl Into<String>,
        to: impl Into<String>,
        residence_time: f64,
        time_units: impl Into<String>,
    ) -> &mut Self {
        self.flows.push(ProcessFlow {
            from: from.into(),
            to: to.into(),
            residence_time,
            time_units: time_units.into(),
        });
        self
    }
    pub fn seal(&self) -> ProcessTopologyGraphV1 {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"process_topology_graph_v1");
        h.field("plant", self.plant.as_bytes());
        for u in &self.units {
            h.field("unit_id", u.id.as_bytes());
            h.field("unit_kind", u.kind.as_bytes());
        }
        for f in &self.flows {
            h.field("flow_from", f.from.as_bytes());
            h.field("flow_to", f.to.as_bytes());
            h.f64q("residence_time", f.residence_time);
            h.field("time_units", f.time_units.as_bytes());
        }
        ProcessTopologyGraphV1 {
            plant: self.plant.clone(),
            units: self.units.clone(),
            flows: self.flows.clone(),
            topology_hash: h.finalize_hex(),
        }
    }
}

impl ProcessTopologyGraphV1 {
    /// True iff `up` reaches `down` by following flows (directed reachability) — i.e. `up` is upstream.
    pub fn is_upstream_of(&self, up: &str, down: &str) -> bool {
        // Simple DFS over the flow edges.
        let mut stack = vec![up.to_string()];
        let mut seen: Vec<String> = Vec::new();
        while let Some(n) = stack.pop() {
            if n == down {
                return true;
            }
            if seen.contains(&n) {
                continue;
            }
            seen.push(n.clone());
            for f in self.flows.iter().filter(|f| f.from == n) {
                stack.push(f.to.clone());
            }
        }
        false
    }

    pub fn verify(&self) -> bool {
        let mut b = TopologyBuilder::new(self.plant.clone());
        b.units = self.units.clone();
        b.flows = self.flows.clone();
        b.seal().topology_hash == self.topology_hash
    }

    /// Render as a Graphviz DOT digraph (units as boxes, flows labelled with residence time).
    pub fn to_dot(&self) -> String {
        let esc = |s: &str| s.replace('"', "'");
        let mut out = format!(
            "digraph process_topology {{\n  label=\"{}\";\n  rankdir=LR;\n",
            esc(&self.plant)
        );
        for u in &self.units {
            out.push_str(&format!(
                "  \"{}\" [shape=box, label=\"{} ({})\"];\n",
                esc(&u.id),
                esc(&u.id),
                esc(&u.kind)
            ));
        }
        for f in &self.flows {
            out.push_str(&format!(
                "  \"{}\" -> \"{}\" [label=\"τ={} {}\"];\n",
                esc(&f.from),
                esc(&f.to),
                f.residence_time,
                esc(&f.time_units)
            ));
        }
        out.push_str("}\n");
        out
    }
}

/// A hash-sealed residence-time alignment between two units' residual streams (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResidenceTimeAlignmentV1 {
    pub upstream_unit: String,
    pub downstream_unit: String,
    pub declared_residence_time: f64,
    /// Sampling interval used to convert residence time to a sample lag.
    pub dt: f64,
    /// Residence time expressed in samples (`round(residence_time / dt)`).
    pub lag_samples: usize,
    /// Pearson correlation between `upstream[t]` and `downstream[t + lag]` over the overlap.
    pub aligned_correlation: f64,
    pub alignment_hash: String,
}

impl ResidenceTimeAlignmentV1 {
    fn seal(up: &str, down: &str, rt: f64, dt: f64, lag: usize, corr: f64) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"residence_time_alignment_v1");
        h.field("upstream_unit", up.as_bytes());
        h.field("downstream_unit", down.as_bytes());
        h.f64q("declared_residence_time", rt);
        h.f64q("dt", dt);
        h.u64("lag_samples", lag as u64);
        h.f64q("aligned_correlation", corr);
        h.finalize_hex()
    }

    /// Align `downstream` to `upstream` by the declared residence-time lag and compute the at-lag
    /// Pearson correlation over the overlap. `dt > 0`; a degenerate (constant) stream yields corr 0.
    pub fn build(
        upstream_unit: impl Into<String>,
        downstream_unit: impl Into<String>,
        upstream: &[f64],
        downstream: &[f64],
        declared_residence_time: f64,
        dt: f64,
    ) -> Self {
        let lag_samples = if dt > 0.0 {
            (declared_residence_time / dt).round().max(0.0) as usize
        } else {
            0
        };
        // Pairs (upstream[t], downstream[t+lag]) over the valid overlap.
        let mut xs: Vec<f64> = Vec::new();
        let mut ys: Vec<f64> = Vec::new();
        for (t, &x) in upstream.iter().enumerate() {
            if let Some(&y) = downstream.get(t + lag_samples) {
                if x.is_finite() && y.is_finite() {
                    xs.push(x);
                    ys.push(y);
                }
            }
        }
        let aligned_correlation = pearson(&xs, &ys);
        let up = upstream_unit.into();
        let down = downstream_unit.into();
        let alignment_hash = Self::seal(
            &up,
            &down,
            declared_residence_time,
            dt,
            lag_samples,
            aligned_correlation,
        );
        ResidenceTimeAlignmentV1 {
            upstream_unit: up,
            downstream_unit: down,
            declared_residence_time,
            dt,
            lag_samples,
            aligned_correlation,
            alignment_hash,
        }
    }

    pub fn verify(&self) -> bool {
        Self::seal(
            &self.upstream_unit,
            &self.downstream_unit,
            self.declared_residence_time,
            self.dt,
            self.lag_samples,
            self.aligned_correlation,
        ) == self.alignment_hash
    }
}

/// Pearson correlation of two equal-length slices; returns 0.0 for <2 points or zero variance.
fn pearson(xs: &[f64], ys: &[f64]) -> f64 {
    let n = xs.len().min(ys.len());
    if n < 2 {
        return 0.0;
    }
    let nf = n as f64;
    let mx = xs[..n].iter().sum::<f64>() / nf;
    let my = ys[..n].iter().sum::<f64>() / nf;
    let mut sxy = 0.0;
    let mut sxx = 0.0;
    let mut syy = 0.0;
    for i in 0..n {
        let dx = xs[i] - mx;
        let dy = ys[i] - my;
        sxy += dx * dy;
        sxx += dx * dx;
        syy += dy * dy;
    }
    if sxx <= 0.0 || syy <= 0.0 {
        return 0.0;
    }
    sxy / (sxx.sqrt() * syy.sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The P64 multi-unit demonstrator: feed → reactor → separator with declared residence times.
    fn demonstrator() -> ProcessTopologyGraphV1 {
        let mut b = TopologyBuilder::new("feed_reactor_separator");
        b.unit("feed", "feed")
            .unit("reactor", "reactor")
            .unit("separator", "separator");
        b.flow("feed", "reactor", 3.0, "min")
            .flow("reactor", "separator", 5.0, "min");
        b.seal()
    }

    #[test]
    fn topology_reachability_and_seal() {
        let g = demonstrator();
        assert!(g.is_upstream_of("feed", "separator")); // feed -> reactor -> separator
        assert!(g.is_upstream_of("reactor", "separator"));
        assert!(!g.is_upstream_of("separator", "feed")); // not the reverse
        assert!(g.verify());
        assert!(g.to_dot().contains("τ=3"));
    }

    #[test]
    fn residence_time_alignment_finds_the_lag() {
        // downstream is upstream delayed by 3 samples; with dt=1, declared τ=3 → lag 3 → corr ≈ 1.
        let upstream: Vec<f64> = (0..50).map(|i| (i as f64 * 0.3).sin()).collect();
        let mut downstream = vec![0.0; 3];
        downstream.extend(upstream.iter().take(47).copied());
        let a = ResidenceTimeAlignmentV1::build(
            "reactor",
            "separator",
            &upstream,
            &downstream,
            3.0,
            1.0,
        );
        assert_eq!(a.lag_samples, 3);
        assert!(
            a.aligned_correlation > 0.99,
            "at-lag correlation should be near 1, got {}",
            a.aligned_correlation
        );
        assert!(a.verify());
    }
}
