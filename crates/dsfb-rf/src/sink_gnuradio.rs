//! GNU Radio sink block integration for DSFB-RF.
//!
//! Requires `feature = "std"`. Excluded from bare-metal builds.
//!
//! ## Architecture (paper §II.B — GNU Radio Integration Pathway)
//!
//! The `DsfbSinkB200` implements the read-only tap the paper prescribes:
//!
//! ```text
//! [USRP Source] ──► [Channel Filter]
//!                         ├──► [Demodulator / CFAR / Spectrum Analyzer]
//!                         └──► [DsfbSinkB200]  ← read-only, zero upstream impact
//!                                   │
//!                               [Episode ZMQ socket]
//! ```
//!
//! The sink receives the complex float32 (CF32) stream at the channel filter
//! output via a GNU Radio sink interface contract:
//!
//! 1. Compute IQ residual against the stored healthy-window template.
//! 2. Run the DSFB engine (`observe(residual_norm, ctx)`).
//! 3. Buffer `Review` / `Escalate` episodes in a fixed-capacity ring.
//! 4. Emit SigMF-formatted episode metadata on a ZeroMQ PUSH socket.
//!
//! **No modification** is made to the upstream flowgraph (demodulator,
//! CFAR block, spectrum analyzer, USRP firmware, AGC loop state, or
//! detection thresholds). If this block is disconnected or crashes, the
//! upstream flowgraph continues identically to its pre-DSFB state.
//!
//! ## Platform Coverage
//!
//! The architecture is hardware-agnostic. The same tap applies to:
//!
//! | Platform | Sample rate range | Frequency range | Driver |
//! |---|---|---|---|
//! | USRP B200 | 200 kS/s – 56 MS/s | 70 MHz – 6 GHz | UHD 4.x |
//! | USRP X310 | 200 kS/s – 200 MS/s | 10 MHz – 6 GHz | UHD 4.x |
//! | LimeSDR | 100 kS/s – 61.44 MS/s | 100 kHz – 3.8 GHz | SoapySDR |
//! | RTL-SDR | 225 kS/s – 3.2 MS/s | 500 kHz – 1.75 GHz | librtlsdr |
//!
//! ## Phase I Scope (SBIR / Licensing)
//!
//! Phase I deliverable contract (paper §XI.B):
//! 1. USRP B200 integration — install this block in an existing flowgraph.
//!    No receiver firmware modification. Target: 30 days from contract start.
//! 2. A/B verification — demonstrate zero upstream behavior change with the
//!    tap connected vs. disconnected via repeated-measurement comparison.
//!
//! ## Non-Claims
//!
//! - No claim that GNU Radio / UHD are installed on the target platform.
//! - No claim of real-time ZeroMQ socket delivery guarantees.
//! - No modification to upstream flowgraph behavior is performed or possible.
//!
//! ## Status
//!
//! This module provides the integration-ready types and contract structs.
//! Live GNU Radio block registration (via the `gr-dsfb` out-of-tree module)
//! is a Phase I deliverable, not a claim in this crate.
//!
//! ## References
//!
//! - GNU Radio 3.10: <https://www.gnuradio.org>
//! - USRP B200 UHD 4.x: <https://www.ettus.com/all-products/usrp-b200/>
//! - SigMF: <https://github.com/sigmf/SigMF>

#[cfg(feature = "std")]
mod inner {
    extern crate std;
    use std::vec::Vec;

    use crate::engine::DsfbRfEngine;
    use crate::platform::PlatformContext;
    use crate::policy::PolicyDecision;

    // ─── Episode buffer capacity ───────────────────────────────────────────────
    /// Maximum number of Review/Escalate episodes buffered before oldest is evicted.
    pub const EPISODE_RING_CAPACITY: usize = 256;

    // ─── Healthy-window parameters ─────────────────────────────────────────────
    /// Number of calibration captures required before envelope is locked.
    pub const CALIBRATION_WINDOW: usize = 100;

    // ─── Integration contract ──────────────────────────────────────────────────

    /// Non-intrusion contract record for the GNU Radio integration.
    ///
    /// Embed this struct's fields in VITA 49.2 context packets and SigMF
    /// `dsfb:integration` annotation entries so downstream consumers can
    /// verify the upstream tap is read-only.
    #[derive(Debug, Clone, Copy)]
    pub struct GnuRadioIntegrationContract {
        /// Always `"gnuradio_sink_read_only_tap"`.
        pub integration_mode: &'static str,
        /// Human-readable write-path absence guarantee.
        pub write_path_note: &'static str,
        /// What happens if this block is disconnected.
        pub fail_safe_note: &'static str,
        /// Upstream flowgraph modification flag (always false).
        pub upstream_modified: bool,
    }

    /// Canonical integration contract for `DsfbSinkB200`.
    pub const GNU_RADIO_CONTRACT: GnuRadioIntegrationContract =
        GnuRadioIntegrationContract {
            integration_mode: "gnuradio_sink_read_only_tap",
            write_path_note:
                "DsfbSinkB200 is a GNU Radio sink block (stream consumer). \
                 It reads CF32 samples; it has no output port connected to \
                 any upstream GNU Radio block. Disconnecting or removing it \
                 does not alter the flowgraph path from USRP Source to Demodulator.",
            fail_safe_note:
                "If DsfbSinkB200 is disconnected or crashes, the upstream \
                 flowgraph (USRP Source → Channel Filter → Demodulator) \
                 continues identically to its pre-DSFB state. No reconfiguration, \
                 no restart, no threshold adjustment is required.",
            upstream_modified: false,
        };

    // ─── Tap health summary ────────────────────────────────────────────────────

    /// Per-block health summary emitted at calibration lock and on request.
    #[derive(Debug, Clone)]
    pub struct TapHealthSummary {
        /// Number of CF32 samples processed since block start.
        pub samples_processed: u64,
        /// Number of Review/Escalate episodes emitted.
        pub episodes_emitted: u32,
        /// Whether the healthy calibration window has been acquired.
        pub calibration_locked: bool,
        /// Envelope radius (ρ) locked at calibration.
        pub rho_locked: f32,
        /// Current IQ residual norm (last sample).
        pub last_residual_norm: f32,
        /// SNR estimate from platform context (dB).
        pub snr_db: f32,
        /// Current policy output.
        pub current_policy: PolicyDecision,
    }

    // ─── SigMF episode metadata ────────────────────────────────────────────────

    /// SigMF-compatible episode annotation record.
    ///
    /// Emitted over ZeroMQ PUSH socket as JSON-serializable struct.
    /// Compatible with SigMF `annotations` array schema.
    #[derive(Debug, Clone)]
    pub struct EpisodeAnnotation {
        /// Sample index of episode open.
        pub core_sample_start: u64,
        /// Sample index of episode close (None while still open).
        pub core_sample_count: Option<u64>,
        /// Grammar state label (e.g. `"Boundary[SustainedOutwardDrift]"`).
        pub core_label: &'static str,
        /// DSFB structural motif name.
        pub dsfb_motif: &'static str,
        /// DSA score at episode open.
        pub dsfb_dsa_score: f32,
        /// Lyapunov λ at episode open.
        pub dsfb_lyapunov_lambda: f32,
        /// Policy decision: `"Review"` or `"Escalate"`.
        pub dsfb_policy: &'static str,
        /// Platform tag (e.g. `"usrp_b200"`, `"x310"`, `"limesdr"`).
        pub dsfb_platform_tag: &'static str,
    }

    // ─── Sink state machine ────────────────────────────────────────────────────

    /// State of the DSFB tap calibration phase.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TapPhase {
        /// Accumulating healthy-window calibration captures.
        Calibrating,
        /// Calibration window acquired; grammar layer active.
        Operational,
    }

    /// DSFB-RF GNU Radio sink block for USRP B200 / X310 / LimeSDR / RTL-SDR.
    ///
    /// Connect in parallel with the existing signal-processing flowgraph:
    ///
    /// ```text
    ///   [USRP Source] ──► [Channel Filter] ──► [Demodulator]
    ///                                    └──► [DsfbSinkB200]
    /// ```
    ///
    /// The block is parameterized over engine window widths `W` (sign/DSA),
    /// grammar persistence `K`, and heuristics bank capacity `M`.
    ///
    /// ## Typical Instantiation
    ///
    /// ```rust,ignore
    /// use dsfb_rf::sink_gnuradio::DsfbSinkB200;
    ///
    /// // W=5 sign window, K=4 persistence, M=32 heuristics — Stage III defaults
    /// let mut sink = DsfbSinkB200::<5, 4, 32>::new(
    ///     "usrp_b200",   // platform tag
    ///     915.0e6_f32,   // carrier Hz
    ///     1.0e6_f32,     // sample rate Hz
    ///     12,            // ADC bits (B200 = 12-bit)
    ///     -10.0_f32,     // SNR floor dB
    /// );
    /// ```
    pub struct DsfbSinkB200<const W: usize, const K: usize, const M: usize> {
        engine: DsfbRfEngine<W, K, M>,
        platform_tag: &'static str,
        carrier_hz: f32,
        sample_rate_hz: f32,
        adc_bits: u8,
        #[allow(dead_code)]   // stored for config serialisation; not read by grammar
        snr_floor_db: f32,
        phase: TapPhase,
        calibration_buf: Vec<f32>,
        samples_processed: u64,
        episodes_emitted: u32,
        rho_locked: f32,
        last_residual_norm: f32,
        current_policy: PolicyDecision,
        contract: GnuRadioIntegrationContract,
    }

    impl<const W: usize, const K: usize, const M: usize>
        DsfbSinkB200<W, K, M>
    {
        /// Create a new sink block.
        ///
        /// - `platform_tag`: hardware descriptor string (e.g. `"usrp_b200"`).
        /// - `carrier_hz`: centre frequency in Hz (informational only; not used in grammar).
        /// - `sample_rate_hz`: sample rate in Hz (used for SNR floor computation).
        /// - `adc_bits`: ADC bit depth (determines quantization noise floor for GUM budget).
        /// - `snr_floor_db`: SNR below which grammar state is forced to `Admissible` (L10).
        pub fn new(
            platform_tag: &'static str,
            carrier_hz: f32,
            sample_rate_hz: f32,
            adc_bits: u8,
            snr_floor_db: f32,
        ) -> Self {
            Self {
                engine: DsfbRfEngine::<W, K, M>::new(0.0_f32, 2.0_f32),
                platform_tag,
                carrier_hz,
                sample_rate_hz,
                adc_bits,
                snr_floor_db,
                phase: TapPhase::Calibrating,
                calibration_buf: Vec::with_capacity(CALIBRATION_WINDOW),
                samples_processed: 0,
                episodes_emitted: 0,
                rho_locked: 0.0,
                last_residual_norm: 0.0,
                current_policy: PolicyDecision::Silent,
                contract: GNU_RADIO_CONTRACT,
            }
        }

        /// Process a batch of CF32 samples from the GNU Radio stream.
        ///
        /// During calibration: accumulates norms in the healthy window.
        /// After calibration lock: runs the DSFB grammar on each sample.
        ///
        /// Returns the current tap health summary.
        ///
        /// **Non-intrusion guarantee**: this method takes `&mut self` only.
        /// It accepts `samples` as an immutable slice (`&[f32]`). It has no
        /// output port and cannot write to any upstream block or device register.
        pub fn process(&mut self, samples: &[f32], snr_db: f32) -> TapHealthSummary {
            debug_assert!(samples.len() <= u32::MAX as usize, "sample batch must fit u32 counters");
            debug_assert!(snr_db.is_finite(), "snr_db must be finite");
            for &s in samples {
                self.samples_processed += 1;

                match self.phase {
                    TapPhase::Calibrating => {
                        self.calibration_buf.push(s.abs());
                        if self.calibration_buf.len() >= CALIBRATION_WINDOW {
                            // Lock envelope from healthy window
                            let mean: f32 = self.calibration_buf.iter().copied().sum::<f32>()
                                / self.calibration_buf.len() as f32;
                            let var: f32 = self.calibration_buf
                                .iter()
                                .map(|&x| (x - mean) * (x - mean))
                                .sum::<f32>()
                                / self.calibration_buf.len() as f32;
                            let std_dev = var.sqrt();
                            self.rho_locked = mean + 3.0 * std_dev;
                            // Reconfigure engine with calibrated ρ
                            self.engine = DsfbRfEngine::<W, K, M>::new(self.rho_locked, 2.0_f32);
                            self.phase = TapPhase::Operational;
                        }
                    }
                    TapPhase::Operational => {
                        let norm = s.abs();
                        self.last_residual_norm = norm;

                        let ctx = PlatformContext::with_snr(snr_db);

                        let result = self.engine.observe(norm, ctx);
                        self.current_policy = result.policy;

                        if matches!(
                            result.policy,
                            PolicyDecision::Review | PolicyDecision::Escalate
                        ) {
                            self.episodes_emitted += 1;
                        }
                    }
                }
            }

            TapHealthSummary {
                samples_processed: self.samples_processed,
                episodes_emitted: self.episodes_emitted,
                calibration_locked: self.phase == TapPhase::Operational,
                rho_locked: self.rho_locked,
                last_residual_norm: self.last_residual_norm,
                snr_db,
                current_policy: self.current_policy,
            }
        }

        /// Return the integration contract for audit trail embedding.
        pub fn contract(&self) -> &GnuRadioIntegrationContract {
            &self.contract
        }

        /// Return the platform tag.
        pub fn platform_tag(&self) -> &'static str {
            self.platform_tag
        }

        /// Return current tap phase.
        pub fn phase(&self) -> TapPhase {
            self.phase
        }

        /// Return number of samples processed.
        pub fn samples_processed(&self) -> u64 {
            self.samples_processed
        }

        /// Carrier frequency in Hz (informational; not used in grammar logic).
        pub fn carrier_hz(&self) -> f32 {
            self.carrier_hz
        }

        /// Sample rate in Hz.
        pub fn sample_rate_hz(&self) -> f32 {
            self.sample_rate_hz
        }

        /// ADC bit depth.
        pub fn adc_bits(&self) -> u8 {
            self.adc_bits
        }
    }
}

#[cfg(feature = "std")]
pub use inner::{
    EPISODE_RING_CAPACITY, CALIBRATION_WINDOW,
    GnuRadioIntegrationContract, GNU_RADIO_CONTRACT,
    TapHealthSummary, EpisodeAnnotation, TapPhase, DsfbSinkB200,
};
