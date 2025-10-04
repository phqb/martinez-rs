use core::{cell::RefCell, cmp::Reverse};
use std::rc::Rc;

use crate::{
    compute_fields::compute_fields,
    min_heap::MinHeap,
    operation::Operation,
    possible_intersection::possible_intersection,
    sweep_event::{SweepEvent, SweepEventOrderedByCompareEvent, SweepEventOrderedByCompareSegment}, tree::SweepEventTree,
};

pub(crate) fn subdivide(
    event_queue: &mut MinHeap<SweepEventOrderedByCompareEvent>,
    sbbox: &[f64; 4],
    cbbox: &[f64; 4],
    operation: Operation,
) -> Vec<Rc<RefCell<SweepEvent>>> {
    let mut sweep_line = SweepEventTree::new();
    let mut sorted_events: Vec<Rc<RefCell<SweepEvent>>> = vec![];

    let right_bound = sbbox[2].min(cbbox[2]);

    while let Some(Reverse(SweepEventOrderedByCompareEvent(event))) = event_queue.pop() {
        sorted_events.push(event.clone());

        // optimization by bboxes for intersection and difference goes here
        if (operation == Operation::Intersection && event.borrow().point[0] > right_bound)
            || (operation == Operation::Difference && event.borrow().point[0] > sbbox[2])
        {
            break;
        }

        if event.borrow().left {
            let event_index = sweep_line.insert(SweepEventOrderedByCompareSegment(event.clone()));

            let prev = if event_index > 0 {
                Some(sweep_line[event_index - 1].0.clone())
            } else {
                None
            };

            let next = if event_index + 1 < sweep_line.len() {
                Some(sweep_line[event_index + 1].0.clone())
            } else {
                None
            };

            compute_fields(event.clone(), prev.clone(), operation);

            if let Some(next) = next {
                if possible_intersection(event.clone(), next.clone(), event_queue) == 2 {
                    compute_fields(event.clone(), prev.clone(), operation);
                    compute_fields(next.clone(), Some(event.clone()), operation);
                }
            }

            if let Some(prev) = prev {
                if possible_intersection(prev.clone(), event.clone(), event_queue) == 2 {
                    let prevprev = if event_index > 1 {
                        Some(sweep_line[event_index - 2].0.clone())
                    } else {
                        None
                    };
                    compute_fields(prev.clone(), prevprev.clone(), operation);
                    compute_fields(event.clone(), Some(prev.clone()), operation);
                }
            }
        } else {
            let event = event.borrow().other_event.as_ref().unwrap().clone();
            let event_index = sweep_line
                .iter()
                .position(|x| x == &SweepEventOrderedByCompareSegment(event.clone()));
            if let Some(event_index) = event_index {
                let prev = if event_index > 0 {
                    Some(sweep_line[event_index - 1].0.clone())
                } else {
                    None
                };

                let next = if event_index + 1 < sweep_line.len() {
                    Some(sweep_line[event_index + 1].0.clone())
                } else {
                    None
                };

                sweep_line.remove(event_index);

                if let (Some(prev), Some(next)) = (prev, next) {
                    possible_intersection(prev.clone(), next.clone(), event_queue);
                }
            }
        }
    }

    sorted_events
}
