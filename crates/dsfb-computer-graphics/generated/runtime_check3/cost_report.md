# Cost Report

“The experiment is intended to demonstrate behavioral differences rather than establish optimal performance.”

“The DSFB supervisory layer can be implemented with local operations and limited temporal memory, with expected cost scaling linearly with pixel count and amenable to reduced-resolution evaluation.”

“The framework is compatible with tiled and asynchronous GPU execution.”

## Mode

- Host-realistic mode

## Buffers

| Buffer | Bytes / pixel | Notes |
| --- | ---: | --- |
| residual | 4 | single-channel scalar residual |
| depth disagreement | 4 | single-channel depth cue |
| normal disagreement | 4 | single-channel normal cue |
| motion disagreement | 4 | single-channel motion cue |
| neighborhood inconsistency | 4 | single-channel neighborhood cue |
| trust | 4 | single-channel supervisory trust |
| alpha | 4 | single-channel alpha modulation |
| intervention | 4 | single-channel hazard / response strength |

## Stages

| Stage | Approx ops / pixel | Reads / pixel | Writes / pixel | Reduction note |
| --- | ---: | ---: | ---: | --- |
| Residual evaluation | 10 | 2 | 1 | Local arithmetic only |
| Depth/normal disagreement | 12 | 4 | 2 | Can share reprojection fetches |
| Motion / neighborhood proxies | 18 | 8 | 2 | Tile aggregation is viable |
| Trust and alpha update | 14 | 6 | 3 | Trust may run at half resolution |
| Blend modulation | 6 | 2 | 1 | Fuse with temporal resolve |

## Resolution Footprints

| Resolution | Pixels | Approx memory (MB) |
| --- | ---: | ---: |
| 1280x720 | 921600 | 28.12 |
| 1920x1080 | 2073600 | 63.28 |
| 3840x2160 | 8294400 | 253.12 |

## Notes

- Host-realistic mode excludes synthetic visibility hints and uses only signals plausible in an engine temporal pipeline.
- The DSFB supervisory layer can be implemented with local operations and limited temporal memory, with expected cost scaling linearly with pixel count and amenable to reduced-resolution evaluation.
- The framework is compatible with tiled and asynchronous GPU execution.
