# MATLAB / Simulink Wrapper Scaffold

This directory contains a pre-built wrapper path for host environments that have MATLAB/Simulink toolchains available.

Included files:

- `dsfb_battery_sfun_stub.c`
- `example_block_io.md`

The wrapper is a scaffold only:

- it is not compiled by the crate
- it is not presented as a verified Simulink deployment
- it reuses the narrow C ABI exposed by `include/dsfb_battery_ffi.h`
