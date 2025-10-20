use crate::{
    compute_fields::compute_fields,
    min_heap::MinHeapByCompareEvents,
    operation::Operation,
    possible_intersection::possible_intersection,
    sweep_event::{SweepEventArena, SweepEventId},
    tree_by_compare_segments::TreeByCompareSegments,
};

#[cfg(any(test, not(feature = "test_reuse")))]
pub(crate) fn subdivide(
    event_queue: &mut MinHeapByCompareEvents,
    sbbox: &[f64; 4],
    cbbox: &[f64; 4],
    operation: Operation,
    arena: &mut SweepEventArena,
) -> Vec<SweepEventId> {
    let mut sweep_line = TreeByCompareSegments::new();
    let mut sorted_events: Vec<SweepEventId> = vec![];

    subdivide_reuse(
        event_queue,
        sbbox,
        cbbox,
        operation,
        arena,
        &mut sweep_line,
        &mut sorted_events,
    );

    sorted_events
}

pub(crate) fn subdivide_reuse(
    event_queue: &mut MinHeapByCompareEvents,
    sbbox: &[f64; 4],
    cbbox: &[f64; 4],
    operation: Operation,
    arena: &mut SweepEventArena,
    sweep_line: &mut TreeByCompareSegments,
    sorted_events: &mut Vec<SweepEventId>,
) {
    sweep_line.clear();
    sorted_events.clear();

    let right_bound = sbbox[2].min(cbbox[2]);

    while let Some(event) = event_queue.pop(arena) {
        sorted_events.push(event);

        // optimization by bboxes for intersection and difference goes here
        if (operation == Operation::Intersection && arena[event].point[0] > right_bound)
            || (operation == Operation::Difference && arena[event].point[0] > sbbox[2])
        {
            break;
        }

        if arena[event].left {
            let event_node = sweep_line.insert(event, arena);

            let prev = sweep_line.prev(&event_node);
            let prev_event = prev.as_ref().map(|node| node.value);
            let next = sweep_line.next(&event_node);
            let next_event = next.as_ref().map(|node| node.value);

            compute_fields(event, prev_event, operation, arena);

            if let Some(next_event) = next_event {
                if possible_intersection(event, next_event, event_queue, arena) == 2 {
                    compute_fields(event, prev_event, operation, arena);
                    compute_fields(next_event, Some(event), operation, arena);
                }
            }

            if let (Some(prev), Some(prev_event)) = (prev, prev_event) {
                if possible_intersection(prev_event, event, event_queue, arena) == 2 {
                    let prevprev_event = sweep_line.prev(&prev).map(|node| node.value);
                    compute_fields(prev_event, prevprev_event, operation, arena);
                    compute_fields(event, Some(prev_event), operation, arena);
                }
            }
        } else {
            let event = arena[event].other_event.unwrap();
            let event_node = sweep_line.find(event, arena);
            if let Some(event_node) = event_node {
                let prev = sweep_line.prev(&event_node).map(|node| node.value);
                let next = sweep_line.next(&event_node).map(|node| node.value);

                sweep_line.remove(event_node);

                if let (Some(prev), Some(next)) = (prev, next) {
                    possible_intersection(prev, next, event_queue, arena);
                }
            }
        }
    }
}
