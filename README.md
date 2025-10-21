# martinez-rs

This is a Rust port of [https://github.com/w8r/martinez@3d55204](https://github.com/w8r/martinez/tree/3d55204b7263977127a3b315c89f888e2484d319): Martinez-Rueda polygon clipping algorithm.

## Features

* Arena-based, no `Rc<RefCell<_>>` shenanigan.
* Only 1 dependency `robust = "1.2.0"`.
* Reusing allocation.

## API

* `fn union(subject: &[Polygon], clipping: &[Polygon]) -> Option<MultiPolygon>`
* `fn diff(subject: &[Polygon], clipping: &[Polygon]) -> Option<MultiPolygon>`
* `fn xor(subject: &[Polygon], clipping: &[Polygon]) -> Option<MultiPolygon>`
* `fn intersection(subject: &[Polygon], clipping: &[Polygon]) -> Option<MultiPolygon>`

Where

* `type Point = [f64; 2]`
* `type Polygon = Vec<Vec<Point>>`
* `type MultiPolygon = Vec<Polygon>`

### Reusing allocation API

```rust
let mut b = Boolean::default();
let mut result = ReusableResult::default();
if b.union(subject, clipping, &mut result).is_some() {
     for pi in 0..result.num_polygons() { // akin to iterating over MultiPolygon
          for ci in 0..self.num_contours(pi) { // akin to iterating over Polygon
               let contour: &[[f64; 2]] = self.contour_points(pi, ci);
          }
     }
}

// then we can reuse b and result like this 
// b.union(_, _, &mut result)
// b.diff(_, _, &mut result)
// b.xor(_, _, &mut result)
// b.intersection(_, _, &mut result)
```

## Benchmark

```bash
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

### Reusing allocation

```bash
cargo bench --features bench # to get baseline benchmark
cargo bench --features bench,bench_reuse
```

```
     Running benches/hole_hole.rs (target/release/deps/hole_hole-d1af5d6bea2efcce)
Hole_Hole/martinez_rs::union
                        time:   [6.2341 µs 6.3839 µs 6.5479 µs]
                        thrpt:  [152.72 Kelem/s 156.64 Kelem/s 160.41 Kelem/s]
                 change:
                        time:   [−31.270% −27.063% −22.994%] (p = 0.00 < 0.05)
                        thrpt:  [+29.860% +37.105% +45.497%]
                        Performance has improved.

     Running benches/asia_union.rs (target/release/deps/asia_union-31194db4689f8931)
Asia union/martinez_rs::union
                        time:   [18.359 ms 18.795 ms 19.259 ms]
                        thrpt:  [51.923  elem/s 53.205  elem/s 54.470  elem/s]
                 change:
                        time:   [−8.5921% −5.8915% −2.9318%] (p = 0.00 < 0.05)
                        thrpt:  [+3.0203% +6.2603% +9.3998%]
                        Performance has improved.

     Running benches/states_source.rs (target/release/deps/states_source-88f07f73142e6d26)
State clip/martinez_rs::union
                        time:   [1.1203 ms 1.1411 ms 1.1640 ms]
                        thrpt:  [859.08  elem/s 876.37  elem/s 892.61  elem/s]
                 change:
                        time:   [−10.283% −5.4709% −0.0627%] (p = 0.04 < 0.05)
                        thrpt:  [+0.0627% +5.7875% +11.462%]
                        Change within noise threshold.
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
