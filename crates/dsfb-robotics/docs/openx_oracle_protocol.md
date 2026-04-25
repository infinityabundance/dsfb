# Open X-Embodiment — oracle protocol

Dataset: **Open X-Embodiment / RT-X 2024** (Open X-Embodiment Collaboration).

## Provenance

- **Reference:** Open X-Embodiment Collaboration, "Open X-Embodiment: Robotic Learning Datasets and RT-X Models," *RSS 2024*. [arXiv:2310.08864](https://arxiv.org/abs/2310.08864).
- **Public archive:** TFDS `open_x_embodiment`. This crate ingests the NYU-ROT subset.
- **Platform:** 22 robot embodiments aggregated; the NYU-ROT subset is on a real WidowX-class arm.
- **Sampling rate:** varies by embodiment.

## Licence and redistribution

- **Licence:** Apache-2.0 (TFDS metadata) plus per-source licences attributable upstream.
- **This crate ships:** preprocessed scalar residual stream at [`data/processed/openx.csv`](../data/processed/openx.csv) (98 timesteps, NYU-ROT slice).

## Fetch path

1. The preprocessor fetches the NYU-ROT TFRecord shard into `data/openx/`.
2. Residual = Euclidean norm of joint-state deviation from per-episode early-window nominal.
3. Run `paper-lock openx`.

## Residual construction

`r(k) = ‖observation.state(k) − nominal‖₂`.

## DSFB bounded claim

Surfaces cross-embodiment kinematic-residual variation as recurrent grammar structure. No claim about RT-X policy performance.

## Reproducibility

- `paper-lock openx`. Smoke-test: `paper-lock openx --fixture`.
