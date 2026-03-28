// SPDX-License-Identifier: Apache-2.0

#ifndef DSFB_BATTERY_FFI_H
#define DSFB_BATTERY_FFI_H

#include <stddef.h>
#include <stdint.h>

typedef struct {
  size_t healthy_window;
  size_t drift_window;
  size_t drift_persistence;
  size_t slew_persistence;
  double drift_threshold;
  double slew_threshold;
  double eol_fraction;
  double boundary_fraction;
} DsfbBatteryConfig;

typedef struct {
  size_t dsfb_alarm_cycle;
  size_t threshold_85pct_cycle;
  size_t eol_80pct_cycle;
  size_t first_boundary_cycle;
  size_t first_violation_cycle;
  size_t t_star;
} DsfbBatterySummary;

DsfbBatteryConfig dsfb_battery_default_config(void);
int32_t dsfb_battery_evaluate_grammar_state(
    double residual,
    double envelope_rho,
    double drift,
    double slew,
    size_t drift_counter,
    size_t slew_counter,
    DsfbBatteryConfig config);
int32_t dsfb_battery_run_capacity_summary(
    const double *capacities_ptr,
    size_t len,
    DsfbBatteryConfig config,
    DsfbBatterySummary *out_summary);

#endif

