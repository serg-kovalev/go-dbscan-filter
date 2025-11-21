#[cfg(test)]
mod tests {
    use crate::cluster::{Point, PointList, db_scan};
    use crate::{build_labels, filter_points, read_points_and_csv};
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_main_program() {
        // Create a test CSV file
        let test_csv = "latitude,longitude
40.7128,-74.0060
40.7130,-74.0062
40.7132,-74.0064
40.7500,-73.9900
40.7502,-73.9902
40.7504,-73.9904
40.8000,-73.9500
41.0000,-74.0000";

        let test_file = PathBuf::from("test_points_rust.csv");
        fs::write(&test_file, test_csv).expect("Failed to create test CSV");

        // Read points
        let (points, _) = read_points_and_csv(&test_file).expect("Failed to read CSV");

        assert_eq!(points.len(), 8);

        // Test DBSCAN
        let (clusters, noise) = db_scan(&points, 0.1, 3);

        assert!(!clusters.is_empty() || !noise.is_empty());

        // Build labels and test filtering
        let labels = build_labels(&clusters, &noise, points.len());
        let filtered_indices = filter_points(&points, &labels);

        // Verify filtering logic:
        // 1. All outliers should be included
        for &noise_idx in &noise {
            assert!(
                filtered_indices.contains(&noise_idx),
                "Noise point at index {} should be in filtered results",
                noise_idx
            );
        }

        // 2. First point in each cluster should be included
        for cluster in &clusters {
            if !cluster.points.is_empty() {
                let first_point = cluster.points[0];
                assert!(
                    filtered_indices.contains(&first_point),
                    "First point of cluster {} (index {}) should be in filtered results",
                    cluster.c,
                    first_point
                );
            }
        }

        // Clean up
        fs::remove_file(&test_file).ok();
    }

    #[test]
    fn test_filter_points_logic() {
        // Test the Ruby-style filtering logic
        let test_cases = vec![
            ("all outliers", vec![-1, -1, -1], 3, vec![0, 1, 2]),
            ("single cluster", vec![0, 0, 0], 1, vec![0]),
            ("two clusters", vec![0, 0, 1, 1], 2, vec![0, 2]),
            (
                "mixed outliers and clusters",
                vec![-1, 0, 0, -1, 1, 1],
                4,
                vec![0, 1, 3, 4],
            ),
        ];

        for (name, labels, expected_count, expected_indices) in test_cases {
            // Create mock points matching the labels length
            let points: PointList = (0..labels.len())
                .map(|i| Point([i as f64, i as f64]))
                .collect();

            let result = filter_points(&points, &labels);
            assert_eq!(result.len(), expected_count, "Test case: {}", name);
            for (i, &expected_idx) in expected_indices.iter().enumerate() {
                if i < result.len() {
                    assert_eq!(result[i], expected_idx, "Test case: {}", name);
                }
            }
        }
    }
}
