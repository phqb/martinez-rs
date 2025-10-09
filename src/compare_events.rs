use core::cmp::Ordering;

use crate::{
    signed_area::signed_area,
    sweep_event::{SweepEvent, SweepEventArena},
};

pub(crate) fn compare_events(
    e1: &SweepEvent,
    e2: &SweepEvent,
    arena: &SweepEventArena,
) -> Ordering {
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

    special_cases(e1, e2, arena)
}

fn special_cases(e1: &SweepEvent, e2: &SweepEvent, arena: &SweepEventArena) -> Ordering {
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
        &arena[e1.other_event.expect("SweepEvent.other_event")].point,
        &arena[e2.other_event.expect("SweepEvent.other_event")].point,
    ) != 0.0
    {
        // the event associate to the bottom segment is processed first
        if !e1.is_below(
            &arena[e2.other_event.expect("SweepEvent.other_event")].point,
            arena,
        ) {
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
    use core::cmp::Ordering;

    use crate::{
        compare_events::compare_events,
        min_heap::MinHeapByCompareEvents,
        sweep_event::{SweepEvent, SweepEventArena, SweepEventDeepEqual},
    };

    #[test]
    fn queue_should_process_least_by_x_sweep_event_first() {
        let mut arena = SweepEventArena::new();
        let mut queue = MinHeapByCompareEvents::new();
        let e1_id = SweepEvent::with_point([0.0, 0.0], &mut arena);
        let e2_id = SweepEvent::with_point([0.5, 0.5], &mut arena);

        queue.push(e1_id, &arena);
        queue.push(e2_id, &arena);

        assert_eq!(
            Some(SweepEventDeepEqual(arena[e1_id].clone(), &arena)),
            queue
                .pop(&arena)
                .map(|e| SweepEventDeepEqual(arena[e].clone(), &arena))
        );
        assert_eq!(
            Some(SweepEventDeepEqual(arena[e2_id].clone(), &arena)),
            queue
                .pop(&arena)
                .map(|e| SweepEventDeepEqual(arena[e].clone(), &arena))
        );
    }

    #[test]
    fn queue_should_process_least_by_y_sweep_event_first() {
        let mut arena = SweepEventArena::new();
        let mut queue = MinHeapByCompareEvents::new();
        let e1_id = SweepEvent::with_point([0.0, 0.0], &mut arena);
        let e2_id = SweepEvent::with_point([0.0, 0.5], &mut arena);

        queue.push(e1_id, &arena);
        queue.push(e2_id, &arena);

        assert_eq!(
            Some(SweepEventDeepEqual(arena[e1_id].clone(), &arena)),
            queue
                .pop(&arena)
                .map(|e| SweepEventDeepEqual(arena[e].clone(), &arena))
        );
        assert_eq!(
            Some(SweepEventDeepEqual(arena[e2_id].clone(), &arena)),
            queue
                .pop(&arena)
                .map(|e| SweepEventDeepEqual(arena[e].clone(), &arena))
        );
    }

    #[test]
    fn queue_should_pop_least_by_left_prop_sweep_event_first() {
        let mut arena = SweepEventArena::new();
        let mut queue = MinHeapByCompareEvents::new();
        let e1_id = SweepEvent::with_point_and_left([0.0, 0.0], true, &mut arena);
        let e2_id = SweepEvent::with_point_and_left([0.0, 0.0], false, &mut arena);

        queue.push(e1_id, &arena);
        queue.push(e2_id, &arena);

        assert_eq!(
            Some(SweepEventDeepEqual(arena[e2_id].clone(), &arena)),
            queue
                .pop(&arena)
                .map(|e| SweepEventDeepEqual(arena[e].clone(), &arena))
        );
        assert_eq!(
            Some(SweepEventDeepEqual(arena[e1_id].clone(), &arena)),
            queue
                .pop(&arena)
                .map(|e| SweepEventDeepEqual(arena[e].clone(), &arena))
        );
    }

    #[test]
    fn sweep_event_comparision_x_coordinates() {
        let mut arena = SweepEventArena::new();
        let e1_id = SweepEvent::with_point([0.0, 0.0], &mut arena);
        let e2_id = SweepEvent::with_point([0.5, 0.5], &mut arena);

        assert_eq!(
            compare_events(&arena[e1_id], &arena[e2_id], &arena),
            Ordering::Less
        );
        assert_eq!(
            compare_events(&arena[e2_id], &arena[e1_id], &arena),
            Ordering::Greater
        );
    }

    #[test]
    fn sweep_event_comparision_y_coordinates() {
        let mut arena = SweepEventArena::new();
        let e1_id = SweepEvent::with_point([0.0, 0.0], &mut arena);
        let e2_id = SweepEvent::with_point([0.0, 0.5], &mut arena);

        assert_eq!(
            compare_events(&arena[e1_id], &arena[e2_id], &arena),
            Ordering::Less
        );
        assert_eq!(
            compare_events(&arena[e2_id], &arena[e1_id], &arena),
            Ordering::Greater
        );
    }

    #[test]
    fn sweep_event_comparision_not_left_first() {
        let mut arena = SweepEventArena::new();
        let e1_id = SweepEvent::with_point_and_left([0.0, 0.0], true, &mut arena);
        let e2_id = SweepEvent::with_point_and_left([0.0, 0.0], false, &mut arena);

        assert_eq!(
            compare_events(&arena[e1_id], &arena[e2_id], &arena),
            Ordering::Greater
        );
        assert_eq!(
            compare_events(&arena[e2_id], &arena[e1_id], &arena),
            Ordering::Less
        );
    }

    #[test]
    fn sweep_event_comparison_shared_start_point_not_collinear_edges() {
        let mut arena = SweepEventArena::new();
        let e1_id = SweepEvent::new(
            [0.0, 0.0],
            true,
            Some(SweepEvent::with_point_and_left(
                [1.0, 1.0],
                false,
                &mut arena,
            )),
            false,
            None,
            &mut arena,
        );
        let e2_id = SweepEvent::new(
            [0.0, 0.0],
            true,
            Some(SweepEvent::with_point_and_left(
                [2.0, 3.0],
                false,
                &mut arena,
            )),
            false,
            None,
            &mut arena,
        );

        assert_eq!(
            compare_events(&arena[e1_id], &arena[e2_id], &arena),
            Ordering::Less,
            "lower is processed first"
        );
        assert_eq!(
            compare_events(&arena[e2_id], &arena[e1_id], &arena),
            Ordering::Greater,
            "higher is processed second"
        );
    }

    #[test]
    fn sweep_event_comparison_collinear_edges() {
        let mut arena = SweepEventArena::new();
        let e1_id = SweepEvent::new(
            [0.0, 0.0],
            true,
            Some(SweepEvent::with_point_and_left(
                [1.0, 1.0],
                false,
                &mut arena,
            )),
            true,
            None,
            &mut arena,
        );
        let e2_id = SweepEvent::new(
            [0.0, 0.0],
            true,
            Some(SweepEvent::with_point_and_left(
                [2.0, 2.0],
                false,
                &mut arena,
            )),
            false,
            None,
            &mut arena,
        );

        assert_eq!(
            compare_events(&arena[e1_id], &arena[e2_id], &arena),
            Ordering::Less,
            "clipping is processed first"
        );
        assert_eq!(
            compare_events(&arena[e2_id], &arena[e1_id], &arena),
            Ordering::Greater,
            "subject is processed second"
        );
    }
}
