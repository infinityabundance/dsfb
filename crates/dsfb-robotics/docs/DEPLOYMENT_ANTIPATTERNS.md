# Deployment Anti-Patterns — `dsfb-robotics`

This document lists the five most common ways DSFB output is
**incorrectly** consumed in deployment, alongside the correct usage
for each. The companion paper §11 (Worked Example), §10.21 ("Where
DSFB Adds Nothing"), and §12.5 (Threats to Validity) cover the same
ground in academic terms; this document is the operator-facing
practitioner guide.

If you are about to wire DSFB into a production pipeline, read this
first. If you are reviewing someone else's DSFB integration, this is
the checklist.

---

## Anti-pattern 1: Treating DSFB as a fault classifier

**Don't:** Read a Boundary or Violation episode and conclude
"the bearing is faulted" / "the controller is broken" / "the model
is wrong". The grammar tag names a *structural* posture relative to
the calibration envelope, not a *physical* posture relative to a
fault library.

**Do:** Read a non-Admissible episode as "the residual is structured
here, an operator should look at this segment of the trajectory".
The downstream physical interpretation is the operator's job, using
domain expertise the framework cannot provide.

---

## Anti-pattern 2: Using compression-ratio as a quality score

**Don't:** Score datasets by their compression ratio
($(B + V)/N$) and conclude "the higher the compression, the
healthier the system" or vice versa. The compression ratio is a
*structural fingerprint* of the underlying experiment / task /
trajectory shape, not a quality metric. \S10.4 (FEMTO-ST) shows
compression 0.479 with 1219 Violations because the experiment is a
run-to-failure trajectory; \S10.20 (Sorrentino) shows compression
0.891 with 3387 Violations because the experiment is a deliberately-
perturbed balancing trial. Neither dataset is "higher quality" or
"lower quality" than the other.

**Do:** Compare compression ratios *within* a single deployment over
time, not across deployments. A drift in compression ratio at fixed
calibration is one signal worth a triage look. A cross-deployment
compression-ratio leaderboard is meaningless.

---

## Anti-pattern 3: Tuning $\beta$ to hit a target compression

**Don't:** Adjust the boundary-fraction $\beta$ until the
compression ratio matches a desired value (e.g. "we want 5 %
review surface"). The sensitivity grid (\S10.X in the paper)
documents that $\beta$ has the largest single-parameter influence
on compression: spread of 0.392 across $\beta \in \{0.3, 0.4, 0.5,
0.6, 0.7\}$. Tuning $\beta$ at deployment time defeats the
pre-registered protocol freeze (`paper-lock-protocol-frozen-v1`)
and destroys the cross-deployment comparability of the structural
surface.

**Do:** Use the canonical $(W=8, K=4, \beta=0.5, \delta_s=0.05)$
parameter set as shipped. If the structural surface is too noisy
for your operator workload, the right intervention is upstream
(narrow the calibration window, tighten the residual-source
preprocessing, increase the noise-floor cut), not knob-tuning the
FSM.

---

## Anti-pattern 4: Treating the explain narrative as a diagnosis

**Don't:** Show the `paper-lock --explain` narrative to an end user
as a fault explanation. The narrative is a *post-commit description*
of the grammar's reasoning ("Boundary triggered when the residual
entered the (β·ρ, ρ] band with sustained-outward-drift or
abrupt-slew or recurrent-grazing structure"); it is not a physical
explanation of the underlying robot state.

**Do:** Use the explain narrative as a triage *prompt*: "the FSM
committed Boundary at index 1882 with this structural condition;
the operator should now look at the raw residual trace, the upstream
controller logs, and the physical state of the system at this
sample." The narrative is a starting point for a human review, not
the conclusion.

---

## Anti-pattern 5: Forgetting the read-only / non-interference contract

**Don't:** Wire DSFB output back into a control loop or as an
emergency stop trigger. The framework is designed as a strictly
read-only side-channel observer. Any closed-loop integration
(grammar state → controller action) is outside the framework's
scope and breaks the audit trail; it is also outside the safety
analysis the framework supports.

**Do:** Pipe DSFB output to a human-review channel (operator
dashboard, ticketing queue, audit log) only. If a closed-loop
intervention is needed, design it independently with separate
safety analysis; DSFB can serve as one input among many but
cannot be the sole controller-modifying signal.

---

## Operator checklist before deployment

- [ ] You have read companion paper §10.21 ("Where DSFB Adds Nothing
      Structurally") and confirmed your dataset is not in the
      silent-augment posture.
- [ ] You have read §12.5 (Threats to Validity) and identified
      which threats apply to your residual stream.
- [ ] Your deployment uses the canonical $(W, K, \beta, \delta_s)$
      tuple and explicitly documents any deviation.
- [ ] DSFB output flows into a human-review channel only.
- [ ] You have run `bash scripts/reproduce.sh` and your local
      `audit/checksums.txt` matches the committed version.
- [ ] You have read this anti-patterns document end-to-end.
