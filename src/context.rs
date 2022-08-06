use crate::text::TextTrace;

/// Content at each level that leads to the table
pub struct ContentHierarchy {
    // level of the heading, level 0 indicate the beginning of the document
    // but should not be used
    pub level: usize,
    // title of the level (header)
    pub heading: TextTrace,
    // content of each level (with the trace), the trace includes information
    // of the containing element
    pub content_before: Vec<TextTrace>,
    // only non empty if this is at the same level of the table (lowest level)
    pub content_after: Vec<TextTrace>,
}

impl ContentHierarchy {
    pub fn new(level: usize, heading: TextTrace) -> Self {
        ContentHierarchy {
            level,
            heading,
            content_before: Vec::new(),
            content_after: Vec::new(),
        }
    }
}
