# Blocker Check

## Removed

- Host-realistic DSFB mode exists and is reported separately from visibility-assisted mode.
- Stronger baselines are present and scored across multiple scenarios.
- A bounded neutral scenario is included to expose false positives.
- Demo B enforces fixed-budget fairness across multiple policies.

## Partially Removed

- Strong heuristic baselines are now explicit, but they remain competitive on some scenarios.
- Cost confidence is better because buffers and stages are explicit; GPU timing is now measured at 160×96 (RTX 4080 SUPER: 0.29 ms dispatch) and at 854×480 via DAVIS/Sintel external replay (RTX 4080 SUPER: ~3.5 ms dispatch). CPU proxy timing is available at higher resolutions. Engine-integrated GPU profiling still requires a real engine capture.
- 4K synthetic probe executed via wgpu with raised binding limits (max_storage_buffer_binding_size=u32::MAX). See gpu_execution_report.md for probe result.

## Remaining

- The scenario suite is still synthetic and does not prove production-scene generalization.
- The strong heuristic baseline remains competitive on some cases, so the crate supports evaluation diligence rather than universal win claims.
- Real engine capture remains the primary remaining external blocker. Frame graph position, async compute compatibility, LDS optimization, and the 4K dispatch probe are all complete internally.
