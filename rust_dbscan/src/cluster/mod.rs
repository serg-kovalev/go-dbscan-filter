//! Package cluster implements DBScan clustering on (lat, lon) using K-D Tree
pub mod dbscan;
pub mod distance;
pub mod kdtree;
pub mod point;

#[cfg(test)]
mod dbscan_test;
#[cfg(test)]
mod distance_test;
#[cfg(test)]
mod point_test;

pub use point::{Cluster, Point, PointList};
// Public API exports - allow unused imports as these are part of the public API
#[allow(unused_imports)]
pub use dbscan::{DBScan, RegionQuery, db_scan, region_query};
#[allow(unused_imports)]
pub use distance::{
    DEGREE_RAD, DegreeRad, DistanceSpherical, DistanceSphericalFast, EARTH_R, EarthR, FastCos,
    FastSine,
};
#[allow(unused_imports)]
pub use kdtree::{KDTree, NewKDTree, new_kd_tree};
