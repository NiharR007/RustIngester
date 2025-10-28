/// Simple sign-random-projection LSH (very naive). Deterministic seeds for demo.
use ndarray::{Array1, Array2};

pub struct Lsh {
    projections: Array2<f32>,
    buckets: usize,
}

impl Lsh {
    pub fn new(dim: usize, buckets: usize) -> Self {
        use rand::{Rng, SeedableRng};
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let projections = Array2::from_shape_fn((buckets as usize, dim), |_| rng.gen::<f32>());
        Self { projections, buckets }
    }

    pub fn hash(&self, v: &[f32]) -> usize {
        let v = Array1::from_vec(v.to_vec());
        let mut bits = 0usize;
        for (i, row) in self.projections.outer_iter().enumerate() {
            let dot = row.dot(&v);
            if dot > 0.0 && i < 63 { // Prevent overflow on 64-bit systems
                bits |= 1 << i;
            }
        }
        bits % self.buckets
    }
}
