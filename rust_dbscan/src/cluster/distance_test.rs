#[cfg(test)]
mod tests {
    use crate::cluster::Point;
    use crate::cluster::distance::{
        DEGREE_RAD, EARTH_R, distance_spherical, distance_spherical_fast, fast_cos,
    };

    #[test]
    fn test_fast_cos() {
        assert!((fast_cos(0.0) - 1.0).abs() < 0.001);
        assert!((fast_cos(0.1) - 0.1_f64.cos()).abs() < 0.001);
        assert!((fast_cos(-0.1) - (-0.1_f64).cos()).abs() < 0.001);
        assert!((fast_cos(1.0) - 1.0_f64.cos()).abs() < 0.001);
    }

    #[test]
    fn test_distance_spherical() {
        let p1 = Point([30.244759, 59.955982]);
        let p2 = Point([30.24472, 59.955975]);
        let expected = 0.0023064907653812116;
        let actual1 = distance_spherical(&p1, &p2);
        let actual2 = distance_spherical(&p2, &p1);
        assert!((actual1 - expected).abs() < 1e-10);
        assert!((actual2 - expected).abs() < 1e-10);
        assert_eq!(distance_spherical(&p1, &p1), 0.0);
        assert_eq!(distance_spherical(&p2, &p2), 0.0);
    }

    #[test]
    fn test_distance_spherical_fast() {
        let p1 = Point([30.244759, 59.955982]);
        let p2 = Point([30.24472, 59.955975]);
        let expected = 4.3026720164084415e-10;
        let actual1 = distance_spherical_fast(&p1, &p2);
        let actual2 = distance_spherical_fast(&p2, &p1);
        assert!((actual1 - expected).abs() < 1e-15);
        assert!((actual2 - expected).abs() < 1e-15);
        assert_eq!(distance_spherical_fast(&p1, &p1), 0.0);
        assert_eq!(distance_spherical_fast(&p2, &p2), 0.0);

        assert!(
            ((distance_spherical_fast(&p1, &p2).sqrt() * DEGREE_RAD * EARTH_R
                - distance_spherical(&p1, &p2))
            .abs()
                < 0.000001)
        );
    }
}
