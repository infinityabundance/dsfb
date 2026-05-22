//! R.6a scaffolding: a small owning wrapper for `cudaMallocHost`-
//! allocated (page-locked) host memory.
//!
//! Why this exists separately from `Vec<T>`: page-locked memory must
//! be freed with `cudaFreeHost`, not the global allocator, so we
//! cannot just hand a `Vec<T>` constructed from `cudaMallocHost`
//! storage back to the standard library. `PinnedHostBuf<T>` owns the
//! raw pointer + length + capacity, exposes the same `&[T]` /
//! `&mut [T]` / `as_ptr()` / `as_mut_ptr()` surface the rest of the
//! crate uses, and frees through `dsfb_gpu_free_pinned_bytes` in its
//! `Drop`.
//!
//! Determinism note: pinned-vs-pageable is a memory-class hint to
//! the CUDA runtime; it changes how the runtime stages H2D/D2H
//! traffic but **not** what bytes the kernel sees. A `PinnedHostBuf<T>`
//! is byte-equivalent to a `Vec<T>` of the same contents at every
//! point a kernel could observe. The R.6b byte-equivalence
//! acceptance test will exercise this directly once the workspace
//! consumes pinned buffers in its async dispatch path.
//!
//! Safety notes:
//! * `cudaMallocHost(0, ...)` is well-defined and returns null on
//!   modern CUDA; we treat zero-sized requests as no-ops and store a
//!   null pointer with `len = 0`.
//! * `T: Copy` is required so the wrapper does not have to run
//!   destructors over the buffer at drop time. Pinned memory is for
//!   pipeline cell types (`WindowFeature`, `CandidateInterval`,
//!   `u8` digest scratch, `i32` counters) — all of which are `Copy`.
//! * The wrapper is `!Send` / `!Sync` because `cudaMallocHost`
//!   memory is associated with the CUDA context that allocated it.
//!   This is enforced indirectly by the raw `*mut T` field.

#![cfg(feature = "cuda")]

use core::ffi::c_int;
use core::marker::PhantomData;
use core::mem::size_of;
use core::ops::{Deref, DerefMut};

use crate::ffi;
use crate::GpuError;

/// Owning wrapper for a `cudaMallocHost`-allocated slice of `T`.
///
/// The slice has fixed length set at construction; growth is not
/// supported (pipeline buffers are sized exactly once per workspace
/// and reused). All elements are initialised to `T::default()` at
/// construction so the slice is safe to read before the first
/// pipeline dispatch writes into it.
///
/// `T` must be both `Copy` and `Default` so the wrapper can fill the
/// allocation without running per-element constructors and can hand
/// out borrowed slices without owning destructors. Both invariants
/// hold for every `#[repr(C)]` cell type in this crate.
pub struct PinnedHostBuf<T: Copy + Default> {
    ptr: *mut T,
    len: usize,
    _marker: PhantomData<T>,
}

impl<T: Copy + Default> PinnedHostBuf<T> {
    /// Allocate a pinned host buffer holding `len` `T`s, all
    /// initialised to `T::default()`.
    ///
    /// # Errors
    ///
    /// * `GpuError::KernelFailed(code)` if `cudaMallocHost` returns a
    ///   non-zero `cudaError_t`. The wrapper does not retain a
    ///   half-initialised allocation; `*ptr` is null in that case.
    pub fn new(len: usize) -> Result<Self, GpuError> {
        if len == 0 {
            return Ok(Self {
                ptr: core::ptr::null_mut(),
                len: 0,
                _marker: PhantomData,
            });
        }
        let size = (len as u64) * (size_of::<T>() as u64);
        let mut raw: *mut u8 = core::ptr::null_mut();
        #[allow(unsafe_code)]
        let status: c_int = unsafe { ffi::dsfb_gpu_alloc_pinned_bytes(size, &mut raw) };
        if status != 0 || raw.is_null() {
            return Err(GpuError::KernelFailed(status));
        }
        let ptr = raw.cast::<T>();
        // Initialise the allocation: every cell type in the pipeline
        // is `Copy + Default`, so a plain element-by-element write
        // is safe and avoids handing a future dispatch a buffer with
        // uninitialised bytes (which would surface as nondeterminism
        // if the dispatcher ever read before writing).
        #[allow(unsafe_code)]
        unsafe {
            for i in 0..len {
                ptr.add(i).write(T::default());
            }
        }
        Ok(Self {
            ptr,
            len,
            _marker: PhantomData,
        })
    }

    /// Length of the buffer in `T` elements.
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// `true` if the buffer holds zero elements.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Raw pointer for FFI handoff. Always non-null when `len > 0`.
    #[must_use]
    pub fn as_ptr(&self) -> *const T {
        self.ptr
    }

    /// Mutable raw pointer for FFI handoff.
    #[must_use]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.ptr
    }

    /// Immutable slice view. Empty when `len == 0`.
    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        if self.len == 0 {
            &[]
        } else {
            // Safety: `ptr` and `len` come from `Self::new`, which
            // initialises every element via `T::default()` before
            // returning. The pointer remains valid for the wrapper's
            // lifetime and is freed in `Drop` (not before).
            #[allow(unsafe_code)]
            unsafe {
                core::slice::from_raw_parts(self.ptr, self.len)
            }
        }
    }

    /// Mutable slice view. Empty when `len == 0`.
    #[must_use]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        if self.len == 0 {
            &mut []
        } else {
            #[allow(unsafe_code)]
            unsafe {
                core::slice::from_raw_parts_mut(self.ptr, self.len)
            }
        }
    }
}

impl<T: Copy + Default> Deref for PinnedHostBuf<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T: Copy + Default> DerefMut for PinnedHostBuf<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T: Copy + Default> Drop for PinnedHostBuf<T> {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            // Safety: `ptr` was returned by
            // `dsfb_gpu_alloc_pinned_bytes` and has not been freed
            // elsewhere. `cudaFreeHost` accepts a null pointer
            // (returns success); we still gate on non-null for
            // clarity. The status is intentionally ignored — there
            // is nothing useful to do with a free-time error during
            // `Drop`, mirroring the existing `dsfb_gpu_workspace_free`
            // pattern.
            #[allow(unsafe_code)]
            unsafe {
                let _ = ffi::dsfb_gpu_free_pinned_bytes(self.ptr.cast::<u8>());
            }
            self.ptr = core::ptr::null_mut();
            self.len = 0;
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    /// R.6a smoke test: allocate a small pinned buffer, write a
    /// recognisable pattern, read it back. Catches any FFI-level
    /// breakage between `cudaMallocHost` / pointer hand-off /
    /// element access / `cudaFreeHost`.
    #[test]
    fn pinned_host_buf_round_trip_u32() {
        let mut buf: PinnedHostBuf<u32> = PinnedHostBuf::new(64).expect("allocate pinned");
        assert_eq!(buf.len(), 64);
        assert!(!buf.is_empty());
        for (i, slot) in buf.as_mut_slice().iter_mut().enumerate() {
            *slot = (i as u32) * 7 + 13;
        }
        for (i, value) in buf.as_slice().iter().enumerate() {
            assert_eq!(*value, (i as u32) * 7 + 13);
        }
        // Drop happens at end of scope; cudaFreeHost is the only way
        // to release the page-locked allocation safely.
    }

    /// Zero-length construction is a documented no-op: returns a
    /// wrapper with a null pointer and zero length, no FFI call.
    #[test]
    fn pinned_host_buf_zero_length_is_empty() {
        let buf: PinnedHostBuf<u32> = PinnedHostBuf::new(0).expect("zero-length allocate");
        assert_eq!(buf.len(), 0);
        assert!(buf.is_empty());
        assert!(buf.as_ptr().is_null());
        assert_eq!(buf.as_slice(), &[] as &[u32]);
    }
}
