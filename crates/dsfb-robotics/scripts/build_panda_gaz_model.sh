#!/usr/bin/env bash
# Build the vendored Gaz 2019 Panda dynamic-model cpp library.
#
# This builds the upstream `panda_dyn_model_example` cpp project plus the
# crate-local `compute_tau_pred` driver that consumes recorded q/dq and
# emits the literal Gaz published-model torque prediction tau_pred(k) for
# each timestep. The output is then differenced against the recorded
# torques in `scripts/compute_published_residuals.py` to produce
# `data/processed/panda_gaz_published.csv` — the literal Gaz
# identification residual that the §10.5 paper-lock census now reads.
#
# Pre-reqs: cmake ≥ 3.5, Eigen3, a C++11 compiler.
# Idempotent: re-builds only if the build directory is missing.

set -euo pipefail

CRATE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MODEL_DIR="${CRATE_ROOT}/data/panda_gaz/upstream_model"
BUILD_DIR="${MODEL_DIR}/build"

if [[ ! -d "${MODEL_DIR}" ]]; then
    echo "ERROR: vendored model directory not found at ${MODEL_DIR}" >&2
    echo "       The Gaz 2019 cpp model must be vendored under data/panda_gaz/upstream_model/" >&2
    exit 2
fi

if [[ -x "${BUILD_DIR}/compute_tau_pred" ]]; then
    echo "OK: ${BUILD_DIR}/compute_tau_pred already built; skipping (delete the build/ dir to rebuild)."
    exit 0
fi

mkdir -p "${BUILD_DIR}"
cd "${BUILD_DIR}"
cmake -DCMAKE_POLICY_VERSION_MINIMUM=3.5 ..
make -j2 compute_tau_pred

echo "OK: built ${BUILD_DIR}/compute_tau_pred"
