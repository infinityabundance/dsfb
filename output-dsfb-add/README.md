# output-dsfb-add

This directory is intentionally kept free of committed run data.

`cargo run -p dsfb-add --bin dsfb_add_sweep` creates a fresh timestamped directory here:

```text
output-dsfb-add/<YYYY-MM-DDTHH-MM-SSZ>/
```

Expected runtime contents:

- `aet_sweep.csv`
- `tcp_sweep.csv`
- `rlt_sweep.csv`
- `iwlt_sweep.csv`
- `tcp_points/points_lambda_<idx>.csv`

Google Colab writes figure PNGs back into the same timestamped directory after loading the CSV outputs.
