use hashbrown::HashMap;

use crate::misc::{ChainN, ITree, PreorderTraversal};
use pyo3::{prelude::*, types::PyString};

#[derive(Debug, Clone, PartialEq, Eq)]
#[pyclass(module = "table_extractor.text")]
pub struct TextTrace {
    #[pyo3(get)]
    pub text: String,
    // html elements are stored in order
    pub trace: Vec<TextHTMLElement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[pyclass(module = "table_extractor.text")]
pub struct TextHTMLElement {
    #[pyo3(get)]
    pub tag: String,
    #[pyo3(get)]
    pub start: usize,
    #[pyo3(get)]
    pub end: usize,
    pub attrs: HashMap<String, String>,
    pub children: Vec<TextHTMLElement>,
}

impl TextTrace {
    pub fn empty() -> TextTrace {
        TextTrace {
            text: String::new(),
            trace: Vec::new(),
        }
    }

    pub fn from_str(text: &str) -> TextTrace {
        TextTrace {
            text: text.to_owned(),
            trace: Vec::new(),
        }
    }

    pub fn merge(&mut self, text: TextTrace) {
        for mut el in text.trace {
            el.shift(self.text.len());
            self.trace.push(el);
        }
        self.text.push_str(&text.text);
    }

    pub fn preorder_traversal<'s>(
        &'s self,
    ) -> ChainN<PreorderTraversal<'s, TextHTMLElement, TextHTMLElement>, &TextHTMLElement> {
        ChainN {
            iterators: self
                .trace
                .iter()
                .map(|el| PreorderTraversal::new(el))
                .collect::<Vec<_>>(),
            index: 0,
        }
    }

    pub fn to_bare_html(&self) -> String {
        let mut tokens = Vec::<&str>::with_capacity(2 + self.trace.len());
        let mut closing_tags = Vec::<&TextHTMLElement>::new();
        let mut pointer = 0;

        for token in self.preorder_traversal() {
            // println!(
            //     "------before\n\t>> pointer: {}\n\t>> token: {:?}\n\t>> tokens: {:?}\n\t>> closing_tags: {:?}",
            //     pointer, token, tokens, closing_tags
            // );

            while let Some(closing_tag) = closing_tags.last() {
                if closing_tag.end <= token.start {
                    // this tag is closed
                    if token.start == token.end {
                        // this token is empty, but is it part of this closing tag?
                        // if not, we can continue to closing tag
                        // if yes, we break here.
                        // and it only happens when its a direct children
                        if closing_tag
                            .children
                            .iter()
                            .any(|child| std::ptr::eq(child, token))
                        {
                            break;
                        }
                    }
                    tokens.push(&self.text[pointer..closing_tag.end]);
                    tokens.push("</");
                    tokens.push(&closing_tag.tag);
                    tokens.push(">");
                    pointer = token.end;
                    closing_tags.pop();
                } else {
                    break;
                }
            }

            tokens.push(&self.text[pointer..token.start]);
            tokens.push("<");
            tokens.push(&token.tag);
            tokens.push(">");

            pointer = token.start;
            closing_tags.push(token);

            // println!(
            //     "------after\n\t>> pointer: {}\n\t>> token: {:?}\n\t>> tokens: {:?}\n\t>> closing_tags: {:?}",
            //     pointer, token, tokens, closing_tags
            // );
        }

        for closing_tag in closing_tags.iter().rev() {
            tokens.push(&self.text[pointer..closing_tag.end]);
            tokens.push("</");
            tokens.push(&closing_tag.tag);
            tokens.push(">");
            pointer = closing_tag.end;
        }
        tokens.push(&self.text[pointer..]);
        tokens.join("")
    }
}

#[pymethods]
impl TextTrace {
    #[getter]
    fn text(&self) -> PyResult<&String> {
        Ok(&self.text)
    }
}

impl TextHTMLElement {
    pub fn shift(&mut self, offset: usize) {
        self.start += offset;
        self.end += offset;
        for child in self.children.iter_mut() {
            child.shift(offset);
        }
    }
}

#[pymethods]
impl TextHTMLElement {
    fn get_attr(&self, name: &str) -> PyResult<Option<&str>> {
        Ok(self.attrs.get(name).map(|s| s.as_str()))
    }

    fn has_attr(&self, name: &str) -> PyResult<bool> {
        Ok(self.attrs.contains_key(name))
    }
}

impl ITree<TextHTMLElement> for TextHTMLElement {
    fn get_root_<'s>(&'s self) -> &'s TextHTMLElement {
        self
    }

    fn get_children_<'s>(&'s self, node: &'s TextHTMLElement) -> &'s [TextHTMLElement] {
        return &node.children;
    }
}
