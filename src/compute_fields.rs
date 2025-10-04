use core::cell::RefCell;
use std::rc::Rc;

use crate::{
    edge_type::EdgeType,
    operation::Operation,
    sweep_event::{ResultTransitionType, SweepEvent},
};

pub(crate) fn compute_fields(
    event: Rc<RefCell<SweepEvent>>,
    prev: Option<Rc<RefCell<SweepEvent>>>,
    operation: Operation,
) {
    // compute inOut and otherInOut fields
    if prev.is_none() {
        event.borrow_mut().in_out = false;
        event.borrow_mut().other_in_out = true;

    // previous line segment in sweepline belongs to the same polygon
    } else {
        let prev = prev.unwrap();

        if event.borrow().is_subject == prev.borrow().is_subject {
            event.borrow_mut().in_out = !prev.borrow().in_out;
            event.borrow_mut().other_in_out = prev.borrow().other_in_out;

        // previous line segment in sweepline belongs to the clipping polygon
        } else {
            event.borrow_mut().in_out = !prev.borrow().other_in_out;
            event.borrow_mut().other_in_out = if prev.borrow().is_vertical() {
                !prev.borrow().in_out
            } else {
                prev.borrow().in_out
            };
        }

        // compute prevInResult field
        event.borrow_mut().prev_in_result =
            if !in_result(&prev.borrow(), operation) || prev.borrow().is_vertical() {
                prev.borrow().prev_in_result.clone()
            } else {
                Some(prev.clone())
            };
    }

    // check if the line segment belongs to the Boolean operation
    let is_in_result = in_result(&event.borrow(), operation);
    if is_in_result {
        let transition = determine_result_transition(&event.borrow(), operation);
        event.borrow_mut().result_transition = transition;
    } else {
        event.borrow_mut().result_transition = ResultTransitionType::NotInResult;
    }
}

fn in_result(event: &SweepEvent, operation: Operation) -> bool {
    match event.edge_type {
        EdgeType::Normal => match operation {
            Operation::Intersection => !event.other_in_out,
            Operation::Union => event.other_in_out,
            Operation::Difference => {
                (event.is_subject && event.other_in_out)
                    || (!event.is_subject && !event.other_in_out)
            }
            Operation::Xor => true,
        },
        EdgeType::SameTransition => {
            operation == Operation::Intersection || operation == Operation::Union
        }
        EdgeType::DifferentTransition => operation == Operation::Difference,
        EdgeType::NonContributing => false,
    }
}

fn determine_result_transition(event: &SweepEvent, operation: Operation) -> ResultTransitionType {
    let this_in = !event.in_out;
    let that_in = !event.other_in_out;

    let is_in = match operation {
        Operation::Intersection => this_in && that_in,
        Operation::Union => this_in || that_in,
        Operation::Xor => this_in ^ that_in,
        Operation::Difference => {
            if event.is_subject {
                this_in && !that_in
            } else {
                that_in && !this_in
            }
        }
    };

    if is_in {
        ResultTransitionType::OutIn
    } else {
        ResultTransitionType::InOut
    }
}
