use hashbrown::HashMap;
use scraper::node::Attributes;

pub fn convert_attrs(attrs: &Attributes) -> HashMap<String, String> {
    attrs
        .iter()
        .map(|(k, v)| (k.local.to_string(), v.to_string()))
        .collect::<HashMap<_, _>>()
}

pub trait ITree<N> {
    fn get_root<'s>(&'s self) -> &'s N;
    fn get_children<'s>(&'s self, node: &'s N) -> &'s [N];
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
                self.stack.push((&node_children[child_index], 0));
                self.stack[n1].1 += 1;
                return Some(&node_children[child_index]);
            }

            // no child to return, done at this level, so we move up
            self.stack.pop();
        }
    }
}

pub struct ChainN<I, V>
where
    I: Iterator<Item = V>,
{
    pub iterators: Vec<I>,
    pub index: usize,
}

impl<I, V> Iterator for ChainN<I, V>
where
    I: Iterator<Item = V>,
{
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.iterators.len() {
            if let Some(value) = self.iterators[self.index].next() {
                return Some(value);
            }
            self.index += 1;
        }
        return None;
    }
}

pub enum Enum2<A, B> {
    Type1(A),
    Type2(B),
}

impl<A, B> Enum2<A, B> {
    pub fn is_type2(&self) -> bool {
        if let Enum2::Type2(_) = self {
            return true;
        }
        return false;
    }
}
