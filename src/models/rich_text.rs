use hashbrown::HashMap;
use pyo3::exceptions::PyKeyError;
use std::fmt;

use pyo3::{prelude::*, types::PyDict, types::PyList};
use serde::{Deserialize, Serialize};

use crate::misc::range_iter::RangeIter;
use crate::misc::tree::iterator::ITree;
use crate::misc::tree::simple_tree::SimpleTree;

pub const PSEUDO_TAG: &str = "";

#[pyclass(module = "rsoup.core")]
#[derive(Clone, PartialEq, Eq, Deserialize, Serialize)]
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
#[pyclass(module = "rsoup.core")]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct RichTextElement {
    #[pyo3(get)]
    pub tag: String,
    #[pyo3(get)]
    pub start: usize,
    #[pyo3(get)]
    pub end: usize,
    #[pyo3(get)]
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
}

#[pymethods]
impl RichText {
    #[staticmethod]
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

    pub fn len(&self) -> usize {
        self.text.len()
    }

    pub fn iter_element_id(&self) -> RangeIter {
        RangeIter {
            start: 0,
            end: self.element.len(),
        }
    }

    pub fn iter_element_id_preorder(
        slf: Py<RichText>,
        py: Python,
    ) -> RichTextElementIdPreorderIter {
        RichTextElementIdPreorderIter::new(slf.clone_ref(py))
    }

    pub fn get_element_tag_by_id(&self, id: usize) -> String {
        self.element.get_node(id).tag.clone()
    }

    pub fn get_element_by_id(&self, id: usize) -> RichTextElement {
        self.element.get_node(id).clone()
    }

    pub fn set_element_by_id(&mut self, id: usize, element: RichTextElement) {
        self.element.update_node(id, element);
    }

    pub fn set_element_attr_by_id(&mut self, id: usize, attr: &str, value: &str) {
        self.element
            .get_node_mut(id)
            .attrs
            .insert(attr.to_owned(), value.to_owned());
    }

    pub fn get_element_attr_by_id(&self, id: usize, attr: &str) -> Option<String> {
        self.element
            .get_node(id)
            .attrs
            .get(attr)
            .map(ToOwned::to_owned)
    }

    #[args("*", render_outer_element = "true", render_element_attrs = "false")]
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
            // println!(
            //     "------before\n\t>> pointer: {}\n\t>> token: {:?}\n\t>> tokens: {:?}\n\t>> closing_tags: {:?}",
            //     pointer, token, tokens, closing_tag_ids.iter().map(|id| self.element.get_node(*id)).collect::<Vec<_>>()
            // );

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

            // println!(
            //     "------after\n\t>> pointer: {}\n\t>> token: {:?}\n\t>> tokens: {:?}\n\t>> closing_tags: {:?}",
            //     pointer, token, tokens, closing_tag_ids.iter().map(|id| self.element.get_node(*id)).collect::<Vec<_>>()
            // );
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

    pub fn to_dict(&self, py: Python) -> PyResult<Py<PyDict>> {
        let tree = PyDict::new(py);

        tree.set_item("root", self.element.get_root_id())?;
        tree.set_item(
            "nodes",
            self.element
                .iter()
                .iter()
                .map(|u| u.to_dict(py))
                .collect::<PyResult<Vec<_>>>()?,
        )?;
        tree.set_item("node2children", &self.element.node2children)?;

        let d = PyDict::new(py);
        d.set_item("text", &self.text)?;
        d.set_item("element", tree)?;
        Ok(d.into_py(py))
    }

    #[staticmethod]
    pub fn from_dict(obj: &PyDict) -> PyResult<Self> {
        let text = obj
            .get_item("text")
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("text"))?
            .extract::<String>()?;

        let elem_obj = obj
            .get_item("element")
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("element"))?
            .downcast::<PyDict>()?;
        let root = elem_obj
            .get_item("root")
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("root in element"))?
            .extract::<usize>()?;
        let nodes = elem_obj
            .get_item("nodes")
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("nodes in element"))?
            .downcast::<PyList>()?
            .iter()
            .map(|o| RichTextElement::from_dict(o.downcast::<PyDict>()?))
            .collect::<PyResult<Vec<_>>>()?;
        let node2children = elem_obj
            .get_item("node2children")
            .ok_or_else(|| {
                PyErr::new::<pyo3::exceptions::PyKeyError, _>("node2children in element")
            })?
            .extract::<Vec<Vec<usize>>>()?;

        Ok(RichText {
            text,
            element: SimpleTree::from_data(root, nodes, node2children),
        })
    }
}

#[pymethods]
impl RichTextElement {
    fn get_attr(&self, name: &str) -> PyResult<&String> {
        self.attrs
            .get(name)
            .ok_or_else(|| PyKeyError::new_err(format!("{name} not found")))
    }

    fn has_attr(&self, name: &str) -> PyResult<bool> {
        Ok(self.attrs.contains_key(name))
    }

    fn to_dict(&self, py: Python) -> PyResult<Py<PyDict>> {
        let d = PyDict::new(py);
        d.set_item("tag", &self.tag)?;
        d.set_item("start", self.start)?;
        d.set_item("end", self.end)?;
        d.set_item("attrs", &self.attrs)?;
        Ok(d.into_py(py))
    }

    #[staticmethod]
    fn from_dict(obj: &PyDict) -> PyResult<RichTextElement> {
        let tag = obj
            .get_item("tag")
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("tag"))?
            .extract::<String>()?;
        let start = obj
            .get_item("start")
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("start"))?
            .extract::<usize>()?;
        let end = obj
            .get_item("end")
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("end"))?
            .extract::<usize>()?;
        let attrs = obj
            .get_item("attrs")
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("attrs"))?
            .extract::<HashMap<String, String>>()?;
        Ok(RichTextElement {
            tag,
            start,
            end,
            attrs,
        })
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

#[pyclass(module = "rsoup.core")]
pub struct RichTextElementIdPreorderIter {
    text: Py<RichText>,
    stack: Vec<(usize, usize)>,
    inited: bool,
}

impl RichTextElementIdPreorderIter {
    pub fn new(text: Py<RichText>) -> Self {
        RichTextElementIdPreorderIter {
            text,
            stack: Vec::new(),
            inited: false,
        }
    }
}

#[pymethods]
impl RichTextElementIdPreorderIter {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(&mut self, py: Python) -> Option<usize> {
        let text = self.text.borrow(py);
        loop {
            if self.stack.len() == 0 {
                if self.inited {
                    return None;
                }
                self.inited = true;
                self.stack.push((text.element.get_root_id(), 0));
                return Some(self.stack[self.stack.len() - 1].0);
            }

            // current element has been returned previously
            // so we will try to return its child
            let n1 = self.stack.len() - 1;
            let (node, child_index) = self.stack[n1];
            let node_children = text.element.get_child_ids(node);

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
