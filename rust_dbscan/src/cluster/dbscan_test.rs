#[cfg(test)]
mod tests {
    use crate::cluster::{DEGREE_RAD, EARTH_R, Point, db_scan, new_kd_tree, region_query};

    #[test]
    fn test_range_query_kdtree() {
        // Verify that KD-Tree & RangeQuery give the same results
        let points = vec![
            Point([30.244759, 59.955982]),
            Point([30.24472, 59.955975]),
            Point([30.244358, 59.96698]),
            Point([30.258387, 59.951557]),
            Point([30.434124, 60.029499]),
        ];
        let tree = new_kd_tree(points.clone());
        let eps = 0.8 / EARTH_R / DEGREE_RAD;

        for pt in &points {
            let mut pts1 = tree.in_range(pt, eps, Vec::new());
            let mut pts2 = region_query(&points, pt, eps);
            pts1.sort();
            pts2.sort();
            assert_eq!(pts1, pts2);
        }
    }

    #[test]
    fn test_dbscan_basic() {
        let points = vec![
            Point([30.244759, 59.955982]),
            Point([30.24472, 59.955975]),
            Point([30.244358, 59.96698]),
            Point([30.258387, 59.951557]),
            Point([30.434124, 60.029499]),
        ];
        let (clusters, noise) = db_scan(&points, 0.8, 2);

        // Verify that clusters + noise cover whole set of points
        let mut all_points = vec![false; points.len()];
        for &i in &noise {
            all_points[i] = true;
        }
        for cluster in &clusters {
            for &i in &cluster.points {
                all_points[i] = true;
            }
        }
        assert!(all_points.iter().all(|&b| b));
    }
}
