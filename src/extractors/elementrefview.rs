use crate::{
    extractors::text::{get_rich_text, get_text},
    models::rich_text::RichText,
};
use hashbrown::HashSet;
use pyo3::{
    prelude::*,
    types::{PyList, PyString},
};
use scraper::{node::Attrs, CaseSensitivity, ElementRef, Node, Selector};

#[pyclass(module = "rsoup.core", unsendable)]
pub struct ElementRefView(pub ElementRef<'static>);

impl ElementRefView {
    pub fn new(element: ElementRef<'_>) -> Self {
        let element: ElementRef<'static> = unsafe { std::mem::transmute(element) };
        ElementRefView(element)
    }
}

#[pymethods]
impl ElementRefView {
    pub fn html(&self) -> String {
        self.0.html()
    }

    pub fn inner_html(&self) -> String {
        self.0.inner_html()
    }

    pub fn select(&self, query: &str) -> PyResult<Vec<ElementRefView>> {
        let selector = Selector::parse(query).map_err(|_err| {
            pyo3::exceptions::PyValueError::new_err(format!("Invalid css selector: {}", query))
        })?;
        // can't return a wrapper of select because select borrows selector, if we convert its scope to static, the selector
        // will dropped after this function, rendering the select invalid
        Ok(self
            .0
            .select(&selector)
            .map(|el| ElementRefView::new(el))
            .collect::<Vec<_>>())
    }

    pub fn get_text(&self) -> String {
        get_text(&self.0)
    }

    /// Get rich text from this element.
    pub fn get_rich_text(&self, cfg: &RichTextConfig) -> PyResult<RichText> {
        Ok(get_rich_text(
            &self.0,
            &cfg.ignored_tags,
            cfg.only_inline_tags,
            &cfg.discard_tags,
            &cfg.keep_tags,
        ))
    }

    pub fn name(&self) -> &str {
        self.0.value().name()
    }

    pub fn id(&self) -> Option<&str> {
        self.0.value().id()
    }

    pub fn classes(&self) -> Vec<&str> {
        self.0.value().classes().collect()
    }

    pub fn attr(&self, name: &str) -> Option<&str> {
        self.0.value().attr(name)
    }

    pub fn attrs(&self) -> AttrsView {
        AttrsView::new(self.0.value().attrs())
    }

    #[args(case_sensitive = "true")]
    pub fn has_class(&self, cls: &str, case_sensitive: bool) -> bool {
        self.0.value().has_class(
            cls,
            if case_sensitive {
                CaseSensitivity::CaseSensitive
            } else {
                CaseSensitivity::AsciiCaseInsensitive
            },
        )
    }
}

#[pyclass(module = "rsoup.core")]
pub struct RichTextConfig {
    ignored_tags: HashSet<String>,
    only_inline_tags: bool,
    discard_tags: HashSet<String>,
    keep_tags: HashSet<String>,
}

#[pymethods]
impl RichTextConfig {
    #[new]
    pub fn new(
        ignored_tags: &PyList,
        only_inline_tags: bool,
        discard_tags: &PyList,
        keep_tags: &PyList,
    ) -> PyResult<Self> {
        let ignored_tags = ignored_tags
            .into_iter()
            .map(|item| {
                item.downcast::<PyString>()
                    .map(|s| s.to_string())
                    .map_err(|_| {
                        pyo3::exceptions::PyTypeError::new_err(
                            "ignored_tags must be a list of strings",
                        )
                    })
            })
            .collect::<PyResult<HashSet<String>>>()?;

        let discard_tags = discard_tags
            .into_iter()
            .map(|item| {
                item.downcast::<PyString>()
                    .map(|s| s.to_string())
                    .map_err(|_| {
                        pyo3::exceptions::PyTypeError::new_err(
                            "discard_tags must be a list of strings",
                        )
                    })
            })
            .collect::<PyResult<HashSet<String>>>()?;

        let keep_tags = keep_tags
            .into_iter()
            .map(|item| {
                item.downcast::<PyString>()
                    .map(|s| s.to_string())
                    .map_err(|_| {
                        pyo3::exceptions::PyTypeError::new_err(
                            "keep_tags must be a list of strings",
                        )
                    })
            })
            .collect::<PyResult<HashSet<String>>>()?;

        Ok(RichTextConfig {
            ignored_tags,
            only_inline_tags,
            discard_tags,
            keep_tags,
        })
    }
}

#[pyclass(module = "rsoup.core", unsendable)]
pub struct AttrsView(pub Attrs<'static>);

impl AttrsView {
    pub fn new(attrs: Attrs<'_>) -> Self {
        let attrs: Attrs<'static> = unsafe { std::mem::transmute(attrs) };
        AttrsView(attrs)
    }
}

#[pymethods]
impl AttrsView {
    pub fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    pub fn __next__(&mut self) -> Option<(&str, &str)> {
        self.0.next()
    }
}
