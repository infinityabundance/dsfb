# Demo B — Engine-Native Capture

ENGINE_NATIVE_CAPTURE_MISSING=true

**engine_source_category:** pending

**fixed_budget_equal:** true (all policies enforce identical total sample budget)

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

## Policies Compared

1. Uniform baseline
2. Gradient magnitude
3. Local contrast
4. Variance proxy
5. Combined heuristic
6. DSFB imported trust
7. Hybrid trust+variance

## Demo B: PENDING

No real engine-native capture was provided. Demo B allocation cannot be evaluated.

### Expected Demo B output when capture is provided

| Policy | Mean samples/px | ROI coverage | Non-ROI penalty |
|--------|----------------|-------------|-----------------|
| uniform | TBD | TBD | TBD |
| gradient | TBD | TBD | TBD |
| contrast | TBD | TBD | TBD |
| variance | TBD | TBD | TBD |
| combined_heuristic | TBD | TBD | TBD |
| DSFB imported trust | TBD | TBD | TBD |
| hybrid | TBD | TBD | TBD |

## Proxy vs Renderer-Integrated Distinction

This is a **proxy allocation study**: sample counts are allocated by policy but are not fed back into a renderer sampling loop. **Renderer-integrated sampling — where the allocated counts actually drive a real-time render pass — is still pending.** This requires explicit renderer integration work beyond buffer export.

## aliasing vs variance Coverage

- aliasing pressure: high gradient magnitude signals edge/feature pressure
- variance pressure: temporal variance proxy signals noise/instability pressure
- Both are evaluated per-capture when a real capture is provided

## What Is Not Proven

- Renderer-integrated sample feedback is not proven (proxy allocation only)
- Engine-native Demo B on real capture is pending

## Remaining Blockers

- **EXTERNAL**: No real engine capture has been provided.
- **EXTERNAL**: Renderer-integrated sampling requires engine integration work.
