/* SPDX-License-Identifier: Apache-2.0 */

#include "../../include/dsfb_battery_ffi.h"

#include <stdio.h>

int main(void) {
  const double capacities[] = {
      2.000, 2.001, 1.999, 1.998, 1.996, 1.992, 1.988, 1.980, 1.970, 1.955,
      1.940, 1.925, 1.905, 1.885, 1.860, 1.835, 1.810, 1.785, 1.760, 1.735,
      1.710, 1.690, 1.670, 1.650, 1.630, 1.610, 1.595, 1.580, 1.565, 1.550};
  const size_t len = sizeof(capacities) / sizeof(capacities[0]);

  DsfbBatteryConfig cfg = dsfb_battery_default_config();
  DsfbBatterySummary summary = {0};

  if (dsfb_battery_run_capacity_summary(capacities, len, cfg, &summary) != 0) {
    fprintf(stderr, "dsfb_battery_run_capacity_summary failed\n");
    return 1;
  }

  printf("DSFB alarm cycle: %zu\n", summary.dsfb_alarm_cycle);
  printf("85%% threshold cycle: %zu\n", summary.threshold_85pct_cycle);
  printf("80%% EOL cycle: %zu\n", summary.eol_80pct_cycle);
  printf("First boundary cycle: %zu\n", summary.first_boundary_cycle);
  printf("First violation cycle: %zu\n", summary.first_violation_cycle);
  printf("Theorem t_star: %zu\n", summary.t_star);

  return 0;
}
