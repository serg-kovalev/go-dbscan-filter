use super::distance::{DEGREE_RAD, EARTH_R};
use super::kdtree::new_kd_tree;
use super::point::{Cluster, Point, PointList};
use bitvec::prelude::*;

// DBSCAN algorithm pseudocode (from <http://en.wikipedia.org/wiki/DBSCAN>):
//
// DBSCAN(D, eps, MinPts)
//    C = 0
//    for each unvisited point P in dataset D
//       mark P as visited
//       NeighborPts = regionQuery(P, eps)
//       if sizeof(NeighborPts) < MinPts
//          mark P as NOISE
//       else
//          C = next cluster
//          expandCluster(P, NeighborPts, C, eps, MinPts)
//
// expandCluster(P, NeighborPts, C, eps, MinPts)
//    add P to cluster C
//    for each point P' in NeighborPts
//       if P' is not visited
//          mark P' as visited
//          NeighborPts' = regionQuery(P', eps)
//          if sizeof(NeighborPts') >= MinPts
//             NeighborPts = NeighborPts joined with NeighborPts'
//       if P' is not yet member of any cluster
//          add P' to cluster C
//
// regionQuery(P, eps)
//    return all points within P's eps-neighborhood (including P)

/// Clusters incoming points using DBSCAN algorithm
///
/// # Arguments
///
/// * `points` - List of points to cluster
/// * `eps` - Clustering radius in kilometers
/// * `min_points` - Minimum number of points in eps-neighbourhood (density threshold)
///
/// # Returns
///
/// A tuple `(clusters, noise)` where:
/// - `clusters` is a vector of found clusters
/// - `noise` is a vector of point indices that are outliers (not in any cluster)
pub fn db_scan(points: &PointList, eps: f64, min_points: usize) -> (Vec<Cluster>, Vec<usize>) {
    let mut visited = vec![false; points.len()];
    let mut members = vec![false; points.len()];
    let mut clusters = Vec::new();
    let mut noise = Vec::new();
    let mut c = 0;
    // Clone points for KD-tree construction (tree needs ownership)
    let kd_tree = new_kd_tree(points.clone());

    // Our SphericalDistanceFast returns distance which is not mutiplied
    // by EarthR * DegreeRad, adjust eps accordingly
    let eps = eps / EARTH_R / DEGREE_RAD;

    let mut neighbor_unique = bitvec![0; points.len()];

    for i in 0..points.len() {
        if visited[i] {
            continue;
        }
        visited[i] = true;

        let neighbor_pts = kd_tree.in_range(&points[i], eps, Vec::new());
        if neighbor_pts.len() < min_points {
            noise.push(i);
        } else {
            let mut cluster = Cluster { c, points: vec![i] };
            members[i] = true;
            c += 1;
            // expandCluster goes here inline
            neighbor_unique.fill(false);
            for &j in &neighbor_pts {
                neighbor_unique.set(j, true);
            }

            let mut neighbor_pts = neighbor_pts;
            let mut j = 0;
            // Use while loop to handle dynamic growth of neighbor_pts during iteration
            while j < neighbor_pts.len() {
                let k = neighbor_pts[j];
                if !visited[k] {
                    visited[k] = true;
                    let more_neighbors = kd_tree.in_range(&points[k], eps, Vec::new());
                    if more_neighbors.len() >= min_points {
                        for &p in &more_neighbors {
                            if !neighbor_unique[p] {
                                neighbor_pts.push(p);
                                neighbor_unique.set(p, true);
                            }
                        }
                    }
                }

                if !members[k] {
                    cluster.points.push(k);
                    members[k] = true;
                }
                j += 1;
            }
            clusters.push(cluster);
        }
    }

    (clusters, noise)
}

/// Simple O(N) way to find points in neighbourhood
///
/// This is roughly equivalent to `kd_tree.in_range(points[i], eps, vec![])`
#[allow(dead_code)] // Part of public API, may be used by external code
pub fn region_query(points: &PointList, p: &Point, eps: f64) -> Vec<usize> {
    let mut result = Vec::new();

    for (i, point) in points.iter().enumerate() {
        if point.sq_dist(p) < eps * eps {
            result.push(i);
        }
    }

    result
}

// Re-export with Go-style names for compatibility
pub use db_scan as DBScan;
pub use region_query as RegionQuery;
