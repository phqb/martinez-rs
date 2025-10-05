// use crate::{
//     connect_edges::connect_edges, fill_queue::fill_queue, operation::Operation,
//     subdivide_segments::subdivide,
// };

mod compare_events;
mod compare_segments;
mod compute_fields;
mod connect_edges;
mod contour;
mod divide_segment;
mod edge_type;
mod equals;
mod fill_queue;
// #[cfg(test)]
// mod generic_test_cases;
mod min_heap;
mod operation;
mod possible_intersection;
mod segment_intersection;
mod signed_area;
mod subdivide_segments;
mod sweep_event;
// mod sweep_line;
mod tree;

// fn trivial_operation(
//     subject: &[Vec<Vec<[f64; 2]>>],
//     clipping: &[Vec<Vec<[f64; 2]>>],
//     operation: Operation,
// ) -> Option<Vec<Vec<Vec<[f64; 2]>>>> {
//     if subject.is_empty() || clipping.is_empty() {
//         match operation {
//             Operation::Intersection => Some(vec![]),
//             Operation::Difference => Some(subject.to_vec()),
//             Operation::Union | Operation::Xor => {
//                 if subject.is_empty() {
//                     Some(clipping.to_vec())
//                 } else {
//                     Some(subject.to_vec())
//                 }
//             }
//         }
//     } else {
//         None
//     }
// }

// fn compare_bboxes(
//     subject: &[Vec<Vec<[f64; 2]>>],
//     clipping: &[Vec<Vec<[f64; 2]>>],
//     sbbox: &[f64; 4],
//     cbbox: &[f64; 4],
//     operation: Operation,
// ) -> Option<Vec<Vec<Vec<[f64; 2]>>>> {
//     if sbbox[0] > cbbox[2] || cbbox[0] > sbbox[2] || sbbox[1] > cbbox[3] || cbbox[1] > sbbox[3] {
//         match operation {
//             Operation::Intersection => Some(vec![]),
//             Operation::Difference => Some(subject.to_vec()),
//             Operation::Union | Operation::Xor => {
//                 Some(subject.iter().chain(clipping).cloned().collect::<Vec<_>>())
//             }
//         }
//     } else {
//         None
//     }
// }

// fn boolean(
//     subject: &[Vec<Vec<[f64; 2]>>],
//     clipping: &[Vec<Vec<[f64; 2]>>],
//     operation: Operation,
// ) -> Option<Vec<Vec<Vec<[f64; 2]>>>> {
//     if let Some(trivial) = trivial_operation(subject, clipping, operation) {
//         return if trivial.is_empty() {
//             None
//         } else {
//             Some(trivial)
//         };
//     }

//     let mut sbbox = [f64::INFINITY, f64::INFINITY, -f64::INFINITY, -f64::INFINITY];
//     let mut cbbox = [f64::INFINITY, f64::INFINITY, -f64::INFINITY, -f64::INFINITY];

//     let mut event_queue = fill_queue(subject, clipping, &mut sbbox, &mut cbbox, Some(operation));

//     if let Some(trivial) = compare_bboxes(subject, clipping, &sbbox, &cbbox, operation) {
//         return if trivial.is_empty() {
//             None
//         } else {
//             Some(trivial)
//         };
//     }

//     let sorted_events = subdivide(&mut event_queue, &sbbox, &cbbox, operation);

//     let contours = connect_edges(&sorted_events);

//     let mut polygons = vec![];
//     for contour in contours.iter() {
//         if contour.is_exterior() {
//             let mut rings = vec![contour.points.clone()];
//             for hole_id in contour.hole_ids.iter() {
//                 rings.push(contours[*hole_id as usize].points.clone());
//             }
//             polygons.push(rings);
//         }
//     }

//     Some(polygons)
// }

// pub type Point = [f64; 2];
// pub type Polygon = Vec<Vec<Point>>;
// pub type MultiPolygon = Vec<Polygon>;

// pub fn union(subject: &[Polygon], clipping: &[Polygon]) -> Option<MultiPolygon> {
//     boolean(subject, clipping, Operation::Union)
// }

// pub fn diff(subject: &[Polygon], clipping: &[Polygon]) -> Option<MultiPolygon> {
//     boolean(subject, clipping, Operation::Difference)
// }

// #[cfg(test)]
// pub(crate) fn diff_ba(a: &[Polygon], b: &[Polygon]) -> Option<MultiPolygon> {
//     boolean(b, a, Operation::Difference)
// }

// pub fn xor(subject: &[Polygon], clipping: &[Polygon]) -> Option<MultiPolygon> {
//     boolean(subject, clipping, Operation::Xor)
// }

// pub fn intersection(subject: &[Polygon], clipping: &[Polygon]) -> Option<MultiPolygon> {
//     boolean(subject, clipping, Operation::Intersection)
// }
