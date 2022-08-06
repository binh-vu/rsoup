use ego_tree::NodeRef;
use scraper::Node;

use crate::helper::{ITree, PreorderTraversal};

pub(crate) struct SubTree<'s> {
    pub root: usize,
    pub nodes: Vec<NodeRef<'s, Node>>,
    pub id2children: Vec<Vec<usize>>,
}

impl<'s> ITree<usize> for SubTree<'s> {
    fn get_root<'t>(&'t self) -> &'t usize {
        &self.root
    }

    fn get_children<'t>(&'t self, node: &'t usize) -> &'t [usize] {
        &self.id2children[*node]
    }
}

impl<'s> SubTree<'s> {
    pub fn new() -> Self {
        SubTree {
            root: 0,
            nodes: Vec::new(),
            id2children: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn get_node(&self, node_id: usize) -> &NodeRef<'s, Node> {
        &self.nodes[node_id]
    }

    pub fn add_node(&mut self, node: NodeRef<'s, Node>) -> usize {
        let nodeid = self.nodes.len();
        self.nodes.push(node);
        self.id2children.push(Vec::new());
        nodeid
    }

    pub fn add_new_child(&mut self, parent_id: usize, node: NodeRef<'s, Node>) -> usize {
        let nodeid = self.add_node(node);
        self.id2children[parent_id].push(nodeid);
        nodeid
    }

    pub fn add_existing_child(&mut self, parent_id: usize, child_id: usize) {
        self.id2children[parent_id].push(child_id);
    }

    pub fn preorder_traversal<'t>(&'t self) -> PreorderTraversal<'t, SubTree<'s>, usize> {
        PreorderTraversal::new(self)
    }
}
