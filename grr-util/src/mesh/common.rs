pub use grr::{Buffer, Device, VertexArray};
use num_traits::Float;

pub type V3 = nalgebra::Vector3<f32>;

/// Return `n` equally spaced values from start to end.
pub fn linspace<F: Float + 'static>(start: F, end: F, n: usize) -> impl Iterator<Item = F> {
    let df = (end - start) / F::from(n - 1).unwrap();
    (0..n).map(move |i| start + df.clone() * F::from(i).unwrap())
}
