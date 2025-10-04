use core::{
    cell::RefCell,
    cmp::{Ordering, Reverse},
};
use std::rc::Rc;

use crate::{
    compare_events::compare_events,
    min_heap::MinHeap,
    sweep_event::{SweepEvent, SweepEventOrderedByCompareEvent},
};

pub(crate) fn divide_segment(
    se: Rc<RefCell<SweepEvent>>,
    p: &[f64; 2],
    queue: &mut MinHeap<SweepEventOrderedByCompareEvent>,
) {
    let r = SweepEvent::new(*p, false, Some(se.clone()), se.borrow().is_subject, None);
    let l = SweepEvent::new(
        *p,
        true,
        Some(se.borrow().other_event.as_ref().unwrap().clone()),
        se.borrow().is_subject,
        None,
    );

    r.borrow_mut().contour_id = se.borrow().contour_id;
    l.borrow_mut().contour_id = se.borrow().contour_id;

    // avoid a rounding error. The left event would be processed after the right event
    if compare_events(
        &l.borrow(),
        &se.borrow().other_event.as_ref().unwrap().borrow(),
    ) > Ordering::Equal
    {
        se.borrow().other_event.as_ref().unwrap().borrow_mut().left = true;
        l.borrow_mut().left = false;
    }

    let se_other_event = se.borrow().other_event.as_ref().cloned().unwrap();
    se_other_event.borrow_mut().other_event = Some(l.clone());
    se.borrow_mut().other_event = Some(r.clone());

    queue.push(Reverse(SweepEventOrderedByCompareEvent(l)));
    queue.push(Reverse(SweepEventOrderedByCompareEvent(r)));
}

#[cfg(test)]
mod tests {
    use core::cmp::{Ordering, Reverse};

    use geojson::{GeoJson, Value};

    use crate::{
        compare_segments::compare_segments,
        divide_segment::divide_segment,
        equals::equals,
        fill_queue::fill_queue,
        min_heap::MinHeap,
        operation::Operation,
        possible_intersection::possible_intersection,
        segment_intersection::intersection,
        subdivide_segments::subdivide,
        sweep_event::{
            SweepEvent, SweepEventDeepEqual, SweepEventOrderedByCompareEvent,
            SweepEventOrderedByCompareSegment,
        },
        tree::SweepEventTree,
    };

    #[test]
    fn divide_segments_divide_2_segments() {
        let se1 = SweepEvent::new(
            [0.0, 0.0],
            true,
            Some(SweepEvent::with_point_and_left([5.0, 5.0], false)),
            true,
            None,
        );
        let se2 = SweepEvent::new(
            [0.0, 5.0],
            true,
            Some(SweepEvent::with_point_and_left([5.0, 0.0], false)),
            false,
            None,
        );

        let mut q: MinHeap<SweepEventOrderedByCompareEvent> = MinHeap::new();

        q.push(Reverse(SweepEventOrderedByCompareEvent(se1.clone())));
        q.push(Reverse(SweepEventOrderedByCompareEvent(se2.clone())));

        let inter = intersection(
            &se1.borrow().point,
            &se1.borrow().other_event.as_ref().unwrap().borrow().point,
            &se2.borrow().point,
            &se2.borrow().other_event.as_ref().unwrap().borrow().point,
            false,
        )
        .unwrap();

        divide_segment(se1.clone(), &inter[0], &mut q);
        divide_segment(se2.clone(), &inter[0], &mut q);

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

        let mut q: MinHeap<SweepEventOrderedByCompareEvent> = MinHeap::new();

        let se1 = SweepEvent::new(
            subject[0][3][..2].try_into().unwrap(),
            true,
            Some(SweepEvent::with_point_and_left(
                subject[0][2][..2].try_into().unwrap(),
                false,
            )),
            true,
            None,
        );
        let se2 = SweepEvent::new(
            clipping[0][0][..2].try_into().unwrap(),
            true,
            Some(SweepEvent::with_point_and_left(
                clipping[0][1][..2].try_into().unwrap(),
                false,
            )),
            false,
            None,
        );

        assert_eq!(possible_intersection(se1.clone(), se2.clone(), &mut q), 1);
        assert_eq!(q.len(), 4);

        let e = q.pop().unwrap();
        assert_eq!(
            e.0.0.borrow().point,
            [100.79403384562251, 233.41363754101192]
        );
        assert_eq!(
            e.0.0.borrow().other_event.as_ref().unwrap().borrow().point,
            [56.0, 181.0],
            "1",
        );

        let e = q.pop().unwrap();
        assert_eq!(
            e.0.0.borrow().point,
            [100.79403384562251, 233.41363754101192]
        );
        assert_eq!(
            e.0.0.borrow().other_event.as_ref().unwrap().borrow().point,
            [16.0, 282.0],
            "2",
        );

        let e = q.pop().unwrap();
        assert_eq!(
            e.0.0.borrow().point,
            [100.79403384562251, 233.41363754101192]
        );
        assert_eq!(
            e.0.0.borrow().other_event.as_ref().unwrap().borrow().point,
            [153.0, 203.5],
            "3",
        );

        let e = q.pop().unwrap();
        assert_eq!(
            e.0.0.borrow().point,
            [100.79403384562251, 233.41363754101192]
        );
        assert_eq!(
            e.0.0.borrow().other_event.as_ref().unwrap().borrow().point,
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
        let mut q = fill_queue(&[subject], &[clipping], &mut sbbox, &mut cbbox, None);
        let p0 = [16.0, 282.0];
        let p1 = [298.0, 359.0];
        let p2 = [156.0, 203.5];

        let te = SweepEvent::new(p0, true, None, true, None);
        let te2 = SweepEvent::new(p1, false, Some(te.clone()), false, None);
        te.borrow_mut().other_event = Some(te2.clone());

        let te3 = SweepEvent::new(p0, true, None, true, None);
        let te4 = SweepEvent::new(p2, true, Some(te3.clone()), false, None);
        te3.borrow_mut().other_event = Some(te4.clone());

        let mut tr = SweepEventTree::new();
        tr.insert(SweepEventOrderedByCompareSegment(te.clone()));
        tr.insert(SweepEventOrderedByCompareSegment(te3.clone()));

        assert_eq!(
            tr.find(&SweepEventOrderedByCompareSegment(te.clone()))
                .map(|e| SweepEventDeepEqual(e.0.clone())),
            Some(SweepEventDeepEqual(te.clone())),
        );
        assert_eq!(
            tr.find(&SweepEventOrderedByCompareSegment(te3.clone()))
                .map(|e| SweepEventDeepEqual(e.0.clone())),
            Some(SweepEventDeepEqual(te3.clone())),
        );

        assert_eq!(
            compare_segments(&te.borrow(), &te3.borrow()),
            Ordering::Greater
        );
        assert_eq!(
            compare_segments(&te3.borrow(), &te.borrow()),
            Ordering::Less
        );

        let segments = subdivide(&mut q, &sbbox, &cbbox, Operation::Intersection);
        let left_segments = segments
            .iter()
            .filter(|s| s.borrow().left)
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
            for seg in left_segments.iter() {
                passed = equals(&seg.borrow().point, &data.l)
                    && equals(
                        &seg.borrow().other_event.as_ref().unwrap().borrow().point,
                        &data.r,
                    )
                    && seg.borrow().in_out == data.in_out
                    && seg.borrow().other_in_out == data.other_in_out
                    && seg.borrow().in_result() == data.in_result
                    && match (&seg.borrow().prev_in_result, &data.prev_in_result) {
                        (Some(seg), Some(data)) => {
                            equals(&seg.borrow().point, &data.l)
                                && equals(
                                    &seg.borrow().other_event.as_ref().unwrap().borrow().point,
                                    &data.r,
                                )
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
