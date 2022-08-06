use crate::misc::PreorderTraversal;

use super::iterator::ITree;

/// A simple vector-based tree. Nodes are ordered based on their insertion order.
pub struct SimpleTree<N> {
    root: usize,
    nodes: Vec<N>,
    node2children: Vec<Vec<usize>>,
}

impl<N> SimpleTree<N> {
    pub fn new() -> SimpleTree<N> {
        SimpleTree {
            root: 0,
            nodes: Vec::new(),
            node2children: Vec::new(),
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    #[inline]
    pub fn root(&self) -> usize {
        self.root
    }

    #[inline]
    pub fn get_node(&self, uid: usize) -> &N {
        &self.nodes[uid]
    }

    pub fn add_node(&mut self, node: N) -> usize {
        let uid = self.nodes.len();
        self.nodes.push(node);
        self.node2children.push(Vec::new());
        uid
    }

    pub fn add_child(&mut self, parent_id: usize, child_id: usize) {
        if child_id == self.root {
            self.root = parent_id;
        }
        self.node2children[parent_id].push(child_id)
    }

    #[inline]
    pub fn get_children(&self, uid: usize) -> &[usize] {
        &self.node2children[uid]
    }

    pub fn iter_preorder<'s>(&'s self) -> PreorderTraversal<'s, SimpleTree<N>, usize> {
        PreorderTraversal::new(self)
    }

    #[inline]
    pub fn iter(&self) -> &[N] {
        &self.nodes
    }
}

impl<N> ITree<usize> for SimpleTree<N> {
    fn get_root_(&self) -> &usize {
        &self.root
    }

    fn get_children_(&self, node: &usize) -> &[usize] {
        &self.node2children[*node]
    }
}
