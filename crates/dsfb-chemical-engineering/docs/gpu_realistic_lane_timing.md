# Realistic-lane + end-to-end GPU timing — capture procedure + measured result (B6)

*Panel recommendation B6. The committed Nsight tables (`paper/tables/cuda_perf.tex`, `cuda_ncu.tex`) report the
evidence-factory kernel at **large stress sizes** (1024–4096 lanes, ~64 MB) and as **kernel-only** time (CUDA events;
host↔device transfer excluded — `bench.rs:4`). Those are the throughput-roofline ceiling, and the right axis there is
GB/s. The 20 evaluation datasets are **5–700 lanes** (most ≤128); at those sizes the kernel is a near-fixed
~1.8–2.8 ms floor over only kilobytes of data, so the meaningful deployment number is **absolute sealing latency, not
GB/s**. GB/s at deployment sizes divides tiny bytes by a constant floor — it measures the floor, not bandwidth. This
doc records (1) the measured RTX 4080 SUPER result and (2) how to capture it, so the paper can report a
deployment-representative latency alongside the stress-size throughput. The maintainer runs these on the real RTX 4080
SUPER (the sandbox has no GPU); the numbers in §0 are that measured run.*

## 0. Measured result (RTX 4080 SUPER, driver 595.71.05, CUDA 13.2 — maintainer's run)

The four realistic evaluation-dataset shapes (Nsight Systems, **median of 5 runs** per shape). Kernel ms and GB/s use
the kernel-duration basis (GB/s = bytes\,/\,kernel time, as in Table 8); end-to-end = HtoD + kernel + DtoH (median
HtoD 3.8/17.0/24.9/107.9~$\mu$s$\to$ms, DtoH ~2--3.5~$\mu$s):

| Shape (lanes×samples) | Dataset    | MB   | Kernel ms | Evidence GB/s | End-to-end ms |
|-----------------------|------------|------|-----------|---------------|---------------|
| 5×800                 | CSTR       | 0.03 | 2.276     | 0.01          | 2.282         |
| 52×960                | TEP IDV    | 0.38 | 2.746     | 0.15          | 2.765         |
| 128×600               | gas-sensor | 0.59 | 1.763     | 0.35          | 1.790         |
| 590×600               | SECOM      | 2.70 | 1.896     | 1.49          | 2.007         |

(Committed to the paper as the separate `tab:cudaperfrealistic` companion to Table 8, backed by the 20
`reports/nsys_realistic_*x*_run*.txt` captures.)

**Reading of the data — this corrects an earlier hypothesis in this doc.** Full-dataset evidence sealing holds at
**~1.8–2.8 ms end-to-end across the entire realistic envelope**. The cost is a **fixed kernel floor** (kernel launch +
one-thread-per-lane serial SHA over a handful of lanes), **not** host↔device transfer: even at 590 lanes H2D is
~0.11 ms against a ~1.9 ms kernel, and at the small shapes transfer is a few microseconds. So the kernel floor
dominates and transfer is negligible — the opposite of the "transfer dominates end to end" framing this doc previously
carried. The naive "Evidence GB/s" at these sizes (0.01–1.5) is **not a throughput measurement**: it divides kilobytes
by that constant floor, i.e. it reports the floor, not bandwidth. GB/s is only meaningful at the stress sizes that
saturate the device (Table 8). The deployment-relevant facts are the **~1.8–2.8 ms absolute latency** and that the GPU
`evidence_root` is **byte-identical to the CPU** — the GPU's value at realistic sizes is auditable determinism, not
speed.

## 1. Realistic-lane kernel timing (the `profile` subcommand)

`profile` honours `DSFB_BENCH_LANES` / `DSFB_BENCH_SAMPLES` (`cli.rs:341`), runs ONE evidence_kernel launch at that
exact shape, and prints `<ms> ms <GB/s> GB/s` via a CUDA event (kernel-only; needs NO sudo and NO Nsight). Run the
actual dataset shapes to get the per-shape kernel floor:

```fish
# fish shell (CachyOS). Each line: <lanes>x<samples> ≈ a real dataset shape. Use `profile` (NOT `bench`:
# `bench` is a FIXED-size sweep and ignores these env vars).
for dims in "5 800" "52 960" "128 600" "590 600"
    set -l p (string split ' ' $dims)
    env DSFB_BENCH_LANES=$p[1] DSFB_BENCH_SAMPLES=$p[2] \
        cargo run --release -p dsfb-chemical-engineering-cuda --features cuda -- profile
end
```

Representative shapes: `5×800` CSTR, `52×960` TEP IDV, `128×600` gas-sensor, `590×600` SECOM. Confirmed result: the
kernel sits at a near-fixed ~1.8–2.8 ms floor regardless of lane count (low occupancy: the kernel is
SHA-compute/parallelism-bound, one thread per lane), so the per-shape GB/s is just that floor expressed as bandwidth
over tiny data — it is **not** the metric. The metric is the absolute ms in §0.

## 2. End-to-end (transfer-inclusive) timing via Nsight Systems

`profile`'s CUDA event is kernel-only; to measure the deployment-relevant **H2D + kernel + D2H** wall time at a
realistic size, wrap the `profile` binary in `nsys` and read the memcpy + kernel rows from the summary:

```fish
# Realistic 128-lane case end to end. nsys captures the full timeline incl. cudaMemcpy H2D/D2H.
# On hosts with restricted profiling, prefix `sudo` and point at the prebuilt binary directly, e.g.:
#   sudo env DSFB_BENCH_LANES=128 DSFB_BENCH_SAMPLES=600 \
#       nsys profile --trace=cuda --stats=true --force-overwrite=true \
#       -o reports/nsys_realistic_128x600 ./target/release/dsfb-chem-cuda profile
env DSFB_BENCH_LANES=128 DSFB_BENCH_SAMPLES=600 \
    nsys profile --stats=true --force-overwrite=true \
    -o reports/nsys_realistic_128x600 \
    (cargo build --release -p dsfb-chemical-engineering-cuda --features cuda --message-format=json \
     | jq -r 'select(.executable!=null).executable' | head -1) profile
# Then read the "CUDA GPU MemOps Summary" (HtoD/DtoH bytes + time) and "CUDA GPU Kernel Summary"
# from the printed stats: end-to-end ≈ H2D + kernel + D2H (= the §0 table).
```

(Or simply `bash crates/dsfb-chemical-engineering-cuda/scripts/run_nsight.sh 5`, which drives `nsys`/`ncu` across
variants and writes `reports/nsys_*.txt`. The realistic dataset shapes are built into its `VARIANTS` map as
`r1=5×800` (CSTR), `r2=52×960` (TEP IDV), `r3=128×600` (gas-sensor), `r4=590×600` (SECOM) alongside the three stress
sizes — so one run captures both, and each realistic `nsys_*.txt` carries the MemOps + Kernel summary to read
end-to-end from. The §0 table is the median of that run.)

## 3. Folding the result into the paper (honest framing — absolute latency, not GB/s)

The realistic regime is reported as **absolute sealing latency + determinism**, NOT as a GB/s row competing with the
stress-size throughput:

- Report that the four realistic dataset shapes seal end-to-end in **~1.8–2.8 ms**, byte-identical to the CPU
  `evidence_root`. State explicitly that **GB/s is not a meaningful axis at deployment sizes** (a fixed kernel floor
  over kilobytes — it measures the floor, not bandwidth); the stress variants in Table 8 remain the
  throughput-roofline ceiling.
- One sentence tying it to the existing Q35/Q41 concession: at realistic 5–128 lane counts the kernel is a small
  fixed-cost sealing contract, and the GPU's contribution there is **byte-exact auditable determinism, not speed** —
  which the ~1.8–2.8 ms latency confirms. This **sharpens** the existing concession; it does not retract any claim.

This **broadens** the disclosure (it states the full operating envelope — stress ceiling AND realistic floor — and the
boundary where GB/s stops being meaningful) without narrowing any claim. The stress-size GB/s (4.87/2.43/9.77), the V2
26.7 GB/s end-to-end figure, and the 636.8 GB/s memory roofline are all unchanged; this **adds** the latency axis.
Publish only the measured RTX 4080 SUPER numbers in §0 — no estimates.
