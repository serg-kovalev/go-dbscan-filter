#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn test_centroid_and_bounds() {
        let points = vec![
            Point([30.244759, 59.955982]),
            Point([30.24472, 59.955975]),
            Point([30.244358, 59.96698]),
        ];
        let c1 = Cluster {
            c: 0,
            points: vec![0, 1, 2],
        };

        let (center, min, max) = c1.centroid_and_bounds(&points);
        assert_eq!(center.0[0], 30.244612333333333);
        assert_eq!(center.0[1], 59.95964566666667);
        assert_eq!(min.0[0], 30.244358);
        assert_eq!(min.0[1], 59.955975);
        assert_eq!(max.0[0], 30.244759);
        assert_eq!(max.0[1], 59.96698);
    }
}
