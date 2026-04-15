# Security Policy

## Scope

`dsfb-gray` is a read-only telemetry interpretation and crate-audit library.
Security-sensitive areas include:

- scan artifact signing and DSSE export
- file-system handling for scan output generation
- deterministic parser and report-generation code paths
- any future networked or hosted integrations

## Reporting

Report suspected vulnerabilities privately to:

- `riaan@invariantforge.net`

Include:

- affected version or commit
- impact summary
- minimal reproduction steps
- whether the issue affects the runtime observer, the static scanner, or both

## Response Goals

- acknowledge receipt within 5 business days
- confirm severity and next action after triage
- publish a fix or mitigation note when available

## Supported Review Posture

The project treats the following as security-relevant review surfaces:

- unsafe and FFI boundaries
- panic paths in scanner/export code
- artifact-signing and provenance metadata
- raw-handle and file-lifecycle paths
- dependency and supply-chain drift

DSFB audit reports are guidance for code-quality and review readiness. They are
not a security certification.
