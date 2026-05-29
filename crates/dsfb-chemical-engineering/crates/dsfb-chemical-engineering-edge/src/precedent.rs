//! Structural-similarity / case-law objects (P65) — all **advisory similarity, never identity**.
//!
//! Five hash-sealed objects that let episodes be compared structurally without ever claiming two
//! episodes are *the same fault*:
//! - [`EpisodeShapeHashV1`] — a structural fingerprint of an episode (its motif sequence + a coarse
//!   feature vector) with an exact `shape_hash` and a `distance` for similarity.
//! - [`MotifNearestNeighborV1`] — nearest catalogued motifs to a query shape (top-k by distance).
//! - [`CrossRunEpisodeRecurrenceV1`] — the same shape recurring across runs (exact + near).
//! - [`CaseLawPrecedentIndexV1`] — an index of past cases; "this resembles precedent X" (advisory).
//! - [`FleetPlantComparisonV1`] — shapes shared across plants in a fleet (advisory).
//!
//! Every result carries an explicit advisory note: structural resemblance is a retrieval hint, not a
//! claim that the underlying mechanism is identical. Additive + off the replay path.

use serde::{Deserialize, Serialize};

use crate::hashing::CanonicalHasher;

/// The advisory note attached to every similarity result. Single source of truth.
pub const SIMILARITY_ADVISORY: &str =
    "Structural similarity only: a retrieval hint that two episodes have a similar drift/slew/motif \
     shape. NOT a claim that the underlying fault mechanism is identical, and not a causal claim.";

/// A structural fingerprint of an episode (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpisodeShapeHashV1 {
    pub episode_ref: String,
    pub n_steps: usize,
    /// Quantized grammar-state-per-step (the motif sequence) — the discrete shape.
    pub motif_sequence: Vec<u8>,
    /// Coarse continuous features for distance (e.g. [duration, peak_drift, peak_slew, exceedance_frac]).
    pub feature_vector: Vec<f64>,
    /// Exact hash of the shape (motif sequence + quantized features).
    pub shape_hash: String,
}

impl EpisodeShapeHashV1 {
    pub fn build(
        episode_ref: impl Into<String>,
        motif_sequence: Vec<u8>,
        feature_vector: Vec<f64>,
    ) -> Self {
        let episode_ref = episode_ref.into();
        let n_steps = motif_sequence.len();
        let mut h = CanonicalHasher::new();
        h.field("schema", b"episode_shape_hash_v1");
        h.field("episode_ref", episode_ref.as_bytes());
        h.field("motif_sequence", &motif_sequence);
        for &f in &feature_vector {
            h.f64q("feature", f);
        }
        let shape_hash = h.finalize_hex();
        EpisodeShapeHashV1 {
            episode_ref,
            n_steps,
            motif_sequence,
            feature_vector,
            shape_hash,
        }
    }

    /// Euclidean distance between feature vectors (the similarity metric). Differing lengths compare on
    /// the common prefix + a penalty for the length mismatch, so longer/shorter shapes are not "equal".
    pub fn distance(&self, other: &EpisodeShapeHashV1) -> f64 {
        let n = self.feature_vector.len().min(other.feature_vector.len());
        let mut s = 0.0;
        for i in 0..n {
            let d = self.feature_vector[i] - other.feature_vector[i];
            s += d * d;
        }
        // Penalise unmatched feature dimensions (treat the missing side as 0).
        for &x in self
            .feature_vector
            .iter()
            .skip(n)
            .chain(other.feature_vector.iter().skip(n))
        {
            s += x * x;
        }
        s.sqrt()
    }

    pub fn verify(&self) -> bool {
        Self::build(
            self.episode_ref.clone(),
            self.motif_sequence.clone(),
            self.feature_vector.clone(),
        )
        .shape_hash
            == self.shape_hash
    }
}

/// One nearest-neighbour hit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MotifNeighbor {
    pub motif_id: String,
    pub distance: f64,
}

/// Nearest catalogued motifs to a query shape (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MotifNearestNeighborV1 {
    pub query_ref: String,
    /// Top-k neighbours, ascending by distance.
    pub neighbors: Vec<MotifNeighbor>,
    pub nearest_motif_id: String,
    pub nearest_distance: f64,
    pub advisory: String,
    pub result_hash: String,
}

impl MotifNearestNeighborV1 {
    /// Query `library` (id → shape) for the `k` nearest motifs to `query` by feature distance.
    pub fn query(
        query: &EpisodeShapeHashV1,
        library: &[(String, EpisodeShapeHashV1)],
        k: usize,
    ) -> Self {
        let mut hits: Vec<MotifNeighbor> = library
            .iter()
            .map(|(id, shp)| MotifNeighbor {
                motif_id: id.clone(),
                distance: query.distance(shp),
            })
            .collect();
        // Deterministic sort: distance asc, then motif_id for ties.
        hits.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(core::cmp::Ordering::Equal)
                .then_with(|| a.motif_id.cmp(&b.motif_id))
        });
        hits.truncate(k);
        let (nearest_motif_id, nearest_distance) = hits
            .first()
            .map(|n| (n.motif_id.clone(), n.distance))
            .unwrap_or_default();
        let result_hash = Self::seal(
            &query.episode_ref,
            &hits,
            &nearest_motif_id,
            nearest_distance,
        );
        MotifNearestNeighborV1 {
            query_ref: query.episode_ref.clone(),
            neighbors: hits,
            nearest_motif_id,
            nearest_distance,
            advisory: SIMILARITY_ADVISORY.to_string(),
            result_hash,
        }
    }

    fn seal(query_ref: &str, hits: &[MotifNeighbor], nearest_id: &str, nearest_d: f64) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"motif_nearest_neighbor_v1");
        h.field("query_ref", query_ref.as_bytes());
        for n in hits {
            h.field("motif_id", n.motif_id.as_bytes());
            h.f64q("distance", n.distance);
        }
        h.field("nearest_motif_id", nearest_id.as_bytes());
        h.f64q("nearest_distance", nearest_d);
        h.field("advisory", SIMILARITY_ADVISORY.as_bytes());
        h.finalize_hex()
    }

    pub fn verify(&self) -> bool {
        self.advisory == SIMILARITY_ADVISORY
            && Self::seal(
                &self.query_ref,
                &self.neighbors,
                &self.nearest_motif_id,
                self.nearest_distance,
            ) == self.result_hash
    }
}

/// One occurrence of a recurring shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecurrenceHit {
    pub run_id: String,
    pub episode_ref: String,
    pub distance: f64,
}

/// The same episode shape recurring across runs (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrossRunEpisodeRecurrenceV1 {
    /// The query shape's exact hash.
    pub shape_hash: String,
    /// Occurrences across runs within `distance_tol` of the query (exact match ⇒ distance 0).
    pub occurrences: Vec<RecurrenceHit>,
    pub distance_tol: f64,
    pub n_runs: usize,
    pub recurrence_hash: String,
}

impl CrossRunEpisodeRecurrenceV1 {
    /// Find occurrences of `query`'s shape across `runs` (run_id, episode shape) within `distance_tol`.
    pub fn find(
        query: &EpisodeShapeHashV1,
        runs: &[(String, EpisodeShapeHashV1)],
        distance_tol: f64,
    ) -> Self {
        let mut occurrences: Vec<RecurrenceHit> = runs
            .iter()
            .filter_map(|(run_id, shp)| {
                let d = query.distance(shp);
                (d <= distance_tol).then(|| RecurrenceHit {
                    run_id: run_id.clone(),
                    episode_ref: shp.episode_ref.clone(),
                    distance: d,
                })
            })
            .collect();
        occurrences.sort_by(|a, b| {
            a.run_id
                .cmp(&b.run_id)
                .then_with(|| a.episode_ref.cmp(&b.episode_ref))
        });
        let n_runs = {
            let mut ids: Vec<&String> = occurrences.iter().map(|o| &o.run_id).collect();
            ids.sort();
            ids.dedup();
            ids.len()
        };
        let recurrence_hash = Self::seal(&query.shape_hash, &occurrences, distance_tol, n_runs);
        CrossRunEpisodeRecurrenceV1 {
            shape_hash: query.shape_hash.clone(),
            occurrences,
            distance_tol,
            n_runs,
            recurrence_hash,
        }
    }

    fn seal(shape_hash: &str, occ: &[RecurrenceHit], tol: f64, n_runs: usize) -> String {
        let mut h = CanonicalHasher::new();
        h.field("schema", b"cross_run_episode_recurrence_v1");
        h.field("shape_hash", shape_hash.as_bytes());
        for o in occ {
            h.field("run_id", o.run_id.as_bytes());
            h.field("episode_ref", o.episode_ref.as_bytes());
            h.f64q("distance", o.distance);
        }
        h.f64q("distance_tol", tol);
        h.u64("n_runs", n_runs as u64);
        h.finalize_hex()
    }

    pub fn verify(&self) -> bool {
        Self::seal(
            &self.shape_hash,
            &self.occurrences,
            self.distance_tol,
            self.n_runs,
        ) == self.recurrence_hash
    }
}

/// One precedent (past case) in the case-law index.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Precedent {
    pub case_id: String,
    pub shape_hash: String,
    pub label: String,
    pub disposition: String,
}

/// A case-law precedent index keyed by episode shape (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaseLawPrecedentIndexV1 {
    pub precedents: Vec<Precedent>,
    pub index_hash: String,
}

impl CaseLawPrecedentIndexV1 {
    pub fn build(mut precedents: Vec<Precedent>) -> Self {
        precedents.sort_by(|a, b| a.case_id.cmp(&b.case_id));
        let mut h = CanonicalHasher::new();
        h.field("schema", b"case_law_precedent_index_v1");
        for p in &precedents {
            h.field("case_id", p.case_id.as_bytes());
            h.field("shape_hash", p.shape_hash.as_bytes());
            h.field("label", p.label.as_bytes());
            h.field("disposition", p.disposition.as_bytes());
        }
        let index_hash = h.finalize_hex();
        CaseLawPrecedentIndexV1 {
            precedents,
            index_hash,
        }
    }

    /// Find the precedent whose stored shape (looked up in `shapes`) is nearest the `query`. Returns
    /// `(precedent, distance)` — advisory: "this resembles precedent X". `None` if the index is empty.
    pub fn nearest_precedent<'a>(
        &'a self,
        query: &EpisodeShapeHashV1,
        shapes: &[(String, EpisodeShapeHashV1)],
    ) -> Option<(&'a Precedent, f64)> {
        self.precedents
            .iter()
            .filter_map(|p| {
                shapes
                    .iter()
                    .find(|(id, _)| id == &p.shape_hash)
                    .map(|(_, shp)| (p, query.distance(shp)))
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(core::cmp::Ordering::Equal))
    }

    pub fn verify(&self) -> bool {
        Self::build(self.precedents.clone()).index_hash == self.index_hash
    }
}

/// Per-plant episode summary for fleet comparison.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlantEpisodeSummary {
    pub plant_id: String,
    pub n_episodes: usize,
    /// The most frequent episode shape_hash at this plant.
    pub dominant_shape_hash: String,
}

/// Cross-plant (fleet) episode-shape comparison (schema v1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FleetPlantComparisonV1 {
    pub plants: Vec<PlantEpisodeSummary>,
    /// Dominant shape_hashes shared by more than one plant (advisory fleet-wide patterns).
    pub shared_shapes: Vec<String>,
    pub comparison_hash: String,
    pub advisory: String,
}

impl FleetPlantComparisonV1 {
    pub fn build(mut plants: Vec<PlantEpisodeSummary>) -> Self {
        plants.sort_by(|a, b| a.plant_id.cmp(&b.plant_id));
        // shared_shapes = dominant shapes appearing at >1 plant.
        let mut shared: Vec<String> = Vec::new();
        for (i, p) in plants.iter().enumerate() {
            let count = plants
                .iter()
                .filter(|q| q.dominant_shape_hash == p.dominant_shape_hash)
                .count();
            if count > 1
                && !shared.contains(&p.dominant_shape_hash)
                && !p.dominant_shape_hash.is_empty()
            {
                let _ = i;
                shared.push(p.dominant_shape_hash.clone());
            }
        }
        shared.sort();
        let mut h = CanonicalHasher::new();
        h.field("schema", b"fleet_plant_comparison_v1");
        for p in &plants {
            h.field("plant_id", p.plant_id.as_bytes());
            h.u64("n_episodes", p.n_episodes as u64);
            h.field("dominant_shape_hash", p.dominant_shape_hash.as_bytes());
        }
        for s in &shared {
            h.field("shared_shape", s.as_bytes());
        }
        h.field("advisory", SIMILARITY_ADVISORY.as_bytes());
        let comparison_hash = h.finalize_hex();
        FleetPlantComparisonV1 {
            plants,
            shared_shapes: shared,
            comparison_hash,
            advisory: SIMILARITY_ADVISORY.to_string(),
        }
    }

    pub fn verify(&self) -> bool {
        self.advisory == SIMILARITY_ADVISORY
            && Self::build(self.plants.clone()).comparison_hash == self.comparison_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shape(reff: &str, seq: &[u8], feats: &[f64]) -> EpisodeShapeHashV1 {
        EpisodeShapeHashV1::build(reff, seq.to_vec(), feats.to_vec())
    }

    #[test]
    fn shape_hash_deterministic_distance_and_verify() {
        let a = shape("e0", &[1, 2, 2, 3], &[4.0, 0.8, 1.2, 0.5]);
        let b = shape("e0", &[1, 2, 2, 3], &[4.0, 0.8, 1.2, 0.5]);
        assert_eq!(a.shape_hash, b.shape_hash);
        assert!(a.verify());
        assert!(a.distance(&b) < 1e-12);
        let c = shape("e1", &[1, 2, 2, 3], &[9.0, 0.0, 0.0, 0.0]);
        assert!(a.distance(&c) > 0.0);
    }

    #[test]
    fn nearest_neighbor_sorts_and_carries_advisory() {
        let q = shape("q", &[1, 2], &[1.0, 1.0]);
        let lib = vec![
            ("far".to_string(), shape("a", &[1, 2], &[5.0, 5.0])),
            ("near".to_string(), shape("b", &[1, 2], &[1.1, 0.9])),
        ];
        let nn = MotifNearestNeighborV1::query(&q, &lib, 2);
        assert_eq!(nn.nearest_motif_id, "near");
        assert_eq!(nn.neighbors[0].motif_id, "near");
        assert!(nn.advisory.contains("Structural similarity only"));
        assert!(nn.verify());
    }

    #[test]
    fn recurrence_finds_same_shape_across_runs() {
        let q = shape("q", &[1, 2, 3], &[1.0, 1.0, 1.0]);
        let runs = vec![
            (
                "run_a".to_string(),
                shape("ea", &[1, 2, 3], &[1.0, 1.0, 1.0]),
            ), // exact
            (
                "run_b".to_string(),
                shape("eb", &[1, 2, 3], &[9.0, 9.0, 9.0]),
            ), // far
        ];
        let r = CrossRunEpisodeRecurrenceV1::find(&q, &runs, 0.01);
        assert_eq!(r.occurrences.len(), 1);
        assert_eq!(r.n_runs, 1);
        assert!(r.verify());
    }

    #[test]
    fn precedent_index_and_fleet_comparison_verify() {
        let idx = CaseLawPrecedentIndexV1::build(vec![Precedent {
            case_id: "C-001".into(),
            shape_hash: "abc".into(),
            label: "reactor thermal excursion".into(),
            disposition: "watch".into(),
        }]);
        assert!(idx.verify());
        let fleet = FleetPlantComparisonV1::build(vec![
            PlantEpisodeSummary {
                plant_id: "P1".into(),
                n_episodes: 3,
                dominant_shape_hash: "abc".into(),
            },
            PlantEpisodeSummary {
                plant_id: "P2".into(),
                n_episodes: 2,
                dominant_shape_hash: "abc".into(),
            },
            PlantEpisodeSummary {
                plant_id: "P3".into(),
                n_episodes: 1,
                dominant_shape_hash: "xyz".into(),
            },
        ]);
        assert_eq!(fleet.shared_shapes, vec!["abc".to_string()]); // abc shared by P1+P2
        assert!(fleet.verify());
    }
}
