# NASA Power of 10 Alignment Helper

Status: Mapping helper only. No full NASA Power of 10 compliance claim is made.

| NASA-style rule concern | Current crate status | Evidence | Gap / note |
|---|---|---|---|
| Simple control flow | Partial | Core engine modules are straight-line and finite-state oriented | Host-side helper/reporting modules are broader than the smallest critical subset |
| Fixed upper bounds on loops | Partial | Core loops iterate over known-length input slices | Batch processing still depends on input sequence length |
| No dynamic memory in critical code | Not yet satisfied | The core path currently uses `Vec` / `String` under `alloc` | The crate has a `no_std + alloc` core path, not a heapless one |
| No recursion | Satisfied by current helper scan | Addendum safe-subset and compliance scans | Heuristic scan only, not a formal proof |
| Assertions for critical invariants | Partial | Existing tests plus Kani harness scaffolds | Assertions are not yet embedded across every helper path |
| Small functions / modularity | Partial | Core math/detection separation is explicit | Host-side helper modules are larger by design |
| Restrict preprocessor complexity | Partial | Rust feature gating is straightforward | Multiple optional helper layers exist |
| Strong static analysis | Partial | Safe-subset scan, Kani harness scaffolds, standard tests | Not a substitute for a qualified static-analysis toolchain |
| Single point of entry / exit discipline | Partial | Core helper functions are disciplined | Host-side orchestration is multi-function batch code |
| Exhaustive testability | Partial | Existing test suite plus addendum/Kani scaffolds | No claim of full safety-critical coverage |
