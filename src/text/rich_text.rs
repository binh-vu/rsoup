use hashbrown::HashMap;
use std::fmt;

use crate::misc::{ITree, SimpleTree};
use pyo3::prelude::*;

pub const PSEUDO_TAG: &str = "";

#[derive(Clone, PartialEq, Eq)]
#[pyclass(module = "table_extractor.table_extractor")]
pub struct RichText {
    #[pyo3(get)]
    pub text: String,
    // html elements creating this text, the root of the tree
    // is a pseudo-element, most often, it will be the html element containing
    // the text, but if we are dealing with a text node, tag will be empty
    // or after we merge, the tag will be empty
    pub element: SimpleTree<RichTextElement>,
}

/// Represent an html element.
#[derive(Debug, Clone, PartialEq, Eq)]
#[pyclass(module = "table_extractor.table_extractor")]
pub struct RichTextElement {
    #[pyo3(get)]
    pub tag: String,
    #[pyo3(get)]
    pub start: usize,
    #[pyo3(get)]
    pub end: usize,
    pub attrs: HashMap<String, String>,
}

impl RichText {
    // Create an empty rich text, you should not use this function directly to
    // build a rich text as the tree has the PSEUDO_TAG.
    pub fn empty() -> RichText {
        RichText {
            text: String::new(),
            element: SimpleTree::new(RichTextElement {
                tag: PSEUDO_TAG.to_owned(),
                start: 0,
                end: 0,
                attrs: HashMap::new(),
            }),
        }
    }

    pub fn from_str(text: &str) -> RichText {
        RichText {
            text: text.to_owned(),
            element: SimpleTree::new(RichTextElement {
                tag: PSEUDO_TAG.to_owned(),
                start: 0,
                end: text.len(),
                attrs: HashMap::new(),
            }),
        }
    }

    pub fn get_tag(&self) -> &str {
        self.element.get_root().tag.as_str()
    }

    pub fn validate(&self) -> bool {
        let root_id = self.element.get_root_id();
        let root = self.element.get_root();
        let mut is_valid = root.start == 0 && root.end == self.text.len();

        for node_id in self.element.iter_id_preorder() {
            let node = self.element.get_node(*node_id);
            if *node_id != root_id {
                is_valid = is_valid && node.tag != PSEUDO_TAG;
            }

            is_valid = is_valid && node.start <= node.end;
            let child_ids = self.element.get_child_ids(*node_id);
            for (i, child_id) in child_ids.iter().enumerate() {
                let child = self.element.get_node(*child_id);
                is_valid = is_valid && node.start <= child.start && node.end >= child.end;
                if i > 0 {
                    let prev_child = self.element.get_node(child_ids[i - 1]);
                    is_valid = is_valid && child.start >= prev_child.end;
                }
            }
        }
        is_valid
    }

    pub fn to_html(&self, render_outer_element: bool, render_element_attrs: bool) -> String {
        let mut tokens = Vec::<&str>::with_capacity(2 + self.element.len());
        // keep track of pending tags that need to be closed
        let mut closing_tag_ids = Vec::<usize>::new();
        let mut pointer = 0;
        let mut it = self.element.iter_id_preorder();
        let mut string_pools = Vec::new();
        let mut pending_ops = Vec::new();

        if !render_outer_element {
            it.next();
        }

        for token_id in it {
            let token = self.element.get_node(*token_id);
            println!(
                "------before\n\t>> pointer: {}\n\t>> token: {:?}\n\t>> tokens: {:?}\n\t>> closing_tags: {:?}",
                pointer, token, tokens, closing_tag_ids.iter().map(|id| self.element.get_node(*id)).collect::<Vec<_>>()
            );

            while let Some(closing_tag_id) = closing_tag_ids.last() {
                let closing_tag = self.element.get_node(*closing_tag_id);

                if closing_tag.end <= token.start {
                    // this tag is closed
                    if token.start == token.end {
                        // this token is empty, but is it part of this closing tag?
                        // if not, we can continue to closing tag
                        // if yes, we break here.
                        // and it only happens when its a direct children
                        if self
                            .element
                            .get_child_ids_ref(closing_tag_id)
                            .iter()
                            .any(|child_id| child_id == token_id)
                        {
                            break;
                        }
                    }
                    tokens.push(&self.text[pointer..closing_tag.end]);
                    tokens.push("</");
                    tokens.push(&closing_tag.tag);
                    tokens.push(">");
                    pointer = closing_tag.end;
                    closing_tag_ids.pop();
                } else {
                    break;
                }
            }

            tokens.push(&self.text[pointer..token.start]);
            tokens.push("<");
            tokens.push(&token.tag);
            if render_element_attrs {
                for (name, value) in token.attrs.iter() {
                    tokens.push(" ");
                    tokens.push(name);
                    tokens.push("=\"");
                    string_pools.push(value.replace("\"", "\\\""));
                    pending_ops.push(tokens.len());
                    tokens.push("");
                    tokens.push("\"");
                }
            }
            tokens.push(">");

            pointer = token.start;
            closing_tag_ids.push(*token_id);

            println!(
                "------after\n\t>> pointer: {}\n\t>> token: {:?}\n\t>> tokens: {:?}\n\t>> closing_tags: {:?}",
                pointer, token, tokens, closing_tag_ids.iter().map(|id| self.element.get_node(*id)).collect::<Vec<_>>()
            );
        }

        for closing_tag_id in closing_tag_ids.iter().rev() {
            let closing_tag = self.element.get_node(*closing_tag_id);
            tokens.push(&self.text[pointer..closing_tag.end]);
            tokens.push("</");
            tokens.push(&closing_tag.tag);
            tokens.push(">");
            pointer = closing_tag.end;
        }
        tokens.push(&self.text[pointer..]);

        // update tokens
        for (i, j) in pending_ops.into_iter().enumerate() {
            tokens[j] = &string_pools[i];
        }

        tokens.join("")
    }
}

#[pymethods]
impl RichText {
    #[getter]
    fn text(&self) -> PyResult<&String> {
        Ok(&self.text)
    }
}

#[pymethods]
impl RichTextElement {
    fn get_attr(&self, name: &str) -> PyResult<Option<&str>> {
        Ok(self.attrs.get(name).map(|s| s.as_str()))
    }

    fn has_attr(&self, name: &str) -> PyResult<bool> {
        Ok(self.attrs.contains_key(name))
    }
}

impl fmt::Display for RichText {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "`{}`", self.to_html(false, false))
    }
}

impl fmt::Debug for RichText {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "`{}`", self.to_html(true, false))
    }
}
