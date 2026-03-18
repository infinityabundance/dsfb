use crate::lattice::Lattice;

#[derive(Clone, Debug)]
pub struct PointDefectSpec {
    pub site: usize,
    pub mass_scale: f64,
    pub spring_index: usize,
    pub spring_scale: f64,
}

pub fn point_defect(base: &Lattice, spec: &PointDefectSpec) -> Lattice {
    let mut lattice = base.clone();
    lattice.label = "point_defect".to_string();
    if spec.site < lattice.masses.len() {
        lattice.masses[spec.site] *= spec.mass_scale;
    }
    if spec.spring_index < lattice.springs.len() {
        lattice.springs[spec.spring_index] *= spec.spring_scale;
    }
    lattice
}

pub fn distributed_strain(base: &Lattice, strength: f64) -> Lattice {
    let mut lattice = base.clone();
    lattice.label = "distributed_strain".to_string();
    let count = (lattice.springs.len() - 1) as f64;
    for (index, spring) in lattice.springs.iter_mut().enumerate() {
        let position = index as f64 / count;
        let normalized = 2.0 * position - 1.0;
        *spring *= (1.0 + strength * normalized).max(0.15);
    }
    lattice
}

pub fn grouped_cluster(base: &Lattice, center: usize, width: f64, strength: f64) -> Lattice {
    let mut lattice = base.clone();
    lattice.label = "group_mode_cluster".to_string();

    for (index, spring) in lattice.springs.iter_mut().enumerate() {
        let distance = index as f64 - center as f64;
        let envelope = (-0.5 * (distance / width).powi(2)).exp();
        *spring *= (1.0 - strength * envelope).max(0.18);
    }

    for (index, mass) in lattice.masses.iter_mut().enumerate() {
        let distance = index as f64 - center as f64;
        let envelope = (-0.5 * (distance / (width + 0.5)).powi(2)).exp();
        *mass *= 1.0 + 0.18 * envelope;
    }

    lattice
}

pub fn global_softening(base: &Lattice, spring_scale: f64) -> Lattice {
    let mut lattice = base.clone();
    lattice.label = format!("softening_scale_{spring_scale:.3}");
    for spring in &mut lattice.springs {
        *spring *= spring_scale.max(0.05);
    }
    lattice
}
