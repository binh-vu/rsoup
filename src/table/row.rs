use hashbrown::HashMap;
use pyo3::{prelude::*, types::PyDict};
use serde::{Deserialize, Serialize};

use super::Cell;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[pyclass(module = "rsoup.rsoup")]
pub struct Row {
    #[pyo3(get)]
    pub cells: Vec<Cell>,
    #[pyo3(get)]
    pub attrs: HashMap<String, String>,
}

#[pymethods]
impl Row {
    pub(super) fn to_dict(&self, py: Python) -> PyResult<Py<PyDict>> {
        let o = PyDict::new(py);

        o.set_item("attrs", &self.attrs)?;
        o.set_item(
            "cells",
            &self
                .cells
                .iter()
                .map(|c| c.to_dict(py))
                .collect::<PyResult<Vec<_>>>()?,
        )?;
        Ok(o.into_py(py))
    }
}
