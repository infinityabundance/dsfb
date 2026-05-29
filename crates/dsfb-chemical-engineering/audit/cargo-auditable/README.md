# cargo-auditable — embedded dependency manifest (SBOM)

`cargo auditable build` embeds the exact dependency tree into the compiled binary, so the *shipped artifact* can be
audited later without its source tree or lockfile. The `edge` release binary `dsfb-chem-edge` was built this way
(`build.txt`), and `cargo audit bin` then read the embedded data back out —
**"Found 'cargo auditable' data in target/release/dsfb-chem-edge (34 dependencies)"** — and scanned those 34
dependencies against the RustSec advisory database: **clean** (`audit-bin.txt`).

## What it does NOT certify
This proves the binary carries a verifiable, machine-readable bill of materials and that its embedded dependencies
had no known RustSec advisory at build time. It is provenance / SBOM tooling — **not** a proof of binary
correctness or of the absence of vulnerabilities in the first-party code.
