# dsfb_fusion_figures notebook

Notebook file: `dsfb_fusion_figures.ipynb`

## Reproduce data first

From workspace root:

```bash
cargo run --release -p dsfb-fusion-bench -- --run-default
cargo run --release -p dsfb-fusion-bench -- --run-sweep
```

This produces:
- `output-dsfb-fusion-bench/<timestamp>/summary.csv`
- `output-dsfb-fusion-bench/<timestamp>/heatmap.csv`
- `output-dsfb-fusion-bench/<timestamp>/sim-dsfb-fusion-bench.csv` (also writes `trajectories.csv`)

## Run notebook

- Open `dsfb_fusion_figures.ipynb` in Google Colab or Jupyter.
- Keep `DATA_URLS` empty to load local `output-dsfb-fusion-bench/<timestamp>/*.csv`, or set URL strings.
- In Google Colab, click `Run all` first.
- If no local files are found, Colab upload prompts are used.
- When prompted, click `Browse` in the file picker and upload:
- `summary.csv`
- `heatmap.csv`
- `sim-dsfb-fusion-bench.csv` (or `trajectories.csv`)

## Notebook outputs

Saved in `./figures/`:
- `figure1_error_trajectories.png` and `.pdf`
- `figure2_trust_weights.png` and `.pdf`
- `figure3_peak_err_heatmap.png` and `.pdf`
- `figure4_false_downweight_heatmap.png` and `.pdf` (if available)
