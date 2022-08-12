use crate::text::RichText;
use pyo3::{prelude::*, types::PyDict};
use serde::{Deserialize, Serialize};

/// Content at each level that leads to the table
#[derive(Debug, Clone, Deserialize, Serialize)]
#[pyclass(module = "rsoup.rsoup")]
pub struct ContentHierarchy {
    // level of the heading, level 0 indicate the beginning of the document
    // but should not be used
    #[pyo3(get, set)]
    pub level: usize,
    // title of the level (header)
    #[pyo3(get, set)]
    pub heading: RichText,
    // content of each level (with the trace), the trace includes information
    // of the containing element
    #[pyo3(get, set)]
    pub content_before: Vec<RichText>,
    // only non empty if this is at the same level of the table (lowest level)
    #[pyo3(get, set)]
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

#[pymethods]
impl ContentHierarchy {
    pub fn to_dict(&self, py: Python) -> PyResult<Py<PyDict>> {
        let d = PyDict::new(py);
        d.set_item("level", self.level)?;
        d.set_item("heading", self.heading.to_dict(py)?)?;
        d.set_item(
            "content_before",
            self.content_before
                .iter()
                .map(|t| t.to_dict(py))
                .collect::<PyResult<Vec<_>>>()?,
        )?;
        d.set_item(
            "content_after",
            self.content_after
                .iter()
                .map(|t| t.to_dict(py))
                .collect::<PyResult<Vec<_>>>()?,
        )?;
        Ok(d.into_py(py))
    }
}
