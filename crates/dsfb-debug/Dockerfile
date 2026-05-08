# DSFB-Debug — reviewer reproducibility container (Phase η.8, Session 18).
#
# One-command boot of the entire DSFB-Debug verification chain on a
# clean machine. From a fresh checkout:
#
#     docker build -t dsfb-debug:reviewer .
#     docker run --rm dsfb-debug:reviewer
#
# Default CMD runs `cargo test --features "std paper-lock" -- --nocapture`
# which fires every per-fixture eval test, the LO-CV harness, the
# sensitivity sweep, the axis ablation, the detector subset opt, the
# timing benchmark, and the property-test suite end-to-end.
#
# Pinned to rust:1.75-bookworm to match the rust-toolchain.toml MSRV.
# No Cargo runtime dependencies in the no_std core; the demo feature
# adds plotters + zip; both vendored via cargo's own resolution.
#
# Theorem 9 deterministic replay verified inside every test invoked
# by the default CMD; reproducibility is byte-identical across
# reviewer machines that match the rust:1.75-bookworm digest.

FROM rust:1.75-bookworm

WORKDIR /work/dsfb-debug

# Bring source tree in. .dockerignore should exclude target/ +
# build artefacts; reviewers running `docker build` get a clean
# build from source.
COPY . .

# Pre-fetch dependencies + build the crate so first `docker run`
# is snappy. Build with full features so the LO-CV harness +
# audit modules are wired.
RUN cargo build --features "std paper-lock"

# Default verification chain. Re-runs every fixture eval + LO-CV
# + sensitivity + ablation + subset opt + timing + property tests.
# Output is the verbatim test stdout reviewers should match against
# the published audit ledgers in docs/audit/.
CMD ["cargo", "test", "--features", "std paper-lock", "--", "--nocapture"]
