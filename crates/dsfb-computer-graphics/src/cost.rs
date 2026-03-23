use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum CostMode {
    Minimal,
    HostRealistic,
    FullResearchDebug,
}

impl CostMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Minimal => "minimal",
            Self::HostRealistic => "host_realistic",
            Self::FullResearchDebug => "full_research_debug",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Minimal => "Minimal mode",
            Self::HostRealistic => "Host-realistic mode",
            Self::FullResearchDebug => "Full research/debug mode",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct BufferCost {
    pub name: String,
    pub bytes_per_pixel: usize,
    pub notes: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct StageCost {
    pub stage: String,
    pub approximate_ops_per_pixel: usize,
    pub approximate_reads_per_pixel: usize,
    pub approximate_writes_per_pixel: usize,
    pub reduction_note: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ResolutionFootprint {
    pub width: usize,
    pub height: usize,
    pub total_pixels: usize,
    pub memory_megabytes: f32,
}

#[derive(Clone, Debug, Serialize)]
pub struct CostReport {
    pub mode: CostMode,
    pub buffers: Vec<BufferCost>,
    pub stages: Vec<StageCost>,
    pub footprints: Vec<ResolutionFootprint>,
    pub estimated_total_ops_per_pixel: usize,
    pub estimated_total_reads_per_pixel: usize,
    pub estimated_total_writes_per_pixel: usize,
    pub notes: Vec<String>,
}

pub fn build_cost_report(mode: CostMode) -> CostReport {
    let (buffers, stages, notes) = match mode {
        CostMode::Minimal => (
            vec![
                buffer("residual", 4, "single-channel scalar residual"),
                buffer("response", 4, "single-channel hazard / trigger"),
                buffer("alpha", 4, "single-channel alpha modulation"),
            ],
            vec![
                stage("Residual evaluation", 8, 2, 1, "Fuse with resolve when possible"),
                stage("Response update", 6, 2, 1, "May be reduced-resolution"),
                stage("Blend modulation", 6, 2, 1, "Can be fused into TAA resolve"),
            ],
            vec![
                "Minimal mode corresponds to fixed heuristic gating without full trust diagnostics."
                    .to_string(),
            ],
        ),
        CostMode::HostRealistic => (
            vec![
                buffer("residual", 4, "single-channel scalar residual"),
                buffer("depth disagreement", 4, "single-channel depth cue"),
                buffer("normal disagreement", 4, "single-channel normal cue"),
                buffer("motion disagreement", 4, "single-channel motion cue"),
                buffer("neighborhood inconsistency", 4, "single-channel neighborhood cue"),
                buffer("trust", 4, "single-channel supervisory trust"),
                buffer("alpha", 4, "single-channel alpha modulation"),
                buffer("intervention", 4, "single-channel hazard / response strength"),
            ],
            vec![
                stage("Residual evaluation", 10, 2, 1, "Local arithmetic only"),
                stage("Depth/normal disagreement", 12, 4, 2, "Can share reprojection fetches"),
                stage("Motion / neighborhood proxies", 18, 8, 2, "Tile aggregation is viable"),
                stage("Trust and alpha update", 14, 6, 3, "Trust may run at half resolution"),
                stage("Blend modulation", 6, 2, 1, "Fuse with temporal resolve"),
            ],
            vec![
                "Host-realistic mode excludes synthetic visibility hints and uses only signals plausible in an engine temporal pipeline."
                    .to_string(),
                "The DSFB supervisory layer can be implemented with local operations and limited temporal memory, with expected cost scaling linearly with pixel count and amenable to reduced-resolution evaluation."
                    .to_string(),
                "The framework is compatible with tiled and asynchronous GPU execution."
                    .to_string(),
            ],
        ),
        CostMode::FullResearchDebug => (
            vec![
                buffer("residual", 4, "single-channel scalar residual"),
                buffer("visibility hint", 4, "synthetic comparison/debug-only visibility cue"),
                buffer("depth disagreement", 4, "single-channel depth cue"),
                buffer("normal disagreement", 4, "single-channel normal cue"),
                buffer("motion disagreement", 4, "single-channel motion cue"),
                buffer("neighborhood inconsistency", 4, "single-channel neighborhood cue"),
                buffer("thin proxy", 4, "single-channel thin/local-contrast cue"),
                buffer("history instability", 4, "single-channel history instability cue"),
                buffer("trust", 4, "single-channel supervisory trust"),
                buffer("alpha", 4, "single-channel alpha modulation"),
                buffer("intervention", 4, "single-channel hazard / response strength"),
                buffer("state labels", 1, "debug structural-state labels"),
            ],
            vec![
                stage("Residual evaluation", 10, 2, 1, "Local arithmetic only"),
                stage("All proxy synthesis", 26, 12, 5, "Debug mode keeps all intermediate fields"),
                stage("Grammar / trust update", 18, 8, 3, "Tile aggregation remains possible"),
                stage("Alpha / intervention update", 8, 2, 2, "May be fused downstream"),
                stage("Debug output writes", 4, 0, 5, "Debug-only cost, not required in deployment"),
            ],
            vec![
                "Full research/debug mode keeps all intermediate cues and state exports for ablation and report generation."
                    .to_string(),
                "Synthetic visibility hints in this mode are intended only for comparison, not as a deployment claim."
                    .to_string(),
            ],
        ),
    };

    let estimated_total_ops_per_pixel = stages
        .iter()
        .map(|stage| stage.approximate_ops_per_pixel)
        .sum();
    let estimated_total_reads_per_pixel = stages
        .iter()
        .map(|stage| stage.approximate_reads_per_pixel)
        .sum();
    let estimated_total_writes_per_pixel = stages
        .iter()
        .map(|stage| stage.approximate_writes_per_pixel)
        .sum();
    let bytes_per_pixel = buffers
        .iter()
        .map(|buffer| buffer.bytes_per_pixel)
        .sum::<usize>();
    let footprints = [(1280usize, 720usize), (1920, 1080), (3840, 2160)]
        .into_iter()
        .map(|(width, height)| ResolutionFootprint {
            width,
            height,
            total_pixels: width * height,
            memory_megabytes: bytes_per_pixel as f32 * (width * height) as f32 / (1024.0 * 1024.0),
        })
        .collect();

    CostReport {
        mode,
        buffers,
        stages,
        footprints,
        estimated_total_ops_per_pixel,
        estimated_total_reads_per_pixel,
        estimated_total_writes_per_pixel,
        notes,
    }
}

fn buffer(name: &str, bytes_per_pixel: usize, notes: &str) -> BufferCost {
    BufferCost {
        name: name.to_string(),
        bytes_per_pixel,
        notes: notes.to_string(),
    }
}

fn stage(
    stage: &str,
    approximate_ops_per_pixel: usize,
    approximate_reads_per_pixel: usize,
    approximate_writes_per_pixel: usize,
    reduction_note: &str,
) -> StageCost {
    StageCost {
        stage: stage.to_string(),
        approximate_ops_per_pixel,
        approximate_reads_per_pixel,
        approximate_writes_per_pixel,
        reduction_note: reduction_note.to_string(),
    }
}
