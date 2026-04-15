# Changelog

All notable changes to `dsfb-gray` should be recorded here.

The release discipline for this repository is:

- document public API additions and behavior changes
- document scoring-method changes explicitly
- document generated-artifact format changes explicitly
- avoid silent semver-significant changes to observer, scanner, or attestation outputs

## Unreleased

- Added `TelemetryAdapter`, static-prior support, and `ReasonEvidence` to the runtime observer surface.
- Added reproducible public-artifact generation through `dsfb-regenerate-public-artifacts`.
- Added generated README / paper result and evidence sections.
- Added scan profiles, remediation guidance, evidence IDs, and derived runtime structural priors.
