# `dsfb-atlas` Audit Folder

This directory holds the audit documentation, scripts, harnesses, fuzz
target, and machine-readable reports for the `dsfb-atlas` crate.

```
audit/
├── README.md          ← this file
├── AUDIT.md           ← per-tool description, invocation, pass criteria
├── scripts/
│   ├── run_all.sh     ← end-to-end audit runner
│   ├── dsfb_gray.sh   ← threat-surface scan
│   ├── miri.sh        ← undefined-behaviour checker
│   ├── kani.sh        ← bounded model checker
│   └── fuzz.sh        ← libfuzzer-driven fuzzer
├── fuzz/              ← cargo-fuzz scaffolding (own Cargo.toml)
│   ├── Cargo.toml
│   ├── fuzz_targets/yaml_part.rs
│   └── corpus/yaml_part/  ← seeded with the 10 real P*.yaml files
└── reports/
    ├── dsfb_gray.json
    ├── miri.txt
    ├── kani.txt
    └── fuzz.txt
```

Run all four audits sequentially:

```bash
cd crates/dsfb-atlas/audit
./scripts/run_all.sh
```

Each script writes a machine-readable report into `reports/` and exits
non-zero on failure. See [`AUDIT.md`](./AUDIT.md) for tool descriptions,
expected runtimes, and pass criteria.

## Latest run summary

| Tool         | Verdict | Report                      |
|--------------|---------|-----------------------------|
| `dsfb-gray`  | PASS    | `reports/dsfb_gray.json`    |
| Miri         | PASS    | `reports/miri.txt`          |
| Kani         | PASS    | `reports/kani.txt`          |
| cargo-fuzz   | PASS    | `reports/fuzz.txt`          |

The crate is ~600 LOC of pure-data-pipeline Rust with no `unsafe`, no
FFI, no concurrency, and no network access. The audit posture is
intentionally conservative; the negative results are the load-bearing
claim.
