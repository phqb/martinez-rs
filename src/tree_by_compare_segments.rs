#[cfg(feature = "no_splay_tree")]
use crate::sorted_list::{ElementRef, SortedList};
#[cfg(not(feature = "no_splay_tree"))]
use crate::splay_tree::{NodeRef, SplayTree};
use crate::{
    compare_segments::compare_segments,
    sweep_event::{SweepEventArena, SweepEventId},
};

#[cfg(feature = "no_splay_tree")]
pub(crate) struct TreeByCompareSegments(SortedList<SweepEventId>);

#[cfg(feature = "no_splay_tree")]
impl TreeByCompareSegments {
    pub fn new() -> Self {
        Self(SortedList::new())
    }

    pub fn insert(
        &mut self,
        se: SweepEventId,
        arena: &SweepEventArena,
    ) -> ElementRef<SweepEventId> {
        self.0
            .insert(se, |a, b| compare_segments(&arena[*a], &arena[*b], arena))
    }

    pub fn remove(&mut self, e: ElementRef<SweepEventId>) {
        self.0.remove(e);
    }

    pub fn find(
        &mut self,
        se: SweepEventId,
        arena: &SweepEventArena,
    ) -> Option<ElementRef<SweepEventId>> {
        self.0
            .find(&se, |a, b| compare_segments(&arena[*a], &arena[*b], arena))
    }

    pub fn next(&self, e: &ElementRef<SweepEventId>) -> Option<ElementRef<SweepEventId>> {
        self.0.next(e)
    }

    pub fn prev(&self, e: &ElementRef<SweepEventId>) -> Option<ElementRef<SweepEventId>> {
        self.0.prev(e)
    }

    #[allow(dead_code)]
    pub fn min(&mut self) -> Option<ElementRef<SweepEventId>> {
        self.0.min()
    }

    #[allow(dead_code)]
    pub fn max(&mut self) -> Option<ElementRef<SweepEventId>> {
        self.0.max()
    }
}

#[derive(Default)]
#[cfg(not(feature = "no_splay_tree"))]
pub(crate) struct TreeByCompareSegments(SplayTree<SweepEventId>);

#[cfg(not(feature = "no_splay_tree"))]
impl TreeByCompareSegments {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self(SplayTree::new())
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn insert(&mut self, se: SweepEventId, arena: &SweepEventArena) -> NodeRef<SweepEventId> {
        self.0
            .insert(se, |a, b| compare_segments(&arena[*a], &arena[*b], arena))
    }

    pub fn remove(&mut self, node_ref: NodeRef<SweepEventId>) {
        self.0.remove(node_ref);
    }

    pub fn find(
        &mut self,
        se: SweepEventId,
        arena: &SweepEventArena,
    ) -> Option<NodeRef<SweepEventId>> {
        self.0
            .find(&se, |a, b| compare_segments(&arena[*a], &arena[*b], arena))
    }

    pub fn next(&self, node_ref: &NodeRef<SweepEventId>) -> Option<NodeRef<SweepEventId>> {
        self.0.next(node_ref)
    }

    pub fn prev(&self, node_ref: &NodeRef<SweepEventId>) -> Option<NodeRef<SweepEventId>> {
        self.0.prev(node_ref)
    }

    #[allow(dead_code)]
    pub fn min(&mut self) -> Option<NodeRef<SweepEventId>> {
        self.0.min()
    }

    #[allow(dead_code)]
    pub fn max(&mut self) -> Option<NodeRef<SweepEventId>> {
        self.0.max()
    }
}
