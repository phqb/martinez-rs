# martinez-rs

This is a Rust port of [https://github.com/w8r/martinez@3d55204](https://github.com/w8r/martinez/tree/3d55204b7263977127a3b315c89f888e2484d319): Martinez-Rueda polygon clipping algorithm.

## Features

* Arena-based, no `Rc<RefCell<_>>` shenanigan.
* Only 1 dependency `robust = "1.2.0"`.

## API

* `fn union(subject: &[Polygon], clipping: &[Polygon]) -> Option<MultiPolygon>`
* `fn diff(subject: &[Polygon], clipping: &[Polygon]) -> Option<MultiPolygon>`
* `fn xor(subject: &[Polygon], clipping: &[Polygon]) -> Option<MultiPolygon>`
* `fn intersection(subject: &[Polygon], clipping: &[Polygon]) -> Option<MultiPolygon>`

Where

* `type Point = [f64; 2]`
* `type Polygon = Vec<Vec<Point>>`
* `type MultiPolygon = Vec<Polygon>`

## Benchmark

```
cargo bench --features bench
```

My output (reduced) on Windows WSL on ThinkPad P14s Gen 5 Intel:
```
     Running benches/hole_hole.rs (target/release/deps/hole_hole-bf91b19cc9d77348)
Hole_Hole/martinez_rs::union
                        time:   [8.8146 µs 8.9886 µs 9.1654 µs]
                        thrpt:  [109.11 Kelem/s 111.25 Kelem/s 113.45 Kelem/s]

     Running benches/asia_union.rs (target/release/deps/asia_union-ece3ccfc91b83d13)
Asia union/martinez_rs::union
                        time:   [20.450 ms 20.835 ms 21.231 ms]
                        thrpt:  [47.101  elem/s 47.995  elem/s 48.899  elem/s]

     Running benches/states_source.rs (target/release/deps/states_source-b47c8793cb4ecfec)
State clip/martinez_rs::union
                        time:   [1.2449 ms 1.2733 ms 1.3026 ms]
                        thrpt:  [767.68  elem/s 785.36  elem/s 803.29  elem/s]
```

## Ported tests

 - [x] compare_events.test.js -> compare_events::tests
 - [x] compare_segments.test.js -> compare_segments::tests
 - [x] compute_fields.test.js
    * There is no test
 - [x] divide_segment.test.js -> divide_segment::tests
 - [x] featureTypes.test.js -> feature_types_test
 - [x] genericTestCases.test.js -> generic_test_cases
 - [x] index.test.js -> tests
 - [x] segment_intersection.test.js -> segment_intersection::tests
 - [x] signed_area.test.js -> signed_area::tests
 - [x] sweep_event.test.js -> sweep_event::tests
 - [x] sweep_line.test.js -> sweep_line::tests
 - [x] types.ts -> tests

## TODO

- [ ] no_std
