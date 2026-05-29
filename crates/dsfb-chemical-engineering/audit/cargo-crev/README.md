# cargo-crev — community dependency reviews

`cargo crev` checks each dependency against the **distributed web-of-trust review database** — community-published,
cryptographically-signed code reviews. `cargo-crev.txt` lists every dependency with its review status; the run
exits **255** because **no community trust reviews are available** for these versions in the local proof database
(and no trust root is configured) — the common, honest state for a small, modern dependency set.

## How to read this
Exit 255 / empty review columns mean **"no one in the crev web-of-trust has published a review of this exact
version,"** not "unsafe." The first-party crates (the `dsfb-chemical-engineering-*` rows marked `*`) are local
path crates and are not expected to carry external community reviews.

## What it does NOT certify
Absence of crev reviews is absence of *community attestation*, not presence of risk. Establishing a trust root and
fetching proof repositories (`cargo crev repo fetch all`) would populate the review data; until then this folder
honestly records "no reviews available," not a pass.
