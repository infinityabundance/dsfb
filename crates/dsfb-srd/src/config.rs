use std::fmt::Write;
use std::path::{Path, PathBuf};

pub const CRATE_NAME: &str = "dsfb-srd";
pub const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Debug)]
pub struct SimulationConfig {
    pub n_events: usize,
    pub n_channels: usize,
    pub causal_window: usize,
    pub tau_steps: usize,
    pub shock_start: usize,
    pub shock_end: usize,
    pub beta: f64,
    pub envelope_decay: f64,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            n_events: 2_000,
            n_channels: 4,
            causal_window: 24,
            tau_steps: 401,
            shock_start: 800,
            shock_end: 1_200,
            beta: 4.0,
            envelope_decay: 0.97,
        }
    }
}

impl SimulationConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.n_events < 2 {
            return Err("n_events must be at least 2".to_string());
        }
        if self.n_channels == 0 {
            return Err("n_channels must be at least 1".to_string());
        }
        if self.causal_window == 0 {
            return Err("causal_window must be at least 1".to_string());
        }
        if self.tau_steps < 2 {
            return Err("tau_steps must be at least 2".to_string());
        }
        if self.shock_start >= self.shock_end {
            return Err("shock_start must be less than shock_end".to_string());
        }
        if self.shock_end > self.n_events {
            return Err("shock_end must be less than or equal to n_events".to_string());
        }
        if !(self.beta.is_finite() && self.beta > 0.0) {
            return Err("beta must be a finite value greater than 0".to_string());
        }
        if !(self.envelope_decay.is_finite() && (0.0..=1.0).contains(&self.envelope_decay)) {
            return Err("envelope_decay must be finite and lie in [0, 1]".to_string());
        }
        Ok(())
    }

    pub fn from_args<I, S>(args: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut config = Self::default();
        let mut args = args.into_iter().map(Into::into).peekable();

        while let Some(arg) = args.next() {
            let (flag, inline_value) = match arg.split_once('=') {
                Some((flag, value)) => (flag.to_string(), Some(value.to_string())),
                None => (arg, None),
            };

            let value =
                |name: &str, inline_value: Option<String>, args: &mut std::iter::Peekable<_>| {
                    inline_value
                        .or_else(|| args.next())
                        .ok_or_else(|| format!("missing value for {name}"))
                };

            match flag.as_str() {
                "--n-events" => {
                    config.n_events =
                        parse_usize("--n-events", value("--n-events", inline_value, &mut args)?)?;
                }
                "--n-channels" => {
                    config.n_channels = parse_usize(
                        "--n-channels",
                        value("--n-channels", inline_value, &mut args)?,
                    )?;
                }
                "--causal-window" => {
                    config.causal_window = parse_usize(
                        "--causal-window",
                        value("--causal-window", inline_value, &mut args)?,
                    )?;
                }
                "--tau-steps" => {
                    config.tau_steps = parse_usize(
                        "--tau-steps",
                        value("--tau-steps", inline_value, &mut args)?,
                    )?;
                }
                "--shock-start" => {
                    config.shock_start = parse_usize(
                        "--shock-start",
                        value("--shock-start", inline_value, &mut args)?,
                    )?;
                }
                "--shock-end" => {
                    config.shock_end = parse_usize(
                        "--shock-end",
                        value("--shock-end", inline_value, &mut args)?,
                    )?;
                }
                "--beta" => {
                    config.beta = parse_f64("--beta", value("--beta", inline_value, &mut args)?)?;
                }
                "--envelope-decay" => {
                    config.envelope_decay = parse_f64(
                        "--envelope-decay",
                        value("--envelope-decay", inline_value, &mut args)?,
                    )?;
                }
                _ => {
                    return Err(format!(
                        "unrecognized argument `{flag}`\n\n{}",
                        Self::usage(
                            "cargo run --manifest-path crates/dsfb-srd/Cargo.toml --release --bin dsfb-srd-generate --"
                        )
                    ));
                }
            }
        }

        config.validate()?;
        Ok(config)
    }

    pub fn usage(program: &str) -> String {
        format!(
            "Usage:\n  {program} [options]\n\nOptions:\n  \
--n-events <usize>\n  \
--n-channels <usize>\n  \
--causal-window <usize>\n  \
--tau-steps <usize>\n  \
--shock-start <usize>\n  \
--shock-end <usize>\n  \
--beta <f64>\n  \
--envelope-decay <f64>\n  \
--help"
        )
    }

    pub fn tau_thresholds(&self) -> Vec<f64> {
        let denominator = (self.tau_steps - 1) as f64;
        (0..self.tau_steps)
            .map(|index| index as f64 / denominator)
            .collect()
    }

    pub fn scaled_for_n_events(&self, n_events: usize) -> Self {
        let mut scaled = self.clone();
        let scale = n_events as f64 / self.n_events as f64;
        scaled.n_events = n_events;
        scaled.shock_start = scale_index(self.shock_start, scale, n_events.saturating_sub(1));
        scaled.shock_end = scale_index(self.shock_end, scale, n_events);
        if scaled.shock_end <= scaled.shock_start {
            scaled.shock_end = (scaled.shock_start + 1).min(n_events);
        }
        scaled
    }

    pub fn canonical_json(&self) -> String {
        format!(
            concat!(
                "{{",
                "\"crate\":\"{}\",",
                "\"version\":\"{}\",",
                "\"n_events\":{},",
                "\"n_channels\":{},",
                "\"causal_window\":{},",
                "\"tau_steps\":{},",
                "\"shock_start\":{},",
                "\"shock_end\":{},",
                "\"beta\":{},",
                "\"envelope_decay\":{}",
                "}}"
            ),
            CRATE_NAME,
            CRATE_VERSION,
            self.n_events,
            self.n_channels,
            self.causal_window,
            self.tau_steps,
            self.shock_start,
            self.shock_end,
            canonical_float(self.beta),
            canonical_float(self.envelope_decay),
        )
    }

    pub fn config_hash(&self) -> String {
        sha256_hex(self.canonical_json().as_bytes())
    }

    pub fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .map(Path::to_path_buf)
            .expect("dsfb-srd must live under <repo>/crates/dsfb-srd")
    }

    pub fn output_root() -> PathBuf {
        Self::repo_root().join("output-dsfb-srd")
    }
}

pub fn compute_run_id(config: &SimulationConfig) -> String {
    let config_hash = config.config_hash();
    config_hash.chars().take(32).collect()
}

fn parse_usize(name: &str, raw: String) -> Result<usize, String> {
    raw.parse::<usize>()
        .map_err(|_| format!("invalid integer for {name}: `{raw}`"))
}

fn parse_f64(name: &str, raw: String) -> Result<f64, String> {
    raw.parse::<f64>()
        .map_err(|_| format!("invalid floating-point value for {name}: `{raw}`"))
}

fn scale_index(index: usize, scale: f64, upper_bound: usize) -> usize {
    ((index as f64 * scale).round() as usize).min(upper_bound)
}

fn canonical_float(value: f64) -> String {
    format!("{value:.12}")
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = sha256(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        let _ = write!(&mut output, "{byte:02x}");
    }
    output
}

fn sha256(bytes: &[u8]) -> [u8; 32] {
    const INITIAL_STATE: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];

    const ROUND_CONSTANTS: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let bit_len = (bytes.len() as u64) * 8;
    let mut padded = bytes.to_vec();
    padded.push(0x80);
    while (padded.len() + 8) % 64 != 0 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    let mut state = INITIAL_STATE;

    for chunk in padded.chunks_exact(64) {
        let mut words = [0u32; 64];
        for (index, slot) in words.iter_mut().take(16).enumerate() {
            let start = index * 4;
            *slot = u32::from_be_bytes([
                chunk[start],
                chunk[start + 1],
                chunk[start + 2],
                chunk[start + 3],
            ]);
        }
        for index in 16..64 {
            let s0 = words[index - 15].rotate_right(7)
                ^ words[index - 15].rotate_right(18)
                ^ (words[index - 15] >> 3);
            let s1 = words[index - 2].rotate_right(17)
                ^ words[index - 2].rotate_right(19)
                ^ (words[index - 2] >> 10);
            words[index] = words[index - 16]
                .wrapping_add(s0)
                .wrapping_add(words[index - 7])
                .wrapping_add(s1);
        }

        let mut a = state[0];
        let mut b = state[1];
        let mut c = state[2];
        let mut d = state[3];
        let mut e = state[4];
        let mut f = state[5];
        let mut g = state[6];
        let mut h = state[7];

        for index in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(ROUND_CONSTANTS[index])
                .wrapping_add(words[index]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
        state[5] = state[5].wrapping_add(f);
        state[6] = state[6].wrapping_add(g);
        state[7] = state[7].wrapping_add(h);
    }

    let mut digest = [0u8; 32];
    for (index, word) in state.iter().enumerate() {
        let bytes = word.to_be_bytes();
        let start = index * 4;
        digest[start..start + 4].copy_from_slice(&bytes);
    }
    digest
}

#[cfg(test)]
mod tests {
    use super::{compute_run_id, sha256_hex, SimulationConfig};

    #[test]
    fn sha256_matches_known_digest() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn run_id_is_deterministic() {
        let config = SimulationConfig::default();
        assert_eq!(compute_run_id(&config), compute_run_id(&config));
    }
}
