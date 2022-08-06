use std::marker::PhantomData;

use hashbrown::HashMap;
use scraper::node::Attributes;

pub fn convert_attrs(attrs: &Attributes) -> HashMap<String, String> {
    attrs
        .iter()
        .map(|(k, v)| (k.local.to_string(), v.to_string()))
        .collect::<HashMap<_, _>>()
}

/// Represent a recursive tree
pub trait ITree<N>
where
    N: Copy,
{
    fn get_root(&self) -> N;
    fn get_children(&self, id: N) -> &[N];
}

pub struct PreorderTraversal<'s, T, N>
where
    T: ITree<N>,
    N: Copy,
{
    tree: &'s T,
    stack: Vec<(N, usize)>,
    inited: bool,
}

impl<'s, T, N> Iterator for PreorderTraversal<'s, T, N>
where
    T: ITree<N>,
    N: Copy,
{
    type Item = N;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.stack.len() == 0 {
                if self.inited {
                    return None;
                }
                self.inited = true;
                self.stack.push((self.tree.get_root(), 0));
                return Some(self.stack[self.stack.len() - 1].0);
            }

            // current element has been returned previously
            // so we will try to return its child
            let n1 = self.stack.len() - 1;
            let (node, child_index) = self.stack[n1];
            let node_children = self.tree.get_children(node);

            if child_index < node_children.len() {
                // add this child to stack
                self.stack.push((node_children[child_index], 0));
                self.stack[n1].1 += 1;
                return Some(node_children[child_index]);
            }

            // no child to return, done at this level, so we move up
            self.stack.pop();
        }
    }
}
