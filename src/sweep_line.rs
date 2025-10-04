#[cfg(test)]
mod tests {
    use geojson::{GeoJson, Value};

    use crate::{
        sweep_event::{SweepEvent, SweepEventDeepEqual, SweepEventOrderedByCompareSegment},
        tree::SweepEventTree,
    };

    #[test]
    fn sweep_line() {
        let data = include_str!("../test_data/fixtures/two_triangles.geojson")
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

        let ef = SweepEvent::new(
            subject[0][0][..2].try_into().unwrap(),
            true,
            Some(SweepEvent::with_point_and_left(
                subject[0][2][..2].try_into().unwrap(),
                false,
            )),
            true,
            None,
        );
        let eg = SweepEvent::new(
            subject[0][0][..2].try_into().unwrap(),
            true,
            Some(SweepEvent::with_point_and_left(
                subject[0][1][..2].try_into().unwrap(),
                false,
            )),
            true,
            None,
        );

        let mut tree = SweepEventTree::new();
        tree.insert(SweepEventOrderedByCompareSegment(ef.clone()));
        tree.insert(SweepEventOrderedByCompareSegment(eg.clone()));

        assert_eq!(
            tree.find(&SweepEventOrderedByCompareSegment(ef.clone()))
                .map(|e| SweepEventDeepEqual(e.0.clone())),
            Some(SweepEventDeepEqual(ef.clone())),
            "able to retrieve node",
        );
        assert_eq!(
            tree.min().map(|e| SweepEventDeepEqual(e.0.clone())),
            Some(SweepEventDeepEqual(ef.clone())),
            "EF is at the begin",
        );
        assert_eq!(
            tree.max().map(|e| SweepEventDeepEqual(e.0.clone())),
            Some(SweepEventDeepEqual(eg.clone())),
            "EG is at the end",
        );

        let it = tree
            .find(&SweepEventOrderedByCompareSegment(ef.clone()))
            .unwrap();
        assert_eq!(
            tree.next(&it).map(|e| SweepEventDeepEqual(e.0.clone())),
            Some(SweepEventDeepEqual(eg.clone())),
        );

        let it = tree
            .find(&SweepEventOrderedByCompareSegment(eg.clone()))
            .unwrap();
        assert_eq!(
            tree.prev(&it).map(|e| SweepEventDeepEqual(e.0.clone())),
            Some(SweepEventDeepEqual(ef.clone())),
        );

        let da = SweepEvent::new(
            clipping[0][0][..2].try_into().unwrap(),
            true,
            Some(SweepEvent::with_point_and_left(
                clipping[0][2][..2].try_into().unwrap(),
                false,
            )),
            true,
            None,
        );
        let dc = SweepEvent::new(
            clipping[0][0][..2].try_into().unwrap(),
            true,
            Some(SweepEvent::with_point_and_left(
                clipping[0][1][..2].try_into().unwrap(),
                false,
            )),
            true,
            None,
        );

        tree.insert(SweepEventOrderedByCompareSegment(da.clone()));
        tree.insert(SweepEventOrderedByCompareSegment(dc.clone()));

        let begin = tree.min().unwrap();
        assert_eq!(
            SweepEventDeepEqual(begin.0.clone()),
            SweepEventDeepEqual(da.clone()),
            "DA",
        );
        let begin = tree.next(&begin).unwrap();
        assert_eq!(
            SweepEventDeepEqual(begin.0.clone()),
            SweepEventDeepEqual(dc.clone()),
            "DC",
        );
        let begin = tree.next(&begin).unwrap();
        assert_eq!(
            SweepEventDeepEqual(begin.0.clone()),
            SweepEventDeepEqual(ef.clone()),
            "EF",
        );
        let begin = tree.next(&begin).unwrap();
        assert_eq!(
            SweepEventDeepEqual(begin.0.clone()),
            SweepEventDeepEqual(eg.clone()),
            "EG",
        );
    }
}
