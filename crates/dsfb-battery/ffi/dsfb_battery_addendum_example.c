// SPDX-License-Identifier: Apache-2.0

#include <stdio.h>

#include "../include/dsfb_battery_ffi.h"

int main(void) {
  DsfbBatteryConfig cfg = dsfb_battery_default_config();
  DsfbBatteryStepStatus step =
      dsfb_battery_evaluate_step_status(-0.06, 0.05, -0.003, -0.002, 12, 8, cfg);

  printf("state_code=%d\n", step.state_code);
  printf("color_code=%d\n", step.color_code);
  printf("reason_code=%d\n", step.reason_code);
  printf("advisory_only=%d\n", step.advisory_only);
  printf("valid=%d\n", step.valid);
  return 0;
}
