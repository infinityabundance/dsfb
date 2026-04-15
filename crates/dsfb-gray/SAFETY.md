# Safety Notes

## Current Safety Posture

The core DSFB observer is designed to preserve a strict non-interference
contract:

- telemetry is accepted through immutable references
- observer removal must not change upstream system behavior
- the crate denies `unsafe_code`
- the core observer remains available without the default `std` feature

## What The Crate Does Not Claim

`dsfb-gray` does not claim:

- certification against any safety standard
- proof of correctness for a target system
- proof of worst-case execution time
- proof that a scanned crate is suitable for any DAL, SIL, ASIL, or mission role

## Review Expectations

Users applying DSFB in safety-relevant environments should review:

- admissibility-envelope calibration
- hysteresis and persistence-window assumptions
- static audit findings related to boundedness and state handling
- scan report caveats and evidence blocks
- the target system's own safety case and verification evidence

## Audit Interpretation

The DSFB Gray Static Crate Scan Report is a structured improvement and
review-readiness instrument. It can support internal standards-oriented review,
but it is not itself a safety certificate.
