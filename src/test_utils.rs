use geojson::{Feature, GeoJson, Value};

use crate::MultiPolygon;

pub fn geojson_multipolygon_to_multipolygon(mp: Vec<Vec<Vec<Vec<f64>>>>) -> MultiPolygon {
    mp.iter()
        .map(|poly| {
            poly.iter()
                .map(|c| c.iter().map(|p| [p[0], p[1]]).collect::<Vec<_>>())
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
}

pub fn geojson_to_multipolygon(g: &GeoJson) -> MultiPolygon {
    let geometry = match g {
        GeoJson::Geometry(g) => Some(g),
        GeoJson::Feature(Feature {
            geometry: Some(g), ..
        }) => Some(g),
        _ => None,
    };
    geojson_multipolygon_to_multipolygon(match geometry.map(|g| &g.value) {
        Some(Value::MultiPolygon(m)) => m.clone(),
        Some(Value::Polygon(p)) => vec![p.clone()],
        _ => panic!("expected MultiPolygon or Polygon"),
    })
}

pub fn geojson_feature_to_multipolygon(feature: &Feature) -> MultiPolygon {
    geojson_multipolygon_to_multipolygon(match feature.geometry.as_ref().map(|g| &g.value) {
        Some(Value::MultiPolygon(m)) => m.clone(),
        Some(Value::Polygon(p)) => vec![p.clone()],
        _ => panic!("expected MultiPolygon or Polygon"),
    })
}
