use ego_tree::{NodeRef, Tree};
use scraper::Node;

use super::{line::Line, BLOCK_ELEMENTS};

/// Get text from an element as similar as possible to the rendered text.
///
/// For how the browser rendering whitespace, see: https://developer.mozilla.org/en-US/docs/Web/API/Document_Object_Model/Whitespace
///
/// Rules:
/// 1. Each block element is rendered in separated line
/// 2. Empty lines are skipped
/// 3. Consecutive whitespace is collapsed into one space
/// 4. Leading and trailing whitespace is removed
///
/// # Arguments
///
/// * `el` - element to extract text from
pub fn get_text(el: &NodeRef<Node>) -> String {
    let mut stream = el.children().rev().collect::<Vec<_>>();
    let mut paragraph = Vec::with_capacity(stream.len());
    let mut line = Line::with_capacity(stream.len());

    // create a marker to breakline
    let tree = Tree::new(Node::Document);
    let bl_marker = tree.root();

    while let Some(node) = stream.pop() {
        match node.value() {
            Node::Element(node_el) => {
                if BLOCK_ELEMENTS.contains(node_el.name()) {
                    // create a newline if the current line is not empty
                    // (the empty line will be skipped)
                    if line.tokens.len() > 0 {
                        paragraph.extend(line.tokens.iter());
                        paragraph.push("\n");
                    }
                    line.clear();

                    // put a marker to remember to breakline
                    stream.push(bl_marker);
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
                // unimplemented!()
                if line.tokens.len() > 0 {
                    paragraph.extend(line.tokens.iter());
                    paragraph.push("\n");
                }
                line.clear();

                if node.has_children() {
                    stream.push(bl_marker);
                    stream.extend(node.children().rev());
                }
            }
            _ => {
                // fragment, doctype, comment are ignored
            }
        }
    }

    if line.tokens.len() > 0 {
        paragraph.extend(line.tokens.iter());
        paragraph.push("\n");
    }
    paragraph.pop(); // remove the extra \n character
    paragraph.join("")
}
