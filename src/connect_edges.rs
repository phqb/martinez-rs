use core::cmp::Ordering;
use std::collections::HashSet;

use crate::{
    compare_events::compare_events,
    contour::Contour,
    sweep_event::{ResultTransitionType, SweepEventArena, SweepEventId},
};

fn order_events(sorted_events: &[SweepEventId], arena: &mut SweepEventArena) -> Vec<SweepEventId> {
    let mut result_events = vec![];
    for &event in sorted_events {
        if (arena[event].left && arena[event].in_result())
            || (!arena[event].left
                && arena[arena[event].other_event.expect("SweepEvent.other_event")].in_result())
        {
            result_events.push(event);
        }
    }
    // Due to overlapping edges the resultEvents array can be not wholly sorted
    let mut sorted = false;
    while !sorted {
        sorted = true;
        for i in 0..result_events.len() {
            if (i + 1) < result_events.len()
                && compare_events(
                    &arena[result_events[i]],
                    &arena[result_events[i + 1]],
                    arena,
                ) == Ordering::Greater
            {
                result_events.swap(i, i + 1);
                sorted = false;
            }
        }
    }

    #[allow(clippy::needless_range_loop)]
    for i in 0..result_events.len() {
        arena[result_events[i]].other_pos = i as i64;
    }

    // imagine, the right event is found in the beginning of the queue,
    // when his left counterpart is not marked yet
    for &event in result_events.iter() {
        if !arena[event].left {
            let tmp = arena[event].other_pos;
            arena[event].other_pos =
                arena[arena[event].other_event.expect("SweepEvent.other_event")].other_pos;
            let other_event = arena[event].other_event.expect("SweepEvent.other_event");
            arena[other_event].other_pos = tmp;
        }
    }

    result_events
}

fn next_pos(
    pos: i64,
    result_events: &[SweepEventId],
    processed: &HashSet<i64>,
    orig_pos: i64,
    arena: &SweepEventArena,
) -> i64 {
    let mut new_pos = pos + 1;
    let p = arena[result_events[pos as usize]].point;
    let length = result_events.len() as i64;
    let mut p1 = [0.0; 2];

    if new_pos < length {
        p1 = arena[result_events[new_pos as usize]].point;
    }

    while new_pos < length && p1[0] == p[0] && p1[1] == p[1] {
        if !processed.contains(&new_pos) {
            return new_pos;
        } else {
            new_pos += 1;
        }
        if new_pos < length {
            p1 = arena[result_events[new_pos as usize]].point;
        }
    }

    new_pos = pos - 1;

    while processed.contains(&new_pos) && new_pos > orig_pos {
        new_pos -= 1;
    }

    new_pos
}

fn initialize_contour_from_context(
    event: SweepEventId,
    contours: &mut [Contour],
    contour_id: i64,
    arena: &mut SweepEventArena,
    params: (
        &[Vec<Vec<[f64; 2]>>],
        &[Vec<Vec<[f64; 2]>>],
        crate::Operation,
    ),
) -> Contour {
    let mut contour = Contour::default();
    if let Some(prev_in_result) = arena[event].prev_in_result {
        let lower_contour_id = arena[prev_in_result].output_contour_id;
        let lower_result_transition = arena[prev_in_result].result_transition;
        if lower_result_transition > ResultTransitionType::NotInResult {
            // We are inside. Now we have to check if the thing below us is another hole or
            // an exterior contour.
            let lower_contour = &contours[lower_contour_id as usize];
            if let Some(hole_of) = lower_contour.hole_of {
                // The lower contour is a hole => Connect the new contour as a hole to its parent,
                // and use same depth.
                let parent_contour_id = hole_of;
                contours[parent_contour_id as usize]
                    .hole_ids
                    .push(contour_id);
                contour.hole_of = Some(parent_contour_id);
                contour.depth = contours[lower_contour_id as usize].depth;
            } else {
                // The lower contour is an exterior contour => Connect the new contour as a hole,
                // and increment depth.
                contours[lower_contour_id as usize]
                    .hole_ids
                    .push(contour_id);
                contour.hole_of = Some(lower_contour_id);
                contour.depth = contours[lower_contour_id as usize].depth + 1;
            }
        } else {
            // We are outside => this contour is an exterior contour of same depth.
            contour.hole_of = None;
            if lower_contour_id < 0 || lower_contour_id as usize > contours.len() {
                println!("{:?}", params.0);
                println!("{:?}", params.1);
                println!("{:?}", params.2);
            }
            contour.depth = contours[lower_contour_id as usize].depth;
        }
    } else {
        // There is no lower/previous contour => this contour is an exterior contour of depth 0.
        contour.hole_of = None;
        contour.depth = 0;
    }

    contour
}

pub(crate) fn connect_edges(
    sorted_events: &[SweepEventId],
    arena: &mut SweepEventArena,
    params: (
        &[Vec<Vec<[f64; 2]>>],
        &[Vec<Vec<[f64; 2]>>],
        crate::Operation,
    ),
) -> Vec<Contour> {
    let result_events = order_events(sorted_events, arena);

    let mut processed = HashSet::<i64>::new();
    let mut contours = vec![];

    for i in 0..result_events.len() {
        if processed.contains(&(i as i64)) {
            continue;
        }

        let contour_id = contours.len() as i64;
        let mut contour = initialize_contour_from_context(
            result_events[i],
            &mut contours,
            contour_id,
            arena,
            params,
        );

        // Helper macro that combines marking an event as processed with assigning its output contour ID
        macro_rules! mark_as_processed {
            ($pos:ident) => {
                processed.insert($pos);
                if $pos >= 0 && $pos < result_events.len() as i64 {
                    arena[result_events[$pos as usize]].output_contour_id = contour_id;
                }
            };
        }

        let mut pos = i as i64;
        let orig_pos = i as i64;

        let initial = arena[result_events[i]].point;
        contour.points.push(initial);

        loop {
            mark_as_processed!(pos);

            pos = arena[result_events[pos as usize]].other_pos;

            mark_as_processed!(pos);
            contour
                .points
                .push(arena[result_events[pos as usize]].point);

            pos = next_pos(pos, &result_events, &processed, orig_pos, arena);

            if pos == orig_pos || pos >= result_events.len() as i64 || pos < 0 {
                break;
            }
        }
        contours.push(contour);
    }

    contours
}
