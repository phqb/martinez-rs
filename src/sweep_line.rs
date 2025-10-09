#[cfg(test)]
mod tests {
    use geojson::GeoJson;

    use crate::{
        sweep_event::{SweepEvent, SweepEventArena, SweepEventDeepEqual},
        test_utils::geojson_feature_to_multipolygon,
        tree_by_compare_segments::TreeByCompareSegments,
    };

    #[test]
    fn sweep_line() {
        let mut arena = SweepEventArena::new();
        let data = include_str!("../test_data/fixtures/two_triangles.geojson")
            .parse::<GeoJson>()
            .unwrap();
        let data = if let GeoJson::FeatureCollection(data) = data {
            data
        } else {
            panic!("expected GeoJson::FeatureCollection")
        };

        let subject = &geojson_feature_to_multipolygon(&data.features[0])[0];
        let clipping = &geojson_feature_to_multipolygon(&data.features[1])[0];

        let ef_id = SweepEvent::new(
            subject[0][0],
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
        let eg_id = SweepEvent::new(
            subject[0][0],
            true,
            Some(SweepEvent::with_point_and_left(
                subject[0][1],
                false,
                &mut arena,
            )),
            true,
            None,
            &mut arena,
        );

        let mut tree = TreeByCompareSegments::new();
        tree.insert(ef_id, &arena);
        tree.insert(eg_id, &arena);

        assert_eq!(
            tree.find(ef_id, &arena)
                .map(|node| SweepEventDeepEqual(arena[node.value].clone(), &arena)),
            Some(SweepEventDeepEqual(arena[ef_id].clone(), &arena)),
            "able to retrieve node",
        );
        assert_eq!(
            tree.min()
                .map(|node| SweepEventDeepEqual(arena[node.value].clone(), &arena)),
            Some(SweepEventDeepEqual(arena[ef_id].clone(), &arena)),
            "EF is at the begin",
        );
        assert_eq!(
            tree.max()
                .map(|node| SweepEventDeepEqual(arena[node.value].clone(), &arena)),
            Some(SweepEventDeepEqual(arena[eg_id].clone(), &arena)),
            "EG is at the end",
        );

        let it = tree.find(ef_id, &arena).unwrap();
        assert_eq!(
            tree.next(&it)
                .map(|node| SweepEventDeepEqual(arena[node.value].clone(), &arena)),
            Some(SweepEventDeepEqual(arena[eg_id].clone(), &arena)),
        );

        let it = tree.find(eg_id, &arena).unwrap();
        assert_eq!(
            tree.prev(&it)
                .map(|node| SweepEventDeepEqual(arena[node.value].clone(), &arena)),
            Some(SweepEventDeepEqual(arena[ef_id].clone(), &arena)),
        );

        let da_id = SweepEvent::new(
            clipping[0][0],
            true,
            Some(SweepEvent::with_point_and_left(
                clipping[0][2],
                false,
                &mut arena,
            )),
            true,
            None,
            &mut arena,
        );
        let dc_id = SweepEvent::new(
            clipping[0][0],
            true,
            Some(SweepEvent::with_point_and_left(
                clipping[0][1],
                false,
                &mut arena,
            )),
            true,
            None,
            &mut arena,
        );

        tree.insert(da_id, &arena);
        tree.insert(dc_id, &arena);

        let begin = tree.min().unwrap();
        assert_eq!(
            SweepEventDeepEqual(arena[begin.value].clone(), &arena),
            SweepEventDeepEqual(arena[da_id].clone(), &arena),
            "DA",
        );
        let begin = tree.next(&begin).unwrap();
        assert_eq!(
            SweepEventDeepEqual(arena[begin.value].clone(), &arena),
            SweepEventDeepEqual(arena[dc_id].clone(), &arena),
            "DC",
        );
        let begin = tree.next(&begin).unwrap();
        assert_eq!(
            SweepEventDeepEqual(arena[begin.value].clone(), &arena),
            SweepEventDeepEqual(arena[ef_id].clone(), &arena),
            "EF",
        );
        let begin = tree.next(&begin).unwrap();
        assert_eq!(
            SweepEventDeepEqual(arena[begin.value].clone(), &arena),
            SweepEventDeepEqual(arena[eg_id].clone(), &arena),
            "EG",
        );
    }
}
