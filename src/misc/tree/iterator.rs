pub trait ITree<N> {
    fn get_root_<'s>(&'s self) -> &'s N;
    fn get_children_<'s>(&'s self, node: &'s N) -> &'s [N];
}

pub struct PreorderTraversal<'s, T, N>
where
    T: ITree<N>,
{
    tree: &'s T,
    stack: Vec<(&'s N, usize)>,
    inited: bool,
}

impl<'s, T, N> PreorderTraversal<'s, T, N>
where
    T: ITree<N>,
{
    pub fn new(tree: &'s T) -> Self {
        PreorderTraversal {
            tree,
            stack: Vec::new(),
            inited: false,
        }
    }
}

impl<'s, T, N> Iterator for PreorderTraversal<'s, T, N>
where
    T: ITree<N>,
    N: 's,
{
    type Item = &'s N;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.stack.len() == 0 {
                if self.inited {
                    return None;
                }
                self.inited = true;
                self.stack.push((self.tree.get_root_(), 0));
                return Some(self.stack[self.stack.len() - 1].0);
            }

            // current element has been returned previously
            // so we will try to return its child
            let n1 = self.stack.len() - 1;
            let (node, child_index) = self.stack[n1];
            let node_children = self.tree.get_children_(node);

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
