use super::*;
use rand::Rng;
use std::ops::Div;

/// A 2 dimensional Euclidean vector.
#[derive(PartialEq, Debug, Copy, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Dimension2(pub [f64; 2]);

impl Vector for Dimension2 {
    fn magnitude(&self) -> Magnitude {
        let m = self.0.iter().fold(0.0, |acc, v| acc + (v * v)).sqrt();

        Magnitude(m)
    }

    fn random() -> Self {
        Dimension2([
            rand::thread_rng().gen::<f64>(),
            rand::thread_rng().gen::<f64>(),
        ])
    }
}

impl Add for Dimension2 {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self([self.0[0] + other.0[0], self.0[1] + other.0[1]])
    }
}

impl Add<f64> for Dimension2 {
    type Output = Self;

    fn add(self, other: f64) -> Self::Output {
        Self([self.0[0] + other, self.0[1] + other])
    }
}

impl Sub for Dimension2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self([self.0[0] - other.0[0], self.0[1] - other.0[1]])
    }
}

/// Divide a vector by a constant amount.
impl Div<f64> for Dimension2 {
    type Output = Self;

    fn div(self, other: f64) -> Self::Output {
        Self([self.0[0] / other, self.0[1] / other])
    }
}

impl Mul<f64> for Dimension2 {
    type Output = Self;

    fn mul(self, other: f64) -> Self::Output {
        Self([self.0[0] * other, self.0[1] * other])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add() {
        let a = Dimension2([1.0, 2.0]);
        let b = Dimension2([0.1, 0.2]);

        assert_eq!(a + b, Dimension2([1.1, 2.2]));
    }

    #[test]
    fn add_f64_constant() {
        assert_eq!(Dimension2([1.0, 2.0]) + 42.0, Dimension2([43.0, 44.0]));
    }

    #[test]
    fn sub() {
        let a = Dimension2([1.1, 2.2]);
        let b = Dimension2([0.1, 0.2]);

        assert_eq!(a - b, Dimension2([1.0, 2.0]));
    }

    #[test]
    fn mul_f64_constant() {
        let a = Dimension2([1.0, 2.0]);

        assert_eq!(a * 2.0, Dimension2([2.0, 4.0]));
    }

    #[test]
    fn div_f64_constant() {
        assert_eq!(Dimension2([1.0, 2.0]) / 2.0, Dimension2([0.5, 1.0]));
    }

    #[test]
    fn magnitude() {
        assert_eq!(Dimension2([0.0, 0.0]).magnitude(), Magnitude(0.0));

        // Non-zero magnitude
        assert_eq!(
            Dimension2([1.0, 2.0]).magnitude(),
            Magnitude(2.23606797749979)
        );

        // Direction plays no part
        assert_eq!(
            Dimension2([-1.0, -2.0]).magnitude(),
            Magnitude(2.23606797749979)
        );
    }
}
