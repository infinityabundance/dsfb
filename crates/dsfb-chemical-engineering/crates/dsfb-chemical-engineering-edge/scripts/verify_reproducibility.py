#!/usr/bin/env python3
"""P47: independently byte-verify the balance-witness results without redistributing gated data.

The headline real-data witness results (SWaT T101, BATADAL T1) are computed on iTrust/benchmark bytes
that may NOT be redistributed, and the synthetic demonstrators' witness traces are gitignored
(regenerable). That leaves a "trust the recipe" gap: a third party cannot confirm their run reproduces
the paper's numbers. This tool closes it WITHOUT shipping any data — only committed SHA-256 *digests* of
the canonical witness trace (`data/instrumented/EXPECTED_DIGESTS.toml`).

For each balance demonstrator it regenerates the witness via the committed CLI
(`balance-witness <name>`), reads the per-sample witness trace, and hashes a **canonical, platform-
portable** representation of it: each row as `time_index:round(balance_residual*1e6):grammar_state`, with
signed zero normalised. Hashing the rounded-integer residual (not the formatted text) makes the digest
independent of last-ULP float formatting across compilers/platforms.

  - SYNTHETIC demonstrators (three_tank, quadruple_tank, cstr, csth) are openly shareable: anyone can
    regenerate them with `scripts/gen_instrumented.py` and this digest must match.
  - GATED demonstrators (swat_t101, batadal_t1) require the licensed data prepped locally
    (`scripts/prep_{swat,batadal}.py`); if the instrumented CSV is absent the entry is SKIPPED. A holder
    of the licensed data can run this and byte-confirm their run equals the paper's — no bytes shared.

Usage:
    python3 scripts/verify_reproducibility.py            # verify balance-witness traces vs EXPECTED_DIGESTS.toml
    python3 scripts/verify_reproducibility.py --mint      # print computed witness digests (to (re)freeze)
    python3 scripts/verify_reproducibility.py --bundles   # verify the full Court Record bundle_root +
                                                          # evidence_root for all 20 datasets (from the
                                                          # newest `demo` run) vs EXPECTED_BUNDLE_ROOTS.toml
Exit 0 iff every present check matches its committed value (gated skips don't fail). The witness check
also asserts each demonstrator FIRED after its labelled onset, not only that the trace bytes reproduced.
See data/METRICS_DEFINITIONS.toml for the denominators behind the reported rates.
"""
import glob
import hashlib
import json
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))   # the edge crate dir
WS = os.path.dirname(os.path.dirname(HERE))                          # workspace root (holds output dirs)
INSTR = os.path.join(HERE, "data", "instrumented")
MANIFEST = os.path.join(INSTR, "EXPECTED_DIGESTS.toml")
BUNDLE_MANIFEST = os.path.join(HERE, "data", "EXPECTED_BUNDLE_ROOTS.toml")

# name -> shareable? (synthetic demonstrators are reproducible by anyone; gated need licensed data)
DEMONSTRATORS = [
    ("three_tank_instrumented", True),
    ("quadruple_tank_instrumented", True),
    ("cstr_instrumented", True),
    ("csth_instrumented", True),
    ("swat_t101_instrumented", False),
    ("batadal_t1_instrumented", False),
]


def canon_residual(s):
    """Round the balance residual to a 1e-6 grid as an integer, normalising signed zero to 0."""
    q = round(float(s) * 1e6)
    return str(int(q) if q != 0 else 0)


def witness_digest(name):
    """Regenerate the witness trace via the committed CLI; return (digest, fired) or None.

    - `digest`: SHA-256 of the canonical trace (per row `time_index:round(residual·1e6):grammar_state`).
    - `fired`: True iff a non-nominal grammar state (not NOM/SF) appears at or after the fault onset (the
      first `label != 0` row). The digest already covers the grammar_state column, so a stopped-firing
      regression changes the digest; `fired` is the explicit *result* assertion on top of that, so the
      verifier checks the witness fired — not only that the bytes reproduced (closes the exit-code gap).
    Returns None if the demonstrator's input CSV is absent (e.g. a gated dataset not prepped locally)."""
    csv = os.path.join(INSTR, f"{name}.csv")
    if not os.path.exists(csv):
        return None
    r = subprocess.run(
        ["cargo", "run", "--release", "-q", "-p", "dsfb-chemical-engineering-edge", "--",
         "balance-witness", name],
        cwd=os.path.dirname(os.path.dirname(HERE)), capture_output=True, text=True,
    )
    if r.returncode not in (0, 2):  # 0 = witness fired, 2 = ran but did not fire after onset; both valid here
        raise SystemExit(f"balance-witness {name} failed (exit {r.returncode}):\n{r.stderr}")
    trace = os.path.join(INSTR, f"{name}.witness.csv")
    h = hashlib.sha256()
    onset, fired = None, False
    with open(trace) as f:
        header = f.readline().strip().split(",")
        ri, gi, ti = header.index("balance_residual"), header.index("grammar_state"), header.index("time_index")
        li = header.index("label") if "label" in header else None
        for line in f:
            c = line.rstrip("\n").split(",")
            if len(c) <= max(ri, gi, ti):
                continue
            h.update(f"{c[ti]}:{canon_residual(c[ri])}:{c[gi]}\n".encode())
            # Disposition: first labelled-fault row is the onset; a non-nominal state there/after = fired.
            if li is not None and onset is None and c[li].strip() not in ("0", "0.0", ""):
                onset = int(c[ti])
            if onset is not None and int(c[ti]) >= onset and c[gi].strip().upper() not in ("NOM", "SF", ""):
                fired = True
    return h.hexdigest(), fired


def load_manifest():
    digests = {}
    if not os.path.exists(MANIFEST):
        return digests
    for line in open(MANIFEST):
        line = line.strip()
        if line.startswith("#") or "=" not in line:
            continue
        k, v = line.split("=", 1)
        digests[k.strip()] = v.strip().strip('"')
    return digests


def main():
    mint = "--mint" in sys.argv
    expected = {} if mint else load_manifest()
    rows, ok, failed = [], True, []
    for name, shareable in DEMONSTRATORS:
        res = witness_digest(name)
        if res is None:
            rows.append((name, "SKIP (input not present; gated data not prepped)", shareable))
            continue
        d, fired = res
        if mint:
            rows.append((name, d, shareable))
            continue
        want = expected.get(name)
        # Disposition guard: every balance demonstrator is constructed to FIRE after its labelled onset;
        # a run that reproduced the bytes but stopped firing is a real regression (the digest catches it
        # too, since grammar_state is hashed — this is the explicit, human-legible result assertion).
        if not fired:
            rows.append((name, "WITNESS DID NOT FIRE after onset (disposition regression)", shareable)); ok = False; failed.append(name)
        elif want is None:
            rows.append((name, f"NO EXPECTED DIGEST ({d})", shareable)); ok = False; failed.append(name)
        elif want == d:
            rows.append((name, "OK (fired; digest matches)", shareable))
        else:
            rows.append((name, f"MISMATCH got {d[:16]}… want {want[:16]}…", shareable)); ok = False; failed.append(name)

    if mint:
        print("# EXPECTED_DIGESTS.toml — canonical witness-trace SHA-256 (see verify_reproducibility.py).")
        print("# Synthetic demonstrators are reproducible by anyone; gated (swat/batadal) need licensed data.")
        for name, d, _ in rows:
            if d.startswith("SKIP"):
                print(f"# {name} = (skipped: input not present)")
            else:
                print(f'{name} = "{d}"')
        return 0

    print("reproducibility verification (canonical witness-trace digests):")
    for name, status, shareable in rows:
        tag = "synthetic" if shareable else "gated"
        print(f"  {name:32s} [{tag:9s}] {status}")
    if not ok:
        print(f"\nFAILED: {failed}")
        return 1
    print("\nall present demonstrators match their committed digest.")
    return 0


def load_bundle_manifest():
    """Parse EXPECTED_BUNDLE_ROOTS.toml's `[name]` sections into {name: (bundle_root, evidence_root)}."""
    out, cur = {}, None
    if not os.path.exists(BUNDLE_MANIFEST):
        return out
    for line in open(BUNDLE_MANIFEST):
        line = line.strip()
        if line.startswith("#") or not line:
            continue
        if line.startswith("[") and line.endswith("]"):
            cur = line[1:-1]
            out[cur] = {}
        elif "=" in line and cur is not None:
            k, v = line.split("=", 1)
            out[cur][k.strip()] = v.strip().strip('"')
    return {n: (d.get("bundle_root"), d.get("evidence_root")) for n, d in out.items()}


def newest_demo_dir():
    """The newest timestamped `demo` output dir under the workspace root, or None."""
    cands = sorted(glob.glob(os.path.join(WS, "output-dsfb-chemical-engineering", "2*")))
    return cands[-1] if cands else None


def verify_bundles():
    """Check every dataset's Court Record bundle_root + evidence_root (from the newest `demo` run)
    against the committed EXPECTED_BUNDLE_ROOTS.toml — the full operator artifact, not just witnesses."""
    run = newest_demo_dir()
    if run is None:
        print("no demo output found — run `cargo run --release -p dsfb-chemical-engineering-edge -- demo` first.")
        return 1
    expected = load_bundle_manifest()
    print(f"Court Record bundle verification (run: {os.path.relpath(run, WS)}):")
    ok, failed = True, []
    for d in sorted(glob.glob(os.path.join(run, "datasets", "*", "dsfb_chemical_engineering_casefile_v1", "casefile.json"))):
        j = json.load(open(d))
        name, br, er = j["dataset"], j["bundle_root"], j["evidence_root"]
        want = expected.get(name)
        if want is None:
            print(f"  {name:32s} NO EXPECTED ENTRY (bundle={br[:16]}…)"); ok = False; failed.append(name)
        elif (br, er) == want:
            print(f"  {name:32s} OK")
        else:
            print(f"  {name:32s} MISMATCH (bundle {br[:12]}… / evid {er[:12]}…)"); ok = False; failed.append(name)
    missing = set(expected) - {json.load(open(d))["dataset"]
                               for d in glob.glob(os.path.join(run, "datasets", "*", "dsfb_chemical_engineering_casefile_v1", "casefile.json"))}
    for name in sorted(missing):
        print(f"  {name:32s} EXPECTED BUT NOT IN RUN"); ok = False; failed.append(name)
    if not ok:
        print(f"\nFAILED: {failed}")
        return 1
    print(f"\nall {len(expected)} Court Record bundles match their committed roots.")
    return 0


if __name__ == "__main__":
    if "--bundles" in sys.argv:
        sys.exit(verify_bundles())
    sys.exit(main())
