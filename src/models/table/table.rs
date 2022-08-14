use anyhow::Result;
use hashbrown::HashMap;
use pyo3::{
    prelude::*,
    types::{PyBytes, PyDict},
};
use serde::{Deserialize, Serialize};
use serde_json;

use super::{Cell, Row};
use crate::models::{content_hierarchy::ContentHierarchy, rich_text::RichText};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[pyclass(module = "rsoup.rsoup")]
pub struct Table {
    #[pyo3(get, set)]
    pub id: String,
    #[pyo3(get, set)]
    pub url: String,
    #[pyo3(get, set)]
    pub caption: String,
    #[pyo3(get)]
    pub attrs: HashMap<String, String>,
    #[pyo3(get)]
    pub context: Vec<Py<ContentHierarchy>>,
    #[pyo3(get)]
    pub rows: Vec<Py<Row>>,
}

#[pymethods]
impl Table {
    /// Span the table by copying values to merged field
    pub fn span(&self, py: Python) -> PyResult<Table> {
        if self.rows.len() == 0 {
            return Ok(self.clone());
        }

        let mut pi = 0;
        let mut data = vec![];
        let mut pending_ops = HashMap::<(i32, i32), Cell>::new();

        // >>> begin find the max #cols
        // calculate the number of columns as some people may actually set unrealistic colspan as they are lazy..
        // I try to make its behaviour as much closer to the browser as possible.
        // one thing I notice that to find the correct value of colspan, they takes into account the #cells of rows below the current row
        // so we may have to iterate several times

        let mut cols = vec![0; self.rows.len()];
        for (i, py_row) in self.rows.iter().enumerate() {
            let row = py_row.borrow(py);
            cols[i] += row.cells.len();
            for py_cell in &row.cells {
                let cell = py_cell.borrow(py);
                if cell.rowspan > 1 {
                    for j in 1..cell.rowspan {
                        if i + (j as usize) < cols.len() {
                            cols[i + (j as usize)] += 1;
                        }
                    }
                }
            }
        }

        let max_ncols = *cols.iter().enumerate().max_by_key(|x| x.1).unwrap().1 as i32;

        // sometimes they do show an extra cell for over-colspan row, but it's not consistent or at least not easy for me to find the rule
        // so I decide to not handle that. Hope that we don't have many tables like that.
        // >>> finish find the max #cols

        for py_row in &self.rows {
            let row = py_row.borrow(py);
            let mut new_row = Vec::with_capacity(row.cells.len());
            let mut pj = 0;

            for (cell_index, ocell) in row.cells.iter().enumerate() {
                let mut cell = ocell.borrow(py).clone();
                let original_colspan = cell.colspan;
                let original_rowspan = cell.rowspan;
                cell.colspan = 1;
                cell.rowspan = 1;

                // adding cell from the top
                while pending_ops.contains_key(&(pi, pj)) {
                    new_row.push(Py::new(py, pending_ops.remove(&(pi, pj)).unwrap())?);
                    pj += 1;
                }

                // now add cell and expand the column
                for _ in 0..original_colspan {
                    if pending_ops.contains_key(&(pi, pj)) {
                        // exception, overlapping between colspan and rowspan
                        return Err(crate::error::OverlapSpanPyError::new_err("".to_owned()).into());
                    }
                    new_row.push(Py::new(py, cell.clone())?);
                    for ioffset in 1..original_rowspan {
                        pending_ops.insert((pi + ioffset as i32, pj), cell.clone());
                    }
                    pj += 1;

                    if pj >= max_ncols {
                        // our algorithm cannot handle the case where people are bullying the colspan system, and only can handle the case
                        // where the span that goes beyond the maximum number of columns is in the last column.
                        if cell_index != row.cells.len() - 1 {
                            return Err(crate::error::InvalidCellSpanPyError::new_err(
                                "".to_owned(),
                            )
                            .into());
                        } else {
                            break;
                        }
                    }
                }
            }

            // add more cells from the top since we reach the end
            while pending_ops.contains_key(&(pj, pj)) && pj < max_ncols {
                new_row.push(Py::new(py, pending_ops.remove(&(pj, pj)).unwrap())?);
                pj += 1;
            }

            data.push(Py::new(
                py,
                Row {
                    cells: new_row,
                    attrs: row.attrs.clone(),
                },
            )?);
            pi += 1;
        }

        // len(pending_ops) may > 0, but fortunately, it doesn't matter as the browser also does not render that extra empty lines

        Ok(Table {
            id: self.id.clone(),
            url: self.url.clone(),
            caption: self.caption.clone(),
            attrs: self.attrs.clone(),
            context: self.context.clone(),
            rows: data,
        })
    }

    /// Pad an irregular table (missing cells) to make it become a regular table
    ///
    /// This function only return new table when it's padded, otherwise, None.
    pub fn pad(&self, py: Python) -> PyResult<Option<Table>> {
        if self.rows.len() == 0 {
            return Ok(None);
        }

        let borrowed_rows = self
            .rows
            .iter()
            .map(|py_row| py_row.borrow(py))
            .collect::<Vec<_>>();

        let ncols = borrowed_rows[0].cells.len();
        let is_regular_table = borrowed_rows.iter().all(|row| row.cells.len() == ncols);
        if is_regular_table {
            return Ok(None);
        }

        let max_ncols = borrowed_rows
            .iter()
            .map(|row| row.cells.len())
            .max()
            .unwrap();
        let default_cell = Cell {
            is_header: false,
            rowspan: 1,
            colspan: 1,
            attrs: HashMap::new(),
            value: Py::new(py, RichText::empty())?,
        };

        let mut rows = Vec::with_capacity(self.rows.len());
        for r in borrowed_rows {
            let mut row = r.clone();

            let mut newcell = default_cell.clone();
            // heuristic to match header from the previous cell of the same row
            newcell.is_header = row
                .cells
                .last()
                .map_or(false, |cell| cell.borrow(py).is_header);

            while row.cells.len() < max_ncols {
                row.cells.push(Py::new(py, newcell.clone())?);
            }
            rows.push(Py::new(py, row)?);
        }

        Ok(Some(Table {
            id: self.id.clone(),
            url: self.url.clone(),
            caption: self.caption.clone(),
            attrs: self.attrs.clone(),
            context: self.context.clone(),
            rows: rows,
        }))
    }

    pub fn iter_cells(slf: Py<Table>, py: Python) -> super::cell_iter::CellTIter {
        super::cell_iter::CellTIter {
            table: slf.clone_ref(py),
            row_index: 0,
            cell_index: 0,
        }
    }

    pub fn iter_rows(slf: Py<Table>, py: Python) -> super::row_iter::RowIter {
        super::row_iter::RowIter {
            table: slf.clone_ref(py),
            row_index: 0,
        }
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        let out = postcard::to_allocvec(self)?;
        Ok(out)
    }

    #[staticmethod]
    fn from_bytes(bytes: &PyBytes) -> Result<Table> {
        let table = postcard::from_bytes(bytes.as_bytes())?;
        Ok(table)
    }

    fn to_json(&self) -> Result<String> {
        let out = serde_json::to_string(self)?;
        Ok(out)
    }

    #[staticmethod]
    fn from_json(dat: &str) -> Result<Table> {
        let out = serde_json::from_str(dat)?;
        Ok(out)
    }

    fn to_dict(&self, py: Python) -> PyResult<Py<PyDict>> {
        let o = PyDict::new(py);

        o.set_item("id", &self.id)?;
        o.set_item("url", &self.url)?;
        o.set_item("caption", &self.caption)?;
        o.set_item("attrs", &self.attrs)?;
        o.set_item(
            "context",
            &self
                .context
                .iter()
                .map(|c| c.borrow(py).to_dict(py))
                .collect::<PyResult<Vec<_>>>()?,
        )?;
        o.set_item(
            "rows",
            &self
                .rows
                .iter()
                .map(|r| r.borrow(py).to_dict(py))
                .collect::<PyResult<Vec<_>>>()?,
        )?;

        Ok(o.into_py(py))
    }
}
