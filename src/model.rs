use crate::coordinate::Coordinate;
use crate::vector::Vector;
use std::time::Duration;

const FLOAT_ZERO: f64 = 1.0e-8;

/// The Ce algorithm value.
const ERROR_LIMIT: f64 = 0.25;

/// UnitVector contains a vector that has a magnitude of 1.
#[derive(PartialEq, Debug)]
struct UnitVector<V: Vector>(V);

impl<V> UnitVector<V>
where
    V: Vector,
{
    fn new(vec: V) -> Self {
        debug_assert!(1.0 - vec.magnitude().0.abs() < FLOAT_ZERO);
        UnitVector(vec)
    }
}

/// A Vivaldi latency model generic over N dimensional vectors.
///
/// A single Model should be instantiated for each distinct network of nodes the
/// caller participates in.
///
/// Messages exchanged between nodes in the network
/// should include the current model coordinate, and the model should be updated
/// with the measured round-trip time by calling [`observe`](crate::model::Model::observe).
#[derive(Debug)]
pub struct Model<V>
where
    V: Vector + std::fmt::Debug,
{
    coordinate: Coordinate<V>,
}

impl<V> Model<V>
where
    V: Vector + std::fmt::Debug,
{
    /// New initialises a new Vivaldi model.
    ///
    /// The model is generic over any implementation of the
    /// [`Vector`](crate::vector::Vector) trait, which should be specified when
    /// calling this method:
    ///
    /// ```
    /// use vivaldi::{Model, vector::Dimension3};
    ///
    /// let model = Model::<Dimension3>::new();
    /// ```
    pub fn new() -> Model<V> {
        Model {
            coordinate: Coordinate::new(V::default(), 2.0, 0.1),
        }
    }

    /// Observe updates the positional coordinate of the local node.
    ///
    /// This method should be called with the coordinate of the remote node and
    /// the network round-trip time measured by the caller:
    ///
    /// ```
    /// # use vivaldi::{Model, vector::Dimension3, Coordinate};
    /// # let mut model = Model::<Dimension3>::new();
    /// # let mut remote_model = Model::<Dimension3>::new();
    ///
    /// // The remote sends it's current coordinate.
    /// //
    /// // Lets pretend we got this from an RPC response:
    /// let coordinate_from_remote = remote_model.get_coordinate();
    ///
    /// // The local application issues a request and measures the response time
    /// let rtt = std::time::Duration::new(0, 42_000);
    ///
    /// // And then updates the model with the remote coordinate and rtt
    /// model.observe(&coordinate_from_remote, rtt);
    /// ```
    pub fn observe(&mut self, coord: &Coordinate<V>, rtt: Duration) {
        // Sample weight balances local and remote error (1)
        //
        // 		w = ei/(ei + ej)
        //
        let weight = self.coordinate.error() / (self.coordinate.error() + coord.error());

        // Compute relative error of this sample (2)
        //
        // 		es = | ||xi -  xj|| - rtt | / rtt
        //
        let diff_vec = self.coordinate.vector().clone() - coord.vector().clone();
        let diff_mag = diff_vec.magnitude();
        let dist = estimate_rtt(&self.coordinate, &coord).as_secs_f64();
        let relative_error = (dist - rtt.as_secs_f64()).abs() / rtt.as_secs_f64();

        // Update weighted moving average of local error (3)
        //
        // 		ei = es × ce × w + ei × (1 − ce × w)
        //
        let error = relative_error * ERROR_LIMIT * weight
            + self.coordinate.error() * (1.0 - ERROR_LIMIT * weight);

        // Calculate the adaptive timestep (part of 4)
        //
        // 		δ = cc × w
        //
        let weighted_error = ERROR_LIMIT * weight;

        // Weighted force (part of 4)
        //
        // 		δ × ( rtt − ||xi − xj|| )
        //
        let weighted_force = weighted_error * (rtt.as_secs_f64() - dist);

        // Unit vector (part of 4)
        //
        // 		u(xi − xj)
        //
        let unit_vec = unit_vector_for(self.coordinate.vector().clone(), coord.vector().clone());
        let unit_vec = match unit_vec {
            Some(v) => v,
            None => new_random_unit_vec(),
        };

        // Calculate the new height of the local node:
        //
        //      (Old height + coord.Height) * weighted_force / diff_mag.0 + old height
        //
        let mut new_height = self.coordinate.height();
        if diff_mag.0 > FLOAT_ZERO {
            new_height = (self.coordinate.height() + coord.height()) * weighted_force / diff_mag.0
                + self.coordinate.height();
        }

        // Update the local coordinate (4)
        //
        // 		xi = xi + δ × ( rtt − ||xi − xj|| ) × u(xi − xj)
        //
        self.coordinate = Coordinate::new(
            self.coordinate.vector().clone() + unit_vec.0 * weighted_force,
            error,
            new_height,
        );

        // TODO: add gravity
    }

    /// Returns the current positional coordinate of the local node.
    pub fn get_coordinate(&self) -> &Coordinate<V> {
        &self.coordinate
    }
}

/// Returns an estimate round-trip time given two coordinates.
///
/// If `A` and `B` have communicated recently, the local node can estimate the
/// latency between them with a high degree of accuracy.
///
/// If the nodes represented by `A` and `B` have never communicated the
/// estimation will still be fairly accurate given a sufficiently mature, dense
/// model.
pub fn estimate_rtt<V: Vector>(a: &Coordinate<V>, b: &Coordinate<V>) -> Duration {
    let diff = a.vector().clone() - b.vector().clone();

    // Apply the fixed cost height
    let diff = diff.magnitude().0 + a.height() + b.height();

    Duration::from_secs_f64(diff)
}

/// A returns a random unit vector.
fn new_random_unit_vec<V: Vector>() -> UnitVector<V> {
    loop {
        let vec = V::random();
        let mag = vec.magnitude().0;
        if mag > FLOAT_ZERO {
            return UnitVector::new(vec / mag);
        }
    }
}

/// Returns the unit vector, or None if the division by zero is likely or the
/// magnitude too small to generate an accurate vector.
fn unit_vector_for<V: Vector>(from: V, to: V) -> Option<UnitVector<V>> {
    let diff = from - to;

    let mag = diff.magnitude().0;
    if mag < FLOAT_ZERO {
        return None;
    }

    Some(UnitVector::new(diff / mag))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vector::Dimension3;

    macro_rules! reciprocal_measurements {
        ($node_a:ident, $node_b:ident, $n:expr, $rtt:ident) => {
            for _ in 0..$n {
                $node_a.observe(&$node_b.get_coordinate(), $rtt);
                $node_b.observe(&$node_a.get_coordinate(), $rtt);
            }
        };
    }

    macro_rules! assert_within {
        ($node_a:ident, $node_b:ident, $true_value: expr, $max_diff: expr) => {
            let estimated_rtt = estimate_rtt(&$node_a.get_coordinate(), &$node_b.get_coordinate());

            let estimation_error = $true_value / estimated_rtt.as_secs_f64();
            let estimation_error = estimation_error.abs();

            println!("true rtt value:\t{}", $true_value);
            println!("estimated ett:\t{}", estimated_rtt.as_secs_f64());
            println!("error percentage:\t{:.1}", (1.0 - estimation_error) * 100.0);

            assert!(
                estimation_error < (1.0 + $max_diff),
                format!("estimation error {} is above spec", estimation_error)
            );
            assert!(
                estimation_error > (1.0 - $max_diff),
                format!("estimation error {} is below spec", estimation_error)
            );
        };
    }

    macro_rules! assert_within_spec {
        ($node_a:ident, $node_b:ident, $true_value: expr) => {
            // By default, assert to within 11.5%
            //
            // The paper states the median relative error of RTT estimates is
            // ~11%.
            assert_within!($node_a, $node_b, $true_value, 0.115);
        };
    }

    #[test]
    fn unit_vector_to() {
        let from = Dimension3([1.0, 2.0, 3.0]);
        let to = Dimension3([0.5, 1.5, 2.5]);

        assert_eq!(
            unit_vector_for(from, to),
            Some(UnitVector(Dimension3([
                0.5773502691896258,
                0.5773502691896258,
                0.5773502691896258,
            ])))
        );
    }

    #[test]
    fn independent_coords() {
        let mut a = Model::<Dimension3>::new();
        let mut b = Model::<Dimension3>::new();
        let rtt = Duration::new(1, 0);
        reciprocal_measurements!(a, b, 10, rtt);

        assert_ne!(
            a.get_coordinate().vector().0[0],
            a.get_coordinate().vector().0[1]
        );
        assert_ne!(
            a.get_coordinate().vector().0[0],
            a.get_coordinate().vector().0[2]
        )
    }

    #[test]
    fn vector_height_values() {
        #![allow(non_snake_case)]

        let mut dc1_A = Model::<Dimension3>::new();
        let mut dc1_B = Model::<Dimension3>::new();
        let mut dc2_C = Model::<Dimension3>::new();

        let fast_rtt = Duration::new(1, 0);
        let slow_rtt = Duration::new(5, 0);

        reciprocal_measurements!(dc1_A, dc1_B, 1, fast_rtt);
        reciprocal_measurements!(dc1_A, dc2_C, 1, slow_rtt);
        reciprocal_measurements!(dc1_B, dc2_C, 1, slow_rtt);

        let dc1_A_height = dc1_A.get_coordinate().height();
        let dc1_B_height = dc1_B.get_coordinate().height();
        let dc2_C_height = dc2_C.get_coordinate().height();

        reciprocal_measurements!(dc1_A, dc1_B, 1, fast_rtt);
        reciprocal_measurements!(dc1_A, dc2_C, 1, slow_rtt);
        reciprocal_measurements!(dc1_B, dc2_C, 1, slow_rtt);

        assert_ne!(dc1_A.get_coordinate().height(), dc1_A_height);
        assert_ne!(dc1_B.get_coordinate().height(), dc1_B_height);
        assert_ne!(dc2_C.get_coordinate().height(), dc2_C_height);
    }

    #[test]
    fn constant_rtt_2_node_simulation() {
        let rtt = Duration::new(1, 0);

        let mut node_a = Model::<Dimension3>::new();
        let mut node_b = Model::<Dimension3>::new();

        reciprocal_measurements!(node_a, node_b, 10, rtt);

        assert_within_spec!(node_a, node_b, rtt.as_secs_f64());
        assert_within_spec!(node_b, node_a, rtt.as_secs_f64());
    }

    #[test]
    fn constant_rtt_2x2_node_simulation() {
        #![allow(non_snake_case)]

        // Simulates 2 nodes that are close together speaking to a node over a
        // high latency link (think DC to DC).

        let mut dc1_A = Model::<Dimension3>::new();
        let mut dc1_B = Model::<Dimension3>::new();

        let mut dc2_C = Model::<Dimension3>::new();

        let fast_rtt = Duration::new(1, 0);
        let slow_rtt = Duration::new(5, 0);

        for _ in 0..100 {
            reciprocal_measurements!(dc1_A, dc1_B, 1, fast_rtt);
            reciprocal_measurements!(dc1_A, dc2_C, 1, slow_rtt);
            reciprocal_measurements!(dc1_B, dc2_C, 1, slow_rtt);
        }

        assert_within_spec!(dc1_A, dc1_B, fast_rtt.as_secs_f64());
        assert_within_spec!(dc1_A, dc2_C, slow_rtt.as_secs_f64());
        assert_within_spec!(dc1_B, dc1_A, fast_rtt.as_secs_f64());
        assert_within_spec!(dc2_C, dc1_A, slow_rtt.as_secs_f64());
        assert_within_spec!(dc1_B, dc2_C, slow_rtt.as_secs_f64());
        assert_within_spec!(dc2_C, dc1_B, slow_rtt.as_secs_f64());
    }

    #[test]
    fn constant_rtt_2x2_node_simulation_estimate_never_communicated() {
        #![allow(non_snake_case)]

        // Simulates 2 nodes that are close together speaking to a node over a
        // high latency link (think DC to DC).
        //
        // dc1_B and dc2_C never communicate, but their RTT can be estimated.

        let mut dc1_A = Model::<Dimension3>::new();
        let mut dc1_B = Model::<Dimension3>::new();

        let mut dc2_C = Model::<Dimension3>::new();

        let fast_rtt = Duration::new(1, 0);
        let slow_rtt = Duration::new(5, 0);

        for _ in 0..100 {
            reciprocal_measurements!(dc1_A, dc1_B, 1, fast_rtt);
            reciprocal_measurements!(dc1_A, dc2_C, 1, slow_rtt);
        }

        assert_within_spec!(dc1_A, dc1_B, fast_rtt.as_secs_f64());
        assert_within_spec!(dc1_A, dc2_C, slow_rtt.as_secs_f64());
        assert_within_spec!(dc1_B, dc1_A, fast_rtt.as_secs_f64());
        assert_within_spec!(dc2_C, dc1_A, slow_rtt.as_secs_f64());

        // Two nodes that have never communicated can estimate the rtt, but to a
        // lesser accuracy.
        //
        // This verifies the nodes are within ±25%. With more nodes
        // communicating, the estimation would become increasingly accurate.
        assert_within!(dc1_B, dc2_C, slow_rtt.as_secs_f64(), 0.25);
        assert_within!(dc2_C, dc1_B, slow_rtt.as_secs_f64(), 0.25);
    }
}
