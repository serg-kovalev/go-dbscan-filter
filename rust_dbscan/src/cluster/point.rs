//! Package cluster implements DBScan clustering on (lat, lon) using K-D Tree

/// Point represents a geographic coordinate (longitude, latitude)
///
/// The point is stored as [longitude, latitude] where:
/// - `[0]` is longitude
/// - `[1]` is latitude
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point(pub [f64; 2]);

/// PointList is a collection of Points
pub type PointList = Vec<Point>;

/// Cluster represents a result of DBScan clustering work
#[derive(Debug, Clone)]
pub struct Cluster {
    /// Cluster ID
    pub c: usize,
    /// Indices of points belonging to this cluster
    pub points: Vec<usize>,
}

impl Point {
    /// Returns squared (without sqrt & normalization) distance between two points
    pub fn sq_dist(&self, b: &Point) -> f64 {
        use super::distance::DistanceSphericalFast;
        DistanceSphericalFast(self, b)
    }

    /// Checks if this point is less than or equal to another point
    /// (a <= b)
    #[allow(dead_code)] // Part of public API, may be used by external code
    pub fn less_eq(&self, b: &Point) -> bool {
        self.0[0] <= b.0[0] && self.0[1] <= b.0[1]
    }

    /// Checks if this point is greater than or equal to another point
    /// (a >= b)
    #[allow(dead_code)] // Part of public API, may be used by external code
    pub fn greater_eq(&self, b: &Point) -> bool {
        self.0[0] >= b.0[0] && self.0[1] >= b.0[1]
    }
}

impl Cluster {
    /// Calculates center and cluster bounds
    ///
    /// Returns `(center, min, max)` where:
    /// - `center` is the centroid of the cluster
    /// - `min` is the minimum point (bottom-left)
    /// - `max` is the maximum point (top-right)
    ///
    /// # Panics
    ///
    /// Panics if the cluster is empty
    #[allow(dead_code)] // Part of public API, may be used by external code
    pub fn centroid_and_bounds(&self, points: &PointList) -> (Point, Point, Point) {
        if self.points.is_empty() {
            panic!("empty cluster");
        }

        let mut min = Point([180.0, 90.0]);
        let mut max = Point([-180.0, -90.0]);
        let mut center = Point([0.0, 0.0]);

        for &i in &self.points {
            let pt = points[i];

            for j in 0..2 {
                center.0[j] += pt.0[j];

                if pt.0[j] < min.0[j] {
                    min.0[j] = pt.0[j];
                }
                if pt.0[j] > max.0[j] {
                    max.0[j] = pt.0[j];
                }
            }
        }

        for j in 0..2 {
            center.0[j] /= self.points.len() as f64;
        }

        (center, min, max)
    }
}

/// Checks if (innerMin, innerMax) rectangle is inside (outerMin, outerMax) rectangle
#[allow(dead_code)] // Part of public API, may be used by external code
pub fn inside(inner_min: &Point, inner_max: &Point, outer_min: &Point, outer_max: &Point) -> bool {
    inner_min.greater_eq(outer_min) && inner_max.less_eq(outer_max)
}
