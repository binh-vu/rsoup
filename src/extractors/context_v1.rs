use crate::{
    context::ContentHierarchy,
    error::TableExtractorError,
    misc::Enum2,
    text::{get_rich_text, rich_text::PSEUDO_TAG, RichText, BLOCK_ELEMENTS},
};

use crate::misc::SimpleTree;
use crate::text::get_rich_text_from_seq;
use anyhow::Result;
use ego_tree::NodeRef;
use hashbrown::HashSet;
use pyo3::prelude::*;
use scraper::Node;

#[derive(Clone)]
#[pyclass(module = "table_extractor.table_extractor")]
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
    pub fn extractor_context<'s>(
        &self,
        table_el: NodeRef<'s, Node>,
    ) -> Result<Vec<ContentHierarchy>> {
        let (tree_before, tree_after) = self.locate_content_before_and_after(table_el)?;

        let mut context_before: Vec<RichText> = vec![];
        let mut context_after: Vec<RichText> = vec![];

        self.flatten_tree(&tree_before, tree_before.get_root_id(), &mut context_before);
        self.flatten_tree(&tree_after, tree_after.get_root_id(), &mut context_after);

        let mut context = vec![ContentHierarchy::new(0, RichText::empty())];
        for c in context_before {
            if self.header_elements.contains(c.get_tag()) {
                let header = c.get_tag()[1..].parse::<usize>().unwrap();
                context.push(ContentHierarchy::new(header, c));
            } else {
                context.last_mut().unwrap().content_before.push(c);
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
            .extend(context_after.into_iter().map(|c| c));

        Ok(context)
    }

    fn flatten_tree(
        &self,
        tree: &SimpleTree<NodeRef<Node>>,
        nodeid: usize,
        output: &mut Vec<RichText>,
    ) {
        let node = tree.get_node(nodeid);
        let node_children = tree.get_child_ids(nodeid);
        if node_children.len() == 0 {
            println!(
                "{:?}",
                get_rich_text(
                    node,
                    &self.ignored_tags,
                    self.only_keep_inline_tags,
                    &self.discard_tags,
                    &self.header_elements,
                )
            );
            output.push(get_rich_text(
                node,
                &self.ignored_tags,
                self.only_keep_inline_tags,
                &self.discard_tags,
                &self.header_elements,
            ));
            // self.flatten_node(node, output);
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
        let mut line: Vec<Enum2<usize, NodeRef<Node>>> = vec![];
        for childid in node_children {
            let child_ref = tree.get_node(*childid);
            if let Some(child_el) = child_ref.value().as_element() {
                if BLOCK_ELEMENTS.contains(child_el.name()) {
                    line.push(Enum2::Type1(*childid));
                } else {
                    line.push(Enum2::Type2(*child_ref));
                }
            } else {
                if child_ref.value().is_text() {
                    line.push(Enum2::Type2(*child_ref));
                }
            }
        }

        let mut lst = vec![];
        for piece in line {
            match piece {
                Enum2::Type1(child_id) => {
                    if lst.len() > 0 {
                        let rich_text = get_rich_text_from_seq(
                            lst,
                            &self.ignored_tags,
                            self.only_keep_inline_tags,
                            &self.discard_tags,
                            &self.header_elements,
                        );
                        if self.is_text_interesting(&rich_text) {
                            output.push(rich_text);
                        }
                        lst = vec![];
                    }
                    self.flatten_tree(tree, child_id, output);
                }
                Enum2::Type2(node_) => {
                    lst.push(node_);
                }
            }
        }
        if lst.len() > 0 {
            let rich_text = get_rich_text_from_seq(
                lst,
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

    // fn flatten_node(&self, node_ref: &NodeRef<Node>, output: &mut Vec<RichText>) {
    //     match node_ref.value() {
    //         Node::Text(text) => output.push(RichText::from_str(text)),
    //         Node::Element(el) => {
    //             if self.discard_tags.contains(el.name()) {
    //                 // skip discard tags
    //                 return;
    //             }

    //             if self.header_elements.contains(el.name()) || !BLOCK_ELEMENTS.contains(el.name()) {
    //                 output.push(get_rich_text(
    //                     node_ref,
    //                     &self.ignored_tags,
    //                     self.only_keep_inline_tags,
    //                     &self.discard_tags,
    //                 ));
    //                 return;
    //             }

    //             let mut line: Vec<Enum2<NodeRef<Node>, NodeRef<Node>>> = vec![];
    //             for child_ref in node_ref.children() {
    //                 if let Some(child_el) = child_ref.value().as_element() {
    //                     if BLOCK_ELEMENTS.contains(child_el.name()) {
    //                         line.push(Enum2::Type1(child_ref));
    //                     } else {
    //                         line.push(Enum2::Type2(child_ref));
    //                     }
    //                 } else {
    //                     if child_ref.value().is_text() {
    //                         line.push(Enum2::Type2(child_ref));
    //                     }
    //                 }
    //             }

    //             let mut lst = vec![];
    //             for piece in line {
    //                 match piece {
    //                     Enum2::Type1(child_ref) => {
    //                         if lst.len() > 0 {
    //                             let rich_text = get_rich_text_from_seq(
    //                                 lst,
    //                                 &self.ignored_tags,
    //                                 self.only_keep_inline_tags,
    //                                 &self.discard_tags,
    //                             );
    //                             if self.is_text_interesting(&rich_text) {
    //                                 output.push(rich_text);
    //                             }
    //                             lst = vec![];
    //                         }
    //                         self.flatten_node(&child_ref, output);
    //                     }
    //                     Enum2::Type2(text) => {
    //                         lst.push(text);
    //                     }
    //                 }
    //             }
    //             if lst.len() > 0 {
    //                 let rich_text = get_rich_text_from_seq(
    //                     lst,
    //                     &self.ignored_tags,
    //                     self.only_keep_inline_tags,
    //                     &self.discard_tags,
    //                 );
    //                 if self.is_text_interesting(&rich_text) {
    //                     output.push(rich_text);
    //                 }
    //             }
    //         }
    //         _ => {}
    //     }
    // }

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
            let parent = parent_ref.value().as_element().ok_or(
                TableExtractorError::InvalidHTMLStructureError(
                    "Parent of an element must be an element",
                ),
            )?;
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
            .ok_or(TableExtractorError::InvalidHTMLStructureError(
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
