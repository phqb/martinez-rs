use crate::{
    edge_type::EdgeType,
    operation::Operation,
    sweep_event::{ResultTransitionType, SweepEvent, SweepEventArena, SweepEventId},
};

pub(crate) fn compute_fields(
    event: SweepEventId,
    prev: Option<SweepEventId>,
    operation: Operation,
    arena: &mut SweepEventArena,
) {
    // compute inOut and otherInOut fields
    if prev.is_none() {
        arena[event].in_out = false;
        arena[event].other_in_out = true;

    // previous line segment in sweepline belongs to the same polygon
    } else {
        let prev = prev.unwrap();

        if arena[event].is_subject == arena[prev].is_subject {
            arena[event].in_out = !arena[prev].in_out;
            arena[event].other_in_out = arena[prev].other_in_out;

        // previous line segment in sweepline belongs to the clipping polygon
        } else {
            arena[event].in_out = !arena[prev].other_in_out;
            arena[event].other_in_out = if arena[prev].is_vertical(arena) {
                !arena[prev].in_out
            } else {
                arena[prev].in_out
            };
        }

        // compute prevInResult field
        arena[event].prev_in_result =
            if !in_result(&arena[prev], operation) || arena[prev].is_vertical(arena) {
                arena[prev].prev_in_result
            } else {
                Some(arena[prev].id)
            };
    }

    // check if the line segment belongs to the Boolean operation
    let is_in_result = in_result(&arena[event], operation);
    if is_in_result {
        let transition = determine_result_transition(&arena[event], operation);
        arena[event].result_transition = transition;
    } else {
        arena[event].result_transition = ResultTransitionType::NotInResult;
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
