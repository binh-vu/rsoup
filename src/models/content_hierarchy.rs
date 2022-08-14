use crate::models::rich_text::RichText;
use pyo3::{prelude::*, types::PyDict};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Content at each level that leads to the table
#[derive(Clone, Deserialize, Serialize)]
#[pyclass(module = "rsoup.rsoup")]
pub struct ContentHierarchy {
    // level of the heading, level 0 indicate the beginning of the document
    // but should not be used
    #[pyo3(get, set)]
    pub level: usize,
    // title of the level (header)
    #[pyo3(get, set)]
    pub heading: Py<RichText>,
    // content of each level (with the trace), the trace includes information
    // of the containing element
    #[pyo3(get)]
    pub content_before: Vec<Py<RichText>>,
    // only non empty if this is at the same level of the table (lowest level)
    #[pyo3(get)]
    pub content_after: Vec<Py<RichText>>,
}

impl ContentHierarchy {
    pub fn new(level: usize, heading: Py<RichText>) -> Self {
        ContentHierarchy {
            level,
            heading,
            content_before: Vec::new(),
            content_after: Vec::new(),
        }
    }
}

#[pymethods]
impl ContentHierarchy {
    pub fn to_dict(&self, py: Python) -> PyResult<Py<PyDict>> {
        let d = PyDict::new(py);
        d.set_item("level", self.level)?;
        d.set_item("heading", self.heading.borrow(py).to_dict(py)?)?;
        d.set_item(
            "content_before",
            self.content_before
                .iter()
                .map(|t| t.borrow(py).to_dict(py))
                .collect::<PyResult<Vec<_>>>()?,
        )?;
        d.set_item(
            "content_after",
            self.content_after
                .iter()
                .map(|t| t.borrow(py).to_dict(py))
                .collect::<PyResult<Vec<_>>>()?,
        )?;
        Ok(d.into_py(py))
    }
}

impl fmt::Debug for ContentHierarchy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Python::with_gil(|py| {
            f.debug_struct("ContentHierarchy")
                .field("level", &self.level)
                .field("heading", &self.heading.borrow(py))
                .field(
                    "content_before",
                    &self
                        .content_before
                        .iter()
                        .map(|l| l.borrow(py))
                        .collect::<Vec<_>>(),
                )
                .field(
                    "content_after",
                    &self
                        .content_after
                        .iter()
                        .map(|l| l.borrow(py))
                        .collect::<Vec<_>>(),
                )
                .finish()
        })
    }
}
