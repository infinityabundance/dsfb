# dsfb-hret

[<image-card alt="Crates.io" src="https://img.shields.io/crates/v/dsfb-hret.svg" ></image-card>](https://crates.io/crates/dsfb-hret)  <!-- Add after publish -->
[<image-card alt="Documentation" src="https://docs.rs/dsfb-hret/badge.svg" ></image-card>](https://docs.rs/dsfb-hret)  <!-- If using rustdoc -->
[<image-card alt="License" src="https://img.shields.io/badge/license-Apache--2.0-blue.svg" ></image-card>](https://github.com/infinityabundance/dsfb/blob/main/LICENSE)

Hierarchical Residual-Envelope Trust (HRET) extends the Drift–Slew Fusion Bootstrap (DSFB) framework for grouped multi-sensor fusion. It introduces group-level residual envelopes to handle correlated disturbances deterministically, with multiplicative trust composition and convex normalization. Reduces to DSFB when groups are singletons.

See the [full paper](https://zenodo.org/records/18783283) for details.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
dsfb-hret = "0.1.0"