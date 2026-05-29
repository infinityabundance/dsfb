"""Orchestrator for the DSFB figure gallery:  python3 -m figures [--run <dir>] [--log <file>] [--groups ABC]

Renders every catalogue group into paper/figures/ (PNG + PDF), writes the figure-provenance manifest
(figure_manifest.json), and appends a verbose record to the build log. Deterministic: re-running over the
same input run produces byte-identical figures (see style.py). Does NOT build the paper.

Group letters: A core · B atlas · C perf/CUDA · D graphs · E forensics · F results · G physics · H regime ·
I practitioner. Default renders all implemented groups.
"""
import os
import sys

from . import style as S

# Map group letter -> render_all(run) callable. Groups are registered as their modules are implemented.
GROUPS = {}


def _register():
    """Import and register each implemented group module (kept lazy so a partial checkout still runs)."""
    from . import core
    GROUPS["A"] = core.render_all
    for letter, modname in (("B", "atlas"), ("C", "perf"), ("D", "graphs"),
                            ("E", "forensics"), ("F", "results"), ("G", "physics"),
                            ("H", "regime"), ("I", "practitioner")):
        try:
            mod = __import__(f"figures.{modname}", fromlist=["render_all"])
            GROUPS[letter] = mod.render_all
        except Exception as e:  # a group not yet present must not abort the others
            S.log(f"  (group {letter}/{modname} not available: {e})")


def main(argv):
    run = S.latest_run()
    log_path = None
    groups = None
    if "--run" in argv:
        run = argv[argv.index("--run") + 1]
    if "--log" in argv:
        log_path = argv[argv.index("--log") + 1]
    if "--groups" in argv:
        groups = argv[argv.index("--groups") + 1].upper()
    S.set_log(log_path)
    S.log(f"DSFB figure gallery — rendering from run: {run}")
    S.log(f"output dir: {S.FIGDIR}")
    _register()
    selected = [g for g in (groups or "".join(sorted(GROUPS))) if g in GROUPS]
    S.log(f"groups: {', '.join(selected)}")
    for g in selected:
        GROUPS[g](run)
    S.write_manifest(os.path.join(run, "figure_manifest.json"), source_run=run)
    # Also drop a copy of the manifest next to the figures for the paper/zip bundle.
    S.write_manifest(os.path.join(S.FIGDIR, "figure_manifest.json"), source_run=run)
    S.log(f"done — {len(S.MANIFEST)} figures rendered.")


if __name__ == "__main__":
    main(sys.argv[1:])
