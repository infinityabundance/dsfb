# Tamper-Evident Residuals

Status: Integrity helper inspired by tamper-evident goals. This is not a literal NIST SP 800-193 compliance claim.

The addendum helper emits:

- `outputs/addendum/.../integrity/tamper_evident_trace.json`
- `outputs/addendum/.../integrity/tamper_evident_verification.json`

Mechanism:

1. Hash the configuration and input series.
2. Seed a chain state from those digests.
3. For each residual/audit record, hash:
   - previous digest
   - cycle
   - grammar state
   - reason code
   - residual
   - drift
   - slew
4. Store the per-record digest and the final root digest.
5. Recompute the chain in verification mode.

This provides a local tamper-evident sequence check for addendum artifacts. It does not replace platform-level secure boot, signing, or recovery controls.
