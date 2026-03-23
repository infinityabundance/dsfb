# Demo B Efficiency Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

This report separates aliasing-limited thin-point cases from variance-limited and mixed-width region cases so fixed-budget wins are not attributed only to sub-pixel line recovery.

| Scenario | Policy | Mean spp | ROI MAE |
| --- | --- | ---: | ---: |
| thin_reveal | uniform | 1.0 | 0.17242 |
| thin_reveal | uniform | 2.0 | 0.17226 |
| thin_reveal | uniform | 4.0 | 0.17206 |
| thin_reveal | uniform | 8.0 | 0.09564 |
| thin_reveal | combined_heuristic | 1.0 | 0.17242 |
| thin_reveal | combined_heuristic | 2.0 | 0.04977 |
| thin_reveal | combined_heuristic | 4.0 | 0.03184 |
| thin_reveal | combined_heuristic | 8.0 | 0.03184 |
| thin_reveal | native_trust | 1.0 | 0.17242 |
| thin_reveal | native_trust | 2.0 | 0.17215 |
| thin_reveal | native_trust | 4.0 | 0.03184 |
| thin_reveal | native_trust | 8.0 | 0.03184 |
| thin_reveal | imported_trust | 1.0 | 0.17242 |
| thin_reveal | imported_trust | 2.0 | 0.03184 |
| thin_reveal | imported_trust | 4.0 | 0.03184 |
| thin_reveal | imported_trust | 8.0 | 0.03184 |
| thin_reveal | hybrid_trust_variance | 1.0 | 0.17242 |
| thin_reveal | hybrid_trust_variance | 2.0 | 0.17200 |
| thin_reveal | hybrid_trust_variance | 4.0 | 0.03184 |
| thin_reveal | hybrid_trust_variance | 8.0 | 0.03184 |
| fast_pan | uniform | 1.0 | 0.01160 |
| fast_pan | uniform | 2.0 | 0.00804 |
| fast_pan | uniform | 4.0 | 0.00504 |
| fast_pan | uniform | 8.0 | 0.00267 |
| fast_pan | combined_heuristic | 1.0 | 0.01160 |
| fast_pan | combined_heuristic | 2.0 | 0.00603 |
| fast_pan | combined_heuristic | 4.0 | 0.00338 |
| fast_pan | combined_heuristic | 8.0 | 0.00237 |
| fast_pan | native_trust | 1.0 | 0.01160 |
| fast_pan | native_trust | 2.0 | 0.00629 |
| fast_pan | native_trust | 4.0 | 0.00319 |
| fast_pan | native_trust | 8.0 | 0.00232 |
| fast_pan | imported_trust | 1.0 | 0.01160 |
| fast_pan | imported_trust | 2.0 | 0.00220 |
| fast_pan | imported_trust | 4.0 | 0.00216 |
| fast_pan | imported_trust | 8.0 | 0.00216 |
| fast_pan | hybrid_trust_variance | 1.0 | 0.01160 |
| fast_pan | hybrid_trust_variance | 2.0 | 0.00381 |
| fast_pan | hybrid_trust_variance | 4.0 | 0.00218 |
| fast_pan | hybrid_trust_variance | 8.0 | 0.00216 |
| diagonal_reveal | uniform | 1.0 | 0.02606 |
| diagonal_reveal | uniform | 2.0 | 0.01607 |
| diagonal_reveal | uniform | 4.0 | 0.01243 |
| diagonal_reveal | uniform | 8.0 | 0.00527 |
| diagonal_reveal | combined_heuristic | 1.0 | 0.02606 |
| diagonal_reveal | combined_heuristic | 2.0 | 0.01245 |
| diagonal_reveal | combined_heuristic | 4.0 | 0.00770 |
| diagonal_reveal | combined_heuristic | 8.0 | 0.00770 |
| diagonal_reveal | native_trust | 1.0 | 0.02606 |
| diagonal_reveal | native_trust | 2.0 | 0.01243 |
| diagonal_reveal | native_trust | 4.0 | 0.00136 |
| diagonal_reveal | native_trust | 8.0 | 0.00770 |
| diagonal_reveal | imported_trust | 1.0 | 0.02606 |
| diagonal_reveal | imported_trust | 2.0 | 0.00770 |
| diagonal_reveal | imported_trust | 4.0 | 0.00770 |
| diagonal_reveal | imported_trust | 8.0 | 0.00770 |
| diagonal_reveal | hybrid_trust_variance | 1.0 | 0.02606 |
| diagonal_reveal | hybrid_trust_variance | 2.0 | 0.01245 |
| diagonal_reveal | hybrid_trust_variance | 4.0 | 0.00770 |
| diagonal_reveal | hybrid_trust_variance | 8.0 | 0.00770 |
| reveal_band | uniform | 1.0 | 0.02183 |
| reveal_band | uniform | 2.0 | 0.01459 |
| reveal_band | uniform | 4.0 | 0.00705 |
| reveal_band | uniform | 8.0 | 0.00368 |
| reveal_band | combined_heuristic | 1.0 | 0.02183 |
| reveal_band | combined_heuristic | 2.0 | 0.00972 |
| reveal_band | combined_heuristic | 4.0 | 0.00578 |
| reveal_band | combined_heuristic | 8.0 | 0.00377 |
| reveal_band | native_trust | 1.0 | 0.02183 |
| reveal_band | native_trust | 2.0 | 0.00932 |
| reveal_band | native_trust | 4.0 | 0.00584 |
| reveal_band | native_trust | 8.0 | 0.00393 |
| reveal_band | imported_trust | 1.0 | 0.02183 |
| reveal_band | imported_trust | 2.0 | 0.00343 |
| reveal_band | imported_trust | 4.0 | 0.00343 |
| reveal_band | imported_trust | 8.0 | 0.00343 |
| reveal_band | hybrid_trust_variance | 1.0 | 0.02183 |
| reveal_band | hybrid_trust_variance | 2.0 | 0.00533 |
| reveal_band | hybrid_trust_variance | 4.0 | 0.00343 |
| reveal_band | hybrid_trust_variance | 8.0 | 0.00343 |
| motion_bias_band | uniform | 1.0 | 0.04464 |
| motion_bias_band | uniform | 2.0 | 0.03229 |
| motion_bias_band | uniform | 4.0 | 0.01788 |
| motion_bias_band | uniform | 8.0 | 0.00829 |
| motion_bias_band | combined_heuristic | 1.0 | 0.04464 |
| motion_bias_band | combined_heuristic | 2.0 | 0.01716 |
| motion_bias_band | combined_heuristic | 4.0 | 0.01059 |
| motion_bias_band | combined_heuristic | 8.0 | 0.00703 |
| motion_bias_band | native_trust | 1.0 | 0.04464 |
| motion_bias_band | native_trust | 2.0 | 0.01789 |
| motion_bias_band | native_trust | 4.0 | 0.01042 |
| motion_bias_band | native_trust | 8.0 | 0.00731 |
| motion_bias_band | imported_trust | 1.0 | 0.04464 |
| motion_bias_band | imported_trust | 2.0 | 0.00974 |
| motion_bias_band | imported_trust | 4.0 | 0.00695 |
| motion_bias_band | imported_trust | 8.0 | 0.00695 |
| motion_bias_band | hybrid_trust_variance | 1.0 | 0.04464 |
| motion_bias_band | hybrid_trust_variance | 2.0 | 0.01498 |
| motion_bias_band | hybrid_trust_variance | 4.0 | 0.00712 |
| motion_bias_band | hybrid_trust_variance | 8.0 | 0.00695 |
| layered_slats | uniform | 1.0 | 0.02711 |
| layered_slats | uniform | 2.0 | 0.01985 |
| layered_slats | uniform | 4.0 | 0.01050 |
| layered_slats | uniform | 8.0 | 0.00526 |
| layered_slats | combined_heuristic | 1.0 | 0.02711 |
| layered_slats | combined_heuristic | 2.0 | 0.00881 |
| layered_slats | combined_heuristic | 4.0 | 0.00472 |
| layered_slats | combined_heuristic | 8.0 | 0.00381 |
| layered_slats | native_trust | 1.0 | 0.02711 |
| layered_slats | native_trust | 2.0 | 0.01018 |
| layered_slats | native_trust | 4.0 | 0.00486 |
| layered_slats | native_trust | 8.0 | 0.00390 |
| layered_slats | imported_trust | 1.0 | 0.02711 |
| layered_slats | imported_trust | 2.0 | 0.00353 |
| layered_slats | imported_trust | 4.0 | 0.00353 |
| layered_slats | imported_trust | 8.0 | 0.00353 |
| layered_slats | hybrid_trust_variance | 1.0 | 0.02711 |
| layered_slats | hybrid_trust_variance | 2.0 | 0.00763 |
| layered_slats | hybrid_trust_variance | 4.0 | 0.00353 |
| layered_slats | hybrid_trust_variance | 8.0 | 0.00353 |
| noisy_reprojection | uniform | 1.0 | 0.04592 |
| noisy_reprojection | uniform | 2.0 | 0.03290 |
| noisy_reprojection | uniform | 4.0 | 0.01811 |
| noisy_reprojection | uniform | 8.0 | 0.00879 |
| noisy_reprojection | combined_heuristic | 1.0 | 0.04592 |
| noisy_reprojection | combined_heuristic | 2.0 | 0.02067 |
| noisy_reprojection | combined_heuristic | 4.0 | 0.01380 |
| noisy_reprojection | combined_heuristic | 8.0 | 0.00740 |
| noisy_reprojection | native_trust | 1.0 | 0.04592 |
| noisy_reprojection | native_trust | 2.0 | 0.02149 |
| noisy_reprojection | native_trust | 4.0 | 0.01332 |
| noisy_reprojection | native_trust | 8.0 | 0.00742 |
| noisy_reprojection | imported_trust | 1.0 | 0.04592 |
| noisy_reprojection | imported_trust | 2.0 | 0.01056 |
| noisy_reprojection | imported_trust | 4.0 | 0.00703 |
| noisy_reprojection | imported_trust | 8.0 | 0.00702 |
| noisy_reprojection | hybrid_trust_variance | 1.0 | 0.04592 |
| noisy_reprojection | hybrid_trust_variance | 2.0 | 0.01499 |
| noisy_reprojection | hybrid_trust_variance | 4.0 | 0.00726 |
| noisy_reprojection | hybrid_trust_variance | 8.0 | 0.00702 |
| heuristic_friendly_pan | uniform | 1.0 | 0.01549 |
| heuristic_friendly_pan | uniform | 2.0 | 0.01030 |
| heuristic_friendly_pan | uniform | 4.0 | 0.00662 |
| heuristic_friendly_pan | uniform | 8.0 | 0.00297 |
| heuristic_friendly_pan | combined_heuristic | 1.0 | 0.01549 |
| heuristic_friendly_pan | combined_heuristic | 2.0 | 0.00692 |
| heuristic_friendly_pan | combined_heuristic | 4.0 | 0.00394 |
| heuristic_friendly_pan | combined_heuristic | 8.0 | 0.00265 |
| heuristic_friendly_pan | native_trust | 1.0 | 0.01549 |
| heuristic_friendly_pan | native_trust | 2.0 | 0.00711 |
| heuristic_friendly_pan | native_trust | 4.0 | 0.00396 |
| heuristic_friendly_pan | native_trust | 8.0 | 0.00274 |
| heuristic_friendly_pan | imported_trust | 1.0 | 0.01549 |
| heuristic_friendly_pan | imported_trust | 2.0 | 0.00252 |
| heuristic_friendly_pan | imported_trust | 4.0 | 0.00241 |
| heuristic_friendly_pan | imported_trust | 8.0 | 0.00241 |
| heuristic_friendly_pan | hybrid_trust_variance | 1.0 | 0.01549 |
| heuristic_friendly_pan | hybrid_trust_variance | 2.0 | 0.00475 |
| heuristic_friendly_pan | hybrid_trust_variance | 4.0 | 0.00241 |
| heuristic_friendly_pan | hybrid_trust_variance | 8.0 | 0.00241 |
| contrast_pulse | uniform | 1.0 | 0.00013 |
| contrast_pulse | uniform | 2.0 | 0.00008 |
| contrast_pulse | uniform | 4.0 | 0.00004 |
| contrast_pulse | uniform | 8.0 | 0.00002 |
| contrast_pulse | combined_heuristic | 1.0 | 0.00013 |
| contrast_pulse | combined_heuristic | 2.0 | 0.00008 |
| contrast_pulse | combined_heuristic | 4.0 | 0.00004 |
| contrast_pulse | combined_heuristic | 8.0 | 0.00002 |
| contrast_pulse | native_trust | 1.0 | 0.00013 |
| contrast_pulse | native_trust | 2.0 | 0.00008 |
| contrast_pulse | native_trust | 4.0 | 0.00004 |
| contrast_pulse | native_trust | 8.0 | 0.00002 |
| contrast_pulse | imported_trust | 1.0 | 0.00013 |
| contrast_pulse | imported_trust | 2.0 | 0.00008 |
| contrast_pulse | imported_trust | 4.0 | 0.00004 |
| contrast_pulse | imported_trust | 8.0 | 0.00002 |
| contrast_pulse | hybrid_trust_variance | 1.0 | 0.00013 |
| contrast_pulse | hybrid_trust_variance | 2.0 | 0.00008 |
| contrast_pulse | hybrid_trust_variance | 4.0 | 0.00004 |
| contrast_pulse | hybrid_trust_variance | 8.0 | 0.00002 |
| stability_holdout | uniform | 1.0 | 0.01007 |
| stability_holdout | uniform | 2.0 | 0.00672 |
| stability_holdout | uniform | 4.0 | 0.00473 |
| stability_holdout | uniform | 8.0 | 0.00274 |
| stability_holdout | combined_heuristic | 1.0 | 0.01007 |
| stability_holdout | combined_heuristic | 2.0 | 0.00363 |
| stability_holdout | combined_heuristic | 4.0 | 0.00268 |
| stability_holdout | combined_heuristic | 8.0 | 0.00216 |
| stability_holdout | native_trust | 1.0 | 0.01007 |
| stability_holdout | native_trust | 2.0 | 0.00412 |
| stability_holdout | native_trust | 4.0 | 0.00264 |
| stability_holdout | native_trust | 8.0 | 0.00215 |
| stability_holdout | imported_trust | 1.0 | 0.01007 |
| stability_holdout | imported_trust | 2.0 | 0.00672 |
| stability_holdout | imported_trust | 4.0 | 0.00473 |
| stability_holdout | imported_trust | 8.0 | 0.00274 |
| stability_holdout | hybrid_trust_variance | 1.0 | 0.01007 |
| stability_holdout | hybrid_trust_variance | 2.0 | 0.00692 |
| stability_holdout | hybrid_trust_variance | 4.0 | 0.00406 |
| stability_holdout | hybrid_trust_variance | 8.0 | 0.00266 |

## Scenario Taxonomy

- `thin_reveal`: taxonomy=`aliasing_limited`, sampling_taxonomy=`coverage-dominated point reveal`
- `fast_pan`: taxonomy=`mixed`, sampling_taxonomy=`thin-band reveal with textured background`
- `diagonal_reveal`: taxonomy=`aliasing_limited`, sampling_taxonomy=`subpixel diagonal coverage case`
- `reveal_band`: taxonomy=`mixed`, sampling_taxonomy=`mixed-width reveal band with aliasing and texture`
- `motion_bias_band`: taxonomy=`mixed`, sampling_taxonomy=`motion-mismatch reveal band`
- `layered_slats`: taxonomy=`mixed`, sampling_taxonomy=`layered slat reveal with mixed stable and unstable zones`
- `noisy_reprojection`: taxonomy=`variance_limited`, sampling_taxonomy=`realism-stress reveal with noisy reprojection`
- `heuristic_friendly_pan`: taxonomy=`edge_trap`, sampling_taxonomy=`competitive baseline reveal`
- `contrast_pulse`: taxonomy=`variance_limited`, sampling_taxonomy=`negative control`
- `stability_holdout`: taxonomy=`variance_limited`, sampling_taxonomy=`negative control`

## What Is Not Proven

- This study does not prove an optimal sampling controller or general renderer superiority.
- External validation is still required on real renderer noise and imported engine buffers.

## Remaining Blockers

- Demo B remains synthetic and still needs real-engine noise and shading complexity for full production confidence.
- External handoff exists for Demo A style supervision, but Demo B still lacks an external renderer allocation trace.
