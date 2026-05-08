# DSFB-Debug — reviewer reproducibility container.
#
# One-command boot of the entire DSFB-Debug verification chain on a
# clean machine. From a fresh checkout:
#
#     docker build -t dsfb-debug:reviewer .
#     docker run --rm dsfb-debug:reviewer
#
# Default CMD runs `bash scripts/reproduce_all.sh`, the same full
# reproduction path documented for local reviewer runs.
#
# Pinned to rust:1.75-bookworm to match the rust-toolchain.toml MSRV.
# No Cargo runtime dependencies in the no_std core; the demo feature
# adds plotters + zip; both vendored via cargo's own resolution.
#
# With fixed source, Cargo.lock, feature flags, vendored fixture bytes,
# and the Rust 1.75 container, the verification path is intended to
# reproduce the published audit ledgers from `docs/audit/`.

FROM rust:1.75-bookworm

WORKDIR /work/dsfb-debug

# Bring source tree in. .dockerignore excludes target/, local papers,
# generated demo outputs, upstream archives, and other build artefacts;
# reviewers running `docker build` get a clean
# build from source.
COPY . .

# Pre-fetch dependencies and compile test binaries under the checked-in
# lockfile. This catches dependency drift during image build while
# keeping the full, stdout-producing run in the default CMD.
RUN cargo build --locked --no-default-features \
    && cargo test --locked --features "std paper-lock" --no-run

# Default verification chain. Re-runs every fixture eval + LO-CV
# + sensitivity + ablation + subset opt + timing + property tests via
# the canonical local reproducer script.
CMD ["bash", "scripts/reproduce_all.sh"]
