//! Residual streams.
//!
//! A *residual* is the difference between an expectation a SQL engine had
//! about something and what actually happened — almost every modern engine
//! computes residuals internally and then logs only shallow summaries of them
//! (`paneldiscussion.txt` in the paperstack lists six families). This module
//! defines the canonical typed residual stream that the DSFB observer and the
//! motif grammar consume.
//!
//! Residual *construction* is engine-specific (see the per-class submodules
//! below); residual *interpretation* is engine-agnostic (see `grammar`).

use serde::{Deserialize, Serialize};

pub mod cache_io;
pub mod cardinality;
pub mod contention;
pub mod plan_regression;
pub mod workload_phase;

/// The five residual classes emitted by SQL engines that DSFB-Database
/// structures. Names and definitions match Section 3 (Residual Taxonomy) of
/// the paper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResidualClass {
    /// Latency vs rolling baseline; plan-hash transitions.
    PlanRegression,
    /// `actual_rows / estimated_rows` divergence per plan node or per query.
    Cardinality,
    /// Lock-wait depth, blocked-by chain length, queue depth.
    Contention,
    /// Buffer / cache hit-ratio drop with I/O-wait amplification.
    CacheIo,
    /// Digest-mix entropy and class-distribution drift across query workload.
    WorkloadPhase,
}

impl ResidualClass {
    pub const ALL: [ResidualClass; 5] = [
        Self::PlanRegression,
        Self::Cardinality,
        Self::Contention,
        Self::CacheIo,
        Self::WorkloadPhase,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::PlanRegression => "plan_regression",
            Self::Cardinality => "cardinality",
            Self::Contention => "contention",
            Self::CacheIo => "cache_io",
            Self::WorkloadPhase => "workload_phase",
        }
    }
}

/// A single residual sample. `t` is logical time (seconds since stream start).
/// `value` is the residual quantity in the class-specific natural units (the
/// units are documented per class in the paper's Table 2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidualSample {
    pub t: f64,
    pub class: ResidualClass,
    /// The residual itself: `actual − expected` (or `actual / expected` for
    /// cardinality, log-transformed). Never NaN; missing values are dropped
    /// at the adapter boundary so downstream code can rely on this.
    pub value: f64,
    /// Optional channel discriminator (e.g. plan_hash, table id, wait_event
    /// name). Used by the motif grammar to scope episodes.
    pub channel: Option<String>,
}

impl ResidualSample {
    pub fn new(t: f64, class: ResidualClass, value: f64) -> Self {
        debug_assert!(value.is_finite(), "residual value must be finite");
        Self {
            t,
            class,
            value,
            channel: None,
        }
    }

    pub fn with_channel(mut self, channel: impl Into<String>) -> Self {
        self.channel = Some(channel.into());
        self
    }
}

/// A typed, time-ordered stream of residuals from a single source (one
/// dataset, one engine, one observation window). Construction is the
/// adapter's responsibility; the stream is otherwise immutable.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResidualStream {
    /// Source label — dataset name, engine, version, subset.
    pub source: String,
    /// Samples sorted by `t` ascending. Adapters MUST sort.
    pub samples: Vec<ResidualSample>,
}

impl ResidualStream {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            samples: Vec::new(),
        }
    }

    pub fn push(&mut self, s: ResidualSample) {
        self.samples.push(s);
    }

    pub fn sort(&mut self) {
        self.samples
            .sort_by(|a, b| a.t.partial_cmp(&b.t).expect("residual t is finite"));
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    pub fn duration(&self) -> f64 {
        match (self.samples.first(), self.samples.last()) {
            (Some(a), Some(b)) => b.t - a.t,
            _ => 0.0,
        }
    }

    /// View-only iterator over samples of a single class (used by the
    /// per-motif state machines).
    pub fn iter_class(
        &self,
        class: ResidualClass,
    ) -> impl Iterator<Item = &ResidualSample> + '_ {
        self.samples.iter().filter(move |s| s.class == class)
    }

    /// Stable hash of the residual stream — used by the
    /// replay-determinism test to confirm bytewise identical runs.
    pub fn fingerprint(&self) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(self.source.as_bytes());
        for s in &self.samples {
            h.update(s.t.to_le_bytes());
            h.update((s.class as u8).to_le_bytes());
            h.update(s.value.to_le_bytes());
            if let Some(c) = &s.channel {
                h.update(c.as_bytes());
            }
            h.update(b"|");
        }
        h.finalize().into()
    }
}
