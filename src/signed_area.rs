use robust::{Coord, orient2d};

/// Signed area of the triangle (p0, p1, p2)
pub(crate) fn signed_area(p0: &[f64; 2], p1: &[f64; 2], p2: &[f64; 2]) -> f64 {
    let res = orient2d(
        Coord { x: p0[0], y: p0[1] },
        Coord { x: p1[0], y: p1[1] },
        Coord { x: p2[0], y: p2[1] },
    );
    if res > 0.0 {
        1.0
    } else if res < 0.0 {
        -1.0
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use crate::signed_area::signed_area;

    #[test]
    fn analytical_signed_area() {
        assert_eq!(
            signed_area(&[0.0, 0.0], &[0.0, 1.0], &[1.0, 1.0]),
            -1.0,
            "negative area"
        );
        assert_eq!(
            signed_area(&[0.0, 1.0], &[0.0, 0.0], &[1.0, 0.0]),
            1.0,
            "positive area"
        );
        assert_eq!(
            signed_area(&[0.0, 0.0], &[1.0, 1.0], &[2.0, 2.0]),
            0.0,
            "collinear, 0 area"
        );

        assert_eq!(
            signed_area(&[-1.0, 0.0], &[2.0, 3.0], &[0.0, 1.0]),
            0.0,
            "point on segment"
        );
        assert_eq!(
            signed_area(&[2.0, 3.0], &[-1.0, 0.0], &[0.0, 1.0]),
            0.0,
            "point on segment"
        );
    }
}
