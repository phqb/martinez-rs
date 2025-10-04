use core::cmp::Ordering;

use crate::{signed_area::signed_area, sweep_event::SweepEvent};

pub(crate) fn compare_events(e1: &SweepEvent, e2: &SweepEvent) -> Ordering {
    let p1 = e1.point;
    let p2 = e2.point;

    // Different x-coordinate
    if p1[0] > p2[0] {
        return Ordering::Greater;
    }
    if p1[0] < p2[0] {
        return Ordering::Less;
    }

    // Different points, but same x-coordinate
    // Event with lower y-coordinate is processed first
    if p1[1] != p2[1] {
        if p1[1] > p2[1] {
            return Ordering::Greater;
        } else {
            return Ordering::Less;
        }
    }

    special_cases(e1, e2)
}

fn special_cases(e1: &SweepEvent, e2: &SweepEvent) -> Ordering {
    let p1 = e1.point;

    // Same coordinates, but one is a left endpoint and the other is
    // a right endpoint. The right endpoint is processed first
    if e1.left != e2.left {
        if e1.left {
            return Ordering::Greater;
        } else {
            return Ordering::Less;
        }
    }

    // Same coordinates, both events
    // are left endpoints or right endpoints.
    // not collinear
    if signed_area(
        &p1,
        &e1.other_event.as_ref().unwrap().borrow().point,
        &e2.other_event.as_ref().unwrap().borrow().point,
    ) != 0.0
    {
        // the event associate to the bottom segment is processed first
        if !e1.is_below(&e2.other_event.as_ref().unwrap().borrow().point) {
            return Ordering::Greater;
        } else {
            return Ordering::Less;
        }
    }

    if !e1.is_subject && e2.is_subject {
        Ordering::Greater
    } else {
        Ordering::Less
    }
}

#[cfg(test)]
mod tests {
    use core::cmp::{Ordering, Reverse};

    use crate::{
        compare_events::compare_events,
        min_heap::MinHeap,
        sweep_event::{SweepEvent, SweepEventDeepEqual, SweepEventOrderedByCompareEvent},
    };

    #[test]
    fn queue_should_process_least_by_x_sweep_event_first() {
        let mut queue: MinHeap<SweepEventOrderedByCompareEvent> = MinHeap::new();
        let e1 = SweepEvent::with_point([0.0, 0.0]);
        let e2 = SweepEvent::with_point([0.5, 0.5]);

        queue.push(Reverse(SweepEventOrderedByCompareEvent(e1.clone())));
        queue.push(Reverse(SweepEventOrderedByCompareEvent(e2.clone())));

        assert_eq!(
            Some(SweepEventDeepEqual(e1)),
            queue.pop().map(|e| SweepEventDeepEqual(e.0.0.clone()))
        );
        assert_eq!(
            Some(SweepEventDeepEqual(e2)),
            queue.pop().map(|e| SweepEventDeepEqual(e.0.0.clone()))
        );
    }

    #[test]
    fn queue_should_process_least_by_y_sweep_event_first() {
        let mut queue: MinHeap<SweepEventOrderedByCompareEvent> = MinHeap::new();
        let e1 = SweepEvent::with_point([0.0, 0.0]);
        let e2 = SweepEvent::with_point([0.0, 0.5]);

        queue.push(Reverse(SweepEventOrderedByCompareEvent(e1.clone())));
        queue.push(Reverse(SweepEventOrderedByCompareEvent(e2.clone())));

        assert_eq!(
            Some(SweepEventDeepEqual(e1)),
            queue.pop().map(|e| SweepEventDeepEqual(e.0.0.clone()))
        );
        assert_eq!(
            Some(SweepEventDeepEqual(e2)),
            queue.pop().map(|e| SweepEventDeepEqual(e.0.0.clone()))
        );
    }

    #[test]
    fn queue_should_pop_least_by_left_prop_sweep_event_first() {
        let mut queue: MinHeap<SweepEventOrderedByCompareEvent> = MinHeap::new();
        let e1 = SweepEvent::with_point_and_left([0.0, 0.0], true);
        let e2 = SweepEvent::with_point_and_left([0.0, 0.0], false);

        queue.push(Reverse(SweepEventOrderedByCompareEvent(e1.clone())));
        queue.push(Reverse(SweepEventOrderedByCompareEvent(e2.clone())));

        assert_eq!(
            Some(SweepEventDeepEqual(e2)),
            queue.pop().map(|e| SweepEventDeepEqual(e.0.0.clone()))
        );
        assert_eq!(
            Some(SweepEventDeepEqual(e1)),
            queue.pop().map(|e| SweepEventDeepEqual(e.0.0.clone()))
        );
    }

    #[test]
    fn sweep_event_comparision_x_coordinates() {
        let e1 = SweepEvent::with_point([0.0, 0.0]);
        let e2 = SweepEvent::with_point([0.5, 0.5]);

        assert_eq!(compare_events(&e1.borrow(), &e2.borrow()), Ordering::Less);
        assert_eq!(
            compare_events(&e2.borrow(), &e1.borrow()),
            Ordering::Greater
        );
    }

    #[test]
    fn sweep_event_comparision_y_coordinates() {
        let e1 = SweepEvent::with_point([0.0, 0.0]);
        let e2 = SweepEvent::with_point([0.0, 0.5]);

        assert_eq!(compare_events(&e1.borrow(), &e2.borrow()), Ordering::Less);
        assert_eq!(
            compare_events(&e2.borrow(), &e1.borrow()),
            Ordering::Greater
        );
    }

    #[test]
    fn sweep_event_comparision_not_left_first() {
        let e1 = SweepEvent::with_point_and_left([0.0, 0.0], true);
        let e2 = SweepEvent::with_point_and_left([0.0, 0.0], false);

        assert_eq!(
            compare_events(&e1.borrow(), &e2.borrow()),
            Ordering::Greater
        );
        assert_eq!(compare_events(&e2.borrow(), &e1.borrow()), Ordering::Less);
    }

    #[test]
    fn sweep_event_comparison_shared_start_point_not_collinear_edges() {
        let e1 = SweepEvent::new(
            [0.0, 0.0],
            true,
            Some(SweepEvent::with_point_and_left([1.0, 1.0], false)),
            false,
            None,
        );
        let e2 = SweepEvent::new(
            [0.0, 0.0],
            true,
            Some(SweepEvent::with_point_and_left([2.0, 3.0], false)),
            false,
            None,
        );

        assert_eq!(
            compare_events(&e1.borrow(), &e2.borrow()),
            Ordering::Less,
            "lower is processed first"
        );
        assert_eq!(
            compare_events(&e2.borrow(), &e1.borrow()),
            Ordering::Greater,
            "higher is processed second"
        );
    }

    #[test]
    fn sweep_event_comparison_collinear_edges() {
        let e1 = SweepEvent::new(
            [0.0, 0.0],
            true,
            Some(SweepEvent::with_point_and_left([1.0, 1.0], false)),
            true,
            None,
        );
        let e2 = SweepEvent::new(
            [0.0, 0.0],
            true,
            Some(SweepEvent::with_point_and_left([2.0, 2.0], false)),
            false,
            None,
        );

        assert_eq!(
            compare_events(&e1.borrow(), &e2.borrow()),
            Ordering::Less,
            "clipping is processed first"
        );
        assert_eq!(
            compare_events(&e2.borrow(), &e1.borrow()),
            Ordering::Greater,
            "subject is processed second"
        );
    }
}
