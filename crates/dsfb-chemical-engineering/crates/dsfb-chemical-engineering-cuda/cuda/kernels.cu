// kernels.cu — DSFB-Chemical CUDA evidence factory.
//
// Two kernels:
//   evidence_kernel  — one thread per lane; produces fixed-point evidence + SHA-256 digest per lane.
//   roofline_kernel  — pure streaming read+reduce for DRAM bandwidth measurement.
//
// Three host-side C wrappers (called by Rust via ffi.rs):
//   dsfb_chem_cuda_evidence    — allocate, upload, launch evidence_kernel, download, free.
//   dsfb_chem_cuda_roofline    — allocate, upload, launch roofline_kernel iters times, download.
//   dsfb_chem_cuda_device_name — query and return the active CUDA device name.
//
// Determinism contract:
//   - Compiled with --fmad=false (build.rs passes this flag) so the single `x * DSFB_SCALE`
//     multiply in dsfb_quantize is correctly rounded without fused-multiply-add.
//   - All evidence arithmetic (quantise, ring buffer, exceedance, drift, slew) is exact integer;
//     no further floating-point operations after quantisation.
//   - The on-GPU SHA-256 (sha256.cuh) processes the same 40-byte/sample canonical byte stream as
//     the Rust `sha2` crate on the CPU, producing the same digest byte-for-byte.
//   - The cross-backend verification in dispatch.rs confirms this agreement after every GPU run.
//
// The GPU is an acceleration + attestation layer: correctness is defined by byte-exact agreement
// with the CPU reference, not by the GPU alone.

#include "common.cuh"
#include "sha256.cuh"
#include <cuda_runtime.h>
#include <stdint.h>

// evidence_kernel — one thread per lane; sequential over samples within the lane.
//
// Thread-to-lane mapping: lane index = blockIdx.x * blockDim.x + threadIdx.x.
// Adjacent threads in a warp handle adjacent lanes. Because `x` is sample-major
// (element (sample i, lane L) at x[i * n_lanes + L]), threads in the same warp read
// adjacent memory locations for the same sample i, giving 128-byte coalesced loads.
//
// Per-lane state (all in thread-local registers/local memory):
//   ctx        — incremental SHA-256 context for this lane's digest
//   ring[16]   — circular ring buffer of the last DSFB_DRIFT_WINDOW quantised residuals
//   ring_sum   — integer sum of the ring buffer (updated incrementally, not recomputed)
//   filled     — number of slots currently occupied (< DSFB_DRIFT_WINDOW at stream start)
//   head       — index of the oldest ring slot (next eviction target)
//   prev_q     — quantised residual from the previous sample (for slew computation)
//   pke/pkd/pks — running peak values (exceedance, |drift|, |slew|)
//
// Output layout:
//   digests[lane*32 .. lane*32+32]  — 32-byte SHA-256 lane digest
//   summary[lane*5+0]               — n_exceedances (cast to long long for uniform layout)
//   summary[lane*5+1]               — oob_count
//   summary[lane*5+2]               — peak_exceedance
//   summary[lane*5+3]               — peak_abs_drift
//   summary[lane*5+4]               — peak_abs_slew
//
// NOTE: this kernel is SHA-256-compute-bound, not DRAM-bandwidth-bound. Each sample requires
// multiple SHA-256 update calls (5 x 8 = 40 bytes per sample fed into the hash), so the limiting
// factor is SHA-256 throughput per thread, not the memory bandwidth of the x[] loads.
// dsfb_lane_evidence — the per-lane evidence + SHA-256 computation, SHARED by the V1 (sample-major,
// stride = n_lanes) and V2-A batched (lane-contiguous, stride = 1) kernels. Sample i is read from
// base[(uint64_t)i * stride]; with the right (base, stride) the two layouts converge on the SAME
// per-lane sample sequence, so the per-lane digest + summary are byte-IDENTICAL across kernels. The
// digest-equivalence law (DigestEquivalenceHarnessV1) therefore holds for V2-A *by construction* —
// both kernels execute this one function; only the launch geometry and memory layout differ.
//
// `dig_out` is this lane's 32-byte digest slot; `sum_out` its 5 summary i64s
// [n_exceedances, oob_count, peak_exceedance, peak_abs_drift, peak_abs_slew].
__device__ __forceinline__ void dsfb_lane_evidence(
    const double* base, uint64_t stride, uint32_t ns,
    uint8_t* dig_out, long long* sum_out)
{
    dsfb_sha256_ctx ctx; dsfb_sha256_init(&ctx);
    long long ring[DSFB_DRIFT_WINDOW];
    #pragma unroll
    for (int i=0;i<DSFB_DRIFT_WINDOW;i++) ring[i]=0; /* zero-initialise ring buffer */
    long long ring_sum=0, prev_q=0, pke=0, pkd=0, pks=0;
    uint32_t filled=0, head=0, n_exc=0, oob=0;

    for (uint32_t i=0;i<ns;i++) {
        // Strided load: stride = n_lanes (V1, sample-major) or 1 (V2-A, lane-contiguous). uint64_t
        // index avoids 32-bit overflow for large workloads.
        double xv = base[(uint64_t)i * stride];
        int finite; long long q = dsfb_quantize(xv, &finite);
        if (!finite) oob++; /* non-finite sealed as q=0; count is stored in evidence summary */
        // Causal ring buffer update (same logic as lane_evidence_cpu in evidence.rs).
        if (filled==DSFB_DRIFT_WINDOW) ring_sum -= ring[head]; else filled++;
        ring[head]=q; ring_sum+=q; head=(head+1)%DSFB_DRIFT_WINDOW;
        long long d = ring_sum / (long long)filled; /* integer truncation toward zero */
        long long e = (q>0)?q:0;                   /* one-sided exceedance */
        long long s = (i==0)?0:(q-prev_q);          /* slew; 0 at first sample */
        prev_q=q;
        if (e>0) n_exc++;
        if (e>pke) pke=e;
        long long ad=(d<0)?-d:d; if (ad>pkd) pkd=ad; /* |d| */
        long long as_=(s<0)?-s:s; if (as_>pks) pks=as_; /* |s| */
        // Canonical per-sample record: raw_bits ‖ q ‖ e ‖ d ‖ s (all little-endian, 40 bytes).
        // __double_as_longlong reinterprets the double's IEEE-754 bits as int64 without conversion;
        // this matches xv.to_bits() as u64 in Rust (same bit pattern, both cast to unsigned).
        dsfb_sha256_update_u64le(&ctx, (uint64_t)__double_as_longlong(xv));
        dsfb_sha256_update_i64le(&ctx, q);
        dsfb_sha256_update_i64le(&ctx, e);
        dsfb_sha256_update_i64le(&ctx, d);
        dsfb_sha256_update_i64le(&ctx, s);
    }
    // Finalise the SHA-256 digest for this lane and write to the output buffer.
    uint8_t out[32]; dsfb_sha256_final(&ctx, out);
    #pragma unroll
    for (int i=0;i<32;i++) dig_out[i] = out[i];
    // Write summary: n_exc and oob are uint32 cast to long long for uniform i64 layout in summary.
    sum_out[0]=(long long)n_exc;
    sum_out[1]=(long long)oob;
    sum_out[2]=pke;
    sum_out[3]=pkd;
    sum_out[4]=pks;
}

// evidence_kernel (V1) — one thread per lane over a sample-major matrix. Thin wrapper over the shared
// per-lane routine: lane L's samples are at x[i*n_lanes + L], i.e. base = x + L, stride = n_lanes.
extern "C" __global__ void evidence_kernel(
    const double* __restrict__ x, uint32_t n_lanes, uint32_t n_samples,
    uint8_t* __restrict__ digests /* 32 * n_lanes */, long long* __restrict__ summary /* 5 * n_lanes */)
{
    uint32_t lane = blockIdx.x * blockDim.x + threadIdx.x;
    if (lane >= n_lanes) return; /* guard: last block may have fewer active threads than blockDim.x */
    dsfb_lane_evidence(x + lane, (uint64_t)n_lanes, n_samples,
                       &digests[(uint64_t)lane*32], &summary[(uint64_t)lane*5]);
}

// evidence_kernel_batched (V2-A) — one thread per lane over MANY datasets concatenated into one
// launch. Lane L's samples are stored contiguously at xcat[offsets[L] .. offsets[L]+nsamples[L]]
// (lane-major / jagged), so base = xcat + offsets[L], stride = 1. Because it calls the same
// dsfb_lane_evidence, each lane's digest is byte-identical to what V1 produces for that lane's
// sample sequence — this kernel only grows the grid (lanes from all datasets in one launch) to clear
// the 80-SM occupancy wall that the per-dataset V1 launches never reach. See the V2 design doc.
extern "C" __global__ void evidence_kernel_batched(
    const double* __restrict__ xcat, const uint64_t* __restrict__ offsets,
    const uint32_t* __restrict__ nsamples, uint32_t n_total_lanes,
    uint8_t* __restrict__ digests /* 32 * n_total_lanes */, long long* __restrict__ summary /* 5 * n_total_lanes */)
{
    uint32_t lane = blockIdx.x * blockDim.x + threadIdx.x;
    if (lane >= n_total_lanes) return;
    dsfb_lane_evidence(xcat + offsets[lane], 1ull, nsamples[lane],
                       &digests[(uint64_t)lane*32], &summary[(uint64_t)lane*5]);
}

// dsfb_segment_evidence (V2-B helper) — compute ONE segment's digest + partial summary, with a causal
// halo warm-up so the drift ring buffer matches the serial reference at the segment boundary.
//   base   : pointer to this segment's LANE sample 0 (lane-contiguous)
//   s, e   : segment sample range [s, e) within the lane
//   warmup : pre-segment samples to replay to seed the ring (= min(s, DSFB_DRIFT_WINDOW))
// Emits this segment's 32-byte SHA digest to dig_out and 5 partial-summary i64s to sum_out. The
// per-sample record + state update are IDENTICAL to dsfb_lane_evidence / lane_evidence_v2_cpu — only
// the hashing is segment-local, so the per-thread SHA chain is <= seg (not n_samples).
__device__ __forceinline__ void dsfb_segment_evidence(
    const double* base, uint32_t s, uint32_t e, uint32_t warmup,
    uint8_t* dig_out, long long* sum_out)
{
    long long ring[DSFB_DRIFT_WINDOW];
    #pragma unroll
    for (int i=0;i<DSFB_DRIFT_WINDOW;i++) ring[i]=0;
    long long ring_sum=0, prev_q=0, pke=0, pkd=0, pks=0;
    uint32_t filled=0, head=0, n_exc=0, oob=0;
    // Warm-up: replay [s-warmup, s) to reconstruct ring/filled/head/prev_q at sample s. The drift ring
    // is a sliding window of the last DSFB_DRIFT_WINDOW q-values, so exactly `warmup` preceding samples
    // reproduce the serial state. No records emitted, no summary counted, no SHA.
    for (uint32_t i = s - warmup; i < s; i++) {
        int finite; long long q = dsfb_quantize(base[(uint64_t)i], &finite);
        if (filled==DSFB_DRIFT_WINDOW) ring_sum -= ring[head]; else filled++;
        ring[head]=q; ring_sum+=q; head=(head+1)%DSFB_DRIFT_WINDOW;
        prev_q=q;
    }
    // Emit: process [s, e), feed each canonical 40-byte record to this segment's SHA, accumulate the
    // partial summary. `i==0` (true only for the very first segment's first sample) zeroes the slew.
    dsfb_sha256_ctx ctx; dsfb_sha256_init(&ctx);
    for (uint32_t i=s;i<e;i++) {
        double xv = base[(uint64_t)i];
        int finite; long long q = dsfb_quantize(xv,&finite);
        if (!finite) oob++;
        if (filled==DSFB_DRIFT_WINDOW) ring_sum -= ring[head]; else filled++;
        ring[head]=q; ring_sum+=q; head=(head+1)%DSFB_DRIFT_WINDOW;
        long long d = ring_sum/(long long)filled;
        long long ev = (q>0)?q:0;
        long long sl = (i==0)?0:(q-prev_q);
        prev_q=q;
        if (ev>0) n_exc++;
        if (ev>pke) pke=ev;
        long long ad=(d<0)?-d:d; if (ad>pkd) pkd=ad;
        long long asl=(sl<0)?-sl:sl; if (asl>pks) pks=asl;
        dsfb_sha256_update_u64le(&ctx,(uint64_t)__double_as_longlong(xv));
        dsfb_sha256_update_i64le(&ctx,q);
        dsfb_sha256_update_i64le(&ctx,ev);
        dsfb_sha256_update_i64le(&ctx,d);
        dsfb_sha256_update_i64le(&ctx,sl);
    }
    uint8_t out[32]; dsfb_sha256_final(&ctx,out);
    #pragma unroll
    for (int i=0;i<32;i++) dig_out[i]=out[i];
    sum_out[0]=(long long)n_exc; sum_out[1]=(long long)oob; sum_out[2]=pke; sum_out[3]=pkd; sum_out[4]=pks;
}

// evidence_kernel_v2_segmented (V2-B) — ONE thread per (lane,segment). Each thread hashes its segment
// independently (with halo warm-up); the host then combines per lane: lane_root_v2 = SHA256(concat of
// the lane's segment digests) and summary = reduction over the lane's segments. NON-equivalent to
// V1/V2-A by design (hash-of-hashes → evidence_root_v2). The win: the per-thread SHA chain shrinks
// from n_samples to <= seg AND the thread count multiplies by segments/lane — directly attacking the
// serial-SHA throughput plateau the V2-A counters exposed.
extern "C" __global__ void evidence_kernel_v2_segmented(
    const double* __restrict__ xcat, const uint64_t* __restrict__ seg_base,
    const uint32_t* __restrict__ seg_start, const uint32_t* __restrict__ seg_end,
    const uint32_t* __restrict__ seg_warmup, uint32_t n_segments,
    uint8_t* __restrict__ seg_digests /* 32*n_segments */, long long* __restrict__ seg_summary /* 5*n_segments */)
{
    uint32_t g = blockIdx.x*blockDim.x + threadIdx.x;
    if (g >= n_segments) return;
    dsfb_segment_evidence(xcat + seg_base[g], seg_start[g], seg_end[g], seg_warmup[g],
                          &seg_digests[(uint64_t)g*32], &seg_summary[(uint64_t)g*5]);
}

// roofline_kernel — pure DRAM streaming read + tree reduction to measure bandwidth.
//
// Each thread reads its strided portion of the double array and accumulates into a local sum.
// A standard parallel reduction within the block (halving tree, shared memory) then produces
// one partial sum per block, written to `partial[blockIdx.x]`. The host sums the partial array
// to get the global checksum. This checksum is the only output; it prevents the compiler from
// treating the reads as dead code.
//
// The purpose of this kernel is NOT evidence production — it is a synthetic DRAM bandwidth
// baseline. Running it with a large array (~512 MB) and `iters` passes gives an empirical
// ceiling for read bandwidth, which can be compared against the evidence_kernel's throughput
// to confirm how far the evidence kernel is below the DRAM roofline.
extern "C" __global__ void roofline_kernel(const double* __restrict__ x, uint64_t n, double* __restrict__ partial) {
    uint64_t idx = (uint64_t)blockIdx.x * blockDim.x + threadIdx.x;
    uint64_t stride = (uint64_t)gridDim.x * blockDim.x; /* grid-stride loop covers all elements */
    double acc = 0.0;
    for (uint64_t i=idx; i<n; i+=stride) acc += x[i];
    // Tree reduction within the block using shared memory.
    // blockDim.x must be a power of 2 (256 threads used in dsfb_chem_cuda_roofline).
    __shared__ double sm[256];
    sm[threadIdx.x]=acc; __syncthreads();
    for (int s=blockDim.x/2; s>0; s>>=1) {
        if (threadIdx.x<s) sm[threadIdx.x]+=sm[threadIdx.x+s];
        __syncthreads();
    }
    if (threadIdx.x==0) partial[blockIdx.x]=sm[0]; /* write block's partial sum */
}

// CK: CUDA error-checking macro. On any non-success cudaError_t, immediately return the code.
// Device memory may leak on early return; this is acceptable for an error path.
#define CK(call) do { cudaError_t _e=(call); if (_e!=cudaSuccess) return (int)_e; } while(0)

// dsfb_chem_cuda_evidence — host wrapper for evidence_kernel.
//
// Data flow:
//   1. Allocate device buffers d_x (input), d_dig (digest output), d_sum (summary output).
//   2. Upload h_x to d_x (H2D transfer).
//   3. Create CUDA events a and b around the kernel launch to time the kernel only.
//   4. Launch evidence_kernel with 128 threads/block, ceil(n_lanes/128) blocks.
//   5. Synchronise on event b, read elapsed time into *elapsed_ms.
//   6. Download d_dig -> h_digests and d_sum -> h_summary (D2H transfer).
//   7. Free device memory and events.
//
// The 128 threads/block choice gives enough parallelism for large lane counts while keeping
// the block small enough to pack multiple blocks onto a single SM.
//
// Returns cudaError_t (0 = success).
extern "C" int dsfb_chem_cuda_evidence(
    const double* h_x, uint32_t n_lanes, uint32_t n_samples,
    uint8_t* h_digests, long long* h_summary, float* elapsed_ms)
{
    double* d_x=nullptr; uint8_t* d_dig=nullptr; long long* d_sum=nullptr;
    uint64_t nx = (uint64_t)n_lanes * n_samples; /* total elements; cast to avoid 32-bit overflow */
    CK(cudaMalloc(&d_x, nx*sizeof(double)));
    CK(cudaMalloc(&d_dig, (uint64_t)n_lanes*32));           /* 32 bytes per lane digest */
    CK(cudaMalloc(&d_sum, (uint64_t)n_lanes*5*sizeof(long long))); /* 5 summary i64s per lane */
    // Pin the input host buffer in place so the H2D runs near full PCIe bandwidth (measured ~2x vs
    // pageable: ~13.5 -> ~26.7 GB/s on a 64 MB residual matrix). Best-effort + digest-irrelevant, as
    // in the V2 wrappers; transfer speed only, not the bytes.
    int _pin = (cudaHostRegister((void*)h_x, nx*sizeof(double), cudaHostRegisterDefault) == cudaSuccess);
    cudaError_t _cpy = cudaMemcpy(d_x, h_x, nx*sizeof(double), cudaMemcpyHostToDevice);
    if (_pin) cudaHostUnregister((void*)h_x);
    if (_cpy != cudaSuccess) return (int)_cpy;
    int threads=128; int blocks=(int)((n_lanes+threads-1)/threads); /* ceiling division */
    cudaEvent_t a,b; cudaEventCreate(&a); cudaEventCreate(&b);
    cudaEventRecord(a); /* start timer just before launch */
    evidence_kernel<<<blocks,threads>>>(d_x, n_lanes, n_samples, d_dig, d_sum);
    cudaEventRecord(b); /* stop timer just after launch (asynchronous) */
    CK(cudaGetLastError()); /* check for launch errors (invalid config, etc.) */
    CK(cudaEventSynchronize(b)); /* block until the kernel completes */
    cudaEventElapsedTime(elapsed_ms, a, b); /* kernel-only time in milliseconds */
    CK(cudaMemcpy(h_digests, d_dig, (uint64_t)n_lanes*32, cudaMemcpyDeviceToHost));
    CK(cudaMemcpy(h_summary, d_sum, (uint64_t)n_lanes*5*sizeof(long long), cudaMemcpyDeviceToHost));
    cudaFree(d_x); cudaFree(d_dig); cudaFree(d_sum);
    cudaEventDestroy(a); cudaEventDestroy(b);
    return 0;
}

// dsfb_chem_cuda_evidence_batched — host wrapper for the V2-A batched kernel.
//
// Input is MANY datasets' lanes concatenated into one launch:
//   h_xcat     — all lanes' samples, lane-contiguous: lane L at h_xcat[offsets[L] .. +nsamples[L]]
//   total_elems— length of h_xcat (sum of all lanes' nsamples)
//   h_offsets  — per-lane start index into h_xcat (n_total_lanes entries)
//   h_nsamples — per-lane sample count (n_total_lanes entries)
// Outputs match the V1 layout (32 bytes + 5 i64 per lane). Because the kernel calls the same
// dsfb_lane_evidence as V1, each lane's digest is byte-identical to a V1 run on that lane — the win
// is purely that one launch covers every dataset, growing the grid past the per-dataset occupancy
// wall. `elapsed_ms` is the kernel-only time (CUDA events); H2D/D2H excluded, as in V1.
//
// Returns cudaError_t (0 = success).
extern "C" int dsfb_chem_cuda_evidence_batched(
    const double* h_xcat, uint64_t total_elems,
    const uint64_t* h_offsets, const uint32_t* h_nsamples, uint32_t n_total_lanes,
    uint8_t* h_digests, long long* h_summary, float* elapsed_ms)
{
    double* d_x=nullptr; uint64_t* d_off=nullptr; uint32_t* d_ns=nullptr;
    uint8_t* d_dig=nullptr; long long* d_sum=nullptr;
    CK(cudaMalloc(&d_x, total_elems*sizeof(double)));
    CK(cudaMalloc(&d_off, (uint64_t)n_total_lanes*sizeof(uint64_t)));
    CK(cudaMalloc(&d_ns, (uint64_t)n_total_lanes*sizeof(uint32_t)));
    CK(cudaMalloc(&d_dig, (uint64_t)n_total_lanes*32));
    CK(cudaMalloc(&d_sum, (uint64_t)n_total_lanes*5*sizeof(long long)));
    // Pin the input host buffer in place so the large H2D runs near full PCIe bandwidth (pageable
    // transfers stage through a driver bounce buffer at ~half speed — nsys measured ~13.5 GB/s here,
    // ~3x the kernel time). Best-effort: if registration fails (already pinned / unsupported), fall
    // back to the plain copy. Digest-irrelevant: pinning changes only transfer speed, not the bytes.
    int _pin = (cudaHostRegister((void*)h_xcat, total_elems*sizeof(double), cudaHostRegisterDefault) == cudaSuccess);
    cudaError_t _cpy = cudaMemcpy(d_x, h_xcat, total_elems*sizeof(double), cudaMemcpyHostToDevice);
    if (_pin) cudaHostUnregister((void*)h_xcat);
    if (_cpy != cudaSuccess) return (int)_cpy;
    CK(cudaMemcpy(d_off, h_offsets, (uint64_t)n_total_lanes*sizeof(uint64_t), cudaMemcpyHostToDevice));
    CK(cudaMemcpy(d_ns, h_nsamples, (uint64_t)n_total_lanes*sizeof(uint32_t), cudaMemcpyHostToDevice));
    int threads=128; int blocks=(int)(((uint64_t)n_total_lanes+threads-1)/threads); /* ceiling division */
    cudaEvent_t a,b; cudaEventCreate(&a); cudaEventCreate(&b);
    cudaEventRecord(a);
    evidence_kernel_batched<<<blocks,threads>>>(d_x, d_off, d_ns, n_total_lanes, d_dig, d_sum);
    cudaEventRecord(b);
    CK(cudaGetLastError());
    CK(cudaEventSynchronize(b));
    cudaEventElapsedTime(elapsed_ms, a, b);
    CK(cudaMemcpy(h_digests, d_dig, (uint64_t)n_total_lanes*32, cudaMemcpyDeviceToHost));
    CK(cudaMemcpy(h_summary, d_sum, (uint64_t)n_total_lanes*5*sizeof(long long), cudaMemcpyDeviceToHost));
    cudaFree(d_x); cudaFree(d_off); cudaFree(d_ns); cudaFree(d_dig); cudaFree(d_sum);
    cudaEventDestroy(a); cudaEventDestroy(b);
    return 0;
}

// dsfb_chem_cuda_evidence_v2_segmented — host wrapper for the V2-B segment-parallel kernel.
//
// Inputs describe `n_segments` independent (lane, segment) work items over the lane-contiguous buffer
// `h_xcat`: for segment g, its lane starts at h_xcat[seg_base[g]], the segment covers samples
// [seg_start[g], seg_end[g]) of that lane, and seg_warmup[g] pre-segment samples are replayed to seed
// the drift ring. Outputs are per-segment: a 32-byte digest and 5 partial-summary i64s. The host
// combines these per lane (concat digests → SHA256 = lane_root_v2; reduce summaries). `elapsed_ms` is
// kernel-only time (CUDA events).
extern "C" int dsfb_chem_cuda_evidence_v2_segmented(
    const double* h_xcat, uint64_t total_elems,
    const uint64_t* h_seg_base, const uint32_t* h_seg_start, const uint32_t* h_seg_end,
    const uint32_t* h_seg_warmup, uint32_t n_segments,
    uint8_t* h_seg_digests, long long* h_seg_summary, float* elapsed_ms)
{
    double* d_x=nullptr; uint64_t* d_base=nullptr; uint32_t* d_start=nullptr; uint32_t* d_end=nullptr;
    uint32_t* d_warm=nullptr; uint8_t* d_dig=nullptr; long long* d_sum=nullptr;
    CK(cudaMalloc(&d_x, total_elems*sizeof(double)));
    CK(cudaMalloc(&d_base, (uint64_t)n_segments*sizeof(uint64_t)));
    CK(cudaMalloc(&d_start, (uint64_t)n_segments*sizeof(uint32_t)));
    CK(cudaMalloc(&d_end, (uint64_t)n_segments*sizeof(uint32_t)));
    CK(cudaMalloc(&d_warm, (uint64_t)n_segments*sizeof(uint32_t)));
    CK(cudaMalloc(&d_dig, (uint64_t)n_segments*32));
    CK(cudaMalloc(&d_sum, (uint64_t)n_segments*5*sizeof(long long)));
    // Pin the input host buffer in place so the large H2D runs near full PCIe bandwidth (pageable
    // transfers stage through a driver bounce buffer at ~half speed — nsys measured ~13.5 GB/s here,
    // ~3x the kernel time). Best-effort: if registration fails (already pinned / unsupported), fall
    // back to the plain copy. Digest-irrelevant: pinning changes only transfer speed, not the bytes.
    int _pin = (cudaHostRegister((void*)h_xcat, total_elems*sizeof(double), cudaHostRegisterDefault) == cudaSuccess);
    cudaError_t _cpy = cudaMemcpy(d_x, h_xcat, total_elems*sizeof(double), cudaMemcpyHostToDevice);
    if (_pin) cudaHostUnregister((void*)h_xcat);
    if (_cpy != cudaSuccess) return (int)_cpy;
    CK(cudaMemcpy(d_base, h_seg_base, (uint64_t)n_segments*sizeof(uint64_t), cudaMemcpyHostToDevice));
    CK(cudaMemcpy(d_start, h_seg_start, (uint64_t)n_segments*sizeof(uint32_t), cudaMemcpyHostToDevice));
    CK(cudaMemcpy(d_end, h_seg_end, (uint64_t)n_segments*sizeof(uint32_t), cudaMemcpyHostToDevice));
    CK(cudaMemcpy(d_warm, h_seg_warmup, (uint64_t)n_segments*sizeof(uint32_t), cudaMemcpyHostToDevice));
    int threads=128; int blocks=(int)(((uint64_t)n_segments+threads-1)/threads);
    cudaEvent_t a,b; cudaEventCreate(&a); cudaEventCreate(&b);
    cudaEventRecord(a);
    evidence_kernel_v2_segmented<<<blocks,threads>>>(d_x, d_base, d_start, d_end, d_warm, n_segments, d_dig, d_sum);
    cudaEventRecord(b);
    CK(cudaGetLastError());
    CK(cudaEventSynchronize(b));
    cudaEventElapsedTime(elapsed_ms, a, b);
    CK(cudaMemcpy(h_seg_digests, d_dig, (uint64_t)n_segments*32, cudaMemcpyDeviceToHost));
    CK(cudaMemcpy(h_seg_summary, d_sum, (uint64_t)n_segments*5*sizeof(long long), cudaMemcpyDeviceToHost));
    cudaFree(d_x); cudaFree(d_base); cudaFree(d_start); cudaFree(d_end); cudaFree(d_warm);
    cudaFree(d_dig); cudaFree(d_sum);
    cudaEventDestroy(a); cudaEventDestroy(b);
    return 0;
}

// dsfb_chem_cuda_roofline — host wrapper for the DRAM bandwidth roofline kernel.
//
// Runs roofline_kernel `iters` times between a single pair of CUDA events so that the timed
// region covers all passes. The `d_part` partial-sum array is only transferred once (after all
// iterations), so the D2H transfer cost is negligible.
//
// The grid: 1024 blocks x 256 threads = 262,144 threads. For large n this fully saturates the
// device. The partial sums (one per block, 1024 doubles) are summed on the host to produce the
// checksum returned to the caller. The checksum is not used for evidence; it only ensures the
// compiler treats all reads as live.
//
// Returns cudaError_t (0 = success).
extern "C" int dsfb_chem_cuda_roofline(const double* h_x, uint64_t n, int iters, double* h_checksum, float* elapsed_ms) {
    double* d_x=nullptr; double* d_part=nullptr;
    int threads=256; int blocks=1024;
    CK(cudaMalloc(&d_x, n*sizeof(double)));
    CK(cudaMalloc(&d_part, (uint64_t)blocks*sizeof(double))); /* one partial sum per block */
    CK(cudaMemcpy(d_x, h_x, n*sizeof(double), cudaMemcpyHostToDevice));
    cudaEvent_t a,b; cudaEventCreate(&a); cudaEventCreate(&b);
    cudaEventRecord(a); /* start timer before first kernel launch */
    for (int it=0; it<iters; it++) roofline_kernel<<<blocks,threads>>>(d_x, n, d_part);
    cudaEventRecord(b); /* stop timer after last launch (all launches are on the same stream) */
    CK(cudaGetLastError());
    CK(cudaEventSynchronize(b)); /* wait for all iters to complete */
    cudaEventElapsedTime(elapsed_ms, a, b); /* total kernel-only time for all iters */
    /* Download partial sums and sum on host (1024 doubles — negligible transfer cost). */
    double* h_part=(double*)malloc((uint64_t)blocks*sizeof(double));
    CK(cudaMemcpy(h_part, d_part, (uint64_t)blocks*sizeof(double), cudaMemcpyDeviceToHost));
    double s=0.0; for (int i=0;i<blocks;i++) s+=h_part[i]; *h_checksum=s;
    free(h_part);
    cudaFree(d_x); cudaFree(d_part); cudaEventDestroy(a); cudaEventDestroy(b);
    return 0;
}

// dsfb_chem_cuda_device_name — copy the active CUDA device name into `buf`.
//
// Queries the current device (as selected by the CUDA runtime) and copies its name into `buf`,
// null-terminated, at most `buflen-1` characters. Used for the execution attestation so that
// the case file records which physical GPU produced the evidence.
extern "C" int dsfb_chem_cuda_device_name(char* buf, int buflen) {
    int dev=0; cudaDeviceProp prop;
    CK(cudaGetDevice(&dev));
    CK(cudaGetDeviceProperties(&prop, dev));
    int i=0; for (; i<buflen-1 && prop.name[i]; i++) buf[i]=prop.name[i]; buf[i]=0; /* null-terminate */
    return 0;
}
