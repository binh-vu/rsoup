use scraper::Html;

pub mod context_v1;
pub mod elementrefview;
pub mod table;
pub mod text;

use pyo3::prelude::*;
use scraper::Selector;

use self::elementrefview::ElementRefView;

#[pyclass(module = "rsoup.core", unsendable)]
pub struct Document {
    pub url: String,
    pub html: Html,
}

#[pymethods]
impl Document {
    #[new]
    pub fn new(url: String, doc: String) -> Self {
        let html = Html::parse_document(&doc);
        Document { url, html }
    }

    pub fn select(&self, query: &str) -> PyResult<Vec<ElementRefView>> {
        let selector = Selector::parse(query).map_err(|_err| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid css selector: {}", query))
        })?;
        // can't return a wrapper of select because select borrows selector, if we convert its scope to static, the selector
        // will dropped after this function, rendering the select invalid
        Ok(self
            .html
            .select(&selector)
            .map(|el| ElementRefView::new(el))
            .collect::<Vec<_>>())
    }
}
