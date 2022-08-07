use ego_tree::{NodeRef, Tree};
use hashbrown::{HashMap, HashSet};
use scraper::Node;

use crate::misc::{convert_attrs, SimpleTree};

use super::{
    line::{Line, Paragraph},
    rich_text::{RichText, RichTextElement, PSEUDO_TAG},
    BLOCK_ELEMENTS, INLINE_ELEMENTS,
};

/// Get text from an element as similar as possible to the rendered text.
/// It also returns **descendants** of the element constituting the text.
///
/// For how the browser rendering whitespace, see: https://developer.mozilla.org/en-US/docs/Web/API/Document_Object_Model/Whitespace
///
/// Rules:
/// 1. Each block element is rendered in separated line
/// 2. Empty lines are skipped
/// 3. Consecutive whitespace is collapsed into one space
/// 4. Leading and trailing whitespace is removed
///
/// However, different from the document, leading space within an element is moved to outside of the element.
///
/// For example:
/// * `Hello <a> World</a>` is equivalent to `Hello <a>World</a>`
/// * `Hello<a> World</a>` is equivalent to `Hello <a>World</a>`
///
/// # Arguments
///
/// * `el` - element to extract text from
/// * `ignored_tags` - set of tags to not include in the trace
/// * `only_inline_tags` - whether to only track inline tags
/// * `discard_tags` - set of tags will be discarded and not included in the text
pub fn get_rich_text<'s>(
    el: &'s NodeRef<Node>,
    ignored_tags: &HashSet<String>,
    only_inline_tags: bool,
    discard_tags: &HashSet<String>,
) -> RichText {
    // create a stack-based stream of elements to simulate
    // the rendering process from left to right
    let mut stream = el.children().rev().collect::<Vec<_>>();
    // create a marker to breakline
    let tree = Tree::new(Node::Document);
    let bl_marker = tree.root();

    // create a marker to exit element
    let tree = Tree::new(Node::Fragment);
    let el_marker = tree.root();

    let tmp = if let Some(el_) = el.value().as_element() {
        RichTextElement {
            tag: el_.name().to_owned(),
            start: 0,
            end: 0,
            attrs: convert_attrs(&el_.attrs),
        }
    } else {
        RichTextElement {
            tag: PSEUDO_TAG.to_owned(),
            start: 0,
            end: 0,
            attrs: HashMap::new(),
        }
    };
    let mut element = SimpleTree::new(tmp);

    get_rich_text_from_stream(
        stream,
        element,
        bl_marker,
        el_marker,
        ignored_tags,
        only_inline_tags,
        discard_tags,
    )
}

pub fn get_rich_text_from_seq(
    mut seq: Vec<NodeRef<Node>>,
    ignored_tags: &HashSet<String>,
    only_inline_tags: bool,
    discard_tags: &HashSet<String>,
) -> RichText {
    // create a marker to breakline
    let tree = Tree::new(Node::Document);
    let bl_marker = tree.root();

    // create a marker to exit element
    let tree = Tree::new(Node::Fragment);
    let el_marker = tree.root();

    let mut element = SimpleTree::new(RichTextElement {
        tag: PSEUDO_TAG.to_owned(),
        start: 0,
        end: 0,
        attrs: HashMap::new(),
    });

    get_rich_text_from_stream(
        seq,
        element,
        bl_marker,
        el_marker,
        ignored_tags,
        only_inline_tags,
        discard_tags,
    )
}

fn get_rich_text_from_stream<'s>(
    mut stream: Vec<NodeRef<'s, Node>>,
    mut element: SimpleTree<RichTextElement>,
    bl_marker: NodeRef<'s, Node>,
    el_marker: NodeRef<'s, Node>,
    ignored_tags: &HashSet<String>,
    only_inline_tags: bool,
    discard_tags: &HashSet<String>,
) -> RichText {
    let mut paragraph = Paragraph::with_capacity(stream.len());
    let mut line = Line::with_capacity(stream.len());
    let mut stack_ptrs = vec![(0, element.get_root_id())];

    while let Some(node) = stream.pop() {
        match node.value() {
            Node::Element(node_el) => {
                if discard_tags.contains(node_el.name()) {
                    continue;
                }

                if BLOCK_ELEMENTS.contains(node_el.name()) {
                    // create a newline
                    // (the empty line will be skipped automatically)
                    paragraph.append(&line);
                    line.clear();

                    // put a marker to remember to breakline
                    stream.push(bl_marker);
                }

                if !ignored_tags.contains(node_el.name())
                    && (!only_inline_tags || (INLINE_ELEMENTS.contains(node_el.name())))
                {
                    // enter this element and track it
                    // due to leading space of element will be moved outside, we have to keep
                    // track of the token index (store that in start), and use end to keep track of start
                    let text_el = RichTextElement {
                        tag: node_el.name().to_string(),
                        start: paragraph.tokens.len() + line.tokens.len(),
                        end: paragraph.len() + line.len(),
                        attrs: convert_attrs(&node_el.attrs),
                    };
                    let node_id = element.add_node(text_el);
                    element.add_child(stack_ptrs.last().unwrap().1, node_id);
                    stack_ptrs.push((stream.len(), node_id));

                    // put a marker to remember when to exit the element
                    stream.push(el_marker);
                }

                // the children of the element are added to the stream for further processing
                stream.extend(node.children().rev());
            }
            Node::Text(text) => {
                line.append(&text);
            }
            Node::Document => {
                // may be we are here because of an iframe (haven't tested) or a marker
                // we put to breakline after escaping a block element
                paragraph.append(&line);
                line.clear();

                if node.has_children() {
                    stream.push(bl_marker);
                    stream.extend(node.children().rev());
                }
            }
            Node::Fragment => {
                // i don't know when we may have a doc fragment except the marker we put here intentionally
                // so if it's not our marker, we skip it
                if stream.len() == stack_ptrs.last().unwrap().0 {
                    // this is our marker, we exit the current element
                    let mut text_el = element.get_node_mut(stack_ptrs.pop().unwrap().1);

                    // here we re-adjust the range of the element
                    // as previous we use the index of token not index of character
                    let start_token = text_el.start;
                    let mut start_pos = text_el.end;

                    if paragraph.tokens.len() > start_token {
                        // this means the line that containing the first character of text_el was merged into the paragraph
                        if paragraph.tokens[start_token] == " " {
                            // skip the leading space (always one space as consecutive spaces are merged)
                            start_pos += 1;
                        }
                    } else {
                        // the line is not finished yet.
                        let line_token = start_token - paragraph.tokens.len();
                        if line_token < line.tokens.len() && line.tokens[line_token] == " " {
                            start_pos += 1
                        }
                    };
                    text_el.start = start_pos;
                    text_el.end = paragraph.len() + line.len();
                }
            }
            _ => {
                // doctype, comment are ignored
            }
        }
    }

    paragraph.append(&line);
    let text = paragraph.to_string();
    element.get_root_mut().end = text.len();

    RichText { text, element }
}
