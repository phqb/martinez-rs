use core::cmp::Ordering;

pub(crate) struct ElementRef<T> {
    pub value: T,
    id: NodeId,
}

type NodeId = usize;

pub struct Node {
    prev: Option<NodeId>,
    next: Option<NodeId>,
}

pub(crate) struct SortedList<T> {
    node_arena: Vec<Node>,
    value_arena: Vec<T>,
    head: Option<NodeId>,
}

impl<T> SortedList<T> {
    pub fn new() -> Self {
        Self {
            node_arena: vec![],
            value_arena: vec![],
            head: None,
        }
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        let mut n = 0;
        let mut itr = self.head;
        while let Some(cur) = itr {
            n += 1;
            itr = self.node_arena[cur].next;
        }
        n
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }
}

#[allow(clippy::type_complexity)]
impl<T: Clone> SortedList<T> {
    pub fn find<'a, Cmp: Fn(&T, &T) -> Ordering + 'a>(
        &mut self,
        value: &T,
        cmp: Cmp,
    ) -> Option<ElementRef<T>> {
        let mut itr = self.head;
        while let Some(cur) = itr {
            if cmp(&self.value_arena[cur], value) == Ordering::Equal {
                return Some(ElementRef {
                    value: self.value_arena[cur].clone(),
                    id: cur,
                });
            }
            itr = self.node_arena[cur].next;
        }
        None
    }

    pub fn insert<'a, Cmp: Fn(&T, &T) -> Ordering + 'a>(
        &mut self,
        value: T,
        cmp: Cmp,
    ) -> ElementRef<T> {
        let mut prev = None;
        let mut itr = self.head;
        while let Some(cur) = itr {
            if cmp(&self.value_arena[cur], &value) <= Ordering::Equal {
                prev = Some(cur);
                itr = self.node_arena[cur].next;
            } else {
                break;
            }
        }

        let id = self.node_arena.len();
        let cloned = value.clone();
        self.value_arena.push(value);
        self.node_arena.push(Node { prev, next: itr });

        if let Some(prev) = prev {
            self.node_arena[prev].next = Some(id);
        } else {
            self.head = Some(id);
        }

        if let Some(itr) = itr {
            self.node_arena[itr].prev = Some(id);
        }

        ElementRef { value: cloned, id }
    }

    pub fn remove(&mut self, e: ElementRef<T>) {
        let prev_id = self.node_arena[e.id].prev;
        let next_id = self.node_arena[e.id].next;

        if let Some(p_id) = prev_id {
            self.node_arena[p_id].next = next_id;
        } else {
            self.head = next_id;
        }

        if let Some(n_id) = next_id {
            self.node_arena[n_id].prev = prev_id;
        }
    }

    pub fn next(&self, e: &ElementRef<T>) -> Option<ElementRef<T>> {
        self.node_arena[e.id].next.map(|id| ElementRef {
            value: self.value_arena[id].clone(),
            id,
        })
    }

    pub fn prev(&self, e: &ElementRef<T>) -> Option<ElementRef<T>> {
        self.node_arena[e.id].prev.map(|id| ElementRef {
            value: self.value_arena[id].clone(),
            id,
        })
    }

    #[allow(dead_code)]
    pub fn min(&self) -> Option<ElementRef<T>> {
        self.head.map(|id| ElementRef {
            value: self.value_arena[id].clone(),
            id,
        })
    }

    #[allow(dead_code)]
    pub fn max(&self) -> Option<ElementRef<T>> {
        let mut itr = self.head;
        while let Some(cur) = itr {
            if let Some(next) = self.node_arena[cur].next {
                itr = Some(next);
            } else {
                return Some(ElementRef {
                    value: self.value_arena[cur].clone(),
                    id: cur,
                });
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to get the list's contents by traversing forward.
    fn collect_forward(list: &SortedList<i32>) -> Vec<i32> {
        let mut vec = Vec::new();
        let mut itr = list.head;
        while let Some(id) = itr {
            vec.push(list.value_arena[id]);
            itr = list.node_arena[id].next;
        }
        vec
    }

    // Helper function to get the list's contents by traversing backward.
    // Crucial for verifying the integrity of `prev` links.
    fn collect_backward(list: &SortedList<i32>) -> Vec<i32> {
        let mut vec = Vec::new();
        let mut itr = list.head;
        while let Some(cur) = itr {
            if let Some(next) = list.node_arena[cur].next {
                itr = Some(next);
            } else {
                break;
            }
        }
        while let Some(id) = itr {
            vec.push(list.value_arena[id]);
            itr = list.node_arena[id].prev;
        }
        vec
    }

    #[test]
    fn test_new_list_is_empty() {
        let list = SortedList::<i32>::new();
        assert!(list.head.is_none());
        assert!(list.min().is_none());
        assert!(list.max().is_none());
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_insert_into_empty_list() {
        let mut list = SortedList::new();
        list.insert(10, |a, b| a.cmp(b));

        assert_eq!(list.head, Some(0));
        assert_eq!(collect_forward(&list), vec![10]);
        assert_eq!(collect_backward(&list), vec![10]);
        assert_eq!(list.min().unwrap().value, 10);
        assert_eq!(list.max().unwrap().value, 10);
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn test_insert_multiple_sorted() {
        let mut list = SortedList::new();
        list.insert(10, |a, b| a.cmp(b));
        list.insert(20, |a, b| a.cmp(b));
        list.insert(30, |a, b| a.cmp(b));

        let expected = vec![10, 20, 30];
        assert_eq!(collect_forward(&list), expected);
        assert_eq!(
            collect_backward(&list),
            expected.iter().rev().cloned().collect::<Vec<_>>()
        );
        assert_eq!(list.min().unwrap().value, 10);
        assert_eq!(list.max().unwrap().value, 30);
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn test_insert_multiple_reverse_sorted() {
        let mut list = SortedList::new();
        list.insert(30, |a, b| a.cmp(b));
        list.insert(20, |a, b| a.cmp(b));
        list.insert(10, |a, b| a.cmp(b));

        let expected = vec![10, 20, 30];
        assert_eq!(collect_forward(&list), expected);
        assert_eq!(
            collect_backward(&list),
            expected.iter().rev().cloned().collect::<Vec<_>>()
        );
        assert_eq!(list.min().unwrap().value, 10);
        assert_eq!(list.max().unwrap().value, 30);
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn test_insert_multiple_random_order() {
        let mut list = SortedList::new();
        list.insert(20, |a, b| a.cmp(b));
        list.insert(40, |a, b| a.cmp(b));
        list.insert(10, |a, b| a.cmp(b));
        list.insert(30, |a, b| a.cmp(b));

        let expected = vec![10, 20, 30, 40];
        assert_eq!(collect_forward(&list), expected);
        assert_eq!(
            collect_backward(&list),
            expected.iter().rev().cloned().collect::<Vec<_>>()
        );
        assert_eq!(list.head.map(|id| list.value_arena[id]), Some(10));
        assert_eq!(list.len(), 4);
    }

    #[test]
    fn test_remove_single_element() {
        let mut list = SortedList::new();
        let el = list.insert(10, |a, b| a.cmp(b));
        list.remove(el);

        assert!(list.head.is_none());
        assert!(collect_forward(&list).is_empty());
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_remove_head() {
        let mut list = SortedList::new();
        let el_10 = list.insert(10, |a, b| a.cmp(b));
        list.insert(20, |a, b| a.cmp(b));
        list.insert(30, |a, b| a.cmp(b));

        list.remove(el_10);

        let expected = vec![20, 30];
        assert_eq!(collect_forward(&list), expected);
        assert_eq!(
            collect_backward(&list),
            expected.iter().rev().cloned().collect::<Vec<_>>()
        );
        assert_eq!(list.min().unwrap().value, 20);
        assert_eq!(list.max().unwrap().value, 30);
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_remove_tail() {
        let mut list = SortedList::new();
        list.insert(10, |a, b| a.cmp(b));
        list.insert(20, |a, b| a.cmp(b));
        let el_30 = list.insert(30, |a, b| a.cmp(b));

        list.remove(el_30);

        let expected = vec![10, 20];
        assert_eq!(collect_forward(&list), expected);
        assert_eq!(
            collect_backward(&list),
            expected.iter().rev().cloned().collect::<Vec<_>>()
        );
        assert_eq!(list.min().unwrap().value, 10);
        assert_eq!(list.max().unwrap().value, 20);
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_remove_middle() {
        let mut list = SortedList::new();
        list.insert(10, |a, b| a.cmp(b));
        let el_20 = list.insert(20, |a, b| a.cmp(b));
        list.insert(30, |a, b| a.cmp(b));

        list.remove(el_20);

        let expected = vec![10, 30];
        assert_eq!(collect_forward(&list), expected);
        assert_eq!(
            collect_backward(&list),
            expected.iter().rev().cloned().collect::<Vec<_>>()
        );
        assert_eq!(list.min().unwrap().value, 10);
        assert_eq!(list.max().unwrap().value, 30);
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_comprehensive_integration() {
        let mut list = SortedList::new();

        // Insert some values
        let el_20 = list.insert(20, |a, b| a.cmp(b));
        list.insert(10, |a, b| a.cmp(b));
        list.insert(40, |a, b| a.cmp(b));
        assert_eq!(collect_forward(&list), vec![10, 20, 40]);
        assert_eq!(list.len(), 3);

        // Remove from the middle
        list.remove(el_20);
        assert_eq!(collect_forward(&list), vec![10, 40]);
        assert_eq!(collect_backward(&list), vec![40, 10]);
        assert_eq!(list.min().unwrap().value, 10);
        assert_eq!(list.max().unwrap().value, 40);
        assert_eq!(list.len(), 2);

        // Add a new min and max
        let el_5 = list.insert(5, |a, b| a.cmp(b));
        let el_50 = list.insert(50, |a, b| a.cmp(b));
        assert_eq!(collect_forward(&list), vec![5, 10, 40, 50]);
        assert_eq!(list.len(), 4);

        // Remove the new head and tail
        list.remove(el_5);
        list.remove(el_50);
        assert_eq!(collect_forward(&list), vec![10, 40]);
        assert_eq!(list.head.map(|id| list.value_arena[id]), Some(10));
        assert_eq!(list.len(), 2);

        // Remove the rest
        list.remove(list.min().unwrap());
        list.remove(list.max().unwrap());
        assert!(list.head.is_none());
        assert_eq!(list.len(), 0);
    }
}
