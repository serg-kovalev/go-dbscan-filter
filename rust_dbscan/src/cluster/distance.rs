use std::f64::consts::PI;

/// Coefficient to translate from degrees to radians
pub const DEGREE_RAD: f64 = PI / 180.0;

/// Earth radius in kilometers
pub const EARTH_R: f64 = 6371.0;

// Re-export for convenience (Go-style names for compatibility)
pub use DEGREE_RAD as DegreeRad;
pub use EARTH_R as EarthR;

use super::point::Point;

/// Calculates spherical (optimized) distance between two points
///
/// # Returns
///
/// Distance in kilometers
#[allow(dead_code)] // Part of public API, may be used by external code
pub fn distance_spherical(p1: &Point, p2: &Point) -> f64 {
    let v1 = (p1.0[1] - p2.0[1]) * DEGREE_RAD;
    let v1 = v1 * v1;

    let v2 = (p1.0[0] - p2.0[0]) * DEGREE_RAD * ((p1.0[1] + p2.0[1]) / 2.0 * DEGREE_RAD).cos();
    let v2 = v2 * v2;

    EARTH_R * (v1 + v2).sqrt()
}

/// Calculates sine approximated to parabola
///
/// Taken from: <http://forum.devmaster.net/t/fast-and-accurate-sine-cosine/9648>
///
/// # Panics
///
/// Panics if `x` is outside the range `[-PI, PI]`
pub fn fast_sine(x: f64) -> f64 {
    const B: f64 = 4.0 / PI;
    const C: f64 = -4.0 / (PI * PI);
    const P: f64 = 0.225;

    if !(-PI..=PI).contains(&x) {
        panic!("out of range");
    }

    let y = B * x + C * x * x.abs();
    P * (y * y.abs() - y) + y
}

/// Calculates cosine from sine
pub fn fast_cos(x: f64) -> f64 {
    let mut x = x + PI / 2.0;
    while x > PI {
        x -= 2.0 * PI;
    }

    fast_sine(x)
}

/// Calculates spherical distance with fast cosine
/// without sqrt and normalization to Earth radius/radians
///
/// To get real distance in km, take sqrt and multiply result by `EARTH_R * DEGREE_RAD`
///
/// In this library eps (distance) is adjusted so that we don't need
/// to do sqrt and multiplication
pub fn distance_spherical_fast(p1: &Point, p2: &Point) -> f64 {
    let v1 = p1.0[1] - p2.0[1];
    let v2 = (p1.0[0] - p2.0[0]) * fast_cos((p1.0[1] + p2.0[1]) / 2.0 * DEGREE_RAD);

    v1 * v1 + v2 * v2
}

// Re-export with Go-style names for compatibility
pub use distance_spherical as DistanceSpherical;
pub use distance_spherical_fast as DistanceSphericalFast;
pub use fast_cos as FastCos;
pub use fast_sine as FastSine;
