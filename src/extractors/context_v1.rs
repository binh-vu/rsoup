use crate::{context::ContentHierarchy, error::TableExtractorError, text::TextTrace};

use anyhow::Result;
use ego_tree::NodeRef;
use hashbrown::HashMap;
use phf::{phf_set, Set};
use scraper::{ElementRef, Html, Node};

static SAME_CONTENT_LEVEL_ELEMENTS: Set<&'static str> =
    phf_set! {"table", "h1", "h2", "h3", "h4", "h5", "h6"};

/// Extracting context that leads to an element in an HTML page
///
/// Assuming that the page follows tree structure. Each header element
/// represents a level (section) in the tree.
///
/// This extractor tries to does it best to detect which text should be kept in the same line
/// and which one is not. However, it does not take into account the style of element (display: block)
/// and hence has to rely on some heuristics. For example, <canvas> is an inline element, however, it
/// is often used as block element so this extractor put it in another line.
pub fn extractor_context<'s>(tree: &Html, table_el: NodeRef<'s, Node>) -> Result<()> {
    let (tree_before, tree_after) = locate_content_before_and_after(table_el)?;

    let content_before = flatten_tree(tree_before);
    let content_after = flatten_tree(tree_after);

    Ok(())
}

/// Flatten the tree into
fn flatten_tree(tree: &TreeBuilder) -> Vec<WrappedTextTrace> {
    let output = Vec::new();
    if tree.is_empty() {
        return output;
    }

    // traverse the tree
    // for each node, if it is a header, we keep its value separated it
    // then, for each element, we convert its to a text trace. what about
    // a parent that we don't have all children?
    // we create a subtree, then we re-call a function to get text trace.
    // after that, we

    return output;
}

/// Finding surrounding content of the element.
///
/// Assuming elements in the document is rendered from top to bottom and
/// left to right. In other words, there is no CSS that do float right/left
/// to make pre/after elements to be appeared out of order.
///
/// Currently, (the logic is not good)
///     * to determine the content before the element, we just keep all elements rendered
/// before this element (we are doing another filter outside of this function in `self.extract`).
///     * to determine the content after the element, we consider only the siblings
/// and stop before they hit a block element (not all block elements) that may be in the same level such as table, etc.
fn locate_content_before_and_after<'s>(
    element: NodeRef<'s, Node>,
) -> Result<(TreeBuilder<'s>, TreeBuilder<'s>)> {
    let mut el = element;
    let mut tree_before = TreeBuilder::new();
    let mut tree_after = TreeBuilder::new();

    while let Some(parent_ref) = el.parent() {
        let parent = parent_ref.value().as_element().ok_or(
            TableExtractorError::InvalidHTMLStructureError(
                "Parent of an element must be an element",
            ),
        )?;
        if parent.name() != "html" {
            break;
        }

        let node = tree_before.add_node(parent_ref);
        for e in parent_ref.children() {
            if e.id() == el.id() {
                // this is the index
                if !tree_before.is_empty() {
                    tree_before.add_existing_child(node, tree_before.root);
                }
                break;
            }
            tree_before.add_new_child(node, e);
        }
        tree_before.root = node;
        el = parent_ref;
    }

    let root = element
        .parent()
        .ok_or(TableExtractorError::InvalidHTMLStructureError(
            "The element we want to locate cannot be a root node",
        ))?;
    let root_id = tree_after.add_node(root);

    for eref in element.next_siblings() {
        let e = eref.value();
        if e.is_element() && SAME_CONTENT_LEVEL_ELEMENTS.contains(e.as_element().unwrap().name()) {
            break;
        }
        tree_after.add_new_child(root_id, eref);
    }

    Ok((tree_before, tree_after))
}

struct TreeBuilder<'s> {
    root: usize,
    nodes: Vec<NodeRef<'s, Node>>,
    id2children: Vec<Vec<usize>>,
}

impl<'s> TreeBuilder<'s> {
    fn new() -> Self {
        TreeBuilder {
            root: 0,
            nodes: Vec::new(),
            id2children: Vec::new(),
        }
    }

    fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    fn add_node(&mut self, node: NodeRef<'s, Node>) -> usize {
        let nodeid = self.nodes.len();
        self.nodes.push(node);
        self.id2children.push(Vec::new());
        nodeid
    }

    fn add_new_child(&mut self, parent_id: usize, node: NodeRef<'s, Node>) -> usize {
        let nodeid = self.add_node(node);
        self.id2children[parent_id].push(nodeid);
        nodeid
    }

    fn add_existing_child(&mut self, parent_id: usize, child_id: usize) {
        self.id2children[parent_id].push(child_id);
    }
}

struct TreeBuilderTraversal<'s, 't> {
    tree: &'s TreeBuilder<'t>,
    stack: Vec<(usize, usize)>,
}

impl<'s, 't> Iterator for TreeBuilderTraversal<'s, 't> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.stack.len() == 0 {
                // we done
                return None;
            }

            let n1 = self.stack.len() - 1;
            let (node, child_index) = self.stack[n1];
            if child_index < self.tree.id2children[node].len() {
                // add this child into the stack and mark this child has been returned
                let child = self.tree.id2children[node][child_index];
                self.stack.push((child, 0));
                self.stack[n1].1 += 1;
                return Some(child);
            }

            // no child can be returned, we done at this level
            self.stack.pop();
        }
    }
}

struct WrappedTextTrace {
    tag: String,
    attrs: HashMap<String, String>,
    text: TextTrace,
}
