#!/usr/bin/env sh
# Rebuild the WASM court simulator and refresh the committed browser assets (the .wasm + the sample stream).
# Run from anywhere; paths are resolved relative to this script.
set -eu
crate_dir="$(cd "$(dirname "$0")/.." && pwd)"
cd "$crate_dir"

cargo build --target wasm32-unknown-unknown --release
# target-dir is redirected to the shared root target/ (see .cargo/config.toml), so the artifact is here:
cp ../../target/wasm/wasm32-unknown-unknown/release/dsfb_chemical_engineering_wasm.wasm web/dsfb_court_sim.wasm
python3 web/gen_sample_residuals.py

echo "refreshed web/dsfb_court_sim.wasm + web/sample_residuals.json"
echo "serve:  (cd web && python3 -m http.server 8000)   then open  http://localhost:8000/"
