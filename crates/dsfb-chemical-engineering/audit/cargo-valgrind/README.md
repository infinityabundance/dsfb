# cargo-valgrind — runtime memory checking (RUN; CLEAN: 0 errors in DSFB code)

**Result: `ERROR SUMMARY: 0 errors from 0 contexts` on the DSFB pipeline** (verify-replay, which runs the full
deterministic pipeline and emits correct replay hashes), once the musl-static-startup allocator false positives are
suppressed — see [`run_musl_clean.txt`](run_musl_clean.txt). 112 musl-allocator false positives were suppressed via
[`dsfb-musl-startup.supp`](dsfb-musl-startup.supp); **0 real memory errors remain** in DSFB code. Path to that result
below (it took some work — valgrind vs this host's AVX-512 + musl).


[`cargo-valgrind`](https://github.com/jfrimmel/cargo-valgrind) runs a crate's binary/tests under
[Valgrind](https://valgrind.org/) **Memcheck**, reporting at *runtime*: leaks, invalid reads/writes, use of
uninitialised memory, and double frees. The dynamic complement to the static `#![forbid(unsafe_code)]` posture and
to Miri's interpreter.

## What was actually done (2026-05-27, this host)
Valgrind **3.24.0 was built from source to `~/.local`** (no sudo needed) and `cargo-valgrind` was installed
(`cargo install cargo-valgrind`). Memcheck was then run on the `edge` binary. Two real, evidenced outcomes:

- **glibc-dynamic binary → SIGILL (valgrind cannot decode this host's AVX-512).** Valgrind aborts with
  `vex amd64->IR: unhandled instruction bytes: 0x62 0xF1 0x7F 0x48 ...` — the `0x62` prefix is an **EVEX / AVX-512**
  store that glibc's optimised `memcpy/memset` ifuncs use on this CPU, and valgrind 3.24's VEX decoder does not
  support it. Rebuilding the Rust binary at `x86-64-v3` (no AVX-512) did **not** help, because the AVX-512 is in the
  host **glibc**, not the binary. Evidence: [`run_glibc_sigill.txt`](run_glibc_sigill.txt).
- **static-musl binary → runs to completion, but musl-malloc false positives.** Built
  `--target x86_64-unknown-linux-musl` (its own libc, no AVX-512 ifuncs); valgrind then **runs the whole program**
  (`verify-replay` produced its correct replay hashes). It reports **112 errors from 12 contexts**, but every one is
  *"Conditional jump or move depends on uninitialised value(s)"* inside **musl's allocator**
  (`__libc_malloc_impl` / `enframe`) — the well-known musl+valgrind false-positive class, **not** DSFB code (the DSFB
  frames are only the callers that reached `malloc`). Evidence: [`run_musl.txt`](run_musl.txt).

## The clean run (achieved — verified commands)
Build a **static-musl** binary (its own libc, no host-glibc AVX-512 ifuncs) and suppress the musl-allocator false
positives:
```fish
rustup target add x86_64-unknown-linux-musl
cargo build -p dsfb-chemical-engineering-edge --target x86_64-unknown-linux-musl
valgrind --leak-check=full --show-leak-kinds=all \
  --suppressions=audit/cargo-valgrind/dsfb-musl-startup.supp \
  target/x86_64-unknown-linux-musl/debug/dsfb-chem-edge verify-replay
# => ERROR SUMMARY: 0 errors from 0 contexts (suppressed: 112 from 12)
```
[`dsfb-musl-startup.supp`](dsfb-musl-startup.supp) suppresses errors originating in musl's allocator
(`enframe`/`alloc_slot`/`alloc_group`/`__libc_malloc_impl`) — the documented musl+valgrind false-positive class. With
those gone, **0 real memory errors remain** in the DSFB pipeline, which runs to completion and emits correct replay
hashes (`run_musl_clean.txt`).

## Why the glibc-dynamic route is blocked here (root cause)
The offending bytes `0x62 0xF1 0x7F 0x48 0x7F ...` are a `vmovdqu32` storing a 512-bit ZMM register (EVEX, map 0F,
opcode 7F). valgrind **3.25.1** (latest) still aborts on it: its LibVEX decoder lacks this exact EVEX permutation.
Crucially, the AVX-512 is emitted by the **dynamic linker (`ld.so`)** during early CPU-profiled relocation/memory
clearing — *before* `main()` — so `GLIBC_TUNABLES=glibc.cpu.hwcaps=-AVX512F[_Usable]` / `-AVX512*` /
`Prefer_No_AVX512` do **not** prevent it (all still SIGILL, evidenced), and rebuilding the Rust binary at
`x86-64-v3` does not help (the instruction is in glibc, not the binary). The proper long-term fix is a LibVEX decode
rule for that byte string (a valgrind Bugzilla item); the static-musl route above sidesteps `ld.so` entirely and is
the working path.

The `no_std` crates (`core`/`atlas`/`corpus`) are **out of scope** — they do not allocate, so Memcheck has nothing to
check; their memory story is `forbid(unsafe_code)` + Miri + the bounded ring-buffer budget. The CUDA device path
needs `compute-sanitizer`, not Valgrind.

## What it does NOT certify
A clean Memcheck proves only the *exercised executions* had no detected memory error — sampling, not a proof, and
only over code the run touched. It says nothing about logical correctness or the GPU path.
