use std::ops::Deref;

use crate::sweep_event::SweepEventOrderedByCompareSegment;

pub(crate) struct SweepEventTree(pub Vec<SweepEventOrderedByCompareSegment>);

impl SweepEventTree {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn insert(&mut self, x: SweepEventOrderedByCompareSegment) -> usize {
        let index = self.0.binary_search(&x).unwrap_or_else(|e| e);
        self.0.insert(index, x);
        index
    }

    pub fn remove(&mut self, index: usize) {
        self.0.remove(index);
    }

    #[cfg(test)]
    pub fn find(
        &self,
        x: &SweepEventOrderedByCompareSegment,
    ) -> Option<SweepEventOrderedByCompareSegment> {
        self.0.iter().find(|e| *e == x).cloned()
    }

    #[cfg(test)]
    pub fn next(
        &self,
        x: &SweepEventOrderedByCompareSegment,
    ) -> Option<SweepEventOrderedByCompareSegment> {
        self.0.iter().position(|e| e == x).and_then(|i| {
            if i + 1 < self.0.len() {
                Some(self.0[i + 1].clone())
            } else {
                None
            }
        })
    }

    #[cfg(test)]
    pub fn prev(
        &self,
        x: &SweepEventOrderedByCompareSegment,
    ) -> Option<SweepEventOrderedByCompareSegment> {
        self.0.iter().position(|e| e == x).and_then(|i| {
            if i > 0 {
                Some(self.0[i - 1].clone())
            } else {
                None
            }
        })
    }

    #[cfg(test)]
    pub fn min(&self) -> Option<SweepEventOrderedByCompareSegment> {
        self.0.first().cloned()
    }

    #[cfg(test)]
    pub fn max(&self) -> Option<SweepEventOrderedByCompareSegment> {
        self.0.last().cloned()
    }
}

impl Deref for SweepEventTree {
    type Target = Vec<SweepEventOrderedByCompareSegment>;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}