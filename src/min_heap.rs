use core::cmp::Ordering;

use crate::{
    compare_events::compare_events,
    sweep_event::{SweepEventArena, SweepEventId},
};

pub(crate) struct MinHeap<T> {
    data: Vec<T>,
}

impl<T> MinHeap<T> {
    pub fn new() -> Self {
        Self { data: vec![] }
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn push<'a, Cmp: Fn(&T, &T) -> Ordering + 'a>(&mut self, item: T, cmp: Cmp) {
        self.data.push(item);
        self.sift_up(self.data.len() - 1, cmp);
    }

    pub fn pop<'a, Cmp: Fn(&T, &T) -> Ordering + 'a>(&mut self, cmp: Cmp) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let last_index = self.data.len() - 1;
        self.data.swap(0, last_index);
        let min_item = self.data.pop();

        if !self.is_empty() {
            self.sift_down(0, cmp);
        }

        min_item
    }

    #[allow(dead_code)]
    pub fn peek(&self) -> Option<&T> {
        self.data.first()
    }

    fn sift_up<'a, Cmp: Fn(&T, &T) -> Ordering + 'a>(&mut self, mut index: usize, cmp: Cmp) {
        while index > 0 {
            let parent_index = Self::parent_index(index);
            // We use `partial_cmp` and `unwrap` because `PartialOrd` guarantees
            // that we can compare the items.
            if cmp(&self.data[index], &self.data[parent_index]) == Ordering::Less {
                self.data.swap(index, parent_index);
                index = parent_index;
            } else {
                // Heap property is satisfied, we can stop.
                break;
            }
        }
    }

    fn sift_down<'a, Cmp: Fn(&T, &T) -> Ordering + 'a>(&mut self, mut index: usize, cmp: Cmp) {
        let last_index = self.data.len() - 1;

        loop {
            let left_child_index = Self::left_child_index(index);
            let right_child_index = Self::right_child_index(index);
            let mut smallest_index = index;

            // Find the index of the smallest element among the current node and its children.
            if left_child_index <= last_index
                && cmp(&self.data[left_child_index], &self.data[smallest_index]) == Ordering::Less
            {
                smallest_index = left_child_index;
            }
            if right_child_index <= last_index
                && cmp(&self.data[right_child_index], &self.data[smallest_index]) == Ordering::Less
            {
                smallest_index = right_child_index;
            }

            // If the smallest element is not the current one, swap them and continue.
            if smallest_index != index {
                self.data.swap(index, smallest_index);
                index = smallest_index;
            } else {
                // The current node is smaller than its children, heap property is restored.
                break;
            }
        }
    }

    #[inline]
    fn parent_index(index: usize) -> usize {
        (index - 1) / 2
    }

    #[inline]
    fn left_child_index(index: usize) -> usize {
        2 * index + 1
    }

    #[inline]
    fn right_child_index(index: usize) -> usize {
        2 * index + 2
    }
}

pub(crate) struct MinHeapByCompareEvents(MinHeap<SweepEventId>);

impl MinHeapByCompareEvents {
    pub fn new() -> Self {
        Self(MinHeap::new())
    }

    pub fn push(&mut self, id: SweepEventId, arena: &SweepEventArena) {
        self.0.push(id, |a_id, b_id| {
            compare_events(&arena[*a_id], &arena[*b_id], arena)
        });
    }

    pub fn pop(&mut self, arena: &SweepEventArena) -> Option<SweepEventId> {
        self.0
            .pop(|a_id, b_id| compare_events(&arena[*a_id], &arena[*b_id], arena))
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::min_heap::MinHeap;

    #[test]
    fn min_heap() {
        let mut h = MinHeap::<u64>::new();
        let cmp = |a: &u64, b: &u64| a.cmp(b);
        assert!(h.is_empty());
        assert_eq!(h.peek(), None);

        h.push(10, cmp);
        h.push(4, cmp);
        h.push(15, cmp);
        h.push(2, cmp);
        h.push(20, cmp);
        h.push(1, cmp);

        assert_eq!(h.len(), 6);
        assert_eq!(h.peek(), Some(&1));

        let mut actual = vec![];
        while let Some(item) = h.pop(cmp) {
            actual.push(item);
        }
        assert_eq!(actual, vec![1, 2, 4, 10, 15, 20]);
    }
}
