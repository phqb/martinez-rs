use crate::{
    connect_edges::connect_edges, fill_queue::fill_queue, operation::Operation,
    subdivide_segments::subdivide, sweep_event::SweepEventArena,
};

mod compare_events;
mod compare_segments;
mod compute_fields;
mod connect_edges;
mod contour;
mod divide_segment;
mod edge_type;
mod equals;
#[cfg(test)]
mod feature_types_test;
mod fill_queue;
#[cfg(test)]
mod generic_test_cases;
mod min_heap;
mod operation;
mod possible_intersection;
mod segment_intersection;
mod signed_area;
#[cfg(feature = "no_splay_tree")]
mod sorted_list;
#[cfg(not(feature = "no_splay_tree"))]
mod splay_tree;
mod subdivide_segments;
mod sweep_event;
#[cfg(test)]
mod sweep_line;
#[cfg(any(test, feature = "bench"))]
pub mod test_utils;
mod tree_by_compare_segments;

fn trivial_operation(
    subject: &[Vec<Vec<[f64; 2]>>],
    clipping: &[Vec<Vec<[f64; 2]>>],
    operation: Operation,
) -> Option<Vec<Vec<Vec<[f64; 2]>>>> {
    if subject.is_empty() || clipping.is_empty() {
        match operation {
            Operation::Intersection => Some(vec![]),
            Operation::Difference => Some(subject.to_vec()),
            Operation::Union | Operation::Xor => {
                if subject.is_empty() {
                    Some(clipping.to_vec())
                } else {
                    Some(subject.to_vec())
                }
            }
        }
    } else {
        None
    }
}

fn compare_bboxes(
    subject: &[Vec<Vec<[f64; 2]>>],
    clipping: &[Vec<Vec<[f64; 2]>>],
    sbbox: &[f64; 4],
    cbbox: &[f64; 4],
    operation: Operation,
) -> Option<Vec<Vec<Vec<[f64; 2]>>>> {
    if sbbox[0] > cbbox[2] || cbbox[0] > sbbox[2] || sbbox[1] > cbbox[3] || cbbox[1] > sbbox[3] {
        match operation {
            Operation::Intersection => Some(vec![]),
            Operation::Difference => Some(subject.to_vec()),
            Operation::Union | Operation::Xor => {
                Some(subject.iter().chain(clipping).cloned().collect::<Vec<_>>())
            }
        }
    } else {
        None
    }
}

fn boolean(
    subject: &[Vec<Vec<[f64; 2]>>],
    clipping: &[Vec<Vec<[f64; 2]>>],
    operation: Operation,
) -> Option<Vec<Vec<Vec<[f64; 2]>>>> {
    if let Some(trivial) = trivial_operation(subject, clipping, operation) {
        return if trivial.is_empty() {
            None
        } else {
            Some(trivial)
        };
    }

    let mut sbbox = [f64::INFINITY, f64::INFINITY, -f64::INFINITY, -f64::INFINITY];
    let mut cbbox = [f64::INFINITY, f64::INFINITY, -f64::INFINITY, -f64::INFINITY];

    let mut arena = SweepEventArena::new();
    let mut event_queue = fill_queue(
        subject,
        clipping,
        &mut sbbox,
        &mut cbbox,
        Some(operation),
        &mut arena,
    );

    if let Some(trivial) = compare_bboxes(subject, clipping, &sbbox, &cbbox, operation) {
        return if trivial.is_empty() {
            None
        } else {
            Some(trivial)
        };
    }

    let sorted_events = subdivide(&mut event_queue, &sbbox, &cbbox, operation, &mut arena);

    let contours = connect_edges(&sorted_events, &mut arena, (subject, clipping, operation));

    let mut polygons = vec![];
    for contour in contours.iter() {
        if contour.is_exterior() {
            let mut rings = vec![contour.points.clone()];
            for hole_id in contour.hole_ids.iter() {
                rings.push(contours[*hole_id as usize].points.clone());
            }
            polygons.push(rings);
        }
    }

    Some(polygons)
}

pub type Point = [f64; 2];
pub type Polygon = Vec<Vec<Point>>;
pub type MultiPolygon = Vec<Polygon>;

pub fn union(subject: &[Polygon], clipping: &[Polygon]) -> Option<MultiPolygon> {
    boolean(subject, clipping, Operation::Union)
}

pub fn diff(subject: &[Polygon], clipping: &[Polygon]) -> Option<MultiPolygon> {
    boolean(subject, clipping, Operation::Difference)
}

#[cfg(test)]
pub(crate) fn diff_ba(a: &[Polygon], b: &[Polygon]) -> Option<MultiPolygon> {
    boolean(b, a, Operation::Difference)
}

pub fn xor(subject: &[Polygon], clipping: &[Polygon]) -> Option<MultiPolygon> {
    boolean(subject, clipping, Operation::Xor)
}

pub fn intersection(subject: &[Polygon], clipping: &[Polygon]) -> Option<MultiPolygon> {
    boolean(subject, clipping, Operation::Intersection)
}

#[cfg(test)]
mod tests {
    use geojson::GeoJson;

    use crate::{
        MultiPolygon, Polygon, fill_queue::fill_queue, sweep_event::SweepEventArena,
        test_utils::geojson_feature_to_multipolygon,
    };

    #[test]
    fn types() {
        let poly1: Polygon = vec![vec![
            [22.0, 24.0],
            [38.0, 24.0],
            [38.0, 37.0],
            [22.0, 37.0],
            [22.0, 24.0],
        ]];
        let poly2: Polygon = vec![vec![
            [32.0, 34.0],
            [48.0, 34.0],
            [48.0, 47.0],
            [32.0, 47.0],
            [32.0, 34.0],
        ]];
        let multi_poly: MultiPolygon = vec![
            vec![vec![
                [22.0, 24.0],
                [38.0, 24.0],
                [38.0, 37.0],
                [22.0, 37.0],
                [22.0, 24.0],
            ]],
            vec![vec![
                [32.0, 34.0],
                [48.0, 34.0],
                [48.0, 47.0],
                [32.0, 47.0],
                [32.0, 34.0],
            ]],
        ];

        crate::xor(&[poly1.clone()], &[poly2.clone()]);
        crate::diff(&[poly1.clone()], &[poly2.clone()]);
        crate::union(&[poly1.clone()], &[poly2.clone()]);
        crate::intersection(&[poly1.clone()], &[poly2.clone()]);

        crate::xor(&multi_poly, &[poly2.clone()]);
        crate::diff(&multi_poly, &[poly2.clone()]);
        crate::union(&multi_poly, &[poly2.clone()]);
        crate::intersection(&multi_poly, &[poly2.clone()]);
    }

    #[test]
    fn fill_event_queue() {
        let data = include_str!("../test_data/fixtures/two_triangles.geojson")
            .parse::<GeoJson>()
            .unwrap();
        let data = if let GeoJson::FeatureCollection(data) = data {
            data
        } else {
            panic!("expected GeoJson::FeatureCollection")
        };
        let subject = geojson_feature_to_multipolygon(&data.features[0]);
        let clipping = geojson_feature_to_multipolygon(&data.features[1]);

        let mut sbbox = [f64::INFINITY, f64::INFINITY, -f64::INFINITY, -f64::INFINITY];
        let mut cbbox = [f64::INFINITY, f64::INFINITY, -f64::INFINITY, -f64::INFINITY];
        let mut arena = SweepEventArena::new();
        let mut q = fill_queue(
            &subject, &clipping, &mut sbbox, &mut cbbox, None, &mut arena,
        );

        {
            // bboxes
            assert_eq!(sbbox, [20.0, -113.5, 226.5, 74.0], "subject bbox");
            assert_eq!(cbbox, [54.5, -198.0, 239.5, 33.5], "clipping bbox");
        }

        {
            // point 0
            let current_point_id = q.pop(&arena).unwrap();
            let current_point = arena[current_point_id].clone();

            assert_eq!(current_point.point, [20.0, -23.5]);
            assert!(current_point.left, "is left");
            assert_eq!(
                arena[current_point.other_event.unwrap()].point,
                [226.5, -113.5],
                "other event"
            );
            assert!(
                !arena[current_point.other_event.unwrap()].left,
                "other event is right"
            );
        }

        {
            // point 1
            let current_point_id = q.pop(&arena).unwrap();
            let current_point = arena[current_point_id].clone();

            assert_eq!(current_point.point, [20.0, -23.5]);
            assert!(current_point.left, "is left");
            assert_eq!(
                arena[current_point.other_event.unwrap()].point,
                [170.0, 74.0],
                "other event"
            );
            assert!(
                !arena[current_point.other_event.unwrap()].left,
                "other event is right"
            );
        }

        {
            // point 2
            let current_point_id = q.pop(&arena).unwrap();
            let current_point = arena[current_point_id].clone();

            assert_eq!(current_point.point, [54.5, -170.5]);
            assert!(current_point.left, "is left");
            assert_eq!(
                arena[current_point.other_event.unwrap()].point,
                [239.5, -198.0],
                "other event"
            );
            assert!(
                !arena[current_point.other_event.unwrap()].left,
                "other event is right"
            );
        }

        {
            // point 3
            let current_point_id = q.pop(&arena).unwrap();
            let current_point = arena[current_point_id].clone();

            assert_eq!(current_point.point, [54.5, -170.5]);
            assert!(current_point.left, "is left");
            assert_eq!(
                arena[current_point.other_event.unwrap()].point,
                [140.5, 33.5],
                "other event"
            );
            assert!(
                !arena[current_point.other_event.unwrap()].left,
                "other event is right"
            );
        }

        {
            // point 4
            let current_point_id = q.pop(&arena).unwrap();
            let current_point = arena[current_point_id].clone();

            assert_eq!(current_point.point, [140.5, 33.5]);
            assert!(!current_point.left, "is right");
            assert_eq!(
                arena[current_point.other_event.unwrap()].point,
                [54.5, -170.5],
                "other event"
            );
            assert!(
                arena[current_point.other_event.unwrap()].left,
                "other event is left"
            );
        }

        {
            // point 5
            let current_point_id = q.pop(&arena).unwrap();
            let current_point = arena[current_point_id].clone();

            assert_eq!(current_point.point, [140.5, 33.5]);
            assert!(current_point.left, "is left");
            assert_eq!(
                arena[current_point.other_event.unwrap()].point,
                [239.5, -198.0],
                "other event"
            );
            assert!(
                !arena[current_point.other_event.unwrap()].left,
                "other event is right"
            );
        }

        {
            // point 6
            let current_point_id = q.pop(&arena).unwrap();
            let current_point = arena[current_point_id].clone();

            assert_eq!(current_point.point, [170.0, 74.0]);
            assert!(!current_point.left, "is right");
            assert_eq!(
                arena[current_point.other_event.unwrap()].point,
                [20.0, -23.5],
                "other event"
            );
            assert!(
                arena[current_point.other_event.unwrap()].left,
                "other event is left"
            );
        }

        {
            // point 7
            let current_point_id = q.pop(&arena).unwrap();
            let current_point = arena[current_point_id].clone();

            assert_eq!(current_point.point, [170.0, 74.0]);
            assert!(current_point.left, "is left");
            assert_eq!(
                arena[current_point.other_event.unwrap()].point,
                [226.5, -113.5],
                "other event"
            );
            assert!(
                !arena[current_point.other_event.unwrap()].left,
                "other event is right"
            );
        }

        {
            // point 8
            let current_point_id = q.pop(&arena).unwrap();
            let current_point = arena[current_point_id].clone();

            assert_eq!(current_point.point, [226.5, -113.5]);
            assert!(!current_point.left, "is right");
            assert_eq!(
                arena[current_point.other_event.unwrap()].point,
                [20.0, -23.5],
                "other event"
            );
            assert!(
                arena[current_point.other_event.unwrap()].left,
                "other event is left"
            );
        }

        {
            // point 9
            let current_point_id = q.pop(&arena).unwrap();
            let current_point = arena[current_point_id].clone();

            assert_eq!(current_point.point, [226.5, -113.5]);
            assert!(!current_point.left, "is right");
            assert_eq!(
                arena[current_point.other_event.unwrap()].point,
                [170.0, 74.0],
                "other event"
            );
            assert!(
                arena[current_point.other_event.unwrap()].left,
                "other event is left"
            );
        }

        {
            // point 10
            let current_point_id = q.pop(&arena).unwrap();
            let current_point = arena[current_point_id].clone();

            assert_eq!(current_point.point, [239.5, -198.0]);
            assert!(!current_point.left, "is right");
            assert_eq!(
                arena[current_point.other_event.unwrap()].point,
                [54.5, -170.5],
                "other event"
            );
            assert!(
                arena[current_point.other_event.unwrap()].left,
                "other event is left"
            );
        }

        {
            // point 11
            let current_point_id = q.pop(&arena).unwrap();
            let current_point = arena[current_point_id].clone();

            assert_eq!(current_point.point, [239.5, -198.0]);
            assert!(!current_point.left, "is right");
            assert_eq!(
                arena[current_point.other_event.unwrap()].point,
                [140.5, 33.5],
                "other event"
            );
            assert!(
                arena[current_point.other_event.unwrap()].left,
                "other event is left"
            );
        }
    }
}
