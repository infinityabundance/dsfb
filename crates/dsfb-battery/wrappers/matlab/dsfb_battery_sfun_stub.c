/* SPDX-License-Identifier: Apache-2.0
 *
 * Simulink S-Function style scaffold for the dsfb-battery addendum layer.
 * This file is intentionally a build scaffold and is not compiled by Cargo.
 */

#include "../../include/dsfb_battery_ffi.h"

/*
 * Expected block inputs:
 *   u[0] residual
 *   u[1] envelope_rho
 *   u[2] drift
 *   u[3] slew
 *   u[4] drift_counter
 *   u[5] slew_counter
 *
 * Expected block outputs:
 *   y[0] state_code
 *   y[1] color_code
 *   y[2] reason_code
 *   y[3] valid
 */

void dsfb_battery_step_stub(const double *u, double *y) {
  DsfbBatteryConfig cfg = dsfb_battery_default_config();
  DsfbBatteryStepStatus step =
      dsfb_battery_evaluate_step_status(u[0], u[1], u[2], u[3],
                                        (size_t)u[4], (size_t)u[5], cfg);
  y[0] = (double)step.state_code;
  y[1] = (double)step.color_code;
  y[2] = (double)step.reason_code;
  y[3] = (double)step.valid;
}
