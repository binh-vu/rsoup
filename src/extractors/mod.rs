use scraper::Html;

pub mod context_v1;
pub mod subtree;
pub mod table;

pub struct Document<'s> {
    url: &'s str,
    doc: &'s str,
    html: Html,
}

impl<'s> Document<'s> {
    pub fn new(url: &'s str, doc: &'s str) -> Self {
        Document {
            url,
            doc,
            html: Html::parse_document(doc),
        }
    }
}
