use pyo3::prelude::*;

use super::{Row, Table};

#[pyclass(module = "rsoup.core", unsendable)]
pub struct RowIter {
    pub table: Py<Table>,
    pub row_index: usize,
}

#[pymethods]
impl RowIter {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(&mut self, py: Python) -> Option<Py<Row>> {
        let table = self.table.borrow(py);
        if self.row_index >= table.rows.len() {
            None
        } else {
            let row = table.rows[self.row_index].clone_ref(py);
            self.row_index += 1;
            Some(row)
        }
    }
}
