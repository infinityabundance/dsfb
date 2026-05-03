# Phosphoric Host Intrinsics

Host-profile programs may call a small, frozen set of intrinsic
functions that are not implementable in v0 grammar without becoming
verbose enough to obscure their use site. Each intrinsic is admitted
explicitly, with a one-paragraph justification, and is callable only
from the `host` profile.

## `cmp_bytes`

```phos
fn cmp_bytes(s: Slice[u8, 65536], i: u32, pat: [u8; 16], pat_len: u32) -> bool
```

**Justification.** The lexer compares short byte runs against keyword
patterns thousands of times per compilation. Without `cmp_bytes`, the
lexer expresses each comparison as either a `seq4`/`seq5`/`seq7`/`seq10`
recursion (which itself nests 4–10 levels of byte-pair matches and
generates an 11-deep cascade at each keyword-length bucket) or as an
inline byte-by-byte nested-match (the kw_len2/kw_len3 form, which
fans out 6–7 levels deep per call site). Both shapes are doctrinally
equivalent and individually defensible, but together they pin the
lexer at roughly 740 real LOC and force every reader to walk identical
nested-match templates seven times. `cmp_bytes` collapses them into a
single bounded-loop primitive: it reads `pat_len` bytes from the
source slice starting at offset `i`, compares them to the leading
`pat_len` bytes of `pat` (a fixed 16-byte buffer; tail unused), and
returns `true` iff every byte matches. The pattern buffer is fixed at
16 bytes so no v0 keyword (max 10 bytes for `capability`) requires
varying-size arrays; callers pad shorter patterns with zeros and pass
the actual length in `pat_len`. This is the only host intrinsic
admitted as part of v0.1 step 3 (lexer slim).

**Effect**. `cmp_bytes` is pure: it reads from caller-supplied buffers
and returns a `bool`. It declares no host effects.

**Capacity.** The source slice is fixed at 65536 bytes, matching the
lexer's source-buffer ceiling. The pattern buffer is fixed at 16
bytes, larger than the longest v0 keyword (`capability`, 10 bytes).
Both are caller-allocated; the intrinsic does not allocate.

**Conformance.** A positive case (a 4-byte pattern matching a 4-byte
prefix at offset 0) and a negative case (a length mismatch) belong
to `tests/conformance/host/`. Both test cases consume the intrinsic
exactly as the lexer uses it.

## Stability

The `cmp_bytes` signature is frozen at v0.1 step 3 landing. Any
change to the parameter shape, capacity ceiling, or return type is
a breaking change to host intrinsics and requires a coordinated
update to this document, the lexer call sites, and a new
conformance entry.
