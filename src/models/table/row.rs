use hashbrown::HashMap;
use pyo3::{prelude::*, types::PyDict};
use serde::{Deserialize, Serialize};

use super::Cell;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[pyclass(module = "rsoup.rsoup")]
pub struct Row {
    #[pyo3(get)]
    pub cells: Vec<Py<Cell>>,
    #[pyo3(get)]
    pub attrs: HashMap<String, String>,
}

#[pymethods]
impl Row {
    fn iter_cells(slf: Py<Row>, py: Python) -> super::cell_iter::CellRIter {
        super::cell_iter::CellRIter {
            row: slf.clone_ref(py),
            cell_index: 0,
        }
    }

    pub(super) fn to_dict(&self, py: Python) -> PyResult<Py<PyDict>> {
        let o = PyDict::new(py);

        o.set_item("attrs", &self.attrs)?;
        o.set_item(
            "cells",
            &self
                .cells
                .iter()
                .map(|c| c.borrow(py).to_dict(py))
                .collect::<PyResult<Vec<_>>>()?,
        )?;
        Ok(o.into_py(py))
    }
}
