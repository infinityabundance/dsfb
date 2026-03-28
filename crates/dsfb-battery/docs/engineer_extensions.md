# Engineer-Facing Extensions

These helper workflows are additive to the current paper-facing mono-cell path. They do not change the default `dsfb-battery-demo` behavior, existing production figures, or the production `stage2_detection_results.json` contract.

## Shadow-Mode Integration

The intended integration pattern is read-only and advisory-only:

1. ingest telemetry or upstream residuals in a non-interfering path
2. evaluate DSFB classifications and helper metrics
3. emit advisory output plus an optional validity/freshness token
4. ignore DSFB output when the token is absent or stale
5. do not feed DSFB output back into estimator tuning, actuation, or protection logic

## Adaptive Residual Handoff

The production crate still derives residuals from capacity relative to a healthy baseline. The helper integration mode can also read externally supplied residual, drift, and slew sequences. This is an interface convenience only; it does not claim empirical validation on adaptive ECM observers in the current dataset.

## Partial Observability

The current production path is single-channel and capacity-only. Because of that, the crate does not run a genuine partial-observability experiment here. The scaffolded helper notes instead describe how future multi-channel validity checks could mark insufficient observability when required channels are absent.

## Optional FFI Example

```c
#include "dsfb_battery_ffi.h"

DsfbBatteryConfig cfg = dsfb_battery_default_config();
DsfbBatterySummary summary = {0};
int rc = dsfb_battery_run_capacity_summary(capacities, len, cfg, &summary);
if (rc == 0) {
  /* advisory-only summary is now available in summary */
}
```

