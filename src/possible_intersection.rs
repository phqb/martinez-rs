use core::{cell::RefCell, cmp::Ordering};
use std::rc::Rc;

use crate::{
    compare_events::compare_events,
    divide_segment::divide_segment,
    edge_type::EdgeType,
    equals::equals,
    min_heap::MinHeap,
    segment_intersection::intersection,
    sweep_event::{SweepEvent, SweepEventOrderedByCompareEvent},
};

pub(crate) fn possible_intersection(
    se1: Rc<RefCell<SweepEvent>>,
    se2: Rc<RefCell<SweepEvent>>,
    queue: &mut MinHeap<SweepEventOrderedByCompareEvent>,
) -> usize {
    let inter = intersection(
        &se1.borrow().point,
        &se1.borrow().other_event.as_ref().unwrap().borrow().point,
        &se2.borrow().point,
        &se2.borrow().other_event.as_ref().unwrap().borrow().point,
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
        && (equals(&se1.borrow().point, &se2.borrow().point)
            || equals(
                &se1.borrow().other_event.as_ref().unwrap().borrow().point,
                &se2.borrow().other_event.as_ref().unwrap().borrow().point,
            ))
    {
        return 0;
    }

    if n_intersections == 2 && se1.borrow().is_subject == se2.borrow().is_subject {
        return 0;
    }

    // The line segments associated to se1 and se2 intersect
    if n_intersections == 1 {
        // if the intersection point is not an endpoint of se1
        if !equals(&se1.borrow().point, &inter[0])
            && !equals(
                &se1.borrow().other_event.as_ref().unwrap().borrow().point,
                &inter[0],
            )
        {
            divide_segment(se1, &inter[0], queue);
        }

        // if the intersection point is not an endpoint of se2
        if !equals(&se2.borrow().point, &inter[0])
            && !equals(
                &se2.borrow().other_event.as_ref().unwrap().borrow().point,
                &inter[0],
            )
        {
            divide_segment(se2, &inter[0], queue);
        }
        return 1;
    }

    // The line segments associated to se1 and se2 overlap
    let mut events: Vec<Rc<RefCell<SweepEvent>>> = vec![];
    let mut left_coincide = false;
    let mut right_coincide = false;

    if equals(&se1.borrow().point, &se2.borrow().point) {
        left_coincide = true; // linked
    } else if compare_events(&se1.borrow(), &se2.borrow()) == Ordering::Greater {
        events.extend_from_slice(&[se2.clone(), se1.clone()]);
    } else {
        events.extend_from_slice(&[se1.clone(), se2.clone()]);
    }

    if equals(
        &se1.borrow().other_event.as_ref().unwrap().borrow().point,
        &se2.borrow().other_event.as_ref().unwrap().borrow().point,
    ) {
        right_coincide = true;
    } else if compare_events(
        &se1.borrow().other_event.as_ref().unwrap().borrow(),
        &se2.borrow().other_event.as_ref().unwrap().borrow(),
    ) == Ordering::Greater
    {
        events.extend_from_slice(&[
            se2.borrow().other_event.as_ref().unwrap().clone(),
            se1.borrow().other_event.as_ref().unwrap().clone(),
        ]);
    } else {
        events.extend_from_slice(&[
            se1.borrow().other_event.as_ref().unwrap().clone(),
            se2.borrow().other_event.as_ref().unwrap().clone(),
        ]);
    }

    #[allow(clippy::overly_complex_bool_expr)]
    if (left_coincide && right_coincide) || left_coincide {
        // both line segments are equal or share the left endpoint
        se2.borrow_mut().edge_type = EdgeType::NonContributing;
        se1.borrow_mut().edge_type = if se2.borrow().in_out == se1.borrow().in_out {
            EdgeType::SameTransition
        } else {
            EdgeType::DifferentTransition
        };

        if left_coincide && !right_coincide {
            // honestly no idea, but changing events selection from [2, 1]
            // to [0, 1] fixes the overlapping self-intersecting polygons issue
            divide_segment(
                events[1].borrow().other_event.as_ref().unwrap().clone(),
                &events[0].borrow().point,
                queue,
            );
        }
        return 2;
    }

    // the line segments share the right endpoint
    if right_coincide {
        divide_segment(events[0].clone(), &events[1].borrow().point, queue);
        return 3;
    }

    // no line segment includes totally the other one
    if !std::ptr::eq(
        &events[0].borrow(),
        &events[3].borrow().other_event.as_ref().unwrap().borrow(),
    ) {
        divide_segment(events[0].clone(), &events[1].borrow().point, queue);
        divide_segment(events[1].clone(), &events[2].borrow().point, queue);
        return 3;
    }

    // one line segment includes the other one
    divide_segment(events[0].clone(), &events[1].borrow().point, queue);
    divide_segment(
        events[3].borrow().other_event.as_ref().unwrap().clone(),
        &events[2].borrow().point,
        queue,
    );

    3
}
