# Source Traceability Workflow

`dsfb-semiotics-engine` now carries machine-parsable theorem-to-code trace tags in implementation
source so that reviewers can trace paper-level items to concrete code regions without manual
archaeology.

## Tag Format

Use one tag per source line comment:

```text
TRACE:TYPE:ID:SHORT_TITLE[:NOTE]
```

Rules:

- `TYPE` must be one of `THEOREM`, `PROPOSITION`, `LEMMA`, `COROLLARY`, `DEFINITION`,
  `ASSUMPTION`, `ALGORITHM`, `CLAIM`, or `INTERFACE`
- `ID` must use uppercase hyphenated tokens such as `DEF-RESIDUAL` or
  `ALG-SEMANTIC-RETRIEVAL`
- `SHORT_TITLE` must be concise and recognizable
- `NOTE` is optional and should describe the implementation role briefly
- keep tags in implementation source, tests, or interface code, not only in prose docs

Example:

```rust
// TRACE:DEFINITION:DEF-RESIDUAL:Residual construction:Implements observed minus predicted residual formation.
```

## Where Tags Belong

Tag the code region where the paper-level concept is actually realized:

- definitions on the function or type that computes that definition
- algorithms on the function that performs the deterministic procedure
- assumptions on the code that enforces or operationalizes the assumption
- interface tags on C ABI, C++ wrapper, Python binding, or similar external boundaries
- executable-evidence claims on tests when a test is the main practical evidence

## Regenerating The Matrix

From the crate root:

```bash
cargo run --manifest-path Cargo.toml --bin dsfb-traceability
```

That rewrites:

- [`docs/THEOREM_TO_CODE_TRACEABILITY.md`](THEOREM_TO_CODE_TRACEABILITY.md)

To check freshness without rewriting:

```bash
cargo run --manifest-path Cargo.toml --bin dsfb-traceability -- --check
```

## Maintenance Workflow

1. Add or update `TRACE:` tags in the relevant source.
2. Regenerate the matrix.
3. Review the rendered file and line mappings.
4. Commit both the source changes and the regenerated matrix.
5. Let the crate-local QA gate verify freshness.

## Auditor Reading Guidance

The generated matrix is a traceability aid. It shows which source locations claim to implement or
evidence a theorem, definition, assumption, algorithm, claim, or interface item. It does not prove
correctness by itself; it shortens the path from theory statements to auditable implementation
regions.
