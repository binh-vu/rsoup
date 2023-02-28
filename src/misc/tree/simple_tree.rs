use crate::misc::tree::iterator::{ITree, IdPreorderTraversal, NodePreorderTraversal};
use serde::{Deserialize, Serialize};

/// A simple vector-based tree. Nodes are ordered based on their insertion order.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct SimpleTree<N> {
    root: usize,
    nodes: Vec<N>,
    pub node2children: Vec<Vec<usize>>,
}

impl<N> SimpleTree<N> {
    pub fn empty() -> SimpleTree<N> {
        SimpleTree {
            root: 0,
            nodes: Vec::new(),
            node2children: Vec::new(),
        }
    }

    pub fn new(node: N) -> SimpleTree<N> {
        SimpleTree {
            root: 0,
            nodes: vec![node],
            node2children: vec![vec![]],
        }
    }

    pub fn from_data(root: usize, nodes: Vec<N>, node2children: Vec<Vec<usize>>) -> SimpleTree<N> {
        SimpleTree {
            root,
            nodes,
            node2children,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    #[inline]
    pub fn get_root_id(&self) -> usize {
        self.root
    }

    #[inline]
    pub fn get_root(&self) -> &N {
        &self.nodes[self.root]
    }

    #[inline]
    pub fn get_root_mut(&mut self) -> &mut N {
        &mut self.nodes[self.root]
    }

    #[inline]
    pub fn get_node(&self, uid: usize) -> &N {
        &self.nodes[uid]
    }

    #[inline]
    pub fn get_node_mut(&mut self, uid: usize) -> &mut N {
        &mut self.nodes[uid]
    }

    pub fn add_node(&mut self, node: N) -> usize {
        let uid = self.nodes.len();
        self.nodes.push(node);
        self.node2children.push(Vec::new());
        uid
    }

    #[inline]
    pub fn update_node(&mut self, id: usize, node: N) {
        self.nodes[id] = node;
    }

    pub fn add_child(&mut self, parent_id: usize, child_id: usize) {
        if child_id == self.root {
            self.root = parent_id;
        }
        self.node2children[parent_id].push(child_id)
    }

    #[inline]
    pub fn get_child_ids(&self, uid: usize) -> &[usize] {
        &self.node2children[uid]
    }

    pub fn iter_id_preorder<'s>(&'s self) -> IdPreorderTraversal<'s, SimpleTree<N>, usize, N> {
        IdPreorderTraversal::new(self)
    }

    pub fn iter_node_preorder<'s>(&'s self) -> NodePreorderTraversal<'s, SimpleTree<N>, usize, N> {
        NodePreorderTraversal::new(self)
    }

    #[inline]
    pub fn iter(&self) -> &[N] {
        &self.nodes
    }

    #[inline]
    pub fn iter_mut(&mut self) -> &mut [N] {
        &mut self.nodes
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn merge_subtree(&mut self, parent_id: usize, mut subtree: SimpleTree<N>) {
        let id_offset = self.nodes.len();
        self.nodes.extend(subtree.nodes.into_iter());
        // update ids of children in node => children in the subtree
        for children in subtree.node2children.iter_mut() {
            for child_id in children {
                *child_id += id_offset;
            }
        }
        self.node2children.extend(subtree.node2children.into_iter());
        self.node2children[parent_id].push(subtree.root + id_offset);
    }

    /// Merge direct children of root of the subtree into this tree
    pub fn merge_subtree_no_root(&mut self, parent_id: usize, mut subtree: SimpleTree<N>) {
        let id_offset = self.nodes.len();
        let subtree_root = subtree.get_root_id();

        let mut it = subtree.nodes.into_iter();
        if subtree_root > 0 {
            self.nodes.extend((&mut it).take(subtree_root));
        }
        it.next();
        self.nodes.extend(it);

        // update ids of children in node => children in the subtree
        for children in subtree.node2children.iter_mut() {
            for child_id in children {
                if *child_id > subtree_root {
                    *child_id += id_offset - 1;
                } else {
                    *child_id += id_offset;
                }
            }
        }
        self.node2children[parent_id].extend_from_slice(&subtree.node2children[subtree_root]);

        // add children
        let mut it = subtree.node2children.into_iter();
        if subtree_root > 0 {
            self.node2children.extend((&mut it).take(subtree_root));
        }
        it.next();
        self.node2children.extend(it);
    }

    pub fn validate(&self) -> bool {
        let mut is_valid = true;
        for child_ids in &self.node2children {
            is_valid = is_valid
                && child_ids
                    .iter()
                    .all(|&child_id| child_id < self.nodes.len());
        }
        is_valid = is_valid && self.iter_id_preorder().count() == self.nodes.len();
        is_valid
    }

    pub fn to_string(&self, key: &dyn Fn(usize) -> String) -> String {
        let mut buffer = Vec::<String>::with_capacity(self.len());

        struct RecurFn<'s> {
            f: &'s dyn Fn(&RecurFn, usize, usize, &mut Vec<String>),
        }
        let func = RecurFn {
            f: &|func, node_id: usize, depth: usize, buffer: &mut Vec<String>| {
                let indent = " ".repeat(depth * 4);

                buffer.push(indent.clone());
                buffer.push(key(node_id));

                if self.node2children[node_id].len() > 0 {
                    buffer.push(" -> {\n".to_owned());
                    for child_id in self.node2children[node_id].iter() {
                        (func.f)(func, *child_id, depth + 1, buffer);
                    }
                    buffer.push(indent);
                    buffer.push("}\n".to_owned());
                } else {
                    buffer.push("\n".to_owned());
                }
            },
        };

        (func.f)(&func, self.root, 0, &mut buffer);
        buffer.join("")
    }
}

impl<N> ITree<usize, N> for SimpleTree<N> {
    fn get_root_id_ref(&self) -> &usize {
        &self.root
    }

    fn get_node_by_id_ref<'s>(&'s self, id: &'s usize) -> &'s N {
        &self.nodes[*id]
    }

    fn get_child_ids_ref(&self, node: &usize) -> &[usize] {
        &self.node2children[*node]
    }
}
