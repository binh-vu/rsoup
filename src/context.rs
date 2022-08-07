use crate::text::RichText;

/// Content at each level that leads to the table
#[derive(Debug, Clone)]
pub struct ContentHierarchy {
    // level of the heading, level 0 indicate the beginning of the document
    // but should not be used
    pub level: usize,
    // title of the level (header)
    pub heading: RichText,
    // content of each level (with the trace), the trace includes information
    // of the containing element
    pub content_before: Vec<RichText>,
    // only non empty if this is at the same level of the table (lowest level)
    pub content_after: Vec<RichText>,
}

impl ContentHierarchy {
    pub fn new(level: usize, heading: RichText) -> Self {
        ContentHierarchy {
            level,
            heading,
            content_before: Vec::new(),
            content_after: Vec::new(),
        }
    }
}
