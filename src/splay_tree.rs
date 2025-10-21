use core::cmp::Ordering;

#[cfg_attr(test, derive(PartialEq, Eq, Debug))]
pub(crate) struct NodeRef<T> {
    pub value: T,
    id: NodeId,
}

type NodeId = usize;

#[cfg_attr(test, derive(PartialEq, Eq, Debug))]
pub(crate) struct SplayTreeNode {
    parent: Option<NodeId>,
    left: Option<NodeId>,
    right: Option<NodeId>,
}

impl SplayTreeNode {
    pub fn new(parent: Option<NodeId>, left: Option<NodeId>, right: Option<NodeId>) -> Self {
        Self {
            parent,
            left,
            right,
        }
    }

    #[cfg(test)]
    pub fn root(left: Option<NodeId>, right: Option<NodeId>) -> Self {
        Self {
            parent: None,
            left,
            right,
        }
    }

    pub fn root_no_child() -> Self {
        Self {
            parent: None,
            left: None,
            right: None,
        }
    }

    #[cfg(test)]
    pub fn dangling() -> Self {
        Self {
            parent: None,
            left: None,
            right: None,
        }
    }

    pub fn leaf_with_parent(parent: NodeId) -> Self {
        Self {
            parent: Some(parent),
            left: None,
            right: None,
        }
    }
}

#[derive(Default)]
#[cfg_attr(test, derive(PartialEq, Eq, Debug))]
pub(crate) struct SplayTree<T> {
    node_arena: Vec<SplayTreeNode>,
    value_arena: Vec<T>,
    root: Option<NodeId>,
}

impl<T: Clone> SplayTree<T> {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            node_arena: vec![],
            value_arena: vec![],
            root: None,
        }
    }

    pub fn clear(&mut self) {
        self.node_arena.clear();
        self.value_arena.clear();
        self.root = None;
    }

    fn len_inner(&self, id: Option<NodeId>) -> usize {
        if let Some(id) = id {
            self.len_inner(self.node_arena[id].left) + self.len_inner(self.node_arena[id].right) + 1
        } else {
            0
        }
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.len_inner(self.root)
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    #[cfg(test)]
    pub fn to_vec(&self) -> Vec<T> {
        let mut result = Vec::new();
        if let Some(root_id) = self.root {
            self.to_vec_inner(Some(root_id), &mut result);
        }
        result
    }

    #[cfg(test)]
    fn to_vec_inner(&self, node_id: Option<NodeId>, result: &mut Vec<T>) {
        if let Some(id) = node_id {
            // Traverse left subtree
            self.to_vec_inner(self.node_arena[id].left, result);

            // Visit the current node
            result.push(self.value_arena[id].clone());

            // Traverse right subtree
            self.to_vec_inner(self.node_arena[id].right, result);
        }
    }

    #[allow(dead_code)]
    pub fn min(&self) -> Option<NodeRef<T>> {
        let mut cur = self.root?;
        loop {
            if let Some(left) = self.node_arena[cur].left {
                cur = left;
            } else {
                return Some(NodeRef {
                    value: self.value_arena[cur].clone(),
                    id: cur,
                });
            }
        }
    }

    #[allow(dead_code)]
    pub fn max(&self) -> Option<NodeRef<T>> {
        let mut cur = self.root?;
        loop {
            if let Some(right) = self.node_arena[cur].right {
                cur = right;
            } else {
                return Some(NodeRef {
                    value: self.value_arena[cur].clone(),
                    id: cur,
                });
            }
        }
    }

    pub fn prev(&self, node_ref: &NodeRef<T>) -> Option<NodeRef<T>> {
        if let Some(mut predecessor) = self.node_arena[node_ref.id].left {
            while let Some(right) = self.node_arena[predecessor].right {
                predecessor = right;
            }
            Some(NodeRef {
                value: self.value_arena[predecessor].clone(),
                id: predecessor,
            })
        } else {
            let mut child_id = node_ref.id;
            while let Some(parent) = self.node_arena[child_id].parent {
                if self.node_arena[parent].right == Some(child_id) {
                    return Some(NodeRef {
                        value: self.value_arena[parent].clone(),
                        id: parent,
                    });
                }
                child_id = parent;
            }
            None
        }
    }

    pub fn next(&self, node_ref: &NodeRef<T>) -> Option<NodeRef<T>> {
        if let Some(mut successor) = self.node_arena[node_ref.id].right {
            while let Some(left) = self.node_arena[successor].left {
                successor = left;
            }
            Some(NodeRef {
                value: self.value_arena[successor].clone(),
                id: successor,
            })
        } else {
            let mut child_id = node_ref.id;
            while let Some(parent) = self.node_arena[child_id].parent {
                if self.node_arena[parent].left == Some(child_id) {
                    return Some(NodeRef {
                        value: self.value_arena[parent].clone(),
                        id: parent,
                    });
                }
                child_id = parent;
            }
            None
        }
    }

    pub fn find_static<'a, Cmp: Fn(&T, &T) -> Ordering + 'a>(
        &self,
        value: &T,
        cmp: Cmp,
    ) -> Option<NodeRef<T>> {
        let mut cur = self.root?;
        loop {
            match cmp(value, &self.value_arena[cur]) {
                Ordering::Less => match self.node_arena[cur].left {
                    Some(id) => cur = id,
                    None => return None,
                },
                Ordering::Equal => {
                    return Some(NodeRef {
                        value: self.value_arena[cur].clone(),
                        id: cur,
                    });
                }
                Ordering::Greater => match self.node_arena[cur].right {
                    Some(id) => cur = id,
                    None => return None,
                },
            }
        }
    }

    pub fn find<'a, Cmp: Fn(&T, &T) -> Ordering + 'a>(
        &mut self,
        value: &T,
        cmp: Cmp,
    ) -> Option<NodeRef<T>> {
        if let Some(node) = self.find_static(value, cmp) {
            self.splay(node.id);
            Some(node)
        } else {
            None
        }
    }

    pub fn insert<'a, Cmp: Fn(&T, &T) -> Ordering + 'a>(
        &mut self,
        value: T,
        cmp: Cmp,
    ) -> NodeRef<T> {
        if self.root.is_none() {
            let id = self.node_arena.len();
            self.value_arena.push(value);
            self.node_arena.push(SplayTreeNode::root_no_child());
            self.root = Some(id);
            return NodeRef {
                value: self.value_arena[id].clone(),
                id,
            };
        }

        let mut cur = self.root.unwrap();
        let insert_left;
        loop {
            match cmp(&value, &self.value_arena[cur]) {
                Ordering::Less => match self.node_arena[cur].left {
                    Some(id) => cur = id,
                    None => {
                        insert_left = true;
                        break;
                    }
                },
                Ordering::Equal | Ordering::Greater => match self.node_arena[cur].right {
                    Some(id) => cur = id,
                    None => {
                        insert_left = false;
                        break;
                    }
                },
            }
        }

        let id = self.node_arena.len();
        self.value_arena.push(value);
        self.node_arena.push(SplayTreeNode::leaf_with_parent(cur));
        if insert_left {
            self.node_arena[cur].left = Some(id);
        } else {
            self.node_arena[cur].right = Some(id);
        }

        self.splay(id);

        NodeRef {
            value: self.value_arena[id].clone(),
            id,
        }
    }

    fn remove_node_has_zero_children(&mut self, node_id: NodeId) {
        let node = &self.node_arena[node_id];

        debug_assert!(node.left.is_none() && node.right.is_none());

        if let Some(parent) = node.parent {
            if self.node_arena[parent].left == Some(node_id) {
                self.node_arena[parent].left = None;
            } else if self.node_arena[parent].right == Some(node_id) {
                self.node_arena[parent].right = None;
            } else {
                unreachable!("incorrect state: node's parent does not have node as child");
            }
        } else {
            self.root = None;
        }

        self.node_arena[node_id].parent = None;
    }

    fn remove_node_has_one_children(&mut self, node_id: NodeId) {
        let node = SplayTreeNode::new(
            self.node_arena[node_id].parent,
            self.node_arena[node_id].left,
            self.node_arena[node_id].right,
        );

        debug_assert!(node.left.is_some() ^ node.right.is_some());

        let child = node
            .left
            .or(node.right)
            .expect("either left or right child");

        self.node_arena[child].parent = node.parent;
        self.node_arena[node_id].left = None;
        self.node_arena[node_id].right = None;
        self.node_arena[node_id].parent = None;

        if let Some(parent) = node.parent {
            if self.node_arena[parent].left == Some(node_id) {
                self.node_arena[parent].left = Some(child);
            } else if self.node_arena[parent].right == Some(node_id) {
                self.node_arena[parent].right = Some(child);
            } else {
                unreachable!("incorrect state: node's parent does not have node as child");
            }
        } else {
            self.root = Some(child);
        }
    }

    fn remove_node_has_two_children(&mut self, node_id: NodeId) -> Option<NodeId> {
        let node = SplayTreeNode::new(
            self.node_arena[node_id].parent,
            self.node_arena[node_id].left,
            self.node_arena[node_id].right,
        );

        debug_assert!(node.left.is_some() && node.right.is_some());

        let mut predecessor = node.left.expect("left child");
        while let Some(right) = self.node_arena[predecessor].right {
            predecessor = right;
        }

        let mut to_splay = None;

        // if the right most node is the left child of the node itself
        if predecessor == node.left.unwrap() {
            self.node_arena[predecessor].parent = node.parent;
            self.node_arena[predecessor].right = node.right;

            if let Some(parent) = node.parent {
                if self.node_arena[parent].left == Some(node_id) {
                    self.node_arena[parent].left = Some(predecessor);
                } else if self.node_arena[parent].right == Some(node_id) {
                    self.node_arena[parent].right = Some(predecessor);
                } else {
                    unreachable!("incorrect state: node's parent does not have node as child");
                }

                to_splay = Some(predecessor);
            } else {
                self.root = Some(predecessor);
            }

            if let Some(right) = node.right {
                self.node_arena[right].parent = Some(predecessor);
            }
        } else {
            let predecessor_parent = self.node_arena[predecessor].parent.unwrap();
            let predecessor_left = self.node_arena[predecessor].left;

            self.node_arena[predecessor_parent].right = predecessor_left;
            if let Some(id) = predecessor_left {
                self.node_arena[id].parent = Some(predecessor_parent);
            }

            self.node_arena[predecessor].parent = node.parent;
            self.node_arena[predecessor].left = node.left;
            self.node_arena[predecessor].right = node.right;

            if let Some(parent) = node.parent {
                if self.node_arena[parent].left == Some(node_id) {
                    self.node_arena[parent].left = Some(predecessor);
                } else if self.node_arena[parent].right == Some(node_id) {
                    self.node_arena[parent].right = Some(predecessor);
                } else {
                    unreachable!("incorrect state: node's parent does not have node as child");
                }
            } else {
                self.root = Some(predecessor);
            }

            if let Some(left) = node.left {
                self.node_arena[left].parent = Some(predecessor);
            }
            if let Some(right) = node.right {
                self.node_arena[right].parent = Some(predecessor);
            }

            to_splay = Some(predecessor_parent);
        }

        self.node_arena[node_id].parent = None;
        self.node_arena[node_id].left = None;
        self.node_arena[node_id].right = None;

        to_splay
    }

    pub fn remove(&mut self, node_ref: NodeRef<T>) {
        let (left, right) = (
            self.node_arena[node_ref.id].left,
            self.node_arena[node_ref.id].right,
        );
        match (left.is_some(), right.is_some()) {
            (false, false) => {
                self.remove_node_has_zero_children(node_ref.id);
                if let Some(parent) = self.node_arena[node_ref.id].parent {
                    self.splay(parent);
                }
            }
            (false, true) | (true, false) => {
                self.remove_node_has_one_children(node_ref.id);
                if let Some(parent) = self.node_arena[node_ref.id].parent {
                    self.splay(parent);
                }
            }
            (true, true) => {
                if let Some(splay_node) = self.remove_node_has_two_children(node_ref.id) {
                    self.splay(splay_node);
                }
            }
        }
    }
}

impl<T> SplayTree<T> {
    //       p                 x
    //      / \               / \
    //     /   \             /   \
    //    x     C    ==>    A     p
    //   / \                     / \
    //  /   \                   /   \
    // A     B                 B     C
    fn zig_left_to_right(&mut self, x: NodeId, p: NodeId) {
        // p must be root
        debug_assert_eq!(self.node_arena[p].parent, None);
        // x must be left child of p
        debug_assert_eq!(self.node_arena[p].left, Some(x));

        let b = self.node_arena[x].right;
        self.node_arena[x].right = Some(p);
        self.node_arena[p].left = b;

        // Update parent pointers
        self.node_arena[p].parent = Some(x);
        if let Some(b) = b {
            self.node_arena[b].parent = Some(p);
        }
        self.node_arena[x].parent = None;

        self.root = Some(x);
    }

    //    p                     x
    //   / \                   / \
    //  /   \                 /   \
    // A     x      ==>      p     C
    //      / \             / \
    //     /   \           /   \
    //    B     C         A     B
    fn zig_right_to_left(&mut self, x: NodeId, p: NodeId) {
        // p must be root
        debug_assert_eq!(self.node_arena[p].parent, None);
        // x must be right child of p
        debug_assert_eq!(self.node_arena[p].right, Some(x));

        let b = self.node_arena[x].left;
        self.node_arena[x].left = Some(p);
        self.node_arena[p].right = b;

        // Update parent pointers
        self.node_arena[p].parent = Some(x);
        if let Some(b) = b {
            self.node_arena[b].parent = Some(p);
        }
        self.node_arena[x].parent = None;

        self.root = Some(x);
    }

    //        g           x
    //       / \         / \
    //      p   D       A   p
    //     / \             / \
    //    x   C    ==>    B   g
    //   / \                 / \
    //  A   B               C   D
    fn zig_zig_left_to_right(&mut self, x: NodeId, p: NodeId, g: NodeId) {
        // p must NOT be root
        debug_assert!(self.node_arena[p].parent.is_some());
        // x must be left child of p
        debug_assert_eq!(self.node_arena[p].left, Some(x));
        // p must be left child of g
        debug_assert_eq!(self.node_arena[g].left, Some(p));

        // Store grandparent's parent for later update
        let gg = self.node_arena[g].parent;
        let b = self.node_arena[x].right;
        let c = self.node_arena[p].right;

        // First rotation (p becomes the left child of g, x becomes the left child of p)
        // This is equivalent to a zig_left_to_right on (p, g)
        self.node_arena[p].right = Some(g);
        self.node_arena[g].left = c;

        // Second rotation (x becomes the new root of this subtree)
        // This is equivalent to a zig_left_to_right on (x, p)
        self.node_arena[x].right = Some(p);
        self.node_arena[p].left = b;

        // Update parent pointers

        // The new root of the subtree is x
        self.node_arena[x].parent = gg;

        // p is now the right child of x
        self.node_arena[p].parent = Some(x);

        // g is now the right child of p
        self.node_arena[g].parent = Some(p);

        // Update parent pointers of children
        if let Some(b) = b {
            self.node_arena[b].parent = Some(p);
        }
        if let Some(c) = c {
            self.node_arena[c].parent = Some(g);
        }

        // Connect the new subtree to the rest of the tree
        if let Some(gg) = gg {
            // g was either left or right child of gg
            let gg_node = &mut self.node_arena[gg];
            if gg_node.left == Some(g) {
                gg_node.left = Some(x);
            } else {
                debug_assert_eq!(gg_node.right, Some(g));
                gg_node.right = Some(x);
            }
        } else {
            // g was the root, so x is now the new root
            self.root = Some(x);
        }
    }

    //   g                 x
    //  / \               / \
    // A   p             p   D
    //    / \    ==>    / \
    //   B   x         g   C
    //      / \       / \
    //     C   D     A   B
    fn zig_zig_right_to_left(&mut self, x: NodeId, p: NodeId, g: NodeId) {
        // p must NOT be root
        debug_assert!(self.node_arena[p].parent.is_some());
        // x must be right child of p
        debug_assert_eq!(self.node_arena[p].right, Some(x));
        // p must be right child of g
        debug_assert_eq!(self.node_arena[g].right, Some(p));

        // Store grandparent's parent for later update
        let gg = self.node_arena[g].parent;
        let b = self.node_arena[x].left;
        let c = self.node_arena[p].left;

        // First rotation (p becomes the right child of g, x becomes the right child of p)
        // This is equivalent to a zig_right_to_left on (p, g)
        self.node_arena[p].left = Some(g);
        self.node_arena[g].right = c;

        // Second rotation (x becomes the new root of this subtree)
        // This is equivalent to a zig_right_to_left on (x, p)
        self.node_arena[x].left = Some(p);
        self.node_arena[p].right = b;

        // Update parent pointers

        // The new root of the subtree is x
        self.node_arena[x].parent = gg;

        // p is now the left child of x
        self.node_arena[p].parent = Some(x);

        // g is now the left child of p
        self.node_arena[g].parent = Some(p);

        // Update parent pointers of children
        if let Some(b) = b {
            self.node_arena[b].parent = Some(p);
        }
        if let Some(c) = c {
            self.node_arena[c].parent = Some(g);
        }

        // Connect the new subtree to the rest of the tree
        if let Some(gg) = gg {
            // g was either left or right child of gg
            let gg_node = &mut self.node_arena[gg];
            if gg_node.right == Some(g) {
                gg_node.right = Some(x);
            } else {
                debug_assert_eq!(gg_node.left, Some(g));
                gg_node.left = Some(x);
            }
        } else {
            // g was the root, so x is now the new root
            self.root = Some(x);
        }
    }

    //     g             x
    //    / \           / \
    //   p   D         /   \
    //  / \     ==>   p     g
    // A   x         / \   / \
    //    / \       A   B C   D
    //   B   C
    fn zig_zag_left_to_right(&mut self, x: NodeId, p: NodeId, g: NodeId) {
        // p must NOT be root
        debug_assert!(self.node_arena[p].parent.is_some());
        // x must be right child of p
        debug_assert_eq!(self.node_arena[p].right, Some(x));
        // p must be left child of g
        debug_assert_eq!(self.node_arena[g].left, Some(p));

        // Store grandparent's parent for later update
        let gg = self.node_arena[g].parent;
        let b = self.node_arena[x].left;
        let c = self.node_arena[x].right;

        // First rotation (zig_right_to_left on x and p)
        self.node_arena[x].left = Some(p);
        self.node_arena[p].right = b;

        // Second rotation (zig_left_to_right on x and g)
        self.node_arena[x].right = Some(g);
        self.node_arena[g].left = c;

        // Update parent pointers

        // x is the new root of the subtree
        self.node_arena[x].parent = gg;

        // p is now the left child of x
        self.node_arena[p].parent = Some(x);

        // g is now the right child of x
        self.node_arena[g].parent = Some(x);

        // Update parent pointers for subtrees B and C
        if let Some(b) = b {
            self.node_arena[b].parent = Some(p);
        }
        if let Some(c) = c {
            self.node_arena[c].parent = Some(g);
        }

        // Connect the new subtree to the rest of the tree
        if let Some(gg) = gg {
            let gg_node = &mut self.node_arena[gg];
            if gg_node.left == Some(g) {
                gg_node.left = Some(x);
            } else {
                debug_assert_eq!(gg_node.right, Some(g));
                gg_node.right = Some(x);
            }
        } else {
            self.root = Some(x);
        }
    }

    //   g                x
    //  / \              / \
    // A   p            /   \
    //    / \   ==>    g     p
    //   x   D        / \   / \
    //  / \          A   B C   D
    // B   C
    fn zig_zag_right_to_left(&mut self, x: NodeId, p: NodeId, g: NodeId) {
        // p must NOT be root
        debug_assert!(self.node_arena[p].parent.is_some());
        // x must be left child of p
        debug_assert_eq!(self.node_arena[p].left, Some(x));
        // p must be right child of g
        debug_assert_eq!(self.node_arena[g].right, Some(p));

        // Store grandparent's parent for later update
        let gg = self.node_arena[g].parent;
        let b = self.node_arena[x].right;
        let c = self.node_arena[x].left;

        // First rotation (zig_left_to_right on x and p)
        self.node_arena[x].right = Some(p);
        self.node_arena[p].left = b;

        // Second rotation (zig_right_to_left on x and g)
        self.node_arena[x].left = Some(g);
        self.node_arena[g].right = c;

        // Update parent pointers

        // x is the new root of the subtree
        self.node_arena[x].parent = gg;

        // p is now the right child of x
        self.node_arena[p].parent = Some(x);

        // g is now the left child of x
        self.node_arena[g].parent = Some(x);

        // Update parent pointers for subtrees B and C
        if let Some(b) = b {
            self.node_arena[b].parent = Some(p);
        }
        if let Some(c) = c {
            self.node_arena[c].parent = Some(g);
        }

        // Connect the new subtree to the rest of the tree
        if let Some(gg) = gg {
            let gg_node = &mut self.node_arena[gg];
            if gg_node.right == Some(g) {
                gg_node.right = Some(x);
            } else {
                debug_assert_eq!(gg_node.left, Some(g));
                gg_node.left = Some(x);
            }
        } else {
            self.root = Some(x);
        }
    }

    fn splay(&mut self, id: NodeId) {
        // Continue splaying until the node is the root
        while let Some(parent_id) = self.node_arena[id].parent {
            // Case 1: The parent is the root (zig rotation)
            if self.node_arena[parent_id].parent.is_none() {
                if self.node_arena[parent_id].left == Some(id) {
                    self.zig_left_to_right(id, parent_id);
                } else if self.node_arena[parent_id].right == Some(id) {
                    self.zig_right_to_left(id, parent_id);
                } else {
                    unreachable!("incorrect state: node's parent does not have node as child")
                }
            }
            // Case 2: Grandparent exists
            else {
                let grandparent_id = self.node_arena[parent_id].parent.unwrap();

                // Get the parent's and node's position relative to their parents
                let is_p_left = self.node_arena[grandparent_id].left == Some(parent_id);
                let is_x_left = self.node_arena[parent_id].left == Some(id);

                match (is_p_left, is_x_left) {
                    (true, true) => self.zig_zig_left_to_right(id, parent_id, grandparent_id),
                    (false, false) => self.zig_zig_right_to_left(id, parent_id, grandparent_id),
                    (true, false) => self.zig_zag_left_to_right(id, parent_id, grandparent_id),
                    (false, true) => self.zig_zag_right_to_left(id, parent_id, grandparent_id),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::splay_tree::{SplayTree, SplayTreeNode};

    #[test]
    fn test_insert_into_empty_tree() {
        let mut tree = SplayTree::new();
        let cmp = |a: &u64, b: &u64| a.cmp(b);

        let new_node_ref = tree.insert(50, cmp);
        assert_eq!(new_node_ref.value, 50);

        assert_eq!(tree.root, Some(new_node_ref.id));

        assert_eq!(tree.value_arena, vec![50]);
        assert_eq!(tree.node_arena, vec![SplayTreeNode::root_no_child()]);
    }

    #[test]
    fn test_insert_with_splay() {
        let mut tree = SplayTree::new();
        let cmp = |a: &u64, b: &u64| a.cmp(b);

        // Initial insertions to build a small tree
        tree.insert(50, cmp); // root is 50
        tree.insert(30, cmp); // splay(30) makes it root

        // The final insertion, which should be the root
        let new_node_ref = tree.insert(70, cmp);
        assert_eq!(new_node_ref.value, 70);
        assert_eq!(tree.root, Some(new_node_ref.id));

        // The order in area should be the insertion order
        assert_eq!(tree.value_arena, vec![50, 30, 70]);
        assert_eq!(
            tree.node_arena,
            vec![
                // node 50
                SplayTreeNode::new(
                    Some(2), // node 70
                    Some(1), // node 30
                    None,
                ),
                // node 30
                SplayTreeNode::new(
                    Some(0), // node 50
                    None,
                    None,
                ),
                // node 70
                SplayTreeNode::new(
                    None,
                    Some(0), // node 50
                    None,
                ),
            ]
        );
    }

    #[test]
    fn test_insert_duplicate_values() {
        let mut tree = SplayTree::new();
        let cmp = |a: &u64, b: &u64| a.cmp(b);

        // Initial insertions
        tree.insert(50, cmp);
        tree.insert(30, cmp);

        // Insert a duplicate value
        let new_node_ref = tree.insert(50, cmp);

        // Assert that the new duplicate node is the root
        assert_eq!(new_node_ref.value, 50);
        assert_eq!(tree.root, Some(new_node_ref.id));

        // The order in area should be the insertion order
        assert_eq!(tree.value_arena, vec![50, 30, 50]);
        assert_eq!(
            tree.node_arena,
            vec![
                // node 50 (1)
                SplayTreeNode::new(
                    Some(2), // node 50 (2)
                    Some(1), // node 30
                    None,
                ),
                // node 30
                SplayTreeNode::new(
                    Some(0), // node 50 (2)
                    None,
                    None,
                ),
                // node 50 (2)
                SplayTreeNode::new(
                    None,
                    Some(0), // node 50 (1)
                    None,
                ),
            ]
        );
    }

    #[test]
    fn remove_node_has_zero_children() {
        let mut tree = SplayTree::<u64> {
            node_arena: vec![SplayTreeNode::root_no_child()],
            value_arena: vec![10],
            root: Some(0),
        };
        tree.remove_node_has_zero_children(0);
        assert_eq!(
            tree,
            SplayTree {
                node_arena: vec![SplayTreeNode::dangling()],
                value_arena: vec![10],
                root: None,
            }
        );

        let mut tree = SplayTree::<u64> {
            node_arena: vec![
                SplayTreeNode::new(None, Some(1), Some(2)),
                SplayTreeNode::leaf_with_parent(0),
                SplayTreeNode::leaf_with_parent(0),
            ],
            value_arena: vec![20, 10, 30],
            root: Some(0),
        };
        tree.remove_node_has_zero_children(2);
        assert_eq!(
            tree,
            SplayTree {
                node_arena: vec![
                    SplayTreeNode::new(None, Some(1), None,),
                    SplayTreeNode::leaf_with_parent(0),
                    SplayTreeNode::dangling(),
                ],
                value_arena: vec![20, 10, 30],
                root: Some(0),
            }
        );

        let mut tree = SplayTree::<u64> {
            node_arena: vec![
                SplayTreeNode::new(None, Some(1), Some(2)),
                SplayTreeNode::leaf_with_parent(0),
                SplayTreeNode::leaf_with_parent(0),
            ],
            value_arena: vec![20, 10, 30],
            root: Some(0),
        };
        tree.remove_node_has_zero_children(1);
        assert_eq!(
            tree,
            SplayTree {
                node_arena: vec![
                    SplayTreeNode::new(None, None, Some(2),),
                    SplayTreeNode::dangling(),
                    SplayTreeNode::leaf_with_parent(0)
                ],
                value_arena: vec![20, 10, 30],
                root: Some(0),
            }
        );
    }

    #[test]
    fn remove_node_has_one_children() {
        let mut tree = SplayTree::<u64> {
            node_arena: vec![
                SplayTreeNode::root(None, Some(1)),
                SplayTreeNode::leaf_with_parent(0),
            ],
            value_arena: vec![10, 20],
            root: Some(0),
        };
        tree.remove_node_has_one_children(0);
        assert_eq!(
            tree,
            SplayTree {
                node_arena: vec![SplayTreeNode::dangling(), SplayTreeNode::root_no_child(),],
                value_arena: vec![10, 20],
                root: Some(1),
            }
        );

        let mut tree = SplayTree::<u64> {
            node_arena: vec![
                SplayTreeNode::root(Some(1), None),
                SplayTreeNode::leaf_with_parent(0),
            ],
            value_arena: vec![10, 5],
            root: Some(0),
        };
        tree.remove_node_has_one_children(0);
        assert_eq!(
            tree,
            SplayTree {
                node_arena: vec![SplayTreeNode::dangling(), SplayTreeNode::root_no_child(),],
                value_arena: vec![10, 5],
                root: Some(1),
            }
        );

        let mut tree = SplayTree::<u64> {
            node_arena: vec![
                SplayTreeNode::root(Some(1), Some(2)),
                SplayTreeNode::new(Some(0), Some(3), None),
                SplayTreeNode::leaf_with_parent(0),
                SplayTreeNode::leaf_with_parent(1),
            ],
            value_arena: vec![10, 5, 20, 1],
            root: Some(0),
        };
        assert_eq!(tree.to_vec(), vec![1, 5, 10, 20]);
        tree.remove_node_has_one_children(1);
        assert_eq!(
            tree,
            SplayTree {
                node_arena: vec![
                    SplayTreeNode::root(Some(3), Some(2)),
                    SplayTreeNode::dangling(),
                    SplayTreeNode::leaf_with_parent(0),
                    SplayTreeNode::leaf_with_parent(0),
                ],
                value_arena: vec![10, 5, 20, 1],
                root: Some(0),
            }
        );
        assert_eq!(tree.to_vec(), vec![1, 10, 20]);

        let mut tree = SplayTree::<u64> {
            node_arena: vec![
                SplayTreeNode::root(Some(1), Some(2)),
                SplayTreeNode::new(Some(0), None, Some(3)),
                SplayTreeNode::leaf_with_parent(0),
                SplayTreeNode::leaf_with_parent(1),
            ],
            value_arena: vec![10, 5, 20, 7],
            root: Some(0),
        };
        assert_eq!(tree.to_vec(), vec![5, 7, 10, 20]);
        tree.remove_node_has_one_children(1);
        assert_eq!(
            tree,
            SplayTree {
                node_arena: vec![
                    SplayTreeNode::root(Some(3), Some(2)),
                    SplayTreeNode::dangling(),
                    SplayTreeNode::leaf_with_parent(0),
                    SplayTreeNode::leaf_with_parent(0),
                ],
                value_arena: vec![10, 5, 20, 7],
                root: Some(0),
            }
        );
        assert_eq!(tree.to_vec(), vec![7, 10, 20]);
    }

    #[test]
    fn remove_node_has_two_children_where_predecessor_is_left_child() {
        //   10
        //  / \
        // 1   20
        let mut tree = SplayTree::<u64> {
            node_arena: vec![
                SplayTreeNode::root(Some(1), Some(2)),
                SplayTreeNode::leaf_with_parent(0),
                SplayTreeNode::leaf_with_parent(0),
            ],
            value_arena: vec![10, 1, 20],
            root: Some(0),
        };
        assert_eq!(tree.to_vec(), vec![1, 10, 20]);
        let splay_node = tree.remove_node_has_two_children(0);
        assert_eq!(splay_node, None);
        assert_eq!(tree.to_vec(), vec![1, 20]);
        //   1
        //    \
        //     20
        assert_eq!(
            tree,
            SplayTree {
                node_arena: vec![
                    SplayTreeNode::dangling(),
                    SplayTreeNode::root(None, Some(2)),
                    SplayTreeNode::leaf_with_parent(1),
                ],
                value_arena: vec![10, 1, 20],
                root: Some(1),
            },
        );

        //      10
        //     /  \
        //    1   50
        //       /  \
        //     40   60
        let mut tree = SplayTree::<u64> {
            node_arena: vec![
                SplayTreeNode::root(Some(1), Some(2)),
                SplayTreeNode::leaf_with_parent(0),
                SplayTreeNode::new(Some(0), Some(3), Some(4)),
                SplayTreeNode::leaf_with_parent(2),
                SplayTreeNode::leaf_with_parent(2),
            ],
            value_arena: vec![10, 1, 50, 40, 60],
            root: Some(0),
        };
        assert_eq!(tree.to_vec(), vec![1, 10, 40, 50, 60]);
        let splay_node = tree.remove_node_has_two_children(2);
        assert_eq!(splay_node, Some(3));
        assert_eq!(tree.to_vec(), vec![1, 10, 40, 60]);
        //      10
        //     /  \
        //    1   40
        //          \
        //          60
        assert_eq!(
            tree,
            SplayTree {
                node_arena: vec![
                    SplayTreeNode::root(Some(1), Some(3)),
                    SplayTreeNode::leaf_with_parent(0),
                    SplayTreeNode::dangling(),
                    SplayTreeNode::new(Some(0), None, Some(4)),
                    SplayTreeNode::leaf_with_parent(3),
                ],
                value_arena: vec![10, 1, 50, 40, 60],
                root: Some(0),
            },
        );

        //      10
        //     /  \
        //    1   50
        //       /  \
        //     40   60
        //     /
        //    30
        let mut tree = SplayTree::<u64> {
            node_arena: vec![
                SplayTreeNode::root(Some(1), Some(2)),
                SplayTreeNode::leaf_with_parent(0),
                SplayTreeNode::new(Some(0), Some(3), Some(4)),
                SplayTreeNode::new(Some(2), Some(5), None),
                SplayTreeNode::leaf_with_parent(2),
                SplayTreeNode::leaf_with_parent(3),
            ],
            value_arena: vec![10, 1, 50, 40, 60, 30],
            root: Some(0),
        };
        assert_eq!(tree.to_vec(), vec![1, 10, 30, 40, 50, 60]);
        let splay_node = tree.remove_node_has_two_children(2);
        assert_eq!(splay_node, Some(3));
        assert_eq!(tree.to_vec(), vec![1, 10, 30, 40, 60]);
        //      10
        //     /  \
        //    1   40
        //       /  \
        //     30   60
        assert_eq!(
            tree,
            SplayTree {
                node_arena: vec![
                    SplayTreeNode::root(Some(1), Some(3)),
                    SplayTreeNode::leaf_with_parent(0),
                    SplayTreeNode::dangling(),
                    SplayTreeNode::new(Some(0), Some(5), Some(4)),
                    SplayTreeNode::leaf_with_parent(3),
                    SplayTreeNode::leaf_with_parent(3),
                ],
                value_arena: vec![10, 1, 50, 40, 60, 30],
                root: Some(0),
            },
        );
    }

    #[test]
    fn remove_node_has_two_children() {
        //    10
        //   /  \
        //  1    20
        //   \
        //    5
        let mut tree = SplayTree::<u64> {
            node_arena: vec![
                SplayTreeNode::root(Some(1), Some(2)),
                SplayTreeNode::new(Some(0), None, Some(3)),
                SplayTreeNode::leaf_with_parent(0),
                SplayTreeNode::leaf_with_parent(1),
            ],
            value_arena: vec![10, 1, 20, 5],
            root: Some(0),
        };
        assert_eq!(tree.to_vec(), vec![1, 5, 10, 20]);
        let splay_node = tree.remove_node_has_two_children(0);
        assert_eq!(splay_node, Some(1));
        assert_eq!(tree.to_vec(), vec![1, 5, 20]);
        //    5
        //   / \
        //  1   20
        assert_eq!(
            tree,
            SplayTree {
                node_arena: vec![
                    SplayTreeNode::dangling(),
                    SplayTreeNode::leaf_with_parent(3),
                    SplayTreeNode::leaf_with_parent(3),
                    SplayTreeNode::root(Some(1), Some(2)),
                ],
                value_arena: vec![10, 1, 20, 5],
                root: Some(3),
            },
        );

        //    10
        //   /  \
        //  1    50
        //      /  \
        //    40   60
        //     \
        //     45
        let mut tree = SplayTree::<u64> {
            node_arena: vec![
                SplayTreeNode::root(Some(1), Some(2)),
                SplayTreeNode::leaf_with_parent(0),
                SplayTreeNode::new(Some(0), Some(3), Some(4)),
                SplayTreeNode::new(Some(2), None, Some(5)),
                SplayTreeNode::leaf_with_parent(2),
                SplayTreeNode::leaf_with_parent(3),
            ],
            value_arena: vec![10, 1, 50, 40, 60, 45],
            root: Some(0),
        };
        assert_eq!(tree.to_vec(), vec![1, 10, 40, 45, 50, 60]);
        let splay_node = tree.remove_node_has_two_children(2);
        assert_eq!(splay_node, Some(3));
        assert_eq!(tree.to_vec(), vec![1, 10, 40, 45, 60]);
        //    10
        //   /  \
        //  1    45
        //      /  \
        //    40   60
        assert_eq!(
            tree,
            SplayTree {
                node_arena: vec![
                    SplayTreeNode::root(Some(1), Some(5)),
                    SplayTreeNode::leaf_with_parent(0),
                    SplayTreeNode::dangling(),
                    SplayTreeNode::leaf_with_parent(5),
                    SplayTreeNode::leaf_with_parent(5),
                    SplayTreeNode::new(Some(0), Some(3), Some(4)),
                ],
                value_arena: vec![10, 1, 50, 40, 60, 45],
                root: Some(0),
            },
        );

        //    10
        //   /  \
        //  1    50
        //      /  \
        //    40   60
        //    /
        //  35
        let mut tree = SplayTree::<u64> {
            node_arena: vec![
                SplayTreeNode::root(Some(1), Some(2)),
                SplayTreeNode::leaf_with_parent(0),
                SplayTreeNode::new(Some(0), Some(3), Some(4)),
                SplayTreeNode::new(Some(2), Some(5), None),
                SplayTreeNode::leaf_with_parent(2),
                SplayTreeNode::leaf_with_parent(3),
            ],
            value_arena: vec![10, 1, 50, 40, 60, 35],
            root: Some(0),
        };
        assert_eq!(tree.to_vec(), vec![1, 10, 35, 40, 50, 60]);
        let splay_node = tree.remove_node_has_two_children(2);
        assert_eq!(splay_node, Some(3));
        assert_eq!(tree.to_vec(), vec![1, 10, 35, 40, 60]);
        //    10
        //   /  \
        //  1    40
        //      /  \
        //    35   60
        assert_eq!(
            tree,
            SplayTree {
                node_arena: vec![
                    SplayTreeNode::root(Some(1), Some(3)),
                    SplayTreeNode::leaf_with_parent(0),
                    SplayTreeNode::dangling(),
                    SplayTreeNode::new(Some(0), Some(5), Some(4)),
                    SplayTreeNode::leaf_with_parent(3),
                    SplayTreeNode::leaf_with_parent(3),
                ],
                value_arena: vec![10, 1, 50, 40, 60, 35],
                root: Some(0),
            },
        );
    }

    mod compare {
        use rand::seq::SliceRandom;

        use crate::splay_tree::{NodeRef, SplayTree};

        #[test]
        fn should_function_correctly_given_a_non_reverse_cmp() {
            let mut tree = SplayTree::new();
            let cmp = |a: &u64, b: &u64| a.cmp(b).reverse();
            tree.insert(2, cmp);
            let id_1 = tree.insert(1, cmp).id;
            let id_3 = tree.insert(3, cmp).id;
            assert_eq!(tree.len(), 3);
            assert_eq!(tree.min().unwrap(), NodeRef { value: 3, id: id_3 });
            assert_eq!(tree.max().unwrap(), NodeRef { value: 1, id: id_1 });
            let node = tree.find(&3, cmp).unwrap();
            tree.remove(node);
            assert_eq!(tree.len(), 2);
            assert_eq!(tree.value_arena[tree.root.unwrap()], 2);
            assert!(tree.node_arena[tree.root.unwrap()].left.is_none());
            assert_eq!(
                tree.value_arena[tree.node_arena[tree.root.unwrap()].right.unwrap()],
                1
            );
        }

        #[test]
        fn should_support_custom_cmp() {
            #[derive(Clone, Copy, PartialEq, Eq, Debug)]
            struct Element {
                key: u64,
                value: u64,
            }

            let mut objects = (0u64..10)
                .map(|i| Element {
                    key: i,
                    value: i * i,
                })
                .collect::<Vec<_>>();
            objects.shuffle(&mut rand::rng());

            let mut tree = SplayTree::new();
            let cmp = |a: &Element, b: &Element| a.key.cmp(&b.key);

            for obj in objects.iter() {
                tree.insert(*obj, cmp);
            }

            objects.sort_by(cmp);
            assert_eq!(tree.to_vec(), objects);
        }
    }

    mod contains {
        use crate::splay_tree::SplayTree;

        #[test]
        fn should_return_none_if_the_tree_is_empty() {
            let mut tree = SplayTree::new();
            let cmp = |a: &u64, b: &u64| a.cmp(b);
            assert!(tree.find(&1, cmp).is_none());
        }

        #[test]
        fn should_return_whether_the_tree_contains_a_node() {
            let mut tree = SplayTree::new();
            let cmp = |a: &u64, b: &u64| a.cmp(b);
            assert!(tree.find(&1, cmp).is_none());
            assert!(tree.find(&2, cmp).is_none());
            assert!(tree.find(&3, cmp).is_none());
            tree.insert(3, cmp);
            tree.insert(1, cmp);
            tree.insert(2, cmp);
            assert!(tree.find(&1, cmp).is_some());
            assert!(tree.find(&2, cmp).is_some());
            assert!(tree.find(&3, cmp).is_some());
        }

        #[test]
        fn should_return_none_when_the_expected_parent_has_no_children() {
            let mut tree = SplayTree::new();
            let cmp = |a: &u64, b: &u64| a.cmp(b);
            tree.insert(2, cmp);
            assert!(tree.find(&1, cmp).is_none());
            assert!(tree.find(&3, cmp).is_none());
        }
    }

    mod duplicate {
        use crate::splay_tree::SplayTree;

        #[test]
        fn should_allow_inserting_of_duplicate_key() {
            let mut tree = SplayTree::new();
            let cmp = |a: &i64, b: &i64| a.cmp(b);

            for v in [2, 12, 1, -6, 1] {
                tree.insert(v, cmp);
            }

            assert_eq!(tree.to_vec(), vec![-6, 1, 1, 2, 12]);
        }

        #[test]
        fn should_allow_multiple_duplicate_keys_in_a_row() {
            let mut tree = SplayTree::new();
            let cmp = |a: &i64, b: &i64| a.cmp(b);

            for v in [2, 12, 1, 1, -6, 2, 1, 1, 13] {
                tree.insert(v, cmp);
            }

            assert_eq!(tree.to_vec(), vec![-6, 1, 1, 1, 1, 2, 2, 12, 13]);
        }

        #[test]
        fn should_remove_from_a_tree_with_duplicate_keys_correctly() {
            let mut tree = SplayTree::new();
            let cmp = |a: &i64, b: &i64| a.cmp(b);

            for v in [2, 12, 1, 1, -6, 1, 1] {
                tree.insert(v, cmp);
            }

            let mut l = tree.len();
            for i in 0..4 {
                let node = tree.find(&1, cmp).unwrap();
                tree.remove(node);

                if i < 3 {
                    assert!(tree.find(&1, cmp).is_some());
                }
                l -= 1;
                assert_eq!(tree.len(), l);
            }

            assert!(tree.find(&1, cmp).is_none());
        }

        #[test]
        fn should_remove_from_a_tree_with_multiple_duplicate_keys_correctly() {
            let mut tree = SplayTree::new();
            let cmp = |a: &i64, b: &i64| a.cmp(b);

            for v in [2, 12, 1, 1, -6, 1, 1, 2, 0, 2] {
                tree.insert(v, cmp);
            }

            let mut l = tree.len();
            while !tree.is_empty() {
                let node = tree.min().unwrap();
                tree.remove(node);
                l -= 1;
                assert_eq!(tree.len(), l);
            }
        }
    }

    mod empty {
        use crate::splay_tree::SplayTree;

        #[test]
        fn should_return_whether_the_tree_is_empty() {
            let mut tree = SplayTree::new();
            let cmp = |a: &u64, b: &u64| a.cmp(b);

            assert!(tree.is_empty());
            tree.insert(1, cmp);
            assert!(!tree.is_empty());
            let node = tree.find(&1, cmp).unwrap();
            tree.remove(node);
            assert!(tree.is_empty());
        }
    }

    mod find {
        use crate::splay_tree::SplayTree;

        #[test]
        fn should_return_key_as_the_result_of_search() {
            #[derive(Clone, Copy, PartialEq, Eq, Debug)]
            struct Element {
                key: i64,
                value: u64,
            }

            let mut tree = SplayTree::new();
            let cmp = |a: &Element, b: &Element| a.key.cmp(&b.key);

            assert!(tree.find(&Element { key: 1, value: 0 }, cmp).is_none());
            assert!(tree.find(&Element { key: 2, value: 0 }, cmp).is_none());
            assert!(tree.find(&Element { key: 3, value: 0 }, cmp).is_none());
            tree.insert(Element { key: 1, value: 4 }, cmp);
            tree.insert(Element { key: 2, value: 5 }, cmp);
            tree.insert(Element { key: 3, value: 6 }, cmp);

            let old_root = tree.root;
            assert_eq!(
                tree.find(&Element { key: 1, value: 0 }, cmp)
                    .map(|n| n.value),
                Some(Element { key: 1, value: 4 }),
            );
            assert_ne!(old_root, tree.root);
        }

        #[test]
        fn should_allow_finding_node_without_splaying() {
            #[derive(Clone, Copy, PartialEq, Eq, Debug)]
            struct Element {
                key: i64,
                value: i64,
            }

            let mut tree = SplayTree::new();
            let cmp = |a: &Element, b: &Element| a.key.cmp(&b.key);

            assert_eq!(tree.find_static(&Element { key: 1, value: 0 }, cmp), None);
            assert_eq!(tree.find_static(&Element { key: 2, value: 0 }, cmp), None);
            assert_eq!(tree.find_static(&Element { key: 3, value: 0 }, cmp), None);
            tree.insert(Element { key: -2, value: 8 }, cmp);
            tree.insert(Element { key: 1, value: 4 }, cmp);
            tree.insert(Element { key: 2, value: 5 }, cmp);
            tree.insert(Element { key: 3, value: 6 }, cmp);

            tree.find(&Element { key: 2, value: 0 }, cmp);
            let root = tree.root;
            assert_eq!(
                tree.find_static(&Element { key: 1, value: 0 }, cmp)
                    .map(|n| n.value.value),
                Some(4),
            );
            assert_eq!(root, tree.root);

            assert_eq!(
                tree.find_static(&Element { key: 2, value: 0 }, cmp)
                    .map(|n| n.value.value),
                Some(5),
            );
            assert_eq!(root, tree.root);

            assert_eq!(
                tree.find_static(&Element { key: 3, value: 0 }, cmp)
                    .map(|n| n.value.value),
                Some(6),
            );
            assert_eq!(root, tree.root);

            assert_eq!(
                tree.find_static(&Element { key: -2, value: 0 }, cmp)
                    .map(|n| n.value.value),
                Some(8),
            );
            assert_eq!(
                tree.find(&Element { key: 2, value: 0 }, cmp).map(|n| n.id),
                tree.root
            );
        }
    }
}
