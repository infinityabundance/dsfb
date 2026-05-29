# cargo-vet — supply-chain audit-status

`cargo vet` records, per dependency, whether a human has audited that exact version for supply-chain safety. On
this workspace it reports **"Vetting Succeeded (36 exempted)"** (exit 0, `cargo-vet.txt`): the 36 third-party
crates are currently carried as **exemptions** — trusted-by-default but **not yet individually reviewed** — which
is the honest starting state for a project that has not yet imported the public audit registries or performed its
own reviews. The vet store lives in `supply-chain/` at the repo root.

## How to read this
"Succeeded (36 exempted)" is **review-readiness, not a clean audit**: the supply-chain graph is enumerated, pinned,
and the store is initialised, so reviews/imports can be layered on incrementally. It does NOT mean the 36 crates
have been vetted. Importing the Mozilla/Google/Bytecode audit sets (`cargo vet import …`) would convert exemptions
into inherited audits.

## What it does NOT certify
An exemption is an explicit "we have not reviewed this yet" marker, not a safety claim. cargo-vet covers
*third-party dependencies* only — first-party code is the job of dsfb-gray, cargo-geiger, Miri, Kani, clippy, and
the test/proof suite.
