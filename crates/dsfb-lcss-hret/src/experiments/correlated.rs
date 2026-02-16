use anyhow::Result;
use csv::Writer;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rand_distr::{Distribution, Normal};

use crate::{create_run_dir, Args};

pub(crate) fn run_correlated(args: &Args) -> Result<()> {
    let k_channels = 8;
    let group0 = [0usize, 1, 2, 3];
    let group1 = [4usize, 5, 6, 7];
    let groups = [&group0[..], &group1[..]];

    let rho = 0.95;
    let beta = 4.0;
    let beta_g = 4.0;

    let fault_amp = 2.0;
    let fault_start = 200usize;
    let fault_end = fault_start + 40;

    let mut rng = ChaCha8Rng::seed_from_u64(args.seed);
    let process_noise = Normal::new(0.0, 0.01)?;
    let meas_noise = Normal::new(0.0, 0.05)?;

    let run_dir = create_run_dir(&args.output)?;
    println!("  Output: {:?}", run_dir);

    let error_path = run_dir.join("group_error_comparison.csv");
    let mut error_wtr = Writer::from_path(&error_path)?;
    error_wtr.write_record(&["time", "error_channel_only", "error_hierarchical"])?;

    let weight_path = run_dir.join("group_weight_dynamics.csv");
    let mut weight_wtr = Writer::from_path(&weight_path)?;
    weight_wtr.write_record(&[
        "time",
        "mean_group0_weight_channel_only",
        "mean_group0_weight_hierarchical",
        "group_weight",
    ])?;

    let mut x_true = 0.0;
    let mut x_hat_channel = 0.0;
    let mut x_hat_hier = 0.0;

    let mut envelope_channel = vec![0.0f64; k_channels];
    let mut envelope_hier = vec![0.0f64; k_channels];
    let mut group_envelope = vec![0.0f64; groups.len()];

    for t in 0..args.time_steps {
        x_true += process_noise.sample(&mut rng);

        let mut measurements = vec![0.0f64; k_channels];
        for k in 0..k_channels {
            let noise = meas_noise.sample(&mut rng);
            let corrupted = t >= fault_start && t < fault_end && group0.contains(&k);
            let fault = if corrupted { fault_amp } else { 0.0 };
            measurements[k] = x_true + noise + fault;
        }

        let mut weights_channel = vec![0.0f64; k_channels];
        for k in 0..k_channels {
            let residual = measurements[k] - x_hat_channel;
            envelope_channel[k] = rho * envelope_channel[k] + (1.0 - rho) * residual.abs();
            weights_channel[k] = 1.0 / (1.0 + beta * envelope_channel[k]);
        }

        let mut sum_w = 0.0;
        let mut sum_wy = 0.0;
        for k in 0..k_channels {
            sum_w += weights_channel[k];
            sum_wy += weights_channel[k] * measurements[k];
        }
        if sum_w > 0.0 {
            x_hat_channel = sum_wy / sum_w;
        }

        let mut residuals_hier = vec![0.0f64; k_channels];
        for k in 0..k_channels {
            let residual = measurements[k] - x_hat_hier;
            residuals_hier[k] = residual.abs();
            envelope_hier[k] = rho * envelope_hier[k] + (1.0 - rho) * residuals_hier[k];
        }

        let mut group_weights = vec![0.0f64; groups.len()];
        for (g_idx, group) in groups.iter().enumerate() {
            let mut mean_abs = 0.0;
            for k in *group {
                mean_abs += residuals_hier[*k];
            }
            mean_abs /= group.len() as f64;
            group_envelope[g_idx] = rho * group_envelope[g_idx] + (1.0 - rho) * mean_abs;
            group_weights[g_idx] = 1.0 / (1.0 + beta_g * group_envelope[g_idx]);
        }

        let mut weights_hier = vec![0.0f64; k_channels];
        for (g_idx, group) in groups.iter().enumerate() {
            for k in *group {
                let channel_weight = 1.0 / (1.0 + beta * envelope_hier[*k]);
                weights_hier[*k] = channel_weight * group_weights[g_idx];
            }
        }

        let mut sum_w_h = 0.0;
        let mut sum_wy_h = 0.0;
        for k in 0..k_channels {
            sum_w_h += weights_hier[k];
            sum_wy_h += weights_hier[k] * measurements[k];
        }
        if sum_w_h > 0.0 {
            x_hat_hier = sum_wy_h / sum_w_h;
        }

        let error_channel = (x_hat_channel - x_true).abs();
        let error_hier = (x_hat_hier - x_true).abs();

        error_wtr.write_record(&[
            t.to_string(),
            format!("{:.6}", error_channel),
            format!("{:.6}", error_hier),
        ])?;

        let mut mean_group0_channel = 0.0;
        let mut mean_group0_hier = 0.0;
        for k in group0.iter() {
            mean_group0_channel += weights_channel[*k];
            mean_group0_hier += weights_hier[*k];
        }
        mean_group0_channel /= group0.len() as f64;
        mean_group0_hier /= group0.len() as f64;

        weight_wtr.write_record(&[
            t.to_string(),
            format!("{:.6}", mean_group0_channel),
            format!("{:.6}", mean_group0_hier),
            format!("{:.6}", group_weights[0]),
        ])?;
    }

    error_wtr.flush()?;
    weight_wtr.flush()?;

    println!("  Written: {:?}", error_path);
    println!("  Written: {:?}", weight_path);
    println!("  Correlated fault experiment complete!");

    Ok(())
}
