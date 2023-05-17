use hashbrown::HashMap;
use pyo3::{exceptions::PyKeyError, prelude::*, types::PyDict};
use serde::{Deserialize, Serialize};
use std::fmt;

use super::Cell;

#[derive(Clone, Deserialize, Serialize)]
#[pyclass(module = "rsoup.core")]
pub struct Row {
    #[pyo3(get)]
    pub cells: Vec<Py<Cell>>,
    #[pyo3(get)]
    pub attrs: HashMap<String, String>,
}

#[pymethods]
impl Row {
    #[new]
    pub fn new(cells: Vec<Py<Cell>>, attrs: HashMap<String, String>) -> Self {
        Row { cells, attrs }
    }

    fn get_cell(&self, py: Python, ci: usize) -> PyResult<Py<Cell>> {
        if ci >= self.cells.len() {
            return Err(PyKeyError::new_err(format!(
                "Key {} is out of cells' range [0, {})",
                ci,
                self.cells.len()
            )));
        }

        Ok(self.cells[ci].clone_ref(py))
    }

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

    pub(super) fn to_list(&self, py: Python) -> Vec<String> {
        self.cells
            .iter()
            .map(|c| c.borrow(py).value.borrow(py).text.clone())
            .collect()
    }
}

impl fmt::Debug for Row {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Python::with_gil(|py| {
            f.debug_struct("Row")
                .field(
                    "cells",
                    &self.cells.iter().map(|l| l.borrow(py)).collect::<Vec<_>>(),
                )
                .field("attrs", &self.attrs)
                .finish()
        })
    }
}
