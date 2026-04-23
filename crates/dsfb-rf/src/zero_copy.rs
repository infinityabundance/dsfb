//! Zero-copy residual source trait for DMA buffer integration.
//!
//! ## Motivation
//!
//! On embedded RF platforms (Zynq UltraScale+ RFSoC, USRP E310), the IQ
//! sample stream arrives in DMA buffers mapped into the processor's address
//! space. Copying these samples into an intermediate buffer adds latency
//! and CPU overhead that is unacceptable in high-throughput pipelines.
//!
//! The `ResidualSource` trait allows the DSFB engine to tap directly into
//! a DMA buffer — or any other memory-mapped IQ source — without copying.
//! The engine reads residual norms from the source via an immutable borrow,
//! maintaining the non-intrusion contract (no write path, no mutation of
//! upstream data).
//!
//! ## Non-Intrusion Guarantee
//!
//! The trait requires only `&self` access to the source. The engine never
//! takes a mutable reference to the DMA buffer. This is enforced at the
//! type level: `ResidualSource::residual_norms()` returns `&[f32]`, an
//! immutable slice. The Rust borrow checker prevents any write path from
//! being introduced without a compilation error.
//!
//! ## Design
//!
//! - `no_std`, `no_alloc`, zero `unsafe`
//! - Trait-based: implementors provide platform-specific DMA access
//! - Engine accepts `&dyn ResidualSource` or `&impl ResidualSource`
//! - Compatible with `volatile` memory-mapped I/O via safe wrappers

/// A source of residual norm observations.
///
/// Implementors provide access to a contiguous slice of f32 residual norms.
/// This is the zero-copy interface between the platform's IQ data path
/// and the DSFB engine.
///
/// ## Example: DMA Buffer Source
///
/// ```rust,ignore
/// struct DmaResidualSource {
///     buffer: &'static [f32],  // memory-mapped DMA region
///     len: usize,
/// }
///
/// impl ResidualSource for DmaResidualSource {
///     fn residual_norms(&self) -> &[f32] {
///         &self.buffer[..self.len]
///     }
///     fn snr_estimate_db(&self) -> f32 { 15.0 }
///     fn sample_count(&self) -> usize { self.len }
/// }
/// ```
pub trait ResidualSource {
    /// Borrow the current residual norm buffer as an immutable slice.
    ///
    /// This is the zero-copy tap point. The DSFB engine reads from this
    /// slice without copying. The source retains ownership.
    fn residual_norms(&self) -> &[f32];

    /// Current SNR estimate in dB. Return `f32::NAN` if unknown.
    fn snr_estimate_db(&self) -> f32;

    /// Number of valid samples in the current buffer.
    fn sample_count(&self) -> usize;
}

/// A simple owned-slice residual source for testing and host-side pipelines.
///
/// Wraps a `&[f32]` slice as a `ResidualSource`. Zero-copy: no allocation,
/// no copying — just a reference wrapper.
pub struct SliceSource<'a> {
    norms: &'a [f32],
    snr_db: f32,
}

impl<'a> SliceSource<'a> {
    /// Wrap an existing slice as a residual source.
    #[inline]
    pub const fn new(norms: &'a [f32], snr_db: f32) -> Self {
        Self { norms, snr_db }
    }
}

impl<'a> ResidualSource for SliceSource<'a> {
    #[inline]
    fn residual_norms(&self) -> &[f32] { self.norms }
    #[inline]
    fn snr_estimate_db(&self) -> f32 { self.snr_db }
    #[inline]
    fn sample_count(&self) -> usize { self.norms.len() }
}

/// A fixed-capacity ring buffer residual source for embedded/bare-metal.
///
/// Accepts streamed residual norms one at a time and exposes the most
/// recent N observations as a contiguous slice. All storage is stack-allocated.
pub struct RingSource<const N: usize> {
    buffer: [f32; N],
    head: usize,
    count: usize,
    snr_db: f32,
}

impl<const N: usize> RingSource<N> {
    /// Create a new ring source.
    pub const fn new(snr_db: f32) -> Self {
        Self {
            buffer: [0.0; N],
            head: 0,
            count: 0,
            snr_db,
        }
    }

    /// Push a new residual norm into the ring buffer.
    pub fn push(&mut self, norm: f32) {
        self.buffer[self.head] = norm;
        self.head = (self.head + 1) % N;
        if self.count < N { self.count += 1; }
    }

    /// Update the SNR estimate.
    pub fn set_snr_db(&mut self, snr_db: f32) {
        self.snr_db = snr_db;
    }
}

impl<const N: usize> ResidualSource for RingSource<N> {
    fn residual_norms(&self) -> &[f32] {
        // Return the filled portion of the buffer.
        // Note: for a ring buffer, the "most recent N" are not necessarily
        // contiguous from index 0. We return the full filled buffer — the
        // engine processes observations sequentially via observe().
        &self.buffer[..self.count.min(N)]
    }

    fn snr_estimate_db(&self) -> f32 { self.snr_db }
    fn sample_count(&self) -> usize { self.count.min(N) }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice_source_returns_original_data() {
        let data = [0.1_f32, 0.2, 0.3, 0.4];
        let src = SliceSource::new(&data, 15.0);
        assert_eq!(src.residual_norms(), &data);
        assert_eq!(src.snr_estimate_db(), 15.0);
        assert_eq!(src.sample_count(), 4);
    }

    #[test]
    fn ring_source_accumulates() {
        let mut ring = RingSource::<4>::new(10.0);
        ring.push(0.1);
        ring.push(0.2);
        assert_eq!(ring.sample_count(), 2);
        ring.push(0.3);
        ring.push(0.4);
        ring.push(0.5); // overwrites oldest
        assert_eq!(ring.sample_count(), 4);
    }

    #[test]
    fn ring_source_is_residual_source() {
        let mut ring = RingSource::<8>::new(20.0);
        for i in 0..5 {
            ring.push(i as f32 * 0.1);
        }
        let src: &dyn ResidualSource = &ring;
        assert_eq!(src.sample_count(), 5);
        assert_eq!(src.snr_estimate_db(), 20.0);
        assert_eq!(src.residual_norms().len(), 5);
    }

    #[test]
    fn zero_copy_no_allocation() {
        // This test verifies the zero-copy property: the SliceSource
        // wraps an existing slice without any allocation or copying.
        let original = [1.0_f32, 2.0, 3.0];
        let src = SliceSource::new(&original, 0.0);
        let borrowed = src.residual_norms();
        // The pointers should be identical (same memory)
        assert_eq!(borrowed.as_ptr(), original.as_ptr());
    }
}
