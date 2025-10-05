use core::cmp::Ordering;

use crate::{
    compare_events::compare_events,
    divide_segment::divide_segment,
    edge_type::EdgeType,
    equals::equals,
    min_heap::MinHeapByCompareEvents,
    segment_intersection::intersection,
    sweep_event::{SweepEventArena, SweepEventId},
};

pub(crate) fn possible_intersection(
    se1_id: SweepEventId,
    se2_id: SweepEventId,
    queue: &mut MinHeapByCompareEvents,
    arena: &mut SweepEventArena,
) -> usize {
    let se1 = arena[se1_id].clone();
    let se2 = arena[se2_id].clone();

    let inter = intersection(
        &se1.point,
        &arena[se1.other_event.unwrap()].point,
        &se2.point,
        &arena[se2.other_event.unwrap()].point,
        false,
    );

    let inter = if let Some(inter) = inter {
        if inter.is_empty() {
            return 0;
        } else {
            inter
        }
    } else {
        return 0;
    };

    let n_intersections = inter.len();

    // the line segments intersect at an endpoint of both line segments
    if (n_intersections == 1)
        && (equals(&se1.point, &se2.point)
            || equals(
                &arena[se1.other_event.unwrap()].point,
                &arena[se2.other_event.unwrap()].point,
            ))
    {
        return 0;
    }

    if n_intersections == 2 && se1.is_subject == se2.is_subject {
        return 0;
    }

    // The line segments associated to se1 and se2 intersect
    if n_intersections == 1 {
        // if the intersection point is not an endpoint of se1
        if !equals(&se1.point, &inter[0])
            && !equals(&arena[se1.other_event.unwrap()].point, &inter[0])
        {
            divide_segment(se1.id, inter[0], queue, arena);
        }

        // if the intersection point is not an endpoint of se2
        if !equals(&se2.point, &inter[0])
            && !equals(&arena[se2.other_event.unwrap()].point, &inter[0])
        {
            divide_segment(se2.id, inter[0], queue, arena);
        }
        return 1;
    }

    // The line segments associated to se1 and se2 overlap
    let mut events: Vec<SweepEventId> = vec![];
    let mut left_coincide = false;
    let mut right_coincide = false;

    if equals(&se1.point, &se2.point) {
        left_coincide = true; // linked
    } else if compare_events(&se1, &se2, arena) == Ordering::Greater {
        events.push(se2.id);
        events.push(se1.id);
    } else {
        events.push(se1.id);
        events.push(se2.id);
    }

    if equals(
        &arena[se1.other_event.unwrap()].point,
        &arena[se2.other_event.unwrap()].point,
    ) {
        right_coincide = true;
    } else if compare_events(
        &arena[se1.other_event.unwrap()],
        &arena[se2.other_event.unwrap()],
        arena,
    ) == Ordering::Greater
    {
        events.push(se2.other_event.unwrap());
        events.push(se1.other_event.unwrap());
    } else {
        events.push(se1.other_event.unwrap());
        events.push(se2.other_event.unwrap());
    }

    #[allow(clippy::overly_complex_bool_expr)]
    if (left_coincide && right_coincide) || left_coincide {
        // both line segments are equal or share the left endpoint
        arena[se2_id].edge_type = EdgeType::NonContributing;
        arena[se1_id].edge_type = if se2.in_out == se1.in_out {
            EdgeType::SameTransition
        } else {
            EdgeType::DifferentTransition
        };

        if left_coincide && !right_coincide {
            // honestly no idea, but changing events selection from [2, 1]
            // to [0, 1] fixes the overlapping self-intersecting polygons issue
            divide_segment(
                arena[events[1]].other_event.unwrap(),
                arena[events[0]].point,
                queue,
                arena,
            );
        }
        return 2;
    }

    // the line segments share the right endpoint
    if right_coincide {
        divide_segment(events[0], arena[events[1]].point, queue, arena);
        return 3;
    }

    // no line segment includes totally the other one
    if events[0] != arena[events[3]].other_event.unwrap() {
        divide_segment(events[0], arena[events[1]].point, queue, arena);
        divide_segment(events[1], arena[events[2]].point, queue, arena);
        return 3;
    }

    // one line segment includes the other one
    divide_segment(events[0], arena[events[1]].point, queue, arena);
    divide_segment(
        arena[events[3]].other_event.unwrap(),
        arena[events[2]].point,
        queue,
        arena,
    );

    3
}
