fn cross_product(a: &[f64; 2], b: &[f64; 2]) -> f64 {
    (a[0] * b[1]) - (a[1] * b[0])
}

fn dot_product(a: &[f64; 2], b: &[f64; 2]) -> f64 {
    (a[0] * b[0]) + (a[1] * b[1])
}

/// Finds the intersection (if any) between two line segments a and b, given the line segments' end points a1, a2 and b1, b2.
///
/// This algorithm is based on Schneider and Eberly.
/// http://www.cimec.org.ar/~ncalvo/Schneider_Eberly.pdf Page 244.
///
/// Params
///
/// * a1 point of first line
/// * a2 point of first line
/// * b1 point of second line
/// * b2 point of second line
/// * noEndpointTouch whether to skip single touchpoints (meaning connected segments) as intersections
///
/// Returns
///
/// If the lines intersect, the point of intersection. If they overlap, the two end points of the overlapping segment.
/// Otherwise, None.
pub(crate) fn intersection(
    a1: &[f64; 2],
    a2: &[f64; 2],
    b1: &[f64; 2],
    b2: &[f64; 2],
    no_endpoint_touch: bool,
) -> Option<Vec<[f64; 2]>> {
    // The algorithm expects our lines in the form P + sd, where P is a point,
    // s is on the interval [0, 1], and d is a vector.
    // We are passed two points. P can be the first point of each pair. The
    // vector, then, could be thought of as the distance (in x and y components)
    // from the first point to the second point.
    // So first, let's make our vectors:
    let va = [a2[0] - a1[0], a2[1] - a1[1]];
    let vb = [b2[0] - b1[0], b2[1] - b1[1]];
    // We also define a function to convert back to regular point form:

    let to_point =
        |p: &[f64; 2], s: f64, d: &[f64; 2]| -> [f64; 2] { [p[0] + s * d[0], p[1] + s * d[1]] };

    // The rest is pretty much a straight port of the algorithm.
    let e = [b1[0] - a1[0], b1[1] - a1[1]];
    let kross = cross_product(&va, &vb);
    let sqr_kross = kross * kross;
    let sqr_len_a = dot_product(&va, &va);

    // Check for line intersection. This works because of the properties of the
    // cross product -- specifically, two vectors are parallel if and only if the
    // cross product is the 0 vector. The full calculation involves relative error
    // to account for possible very small line segments. See Schneider & Eberly
    // for details.
    if sqr_kross > 0.0 {
        // If they're not parallel, then (because these are line segments) they
        // still might not actually intersect. This code checks that the
        // intersection point of the lines is actually on both line segments.
        let s = cross_product(&e, &vb) / kross;
        #[allow(clippy::manual_range_contains)]
        if s < 0.0 || s > 1.0 {
            // not on line segment a
            return None;
        }
        let t = cross_product(&e, &va) / kross;
        #[allow(clippy::manual_range_contains)]
        if t < 0.0 || t > 1.0 {
            // not on line segment b
            return None;
        }
        if s == 0.0 || s == 1.0 {
            // on an endpoint of line segment a
            return if no_endpoint_touch {
                None
            } else {
                Some(vec![to_point(a1, s, &va)])
            };
        }
        if t == 0.0 || t == 1.0 {
            // on an endpoint of line segment b
            return if no_endpoint_touch {
                None
            } else {
                Some(vec![to_point(b1, t, &vb)])
            };
        }
        return Some(vec![to_point(a1, s, &va)]);
    }

    // If we've reached this point, then the lines are either parallel or the
    // same, but the segments could overlap partially or fully, or not at all.
    // So we need to find the overlap, if any. To do that, we can use e, which is
    // the (vector) difference between the two initial points. If this is parallel
    // with the line itself, then the two lines are the same line, and there will
    // be overlap.
    let kross = cross_product(&e, &va);
    let sqr_kross = kross * kross;

    if sqr_kross > 0.0 {
        // Lines are just parallel, not the same. No overlap.
        return None;
    }

    let sa = dot_product(&va, &e) / sqr_len_a;
    let sb = sa + dot_product(&va, &vb) / sqr_len_a;
    let smin = sa.min(sb);
    let smax = sa.max(sb);

    // this is, essentially, the FindIntersection acting on floats from
    // Schneider & Eberly, just inlined into this function.
    if smin <= 1.0 && smax >= 0.0 {
        // overlap on an end point
        if smin == 1.0 {
            return if no_endpoint_touch {
                None
            } else {
                Some(vec![to_point(a1, if smin > 0.0 { smin } else { 0.0 }, &va)])
            };
        }

        if smax == 0.0 {
            return if no_endpoint_touch {
                None
            } else {
                Some(vec![to_point(a1, if smax < 1.0 { smax } else { 1.0 }, &va)])
            };
        }

        if no_endpoint_touch && smin == 0.0 && smax == 1.0 {
            return None;
        }

        // There's overlap on a segment -- two points of intersection. Return both.
        return Some(vec![
            to_point(a1, if smin > 0.0 { smin } else { 0.0 }, &va),
            to_point(a1, if smax < 1.0 { smax } else { 1.0 }, &va),
        ]);
    }

    None
}

#[cfg(test)]
mod tests {
    use crate::segment_intersection::intersection;

    #[test]
    fn intersection_test() {
        assert_eq!(
            intersection(&[0.0, 0.0], &[1.0, 1.0], &[1.0, 0.0], &[2.0, 2.0], false),
            None,
            "null if no intersections"
        );
        assert_eq!(
            intersection(&[0.0, 0.0], &[1.0, 1.0], &[1.0, 0.0], &[10.0, 2.0], false),
            None,
            "null if no intersections"
        );
        assert_eq!(
            intersection(&[2.0, 2.0], &[3.0, 3.0], &[0.0, 6.0], &[2.0, 4.0], false),
            None,
            "null if no intersections"
        );

        assert_eq!(
            intersection(&[0.0, 0.0], &[1.0, 1.0], &[1.0, 0.0], &[0.0, 1.0], false),
            Some(vec![[0.5, 0.5]]),
            "1 intersection"
        );
        assert_eq!(
            intersection(&[0.0, 0.0], &[1.0, 1.0], &[0.0, 1.0], &[0.0, 0.0], false),
            Some(vec![[0.0, 0.0]]),
            "shared point 1.0"
        );
        assert_eq!(
            intersection(&[0.0, 0.0], &[1.0, 1.0], &[0.0, 1.0], &[1.0, 1.0], false),
            Some(vec![[1.0, 1.0]]),
            "shared point 2.0"
        );

        assert_eq!(
            intersection(&[0.0, 0.0], &[1.0, 1.0], &[0.5, 0.5], &[1.0, 0.0], false),
            Some(vec![[0.5, 0.5]]),
            "T-crossing"
        );

        assert_eq!(
            intersection(&[0.0, 0.0], &[10.0, 10.0], &[1.0, 1.0], &[5.0, 5.0], false),
            Some(vec![[1.0, 1.0], [5.0, 5.0]]),
            "full overlap"
        );
        assert_eq!(
            intersection(&[1.0, 1.0], &[10.0, 10.0], &[1.0, 1.0], &[5.0, 5.0], false),
            Some(vec![[1.0, 1.0], [5.0, 5.0]]),
            "shared point + overlap"
        );
        assert_eq!(
            intersection(&[3.0, 3.0], &[10.0, 10.0], &[0.0, 0.0], &[5.0, 5.0], false),
            Some(vec![[3.0, 3.0], [5.0, 5.0]]),
            "mutual overlap"
        );
        assert_eq!(
            intersection(&[0.0, 0.0], &[1.0, 1.0], &[0.0, 0.0], &[1.0, 1.0], false),
            Some(vec![[0.0, 0.0], [1.0, 1.0]]),
            "full overlap"
        );
        assert_eq!(
            intersection(&[1.0, 1.0], &[0.0, 0.0], &[0.0, 0.0], &[1.0, 1.0], false),
            Some(vec![[1.0, 1.0], [0.0, 0.0]]),
            "full overlap, orientation"
        );

        assert_eq!(
            intersection(&[0.0, 0.0], &[1.0, 1.0], &[1.0, 1.0], &[2.0, 2.0], false),
            Some(vec![[1.0, 1.0]]),
            "collinear, shared point"
        );
        assert_eq!(
            intersection(&[1.0, 1.0], &[0.0, 0.0], &[1.0, 1.0], &[2.0, 2.0], false),
            Some(vec![[1.0, 1.0]]),
            "collinear, shared other point"
        );
        assert_eq!(
            intersection(&[0.0, 0.0], &[1.0, 1.0], &[2.0, 2.0], &[4.0, 4.0], false),
            None,
            "collinear, no overlap"
        );
        assert_eq!(
            intersection(&[0.0, 0.0], &[1.0, 1.0], &[0.0, -1.0], &[1.0, 0.0], false),
            None,
            "parallel"
        );
        assert_eq!(
            intersection(&[1.0, 1.0], &[0.0, 0.0], &[0.0, -1.0], &[1.0, 0.0], false),
            None,
            "parallel, orientation"
        );
        assert_eq!(
            intersection(&[0.0, -1.0], &[1.0, 0.0], &[0.0, 0.0], &[1.0, 1.0], false),
            None,
            "parallel, position"
        );

        assert_eq!(
            intersection(&[0.0, 0.0], &[1.0, 1.0], &[0.0, 1.0], &[0.0, 0.0], true),
            None,
            "shared point 1.0, skip touches"
        );
        assert_eq!(
            intersection(&[0.0, 0.0], &[1.0, 1.0], &[0.0, 1.0], &[1.0, 1.0], true),
            None,
            "shared point 2.0, skip touches"
        );

        assert_eq!(
            intersection(&[0.0, 0.0], &[1.0, 1.0], &[1.0, 1.0], &[2.0, 2.0], true),
            None,
            "collinear, shared point, skip touches"
        );
        assert_eq!(
            intersection(&[1.0, 1.0], &[0.0, 0.0], &[1.0, 1.0], &[2.0, 2.0], true),
            None,
            "collinear, shared other point, skip touches"
        );

        assert_eq!(
            intersection(&[0.0, 0.0], &[1.0, 1.0], &[0.0, 0.0], &[1.0, 1.0], true),
            None,
            "full overlap, skip touches"
        );
        assert_eq!(
            intersection(&[1.0, 1.0], &[0.0, 0.0], &[0.0, 0.0], &[1.0, 1.0], true),
            None,
            "full overlap, orientation, skip touches"
        );

        assert_eq!(
            intersection(&[0.0, 0.0], &[1.0, 1.0], &[1.0, 0.0], &[0.0, 1.0], true),
            Some(vec![[0.5, 0.5]]),
            "1 intersection, skip touches"
        );
    }
}
