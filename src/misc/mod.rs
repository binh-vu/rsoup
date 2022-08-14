pub mod range_iter;
pub mod recursive_iter;
pub mod tree;
pub mod url_converter;

use hashbrown::HashMap;
use scraper::node::Attributes;

pub fn convert_attrs(attrs: &Attributes) -> HashMap<String, String> {
    attrs
        .iter()
        .map(|(k, v)| (k.local.to_string(), v.to_string()))
        .collect::<HashMap<_, _>>()
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
