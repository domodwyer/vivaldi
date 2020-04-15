use super::*;
use rand::Rng;
use std::ops::Div;

/// A 3 dimensional Euclidean vector.
#[derive(PartialEq, Debug, Copy, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Dimension3(pub [f64; 3]);

impl Vector for Dimension3 {
    fn magnitude(&self) -> Magnitude {
        let m = self.0.iter().fold(0.0, |acc, v| acc + (v * v)).sqrt();

        Magnitude(m)
    }

    fn random() -> Self {
        Dimension3([
            rand::thread_rng().gen::<f64>(),
            rand::thread_rng().gen::<f64>(),
            rand::thread_rng().gen::<f64>(),
        ])
    }
}

impl Add for Dimension3 {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self([
            self.0[0] + other.0[0],
            self.0[1] + other.0[1],
            self.0[2] + other.0[2],
        ])
    }
}

impl Add<f64> for Dimension3 {
    type Output = Self;

    fn add(self, other: f64) -> Self::Output {
        Self([self.0[0] + other, self.0[1] + other, self.0[2] + other])
    }
}

impl Sub for Dimension3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self([
            self.0[0] - other.0[0],
            self.0[1] - other.0[1],
            self.0[2] - other.0[2],
        ])
    }
}

/// Divide a vector by a constant amount.
impl Div<f64> for Dimension3 {
    type Output = Self;

    fn div(self, other: f64) -> Self::Output {
        Self([self.0[0] / other, self.0[1] / other, self.0[2] / other])
    }
}

impl Mul<f64> for Dimension3 {
    type Output = Self;

    fn mul(self, other: f64) -> Self::Output {
        Self([self.0[0] * other, self.0[1] * other, self.0[2] * other])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add() {
        let a = Dimension3([1.0, 2.0, 3.0]);
        let b = Dimension3([0.1, 0.2, 0.3]);

        assert_eq!(a + b, Dimension3([1.1, 2.2, 3.3]));
    }

    #[test]
    fn add_f64_constant() {
        assert_eq!(
            Dimension3([1.0, 2.0, 3.0]) + 42.0,
            Dimension3([43.0, 44.0, 45.0])
        );
    }

    #[test]
    fn sub() {
        let a = Dimension3([1.1, 2.2, 3.3]);
        let b = Dimension3([0.1, 0.2, 0.3]);

        assert_eq!(a - b, Dimension3([1.0, 2.0, 3.0]));
    }

    #[test]
    fn mul_f64_constant() {
        let a = Dimension3([1.0, 2.0, 3.0]);

        assert_eq!(a * 2.0, Dimension3([2.0, 4.0, 6.0]));
    }

    #[test]
    fn div_f64_constant() {
        assert_eq!(
            Dimension3([1.0, 2.0, 3.0]) / 2.0,
            Dimension3([0.5, 1.0, 1.5])
        );
    }

    #[test]
    fn magnitude() {
        assert_eq!(Dimension3([0.0, 0.0, 0.0]).magnitude(), Magnitude(0.0));

        // Non-zero magnitude
        assert_eq!(
            Dimension3([1.0, 2.0, 3.0]).magnitude(),
            Magnitude(3.7416573867739413)
        );

        // Direction plays no part
        assert_eq!(
            Dimension3([-1.0, -2.0, -3.0]).magnitude(),
            Magnitude(3.7416573867739413)
        );
    }
}
