# Addendum Layer

This directory contains engineer-facing addendum artifacts that extend the `dsfb-battery` crate beyond the current mono-cell paper scope.

These files are additive only. They do not modify the production mono-cell workflow, the Colab-facing figure path, or the production `stage2_detection_results.json` contract.

Contents:

- `mosa_compatibility.md`
- `mosa_component_map.json`
- `icd.md`
- `ffi_integration.md`
- `nasa_power_of_10_alignment.md`
- `eu_battery_passport_mapping.md`
- `assurance_mapping.md`
- `tamper_evident_residuals.md`
- `matlab_simulink_wrapper.md`
- `mission_bus_mapping.md`
- `seu_resilience.md`

All generated runtime addendum artifacts are written by the opt-in `dsfb-battery-addendum` helper under `outputs/addendum/...`.
