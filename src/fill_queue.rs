use core::cmp::Ordering;

use crate::{
    compare_events::compare_events,
    min_heap::MinHeapByCompareEvents,
    operation::Operation,
    sweep_event::{SweepEvent, SweepEventArena},
};

fn process_polygon(
    contour_or_hole: &[[f64; 2]],
    is_subject: bool,
    depth: i64,
    is_exterior_ring: bool,
    q: &mut MinHeapByCompareEvents,
    bbox: &mut [f64; 4],
    arena: &mut SweepEventArena,
) {
    if contour_or_hole.is_empty() {
        return;
    }

    for i in 0..contour_or_hole.len() - 1 {
        let s1 = contour_or_hole[i];
        let s2 = contour_or_hole[i + 1];

        if s1[0] == s2[0] && s1[1] == s2[1] {
            continue; // skip collapsed edges, or it breaks
        }

        let e1_id = SweepEvent::new(s1, false, None, is_subject, None, arena);
        let e2_id = SweepEvent::new(s2, false, Some(e1_id), is_subject, None, arena);
        arena[e1_id].other_event = Some(e2_id);

        arena[e1_id].contour_id = depth;
        arena[e2_id].contour_id = depth;
        if !is_exterior_ring {
            arena[e1_id].is_exterior_ring = false;
            arena[e2_id].is_exterior_ring = false;
        }

        if compare_events(&arena[e1_id], &arena[e2_id], arena) > Ordering::Equal {
            arena[e2_id].left = true;
        } else {
            arena[e1_id].left = true;
        }

        let x = s1[0];
        let y = s1[1];
        bbox[0] = bbox[0].min(x);
        bbox[1] = bbox[1].min(y);
        bbox[2] = bbox[2].max(x);
        bbox[3] = bbox[3].max(y);

        // Pushing it so the queue is sorted from left to right,
        // with object on the left having the highest priority.
        q.push(e1_id, arena);
        q.push(e2_id, arena);
    }
}

#[cfg(any(test, not(feature = "test_reuse")))]
pub(crate) fn fill_queue(
    subject: &[Vec<Vec<[f64; 2]>>],
    clipping: &[Vec<Vec<[f64; 2]>>],
    sbbox: &mut [f64; 4],
    cbbox: &mut [f64; 4],
    operation: Option<Operation>,
    arena: &mut SweepEventArena,
) -> MinHeapByCompareEvents {
    let mut event_queue = MinHeapByCompareEvents::new();

    fill_queue_reuse(
        subject,
        clipping,
        sbbox,
        cbbox,
        operation,
        arena,
        &mut event_queue,
    );

    event_queue
}

pub(crate) fn fill_queue_reuse(
    subject: &[Vec<Vec<[f64; 2]>>],
    clipping: &[Vec<Vec<[f64; 2]>>],
    sbbox: &mut [f64; 4],
    cbbox: &mut [f64; 4],
    operation: Option<Operation>,
    arena: &mut SweepEventArena,
    event_queue: &mut MinHeapByCompareEvents,
) {
    event_queue.clear();
    let mut initial_contour_id = 0i64;

    for polygon_set in subject {
        for (j, polygon) in polygon_set.iter().enumerate() {
            let is_exterior_ring = j == 0;
            if is_exterior_ring {
                initial_contour_id += 1;
            }
            process_polygon(
                polygon,
                true,
                initial_contour_id,
                is_exterior_ring,
                event_queue,
                sbbox,
                arena,
            );
        }
    }

    for polygon_set in clipping {
        for (j, polygon) in polygon_set.iter().enumerate() {
            let mut is_exterior_ring = j == 0;
            if operation == Some(Operation::Difference) {
                is_exterior_ring = false;
            }
            if is_exterior_ring {
                initial_contour_id += 1;
            }
            process_polygon(
                polygon,
                false,
                initial_contour_id,
                is_exterior_ring,
                event_queue,
                cbbox,
                arena,
            );
        }
    }
}
