#!/usr/bin/env python3
"""Generate the chemometric detector-atlas corpus (TOML) deterministically.

Each entry mirrors the dsfb-gpu-atlas-corpus schema. `implementation_status` honestly marks
whether a detector is EXECUTED by the edge crate's demo pipeline or CATALOGUED as prior-art
surface (defined, with mathematical form + provenance, but not run in the default demo).

Run:  python3 scripts/gen_atlas_corpus.py  ->  corpus/chemometric_atlas.toml

Output: corpus/chemometric_atlas.toml
  57 [[detector]] blocks across 11 primitive families (4 structural families × variable counts):
    A. ClassicalMspc        (14 detectors)
    B. DynamicTemporal      (13 detectors)
    C. NonlinearDistributional (19 detectors)
    D. ProcessStructure     (11 detectors)

DETS tuple layout (positional, 12 fields):
  0  detector_id             short snake_case identifier
  1  display_name            human-readable name
  2  family                  structural family (A/B/C/D above)
  3  primitive_family        the gen_figures.py atlas-chart grouping key
  4  mathematical_form       core mathematical object
  5  decision_functional     OneSided | TwoSided
  6  witness_role            Primary | Supporting
  7  fusion_axes             list of AXIS_* tags
  8  deterministic_status    DeterministicNative | DeterministicNumeric
  9  implementation_status   Executed | Catalogued
  10 origin_domains          list of domain tags
  11 source_basis            literature/prior-art reference

Honesty note: all 57 detectors are prior-art chemometric / statistical methods.
The DSFB grammar/fusion layer that sequences and fuses their outputs is the novel
contribution of this work; no individual detector is claimed as novel here.
"""
import os

# (id, display_name, family, primitive_family, math_form, decision_functional, witness_role,
#  fusion_axes, deterministic_status, impl_status, origin_domains, basis)
E = "Executed"    # run by the edge crate demo pipeline
C = "Catalogued"  # defined prior-art surface; not run in the default demo
D = "DeterministicNative"   # deterministic with only standard arithmetic (no float nondeterminism)
DN = "DeterministicNumeric" # deterministic but uses floating-point routines that may vary across platforms

DETS = [
    # ── A. Classical MSPC ───────────────────────────────────────────────────────
    ("shewhart_max_z", "Shewhart multivariate max |z|", "ClassicalMspc", "ScalarThreshold",
     "StandardisedDeviation", "TwoSided", "Primary", ["AXIS_RESIDUAL_MAGNITUDE"], D, E,
     ["INDUSTRIAL", "TABULAR"], "Shewhart control chart; 3-sigma rule"),
    ("robust_z_mad", "Robust z-score (median/MAD)", "ClassicalMspc", "ScalarThreshold",
     "RobustStandardisedDeviation", "TwoSided", "Primary", ["AXIS_RESIDUAL_MAGNITUDE"], D, E,
     ["INDUSTRIAL", "TABULAR"], "Hampel filter; robust location/scale"),
    ("pca_t2", "PCA Hotelling T-squared (score space)", "ClassicalMspc", "ProjectionResidual",
     "MahalanobisInScoreSpace", "OneSided", "Primary", ["AXIS_LATENT_MAGNITUDE"], D, E,
     ["CHEMOMETRIC", "INDUSTRIAL"], "MSPC T2 monitoring index"),
    ("pca_spe_q", "PCA SPE / Q residual", "ClassicalMspc", "ProjectionResidual",
     "ReconstructionResidualNorm", "OneSided", "Primary", ["AXIS_RESIDUAL_ENERGY"], D, E,
     ["CHEMOMETRIC", "INDUSTRIAL"], "Squared prediction error / Q statistic"),
    ("hotelling_t2_full", "Hotelling T-squared (full covariance)", "ClassicalMspc", "MultivariateHypothesis",
     "MahalanobisDistance", "OneSided", "Primary", ["AXIS_LATENT_MAGNITUDE"], D, C,
     ["TABULAR", "INDUSTRIAL"], "Hotelling 1947 multivariate control"),
    ("pls_score_residual", "PLS score residual", "ClassicalMspc", "ProjectionResidual",
     "LatentScoreResidual", "OneSided", "Primary", ["AXIS_LATENT_MAGNITUDE"], D, C,
     ["CHEMOMETRIC"], "Partial least squares latent monitoring"),
    ("pls_prediction_residual", "PLS prediction residual (Q on PLS)", "ClassicalMspc", "ProjectionResidual",
     "PredictionResidualNorm", "OneSided", "Primary", ["AXIS_RESIDUAL_ENERGY"], D, C,
     ["CHEMOMETRIC"], "PLS regression residual monitoring"),
    ("simca_distance", "SIMCA class distance", "ClassicalMspc", "ProjectionResidual",
     "ClassModelDistance", "OneSided", "Primary", ["AXIS_LATENT_MAGNITUDE"], D, C,
     ["CHEMOMETRIC"], "Soft independent modelling of class analogy"),
    ("contribution_plot", "Contribution plot (variable attribution)", "ClassicalMspc", "ProjectionResidual",
     "ResidualContribution", "OneSided", "Supporting", ["AXIS_ATTRIBUTION"], D, C,
     ["CHEMOMETRIC", "INDUSTRIAL"], "MSPC contribution analysis"),
    ("western_electric_rules", "Western Electric SPC rules", "ClassicalMspc", "SequentialRecurrence",
     "RunLengthPattern", "TwoSided", "Supporting", ["AXIS_RUN_PATTERN"], D, C,
     ["INDUSTRIAL"], "Western Electric handbook zone rules"),
    ("nelson_rules", "Nelson SPC rules", "ClassicalMspc", "SequentialRecurrence",
     "RunLengthPattern", "TwoSided", "Supporting", ["AXIS_RUN_PATTERN"], D, C,
     ["INDUSTRIAL"], "Nelson 1984 control-chart rules"),
    ("tukey_fences", "Tukey fences (IQR outlier)", "ClassicalMspc", "ScalarThreshold",
     "InterquartileFence", "TwoSided", "Supporting", ["AXIS_RESIDUAL_MAGNITUDE"], D, C,
     ["TABULAR"], "Tukey exploratory data analysis"),
    ("multiblock_pca", "Multiblock PCA", "ClassicalMspc", "ProjectionResidual",
     "BlockReconstructionResidual", "OneSided", "Primary", ["AXIS_RESIDUAL_ENERGY"], D, C,
     ["CHEMOMETRIC"], "Multiblock latent monitoring"),
    ("multiscale_pca", "Multiscale PCA", "ClassicalMspc", "Wavelet",
     "WaveletScaleReconstruction", "OneSided", "Primary", ["AXIS_SPECTRAL"], D, C,
     ["CHEMOMETRIC"], "Bakshi multiscale PCA"),
    # ── B. Dynamic / Temporal ───────────────────────────────────────────────────
    ("ewma_spe", "EWMA control chart (on SPE)", "DynamicTemporal", "WindowStatistic",
     "ExponentiallyWeightedMean", "TwoSided", "Primary", ["AXIS_TEMPORAL_DRIFT"], D, E,
     ["INDUSTRIAL", "TIME_SERIES"], "Roberts 1959 EWMA"),
    ("cusum_spe", "CUSUM chart (on SPE)", "DynamicTemporal", "SequentialRecurrence",
     "CumulativeSum", "TwoSided", "Primary", ["AXIS_TEMPORAL_DRIFT"], D, E,
     ["INDUSTRIAL", "TIME_SERIES"], "Page 1954 cumulative sum"),
    ("page_hinkley_spe", "Page-Hinkley test (on SPE)", "DynamicTemporal", "SequentialRecurrence",
     "CumulativeDeviation", "OneSided", "Primary", ["AXIS_CHANGEPOINT"], D, E,
     ["TIME_SERIES"], "Page-Hinkley change detection"),
    ("mann_kendall_spe", "Mann-Kendall trend test (windowed)", "DynamicTemporal", "RankStatistic",
     "SignConcordanceSum", "TwoSided", "Primary", ["AXIS_TEMPORAL_DRIFT"], D, E,
     ["TIME_SERIES", "ENVIRONMENTAL"], "Mann-Kendall nonparametric trend"),
    ("mewma", "MEWMA (multivariate EWMA)", "DynamicTemporal", "WindowStatistic",
     "MultivariateEwma", "OneSided", "Primary", ["AXIS_TEMPORAL_DRIFT"], D, C,
     ["INDUSTRIAL"], "Lowry 1992 MEWMA"),
    ("dpca", "Dynamic PCA (lag-augmented)", "DynamicTemporal", "ProjectionResidual",
     "LagAugmentedReconstruction", "OneSided", "Primary", ["AXIS_RESIDUAL_ENERGY"], D, C,
     ["CHEMOMETRIC", "INDUSTRIAL"], "Ku 1995 dynamic PCA"),
    ("moving_window_pca", "Moving-window PCA", "DynamicTemporal", "ProjectionResidual",
     "AdaptiveReconstruction", "OneSided", "Primary", ["AXIS_RESIDUAL_ENERGY"], D, C,
     ["CHEMOMETRIC"], "Adaptive MSPC"),
    ("recursive_pca", "Recursive PCA", "DynamicTemporal", "ProjectionResidual",
     "RecursiveReconstruction", "OneSided", "Primary", ["AXIS_RESIDUAL_ENERGY"], D, C,
     ["CHEMOMETRIC"], "Li 2000 recursive PCA"),
    ("lagged_autocorr_break", "Autocorrelation-coefficient break", "DynamicTemporal", "WindowStatistic",
     "AutocorrelationShift", "TwoSided", "Supporting", ["AXIS_TEMPORAL_STRUCTURE"], D, C,
     ["TIME_SERIES"], "Early-warning critical slowing down"),
    ("pettitt_changepoint", "Pettitt change-point test", "DynamicTemporal", "RankStatistic",
     "MannWhitneyChangepoint", "TwoSided", "Primary", ["AXIS_CHANGEPOINT"], D, C,
     ["ENVIRONMENTAL", "TIME_SERIES"], "Pettitt 1979"),
    ("snht", "Standard Normal Homogeneity Test", "DynamicTemporal", "SequentialRecurrence",
     "HomogeneityStatistic", "TwoSided", "Primary", ["AXIS_CHANGEPOINT"], D, C,
     ["ENVIRONMENTAL"], "Alexandersson 1986"),
    ("mosum", "MOSUM (moving sum of residuals)", "DynamicTemporal", "WindowStatistic",
     "MovingSum", "TwoSided", "Primary", ["AXIS_CHANGEPOINT"], D, C,
     ["TIME_SERIES"], "Moving-sum change detection"),
    ("buishand_range", "Buishand range test", "DynamicTemporal", "SequentialRecurrence",
     "RescaledRange", "TwoSided", "Primary", ["AXIS_CHANGEPOINT"], D, C,
     ["ENVIRONMENTAL"], "Buishand 1982"),
    # ── C. Nonlinear / Distributional ───────────────────────────────────────────
    ("psi_spe", "Population Stability Index (on SPE)", "NonlinearDistributional", "DistributionDistance",
     "PopulationStabilityIndex", "OneSided", "Primary", ["AXIS_DISTRIBUTION_SHIFT"], D, E,
     ["TABULAR", "TIME_SERIES"], "PSI population drift"),
    ("ks_spe", "Kolmogorov-Smirnov two-sample (on SPE)", "NonlinearDistributional", "DistributionDistance",
     "EmpiricalCdfSupremum", "OneSided", "Primary", ["AXIS_DISTRIBUTION_SHIFT"], D, E,
     ["TABULAR", "TIME_SERIES"], "KS two-sample test"),
    ("knn_spe", "kNN distance (on SPE cloud)", "NonlinearDistributional", "DistributionDistance",
     "KNearestNeighbourDistance", "OneSided", "Primary", ["AXIS_DENSITY"], D, E,
     ["TABULAR"], "kNN-based anomaly distance"),
    ("spectral_entropy_spe", "Spectral entropy shift (on SPE)", "NonlinearDistributional", "Spectral",
     "NormalisedSpectralEntropy", "TwoSided", "Primary", ["AXIS_SPECTRAL"], D, E,
     ["TIME_SERIES"], "Power-spectrum Shannon entropy"),
    ("kpca_reconstruction", "Kernel PCA reconstruction error", "NonlinearDistributional", "ProjectionResidual",
     "KernelReconstructionResidual", "OneSided", "Primary", ["AXIS_RESIDUAL_ENERGY"], DN, C,
     ["CHEMOMETRIC"], "Lee 2004 KPCA monitoring"),
    ("ica_residual", "ICA residual statistic", "NonlinearDistributional", "ProjectionResidual",
     "IndependentComponentResidual", "OneSided", "Primary", ["AXIS_RESIDUAL_ENERGY"], DN, C,
     ["CHEMOMETRIC"], "Kano 2004 ICA monitoring"),
    ("one_class_svm_distance", "One-class SVM residual distance", "NonlinearDistributional", "DistributionDistance",
     "DecisionFunctionMargin", "OneSided", "Primary", ["AXIS_DENSITY"], DN, C,
     ["TABULAR"], "Scholkopf one-class SVM"),
    ("lof_residual", "Local outlier factor residual", "NonlinearDistributional", "DistributionDistance",
     "LocalDensityRatio", "OneSided", "Primary", ["AXIS_DENSITY"], DN, C,
     ["TABULAR"], "Breunig 2000 LOF"),
    ("autoencoder_reconstruction", "Autoencoder reconstruction residual", "NonlinearDistributional", "ProjectionResidual",
     "NonlinearReconstructionResidual", "OneSided", "Primary", ["AXIS_RESIDUAL_ENERGY"], DN, C,
     ["CHEMOMETRIC", "TIME_SERIES"], "Reconstruction-based deep anomaly detection"),
    ("kl_divergence", "Kullback-Leibler divergence", "NonlinearDistributional", "DistributionDistance",
     "KullbackLeibler", "OneSided", "Primary", ["AXIS_DISTRIBUTION_SHIFT"], D, C,
     ["TABULAR"], "KL divergence drift"),
    ("js_divergence", "Jensen-Shannon divergence", "NonlinearDistributional", "DistributionDistance",
     "JensenShannon", "OneSided", "Primary", ["AXIS_DISTRIBUTION_SHIFT"], D, C,
     ["TABULAR"], "Symmetrised KL"),
    ("mmd", "Maximum Mean Discrepancy", "NonlinearDistributional", "DistributionDistance",
     "KernelMeanEmbeddingDistance", "OneSided", "Primary", ["AXIS_DISTRIBUTION_SHIFT"], DN, C,
     ["TABULAR"], "Gretton 2012 MMD"),
    ("wasserstein", "Wasserstein / earth-mover distance", "NonlinearDistributional", "DistributionDistance",
     "OptimalTransportDistance", "OneSided", "Primary", ["AXIS_DISTRIBUTION_SHIFT"], D, C,
     ["TABULAR"], "1D optimal transport"),
    ("energy_distance", "Energy distance", "NonlinearDistributional", "DistributionDistance",
     "EnergyStatistic", "OneSided", "Primary", ["AXIS_DISTRIBUTION_SHIFT"], D, C,
     ["TABULAR"], "Szekely energy distance"),
    ("hellinger", "Hellinger distance", "NonlinearDistributional", "DistributionDistance",
     "BhattacharyyaHellinger", "OneSided", "Primary", ["AXIS_DISTRIBUTION_SHIFT"], D, C,
     ["TABULAR"], "Hellinger distributional distance"),
    ("total_variation", "Total variation distance", "NonlinearDistributional", "DistributionDistance",
     "TotalVariation", "OneSided", "Supporting", ["AXIS_DISTRIBUTION_SHIFT"], D, C,
     ["TABULAR"], "TV distance"),
    ("anderson_darling", "Anderson-Darling test", "NonlinearDistributional", "DistributionDistance",
     "WeightedEcdfStatistic", "OneSided", "Supporting", ["AXIS_DISTRIBUTION_SHIFT"], D, C,
     ["TABULAR"], "Anderson-Darling GOF"),
    ("cramer_von_mises", "Cramer-von Mises test", "NonlinearDistributional", "DistributionDistance",
     "IntegratedEcdfStatistic", "OneSided", "Supporting", ["AXIS_DISTRIBUTION_SHIFT"], D, C,
     ["TABULAR"], "Cramer-von Mises GOF"),
    ("wavelet_energy", "Wavelet coefficient energy", "NonlinearDistributional", "Wavelet",
     "WaveletDetailEnergy", "OneSided", "Primary", ["AXIS_SPECTRAL"], D, C,
     ["TIME_SERIES"], "Wavelet multiresolution energy"),
    # ── D. Process-structure ────────────────────────────────────────────────────
    ("co_drift", "Variable-group co-drift", "ProcessStructure", "ResidualObserver",
     "SignedCoMonotoneCount", "TwoSided", "Primary", ["AXIS_STRUCTURE_COHERENCE"], D, E,
     ["INDUSTRIAL", "CHEMOMETRIC"], "Coherent multi-variable residual drift"),
    ("sensor_bias", "Sensor-bias isolation", "ProcessStructure", "ResidualObserver",
     "IsolationMargin", "OneSided", "Primary", ["AXIS_ATTRIBUTION"], D, E,
     ["INDUSTRIAL"], "Single-variable contribution isolation"),
    ("unit_operation_block", "Unit-operation block detector", "ProcessStructure", "ResidualObserver",
     "BlockResidualEnergy", "OneSided", "Primary", ["AXIS_STRUCTURE_COHERENCE"], D, C,
     ["INDUSTRIAL"], "Unit-wise block monitoring"),
    ("mass_balance_residual", "Mass-balance residual", "ProcessStructure", "ResidualObserver",
     "ConservationResidual", "TwoSided", "Primary", ["AXIS_BALANCE"], D, C,
     ["CHEMICAL"], "Mass conservation closure residual"),
    ("energy_balance_residual", "Energy-balance residual", "ProcessStructure", "ResidualObserver",
     "ConservationResidual", "TwoSided", "Primary", ["AXIS_BALANCE"], D, C,
     ["CHEMICAL"], "Energy conservation closure residual"),
    ("control_action_mismatch", "Control-action/residual mismatch", "ProcessStructure", "ResidualObserver",
     "ManipulatedControlledMismatch", "TwoSided", "Primary", ["AXIS_CONTROL"], D, C,
     ["CONTROL"], "Closed-loop residual mismatch"),
    ("actuator_lag", "Actuator-lag detector", "ProcessStructure", "SequentialRecurrence",
     "DelayedResponseResidual", "OneSided", "Primary", ["AXIS_CONTROL"], D, C,
     ["CONTROL"], "Actuator response-lag signature"),
    ("valve_hunting", "Valve hunting (control-loop oscillation)", "ProcessStructure", "Spectral",
     "LimitCycleSpectralPeak", "OneSided", "Primary", ["AXIS_SPECTRAL"], D, C,
     ["CONTROL"], "Control-loop oscillation diagnosis"),
    ("batch_phase_residual", "Batch-phase residual", "ProcessStructure", "ResidualObserver",
     "PhaseAlignedResidual", "OneSided", "Primary", ["AXIS_PHASE"], D, C,
     ["BATCH", "CHEMOMETRIC"], "Multiway/phase-aligned PCA residual"),
    ("missingness_spike", "Missingness spike", "ProcessStructure", "Missingness",
     "MissingFractionShift", "OneSided", "Supporting", ["AXIS_MISSINGNESS"], D, C,
     ["INDUSTRIAL"], "Missing-data rate anomaly"),
    ("missingness_coupling", "Missingness coupling", "ProcessStructure", "Missingness",
     "JointMissingnessShift", "OneSided", "Supporting", ["AXIS_MISSINGNESS"], D, C,
     ["INDUSTRIAL"], "Correlated missingness anomaly"),
]


def esc(s: str) -> str:
    """Escape backslashes and double-quotes for safe embedding in TOML double-quoted strings."""
    return s.replace("\\", "\\\\").replace('"', '\\"')


def arr(xs):
    """Render a Python list as a TOML inline string array: ["a", "b", ...]."""
    return "[" + ", ".join('"%s"' % esc(x) for x in xs) + "]"


def main():
    """Write corpus/chemometric_atlas.toml from the DETS list.

    The output file begins with a schema header (schema_version, corpus_name,
    detector_count) followed by one [[detector]] block per entry in DETS.
    canonical_id is 1-based and follows the order of the DETS list.

    The file header does NOT contain a `primitive_family` key; this is the property
    that gen_figures.py relies on to skip the header when counting detectors per family.

    All string values are escaped by esc() before insertion.
    Arrays (fusion_axes, origin_domains) are written as TOML inline arrays by arr().

    Writes: corpus/chemometric_atlas.toml
    """
    here = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    out_dir = os.path.join(here, "corpus")
    os.makedirs(out_dir, exist_ok=True)
    out = os.path.join(out_dir, "chemometric_atlas.toml")
    lines = []
    lines.append("# DSFB-Chemical-Engineering — chemometric detector-atlas corpus.")
    lines.append("# Generated by scripts/gen_atlas_corpus.py (deterministic). Do not edit by hand.")
    lines.append("# implementation_status: Executed = run by the demo pipeline; Catalogued = defined")
    lines.append("# prior-art surface (mathematical form + provenance) not run in the default demo.")
    lines.append("")
    # File header block: no `primitive_family` field, so gen_figures.py skips it correctly.
    lines.append("schema_version = 1")
    lines.append('corpus_name = "dsfb-chemical-engineering chemometric detector atlas"')
    lines.append(f"detector_count = {len(DETS)}")
    lines.append("")
    for i, d in enumerate(DETS, start=1):
        (did, name, fam, prim, mform, dfun, wrole, axes, dstat, istat, origins, basis) = d
        lines.append("[[detector]]")
        lines.append(f"canonical_id = {i}")
        lines.append(f'detector_id = "{esc(did)}"')
        lines.append(f'display_name = "{esc(name)}"')
        lines.append(f'family = "{fam}"')
        lines.append(f'primitive_family = "{prim}"')
        lines.append(f'mathematical_form = "{esc(mform)}"')
        lines.append(f'decision_functional = "{dfun}"')
        lines.append(f'witness_role = "{wrole}"')
        lines.append(f"fusion_axes = {arr(axes)}")
        lines.append(f"origin_domains = {arr(origins)}")
        lines.append(f'deterministic_status = "{dstat}"')
        lines.append(f'implementation_status = "{istat}"')
        lines.append(f'source_basis = "{esc(basis)}"')
        lines.append("")
    with open(out, "w") as f:
        f.write("\n".join(lines))
    n_exec = sum(1 for d in DETS if d[9] == E)
    print(f"wrote {out}: {len(DETS)} detectors ({n_exec} executed, {len(DETS)-n_exec} catalogued)")


if __name__ == "__main__":
    main()
