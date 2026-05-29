#!/usr/bin/env bash
# build_cuda.sh — build the CUDA crate with the GPU backend enabled.
# Discovers nvcc at /opt/cuda (or CUDA_HOME). Falls back to the CPU-reference build if no nvcc.
set -euo pipefail
WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "${WORKSPACE_ROOT}"

if [[ -z "${CUDA_HOME:-}" && -d /opt/cuda ]]; then export CUDA_HOME=/opt/cuda; fi
if [[ -n "${CUDA_HOME:-}" ]]; then export PATH="${CUDA_HOME}/bin:${PATH}"; fi

if command -v nvcc >/dev/null 2>&1; then
  echo "nvcc: $(nvcc --version | tail -1)"
  cargo build --release -p dsfb-chemical-engineering-cuda --features cuda
  echo "built with CUDA backend."
else
  echo "nvcc not found — building CPU-reference path (identical evidence, no GPU acceleration)."
  cargo build --release -p dsfb-chemical-engineering-cuda
fi
