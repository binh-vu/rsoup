use crate::text::TextTrace;

/// Content at each level that leads to the table
pub struct ContentHierarchy {
    // level of the heading, level 0 indicate the beginning of the document
    // but should not be used
    pub level: usize,
    // title of the level (header)
    pub heading: TextTrace,
    // partially HTML content, normalized <a>, <b>, <i> tags (breaklines or block text such as div, p are converted to line breaks)
    // other HTML containing content such as <table>, <img>, <video>, <audio> is kept as empty tag.
    pub content_before: Vec<TextTrace>,
}
