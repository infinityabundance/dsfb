//! Topological Data Analysis (TDA) for RF residual streams.
//!
//! ## Theoretical Basis
//!
//! **Persistent Homology (Edelsbrunner et al. 2002):** As a filtration radius ε grows
//! from 0 to ∞, topological features (connected components = Betti₀, loops = Betti₁)
//! are born and die.  Long-lived ("persistent") features indicate genuine signal
//! structure; short-lived features are topological noise.
//!
//! **Betti₀ (Connected Components):** For a Rips complex built on N residual norms
//! {‖r₁‖, …, ‖r_N‖} in ℝ², two points merge at radius ε when their distance < 2ε.
//! Starting with N isolated components (Betti₀ = N), groups merge as ε increases.
//! - **Pure noise:** merges slowly, many long-lived components → high persistence
//! - **Structured (periodic/drift):** clusters merge rapidly → low Betti₀ at threshold
//!
//! **Innovation Score:** Fraction of birth events with lifetime > mean lifetime.
//! Captures topological "surprise" — how much the current window differs from
//! the expected homogeneous random process topology.
//!
//! ## Design
//!
//! - `no_std`, `no_alloc`, zero `unsafe`
//! - Union-Find on ≤ 64 nodes (O(N·α(N)) amortised — effectively O(N))
//! - Rips filtration over 1D delay-coordinate pairs
//! - Fixed-capacity birth/death event log
//!
//! ## References
//!
//! Edelsbrunner, H., Letscher, D. and Zomorodian, A. (2002) "Topological persistence
//!   and simplification," *Discrete & Computational Geometry* 28:511–533.
//!   doi:10.1007/s00454-002-2885-2.
//!
//! Zomorodian, A. and Carlsson, G. (2005) "Computing persistent homology,"
//!   *Discrete & Computational Geometry* 33:249–274. doi:10.1007/s00454-004-1146-y.
//!
//! Bubenik, P. (2015) "Statistical topological data analysis using persistence
//!   landscapes," *JMLR* 16:77–102. https://jmlr.org/papers/v16/bubenik15a.html.

// ── Topological State ──────────────────────────────────────────────────────

/// Topological birth/death event in the Rips filtration.
#[derive(Debug, Clone, Copy)]
pub struct PersistenceEvent {
    /// Filtration radius at which this component was born.
    pub birth_radius: f32,
    /// Filtration radius at which this component died (merged).
    /// `f32::MAX` for the essential component (never dies).
    pub death_radius: f32,
}

impl PersistenceEvent {
    /// Lifetime = death − birth.  MAX for the essential class.
    pub fn lifetime(&self) -> f32 {
        if self.death_radius == f32::MAX { f32::MAX }
        else { self.death_radius - self.birth_radius }
    }
}

/// Topological summary of a residual window.
#[derive(Debug, Clone, Copy)]
pub struct TopologicalState {
    /// Betti₀ at the structural threshold radius.
    pub betti0: u32,
    /// Number of birth events in the filtration (= N).
    pub n_births: u32,
    /// Number of death events (merge events).
    pub n_deaths: u32,
    /// Topological innovation score ∈ [0, 1].
    /// Fraction of components with above-mean lifetime.
    pub innovation_score: f32,
    /// Total persistence (sum of finite lifetimes).
    pub total_persistence: f32,
}

// ── Union-Find ─────────────────────────────────────────────────────────────

/// Path-compressed, union-by-rank disjoint-set data structure.
///
/// Supports ≤ N nodes.  Used for Rips Betti₀ computation via filtration.
pub struct UnionFind<const N: usize> {
    parent: [u8; N],
    rank:   [u8; N],
    count:  usize,
}

impl<const N: usize> UnionFind<N> {
    /// Initialise N isolated components.
    pub const fn new(n: usize) -> Self {
        let mut parent = [0u8; N];
        let mut i = 0usize;
        while i < N { parent[i] = i as u8; i += 1; }
        Self { parent, rank: [0u8; N], count: if n < N { n } else { N } }
    }

    /// Find the representative of node `x` with iterative path compression.
    ///
    /// Two-pass iteration: walk parents until we hit the root, then walk
    /// again to splice every visited node directly onto the root. Bounded
    /// at N iterations per pass — no recursion, no stack growth.
    pub fn find(&mut self, x: usize) -> usize {
        if x >= N {
            return x;
        }
        let mut cursor = x;
        let mut steps = 0usize;
        while self.parent[cursor] as usize != cursor && steps < N {
            cursor = self.parent[cursor] as usize;
            steps += 1;
        }
        let root = cursor;
        let mut cursor = x;
        let mut steps = 0usize;
        while self.parent[cursor] as usize != root && steps < N {
            let next = self.parent[cursor] as usize;
            self.parent[cursor] = root as u8;
            cursor = next;
            steps += 1;
        }
        root
    }

    /// Union the sets containing `a` and `b`.  Returns true if they were in
    /// different sets (a merge happened).
    pub fn union(&mut self, a: usize, b: usize) -> bool {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra == rb { return false; }
        if self.rank[ra] < self.rank[rb] {
            self.parent[ra] = rb as u8;
        } else if self.rank[ra] > self.rank[rb] {
            self.parent[rb] = ra as u8;
        } else {
            self.parent[rb] = ra as u8;
            self.rank[ra] = self.rank[ra].saturating_add(1);
        }
        if self.count > 0 { self.count -= 1; }
        true
    }

    /// Current number of connected components.
    pub fn component_count(&self) -> usize { self.count }
}

// ── Persistence Landscape / Filtration ────────────────────────────────────

/// Fixed-capacity persistence event log.
pub struct PersistenceLog<const E: usize> {
    events: [PersistenceEvent; E],
    len: usize,
}

impl<const E: usize> PersistenceLog<E> {
    const fn new() -> Self {
        Self {
            events: [PersistenceEvent { birth_radius: 0.0, death_radius: f32::MAX }; E],
            len: 0,
        }
    }

    fn push(&mut self, ev: PersistenceEvent) {
        if self.len < E {
            self.events[self.len] = ev;
            self.len += 1;
        }
    }

    /// Slice of recorded events.
    pub fn events(&self) -> &[PersistenceEvent] { &self.events[..self.len] }

    /// Mean finite lifetime.
    fn mean_finite_lifetime(&self) -> f32 {
        let (mut sum, mut cnt) = (0.0_f32, 0u32);
        for ev in self.events() {
            if ev.death_radius < f32::MAX {
                sum += ev.death_radius - ev.birth_radius;
                cnt += 1;
            }
        }
        if cnt == 0 { 0.0 } else { sum / cnt as f32 }
    }

    /// Total persistence (sum of finite lifetimes).
    fn total_persistence(&self) -> f32 {
        self.events().iter()
            .filter(|e| e.death_radius < f32::MAX)
            .map(|e| e.death_radius - e.birth_radius)
            .sum()
    }

    /// Fraction of components with lifetime > mean (topological innovation).
    fn innovation_score(&self) -> f32 {
        let mean = self.mean_finite_lifetime();
        let finite: VecF32<64> = self.events().iter()
            .filter(|e| e.death_radius < f32::MAX)
            .map(|e| e.death_radius - e.birth_radius)
            .collect_fixed();
        if finite.len == 0 { return 0.0; }
        let above = finite.data[..finite.len].iter().filter(|&&l| l > mean).count();
        above as f32 / finite.len as f32
    }
}

// ── Mini fixed-capacity vec for innovation score computation ───────────────

struct VecF32<const C: usize> { data: [f32; C], len: usize }
impl<const C: usize> VecF32<C> {
    fn new() -> Self { Self { data: [0.0; C], len: 0 } }
    fn push(&mut self, v: f32) { if self.len < C { self.data[self.len] = v; self.len += 1; } }
}
trait CollectFixed<T, const C: usize> {
    fn collect_fixed(self) -> VecF32<C>;
}
impl<I: Iterator<Item = f32>, const C: usize> CollectFixed<f32, C> for I {
    fn collect_fixed(mut self) -> VecF32<C> {
        let mut v = VecF32::new();
        while let Some(x) = self.next() { v.push(x); }
        v
    }
}

// ── Primary API ────────────────────────────────────────────────────────────

/// Compute Betti₀ and topological innovation for a window of residual norms.
///
/// Builds a 1D Rips filtration: points are the N residual norm values on ℝ.
/// Two points i, j merge when |norm_i − norm_j| < `radius`.
///
/// - `norms`:  Residual norm window (≥ 2 samples, ≤ 64 samples recommended).
/// - `radius`: Structural threshold radius (should be ≈ ρ/2 from GUM budget).
///
/// Returns `None` if fewer than 2 samples.
pub fn detect_topological_innovation(norms: &[f32], radius: f32) -> Option<TopologicalState> {
    let n = norms.len();
    if n < 2 || n > 64 { return None; }

    let (dists, nd) = collect_sorted_pair_distances(norms);
    let log = run_persistence_filtration(&dists, nd, n);
    let betti0 = compute_betti0_at_radius(&dists, nd, n, radius);

    let n_deaths = log.events().iter()
        .filter(|e| e.death_radius < f32::MAX).count() as u32;

    Some(TopologicalState {
        betti0,
        n_births: n as u32,
        n_deaths,
        innovation_score: log.innovation_score(),
        total_persistence: log.total_persistence(),
    })
}

fn collect_sorted_pair_distances(norms: &[f32]) -> ([(f32, u8, u8); 2016], usize) {
    let n = norms.len();
    let mut dists = [(0.0_f32, 0u8, 0u8); 2016];
    let mut nd = 0usize;
    for i in 0..n {
        for j in (i + 1)..n {
            let d = (norms[i] - norms[j]).abs();
            if nd < 2016 { dists[nd] = (d, i as u8, j as u8); nd += 1; }
        }
    }
    for i in 1..nd {
        let key = dists[i];
        let mut j = i;
        while j > 0 && dists[j - 1].0 > key.0 {
            dists[j] = dists[j - 1];
            j -= 1;
        }
        dists[j] = key;
    }
    (dists, nd)
}

fn run_persistence_filtration(
    dists: &[(f32, u8, u8); 2016], nd: usize, n: usize,
) -> PersistenceLog<64> {
    let mut uf = UnionFind::<64>::new(n);
    let mut log: PersistenceLog<64> = PersistenceLog::new();
    for _ in 0..n {
        log.push(PersistenceEvent { birth_radius: 0.0, death_radius: f32::MAX });
    }
    for k in 0..nd {
        let (d, i, j) = dists[k];
        let ri = uf.find(i as usize);
        let rj = uf.find(j as usize);
        if ri != rj {
            let dying = if ri > rj { ri } else { rj };
            if dying < log.events.len() {
                log.events[dying].death_radius = d;
            }
            uf.union(i as usize, j as usize);
        }
    }
    log
}

fn compute_betti0_at_radius(
    dists: &[(f32, u8, u8); 2016], nd: usize, n: usize, radius: f32,
) -> u32 {
    let mut uf = UnionFind::<64>::new(n);
    for k in 0..nd {
        let (d, i, j) = dists[k];
        if d <= radius {
            uf.union(i as usize, j as usize);
        }
    }
    uf.component_count() as u32
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_none() {
        assert!(detect_topological_innovation(&[], 0.1).is_none());
        assert!(detect_topological_innovation(&[0.1], 0.1).is_none());
    }

    #[test]
    fn constant_norms_single_component() {
        let norms = [0.1_f32; 8];
        let state = detect_topological_innovation(&norms, 0.05).unwrap();
        // All points at same value → all within radius 0 of each other → 1 component
        assert_eq!(state.betti0, 1, "constant norms must merge to 1 component");
    }

    #[test]
    fn widely_spaced_norms_many_components() {
        // 5 norms spread 1.0 apart
        let norms = [0.0_f32, 1.0, 2.0, 3.0, 4.0];
        let state = detect_topological_innovation(&norms, 0.1).unwrap();
        // With radius 0.1, no two points merge → Betti₀ = N
        assert_eq!(state.betti0, 5, "widely spaced: Betti₀ should be 5");
    }

    #[test]
    fn intermediate_radius_merges_clusters() {
        // Two tight clusters: {0.0, 0.02, 0.04} and {1.0, 1.02, 1.04}
        let norms = [0.0_f32, 0.02, 0.04, 1.0, 1.02, 1.04];
        let state = detect_topological_innovation(&norms, 0.05).unwrap();
        // With radius 0.05, the two clusters each merge internally → 2 components
        assert_eq!(state.betti0, 2, "two clusters: Betti₀={}", state.betti0);
    }

    #[test]
    fn union_find_basic() {
        let mut uf = UnionFind::<8>::new(5);
        assert_eq!(uf.component_count(), 5);
        assert!(uf.union(0, 1));
        assert_eq!(uf.component_count(), 4);
        assert!(!uf.union(0, 1)); // already same set
        assert_eq!(uf.component_count(), 4);
        uf.union(2, 3);
        uf.union(0, 2);
        assert_eq!(uf.component_count(), 2);
    }

    #[test]
    fn persistence_event_lifetime() {
        let ev1 = PersistenceEvent { birth_radius: 0.1, death_radius: 0.5 };
        let ev2 = PersistenceEvent { birth_radius: 0.0, death_radius: f32::MAX };
        assert!((ev1.lifetime() - 0.4).abs() < 1e-5);
        assert_eq!(ev2.lifetime(), f32::MAX);
    }
}
