use std::collections::HashSet;

use crate::{
    Polygon,
    connect_edges::connect_edges_reuse,
    contour::Contour,
    fill_queue::fill_queue_reuse,
    min_heap::MinHeapByCompareEvents,
    operation::Operation,
    subdivide_segments::subdivide_reuse,
    sweep_event::{SweepEventArena, SweepEventId},
    tree_by_compare_segments::TreeByCompareSegments,
};

#[derive(Default)]
#[cfg_attr(test, derive(Debug, PartialEq, Clone))]
pub struct ReusableResult {
    points: Vec<[f64; 2]>,
    polygon_lens: Vec<usize>,
    polygon_ends: Vec<usize>, // polygon_ends[i] = sum(polygon_lens[..i+1])
    contour_ends: Vec<usize>,
}

impl ReusableResult {
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    pub fn num_polygons(&self) -> usize {
        self.polygon_lens.len()
    }

    pub fn num_contours(&self, polygon_index: usize) -> usize {
        self.polygon_lens[polygon_index]
    }

    pub fn contour_points(&self, polygon_index: usize, contour_index: usize) -> &[[f64; 2]] {
        let poly_start = if polygon_index > 0 {
            self.polygon_ends[polygon_index - 1]
        } else {
            0
        };
        let contour_end = self.contour_ends[poly_start + contour_index];
        let contour_start = if poly_start + contour_index > 0 {
            self.contour_ends[poly_start + contour_index - 1]
        } else {
            0
        };
        &self.points[contour_start..contour_end]
    }

    pub fn clear(&mut self) {
        self.points.clear();
        self.polygon_lens.clear();
        self.polygon_ends.clear();
        self.contour_ends.clear();
    }

    pub fn into_polygons(self) -> Vec<Vec<Vec<[f64; 2]>>> {
        let mut polygons = vec![];
        let mut polygon = vec![];
        let mut contour = vec![];

        for pi in 0..self.num_polygons() {
            polygon.clear();
            for ci in 0..self.num_contours(pi) {
                contour.clear();
                let contours = self.contour_points(pi, ci);
                contour.extend_from_slice(contours);
                polygon.push(contour.clone());
            }
            polygons.push(polygon.clone());
        }

        polygons
    }

    fn set_polygons<
        MultiPolygons: IntoIterator<Item = Polygon>,
        Polygon: IntoIterator<Item = Contour>,
        Contour: IntoIterator<Item = Point>,
        Point: AsRef<[f64]>,
    >(
        &mut self,
        polygons: MultiPolygons,
    ) {
        self.clear();

        for poly in polygons {
            let mut poly_len = 0usize;
            for contour in poly {
                let mut contour_len = 0usize;
                for point in contour {
                    self.points.push([point.as_ref()[0], point.as_ref()[1]]);
                    contour_len += 1;
                }
                self.contour_ends.push(contour_len);
                poly_len += 1;
            }
            self.polygon_lens.push(poly_len);
        }

        for i in 1..self.contour_ends.len() {
            self.contour_ends[i] += self.contour_ends[i - 1];
        }
        self.calculate_polygon_ends();
    }

    fn calculate_polygon_ends(&mut self) {
        self.polygon_ends.clear();
        if !self.polygon_lens.is_empty() {
            self.polygon_ends.push(self.polygon_lens[0]);
        }
        for i in 1..self.polygon_lens.len() {
            self.polygon_ends
                .push(self.polygon_ends[i - 1] + self.polygon_lens[i]);
        }
    }
}

#[test]
fn test_reusable_result() {
    let polygons = vec![
        vec![
            vec![[1.0f64, 2.0], [3.0, 4.0], [99.0, 99.0]],
            vec![[5.0, 6.0], [7.0, 8.0]],
            vec![[9.0, 10.0], [11.0, 12.0]],
        ],
        vec![
            vec![[13.0, 14.0], [15.0, 16.0], [100.0, 100.0]],
            vec![[17.0, 18.0], [19.0, 20.0]],
        ],
        vec![vec![[21.0, 22.0], [23.0, 24.0]]],
    ];
    let mut r = ReusableResult::default();
    r.set_polygons(&polygons);
    assert_eq!(
        r,
        ReusableResult {
            points: vec![
                [1.0, 2.0],
                [3.0, 4.0],
                [99.0, 99.0],
                [5.0, 6.0],
                [7.0, 8.0],
                [9.0, 10.0],
                [11.0, 12.0],
                [13.0, 14.0],
                [15.0, 16.0],
                [100.0, 100.0],
                [17.0, 18.0],
                [19.0, 20.0],
                [21.0, 22.0],
                [23.0, 24.0],
            ],
            polygon_lens: vec![3, 2, 1],
            polygon_ends: vec![3, 5, 6],
            contour_ends: vec![3, 5, 7, 10, 12, 14],
        }
    );
    assert_eq!(r.into_polygons(), polygons);
}

fn trivial_operation_reuse(
    subject: &[Vec<Vec<[f64; 2]>>],
    clipping: &[Vec<Vec<[f64; 2]>>],
    operation: Operation,
    result: &mut ReusableResult,
) -> Option<()> {
    if subject.is_empty() || clipping.is_empty() {
        match operation {
            Operation::Intersection => {
                result.clear();
                Some(())
            }
            Operation::Difference => {
                result.set_polygons(subject);
                Some(())
            }
            Operation::Union | Operation::Xor => {
                if subject.is_empty() {
                    result.set_polygons(clipping);
                } else {
                    result.set_polygons(subject);
                }
                Some(())
            }
        }
    } else {
        result.clear();
        None
    }
}

fn compare_bboxes_reuse(
    subject: &[Vec<Vec<[f64; 2]>>],
    clipping: &[Vec<Vec<[f64; 2]>>],
    sbbox: &[f64; 4],
    cbbox: &[f64; 4],
    operation: Operation,
    result: &mut ReusableResult,
) -> Option<()> {
    if sbbox[0] > cbbox[2] || cbbox[0] > sbbox[2] || sbbox[1] > cbbox[3] || cbbox[1] > sbbox[3] {
        match operation {
            Operation::Intersection => {
                result.clear();
                Some(())
            }
            Operation::Difference => {
                result.set_polygons(subject);
                Some(())
            }
            Operation::Union | Operation::Xor => {
                result.set_polygons(subject.iter().chain(clipping));
                Some(())
            }
        }
    } else {
        result.clear();
        None
    }
}

#[derive(Default)]
pub struct Boolean {
    se_arena: SweepEventArena,
    event_queue: MinHeapByCompareEvents,
    sweep_line: TreeByCompareSegments,
    sorted_events: Vec<SweepEventId>,
    result_events: Vec<SweepEventId>,
    processed: HashSet<i64>,
    contours: Vec<Contour>,
}

impl Boolean {
    pub(crate) fn boolean(
        &mut self,
        subject: &[Vec<Vec<[f64; 2]>>],
        clipping: &[Vec<Vec<[f64; 2]>>],
        operation: Operation,
        result: &mut ReusableResult,
    ) -> Option<()> {
        if trivial_operation_reuse(subject, clipping, operation, result).is_some() {
            return if result.is_empty() { None } else { Some(()) };
        }

        let mut sbbox = [f64::INFINITY, f64::INFINITY, -f64::INFINITY, -f64::INFINITY];
        let mut cbbox = [f64::INFINITY, f64::INFINITY, -f64::INFINITY, -f64::INFINITY];

        self.se_arena.clear();
        fill_queue_reuse(
            subject,
            clipping,
            &mut sbbox,
            &mut cbbox,
            Some(operation),
            &mut self.se_arena,
            &mut self.event_queue,
        );

        if compare_bboxes_reuse(subject, clipping, &sbbox, &cbbox, operation, result).is_some() {
            return if result.is_empty() { None } else { Some(()) };
        }

        subdivide_reuse(
            &mut self.event_queue,
            &sbbox,
            &cbbox,
            operation,
            &mut self.se_arena,
            &mut self.sweep_line,
            &mut self.sorted_events,
        );

        connect_edges_reuse(
            &self.sorted_events,
            &mut self.se_arena,
            &mut self.result_events,
            &mut self.processed,
            &mut self.contours,
        );

        result.clear();
        for contour in self.contours.iter() {
            if contour.is_exterior() {
                let mut poly_len = 1;
                for point in contour.points.iter() {
                    result.points.push(*point);
                }

                result.contour_ends.push(contour.points.len());
                for hole_id in contour.hole_ids.iter() {
                    poly_len += 1;
                    let points = &self.contours[*hole_id as usize].points;
                    for point in points.iter() {
                        result.points.push(*point);
                    }
                    result.contour_ends.push(points.len());
                }

                result.polygon_lens.push(poly_len);
            }
        }

        for i in 1..result.contour_ends.len() {
            result.contour_ends[i] += result.contour_ends[i - 1];
        }
        result.calculate_polygon_ends();

        Some(())
    }

    pub fn union(
        &mut self,
        subject: &[Polygon],
        clipping: &[Polygon],
        result: &mut ReusableResult,
    ) -> Option<()> {
        self.boolean(subject, clipping, Operation::Union, result)
    }

    pub fn diff(
        &mut self,
        subject: &[Polygon],
        clipping: &[Polygon],
        result: &mut ReusableResult,
    ) -> Option<()> {
        self.boolean(subject, clipping, Operation::Difference, result)
    }

    pub fn xor(
        &mut self,
        subject: &[Polygon],
        clipping: &[Polygon],
        result: &mut ReusableResult,
    ) -> Option<()> {
        self.boolean(subject, clipping, Operation::Difference, result)
    }

    pub fn intersection(
        &mut self,
        subject: &[Polygon],
        clipping: &[Polygon],
        result: &mut ReusableResult,
    ) -> Option<()> {
        self.boolean(subject, clipping, Operation::Difference, result)
    }
}
