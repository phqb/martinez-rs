use core::cmp::Ordering;

use crate::{
    compare_events::compare_events,
    min_heap::MinHeapByCompareEvents,
    sweep_event::{SweepEvent, SweepEventArena, SweepEventId},
};

pub(crate) fn divide_segment(
    se_id: SweepEventId,
    p: [f64; 2],
    queue: &mut MinHeapByCompareEvents,
    arena: &mut SweepEventArena,
) {
    let se = arena[se_id].clone();
    let r_id = SweepEvent::new(p, false, Some(se.id), se.is_subject, None, arena);
    let l_id = SweepEvent::new(p, true, se.other_event, se.is_subject, None, arena);

    arena[r_id].contour_id = se.contour_id;
    arena[l_id].contour_id = se.contour_id;

    // avoid a rounding error. The left event would be processed after the right event
    if compare_events(&arena[l_id], &arena[se.other_event.unwrap()], arena) > Ordering::Equal {
        arena[se.other_event.unwrap()].left = true;
        arena[l_id].left = false;
    }

    arena[se.other_event.unwrap()].other_event = Some(l_id);
    arena[se_id].other_event = Some(r_id);

    queue.push(l_id, arena);
    queue.push(r_id, arena);
}

#[cfg(test)]
mod tests {
    use core::cmp::Ordering;

    use geojson::{GeoJson, Value};

    use crate::{
        compare_segments::compare_segments,
        divide_segment::divide_segment,
        equals::equals,
        fill_queue::fill_queue,
        min_heap::MinHeapByCompareEvents,
        operation::Operation,
        possible_intersection::possible_intersection,
        segment_intersection::intersection,
        subdivide_segments::subdivide,
        sweep_event::{SweepEvent, SweepEventArena, SweepEventDeepEqual},
        test_utils::geojson_feature_to_multipolygon,
        tree_by_compare_segments::TreeByCompareSegments,
    };

    #[test]
    fn divide_segments_divide_2_segments() {
        let mut arena = SweepEventArena::new();
        let se1_id = SweepEvent::new(
            [0.0, 0.0],
            true,
            Some(SweepEvent::with_point_and_left(
                [5.0, 5.0],
                false,
                &mut arena,
            )),
            true,
            None,
            &mut arena,
        );
        let se2_id = SweepEvent::new(
            [0.0, 5.0],
            true,
            Some(SweepEvent::with_point_and_left(
                [5.0, 0.0],
                false,
                &mut arena,
            )),
            false,
            None,
            &mut arena,
        );

        let mut q = MinHeapByCompareEvents::new();

        q.push(se1_id, &arena);
        q.push(se2_id, &arena);

        let inter = intersection(
            &arena[se1_id].point,
            &arena[arena[se1_id].other_event.unwrap()].point,
            &arena[se2_id].point,
            &arena[arena[se2_id].other_event.unwrap()].point,
            false,
        )
        .unwrap();

        divide_segment(se1_id, inter[0], &mut q, &mut arena);
        divide_segment(se2_id, inter[0], &mut q, &mut arena);

        assert_eq!(
            q.len(),
            6,
            "subdivided in 4 segments by intersection point'"
        );
    }

    #[test]
    fn divide_segments_possible_intersections() {
        let data = include_str!("../test_data/fixtures/two_shapes.geojson")
            .parse::<GeoJson>()
            .unwrap();
        let data = if let GeoJson::FeatureCollection(data) = data {
            data
        } else {
            panic!("expected GeoJson::FeatureCollection")
        };

        let subject = &geojson_feature_to_multipolygon(&data.features[0])[0];
        let clipping = &geojson_feature_to_multipolygon(&data.features[1])[0];

        let mut q = MinHeapByCompareEvents::new();

        let mut arena = SweepEventArena::new();
        let se1_id = SweepEvent::new(
            subject[0][3],
            true,
            Some(SweepEvent::with_point_and_left(
                subject[0][2],
                false,
                &mut arena,
            )),
            true,
            None,
            &mut arena,
        );
        let se2_id = SweepEvent::new(
            clipping[0][0],
            true,
            Some(SweepEvent::with_point_and_left(
                clipping[0][1],
                false,
                &mut arena,
            )),
            false,
            None,
            &mut arena,
        );

        assert_eq!(possible_intersection(se1_id, se2_id, &mut q, &mut arena), 1);
        assert_eq!(q.len(), 4);

        let e = q.pop(&arena).unwrap();
        assert_eq!(arena[e].point, [100.79403384562251, 233.41363754101192]);
        assert_eq!(
            arena[arena[e].other_event.unwrap()].point,
            [56.0, 181.0],
            "1",
        );

        let e = q.pop(&arena).unwrap();
        assert_eq!(arena[e].point, [100.79403384562251, 233.41363754101192]);
        assert_eq!(
            arena[arena[e].other_event.unwrap()].point,
            [16.0, 282.0],
            "2",
        );

        let e = q.pop(&arena).unwrap();
        assert_eq!(arena[e].point, [100.79403384562251, 233.41363754101192]);
        assert_eq!(
            arena[arena[e].other_event.unwrap()].point,
            [153.0, 203.5],
            "3",
        );

        let e = q.pop(&arena).unwrap();
        assert_eq!(arena[e].point, [100.79403384562251, 233.41363754101192]);
        assert_eq!(
            arena[arena[e].other_event.unwrap()].point,
            [153.0, 294.5],
            "4",
        );
    }

    #[test]
    fn divide_segments_possible_intersections_on_2_polygons() {
        let data = include_str!("../test_data/fixtures/two_shapes.geojson")
            .parse::<GeoJson>()
            .unwrap();
        let data = if let GeoJson::FeatureCollection(data) = data {
            data
        } else {
            panic!("expected GeoJson::FeatureCollection")
        };

        let subject = data.features[0]
            .geometry
            .as_ref()
            .and_then(|g| {
                if let Value::Polygon(polygon) = &g.value {
                    Some(polygon.as_slice())
                } else {
                    None
                }
            })
            .unwrap();
        let clipping = data.features[1]
            .geometry
            .as_ref()
            .and_then(|g| {
                if let Value::Polygon(polygon) = &g.value {
                    Some(polygon.as_slice())
                } else {
                    None
                }
            })
            .unwrap();

        let subject = subject
            .iter()
            .map(|v| v.iter().map(|p| [p[0], p[1]]).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let clipping = clipping
            .iter()
            .map(|v| v.iter().map(|p| [p[0], p[1]]).collect::<Vec<_>>())
            .collect::<Vec<_>>();

        let mut sbbox = [f64::INFINITY, f64::INFINITY, -f64::INFINITY, -f64::INFINITY];
        let mut cbbox = [f64::INFINITY, f64::INFINITY, -f64::INFINITY, -f64::INFINITY];
        let mut arena = SweepEventArena::new();
        let mut q = fill_queue(
            &[subject],
            &[clipping],
            &mut sbbox,
            &mut cbbox,
            None,
            &mut arena,
        );
        let p0 = [16.0, 282.0];
        let p1 = [298.0, 359.0];
        let p2 = [156.0, 203.5];

        let te_id = SweepEvent::new(p0, true, None, true, None, &mut arena);
        let te2_id = SweepEvent::new(p1, false, Some(te_id), false, None, &mut arena);
        arena[te_id].other_event = Some(te2_id);

        let te3_id = SweepEvent::new(p0, true, None, true, None, &mut arena);
        let te4_id = SweepEvent::new(p2, true, Some(te3_id), false, None, &mut arena);
        arena[te3_id].other_event = Some(te4_id);

        let mut tr = TreeByCompareSegments::new();
        tr.insert(te_id, &arena);
        tr.insert(te3_id, &arena);

        assert_eq!(
            tr.find(te_id, &arena)
                .map(|e| SweepEventDeepEqual(arena[e.value].clone(), &arena)),
            Some(SweepEventDeepEqual(arena[te_id].clone(), &arena)),
        );
        assert_eq!(
            tr.find(te3_id, &arena)
                .map(|e| SweepEventDeepEqual(arena[e.value].clone(), &arena)),
            Some(SweepEventDeepEqual(arena[te3_id].clone(), &arena)),
        );

        assert_eq!(
            compare_segments(&arena[te_id], &arena[te3_id], &arena),
            Ordering::Greater
        );
        assert_eq!(
            compare_segments(&arena[te3_id], &arena[te_id], &arena),
            Ordering::Less
        );

        let segments = subdivide(&mut q, &sbbox, &cbbox, Operation::Intersection, &mut arena);
        let left_segments = segments
            .iter()
            .filter(|s| arena[**s].left)
            .cloned()
            .collect::<Vec<_>>();

        assert_eq!(left_segments.len(), 11);

        let point_e = [16.0, 282.0];
        let point_i = [100.79403384562252, 233.41363754101192];
        let point_g = [298.0, 359.0];
        let point_c = [153.0, 294.5];
        let point_j = [203.36313843035356, 257.5101243166895];
        let point_f = [153.0, 203.5];
        let point_d = [56.0, 181.0];
        let point_a = [108.5, 120.0];
        let point_b = [241.5, 229.5];

        struct Interval {
            l: [f64; 2],
            r: [f64; 2],
            in_out: bool,
            other_in_out: bool,
            in_result: bool,
            prev_in_result: Option<Box<Interval>>,
        }

        let intervals = [
            (
                "EI",
                Interval {
                    l: point_e,
                    r: point_i,
                    in_out: false,
                    other_in_out: true,
                    in_result: false,
                    prev_in_result: None,
                },
            ),
            (
                "IF",
                Interval {
                    l: point_i,
                    r: point_f,
                    in_out: false,
                    other_in_out: false,
                    in_result: true,
                    prev_in_result: None,
                },
            ),
            (
                "FJ",
                Interval {
                    l: point_f,
                    r: point_j,
                    in_out: false,
                    other_in_out: false,
                    in_result: true,
                    prev_in_result: None,
                },
            ),
            (
                "JG",
                Interval {
                    l: point_j,
                    r: point_g,
                    in_out: false,
                    other_in_out: true,
                    in_result: false,
                    prev_in_result: None,
                },
            ),
            (
                "EG",
                Interval {
                    l: point_e,
                    r: point_g,
                    in_out: true,
                    other_in_out: true,
                    in_result: false,
                    prev_in_result: None,
                },
            ),
            (
                "DA",
                Interval {
                    l: point_d,
                    r: point_a,
                    in_out: false,
                    other_in_out: true,
                    in_result: false,
                    prev_in_result: None,
                },
            ),
            (
                "AB",
                Interval {
                    l: point_a,
                    r: point_b,
                    in_out: false,
                    other_in_out: true,
                    in_result: false,
                    prev_in_result: None,
                },
            ),
            (
                "JB",
                Interval {
                    l: point_j,
                    r: point_b,
                    in_out: true,
                    other_in_out: true,
                    in_result: false,
                    prev_in_result: None,
                },
            ),
            (
                "CJ",
                Interval {
                    l: point_c,
                    r: point_j,
                    in_out: true,
                    other_in_out: false,
                    in_result: true,
                    prev_in_result: Some(Box::new(Interval {
                        l: point_f,
                        r: point_j,
                        in_out: false,
                        other_in_out: false,
                        in_result: false,
                        prev_in_result: None,
                    })),
                },
            ),
            (
                "IC",
                Interval {
                    l: point_i,
                    r: point_c,
                    in_out: true,
                    other_in_out: false,
                    in_result: true,
                    prev_in_result: Some(Box::new(Interval {
                        l: point_i,
                        r: point_f,
                        in_out: false,
                        other_in_out: false,
                        in_result: false,
                        prev_in_result: None,
                    })),
                },
            ),
            (
                "DI",
                Interval {
                    l: point_d,
                    r: point_i,
                    in_out: true,
                    other_in_out: true,
                    in_result: false,
                    prev_in_result: None,
                },
            ),
        ];
        for (interval, data) in intervals {
            let mut passed = false;
            for &seg in left_segments.iter() {
                passed = equals(&arena[seg].point, &data.l)
                    && equals(&arena[arena[seg].other_event.unwrap()].point, &data.r)
                    && arena[seg].in_out == data.in_out
                    && arena[seg].other_in_out == data.other_in_out
                    && arena[seg].in_result() == data.in_result
                    && match (arena[seg].prev_in_result, &data.prev_in_result) {
                        (Some(seg), Some(data)) => {
                            equals(&arena[seg].point, &data.l)
                                && equals(&arena[arena[seg].other_event.unwrap()].point, &data.r)
                        }
                        (None, None) => true,
                        _ => false,
                    };
                if passed {
                    break;
                }
            }
            assert!(passed, "{}", interval);
        }
    }
}
