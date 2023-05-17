use scraper::Html;

pub mod context_v1;
pub mod table;
pub mod text;

use pyo3::prelude::*;

#[pyclass(module = "rsoup.core", unsendable)]
pub struct Document {
    pub url: String,
    pub html: Html,
}

#[pyclass(module = "rsoup.core", unsendable)]
pub struct ElementRef {}

#[pymethods]
impl Document {
    #[new]
    pub fn new(url: String, doc: String) -> Self {
        let html = Html::parse_document(&doc);
        Document { url, html }
    }

    // pub fn select(&self, query: &str) -> PyResult<Vec<ElementRef>> {
    //     let selector = Selector::parse(query)?;
    //     unimplemented!()
    // }
}
