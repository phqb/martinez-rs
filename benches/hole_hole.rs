use criterion::{Criterion, criterion_group, criterion_main};
#[cfg(feature = "bench")]
use martinez_rs::test_utils::geojson_feature_to_multipolygon;

#[cfg(feature = "bench")]
fn criterion_benchmark(c: &mut Criterion) {
    use core::hint::black_box;
    use criterion::Throughput;
    use geojson::GeoJson;

    let mut group = c.benchmark_group("Hole_Hole");
    group.throughput(Throughput::Elements(1));

    let hole_hole = include_str!("../test_data/fixtures/hole_hole.geojson")
        .parse::<GeoJson>()
        .unwrap();
    let hole_hole = if let GeoJson::FeatureCollection(hole_hole) = hole_hole {
        hole_hole
    } else {
        panic!("expected GeoJson::FeatureCollection")
    };

    let subject = geojson_feature_to_multipolygon(&hole_hole.features[0]);
    let clipping = geojson_feature_to_multipolygon(&hole_hole.features[1]);

    cfg_if::cfg_if! {
        if #[cfg(feature = "bench_reuse")] {
            let mut m = martinez_rs::Boolean::default();
            let mut result = martinez_rs::ReusableResult::default();

            group.bench_function("martinez_rs::union", |b| {
                b.iter(|| {
                    m.union(
                        black_box(&subject),
                        black_box(&clipping),
                        black_box(&mut result),
                    )
                })
            });
        } else {
            group.bench_function("martinez_rs::union", |b| {
                b.iter(|| martinez_rs::union(black_box(&subject), black_box(&clipping)))
            });
        }
    }

    group.finish();
}

#[cfg(not(feature = "bench"))]
fn criterion_benchmark(_: &mut Criterion) {
    panic!("must run with bench-utils feature")
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
