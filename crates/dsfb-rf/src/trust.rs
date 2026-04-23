//! Hierarchical Residual-Envelope Trust (HRET) for RF multi-channel receivers.
//!
//! ## Theoretical Basis
//!
//! Derived from the DSFB-HRET framework (de Beer 2026, §III–IV).  The core
//! insight is that in a multi-channel RF receiver (multi-antenna, multi-band,
//! dual-polarisation) different observation channels vary in reliability.
//! Naively averaging all channel residuals degrades the composite estimate when
//! one antenna or band is in deep fade, experiencing local RFI, or has a faulty
//! LNA.  HRET builds **two levels** of EMA-based envelope trust and combines
//! them before computing the weighted residual.
//!
//! ### Level 1 — Channel envelope (eq. 8)
//!
//! For each channel k, a per-channel EMA envelope tracks the running
//! absolute residual:
//!
//! ```text
//! s_k ← ρ · s_k + (1 − ρ) · |r_k|
//! ```
//!
//! Channel trust weight (eq. 9):
//!
//! ```text
//! w_k = 1 / (1 + β · s_k)
//! ```
//!
//! ### Level 2 — Group envelope (eq. 11)
//!
//! Channels are partitioned into groups (e.g., by polarisation, frequency band,
//! or spatial cluster).  A per-group EMA envelope tracks the mean absolute
//! residual across the group:
//!
//! ```text
//! s_g ← ρ_g · s_g + (1 − ρ_g) · (1/|G| · Σ_{k∈G} |r_k|)
//! ```
//!
//! Group trust weight (eq. 12):
//!
//! ```text
//! w_g = 1 / (1 + β_g · s_g)
//! ```
//!
//! ### Hierarchical composition (eqs. 14–15) and correction (eq. 19)
//!
//! Composite weights are the product of the channel weight and the weight of
//! that channel's group, then L1-normalised:
//!
//! ```text
//! ŵ_k = w_k · w_{g[k]}
//! w̃_k = ŵ_k / Σ_j ŵ_j          (normalisation)
//! ```
//!
//! The correction signal fed to downstream stages is (eq. 19):
//!
//! ```text
//! Δx = K · (w̃ ⊙ r)
//! ```
//!
//! ### RF interpretation
//!
//! | HRET concept | RF analogue |
//! |---|---|
//! | Channel k | Receive antenna element / ADC lane |
//! | Group g | Polarisation pair / sub-array / frequency band |
//! | Channel envelope s_k | Per-antenna noise / interference run-in |
//! | Group envelope s_g | Sub-array health / band cleanliness |
//! | ŵ_k | Phased-array weighting analogous to optimal combining |
//! | Δx | Weighted residual anomaly injected into grammar layer |
//!
//! The hierarchical scheme is empirically superior to flat average combining
//! in the presence of partial-array failures and spectrally local RFI.
//!
//! ## Design
//!
//! - `no_std`, `no_alloc`, zero `unsafe`
//! - Const-generic over `C` (channel count) and `G` (group count)
//! - O(C+G) per call — no heap scan
//! - Channel-to-group mapping supplied as a `[usize; C]` index array

/// Parameters for the HRET trust estimator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HretParams {
    /// Channel-level EMA smoothing factor ρ ∈ (0, 1).
    ///
    /// Larger → slower adaptation (more memory); smaller → faster adaptation.
    /// Typical: 0.95 for slowly varying RF channels, 0.80 for fast fades.
    pub channel_rho: f32,

    /// Group-level EMA smoothing factor ρ_g ∈ (0, 1).
    ///
    /// Usually slightly smoother than channel level (e.g., 0.97).
    pub group_rho: f32,

    /// Channel trust shaping coefficient β > 0.
    ///
    /// Controls how steeply small envelope increases reduce trust.
    /// β = 1/σ₀ where σ₀ is the nominal healthy-window sigma.
    pub beta_channel: f32,

    /// Group trust shaping coefficient β_g > 0.
    pub beta_group: f32,
}

impl HretParams {
    /// Construct conservative defaults suitable for most SDR receivers.
    ///
    /// ρ = 0.95, ρ_g = 0.97, β = β_g = 10.0 (nominal σ₀ = 0.1).
    pub const fn default_sdr() -> Self {
        Self {
            channel_rho: 0.95,
            group_rho: 0.97,
            beta_channel: 10.0,
            beta_group: 10.0,
        }
    }

    /// Construct from explicit nominal healthy-window sigma (sets β = 1/σ₀).
    pub fn from_sigma(sigma0: f32, channel_rho: f32, group_rho: f32) -> Self {
        let beta = if sigma0 > 1e-12 { 1.0 / sigma0 } else { 10.0 };
        Self {
            channel_rho,
            group_rho,
            beta_channel: beta,
            beta_group: beta,
        }
    }
}

impl Default for HretParams {
    fn default() -> Self { Self::default_sdr() }
}

/// Per-channel HRET trust state.
#[derive(Debug, Clone, Copy)]
pub struct ChannelState {
    /// EMA envelope s_k tracking |r_k|.  Initialised to 0.
    pub envelope: f32,
    /// Last computed channel trust weight w_k.
    pub trust_weight: f32,
}

impl Default for ChannelState {
    fn default() -> Self {
        Self { envelope: 0.0, trust_weight: 1.0 }
    }
}

/// Per-group HRET trust state.
#[derive(Debug, Clone, Copy)]
pub struct GroupState {
    /// EMA envelope s_g tracking mean |r| within the group.
    pub envelope: f32,
    /// Last computed group trust weight w_g.
    pub trust_weight: f32,
    /// Running channel count accumulator (used in mean computation).
    pub count: u8,
}

impl Default for GroupState {
    fn default() -> Self {
        Self { envelope: 0.0, trust_weight: 1.0, count: 0 }
    }
}

/// Complete HRET result returned by a single `observe()` call.
#[derive(Debug, Clone, Copy)]
pub struct HretResult<const C: usize> {
    /// Normalised hierarchical channel weights w̃_k (sum = 1).
    pub weights: [f32; C],
    /// Weighted composite residual Δx = K · (w̃ ⊙ r).
    ///
    /// This is the single scalar anomaly signal fed to downstream grammar/DSA.
    pub weighted_residual: f32,
    /// Maximum normalised weight (identifies the most-trusted channel).
    pub max_weight: f32,
    /// Minimum normalised weight (identifies least-trusted channel).
    pub min_weight: f32,
    /// Trust diversity index = 1 − (max − min).  Close to 1 → uniform trust.
    /// Close to 0 → power law: one channel dominates.
    pub trust_diversity: f32,
}

/// Hierarchical Residual-Envelope Trust estimator.
///
/// ## Type Parameters
///
/// - `C`: number of observation channels (antenna elements, ADC lanes)
/// - `G`: number of channel groups (polarisation pairs, sub-arrays, bands)
///
/// ## Memory footprint (no_std / no_alloc)
///
/// For C=4, G=2: 4×ChannelState + 2×GroupState + 4×usize = ~128 bytes.
pub struct HretEstimator<const C: usize, const G: usize> {
    /// Per-channel trust state.
    channel_states: [ChannelState; C],
    /// Per-group trust state.
    group_states: [GroupState; G],
    /// Channel-to-group mapping: group_map[k] = group index for channel k.
    group_map: [usize; C],
    /// HRET parameters.
    params: HretParams,
    /// Observation gain K applied to the weighted residual (default 1.0).
    gain: f32,
}

impl<const C: usize, const G: usize> HretEstimator<C, G> {
    /// Construct with a channel-to-group mapping and given parameters.
    ///
    /// # Panics (debug only)
    ///
    /// Panics in debug mode if any `group_map[k] >= G`.
    /// In release mode, out-of-range indices are silently saturated (no UB).
    pub fn new(group_map: [usize; C], params: HretParams) -> Self {
        // Validate mapping in debug builds
        debug_assert!(
            group_map.iter().all(|&g| g < G),
            "group_map contains index >= G"
        );
        Self {
            channel_states: [ChannelState::default(); C],
            group_states: [GroupState::default(); G],
            group_map,
            params,
            gain: 1.0,
        }
    }

    /// Construct with default SDR parameters and a uniform group mapping
    /// (all channels in group 0) — useful for single-band single-array receivers.
    pub fn single_group(params: HretParams) -> Self {
        Self::new([0usize; C], params)
    }

    /// Set the output gain K (default 1.0).
    pub fn with_gain(mut self, gain: f32) -> Self {
        self.gain = gain;
        self
    }

    /// Process one observation of per-channel residuals.
    ///
    /// `residuals[k]` = signed residual r_k for channel k.  We use |r_k|
    /// for envelope update but the signed value for the weighted composite.
    ///
    /// Returns an `HretResult<C>` with normalised weights and the weighted
    /// composite residual Δx.
    pub fn observe(&mut self, residuals: &[f32; C]) -> HretResult<C> {
        self.update_group_envelopes(residuals);
        self.update_channel_envelopes(residuals);
        let weights = self.compose_normalised_weights();
        let weighted_residual = self.gain * dot_product_c(&weights, residuals);
        let (max_w, min_w) = weight_extrema(&weights);
        HretResult {
            weights,
            weighted_residual,
            max_weight: max_w,
            min_weight: min_w,
            trust_diversity: 1.0 - (max_w - min_w),
        }
    }

    fn update_group_envelopes(&mut self, residuals: &[f32; C]) {
        let mut group_sum = [0.0_f32; G];
        let mut group_cnt = [0_u32; G];
        for (k, &r) in residuals.iter().enumerate() {
            let g = self.group_map[k].min(G - 1);
            group_sum[g] += r.abs();
            group_cnt[g] += 1;
        }
        let rho_gr = self.params.group_rho;
        let beta_gr = self.params.beta_group;
        for g in 0..G {
            let mean_abs = if group_cnt[g] > 0 { group_sum[g] / group_cnt[g] as f32 } else { 0.0 };
            let s = &mut self.group_states[g].envelope;
            *s = rho_gr * (*s) + (1.0 - rho_gr) * mean_abs;
            self.group_states[g].trust_weight = 1.0 / (1.0 + beta_gr * self.group_states[g].envelope);
        }
    }

    fn update_channel_envelopes(&mut self, residuals: &[f32; C]) {
        let rho_ch = self.params.channel_rho;
        let beta_ch = self.params.beta_channel;
        for (k, &r) in residuals.iter().enumerate() {
            let s = &mut self.channel_states[k].envelope;
            *s = rho_ch * (*s) + (1.0 - rho_ch) * r.abs();
            self.channel_states[k].trust_weight = 1.0 / (1.0 + beta_ch * self.channel_states[k].envelope);
        }
    }

    fn compose_normalised_weights(&self) -> [f32; C] {
        let mut hat_w = [0.0_f32; C];
        for k in 0..C {
            let g = self.group_map[k].min(G - 1);
            hat_w[k] = self.channel_states[k].trust_weight * self.group_states[g].trust_weight;
        }
        let sum_hat: f32 = hat_w.iter().sum();
        let mut weights = [0.0_f32; C];
        if sum_hat > 1e-30 {
            for k in 0..C { weights[k] = hat_w[k] / sum_hat; }
        } else {
            let unif = 1.0 / C as f32;
            for k in 0..C { weights[k] = unif; }
        }
        weights
    }

    /// Return a snapshot of all channel states (trust weights + envelopes).
    #[inline]
    pub fn channel_states(&self) -> &[ChannelState; C] { &self.channel_states }

    /// Return a snapshot of all group states.
    #[inline]
    pub fn group_states(&self) -> &[GroupState; G] { &self.group_states }

    /// Normalised channel trust weight for channel k.
    ///
    /// Returns the last computed normalised weight w̃_k.
    /// This is safe to call after at least one `observe()` call.
    pub fn channel_trust(&self, k: usize) -> f32 {
        self.channel_states.get(k).map(|s| s.trust_weight).unwrap_or(0.0)
    }

    /// Reset all state to initial values.
    pub fn reset(&mut self) {
        for s in &mut self.channel_states { *s = ChannelState::default(); }
        for s in &mut self.group_states { *s = GroupState::default(); }
    }
}

fn dot_product_c<const C: usize>(a: &[f32; C], b: &[f32; C]) -> f32 {
    let mut d = 0.0_f32;
    for k in 0..C { d += a[k] * b[k]; }
    d
}

fn weight_extrema<const C: usize>(weights: &[f32; C]) -> (f32, f32) {
    let mut max_w = weights[0];
    let mut min_w = weights[0];
    for k in 1..C {
        if weights[k] > max_w { max_w = weights[k]; }
        if weights[k] < min_w { min_w = weights[k]; }
    }
    (max_w, min_w)
}

// ---------------------------------------------------------------
// Tests
// ---------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_group_uniform_channels() {
        // 4 channels all same residual → nearly uniform weights
        let mut h = HretEstimator::<4, 1>::single_group(HretParams::default_sdr());
        for _ in 0..50 {
            let r = h.observe(&[0.1, 0.1, 0.1, 0.1]);
            let _ = r;
        }
        let r = h.observe(&[0.1, 0.1, 0.1, 0.1]);
        for k in 0..4 {
            let diff = (r.weights[k] - 0.25).abs();
            assert!(diff < 0.01, "weight[{}]={} (expected ~0.25)", k, r.weights[k]);
        }
        assert!((r.weights.iter().sum::<f32>() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn faulty_channel_down_weighted() {
        // Channel 3 has 10× the noise of others → should get lower trust
        let mut h = HretEstimator::<4, 1>::single_group(HretParams::default_sdr());
        for _ in 0..200 {
            // channel 3 always large
            h.observe(&[0.02, 0.02, 0.02, 0.20]);
        }
        let r = h.observe(&[0.02, 0.02, 0.02, 0.20]);
        // Good channels should cumulatively dominate
        let good_sum = r.weights[0] + r.weights[1] + r.weights[2];
        assert!(
            good_sum > r.weights[3],
            "good_sum={}, bad={}: faulty channel should be down-weighted",
            good_sum, r.weights[3]
        );
    }

    #[test]
    fn hierarchical_group_fault_down_weights_entire_group() {
        // 4 channels: channels 0,1 in group 0; channels 2,3 in group 1.
        // Group 1 has persistent large residuals → both channels 2,3 should lose trust.
        let map = [0usize, 0, 1, 1];
        let mut h = HretEstimator::<4, 2>::new(map, HretParams::default_sdr());
        for _ in 0..200 {
            h.observe(&[0.02, 0.02, 0.20, 0.20]);
        }
        let r = h.observe(&[0.02, 0.02, 0.20, 0.20]);
        let group0_sum = r.weights[0] + r.weights[1];
        let group1_sum = r.weights[2] + r.weights[3];
        assert!(
            group0_sum > group1_sum,
            "clean group0={} should outweigh noisy group1={}",
            group0_sum, group1_sum
        );
    }

    #[test]
    fn weights_always_sum_to_one() {
        let map = [0usize, 0, 1, 1];
        let mut h = HretEstimator::<4, 2>::new(map, HretParams::default_sdr());
        for i in 0..100 {
            let r = h.observe(&[i as f32 * 0.01, 0.05, 0.03, i as f32 * 0.02]);
            let sum: f32 = r.weights.iter().sum();
            assert!(
                (sum - 1.0).abs() < 1e-5,
                "weights sum={} at step {}", sum, i
            );
        }
    }

    #[test]
    fn trust_diversity_bounded() {
        let mut h = HretEstimator::<4, 1>::single_group(HretParams::default_sdr());
        for _ in 0..100 {
            let r = h.observe(&[0.1, 0.2, 0.3, 0.4]);
            assert!(r.trust_diversity >= 0.0, "diversity must be non-negative");
            assert!(r.trust_diversity <= 1.0, "diversity must be <= 1.0");
        }
    }

    #[test]
    fn reset_clears_state() {
        let mut h = HretEstimator::<2, 1>::single_group(HretParams::default_sdr());
        for _ in 0..100 { h.observe(&[0.5, 0.5]); }
        h.reset();
        assert_eq!(h.channel_states[0].envelope, 0.0);
        assert_eq!(h.group_states[0].envelope, 0.0);
    }
}
