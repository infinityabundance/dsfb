# Reproducible local-run container for the DSFB-Chemical-Engineering edge binary.
#
# Purpose: an operator can run the read-only court on their OWN machine on their OWN data, in a pinned
# environment, and share only a redacted hash-linked evidence bundle — the confidential-evaluation path
# (raw plant data never leaves the operator's control; see docs/sbir_operator_data_request.md and
# docs/real_data_dropin.md). CPU-only; no GPU required (the CUDA crate is a separate, optional backend).
#
# Build:  docker build -t dsfb-chem-edge .
# Run:    docker run --rm dsfb-chem-edge verify-replay
#         docker run --rm -v "$PWD/data:/data:ro" dsfb-chem-edge data-readiness /data/your_export.csv
#         docker run --rm -v "$PWD/out:/out" dsfb-chem-edge casefile tennessee_eastman_idv01

# ── Stage 1: build ──────────────────────────────────────────────────────────────────────────────────
FROM rust:1-slim-bookworm AS build
WORKDIR /src
# SOURCE_DATE_EPOCH keeps timestamps deterministic for reproducible artifacts.
ENV SOURCE_DATE_EPOCH=1700000000
COPY . .
# Build only the edge crate (CPU-only; the workspace's CUDA crate needs nvcc and is intentionally skipped).
RUN cargo build --release -p dsfb-chemical-engineering-edge \
    && strip target/release/dsfb-chem-edge

# ── Stage 2: minimal runtime ──────────────────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime
# Non-root by default (the court is read-only; it needs no privileges).
RUN useradd --create-home --uid 10001 dsfb
WORKDIR /work
# The binary resolves its bundled data/ via $DSFB_CHEM_DIR; ship the crate dir so demo/atlas/corpus work.
COPY --from=build /src/target/release/dsfb-chem-edge /usr/local/bin/dsfb-chem-edge
COPY --from=build /src/crates/dsfb-chemical-engineering-edge /opt/dsfb/crates/dsfb-chemical-engineering-edge
ENV DSFB_CHEM_DIR=/opt/dsfb/crates/dsfb-chemical-engineering-edge
USER dsfb
ENTRYPOINT ["dsfb-chem-edge"]
CMD ["verify-replay"]
