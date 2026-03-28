# MATLAB / Simulink Wrapper Path

Status: Wrapper scaffold only. No verified Simulink deployment claim is made.

Wrapper files:

- `wrappers/matlab/dsfb_battery_sfun_stub.c`
- `wrappers/matlab/README.md`
- `wrappers/matlab/example_block_io.md`

Current approach:

- reuse the crate's narrow C ABI from `include/dsfb_battery_ffi.h`
- call `dsfb_battery_evaluate_step_status` for step-oriented block output
- expose:
  - residual
  - envelope rho
  - drift
  - slew
  - drift counter
  - slew counter
- output:
  - grammar state code
  - tri-state color code
  - reason-code code

The wrapper is a build scaffold intended for a host environment with MATLAB/Simulink toolchains. It is not compiled or executed by the crate itself.
