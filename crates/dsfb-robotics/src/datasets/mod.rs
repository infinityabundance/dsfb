//! Per-dataset residual adapters.
//!
//! Each adapter is a `no_std` + `no_alloc` pure function that
//! converts a dataset-specific raw-sample type into a scalar residual
//! norm `‖r(k)‖`. The scalar stream is then fed into the canonical
//! [`crate::engine::DsfbRoboticsEngine`] / [`crate::observe`] pipeline
//! unchanged.
//!
//! ## Dataset index (companion paper §10)
//!
//! | Module | Family | Residual form | Notes |
//! |---|---|---|---|
//! | `cwru` | PHM (bearing) | `\|E_{BPFI}(k) − μ_healthy\|` | Spectral-envelope deviation |
//! | `ims` | PHM (bearing) | `\|HI(k) − HI_nominal\|` | Health-index trajectory |
//! | `kuka_lwr` | Kinematics | `‖ddq − ddq_nominal‖` | Kinematic-residual variant, Simionato 7R |
//! | `femto_st` | PHM (bearing) | `\|vib-HI(k) − HI_calib\|` | PRONOSTIA |
//! | `panda_gaz` | Kinematics | `‖τ_meas − τ_pred(θ̂_panda)‖` | Literal Gaz-cpp model, Gaz 2019 |
//! | `dlr_justin` | Kinematics | `‖τ_meas − τ_interp‖` | Literal Giacomuzzo Zenodo τ_interp |
//! | `ur10_kufieta` | Kinematics | `‖τ_meas − τ_RNEA(URSim)‖` | Literal pinocchio RNEA, Polydoros 2015 |
//! | `cheetah3` | Balancing | `combine(r_F, r_ξ)` | Quadruped MPC + CoM |
//! | `icub_pushrecovery` | Balancing | `combine(r_W, r_ξ)` | Humanoid WBC + centroidal |

pub mod cwru;
pub mod ims;
pub mod kuka_lwr;
pub mod femto_st;
pub mod panda_gaz;
pub mod dlr_justin;
pub mod ur10_kufieta;
pub mod cheetah3;
pub mod icub_pushrecovery;
pub mod droid;
pub mod openx;
pub mod anymal_parkour;
pub mod unitree_g1;
pub mod aloha_static;
pub mod icub3_sorrentino;
pub mod mobile_aloha;
pub mod so100;
pub mod aloha_static_tape;
pub mod aloha_static_screw_driver;
pub mod aloha_static_pingpong_test;

/// Stable dataset identifier, used in `paper-lock` subcommand dispatch
/// and per-dataset audit artefacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DatasetId {
    /// Case Western Reserve University bearing dataset.
    Cwru,
    /// IMS run-to-failure bearing dataset (NASA PCoE).
    Ims,
    /// KUKA LWR-IV+ joint-space identification (Simionato 7R / Jubien 2014 lineage).
    KukaLwr,
    /// FEMTO-ST PRONOSTIA accelerated bearing degradation (IEEE PHM 2012).
    FemtoSt,
    /// Franka Emika Panda dynamic identification (Gaz et al. 2019).
    PandaGaz,
    /// DLR-class 7-DoF Panda measurement-vs-model torque corpus (Giacomuzzo et al. 2024, Zenodo 12516500).
    DlrJustin,
    /// Universal Robots UR10 pick-and-place torque identification (Polydoros et al. IROS 2015).
    Ur10Kufieta,
    /// MIT Mini-Cheetah locomotion open logs (Katz–Di Carlo–Kim 2019; UMich-CURLY dataset).
    Cheetah3,
    /// ergoCub humanoid push-recovery experiment (Romualdi–Viceconte 2024, ami-iit).
    IcubPushRecovery,
    /// DROID distributed robot manipulation dataset (Khazatsky et al. 2024, Stanford/TRI).
    Droid,
    /// Open X-Embodiment cross-robot manipulation corpus (RT-X 2024 collaboration).
    Openx,
    /// ANYmal-C parkour locomotion in the wild (Miki et al., Science Robotics 2022).
    AnymalParkour,
    /// Unitree G1 humanoid teleoperation (Makolon0321 / `unitree_g1_block_stack`).
    UnitreeG1,
    /// ALOHA bimanual static teleoperation (Zhao et al. 2023, LeRobot
    /// aloha_static_coffee corpus) — real physical ALOHA hardware.
    AlohaStatic,
    /// ergoCub Sorrentino balancing-torque-control (ami-iit RAL 2025).
    Icub3Sorrentino,
    /// Mobile ALOHA wipe-wine (Fu, Zhao, Finn 2024, Stanford).
    MobileAloha,
    /// SO-ARM100 pick-and-place (HuggingFace LeRobot, Apache-2.0).
    So100,
    /// ALOHA static tape attachment (LeRobot real bimanual).
    AlohaStaticTape,
    /// ALOHA static screw-driver tool-use (LeRobot real bimanual).
    AlohaStaticScrewDriver,
    /// ALOHA static ping-pong rhythmic transfer (LeRobot real bimanual).
    AlohaStaticPingpongTest,
}

impl DatasetId {
    /// Stable short identifier (kebab-style) for CLI + filesystem use.
    #[inline]
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Cwru => "cwru",
            Self::Ims => "ims",
            Self::KukaLwr => "kuka_lwr",
            Self::FemtoSt => "femto_st",
            Self::PandaGaz => "panda_gaz",
            Self::DlrJustin => "dlr_justin",
            Self::Ur10Kufieta => "ur10_kufieta",
            Self::Cheetah3 => "cheetah3",
            Self::IcubPushRecovery => "icub_pushrecovery",
            Self::Droid => "droid",
            Self::Openx => "openx",
            Self::AnymalParkour => "anymal_parkour",
            Self::UnitreeG1 => "unitree_g1",
            Self::AlohaStatic => "aloha_static",
            Self::Icub3Sorrentino => "icub3_sorrentino",
            Self::MobileAloha => "mobile_aloha",
            Self::So100 => "so100",
            Self::AlohaStaticTape => "aloha_static_tape",
            Self::AlohaStaticScrewDriver => "aloha_static_screw_driver",
            Self::AlohaStaticPingpongTest => "aloha_static_pingpong_test",
        }
    }

    /// Parse a slug back into a `DatasetId`. Returns `None` for unknowns.
    ///
    /// Safe-state policy: the `_ => None` arm is the explicit,
    /// documented fallback state. Unknown slugs are treated as "not a
    /// supported dataset" rather than coerced to a default or panicking.
    /// Callers surface the `None` to the user as an EX_USAGE CLI error
    /// (see the `paper-lock` CLI binary in `src/main.rs` and the
    /// integration tests in `tests/paper_lock_binary.rs`), which is the
    /// intended safe behaviour.
    #[must_use]
    pub fn from_slug(s: &str) -> Option<Self> {
        debug_assert!(s.len() < 64, "slug unreasonably long");
        match s {
            "cwru" => Some(Self::Cwru),
            "ims" => Some(Self::Ims),
            "kuka_lwr" => Some(Self::KukaLwr),
            "femto_st" => Some(Self::FemtoSt),
            "panda_gaz" => Some(Self::PandaGaz),
            "dlr_justin" => Some(Self::DlrJustin),
            "ur10_kufieta" => Some(Self::Ur10Kufieta),
            "cheetah3" => Some(Self::Cheetah3),
            "icub_pushrecovery" => Some(Self::IcubPushRecovery),
            "droid" => Some(Self::Droid),
            "openx" => Some(Self::Openx),
            "anymal_parkour" => Some(Self::AnymalParkour),
            "unitree_g1" => Some(Self::UnitreeG1),
            "aloha_static" => Some(Self::AlohaStatic),
            "icub3_sorrentino" => Some(Self::Icub3Sorrentino),
            "mobile_aloha" => Some(Self::MobileAloha),
            "so100" => Some(Self::So100),
            "aloha_static_tape" => Some(Self::AlohaStaticTape),
            "aloha_static_screw_driver" => Some(Self::AlohaStaticScrewDriver),
            "aloha_static_pingpong_test" => Some(Self::AlohaStaticPingpongTest),
            // SAFE-STATE: unknown slug is the explicitly-named fallback
            // state. Bind it to `unknown` so the arm is named (no
            // wildcard catch-all), assert the input shape, and return
            // None. Callers surface the None as EX_USAGE at the CLI
            // boundary (see `crate::main` and `tests/paper_lock_binary.rs`).
            unknown => {
                debug_assert!(
                    unknown.len() < 64,
                    "slug input bound by callsite preconditions"
                );
                None
            }
        }
    }

    /// Residual-family tag, for table-of-contents emission.
    #[inline]
    #[must_use]
    pub const fn family(self) -> DatasetFamily {
        match self {
            Self::Cwru | Self::Ims | Self::FemtoSt => DatasetFamily::Phm,
            Self::KukaLwr
            | Self::PandaGaz
            | Self::DlrJustin
            | Self::Ur10Kufieta
            | Self::Droid
            | Self::Openx
            | Self::AlohaStatic
            | Self::MobileAloha
            | Self::So100
            | Self::AlohaStaticTape
            | Self::AlohaStaticScrewDriver
            | Self::AlohaStaticPingpongTest => DatasetFamily::Kinematics,
            Self::Cheetah3
            | Self::IcubPushRecovery
            | Self::AnymalParkour
            | Self::UnitreeG1
            | Self::Icub3Sorrentino => DatasetFamily::Balancing,
        }
    }
}

/// Family classification matching the companion paper's §10 grouping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DatasetFamily {
    /// Prognostics / health monitoring (bearing degradation).
    Phm,
    /// Kinematic identification / manipulation residuals (arms, cobots).
    Kinematics,
    /// Balancing / whole-body control residuals (legged platforms).
    Balancing,
}

impl DatasetFamily {
    /// Stable label for table emission.
    #[inline]
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Phm => "PHM",
            Self::Kinematics => "Kinematics",
            Self::Balancing => "Balancing",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_roundtrips_for_every_dataset() {
        for id in [
            DatasetId::Cwru,
            DatasetId::Ims,
            DatasetId::KukaLwr,
            DatasetId::FemtoSt,
            DatasetId::PandaGaz,
            DatasetId::DlrJustin,
            DatasetId::Ur10Kufieta,
            DatasetId::Cheetah3,
            DatasetId::IcubPushRecovery,
            DatasetId::Droid,
            DatasetId::Openx,
            DatasetId::AnymalParkour,
            DatasetId::UnitreeG1,
            DatasetId::AlohaStatic,
        ] {
            let slug = id.slug();
            assert_eq!(DatasetId::from_slug(slug), Some(id), "roundtrip failed for {slug}");
        }
    }

    #[test]
    fn unknown_slug_is_none() {
        assert_eq!(DatasetId::from_slug("nope"), None);
        assert_eq!(DatasetId::from_slug(""), None);
    }

    #[test]
    fn kinematics_family_covers_four_arms() {
        let arms = [DatasetId::KukaLwr, DatasetId::PandaGaz, DatasetId::DlrJustin, DatasetId::Ur10Kufieta];
        for a in arms {
            assert_eq!(a.family(), DatasetFamily::Kinematics);
        }
    }

    #[test]
    fn balancing_family_covers_two_platforms() {
        assert_eq!(DatasetId::Cheetah3.family(), DatasetFamily::Balancing);
        assert_eq!(DatasetId::IcubPushRecovery.family(), DatasetFamily::Balancing);
    }

    #[test]
    fn phm_family_covers_three_bearing_datasets() {
        assert_eq!(DatasetId::Cwru.family(), DatasetFamily::Phm);
        assert_eq!(DatasetId::Ims.family(), DatasetFamily::Phm);
        assert_eq!(DatasetId::FemtoSt.family(), DatasetFamily::Phm);
    }
}
