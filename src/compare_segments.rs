use core::cmp::Ordering;

use crate::{
    compare_events::compare_events, equals::equals, signed_area::signed_area,
    sweep_event::SweepEvent,
};

pub(crate) fn compare_segments(le1: &SweepEvent, le2: &SweepEvent) -> Ordering {
    if std::ptr::eq(le1, le2) {
        return Ordering::Equal;
    }

    // Segments are not collinear
    if signed_area(
        &le1.point,
        &le1.other_event.as_ref().unwrap().borrow().point,
        &le2.point,
    ) != 0.0
        || signed_area(
            &le1.point,
            &le1.other_event.as_ref().unwrap().borrow().point,
            &le2.other_event.as_ref().unwrap().borrow().point,
        ) != 0.0
    {
        // If they share their left endpoint use the right endpoint to sort
        if equals(&le1.point, &le2.point) {
            return if le1.is_below(&le2.other_event.as_ref().unwrap().borrow().point) {
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
        if compare_events(le1, le2) == Ordering::Greater {
            return if le2.is_above(&le1.point) {
                Ordering::Less
            } else {
                Ordering::Greater
            };
        }

        // The line segment associated to e2 has been inserted
        // into S after the line segment associated to e1
        return if le1.is_below(&le2.point) {
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
            let p1 = le1.other_event.as_ref().unwrap().borrow().point;
            let p2 = le2.other_event.as_ref().unwrap().borrow().point;
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

    if compare_events(le1, le2) == Ordering::Greater {
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
        sweep_event::{SweepEvent, SweepEventOrderedByCompareSegment},
        tree::SweepEventTree,
    };
    use core::cmp::Ordering;

    #[test]
    fn compare_segments_not_collinear_shared_left_point_right_point_first() {
        let mut tree = SweepEventTree::new();
        let pt = [0.0, 0.0];
        let se1 = SweepEvent::new(
            pt,
            true,
            Some(SweepEvent::with_point_and_left([1.0, 1.0], false)),
            false,
            None,
        );
        let se2 = SweepEvent::new(
            pt,
            true,
            Some(SweepEvent::with_point_and_left([2.0, 3.0], false)),
            false,
            None,
        );

        tree.insert(SweepEventOrderedByCompareSegment(se1.clone()));
        tree.insert(SweepEventOrderedByCompareSegment(se2.clone()));

        assert_eq!(
            tree.max()
                .unwrap()
                .0
                .borrow()
                .other_event
                .as_ref()
                .unwrap()
                .borrow()
                .point,
            [2.0, 3.0]
        );
        assert_eq!(
            tree.min()
                .unwrap()
                .0
                .borrow()
                .other_event
                .as_ref()
                .unwrap()
                .borrow()
                .point,
            [1.0, 1.0]
        );
    }

    #[test]
    fn compare_segments_not_collinear_different_left_point_right_point_y_coord_to_sort() {
        let mut tree = SweepEventTree::new();
        let se1 = SweepEvent::new(
            [0.0, 1.0],
            true,
            Some(SweepEvent::with_point_and_left([1.0, 1.0], false)),
            false,
            None,
        );
        let se2 = SweepEvent::new(
            [0.0, 2.0],
            true,
            Some(SweepEvent::with_point_and_left([2.0, 3.0], false)),
            false,
            None,
        );

        tree.insert(SweepEventOrderedByCompareSegment(se1.clone()));
        tree.insert(SweepEventOrderedByCompareSegment(se2.clone()));

        assert_eq!(
            tree.min()
                .unwrap()
                .0
                .borrow()
                .other_event
                .as_ref()
                .unwrap()
                .borrow()
                .point,
            [1.0, 1.0]
        );
        assert_eq!(
            tree.max()
                .unwrap()
                .0
                .borrow()
                .other_event
                .as_ref()
                .unwrap()
                .borrow()
                .point,
            [2.0, 3.0]
        );
    }

    #[test]
    fn compare_segments_not_collinear_events_order_in_sweep_line() {
        let se1 = SweepEvent::new(
            [0.0, 1.0],
            true,
            Some(SweepEvent::with_point_and_left([2.0, 1.0], false)),
            false,
            None,
        );
        let se2 = SweepEvent::new(
            [-1.0, 0.0],
            true,
            Some(SweepEvent::with_point_and_left([2.0, 3.0], false)),
            false,
            None,
        );

        let se3 = SweepEvent::new(
            [0.0, 1.0],
            true,
            Some(SweepEvent::with_point_and_left([3.0, 4.0], false)),
            false,
            None,
        );
        let se4 = SweepEvent::new(
            [-1.0, 0.0],
            true,
            Some(SweepEvent::with_point_and_left([3.0, 1.0], false)),
            false,
            None,
        );

        assert_eq!(
            compare_events(&se1.borrow(), &se2.borrow()),
            Ordering::Greater
        );
        assert_eq!(se2.borrow().is_below(&se1.borrow().point), false);
        assert_eq!(se2.borrow().is_above(&se1.borrow().point), true);

        assert_eq!(
            compare_segments(&se1.borrow(), &se2.borrow()),
            Ordering::Less,
            "compare segments",
        );
        assert_eq!(
            compare_segments(&se2.borrow(), &se1.borrow()),
            Ordering::Greater,
            "compare segments inverted",
        );

        assert_eq!(
            compare_events(&se3.borrow(), &se4.borrow()),
            Ordering::Greater,
        );
        assert_eq!(se4.borrow().is_above(&se3.borrow().point), false);
    }

    #[test]
    fn compare_segments_not_collinear_first_point_is_below() {
        let se2 = SweepEvent::new(
            [0.0, 1.0],
            true,
            Some(SweepEvent::with_point_and_left([2.0, 1.0], false)),
            false,
            None,
        );
        let se1 = SweepEvent::new(
            [-1.0, 0.0],
            true,
            Some(SweepEvent::with_point_and_left([2.0, 3.0], false)),
            false,
            None,
        );

        assert_eq!(se1.borrow().is_below(&se2.borrow().point), false);
        assert_eq!(
            compare_segments(&se1.borrow(), &se2.borrow()),
            Ordering::Greater,
            "compare segments",
        );
    }

    #[test]
    fn compare_segments_collinear_segments() {
        let se1 = SweepEvent::new(
            [1.0, 1.0],
            true,
            Some(SweepEvent::with_point_and_left([5.0, 1.0], false)),
            true,
            None,
        );
        let se2 = SweepEvent::new(
            [2.0, 1.0],
            true,
            Some(SweepEvent::with_point_and_left([3.0, 1.0], false)),
            false,
            None,
        );

        assert_ne!(se1.borrow().is_subject, se2.borrow().is_subject);
        assert_eq!(
            compare_segments(&se1.borrow(), &se2.borrow()),
            Ordering::Less,
        );
    }

    #[test]
    fn compare_segments_collinear_shared_left_point() {
        let pt = [0.0, 1.0];

        let se1 = SweepEvent::new(
            pt,
            true,
            Some(SweepEvent::with_point_and_left([5.0, 1.0], false)),
            false,
            None,
        );
        let se2 = SweepEvent::new(
            pt,
            true,
            Some(SweepEvent::with_point_and_left([3.0, 1.0], false)),
            false,
            None,
        );

        se1.borrow_mut().contour_id = 1;
        se2.borrow_mut().contour_id = 2;

        assert_eq!(&se1.borrow().is_subject, &se2.borrow().is_subject);
        assert_eq!(&se1.borrow().point, &se2.borrow().point);

        assert_eq!(
            compare_segments(&se1.borrow(), &se2.borrow()),
            Ordering::Less,
        );

        se1.borrow_mut().contour_id = 2;
        se2.borrow_mut().contour_id = 1;

        assert_eq!(
            compare_segments(&se1.borrow(), &se2.borrow()),
            Ordering::Greater,
        );
    }

    #[test]
    fn compare_segments_collinear_same_polygon_different_left_points() {
        let se1 = SweepEvent::new(
            [1.0, 1.0],
            true,
            Some(SweepEvent::with_point_and_left([5.0, 1.0], false)),
            true,
            None,
        );
        let se2 = SweepEvent::new(
            [2.0, 1.0],
            true,
            Some(SweepEvent::with_point_and_left([3.0, 1.0], false)),
            true,
            None,
        );

        assert_eq!(&se1.borrow().is_subject, &se2.borrow().is_subject);
        assert_ne!(&se1.borrow().point, &se2.borrow().point);
        assert_eq!(
            compare_segments(&se1.borrow(), &se2.borrow()),
            Ordering::Less,
        );
        assert_eq!(
            compare_segments(&se2.borrow(), &se1.borrow()),
            Ordering::Greater,
        );
    }
}
