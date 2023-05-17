use crate::{
    error::RSoupError,
    extractors::text::{get_rich_text, get_rich_text_from_seq, BLOCK_ELEMENTS},
    misc::{
        recursive_iter::{InvExitingSeqState, InvState, InvTree, RecurInvocationBuilder},
        tree::simple_tree::SimpleTree,
    },
    models::{
        content_hierarchy::ContentHierarchy,
        rich_text::{RichText, PSEUDO_TAG},
    },
};

use anyhow::Result;
use ego_tree::NodeRef;
use hashbrown::HashSet;
use pyo3::prelude::*;
use scraper::Node;

#[derive(Clone)]
#[pyclass(module = "rsoup.core")]
pub struct ContextExtractor {
    // do not include those tags in the rich text
    ignored_tags: HashSet<String>,
    // do not include those tags in the context
    discard_tags: HashSet<String>,
    same_content_level_elements: HashSet<String>,
    header_elements: HashSet<String>,

    // whether to only keep inline tags in the text trace
    only_keep_inline_tags: bool,
}

#[pymethods]
impl ContextExtractor {
    #[new]
    #[args(
        "*",
        ignored_tags = "None",
        discard_tags = "None",
        same_content_level_elements = "None",
        header_elements = "None",
        only_keep_inline_tags = "true"
    )]
    fn new(
        ignored_tags: Option<Vec<&str>>,
        discard_tags: Option<Vec<&str>>,
        same_content_level_elements: Option<Vec<&str>>,
        header_elements: Option<Vec<&str>>,
        only_keep_inline_tags: bool,
    ) -> Self {
        let discard_tags_ = HashSet::from_iter(
            discard_tags
                .unwrap_or(["script", "style", "noscript", "table"].to_vec())
                .into_iter()
                .map(str::to_owned),
        );
        let ignored_tags_ = HashSet::from_iter(
            ignored_tags
                .unwrap_or(["div"].to_vec())
                .into_iter()
                .map(str::to_owned),
        );
        let same_content_level_elements_ = HashSet::from_iter(
            same_content_level_elements
                .unwrap_or(["table", "h1", "h2", "h3", "h4", "h5", "h6"].to_vec())
                .into_iter()
                .map(str::to_owned),
        );
        let header_elements_ = HashSet::from_iter(
            header_elements
                .unwrap_or(["h1", "h2", "h3", "h4", "h5", "h6"].to_vec())
                .into_iter()
                .map(str::to_owned),
        );

        ContextExtractor {
            ignored_tags: ignored_tags_,
            discard_tags: discard_tags_,
            same_content_level_elements: same_content_level_elements_,
            header_elements: header_elements_,
            only_keep_inline_tags,
        }
    }
}

impl ContextExtractor {
    pub fn default() -> ContextExtractor {
        let discard_tags = HashSet::from_iter(
            ["script", "style", "noscript", "table"]
                .into_iter()
                .map(str::to_owned),
        );
        let ignored_tags = HashSet::from_iter(["div"].into_iter().map(str::to_owned));
        let same_content_level_elements = HashSet::from_iter(
            ["table", "h1", "h2", "h3", "h4", "h5", "h6"]
                .into_iter()
                .map(str::to_owned),
        );
        let header_elements = HashSet::from_iter(
            ["h1", "h2", "h3", "h4", "h5", "h6"]
                .into_iter()
                .map(str::to_owned),
        );

        ContextExtractor {
            ignored_tags,
            discard_tags,
            same_content_level_elements,
            header_elements,
            only_keep_inline_tags: true,
        }
    }

    /// Extracting context that leads to an element in an HTML page
    ///
    /// Assuming that the page follows tree structure. Each header element
    /// represents a level (section) in the tree.
    ///
    /// This extractor tries to does it best to detect which text should be kept in the same line
    /// and which one is not. However, it does not take into account the style of element (display: block)
    /// and hence has to rely on some heuristics. For example, <canvas> is an inline element, however, it
    /// is often used as block element so this extractor put it in another line.
    pub fn extract_context<'s>(
        &self,
        py: Python,
        table_el: NodeRef<'s, Node>,
    ) -> Result<Vec<ContentHierarchy>> {
        let (tree_before, tree_after) = self.locate_content_before_and_after(table_el)?;

        let mut context_before: Vec<RichText> = vec![];
        let mut context_after: Vec<RichText> = vec![];

        self.flatten_tree_recur(&tree_before, tree_before.get_root_id(), &mut context_before);
        self.flatten_tree_recur(&tree_after, tree_after.get_root_id(), &mut context_after);
        // self.flatten_tree(&tree_before, &mut context_before);
        // self.flatten_tree(&tree_after, &mut context_after);

        let mut context = vec![ContentHierarchy::new(0, Py::new(py, RichText::empty())?)];
        for c in context_before {
            if self.header_elements.contains(c.get_tag()) {
                let header = c.get_tag()[1..].parse::<usize>().unwrap();
                context.push(ContentHierarchy::new(header, Py::new(py, c)?));
            } else {
                context
                    .last_mut()
                    .unwrap()
                    .content_before
                    .push(Py::new(py, c)?);
                continue;
            }
        }

        // we do another filter to make sure the content is related to the element
        // that the header leading to this element must be increasing
        let mut rev_context = vec![];
        let mut header = 10;
        for c in context.into_iter().rev() {
            if c.level < header {
                header = c.level;
                rev_context.push(c);
            }
        }
        rev_context.reverse();
        context = rev_context;
        context.last_mut().unwrap().content_after.extend(
            context_after
                .into_iter()
                .map(|c| Py::new(py, c))
                .collect::<PyResult<Vec<_>>>()?,
        );

        Ok(context)
    }

    pub fn flatten_tree(&self, tree: &SimpleTree<NodeRef<Node>>, output: &mut Vec<RichText>) {
        let mut inv_tree = InvTree::new(vec![tree.get_root_id()]);
        let mut pending_ops = Vec::new();

        while let Some(inv) = inv_tree.next() {
            match inv.state {
                InvState::Entering(nodeid) => {
                    let node = tree.get_node(nodeid);
                    let node_children = tree.get_child_ids(nodeid);
                    if node_children.len() == 0 {
                        self.flatten_node(node, output);
                        continue;
                    }

                    let node_el = node.value().as_element().unwrap();
                    if !BLOCK_ELEMENTS.contains(node_el.name()) {
                        // inline element, but why it's here with a subtree?
                        // this should never happen
                        // silent the error for now
                        let mut next_invs = RecurInvocationBuilder::new();
                        for child_id in node_children {
                            next_invs.push(0, *child_id);
                        }
                        inv_tree.add_recur_invocations(
                            &inv,
                            InvExitingSeqState::new(),
                            next_invs.return_ids,
                            next_invs.invocations,
                        );
                        continue;
                    }

                    // block element, have to check its children
                    let mut exiting_state = InvExitingSeqState::new();
                    let mut next_invs = RecurInvocationBuilder::new();

                    for child_id in node_children {
                        let child_ref = tree.get_node(*child_id);
                        match child_ref.value() {
                            Node::Element(child_el) => {
                                if !BLOCK_ELEMENTS.contains(child_el.name()) {
                                    pending_ops.push(*child_ref);
                                    continue;
                                }

                                if pending_ops.len() > 0 {
                                    let rich_text = get_rich_text_from_seq(
                                        pending_ops,
                                        &self.ignored_tags,
                                        self.only_keep_inline_tags,
                                        &self.discard_tags,
                                        &self.header_elements,
                                    );
                                    if self.is_text_interesting(&rich_text) {
                                        if next_invs.len() > 0 {
                                            exiting_state.push(rich_text);
                                        } else {
                                            output.push(rich_text);
                                        }
                                    }
                                    pending_ops = Vec::new();
                                }

                                // put the next node here
                                next_invs.push(exiting_state.len(), *child_id);
                            }
                            Node::Text(_) => {
                                pending_ops.push(*child_ref);
                            }
                            _ => {}
                        }
                    }

                    if pending_ops.len() > 0 {
                        let rich_text = get_rich_text_from_seq(
                            pending_ops,
                            &self.ignored_tags,
                            self.only_keep_inline_tags,
                            &self.discard_tags,
                            &self.header_elements,
                        );
                        if self.is_text_interesting(&rich_text) {
                            if next_invs.len() > 0 {
                                exiting_state.push(rich_text);
                            } else {
                                output.push(rich_text);
                            }
                        }
                        pending_ops = Vec::new();
                    }

                    if next_invs.invocations.len() > 0 {
                        inv_tree.add_recur_invocations(
                            &inv,
                            exiting_state,
                            next_invs.return_ids,
                            next_invs.invocations,
                        );
                    }
                }
                InvState::Exiting(exiting_state) => {
                    if let Some(parent_id) = inv.parent_id {
                        let parent_exit_state = inv_tree.get_mut_parent_state(parent_id);
                        for _ in parent_exit_state.n_consumed..inv.return_id {
                            output.push(parent_exit_state.pop());
                        }
                    }
                    output.extend(exiting_state.consume());
                }
            }
        }
    }

    pub fn flatten_node(&self, node_ref: &NodeRef<Node>, output: &mut Vec<RichText>) {
        let mut inv_tree = InvTree::new(vec![*node_ref]);
        let mut pending_ops = Vec::new();

        while let Some(inv) = inv_tree.next() {
            match inv.state {
                InvState::Entering(node_ref) => {
                    match node_ref.value() {
                        Node::Element(el) => {
                            if self.discard_tags.contains(el.name()) {
                                // skip discard tags
                                continue;
                            }

                            if self.header_elements.contains(el.name())
                                || !BLOCK_ELEMENTS.contains(el.name())
                            {
                                output.push(get_rich_text(
                                    &node_ref,
                                    &self.ignored_tags,
                                    self.only_keep_inline_tags,
                                    &self.discard_tags,
                                    &self.header_elements,
                                ));
                                continue;
                            }

                            let mut exiting_state = InvExitingSeqState::new();
                            let mut next_invs = RecurInvocationBuilder::new();

                            for child_ref in node_ref.children() {
                                match child_ref.value() {
                                    Node::Element(child_el) => {
                                        if !BLOCK_ELEMENTS.contains(child_el.name()) {
                                            pending_ops.push(child_ref);
                                            continue;
                                        }

                                        if pending_ops.len() > 0 {
                                            let rich_text = get_rich_text_from_seq(
                                                pending_ops,
                                                &self.ignored_tags,
                                                self.only_keep_inline_tags,
                                                &self.discard_tags,
                                                &self.header_elements,
                                            );
                                            if self.is_text_interesting(&rich_text) {
                                                if next_invs.len() > 0 {
                                                    exiting_state.push(rich_text);
                                                } else {
                                                    output.push(rich_text);
                                                }
                                            }
                                            pending_ops = Vec::new();
                                        }

                                        // put the next node here
                                        next_invs.push(exiting_state.len(), child_ref);
                                    }
                                    Node::Text(_) => {
                                        pending_ops.push(child_ref);
                                    }
                                    _ => {}
                                }
                            }

                            if pending_ops.len() > 0 {
                                let rich_text = get_rich_text_from_seq(
                                    pending_ops,
                                    &self.ignored_tags,
                                    self.only_keep_inline_tags,
                                    &self.discard_tags,
                                    &self.header_elements,
                                );
                                if self.is_text_interesting(&rich_text) {
                                    if next_invs.len() > 0 {
                                        exiting_state.push(rich_text);
                                    } else {
                                        output.push(rich_text);
                                    }
                                }
                                pending_ops = Vec::new();
                            }
                            // println!(">>> Before add recur.{}", inv_tree.debug_info());
                            if next_invs.invocations.len() > 0 {
                                inv_tree.add_recur_invocations(
                                    &inv,
                                    exiting_state,
                                    next_invs.return_ids,
                                    next_invs.invocations,
                                );
                            }
                            // println!(">>> After add recur.{}", inv_tree.debug_info());
                        }
                        _ => unreachable!(),
                    }
                }
                InvState::Exiting(exiting_state) => {
                    // println!(
                    //     "Entering exit of node: {}. Debug info: {}",
                    //     inv_tree.stack.len(),
                    //     inv_tree.debug_info()
                    // );

                    // resume the code here.
                    // clear out previous pending ops
                    if let Some(parent_id) = inv.parent_id {
                        let parent_exit_state = inv_tree.get_mut_parent_state(parent_id);
                        // println!("{}..{}", parent_exit_state.n_consumed, inv.return_id);
                        for _ in parent_exit_state.n_consumed..inv.return_id {
                            output.push(parent_exit_state.pop());
                        }
                    }
                    // println!("{:?}", exiting_state);
                    output.extend(exiting_state.consume());
                }
            }
        }
    }

    pub fn flatten_tree_recur(
        &self,
        tree: &SimpleTree<NodeRef<Node>>,
        nodeid: usize,
        output: &mut Vec<RichText>,
    ) {
        let node = tree.get_node(nodeid);
        let node_children = tree.get_child_ids(nodeid);
        if node_children.len() == 0 {
            self.flatten_node_recur(node, output);
            return;
        }

        let node_el = node.value().as_element().unwrap();
        if !BLOCK_ELEMENTS.contains(node_el.name()) {
            // inline element, but why it's here with a subtree?
            // this should never happen
            // silent the error for now
            for childid in node_children {
                self.flatten_tree_recur(tree, *childid, output);
            }
            return;
        }

        // block element, have to check its children
        let mut pending_ops = Vec::new();
        for child_id in node_children {
            let child_ref = tree.get_node(*child_id);
            match child_ref.value() {
                Node::Text(_) => pending_ops.push(*child_ref),
                Node::Element(child_el) => {
                    if !BLOCK_ELEMENTS.contains(child_el.name()) {
                        pending_ops.push(*child_ref);
                        continue;
                    }

                    if pending_ops.len() > 0 {
                        let rich_text = get_rich_text_from_seq(
                            pending_ops,
                            &self.ignored_tags,
                            self.only_keep_inline_tags,
                            &self.discard_tags,
                            &self.header_elements,
                        );
                        if self.is_text_interesting(&rich_text) {
                            output.push(rich_text);
                        }
                        pending_ops = Vec::new();
                    }

                    self.flatten_tree_recur(tree, *child_id, output);
                }
                _ => {}
            }
        }

        if pending_ops.len() > 0 {
            let rich_text = get_rich_text_from_seq(
                pending_ops,
                &self.ignored_tags,
                self.only_keep_inline_tags,
                &self.discard_tags,
                &self.header_elements,
            );
            if self.is_text_interesting(&rich_text) {
                output.push(rich_text);
            }
        }
    }

    pub fn flatten_node_recur(&self, node_ref: &NodeRef<Node>, output: &mut Vec<RichText>) {
        match node_ref.value() {
            // should never go into node::text
            Node::Text(text) => output.push(RichText::from_str(text)),
            Node::Element(el) => {
                if self.discard_tags.contains(el.name()) {
                    // skip discard tags
                    return;
                }

                if self.header_elements.contains(el.name()) || !BLOCK_ELEMENTS.contains(el.name()) {
                    output.push(get_rich_text(
                        node_ref,
                        &self.ignored_tags,
                        self.only_keep_inline_tags,
                        &self.discard_tags,
                        &self.header_elements,
                    ));
                    return;
                }

                let mut pending_ops = Vec::new();
                for child_ref in node_ref.children() {
                    match child_ref.value() {
                        Node::Text(_) => pending_ops.push(child_ref),
                        Node::Element(child_el) => {
                            if !BLOCK_ELEMENTS.contains(child_el.name()) {
                                pending_ops.push(child_ref);
                                continue;
                            }

                            if pending_ops.len() > 0 {
                                let rich_text = get_rich_text_from_seq(
                                    pending_ops,
                                    &self.ignored_tags,
                                    self.only_keep_inline_tags,
                                    &self.discard_tags,
                                    &self.header_elements,
                                );
                                if self.is_text_interesting(&rich_text) {
                                    output.push(rich_text);
                                }
                                pending_ops = Vec::new();
                            }

                            self.flatten_node_recur(&child_ref, output);
                        }
                        _ => {}
                    }
                }

                if pending_ops.len() > 0 {
                    let rich_text = get_rich_text_from_seq(
                        pending_ops,
                        &self.ignored_tags,
                        self.only_keep_inline_tags,
                        &self.discard_tags,
                        &self.header_elements,
                    );
                    if self.is_text_interesting(&rich_text) {
                        output.push(rich_text);
                    }
                }
            }
            _ => {}
        }
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
    pub fn locate_content_before_and_after<'s>(
        &self,
        element: NodeRef<'s, Node>,
    ) -> Result<(SimpleTree<NodeRef<'s, Node>>, SimpleTree<NodeRef<'s, Node>>)> {
        let mut el = element;
        let mut tree_before = SimpleTree::empty();
        let mut tree_after = SimpleTree::empty();

        while let Some(parent_ref) = el.parent() {
            let parent =
                parent_ref
                    .value()
                    .as_element()
                    .ok_or(RSoupError::InvalidHTMLStructureError(
                        "Parent of an element must be an element",
                    ))?;
            if parent.name() == "html" {
                break;
            }

            let node = tree_before.add_node(parent_ref);
            for e in parent_ref.children() {
                if e.id() == el.id() {
                    // last item before the `element`
                    if el.id() != element.id() {
                        // we don't want to include `element` itself
                        tree_before.add_child(node, tree_before.get_root_id());
                    }
                    break;
                }
                let child_id = tree_before.add_node(e);
                tree_before.add_child(node, child_id);
            }
            el = parent_ref;
        }

        let root = element
            .parent()
            .ok_or(RSoupError::InvalidHTMLStructureError(
                "The element we want to locate cannot be a root node in HTML doc",
            ))?;
        let root_id = tree_after.add_node(root);

        for eref in element.next_siblings() {
            let e = eref.value();
            if e.is_element()
                && self
                    .same_content_level_elements
                    .contains(e.as_element().unwrap().name())
            {
                break;
            }
            let child_id = tree_after.add_node(eref);
            tree_after.add_child(root_id, child_id);
        }

        Ok((tree_before, tree_after))
    }

    // test if the text is interesting
    pub fn is_text_interesting(&self, text: &RichText) -> bool {
        return !(text.text.is_empty() && text.element.len() == 1 && text.get_tag() == PSEUDO_TAG);
    }
}
