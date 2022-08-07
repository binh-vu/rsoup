use std::marker::PhantomData;

pub trait ITree<K, V> {
    fn get_root_id_ref<'s>(&'s self) -> &'s K;
    fn get_node_by_id_ref<'s>(&'s self, id: &'s K) -> &'s V;
    fn get_child_ids_ref<'s>(&'s self, id: &'s K) -> &'s [K];
}

pub struct IdPreorderTraversal<'s, T, K, N>
where
    T: ITree<K, N>,
{
    tree: &'s T,
    stack: Vec<(&'s K, usize)>,
    inited: bool,
    phantom: PhantomData<N>,
}

impl<'s, T, K, N> IdPreorderTraversal<'s, T, K, N>
where
    T: ITree<K, N>,
{
    pub fn new(tree: &'s T) -> Self {
        IdPreorderTraversal {
            tree,
            stack: Vec::new(),
            inited: false,
            phantom: PhantomData,
        }
    }
}

impl<'s, T, K, N> Iterator for IdPreorderTraversal<'s, T, K, N>
where
    T: ITree<K, N>,
    K: 's,
{
    type Item = &'s K;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.stack.len() == 0 {
                if self.inited {
                    return None;
                }
                self.inited = true;
                self.stack.push((self.tree.get_root_id_ref(), 0));
                return Some(self.stack[self.stack.len() - 1].0);
            }

            // current element has been returned previously
            // so we will try to return its child
            let n1 = self.stack.len() - 1;
            let (node, child_index) = self.stack[n1];
            let node_children = self.tree.get_child_ids_ref(node);

            if child_index < node_children.len() {
                // add this child to stack
                self.stack.push((&node_children[child_index], 0));
                self.stack[n1].1 += 1;
                return Some(&node_children[child_index]);
            }

            // no child to return, done at this level, so we move up
            self.stack.pop();
        }
    }
}

pub struct NodePreorderTraversal<'s, T, K, N>(IdPreorderTraversal<'s, T, K, N>)
where
    T: ITree<K, N>;

impl<'s, T, K, N> NodePreorderTraversal<'s, T, K, N>
where
    T: ITree<K, N>,
{
    pub fn new(tree: &'s T) -> Self {
        NodePreorderTraversal(IdPreorderTraversal::new(tree))
    }
}

impl<'s, T, K, N> Iterator for NodePreorderTraversal<'s, T, K, N>
where
    T: ITree<K, N>,
    N: 's,
{
    type Item = &'s N;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.0.stack.len() == 0 {
                if self.0.inited {
                    return None;
                }
                self.0.inited = true;
                self.0.stack.push((self.0.tree.get_root_id_ref(), 0));
                return Some(
                    self.0
                        .tree
                        .get_node_by_id_ref(self.0.stack[self.0.stack.len() - 1].0),
                );
            }

            // current element has been returned previously
            // so we will try to return its child
            let n1 = self.0.stack.len() - 1;
            let (node, child_index) = self.0.stack[n1];
            let node_children = self.0.tree.get_child_ids_ref(node);

            if child_index < node_children.len() {
                // add this child to stack
                self.0.stack.push((&node_children[child_index], 0));
                self.0.stack[n1].1 += 1;
                return Some(self.0.tree.get_node_by_id_ref(&node_children[child_index]));
            }

            // no child to return, done at this level, so we move up
            self.0.stack.pop();
        }
    }
}
