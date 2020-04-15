use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;

mod dimension_2;
pub use dimension_2::Dimension2;

mod dimension_3;
pub use dimension_3::Dimension3;

/// An trait to allow the [`Model`](crate::model::Model) to operate in N dimensional Euclidean space.
pub trait Vector:
    Add<Output = Self>
    + Add<f64, Output = Self>
    + Sub<Output = Self>
    + Mul<f64, Output = Self>
    + Div<f64, Output = Self>
    + Clone
    + Default
{
    /// Returns the magnitude of the vector.
    fn magnitude(&self) -> Magnitude;

    /// Returns a random vector.
    fn random() -> Self;
}

/// Magnitude is a newtype alias holding the magnitude value of a vector.
#[derive(PartialEq, Debug)]
pub struct Magnitude(pub f64);
