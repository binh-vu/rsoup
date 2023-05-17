use pyo3::prelude::*;

use super::{Cell, Row, Table};

#[pyclass(module = "rsoup.core", unsendable)]
pub struct CellRIter {
    pub row: Py<Row>,
    pub cell_index: usize,
}

#[pymethods]
impl CellRIter {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(&mut self, py: Python) -> Option<Py<Cell>> {
        let row = self.row.borrow(py);
        if self.cell_index >= row.cells.len() {
            None
        } else {
            let cell = row.cells[self.cell_index].clone_ref(py);
            self.cell_index += 1;
            Some(cell)
        }
    }
}

#[pyclass(module = "rsoup.core", unsendable)]
pub struct CellTIter {
    pub table: Py<Table>,
    pub row_index: usize,
    pub cell_index: usize,
}

#[pymethods]
impl CellTIter {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(&mut self, py: Python) -> Option<Py<Cell>> {
        let table = self.table.borrow(py);

        if self.row_index >= table.rows.len() {
            return None;
        }

        let mut row = table.rows[self.row_index].borrow(py);

        if self.cell_index < row.cells.len() {
            self.cell_index += 1;
            return Some(row.cells[self.cell_index - 1].clone_ref(py));
        }

        // exhausted row, move to next row
        loop {
            self.row_index += 1;
            if self.row_index >= table.rows.len() {
                return None;
            }
            row = table.rows[self.row_index].borrow(py);
            if row.cells.len() > 0 {
                break;
            }
        }

        self.cell_index = 1;
        Some(row.cells[0].clone_ref(py))
    }
}

#[pyclass(module = "rsoup.core", unsendable)]
pub struct CellTEnumerator {
    pub table: Py<Table>,
    pub row_index: usize,
    pub cell_index: usize,
}

#[pymethods]
impl CellTEnumerator {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(&mut self, py: Python) -> Option<(usize, usize, Py<Cell>)> {
        let table = self.table.borrow(py);

        if self.row_index >= table.rows.len() {
            return None;
        }

        let mut row = table.rows[self.row_index].borrow(py);

        if self.cell_index < row.cells.len() {
            let return_val = (
                self.row_index,
                self.cell_index,
                row.cells[self.cell_index].clone_ref(py),
            );
            self.cell_index += 1;
            return Some(return_val);
        }

        // exhausted row, move to next row
        loop {
            self.row_index += 1;
            if self.row_index >= table.rows.len() {
                return None;
            }
            row = table.rows[self.row_index].borrow(py);
            if row.cells.len() > 0 {
                break;
            }
        }

        self.cell_index = 1;
        Some((self.row_index, 0, row.cells[0].clone_ref(py)))
    }
}
