# gr-dsfb — GNU Radio Out-of-Tree Module

**Phase I deliverable**: plug the DSFB-RF structural-anomaly observer into
any existing GNU Radio / USRP flowgraph as a **read-only tap**.

The upstream flowgraph — USRP Source, Channel Filter, Demodulator, CFAR,
Spectrum Analyzer — is **not modified** in any way.  If this block is
disconnected or crashes, the upstream path continues identically to its
pre-DSFB state (paper §VIII-C, Limitation L11).

---

## Architecture

```
[USRP Source] ──► [Channel Filter]
                       ├──► [Demodulator / CFAR / Spectrum Analyzer]
                       └──► [dsfb_sink_b200]     ← read-only tap
                                  │
                              [ZeroMQ PUSH socket]
                                  │
                              [Operator console / SigMF recorder]
```

---

## Installation

### Prerequisites

| Dependency         | Minimum version | Install                              |
|--------------------|-----------------|--------------------------------------|
| GNU Radio          | 3.10            | `apt install gnuradio` / source build |
| Python             | 3.9             | system                               |
| Rust + Cargo       | 1.65            | `curl https://sh.rustup.rs | sh`    |
| CMake              | 3.16            | `apt install cmake`                  |

### Step 1 — Build the Rust shared library

```sh
cd <repo>/dsfb-rf
cargo build --release --features std
# → <repo>/dsfb-rf/target/release/libdsfb_rf.so
```

> On macOS replace `.so` with `.dylib`; on Windows `.dll`.

### Step 2 — Build and install gr-dsfb

```sh
cd <repo>/gr-dsfb
mkdir build && cd build
cmake .. -DDSFB_RF_LIB=$(realpath ../../dsfb-rf/target/release/libdsfb_rf.so)
make -j$(nproc)
sudo make install
sudo ldconfig
```

### Step 3 — Run one of the demo flowgraphs

```sh
python3 <repo>/gr-dsfb/examples/b200_live_tap.py
```

Or open `grc/gr_dsfb_sink_b200.grc` in GNU Radio Companion.

---

## Block Reference — `dsfb_sink_b200`

| Parameter       | Type    | Default      | Description                              |
|-----------------|---------|--------------|------------------------------------------|
| `platform_tag`  | string  | `"usrp_b200"`| Hardware descriptor for Episode metadata |
| `carrier_hz`    | float   | `915.0e6`    | Centre frequency (Hz); informational     |
| `sample_rate`   | float   | `1.0e6`      | Sample rate (Hz)                         |
| `adc_bits`      | int     | `12`         | ADC bit depth (B200 = 12, X310 = 14)    |
| `snr_floor_db`  | float   | `-10.0`      | SNR below which grammar is suppressed    |
| `zmq_endpoint`  | string  | `tcp://*:5560`| ZeroMQ PUSH socket for episodes         |
| `w_pred`        | int     | `5`          | Grammar window width (3, 5, or 7)        |

### Output

The block emits **no stream output**.  It is a pure sink.

Episodes are pushed as JSON on the ZeroMQ socket:

```json
{
  "core:sample_start": 1048576,
  "core:sample_count": 214,
  "core:label": "Boundary[SustainedOutwardDrift]",
  "dsfb:motif": "SustainedOutwardDrift",
  "dsfb:dsa_score": 0.72,
  "dsfb:lyapunov_lambda": 0.031,
  "dsfb:policy": "Review",
  "dsfb:platform_tag": "usrp_b200"
}
```

---

## Non-Intrusion Guarantee

This block:

- ✓ Accepts `CF32` samples as a **read-only** stream consumer  
- ✓ Has **no output port** connected to any upstream GNU Radio block  
- ✓ Does **not** write to USRP registers, AGC state, filter coefficients, or
  detector thresholds  
- ✓ Is **fail-safe**: disconnect or crash → upstream continues identically  

---

## Phase I Deliverable Scope (paper §XI.B)

| Deliverable                        | Status         |
|------------------------------------|----------------|
| Rust-side tap (`DsfbSinkB200`)     | ✅ Shipped (`dsfb-rf` crate) |
| Python GR block wrapper            | ✅ This module |
| GRC block YAML                     | ✅ `grc/gr_dsfb_sink_b200.block.yml` |
| A/B non-intrusion verification     | Demo script `examples/b200_ab_verify.py` |
| ZeroMQ episode JSON output         | ✅ Implemented |
| `gr-dsfb` OOT install + CMake      | ✅ This repo |

Phase II (out of scope for this repo): live hardware-in-the-loop recording
at operator site, VITA 49.2 VRLP framing, classified-network air-gap ports.

---

## License

Apache-2.0 — same as the `dsfb-rf` Rust crate.
