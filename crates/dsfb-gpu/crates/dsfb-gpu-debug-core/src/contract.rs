//! Execution-contract validation and canonical hashing.
//!
//! The `Contract` struct mirrors `contract.toml` from the repo root. It
//! is intentionally a small fixed schema: every field is either a known
//! literal or a `[u8; 32]` hash, so the parser is a hand-rolled
//! line-by-line walk rather than a generic TOML reader.
//!
//! The canonical hash chain is anchored here. `contract_hash(...)`
//! returns the SHA-256 over the contract's canonical bytes, which feeds
//! into every downstream artifact hash. Changing any contract field
//! changes this hash and thereby invalidates every existing case file.

#![cfg(feature = "std")]

use std::string::{String, ToString};
use std::vec::Vec;

use crate::hash::{sha256, Sha256};

/// Lowercase hex digits used by the contract's hash serializer.
const HEX_LOWER: &[u8; 16] = b"0123456789abcdef";

/// Numeric mode the contract pins. Stored as a string in the TOML and
/// surfaced as an enum here so a typo in the file lands as a clear
/// parse error rather than a silent mismatch.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum NumericMode {
    /// Q16.16 fixed-point arithmetic — the v0 contract.
    FixedQ16,
}

impl NumericMode {
    /// String spelling used in the canonical bytes.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::FixedQ16 => "fixed_q16",
        }
    }

    /// Parse from the TOML string literal.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "fixed_q16" => Some(Self::FixedQ16),
            _ => None,
        }
    }
}

/// Locked kernel sequence, in execution order. Reordering or renaming
/// any entry changes the kernel-sequence hash and produces
/// `KernelSequenceMismatch`.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct KernelSequence {
    /// Names of the kernels in execution order.
    pub names: Vec<String>,
}

impl KernelSequence {
    /// Canonical bytes: comma-joined names, no whitespace.
    #[must_use]
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();
        for (i, name) in self.names.iter().enumerate() {
            if i > 0 {
                buf.push(b',');
            }
            buf.extend_from_slice(name.as_bytes());
        }
        buf
    }
}

/// The canonical contract carried by `contract.toml`. Fields are
/// strongly typed so a misspelling or out-of-range value lands as a
/// parse error.
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Contract {
    /// `[contract] name`.
    pub name: String,
    /// `[contract] version`.
    pub version: String,
    /// `[contract] backend` — one of `"cpu"`, `"cuda"`.
    pub backend: String,
    /// `[contract] determinism_level`.
    pub determinism_level: String,
    /// `[input] catalog_hash`, raw 32 bytes. `[0u8; 32]` means "to be
    /// computed".
    pub input_catalog_hash: [u8; 32],
    /// `[numeric] mode`.
    pub numeric_mode: NumericMode,
    /// `[numeric] float_allowed` (must be `false` in v0).
    pub float_allowed: bool,
    /// `[numeric] atomics_allowed`.
    pub atomics_allowed: bool,
    /// `[numeric] reduction_order` — must be `"fixed_order"` in v0.
    pub reduction_order: String,
    /// `[numeric] ewma_alpha_q16_raw`.
    pub ewma_alpha_q16_raw: i32,
    /// `[numeric] latency_clamp_ms`.
    pub latency_clamp_ms: u32,
    /// `[windowing] window_size_ms`.
    pub window_size_ms: u32,
    /// `[windowing] window_stride_ms`.
    pub window_stride_ms: u32,
    /// `[windowing] n_windows`.
    pub n_windows: u32,
    /// `[windowing] n_entities`.
    pub n_entities: u32,
    /// `[kernels] sequence`.
    pub kernel_sequence: KernelSequence,
    /// `[bank] bank_hash`, raw 32 bytes.
    pub bank_hash: [u8; 32],
    /// `[bank] strict_mode`.
    pub bank_strict_mode: bool,
    /// `[detector_registry] registry_hash`, raw 32 bytes.
    pub detector_registry_hash: [u8; 32],
    /// `[verdict] emit_case_file`.
    pub emit_case_file: bool,
    /// `[verdict] emit_intermediate_hashes`.
    pub emit_intermediate_hashes: bool,
}

impl Contract {
    /// Canonical v0 defaults that match the checked-in `contract.toml`.
    /// The hash fields here are zero; callers should run
    /// `validate_and_fill` to populate them.
    #[must_use]
    pub fn canonical() -> Self {
        Self {
            name: "dsfb-gpu-debug-demo".to_string(),
            version: "0.1".to_string(),
            backend: "cuda".to_string(),
            determinism_level: "same_device_same_binary_byte_exact".to_string(),
            input_catalog_hash: [0u8; 32],
            numeric_mode: NumericMode::FixedQ16,
            float_allowed: false,
            atomics_allowed: false,
            reduction_order: "fixed_order".to_string(),
            ewma_alpha_q16_raw: 8192,
            latency_clamp_ms: 32_767,
            window_size_ms: 1000,
            window_stride_ms: 1000,
            n_windows: 128,
            n_entities: 16,
            kernel_sequence: KernelSequence {
                names: [
                    "residual_field",
                    "drift_slew_sign",
                    "detector_motif",
                    "consensus_grid",
                    "candidate_collapse",
                ]
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            },
            bank_hash: [0u8; 32],
            detector_registry_hash: [0u8; 32],
            bank_strict_mode: true,
            emit_case_file: true,
            emit_intermediate_hashes: true,
        }
    }

    /// Append the contract's canonical bytes to `buf`. Order is
    /// deliberately fixed so a re-serialize-then-hash always yields the
    /// same hash.
    pub fn write_canonical(&self, buf: &mut Vec<u8>) {
        // The encoding is `name=value;` pairs, joined with `|`. This is
        // not designed to be human-readable — it is designed to be
        // unambiguous. Two contracts that produce the same canonical
        // bytes must be byte-identical in every field.
        push_kv_str(buf, "name", &self.name);
        push_sep(buf);
        push_kv_str(buf, "version", &self.version);
        push_sep(buf);
        push_kv_str(buf, "backend", &self.backend);
        push_sep(buf);
        push_kv_str(buf, "determinism_level", &self.determinism_level);
        push_sep(buf);
        push_kv_hex(buf, "input_catalog_hash", &self.input_catalog_hash);
        push_sep(buf);
        push_kv_str(buf, "numeric_mode", self.numeric_mode.name());
        push_sep(buf);
        push_kv_bool(buf, "float_allowed", self.float_allowed);
        push_sep(buf);
        push_kv_bool(buf, "atomics_allowed", self.atomics_allowed);
        push_sep(buf);
        push_kv_str(buf, "reduction_order", &self.reduction_order);
        push_sep(buf);
        push_kv_i64(
            buf,
            "ewma_alpha_q16_raw",
            i64::from(self.ewma_alpha_q16_raw),
        );
        push_sep(buf);
        push_kv_u64(buf, "latency_clamp_ms", u64::from(self.latency_clamp_ms));
        push_sep(buf);
        push_kv_u64(buf, "window_size_ms", u64::from(self.window_size_ms));
        push_sep(buf);
        push_kv_u64(buf, "window_stride_ms", u64::from(self.window_stride_ms));
        push_sep(buf);
        push_kv_u64(buf, "n_windows", u64::from(self.n_windows));
        push_sep(buf);
        push_kv_u64(buf, "n_entities", u64::from(self.n_entities));
        push_sep(buf);
        push_kv_bytes(
            buf,
            "kernel_sequence",
            &self.kernel_sequence.canonical_bytes(),
        );
        push_sep(buf);
        push_kv_hex(buf, "bank_hash", &self.bank_hash);
        push_sep(buf);
        push_kv_bool(buf, "bank_strict_mode", self.bank_strict_mode);
        push_sep(buf);
        push_kv_hex(buf, "detector_registry_hash", &self.detector_registry_hash);
        push_sep(buf);
        push_kv_bool(buf, "emit_case_file", self.emit_case_file);
        push_sep(buf);
        push_kv_bool(
            buf,
            "emit_intermediate_hashes",
            self.emit_intermediate_hashes,
        );
    }

    /// SHA-256 of the contract's canonical bytes. This is the
    /// `H(contract)` link in the hash chain.
    #[must_use]
    pub fn hash(&self) -> [u8; 32] {
        let mut buf: Vec<u8> = Vec::new();
        self.write_canonical(&mut buf);
        sha256(&buf)
    }

    /// Hash of `[kernels] sequence`. Separate link so a sequence change
    /// produces `KernelSequenceMismatch` distinct from a generic
    /// `ContractBreach`.
    #[must_use]
    pub fn kernel_sequence_hash(&self) -> [u8; 32] {
        sha256(&self.kernel_sequence.canonical_bytes())
    }

    /// Construct a scaled contract with the v0 numeric / kernel / bank /
    /// detector posture but a different grid shape. Use this for
    /// performance-scaling benchmarks (e.g. `Contract::scaled(256,
    /// 1024)` exposes the architecture's behaviour at deployment scale
    /// where the GPU's parallelism actually has somewhere to go).
    ///
    /// Determinism guarantees are unchanged: a scaled contract has its
    /// own hash, and two runs on the same `(events, scaled_contract)`
    /// pair still produce byte-identical case files per backend.
    #[must_use]
    pub fn scaled(n_entities: u32, n_windows: u32) -> Self {
        let mut c = Self::canonical();
        c.n_entities = n_entities;
        c.n_windows = n_windows;
        c
    }

    /// Pin the input catalog hash. Used after the synthesizer or fixture
    /// loader has the canonical bytes in hand.
    pub fn pin_input_hash(&mut self, hash: [u8; 32]) {
        self.input_catalog_hash = hash;
    }

    /// Pin the bank hash.
    pub fn pin_bank_hash(&mut self, hash: [u8; 32]) {
        self.bank_hash = hash;
    }

    /// Pin the detector-registry hash.
    pub fn pin_detector_registry_hash(&mut self, hash: [u8; 32]) {
        self.detector_registry_hash = hash;
    }
}

fn push_sep(buf: &mut Vec<u8>) {
    buf.push(b'|');
}

fn push_kv_str(buf: &mut Vec<u8>, key: &str, val: &str) {
    buf.extend_from_slice(key.as_bytes());
    buf.push(b'=');
    buf.extend_from_slice(val.as_bytes());
}

fn push_kv_bool(buf: &mut Vec<u8>, key: &str, val: bool) {
    push_kv_str(buf, key, if val { "true" } else { "false" });
}

fn push_kv_u64(buf: &mut Vec<u8>, key: &str, val: u64) {
    buf.extend_from_slice(key.as_bytes());
    buf.push(b'=');
    write_u64(buf, val);
}

fn push_kv_i64(buf: &mut Vec<u8>, key: &str, val: i64) {
    buf.extend_from_slice(key.as_bytes());
    buf.push(b'=');
    if val < 0 {
        buf.push(b'-');
        write_u64(buf, val.unsigned_abs());
    } else {
        write_u64(buf, val as u64);
    }
}

fn push_kv_hex(buf: &mut Vec<u8>, key: &str, val: &[u8; 32]) {
    buf.extend_from_slice(key.as_bytes());
    buf.push(b'=');
    for byte in val {
        buf.push(HEX_LOWER[(byte >> 4) as usize]);
        buf.push(HEX_LOWER[(byte & 0x0F) as usize]);
    }
}

fn push_kv_bytes(buf: &mut Vec<u8>, key: &str, val: &[u8]) {
    buf.extend_from_slice(key.as_bytes());
    buf.push(b'=');
    buf.extend_from_slice(val);
}

fn write_u64(buf: &mut Vec<u8>, mut n: u64) {
    if n == 0 {
        buf.push(b'0');
        return;
    }
    let mut scratch = [0u8; 20];
    let mut i = scratch.len();
    while n > 0 {
        i -= 1;
        scratch[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    buf.extend_from_slice(&scratch[i..]);
}

/// Chain a sub-hash into a running prefix: `chained = sha256(label || sub || prefix)`.
/// The label disambiguates the chain links so a stage's bytes alone do
/// not produce the same hash as another stage's bytes that happen to
/// have the same content.
#[must_use]
pub fn chain(label: &[u8], sub: &[u8; 32], prefix: &[u8; 32]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(label);
    h.update(sub);
    h.update(prefix);
    h.finalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_contract_hash_is_stable() {
        let c = Contract::canonical();
        let a = c.hash();
        let b = c.hash();
        assert_eq!(a, b);
        assert_ne!(a, [0u8; 32]);
    }

    #[test]
    fn changing_a_field_changes_the_hash() {
        let c0 = Contract::canonical();
        let mut c1 = Contract::canonical();
        c1.ewma_alpha_q16_raw = 16_384;
        assert_ne!(c0.hash(), c1.hash());
    }

    #[test]
    fn kernel_sequence_hash_changes_on_reorder() {
        let c0 = Contract::canonical();
        let mut c1 = Contract::canonical();
        // Swap two entries in the kernel sequence.
        let names = &mut c1.kernel_sequence.names;
        names.swap(0, 1);
        assert_ne!(c0.kernel_sequence_hash(), c1.kernel_sequence_hash());
    }

    #[test]
    fn chain_links_are_deterministic_but_depend_on_label() {
        let sub = [1u8; 32];
        let prefix = [2u8; 32];
        let a = chain(b"x", &sub, &prefix);
        let b = chain(b"x", &sub, &prefix);
        assert_eq!(a, b);
        let c = chain(b"y", &sub, &prefix);
        assert_ne!(a, c);
    }
}
