# Cross-Architecture Determinism — Local QEMU Protocol

This document describes how to validate the bit-exact-across-architectures
claim for the `paper-lock` JSON output **before** pushing the
[`.github/workflows/determinism.yml`](../.github/workflows/determinism.yml)
matrix run. The CI workflow exercises real hardware (Linux x86_64,
Linux aarch64, macOS aarch64); the QEMU protocol below lets a developer
exercise an aarch64 build under user-mode emulation locally to catch
architecture-specific divergences before the CI run.

## Pre-requisites

System-level tools (one-time install):

```bash
# Debian / Ubuntu
sudo apt install qemu-user-static gcc-aarch64-linux-gnu

# Arch / CachyOS
sudo pacman -S qemu-user-static aarch64-linux-gnu-gcc \
               aarch64-linux-gnu-binutils
```

Rust target (one-time install):

```bash
rustup target add aarch64-unknown-linux-gnu
```

`.cargo/config.toml` (per-developer or per-workspace) configures the
linker for the cross target:

```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
```

## Run

```bash
cd crates/dsfb-robotics

# 1. Build paper-lock for x86_64 (host)
cargo build --release --features std,paper_lock --bin paper-lock

# 2. Build paper-lock for aarch64
cargo build --release --features std,paper_lock --bin paper-lock \
    --target aarch64-unknown-linux-gnu

# 3. Run both binaries on the same residual CSV; checksum the JSON
mkdir -p audit/cross_arch
for slug in $(target/release/paper-lock --list); do
    target/release/paper-lock "$slug" \
        > "audit/cross_arch/${slug}.x86_64.json" 2>/dev/null
    qemu-aarch64 -L /usr/aarch64-linux-gnu \
        target/aarch64-unknown-linux-gnu/release/paper-lock "$slug" \
        > "audit/cross_arch/${slug}.aarch64.json" 2>/dev/null
done

# 4. Diff the JSON outputs
for slug in $(target/release/paper-lock --list); do
    if diff -q "audit/cross_arch/${slug}.x86_64.json" \
              "audit/cross_arch/${slug}.aarch64.json" > /dev/null; then
        echo "OK  ${slug}"
    else
        echo "DRIFT ${slug}"
        diff "audit/cross_arch/${slug}.x86_64.json" \
             "audit/cross_arch/${slug}.aarch64.json"
    fi
done
```

Expected output: 20 lines of `OK`, zero `DRIFT`. Any drift indicates
either (a) a floating-point operation not following IEEE 754 binary64
semantics on aarch64 (extremely unusual on modern hardware), or (b)
endianness handling somewhere in the JSON serialiser (also extremely
unusual since serde_json emits ASCII).

## Why this is informative

The DSFB engine uses no FMA, no rounding-mode-dependent operations,
no platform-specific intrinsics — only standard Rust f64 arithmetic
plus serde_json text emission. Running under aarch64 QEMU exercises:

- ARMv8 floating-point unit (a different physical FP implementation)
- ARMv8 instruction scheduling (different load-store reordering)
- ARMv8 pointer-tagging behaviour (where Miri also looks)
- A different `serde_json` codegen (cross-compilation can produce
  slightly different inlining decisions)

Equality under all three of these stresses is strong evidence that
the output is determined by the algorithm, not by the host platform.

## Relation to the GitHub Actions matrix

The QEMU protocol covers `aarch64-unknown-linux-gnu` only. The CI
matrix in [`.github/workflows/determinism.yml`](../.github/workflows/determinism.yml)
covers `ubuntu-latest` (x86_64), `ubuntu-24.04-arm` (real aarch64),
and `macos-14` (Apple Silicon, Mach-O / aarch64). QEMU finds most
architecture issues; the macOS leg additionally covers Mach-O linker
quirks and Apple-specific libc oddities. Both layers are
complementary.

## Why this protocol is documented but not executed in this revision

The dsfb-robotics development sandbox does not ship the
`gcc-aarch64-linux-gnu` cross-linker, so `cargo build --target
aarch64-unknown-linux-gnu` fails at the link step from inside the
sandbox. The protocol is therefore a developer-side check; CI
exercises real hardware. A `make qemu` recipe in the parent workspace
or in a future `flake.nix` profile can wire this to one command for
contributors who do install the cross-toolchain.
