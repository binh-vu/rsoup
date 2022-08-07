use ego_tree::{NodeRef, Tree};
use regex::Regex;
use scraper::Node;

use super::BLOCK_ELEMENTS;

lazy_static! {
    static ref RE_WHITESPACE: Regex = Regex::new(r"\s+").unwrap();
}

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
    // create a stack-based stream of elements to simulate
    // the rendering process from left to right
    let mut stream = el.children().rev().collect::<Vec<_>>();
    let mut lines = Vec::with_capacity(stream.len());
    let mut current_line: Vec<String> = Vec::with_capacity(stream.len());

    // create a marker to breakline
    let tree = Tree::new(Node::Document);
    let marker = tree.root();

    while let Some(node) = stream.pop() {
        match node.value() {
            Node::Element(node_el) => {
                if BLOCK_ELEMENTS.contains(node_el.name()) {
                    // create a newline if the current line is not empty
                    // (the empty line will be skipped)
                    let line = process_line(&mut current_line);
                    if line.len() > 0 {
                        lines.push(line);
                    }
                    current_line.clear();

                    // put a marker to remember to breakline
                    stream.push(marker);
                }

                // the children of the element are added to the stream for further processing
                stream.extend(node.children().rev());
            }
            Node::Text(text) => {
                current_line.push(text.text.to_string());
            }
            Node::Document => {
                // may be we are here because of an iframe (haven't tested) or a marker
                // we put to breakline after escaping a block element
                let line = process_line(&mut current_line);
                if line.len() > 0 {
                    lines.push(line);
                }
                current_line.clear();

                if node.has_children() {
                    stream.push(marker);
                    stream.extend(node.children().rev());
                }
            }
            _ => {
                // fragment, doctype, comment are ignored
            }
        }
    }

    if current_line.len() > 0 {
        let line = process_line(&mut current_line);
        if line.len() > 0 {
            lines.push(line);
        }
    }

    lines.join("\n")
}

/// Merge tokens in the line into a single string, and:
/// 1. Multiple consecutive spaces/tab/newlines are replaced by a single space.
/// 2. Leading and trailing spaces are removed.
#[inline(always)]
fn process_line(line: &mut [String]) -> String {
    if line.len() == 0 {
        return String::new();
    }

    if line.len() == 1 {
        return RE_WHITESPACE.replace_all(line[0].trim(), " ").to_string();
    }

    let i1 = line.len() - 1;
    line[0] = line[0].trim_start().to_owned();
    line[i1] = line[i1].trim_end().to_owned();
    RE_WHITESPACE.replace_all(&line.join(""), " ").to_string()
}
