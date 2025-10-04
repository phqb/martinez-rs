use core::{cell::RefCell, cmp::Ordering};
#[cfg(test)]
use std::collections::HashSet;
use std::rc::Rc;

use crate::{
    compare_events::compare_events, compare_segments::compare_segments, edge_type::EdgeType,
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub(crate) enum ResultTransitionType {
    NotInResult = 0,
    OutIn = 1,
    InOut = -1,
}

impl Default for ResultTransitionType {
    fn default() -> Self {
        Self::NotInResult
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct SweepEvent {
    pub point: [f64; 2],
    pub left: bool,
    pub other_event: Option<Rc<RefCell<SweepEvent>>>,
    pub is_subject: bool,
    pub edge_type: EdgeType,
    pub in_out: bool,
    pub other_in_out: bool,
    pub prev_in_result: Option<Rc<RefCell<SweepEvent>>>,
    pub result_transition: ResultTransitionType,
    pub other_pos: i64,
    pub contour_id: i64,
    pub output_contour_id: i64,
    pub is_exterior_ring: bool,
}

impl SweepEvent {
    pub fn new(
        point: [f64; 2],
        left: bool,
        other_event: Option<Rc<RefCell<SweepEvent>>>,
        is_subject: bool,
        edge_type: Option<EdgeType>,
    ) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            point,
            left,
            other_event,
            is_subject,
            edge_type: edge_type.unwrap_or(EdgeType::Normal),
            in_out: false,
            other_in_out: false,
            prev_in_result: None,
            result_transition: ResultTransitionType::NotInResult,
            // connection step
            other_pos: -1,
            contour_id: -1,
            output_contour_id: -1,
            // TODO: Looks unused, remove?
            is_exterior_ring: true,
        }))
    }

    #[cfg(test)]
    pub fn with_point(point: [f64; 2]) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            point,
            ..Default::default()
        }))
    }

    #[cfg(test)]
    pub fn with_point_and_left(point: [f64; 2], left: bool) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            point,
            left,
            ..Default::default()
        }))
    }

    pub fn is_below(&self, p: &[f64; 2]) -> bool {
        let p0 = self.point;
        let p1 = self.other_event.as_ref().unwrap().borrow().point;
        // TODO: use approx?
        if self.left {
            (p0[0] - p[0]) * (p1[1] - p[1]) - (p1[0] - p[0]) * (p0[1] - p[1]) > 0.0
        } else {
            (p1[0] - p[0]) * (p0[1] - p[1]) - (p0[0] - p[0]) * (p1[1] - p[1]) > 0.0
        }
    }

    pub fn is_above(&self, p: &[f64; 2]) -> bool {
        !self.is_below(p)
    }

    pub fn is_vertical(&self) -> bool {
        // TODO: use approx?
        self.point[0] == self.other_event.as_ref().unwrap().borrow().point[0]
    }

    pub fn in_result(&self) -> bool {
        self.result_transition != ResultTransitionType::NotInResult
    }
}

pub(crate) struct SweepEventOrderedByCompareEvent(pub Rc<RefCell<SweepEvent>>);

impl PartialEq for SweepEventOrderedByCompareEvent {
    fn eq(&self, other: &Self) -> bool {
        compare_events(&self.0.borrow(), &other.0.borrow()) == Ordering::Equal
    }
}

impl Eq for SweepEventOrderedByCompareEvent {}

impl PartialOrd for SweepEventOrderedByCompareEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SweepEventOrderedByCompareEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        compare_events(&self.0.borrow(), &other.0.borrow())
    }
}

impl Clone for SweepEventOrderedByCompareEvent {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub(crate) struct SweepEventOrderedByCompareSegment(pub Rc<RefCell<SweepEvent>>);

impl PartialEq for SweepEventOrderedByCompareSegment {
    fn eq(&self, other: &Self) -> bool {
        compare_segments(&self.0.borrow(), &other.0.borrow()) == Ordering::Equal
    }
}

impl Eq for SweepEventOrderedByCompareSegment {}

impl PartialOrd for SweepEventOrderedByCompareSegment {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SweepEventOrderedByCompareSegment {
    fn cmp(&self, other: &Self) -> Ordering {
        compare_segments(&self.0.borrow(), &other.0.borrow())
    }
}

impl Clone for SweepEventOrderedByCompareSegment {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[cfg(test)]
pub(crate) struct SweepEventDeepEqual(pub Rc<RefCell<SweepEvent>>);

#[cfg(test)]
impl graph_safe_compare::Node for SweepEventDeepEqual {
    type Cmp = bool;
    type Id = usize;
    type Index = usize;

    fn id(&self) -> Self::Id {
        self.0.as_ptr() as usize
    }

    fn get_edge(&self, index: &Self::Index) -> Option<Self> {
        let selfz = self.0.borrow();
        match (&selfz.other_event, &selfz.prev_in_result) {
            (Some(other_event), None) => match *index {
                0 => Some(Self(other_event.clone())),
                _ => None,
            },
            (None, Some(prev)) => match *index {
                0 => Some(Self(prev.clone())),
                _ => None,
            },
            (Some(other_event), Some(prev)) => match *index {
                0 => Some(Self(other_event.clone())),
                1 => Some(Self(prev.clone())),
                _ => None,
            },
            (None, None) => None,
        }
    }

    fn equiv_modulo_edges(&self, other: &Self) -> Self::Cmp {
        use crate::equals::equals;

        let a = self.0.borrow();
        let b = other.0.borrow();

        equals(&a.point, &b.point)
            && a.left == b.left
            && a.is_subject == b.is_subject
            && a.edge_type == b.edge_type
            && a.in_out == b.in_out
            && a.other_in_out == b.other_in_out
            && a.result_transition == b.result_transition
            && a.other_pos == b.other_pos
            && a.contour_id == b.contour_id
            && a.output_contour_id == b.output_contour_id
            && a.is_exterior_ring == b.is_exterior_ring
    }
}

#[cfg(test)]
impl PartialEq for SweepEventDeepEqual {
    fn eq(&self, other: &Self) -> bool {
        graph_safe_compare::robust::equiv(Self(self.0.clone()), Self(other.0.clone()))
    }
}

#[cfg(test)]
impl Eq for SweepEventDeepEqual {}

#[cfg(test)]
impl core::fmt::Debug for SweepEventDeepEqual {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let debug = SweepEventDebug {
            inner: self.0.clone(),
            visited: Rc::new(RefCell::new(HashSet::new())),
        };
        debug.fmt(f)
    }
}

#[cfg(test)]
struct SweepEventDebug {
    inner: Rc<RefCell<SweepEvent>>,
    visited: Rc<RefCell<HashSet<usize>>>,
}

#[cfg(test)]
impl SweepEventDebug {
    fn debug(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.visited
            .borrow_mut()
            .insert(self.inner.as_ptr() as usize);

        let selfz = self.inner.borrow();
        let mut debug = f.debug_struct(&format!("SweepEvent({})", self.inner.as_ptr() as usize));

        debug.field("point", &selfz.point);
        debug.field("left", &selfz.left);
        if let Some(other_event) = &selfz.other_event {
            let ptr = other_event.as_ptr() as usize;
            if self.visited.borrow().contains(&ptr) {
                debug.field("other_event", &format!("SweepEvent({ptr})"));
            } else {
                debug.field(
                    "prev_in_result",
                    &SweepEventDebug {
                        inner: other_event.clone(),
                        visited: self.visited.clone(),
                    },
                );
            }
        }
        debug.field("is_subject", &selfz.is_subject);
        debug.field("edge_type", &selfz.edge_type);
        debug.field("in_out", &selfz.in_out);
        debug.field("other_in_out", &selfz.other_in_out);
        if let Some(prev) = &selfz.prev_in_result {
            let ptr = prev.as_ptr() as usize;
            if self.visited.borrow().contains(&ptr) {
                debug.field("prev_in_result", &format!("SweepEvent({ptr})"));
            } else {
                debug.field(
                    "prev_in_result",
                    &SweepEventDebug {
                        inner: prev.clone(),
                        visited: self.visited.clone(),
                    },
                );
            }
        }
        debug.field("result_transition", &selfz.result_transition);
        debug.field("other_pos", &selfz.other_pos);
        debug.field("contour_id", &selfz.contour_id);
        debug.field("output_contour_id", &selfz.output_contour_id);
        debug.field("is_exterior_ring", &selfz.is_exterior_ring);

        debug.finish()
    }
}

#[cfg(test)]
impl core::fmt::Debug for SweepEventDebug {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.debug(f)
    }
}

#[cfg(test)]
#[allow(clippy::bool_assert_comparison)]
mod tests {
    use crate::sweep_event::{SweepEvent, SweepEventDeepEqual};

    #[test]
    fn sweep_event_is_below() {
        let s1 = SweepEvent::new(
            [0.0, 0.0],
            true,
            Some(SweepEvent::with_point_and_left([1.0, 1.0], false)),
            false,
            None,
        );
        let s2 = SweepEvent::new(
            [0.0, 1.0],
            false,
            Some(SweepEvent::with_point_and_left([0.0, 0.0], false)),
            false,
            None,
        );

        assert_eq!(s1.borrow().is_below(&[0.0, 1.0]), true);
        assert_eq!(s1.borrow().is_below(&[1.0, 2.0]), true);
        assert_eq!(s1.borrow().is_below(&[0.0, 0.0]), false);
        assert_eq!(s1.borrow().is_below(&[5.0, -1.0]), false);

        assert_eq!(s2.borrow().is_below(&[0.0, 1.0]), false);
        assert_eq!(s2.borrow().is_below(&[1.0, 2.0]), false);
        assert_eq!(s2.borrow().is_below(&[0.0, 0.0]), false);
        assert_eq!(s2.borrow().is_below(&[5.0, -1.0]), false);
    }

    #[test]
    fn sweep_event_is_above() {
        let s1 = SweepEvent::new(
            [0.0, 0.0],
            true,
            Some(SweepEvent::with_point_and_left([1.0, 1.0], false)),
            false,
            None,
        );
        let s2 = SweepEvent::new(
            [0.0, 1.0],
            false,
            Some(SweepEvent::with_point_and_left([0.0, 0.0], false)),
            false,
            None,
        );

        assert_eq!(s1.borrow().is_above(&[0.0, 1.0]), false);
        assert_eq!(s1.borrow().is_above(&[1.0, 2.0]), false);
        assert_eq!(s1.borrow().is_above(&[0.0, 0.0]), true);
        assert_eq!(s1.borrow().is_above(&[5.0, -1.0]), true);

        assert_eq!(s2.borrow().is_above(&[0.0, 1.0]), true);
        assert_eq!(s2.borrow().is_above(&[1.0, 2.0]), true);
        assert_eq!(s2.borrow().is_above(&[0.0, 0.0]), true);
        assert_eq!(s2.borrow().is_above(&[5.0, -1.0]), true);
    }

    #[test]
    fn sweep_event_is_vertical() {
        assert_eq!(
            SweepEvent::new(
                [0.0, 0.0],
                true,
                Some(SweepEvent::with_point_and_left([0.0, 1.0], false)),
                false,
                None,
            )
            .borrow()
            .is_vertical(),
            true,
        );
        assert_eq!(
            SweepEvent::new(
                [0.0, 0.0],
                true,
                Some(SweepEvent::with_point_and_left([0.0001, 1.0], false)),
                false,
                None,
            )
            .borrow()
            .is_vertical(),
            false,
        );
    }

    #[test]
    fn sweep_event_deep_equal() {
        let se1 = SweepEvent::new([0.1, 0.1], true, None, true, None);
        let se2 = se1.clone();
        assert_eq!(
            SweepEventDeepEqual(se1),
            SweepEventDeepEqual(se2),
            "point to the same memory"
        );

        let se1 = SweepEvent::new([0.1, 0.1], true, None, true, None);
        let se2 = SweepEvent::new([0.1, 0.1], true, None, true, None);
        assert_eq!(
            SweepEventDeepEqual(se1),
            SweepEventDeepEqual(se2),
            "different memory, same value and descendants"
        );

        let se1 = SweepEvent::new([0.1, 0.1], true, None, true, None);
        let se2 = SweepEvent::new([0.1, 0.1], true, Some(se1.clone()), true, None);
        se1.borrow_mut().other_event = Some(se2.clone());
        let se3 = se1.clone();
        assert_eq!(
            SweepEventDeepEqual(se1),
            SweepEventDeepEqual(se3),
            "same memory, cyclic"
        );

        let se1 = SweepEvent::new([0.1, 0.1], true, None, true, None);
        let se2 = SweepEvent::new([0.1, 0.1], true, Some(se1.clone()), true, None);
        se1.borrow_mut().other_event = Some(se2.clone());
        let se3 = SweepEvent::new([0.1, 0.1], true, None, true, None);
        let se4 = SweepEvent::new([0.1, 0.1], true, Some(se3.clone()), true, None);
        se3.borrow_mut().other_event = Some(se4.clone());
        assert_eq!(
            SweepEventDeepEqual(se1),
            SweepEventDeepEqual(se3),
            "different memory, same value and descendants, cyclic"
        );

        let se1 = SweepEvent::new([0.1, 0.1], true, None, true, None);
        let se2 = SweepEvent::new([0.1, 0.1], true, Some(se1.clone()), true, None);
        se1.borrow_mut().other_event = Some(se2.clone());
        let se3 = SweepEvent::new([0.1, 0.1], true, None, true, None);
        let se4 = SweepEvent::new([0.1, 0.2], true, Some(se3.clone()), true, None);
        se3.borrow_mut().other_event = Some(se4.clone());
        assert_ne!(
            SweepEventDeepEqual(se1),
            SweepEventDeepEqual(se3),
            "different value, cyclic"
        );
    }
}
