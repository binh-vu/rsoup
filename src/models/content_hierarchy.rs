use crate::error::into_pyerr;
use crate::models::rich_text::RichText;
use postcard::{from_bytes, to_allocvec};
use pyo3::{prelude::*, types::PyBytes, types::PyDict, types::PyList};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Content at each level that leads to the table
#[derive(Clone, Deserialize, Serialize)]
#[pyclass(module = "rsoup.core")]
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
    #[new]
    pub fn construct(py: Python) -> Self {
        ContentHierarchy::new(0, Py::new(py, RichText::empty()).unwrap())
    }

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

    #[staticmethod]
    pub fn from_dict(py: Python, obj: &PyDict) -> PyResult<Self> {
        let level = obj
            .get_item("level")
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("level"))?
            .extract::<usize>()?;

        let heading = Py::new(
            py,
            RichText::from_dict(
                obj.get_item("heading")
                    .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("heading"))?
                    .downcast::<PyDict>()?,
            )?,
        )?;

        let content_before = obj
            .get_item("content_before")
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("content_before"))?
            .downcast::<PyList>()?
            .iter()
            .map(|o| Py::new(py, RichText::from_dict(o.downcast::<PyDict>()?)?))
            .collect::<PyResult<Vec<_>>>()?;

        let content_after = obj
            .get_item("content_after")
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("content_after"))?
            .downcast::<PyList>()?
            .iter()
            .map(|o| Py::new(py, RichText::from_dict(o.downcast::<PyDict>()?)?))
            .collect::<PyResult<Vec<_>>>()?;

        Ok(ContentHierarchy {
            level,
            heading,
            content_before,
            content_after,
        })
    }

    pub fn __getstate__<'py>(&self, py: Python<'py>) -> PyResult<&'py PyBytes> {
        let out = to_allocvec(&self).map_err(into_pyerr)?;
        Ok(PyBytes::new(py, &out))
    }

    pub fn __setstate__(&mut self, state: &PyBytes) -> PyResult<()> {
        *self = from_bytes::<ContentHierarchy>(state.as_bytes()).map_err(into_pyerr)?;
        Ok(())
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
