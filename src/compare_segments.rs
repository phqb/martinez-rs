use core::cmp::Ordering;

use crate::{
    compare_events::compare_events,
    equals::equals,
    signed_area::signed_area,
    sweep_event::{SweepEvent, SweepEventArena},
};

pub(crate) fn compare_segments(
    le1: &SweepEvent,
    le2: &SweepEvent,
    arena: &SweepEventArena,
) -> Ordering {
    if std::ptr::eq(le1, le2) {
        return Ordering::Equal;
    }

    // Segments are not collinear
    if signed_area(
        &le1.point,
        &arena[le1.other_event.expect("SweepEvent.other_event")].point,
        &le2.point,
    ) != 0.0
        || signed_area(
            &le1.point,
            &arena[le1.other_event.expect("SweepEvent.other_event")].point,
            &arena[le2.other_event.expect("SweepEvent.other_event")].point,
        ) != 0.0
    {
        // If they share their left endpoint use the right endpoint to sort
        if equals(&le1.point, &le2.point) {
            return if le1.is_below(
                &arena[le2.other_event.expect("SweepEvent.other_event")].point,
                arena,
            ) {
                Ordering::Less
            } else {
                Ordering::Greater
            };
        }

        // Different left endpoint: use the left endpoint to sort
        if le1.point[0] == le2.point[0] {
            return if le1.point[1] < le2.point[1] {
                Ordering::Less
            } else {
                Ordering::Greater
            };
        }

        // has the line segment associated to e1 been inserted
        // into S after the line segment associated to e2 ?
        if compare_events(le1, le2, arena) == Ordering::Greater {
            return if le2.is_above(&le1.point, arena) {
                Ordering::Less
            } else {
                Ordering::Greater
            };
        }

        // The line segment associated to e2 has been inserted
        // into S after the line segment associated to e1
        return if le1.is_below(&le2.point, arena) {
            Ordering::Less
        } else {
            Ordering::Greater
        };
    }

    if le1.is_subject == le2.is_subject {
        // same polygon
        let p1 = le1.point;
        let p2 = le2.point;

        if p1[0] == p2[0] && p1[1] == p2[1]
        // if equals(&le1.point, &le2.point)
        {
            let p1 = arena[le1.other_event.expect("SweepEvent.other_event")].point;
            let p2 = arena[le2.other_event.expect("SweepEvent.other_event")].point;
            if p1[0] == p2[0] && p1[1] == p2[1] {
                return Ordering::Equal;
            } else {
                return if le1.contour_id > le2.contour_id {
                    Ordering::Greater
                } else {
                    Ordering::Less
                };
            }
        }
    } else {
        // Segments are collinear, but belong to separate polygons
        return if le1.is_subject {
            Ordering::Less
        } else {
            Ordering::Greater
        };
    }

    if compare_events(le1, le2, arena) == Ordering::Greater {
        Ordering::Greater
    } else {
        Ordering::Less
    }
}

#[cfg(test)]
#[allow(clippy::bool_assert_comparison)]
mod tests {
    use crate::{
        compare_events::compare_events,
        compare_segments::compare_segments,
        sweep_event::{SweepEvent, SweepEventArena},
        tree::TreeByCompareSegments,
    };
    use core::cmp::Ordering;

    #[test]
    fn compare_segments_not_collinear_shared_left_point_right_point_first() {
        let mut arena = SweepEventArena::new();
        let mut tree = TreeByCompareSegments::new();
        let pt = [0.0, 0.0];
        let (_, se1_id) = SweepEvent::new(
            pt,
            true,
            Some(SweepEvent::with_point_and_left([1.0, 1.0], false, &mut arena).1),
            false,
            None,
            &mut arena,
        );
        let (_, se2_id) = SweepEvent::new(
            pt,
            true,
            Some(SweepEvent::with_point_and_left([2.0, 3.0], false, &mut arena).1),
            false,
            None,
            &mut arena,
        );

        tree.insert(se2_id, &arena);
        tree.insert(se1_id, &arena);

        assert_eq!(
            arena[arena[tree.max(&arena).unwrap()].other_event.unwrap()].point,
            [2.0, 3.0]
        );
        assert_eq!(
            arena[arena[tree.min(&arena).unwrap()].other_event.unwrap()].point,
            [1.0, 1.0]
        );
    }

    #[test]
    fn compare_segments_not_collinear_different_left_point_right_point_y_coord_to_sort() {
        let mut arena = SweepEventArena::new();
        let mut tree = TreeByCompareSegments::new();
        let (_, se1_id) = SweepEvent::new(
            [0.0, 1.0],
            true,
            Some(SweepEvent::with_point_and_left([1.0, 1.0], false, &mut arena).1),
            false,
            None,
            &mut arena,
        );
        let (_, se2_id) = SweepEvent::new(
            [0.0, 2.0],
            true,
            Some(SweepEvent::with_point_and_left([2.0, 3.0], false, &mut arena).1),
            false,
            None,
            &mut arena,
        );

        tree.insert(se2_id, &arena);
        tree.insert(se1_id, &arena);

        assert_eq!(
            arena[arena[tree.min(&arena).unwrap()].other_event.unwrap()].point,
            [1.0, 1.0]
        );
        assert_eq!(
            arena[arena[tree.max(&arena).unwrap()].other_event.unwrap()].point,
            [2.0, 3.0]
        );
    }

    #[test]
    fn compare_segments_not_collinear_events_order_in_sweep_line() {
        let mut arena = SweepEventArena::new();
        let (se1, _) = SweepEvent::new(
            [0.0, 1.0],
            true,
            Some(SweepEvent::with_point_and_left([2.0, 1.0], false, &mut arena).1),
            false,
            None,
            &mut arena,
        );
        let (se2, _) = SweepEvent::new(
            [-1.0, 0.0],
            true,
            Some(SweepEvent::with_point_and_left([2.0, 3.0], false, &mut arena).1),
            false,
            None,
            &mut arena,
        );

        let (se3, _) = SweepEvent::new(
            [0.0, 1.0],
            true,
            Some(SweepEvent::with_point_and_left([3.0, 4.0], false, &mut arena).1),
            false,
            None,
            &mut arena,
        );
        let (se4, _) = SweepEvent::new(
            [-1.0, 0.0],
            true,
            Some(SweepEvent::with_point_and_left([3.0, 1.0], false, &mut arena).1),
            false,
            None,
            &mut arena,
        );

        assert_eq!(compare_events(&se1, &se2, &arena), Ordering::Greater);
        assert_eq!(se2.is_below(&se1.point, &arena), false);
        assert_eq!(se2.is_above(&se1.point, &arena), true);

        assert_eq!(
            compare_segments(&se1, &se2, &arena),
            Ordering::Less,
            "compare segments",
        );
        assert_eq!(
            compare_segments(&se2, &se1, &arena),
            Ordering::Greater,
            "compare segments inverted",
        );

        assert_eq!(compare_events(&se3, &se4, &arena), Ordering::Greater,);
        assert_eq!(se4.is_above(&se3.point, &arena), false);
    }

    #[test]
    fn compare_segments_not_collinear_first_point_is_below() {
        let mut arena = SweepEventArena::new();
        let (se2, _) = SweepEvent::new(
            [0.0, 1.0],
            true,
            Some(SweepEvent::with_point_and_left([2.0, 1.0], false, &mut arena).1),
            false,
            None,
            &mut arena,
        );
        let (se1, _) = SweepEvent::new(
            [-1.0, 0.0],
            true,
            Some(SweepEvent::with_point_and_left([2.0, 3.0], false, &mut arena).1),
            false,
            None,
            &mut arena,
        );

        assert_eq!(se1.is_below(&se2.point, &arena), false);
        assert_eq!(
            compare_segments(&se1, &se2, &arena),
            Ordering::Greater,
            "compare segments",
        );
    }

    #[test]
    fn compare_segments_collinear_segments() {
        let mut arena = SweepEventArena::new();
        let (se1, _) = SweepEvent::new(
            [1.0, 1.0],
            true,
            Some(SweepEvent::with_point_and_left([5.0, 1.0], false, &mut arena).1),
            true,
            None,
            &mut arena,
        );
        let (se2, _) = SweepEvent::new(
            [2.0, 1.0],
            true,
            Some(SweepEvent::with_point_and_left([3.0, 1.0], false, &mut arena).1),
            false,
            None,
            &mut arena,
        );

        assert_ne!(se1.is_subject, se2.is_subject);
        assert_eq!(compare_segments(&se1, &se2, &arena), Ordering::Less,);
    }

    #[test]
    fn compare_segments_collinear_shared_left_point() {
        let mut arena = SweepEventArena::new();
        let pt = [0.0, 1.0];

        let (mut se1, _) = SweepEvent::new(
            pt,
            true,
            Some(SweepEvent::with_point_and_left([5.0, 1.0], false, &mut arena).1),
            false,
            None,
            &mut arena,
        );
        let (mut se2, _) = SweepEvent::new(
            pt,
            true,
            Some(SweepEvent::with_point_and_left([3.0, 1.0], false, &mut arena).1),
            false,
            None,
            &mut arena,
        );

        se1.contour_id = 1;
        se2.contour_id = 2;

        assert_eq!(&se1.is_subject, &se2.is_subject);
        assert_eq!(&se1.point, &se2.point);

        assert_eq!(compare_segments(&se1, &se2, &arena), Ordering::Less,);

        se1.contour_id = 2;
        se2.contour_id = 1;

        assert_eq!(compare_segments(&se1, &se2, &arena), Ordering::Greater,);
    }

    #[test]
    fn compare_segments_collinear_same_polygon_different_left_points() {
        let mut arena = SweepEventArena::new();
        let (se1, _) = SweepEvent::new(
            [1.0, 1.0],
            true,
            Some(SweepEvent::with_point_and_left([5.0, 1.0], false, &mut arena).1),
            true,
            None,
            &mut arena,
        );
        let (se2, _) = SweepEvent::new(
            [2.0, 1.0],
            true,
            Some(SweepEvent::with_point_and_left([3.0, 1.0], false, &mut arena).1),
            true,
            None,
            &mut arena,
        );

        assert_eq!(&se1.is_subject, &se2.is_subject);
        assert_ne!(&se1.point, &se2.point);
        assert_eq!(compare_segments(&se1, &se2, &arena), Ordering::Less,);
        assert_eq!(compare_segments(&se2, &se1, &arena), Ordering::Greater,);
    }
}
