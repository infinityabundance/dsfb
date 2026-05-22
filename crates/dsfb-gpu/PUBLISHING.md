# Publishing

This is the crates.io release checklist for the `dsfb-gpu` workspace.
It is deliberately conservative: package locally, test locally, then
publish crates in dependency order.

## Preflight

Run from `crates/dsfb-gpu`:

```sh
cargo test --workspace --all-targets
cargo package --workspace --allow-dirty
cargo publish --dry-run -p dsfb-gpu-debug-core --allow-dirty
```

Before the first upload, dry-runs for dependent crates will fail with
`no matching package named ... found` until their local dependencies are
published and visible in the crates.io index. That is expected for a
new multi-crate release.

## Publish Order

1. `cargo publish -p dsfb-gpu-debug-core`
2. `cargo publish --dry-run -p dsfb-gpu-atlas-corpus`
3. `cargo publish -p dsfb-gpu-atlas-corpus`
4. `cargo publish --dry-run -p dsfb-gpu-debug-cuda`
5. `cargo publish -p dsfb-gpu-debug-cuda`
6. `cargo publish --dry-run -p dsfb-gpu-atlas-registry`
7. `cargo publish -p dsfb-gpu-atlas-registry`
8. `cargo publish --dry-run -p dsfb-gpu-debug-demo`
9. `cargo publish -p dsfb-gpu-debug-demo`

If crates.io index propagation lags after a publish, wait and rerun the
same dry-run. Do not remove `version` from path dependencies; Cargo
uses the version when publishing and the path while developing locally.

## Current Dependency Graph

- `dsfb-gpu-debug-core`: no internal dependencies.
- `dsfb-gpu-atlas-corpus`: depends on `dsfb-gpu-debug-core`.
- `dsfb-gpu-debug-cuda`: depends on `dsfb-gpu-debug-core`.
- `dsfb-gpu-atlas-registry`: depends on `dsfb-gpu-debug-core` and
  `dsfb-gpu-atlas-corpus`.
- `dsfb-gpu-debug-demo`: depends on `dsfb-gpu-debug-core` and
  `dsfb-gpu-debug-cuda`.
