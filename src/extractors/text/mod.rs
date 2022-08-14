mod get_rich_text;
pub mod get_text_v1;
pub mod get_text_v2;
pub mod line;
use phf::{phf_set, Set};

pub use self::get_rich_text::{get_rich_text, get_rich_text_from_seq};
pub use self::get_text_v2::get_text;

/// list of inline elements that will be rendered in same line except <br> tags
/// https://developer.mozilla.org/en-US/docs/Web/HTML/Inline_elements
/// don't use it for now
pub static INLINE_ELEMENTS: Set<&'static str> = phf_set! {
    "a", "abbr", "acronym", "audio", "b",
    "bdi", "bdo", "big", "button", "cite", "canvas",
    "code", "data", "datalist", "del", "dfn", "em",
    "embed", "i", "iframe", "img", "input", "ins",
    "kbd", "label", "map", "mark", "meter",
    "object", "output", "picture", "progress", "q",
    "ruby", "s", "samp", "select", "slot",
    "small", "span", "strong", "sub", "sup", "svg", "template",
    "textarea", "time", "u", "tt", "var", "video", "wbr"
};

/// list of block elements
pub static BLOCK_ELEMENTS: Set<&'static str> = phf_set! {
    "body", "br", "address", "article", "aside",
    "blockquote", "details", "dialog", "dd", "div",
    "dl", "dt", "fieldset", "figcaption", "figure",
    "footer", "form", "h1", "h2", "h3", "h4", "h5",
    "h6", "header", "hgroup", "hr", "li", "main",
    "nav", "ol", "p", "pre", "section", "table", "ul"
};
