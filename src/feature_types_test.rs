use std::fs::File;

use geojson::GeoJson;

use crate::test_utils::geojson_to_multipolygon;

struct TestScenario {
    pub name: &'static str,
    pub subject_poly: &'static str,
}

const OUT_DIR: &str = "test_data/featureTypes/out";

const TEST_SCENARIOS: [TestScenario; 4] = [
    TestScenario {
        name: "polyToClipping",
        subject_poly: "poly",
    },
    TestScenario {
        name: "polyWithHoleToClipping",
        subject_poly: "polyWithHole",
    },
    TestScenario {
        name: "multiPolyToClipping",
        subject_poly: "multiPoly",
    },
    TestScenario {
        name: "multiPolyWithHoleToClipping",
        subject_poly: "multiPolyWithHole",
    },
];

#[test]
fn feature_types_test() {
    let clipping = include_str!("../test_data/featureTypes/clippingPoly.geojson")
        .parse::<GeoJson>()
        .unwrap();
    let clipping = geojson_to_multipolygon(&clipping);

    for tc in TEST_SCENARIOS.iter() {
        let subject = geojson_to_multipolygon(
            &GeoJson::from_reader(
                File::open(format!(
                    "test_data/featureTypes/{}.geojson",
                    tc.subject_poly
                ))
                .unwrap(),
            )
            .unwrap(),
        );

        {
            let inter_expected = geojson_to_multipolygon(
                &GeoJson::from_reader(
                    File::open(format!("{}/intersection/{}.geojson", OUT_DIR, tc.name)).unwrap(),
                )
                .unwrap(),
            );
            let inter_actual = crate::intersection(&subject, &clipping);
            assert_eq!(
                inter_actual,
                Some(inter_expected),
                "{} - Intersect",
                tc.name
            );
        }

        {
            let xor_expected = geojson_to_multipolygon(
                &GeoJson::from_reader(
                    File::open(format!("{}/xor/{}.geojson", OUT_DIR, tc.name)).unwrap(),
                )
                .unwrap(),
            );
            let xor_actual = crate::xor(&subject, &clipping);
            assert_eq!(xor_actual, Some(xor_expected), "{} - XOR", tc.name);
        }

        {
            let diff_expected = geojson_to_multipolygon(
                &GeoJson::from_reader(
                    File::open(format!("{}/difference/{}.geojson", OUT_DIR, tc.name)).unwrap(),
                )
                .unwrap(),
            );
            let diff_actual = crate::diff(&subject, &clipping);
            assert_eq!(diff_actual, Some(diff_expected), "{} - Difference", tc.name);
        }

        {
            let union_expected = geojson_to_multipolygon(
                &GeoJson::from_reader(
                    File::open(format!("{}/union/{}.geojson", OUT_DIR, tc.name)).unwrap(),
                )
                .unwrap(),
            );
            let union_actual = crate::union(&subject, &clipping);
            assert_eq!(union_actual, Some(union_expected), "{} - Union", tc.name);
        }
    }
}
