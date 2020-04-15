use crate::vector::Vector;

/// The minimum "height" a coordinate can have.
///
/// The paper states:
///
/// ```text
/// Each node has a positive height element in its coordinates, so that
/// its height can always be scaled up or down.
/// ```
///
/// So any +ve value can act as the base.
const MIN_HEIGHT: f64 = 1.0e-5;

/// Coordinate represents a point in the Vivaldi model.
///
/// A Coordinate contains the Euclidean coordinate, estimated position error and
/// current height above the Euclidean plane.
#[derive(Debug, Default, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Coordinate<V>
where
    V: Vector,
{
    vector: V,
    error: f64,
    height: f64,
}

impl<V> Coordinate<V>
where
    V: Vector,
{
    /// Returns the current estimated position error.
    pub fn error(&self) -> f64 {
        self.error
    }

    /// Returns the Euclidean coordinate.
    pub fn vector(&self) -> &V {
        &self.vector
    }

    /// Returns the height of the Coordinate above the Euclidean plane.
    pub fn height(&self) -> f64 {
        if self.height < MIN_HEIGHT {
            return MIN_HEIGHT;
        }
        self.height
    }

    pub(crate) fn new(vector: V, error: f64, height: f64) -> Self {
        Coordinate {
            vector,
            error,
            height,
        }
    }
}

#[cfg(feature = "serde")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::vector::Dimension3;

    #[test]
    fn serde() {
        let c = Coordinate::new(Dimension3::default(), 1.0, 2.0);

        let encoded = serde_json::to_string(&c).unwrap();
        let decoded: Coordinate<Dimension3> = serde_json::from_str(&encoded).unwrap();

        assert_eq!(decoded.vector(), c.vector());
        assert_eq!(decoded.error(), c.error());
        assert_eq!(decoded.height(), c.height());
    }
}
