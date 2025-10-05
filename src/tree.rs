use core::cmp::Ordering;

use crate::{
    compare_segments::compare_segments,
    sweep_event::{SweepEventArena, SweepEventId},
};

#[derive(Clone, Copy)]
pub(crate) struct Node {
    pub value: SweepEventId,
    index: usize,
}

pub(crate) struct TreeByCompareSegments(Vec<SweepEventId>);

impl TreeByCompareSegments {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn insert(&mut self, se: SweepEventId, arena: &SweepEventArena) -> Node {
        let index = self
            .0
            .binary_search_by(|e_id| compare_segments(&arena[*e_id], &arena[se], arena))
            .unwrap_or_else(|e| e);
        self.0.insert(index, se);
        Node { value: se, index }
    }

    pub fn remove(&mut self, node_id: Node) {
        self.0.remove(node_id.index);
    }

    pub fn find(&self, se: SweepEventId, arena: &SweepEventArena) -> Option<Node> {
        self.0
            .iter()
            .position(|e_id| compare_segments(&arena[*e_id], &arena[se], arena) == Ordering::Equal)
            .map(|index| Node { value: se, index })
    }

    pub fn next(&self, node: Node) -> Option<Node> {
        if node.index + 1 < self.0.len() {
            Some(Node {
                value: self.0[node.index + 1],
                index: node.index + 1,
            })
        } else {
            None
        }
    }

    pub fn prev(&self, node: Node) -> Option<Node> {
        if node.index > 0 {
            Some(Node {
                value: self.0[node.index - 1],
                index: node.index - 1,
            })
        } else {
            None
        }
    }

    #[cfg(test)]
    pub fn min(&self, _: &SweepEventArena) -> Option<SweepEventId> {
        self.0.first().cloned()
    }

    #[cfg(test)]
    pub fn max(&self, _: &SweepEventArena) -> Option<SweepEventId> {
        self.0.last().cloned()
    }
}
