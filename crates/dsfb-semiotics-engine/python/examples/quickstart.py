from pathlib import Path

import dsfb_engine
import pandas as pd


def main() -> None:
    root = Path(__file__).resolve().parents[2]
    observed = root / "tests" / "fixtures" / "observed_fixture.csv"
    predicted = root / "tests" / "fixtures" / "predicted_fixture.csv"

    frame = pd.read_csv(observed)
    trace = dsfb_engine.run_array(frame["ax"].tolist())
    csv_summary = dsfb_engine.run_csv(str(observed), str(predicted), scenario_id="python_fixture")

    print("array_trace_len=", len(trace))
    print("csv_summary=", csv_summary)


if __name__ == "__main__":
    main()
