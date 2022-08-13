use scraper::Html;

pub mod context_v1;
pub mod table;
pub mod url_converter;

use pyo3::prelude::*;

#[pyclass(module = "rsoup.rsoup", unsendable)]
pub struct Document {
    url: String,
    html: Html,
}

#[pymethods]
impl Document {
    #[new]
    pub fn new(url: String, doc: String) -> Self {
        let html = Html::parse_document(&doc);
        Document { url, html }
    }
}