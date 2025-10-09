use geojson::{Feature, GeoJson, Value};
use test_each_file::test_each_file;

use crate::{MultiPolygon, Polygon, test_utils::geojson_multipolygon_to_multipolygon};

test_each_file!(for ["geojson"] in "./test_data/genericTestCases" => |[data]: [&str; 1]| generic_test([data], None));

test_each_file!(for ["geojson"] in "./test_data/genericTestCases_compare_10_decimals" => |[data]: [&str; 1]| generic_test([data], Some(10)));

struct OperationAndExpectedResult {
    #[allow(clippy::type_complexity)]
    op: Box<dyn FnOnce(&[Polygon], &[Polygon]) -> Option<MultiPolygon>>,
    expected_result: MultiPolygon,
}

fn extract_expected_results(features: &[Feature]) -> Vec<OperationAndExpectedResult> {
    features
        .iter()
        .map(|feature| {
            let operation = feature
                .properties
                .as_ref()
                .expect("expected properties")
                .get("operation")
                .expect("expected .operation property")
                .as_str()
                .expect("expected .operation property to be a string");
            let expected_result = if let Value::MultiPolygon(mp) =
                &feature.geometry.as_ref().expect("expected geometry").value
            {
                geojson_multipolygon_to_multipolygon(mp.clone())
            } else {
                panic!("expected MultiPolygon")
            };
            match operation {
                "union" => OperationAndExpectedResult {
                    op: Box::new(crate::union),
                    expected_result,
                },
                "intersection" => OperationAndExpectedResult {
                    op: Box::new(crate::intersection),
                    expected_result,
                },
                "xor" => OperationAndExpectedResult {
                    op: Box::new(crate::xor),
                    expected_result,
                },
                "diff" => OperationAndExpectedResult {
                    op: Box::new(crate::diff),
                    expected_result,
                },
                "diff_ba" => OperationAndExpectedResult {
                    op: Box::new(crate::diff_ba),
                    expected_result,
                },
                _ => panic!("invalid operation {}", operation),
            }
        })
        .collect()
}

fn generic_test([data]: [&str; 1], approx_decimals: Option<usize>) {
    let data = data.parse::<GeoJson>().unwrap();
    let data = if let GeoJson::FeatureCollection(data) = data {
        data
    } else {
        panic!("expected GeoJson::FeatureCollection")
    };

    if data.features.len() < 2 {
        panic!("test case must contain at least two features");
    }

    let p1_geo = data.features[0].geometry.as_ref().unwrap();
    let p2_geo = data.features[1].geometry.as_ref().unwrap();

    let p1 = geojson_multipolygon_to_multipolygon(match &p1_geo.value {
        Value::Polygon(poly) => vec![poly.clone()],
        Value::MultiPolygon(polys) => polys.clone(),
        _ => panic!("geometry type must be either Polygon or MutiPolygon"),
    });
    let p2 = geojson_multipolygon_to_multipolygon(match &p2_geo.value {
        Value::Polygon(poly) => vec![poly.clone()],
        Value::MultiPolygon(polys) => polys.clone(),
        _ => panic!("geometry type must be either Polygon or MutiPolygon"),
    });

    let expected_results = extract_expected_results(&data.features[2..]);
    for expected_result in expected_results {
        let result = (expected_result.op)(&p1, &p2);
        if let Some(decimals) = approx_decimals {
            assert_eq!(
                Some(round_result(expected_result.expected_result, decimals)),
                result.map(|r| round_result(r, decimals))
            );
        } else {
            assert_eq!(Some(expected_result.expected_result), result);
        }
    }
}

fn round_result(result: Vec<Vec<Vec<[f64; 2]>>>, decimals: usize) -> Vec<Vec<Vec<[String; 2]>>> {
    result
        .into_iter()
        .map(|r| {
            r.into_iter()
                .map(|r| {
                    r.into_iter()
                        .map(|p| {
                            [
                                format!("{:.1$}", p[0], decimals),
                                format!("{:.1$}", p[1], decimals),
                            ]
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
}
