use crate::{
    context::ContentHierarchy,
    error::TableExtractorError,
    helper::{convert_attrs, Enum2, ITree},
    text::{get_text_with_trace, TextHTMLElement, TextTrace, BLOCK_ELEMENTS},
};

use anyhow::Result;
use ego_tree::NodeRef;
use hashbrown::HashSet;
use scraper::Node;

use super::subtree::SubTree;

pub struct ContextExtractor {
    // do not include those tags in the text trace
    ignored_tags: HashSet<String>,
    // do not include those tags in the context
    discard_tags: HashSet<String>,
    same_content_level_elements: HashSet<String>,
    header_elements: HashSet<String>,

    // whether to only keep inline tags in the text trace
    only_keep_inline_tags: bool,
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
    pub fn extractor_context<'s>(
        &self,
        table_el: NodeRef<'s, Node>,
    ) -> Result<Vec<ContentHierarchy>> {
        let (tree_before, tree_after) = self.locate_content_before_and_after(table_el)?;

        let mut context_before: Vec<InclusiveTextTrace> = vec![];
        let mut context_after: Vec<InclusiveTextTrace> = vec![];
        self.flatten_tree(&tree_before, *tree_before.get_root(), &mut context_before);
        self.flatten_tree(&tree_after, *tree_after.get_root(), &mut context_after);

        let mut context = vec![ContentHierarchy::new(0, TextTrace::empty())];
        for c in context_before {
            if self.header_elements.contains(c.get_tag()) {
                let header = c.get_tag()[1..].parse::<usize>().unwrap();
                context.push(ContentHierarchy::new(header, c.0));
            } else {
                context.last_mut().unwrap().content_before.push(c.0);
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
        context
            .last_mut()
            .unwrap()
            .content_after
            .extend(context_after.into_iter().map(|c| c.0));

        Ok(context)
    }

    fn flatten_tree(&self, tree: &SubTree, nodeid: usize, output: &mut Vec<InclusiveTextTrace>) {
        let node = tree.get_node(nodeid);
        let node_children = tree.get_children(&nodeid);
        if node_children.len() == 0 {
            self.flatten_node(node, output);
            return;
        }

        let node_el = node.value().as_element().unwrap();
        if !BLOCK_ELEMENTS.contains(node_el.name()) {
            // inline element, but why it's here with a subtree?
            // this should never happen
            // silent the error for now
            for childid in node_children {
                self.flatten_tree(tree, *childid, output);
            }
            return;
        }

        // block element, have to check its children
        let mut line: Vec<Enum2<usize, InclusiveTextTrace>> = vec![];
        for childid in node_children {
            let child_ref = tree.get_node(*childid);
            if let Some(child_el) = child_ref.value().as_element() {
                if BLOCK_ELEMENTS.contains(child_el.name()) {
                    line.push(Enum2::Type1(*childid));
                } else {
                    line.push(Enum2::Type2(InclusiveTextTrace::from_element(
                        &child_ref,
                        &self.ignored_tags,
                        self.only_keep_inline_tags,
                        &self.discard_tags,
                    )));
                }
            } else {
                if child_ref.value().is_text() {
                    line.push(Enum2::Type2(InclusiveTextTrace::from_text(&child_ref)));
                }
            }
        }

        let mut flag = false;
        for piece in line {
            match piece {
                Enum2::Type1(child_id) => {
                    self.flatten_tree(tree, child_id, output);
                    flag = false;
                }
                Enum2::Type2(text) => {
                    if flag {
                        output.last_mut().unwrap().0.merge(text.0);
                    } else {
                        output.push(text)
                    }
                    flag = true;
                }
            }
        }
    }

    fn flatten_node(&self, node_ref: &NodeRef<Node>, output: &mut Vec<InclusiveTextTrace>) {
        match node_ref.value() {
            Node::Text(_) => output.push(InclusiveTextTrace::from_text(node_ref)),
            Node::Element(el) => {
                if self.discard_tags.contains(el.name()) {
                    // skip discard tags
                    return;
                }

                if self.header_elements.contains(el.name()) || !BLOCK_ELEMENTS.contains(el.name()) {
                    output.push(InclusiveTextTrace::from_element(
                        node_ref,
                        &self.ignored_tags,
                        self.only_keep_inline_tags,
                        &self.discard_tags,
                    ));
                    return;
                }

                // block element, have to check its children
                let mut line: Vec<Enum2<NodeRef<Node>, InclusiveTextTrace>> = vec![];
                for child_ref in node_ref.children() {
                    if let Some(child_el) = child_ref.value().as_element() {
                        if BLOCK_ELEMENTS.contains(child_el.name()) {
                            line.push(Enum2::Type1(child_ref));
                        } else {
                            line.push(Enum2::Type2(InclusiveTextTrace::from_element(
                                &child_ref,
                                &self.ignored_tags,
                                self.only_keep_inline_tags,
                                &self.discard_tags,
                            )));
                        }
                    } else {
                        if child_ref.value().is_text() {
                            line.push(Enum2::Type2(InclusiveTextTrace::from_text(&child_ref)));
                        }
                    }
                }

                let mut flag = false;
                for piece in line {
                    match piece {
                        Enum2::Type1(child_ref) => {
                            self.flatten_node(&child_ref, output);
                            flag = false;
                        }
                        Enum2::Type2(text) => {
                            if flag {
                                output.last_mut().unwrap().0.merge(text.0);
                            } else {
                                output.push(text)
                            }
                            flag = true;
                        }
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
    fn locate_content_before_and_after<'s>(
        &self,
        element: NodeRef<'s, Node>,
    ) -> Result<(SubTree<'s>, SubTree<'s>)> {
        let mut el = element;
        let mut tree_before = SubTree::new();
        let mut tree_after = SubTree::new();

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
            if e.is_element()
                && self
                    .same_content_level_elements
                    .contains(e.as_element().unwrap().name())
            {
                break;
            }
            tree_after.add_new_child(root_id, eref);
        }

        Ok((tree_before, tree_after))
    }
}

pub struct InclusiveTextTrace(TextTrace);

impl InclusiveTextTrace {
    #[inline(always)]
    fn from_text(node: &NodeRef<Node>) -> InclusiveTextTrace {
        let text = node.value().as_text().unwrap();
        InclusiveTextTrace(TextTrace::from_str(text))
    }

    #[inline(always)]
    fn from_element(
        node: &NodeRef<Node>,
        ignored_tags: &HashSet<String>,
        only_inline_tags: bool,
        discard_tags: &HashSet<String>,
    ) -> InclusiveTextTrace {
        let el = node.value().as_element().unwrap();

        let mut text = get_text_with_trace(node, ignored_tags, only_inline_tags, discard_tags);
        let tmp = TextHTMLElement {
            tag: el.name().to_string(),
            attrs: convert_attrs(&el.attrs),
            start: 0,
            end: text.text.len(),
            children: text.trace,
        };
        text.trace = vec![tmp];

        InclusiveTextTrace(text)
    }

    // fn from_node(
    //     node: &NodeRef<Node>,
    //     ignored_tags: &HashSet<String>,
    //     only_inline_tags: bool,
    //     discard_tags: &HashSet<String>,
    // ) -> Option<InclusiveTextTrace> {
    //     match node.value() {
    //         Node::Text(text) => Some(InclusiveTextTrace(TextTrace::from_str(text))),
    //         Node::Element(el) => {
    //             if discard_tags.contains(el.name()) {
    //                 return None;
    //             }

    //             let mut text =
    //                 get_text_with_trace(node, ignored_tags, only_inline_tags, discard_tags);
    //             let tmp = TextHTMLElement {
    //                 tag: el.name().to_string(),
    //                 attrs: convert_attrs(&el.attrs),
    //                 start: 0,
    //                 end: text.text.len(),
    //                 children: text.trace,
    //             };
    //             text.trace = vec![tmp];

    //             Some(InclusiveTextTrace(text))
    //         }
    //         _ => None,
    //     }
    // }

    fn get_tag(&self) -> &str {
        self.0.trace[0].tag.as_str()
    }
}
