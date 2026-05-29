#!/usr/bin/env python3
"""P36: scope-stratify the SWaT T101 balance-witness recall against the OFFICIAL iTrust attack list.

The closure-gate applicability criterion predicts a balance witness fires only when a fault makes a
*conserved* quantity *appear* non-conserved at the meters. For the stage-1 T101 volume balance
(area*dLIT101/dt = FIT101 - FIT201), that means a **sensor spoof on a balance term** (LIT101 / FIT101 /
FIT201). So before counting recall we must separate the attacks the witness *can* see (in-scope) from
those it is *structurally blind* to (out-of-scope, e.g. stage-2..6 points) -- otherwise "13 of 35 fire"
mixes apples and oranges.

GROUND TRUTH (not a derivation): the official iTrust `List_of_attacks_Final` (the 2015 SWaT campaign,
the same 28/12/2015-2/01/2016 run as attack.csv), which names the Attack Point and Start/End time of
every attack. We classify each reconstructed attack segment by timestamp-overlap with that list.

DATA IS NOT REDISTRIBUTED. Inputs live under SWAT_DIR (gitignored): normal.csv, attack.csv, and
List_of_attacks_Final.pdf. This script (the recipe) is committed; only the computed recall result is
disclosed in the paper / docs. Re-uses prep_swat.py's exact segmentation so windows align 1:1.

Run:  python3 scripts/swat_scope_recall.py
"""
import csv
import os
import re
import subprocess
from datetime import datetime

HERE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(HERE, "data", "instrumented")
SWAT_DIR = os.environ.get("SWAT_DIR", os.path.join(OUT, "raw", "swat"))
# The witness trace (derived from controlled SWaT bytes) is NEVER tracked → read from the untracked research/
# quarantine (DSFB_CONTROLLED_DIR overrides).
CONTROLLED = os.environ.get("DSFB_CONTROLLED_DIR", os.path.join(HERE, "..", "..", "research", "controlled"))
BLOCK = 30  # must match prep_swat.py

# A balance term of the T101 closure -> a spoof here makes the conserved volume appear non-conserved.
IN_SCOPE_POINTS = ("LIT101", "FIT101", "FIT201")


def parse_ts(s):
    s = s.strip()
    for fmt in ("%d/%m/%Y %I:%M:%S %p", "%d/%m/%Y %H:%M:%S"):
        try:
            return datetime.strptime(s, fmt)
        except ValueError:
            pass
    return None


def norm_point(p):
    """Normalise an attack-point field to a comparable token: drop the Unicode hyphen (U+2010) the
    iTrust PDF uses, plus ASCII hyphens, commas, semicolons and spaces — so a multi-point field like
    "MV-101, LIT-101" becomes "MV101LIT101", and `"LIT101" in norm` correctly detects the balance term."""
    return re.sub(r"[\s,;‐-]", "", p).upper()


def load_attack_list():
    """Parse the official iTrust attack list (PDF -> text) into [(num, start_dt, end_dt, point_norm)].

    Each attack row spans one calendar day; Start/End are clock times on that day. We pair the End time
    to the row's date (attacks do not cross midnight in this list)."""
    pdf = os.path.join(SWAT_DIR, "List_of_attacks_Final.pdf")
    if not os.path.exists(pdf):
        raise SystemExit(f"attack list not found: {pdf}")
    txt = subprocess.run(["pdftotext", "-layout", pdf, "-"], capture_output=True, text=True).stdout
    attacks = []
    # e.g.  "   3  28/12/2015 11:22:00   11:28:22   LIT-101   Water level ..."  and multi-point fields
    # like "21 ... MV-101, LIT-101   MV-101 is open; ...". Capture the whole Attack-Point column up to
    # the 2+-space gap before the next column (so comma-separated multi-point attacks are kept whole).
    pat = re.compile(r"^\s*(\d+)\s+(\d{1,2}/\d{1,2}/\d{4})\s+(\d{1,2}:\d{2}:\d{2})\s+(\d{1,2}:\d{2}:\d{2})\s+(.+?)\s{2,}")
    for line in txt.splitlines():
        m = pat.match(line)
        if not m:
            continue
        num, date, start, end, point = m.groups()
        s_dt = parse_ts(f"{date} {start}")
        e_dt = parse_ts(f"{date} {end}")
        if s_dt and e_dt:
            attacks.append((int(num), s_dt, e_dt, norm_point(point)))
    return attacks


def read_attack_csv():
    """attack.csv rows as (ts, label) — only the timestamp + Normal/Attack flag are needed here."""
    path = os.path.join(SWAT_DIR, "attack.csv")
    rows = []
    with open(path) as f:
        r = csv.reader(f)
        h = [c.strip() for c in next(r)]
        ci = {c: i for i, c in enumerate(h)}
        ilab = ci.get("Normal/Attack")
        for row in r:
            ts = parse_ts(row[0])
            lab = row[ilab].strip() if ilab is not None else "Attack"
            rows.append((ts, lab))
    return rows


def reconstruct_segments(atk):
    """Replicate prep_swat.py's segmentation: contiguous-time runs (gap > 2 s) with >= BLOCK rows.
    Returns the ordered [(start_ts, end_ts)] of each segment that becomes a labelled window."""
    segs, cur = [], [atk[0]]
    for i in range(1, len(atk)):
        if atk[i][0] is None or atk[i - 1][0] is None or (atk[i][0] - atk[i - 1][0]).total_seconds() > 2:
            segs.append(cur)
            cur = []
        cur.append(atk[i])
    segs.append(cur)
    out = []
    for s in segs:
        if len(s) >= BLOCK:
            ts = [r[0] for r in s if r[0] is not None]
            if ts:
                out.append((min(ts), max(ts)))
    return out


def classify(seg_range, attacks):
    """Match a segment's [start,end] to the overlapping attack(s); in-scope iff any touches a balance term."""
    s0, s1 = seg_range
    hit = [(n, p) for (n, _s, _e, p) in attacks if not (_e < s0 or _s > s1)]
    pts = {p for _, p in hit}
    in_scope = any(any(bt in p for bt in IN_SCOPE_POINTS) for p in pts)
    nums = sorted(n for n, _ in hit)
    return in_scope, nums, sorted(pts)


def witness_fired_windows():
    """Read the committed witness trace; return per-labelled-window FIRED (any non-nominal/non-SF state),
    in order, plus the normal-run (label==0) false-positive rate."""
    path = os.path.join(CONTROLLED, "swat_t101_instrumented.witness.csv")
    rows = list(csv.DictReader(open(path)))
    windows, cur = [], None
    norm_n = norm_fire = 0
    for r in rows:
        lab = int(float(r["label"]))
        st = r["grammar_state"].strip().upper()
        fired = st not in ("NOM", "SF", "")
        if lab == 1:
            if cur is None:
                cur = False
            cur = cur or fired
        else:
            if cur is not None:
                windows.append(cur)
                cur = None
            norm_n += 1
            norm_fire += 1 if fired else 0
    if cur is not None:
        windows.append(cur)
    fp = 100.0 * norm_fire / norm_n if norm_n else float("nan")
    return windows, fp, norm_fire, norm_n


def main():
    attacks = load_attack_list()
    in_scope_attacks = sorted(n for (n, _s, _e, p) in attacks if any(bt in p for bt in IN_SCOPE_POINTS))
    print(f"ground truth: {len(attacks)} physical-impact attacks parsed; "
          f"{len(in_scope_attacks)} touch a T101 balance term {IN_SCOPE_POINTS} -> in-scope: {in_scope_attacks}")

    segs = reconstruct_segments(read_attack_csv())
    fired, fp, nf, nn = witness_fired_windows()
    if len(segs) != len(fired):
        print(f"WARNING: {len(segs)} reconstructed segments vs {len(fired)} witness windows -- "
              f"alignment may be off; reporting on min().")
    n = min(len(segs), len(fired))

    in_tot = in_fire = out_tot = out_quiet = 0
    rows_report = []
    for i in range(n):
        in_scope, nums, pts = classify(segs[i], attacks)
        f = fired[i]
        rows_report.append((i, segs[i][0].strftime("%d %H:%M"), nums, pts, in_scope, f))
        if in_scope:
            in_tot += 1
            in_fire += 1 if f else 0
        else:
            out_tot += 1
            out_quiet += 1 if not f else 0

    print(f"\nwindows: {n}  |  fired total: {sum(1 for r in rows_report if r[5])}")
    print(f"IN-SCOPE  (balance-term spoof): {in_fire}/{in_tot} fired  ->  within-scope recall "
          f"{100.0*in_fire/in_tot:.0f}%" if in_tot else "no in-scope windows")
    print(f"OUT-OF-SCOPE (other stages):    {out_quiet}/{out_tot} correctly quiet "
          f"({100.0*out_quiet/out_tot:.0f}% specificity)" if out_tot else "")
    print(f"normal-run false positives:     {nf}/{nn} = {fp:.1f}%")
    print("\nper-window (idx, seg-start, attack#s, points, in_scope, FIRED):")
    for r in rows_report:
        print(f"  {r[0]:2d}  {r[1]}  #{r[2]}  {r[3]}  in_scope={r[4]}  FIRED={r[5]}")


if __name__ == "__main__":
    main()
