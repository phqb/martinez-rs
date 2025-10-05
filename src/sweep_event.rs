use core::ops::{Deref, DerefMut, Index, IndexMut};
#[cfg(test)]
use std::{cell::RefCell, collections::HashSet, rc::Rc};

use crate::edge_type::EdgeType;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct SweepEventId(usize);

impl Default for SweepEventId {
    fn default() -> Self {
        Self(usize::MAX)
    }
}

pub(crate) struct SweepEventArena(Vec<SweepEvent>);

impl SweepEventArena {
    pub fn new() -> Self {
        Self(vec![])
    }
}

impl Deref for SweepEventArena {
    type Target = Vec<SweepEvent>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SweepEventArena {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Index<SweepEventId> for SweepEventArena {
    type Output = SweepEvent;

    fn index(&self, index: SweepEventId) -> &Self::Output {
        &self.0[index.0]
    }
}

impl IndexMut<SweepEventId> for SweepEventArena {
    fn index_mut(&mut self, index: SweepEventId) -> &mut Self::Output {
        &mut self.0[index.0]
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct SweepEvent {
    pub id: SweepEventId,
    pub point: [f64; 2],
    pub left: bool,
    pub other_event: Option<SweepEventId>,
    pub is_subject: bool,
    pub edge_type: EdgeType,
    pub in_out: bool,
    pub other_in_out: bool,
    pub prev_in_result: Option<SweepEventId>,
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
        other_event: Option<SweepEventId>,
        is_subject: bool,
        edge_type: Option<EdgeType>,
        arena: &mut SweepEventArena,
    ) -> (Self, SweepEventId) {
        let id = SweepEventId(arena.len());
        let event = Self {
            id,
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
        };
        arena.push(event.clone());
        (event, id)
    }

    #[cfg(test)]
    pub fn with_point(point: [f64; 2], arena: &mut SweepEventArena) -> (Self, SweepEventId) {
        let id = SweepEventId(arena.len());
        let event = Self {
            point,
            ..Default::default()
        };
        arena.push(event.clone());
        (event, id)
    }

    #[cfg(test)]
    pub fn with_point_and_left(
        point: [f64; 2],
        left: bool,
        arena: &mut SweepEventArena,
    ) -> (Self, SweepEventId) {
        let id = SweepEventId(arena.len());
        let event = Self {
            point,
            left,
            ..Default::default()
        };
        arena.push(event.clone());
        (event, id)
    }

    pub fn is_below(&self, p: &[f64; 2], arena: &SweepEventArena) -> bool {
        let p0 = self.point;
        let p1 = arena[self.other_event.expect("SweepEvent.other_event")].point;
        // TODO: use approx?
        if self.left {
            (p0[0] - p[0]) * (p1[1] - p[1]) - (p1[0] - p[0]) * (p0[1] - p[1]) > 0.0
        } else {
            (p1[0] - p[0]) * (p0[1] - p[1]) - (p0[0] - p[0]) * (p1[1] - p[1]) > 0.0
        }
    }

    pub fn is_above(&self, p: &[f64; 2], arena: &SweepEventArena) -> bool {
        !self.is_below(p, arena)
    }

    pub fn is_vertical(&self, arena: &SweepEventArena) -> bool {
        self.point[0] == arena[self.other_event.expect("SweepEvent.other_event")].point[0]
    }

    pub fn in_result(&self) -> bool {
        self.result_transition != ResultTransitionType::NotInResult
    }
}

#[cfg(test)]
pub(crate) struct SweepEventDeepEqual<'a>(pub SweepEvent, pub &'a SweepEventArena);

#[cfg(test)]
impl graph_safe_compare::Node for SweepEventDeepEqual<'_> {
    type Cmp = bool;
    type Id = usize;
    type Index = usize;

    fn id(&self) -> Self::Id {
        self.0.id.0
    }

    fn get_edge(&self, index: &Self::Index) -> Option<Self> {
        match (self.0.other_event, self.0.prev_in_result) {
            (Some(other_event), None) => match *index {
                0 => Some(Self(self.1[other_event].clone(), self.1)),
                _ => None,
            },
            (None, Some(prev)) => match *index {
                0 => Some(Self(self.1[prev].clone(), self.1)),
                _ => None,
            },
            (Some(other_event), Some(prev)) => match *index {
                0 => Some(Self(self.1[other_event].clone(), self.1)),
                1 => Some(Self(self.1[prev].clone(), self.1)),
                _ => None,
            },
            (None, None) => None,
        }
    }

    fn equiv_modulo_edges(&self, other: &Self) -> Self::Cmp {
        use crate::equals::equals;

        let a = &self.0;
        let b = &other.0;

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
impl PartialEq for SweepEventDeepEqual<'_> {
    fn eq(&self, other: &Self) -> bool {
        graph_safe_compare::robust::equiv(
            Self(self.0.clone(), self.1),
            Self(other.0.clone(), self.1),
        )
    }
}

#[cfg(test)]
impl Eq for SweepEventDeepEqual<'_> {}

#[cfg(test)]
impl core::fmt::Debug for SweepEventDeepEqual<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let debug = SweepEventDebug {
            inner: self.0.clone(),
            visited: Rc::new(RefCell::new(HashSet::new())),
            arena: self.1,
        };
        debug.fmt(f)
    }
}

#[cfg(test)]
struct SweepEventDebug<'a> {
    inner: SweepEvent,
    visited: Rc<RefCell<HashSet<SweepEventId>>>,
    arena: &'a SweepEventArena,
}

#[cfg(test)]
impl SweepEventDebug<'_> {
    fn debug(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.visited.borrow_mut().insert(self.inner.id);

        let selfz = &self.inner;
        let mut debug = f.debug_struct(&format!("SweepEvent({:?})", selfz.id));

        debug.field("point", &selfz.point);
        debug.field("left", &selfz.left);
        if let Some(other_event) = selfz.other_event {
            let id = self.arena[other_event].id;
            if self.visited.borrow().contains(&id) {
                debug.field("other_event", &format!("SweepEvent({:?})", id));
            } else {
                debug.field(
                    "prev_in_result",
                    &SweepEventDebug {
                        inner: self.arena[other_event].clone(),
                        visited: self.visited.clone(),
                        arena: self.arena,
                    },
                );
            }
        }
        debug.field("is_subject", &selfz.is_subject);
        debug.field("edge_type", &selfz.edge_type);
        debug.field("in_out", &selfz.in_out);
        debug.field("other_in_out", &selfz.other_in_out);
        if let Some(prev) = selfz.prev_in_result {
            let id = self.arena[prev].id;
            if self.visited.borrow().contains(&id) {
                debug.field("prev_in_result", &format!("SweepEvent({:?})", id));
            } else {
                debug.field(
                    "prev_in_result",
                    &SweepEventDebug {
                        inner: self.arena[prev].clone(),
                        visited: self.visited.clone(),
                        arena: self.arena,
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
impl core::fmt::Debug for SweepEventDebug<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.debug(f)
    }
}

#[cfg(test)]
#[allow(clippy::bool_assert_comparison)]
mod tests {
    use crate::sweep_event::{SweepEvent, SweepEventArena, SweepEventDeepEqual};

    #[test]
    fn sweep_event_is_below() {
        let mut arena = SweepEventArena::new();
        let (s1, _) = SweepEvent::new(
            [0.0, 0.0],
            true,
            Some(SweepEvent::with_point_and_left([1.0, 1.0], false, &mut arena).1),
            false,
            None,
            &mut arena,
        );
        let (s2, _) = SweepEvent::new(
            [0.0, 1.0],
            false,
            Some(SweepEvent::with_point_and_left([0.0, 0.0], false, &mut arena).1),
            false,
            None,
            &mut arena,
        );

        assert_eq!(s1.is_below(&[0.0, 1.0], &arena), true);
        assert_eq!(s1.is_below(&[1.0, 2.0], &arena), true);
        assert_eq!(s1.is_below(&[0.0, 0.0], &arena), false);
        assert_eq!(s1.is_below(&[5.0, -1.0], &arena), false);

        assert_eq!(s2.is_below(&[0.0, 1.0], &arena), false);
        assert_eq!(s2.is_below(&[1.0, 2.0], &arena), false);
        assert_eq!(s2.is_below(&[0.0, 0.0], &arena), false);
        assert_eq!(s2.is_below(&[5.0, -1.0], &arena), false);
    }

    #[test]
    fn sweep_event_is_above() {
        let mut arena = SweepEventArena::new();
        let (s1, _) = SweepEvent::new(
            [0.0, 0.0],
            true,
            Some(SweepEvent::with_point_and_left([1.0, 1.0], false, &mut arena).1),
            false,
            None,
            &mut arena,
        );
        let (s2, _) = SweepEvent::new(
            [0.0, 1.0],
            false,
            Some(SweepEvent::with_point_and_left([0.0, 0.0], false, &mut arena).1),
            false,
            None,
            &mut arena,
        );

        assert_eq!(s1.is_above(&[0.0, 1.0], &arena), false);
        assert_eq!(s1.is_above(&[1.0, 2.0], &arena), false);
        assert_eq!(s1.is_above(&[0.0, 0.0], &arena), true);
        assert_eq!(s1.is_above(&[5.0, -1.0], &arena), true);

        assert_eq!(s2.is_above(&[0.0, 1.0], &arena), true);
        assert_eq!(s2.is_above(&[1.0, 2.0], &arena), true);
        assert_eq!(s2.is_above(&[0.0, 0.0], &arena), true);
        assert_eq!(s2.is_above(&[5.0, -1.0], &arena), true);
    }

    #[test]
    fn sweep_event_is_vertical() {
        let mut arena = SweepEventArena::new();
        assert_eq!(
            SweepEvent::new(
                [0.0, 0.0],
                true,
                Some(SweepEvent::with_point_and_left([0.0, 1.0], false, &mut arena).1),
                false,
                None,
                &mut arena,
            )
            .0
            .is_vertical(&arena),
            true,
        );
        assert_eq!(
            SweepEvent::new(
                [0.0, 0.0],
                true,
                Some(SweepEvent::with_point_and_left([0.0001, 1.0], false, &mut arena).1),
                false,
                None,
                &mut arena,
            )
            .0
            .is_vertical(&arena),
            false,
        );
    }

    #[test]
    fn sweep_event_deep_equal() {
        let mut arena = SweepEventArena::new();
        let (se1, _) = SweepEvent::new([0.1, 0.1], true, None, true, None, &mut arena);
        let se2 = se1.clone();
        assert_eq!(
            SweepEventDeepEqual(se1, &arena),
            SweepEventDeepEqual(se2, &arena),
            "point to the same memory"
        );

        let (se1, _) = SweepEvent::new([0.1, 0.1], true, None, true, None, &mut arena);
        let (se2, _) = SweepEvent::new([0.1, 0.1], true, None, true, None, &mut arena);
        assert_eq!(
            SweepEventDeepEqual(se1, &arena),
            SweepEventDeepEqual(se2, &arena),
            "different memory, same value and descendants"
        );

        let (mut se1, se1_id) = SweepEvent::new([0.1, 0.1], true, None, true, None, &mut arena);
        let (_, se2_id) = SweepEvent::new([0.1, 0.1], true, Some(se1_id), true, None, &mut arena);
        se1.other_event = Some(se2_id);
        let se3 = se1.clone();
        assert_eq!(
            SweepEventDeepEqual(se1, &arena),
            SweepEventDeepEqual(se3, &arena),
            "same memory, cyclic"
        );

        let (mut se1, se1_id) = SweepEvent::new([0.1, 0.1], true, None, true, None, &mut arena);
        let (_, se2_id) = SweepEvent::new([0.1, 0.1], true, Some(se1_id), true, None, &mut arena);
        se1.other_event = Some(se2_id);
        let (mut se3, se3_id) = SweepEvent::new([0.1, 0.1], true, None, true, None, &mut arena);
        let (_, se4_id) = SweepEvent::new([0.1, 0.1], true, Some(se3_id), true, None, &mut arena);
        se3.other_event = Some(se4_id);
        assert_eq!(
            SweepEventDeepEqual(se1, &arena),
            SweepEventDeepEqual(se3, &arena),
            "different memory, same value and descendants, cyclic"
        );

        let (mut se1, se1_id) = SweepEvent::new([0.1, 0.1], true, None, true, None, &mut arena);
        let (_, se2_id) = SweepEvent::new([0.1, 0.1], true, Some(se1_id), true, None, &mut arena);
        se1.other_event = Some(se2_id);
        let (mut se3, se3_id) = SweepEvent::new([0.1, 0.1], true, None, true, None, &mut arena);
        let (_, se4_id) = SweepEvent::new([0.1, 0.2], true, Some(se3_id), true, None, &mut arena);
        se3.other_event = Some(se4_id);
        assert_ne!(
            SweepEventDeepEqual(se1, &arena),
            SweepEventDeepEqual(se3, &arena),
            "different value, cyclic"
        );
    }
}
