# martinez-rs

This is a Rust port of [https://github.com/w8r/martinez@3d55204](https://github.com/w8r/martinez/tree/3d55204b7263977127a3b315c89f888e2484d319): Martinez-Rueda polygon clipping algorithm.

## Features

* Arena-based, no `Rc<RefCell<_>>` shenanigan.
* Only 1 dependency `robust-predicates = "0.1.4"`.

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
                        time:   [9.1335 µs 9.3239 µs 9.5235 µs]
                        thrpt:  [105.00 Kelem/s 107.25 Kelem/s 109.49 Kelem/s]

     Running benches/asia_union.rs (target/release/deps/asia_union-ece3ccfc91b83d13)
Asia union/martinez_rs::union
                        time:   [21.666 ms 22.095 ms 22.557 ms]
                        thrpt:  [44.332  elem/s 45.260  elem/s 46.155  elem/s]

     Running benches/states_source.rs (target/release/deps/states_source-b47c8793cb4ecfec)
State clip/martinez_rs::union
                        time:   [1.3214 ms 1.3459 ms 1.3709 ms]
                        thrpt:  [729.43  elem/s 742.98  elem/s 756.79  elem/s]
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

- [ ] No FFI in `robust-predicates`.
- [ ] no_std

## License

The MIT License (MIT)

Copyright (c) 2018 Alexander Milevski

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.