# Failure Modes

DSFB is an observer-only layer. When it fails or stays ambiguous, the upstream monitoring stack still behaves exactly as before. The operator question is therefore not "did DSFB break the tool?" but "how should this DSFB output be interpreted?"

## 1. False Escalation

Cause

- repeated boundary pressure from maintenance transients, recipe transitions, or correlated nuisance structure

Observable signature

- Review or Escalate appears, but the surrounding upstream alarms remain short-lived and the condition resolves without failure follow-through

What the operator sees

- a structured episode with stronger language than the final process outcome justified

How to interpret

- DSFB detected persistent structure, but the structure did not correspond to a failure-producing event in that window

Mitigation

- compare against tool context such as chamber clean, lot transition, or release change
- keep the episode for audit, but downgrade trust in that motif until more field evidence exists

## 2. Missed Structure

Cause

- weak residual organization, sparse data, or a failure mode that does not produce sustained residual structure

Observable signature

- DSFB stays silent while the upstream stack still alarms or the event is discovered elsewhere

What the operator sees

- no DSFB episode, or only Watch-class activity, before a later known issue

How to interpret

- silence means DSFB did not see enough structured evidence in the residual side channel; it does not mean the upstream event was invalid

Mitigation

- keep upstream alarms authoritative
- review whether the residual tap omitted the relevant signal or whether the failure mode is primarily abrupt rather than structural

## 3. Fragmentation Edge Cases

Cause

- intermittent crossings, missing samples, or alternating short runs of pressure and recovery

Observable signature

- several short DSFB episodes appear where an operator would prefer one longer episode

What the operator sees

- repeated Watch or Review objects around one operational issue

How to interpret

- the residual structure is real, but the continuity is weak or broken

Mitigation

- inspect timestamps and missing-data gaps before treating each fragment as a separate issue
- use the traceability chain to see whether the breaks came from motif changes, grammar resets, or explicit policy suppression

## 4. High-Noise Environments

Cause

- unstable raw measurements, poor nominal reference quality, or persistent nuisance variation near the envelope

Observable signature

- elevated boundary activity, frequent motif churn, and low semantic stability

What the operator sees

- too many low-confidence episodes and reduced separation between important and unimportant events

How to interpret

- the side-channel is carrying structure, but the structure is not clean enough for aggressive policy promotion

Mitigation

- verify nominal reference quality
- review high-missingness or high-noise channels before broad deployment
- keep DSFB in advisory mode and calibrate against actual fab review burden instead of promoting more aggressive policy states
