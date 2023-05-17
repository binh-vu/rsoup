use anyhow::Result;
use hashbrown::HashMap;
use pyo3::{
    exceptions::PyKeyError,
    prelude::*,
    types::{PyBytes, PyDict, PyString},
};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fmt;

use super::{Cell, Row};
use crate::models::{content_hierarchy::ContentHierarchy, rich_text::RichText};

#[derive(Clone, Deserialize, Serialize)]
#[pyclass(module = "rsoup.core")]
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
    #[new]
    #[args(
        "*",
        id = "String::new()",
        url = "String::new()",
        caption = "String::new()",
        attrs = "HashMap::new()",
        context = "Vec::new()",
        rows = "Vec::new()"
    )]
    pub fn new(
        id: String,
        url: String,
        caption: String,
        attrs: HashMap<String, String>,
        context: Vec<Py<ContentHierarchy>>,
        rows: Vec<Py<Row>>,
    ) -> Self {
        Self {
            id,
            url,
            caption,
            attrs,
            context,
            rows,
        }
    }

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
        // println!("max_ncols: {}", max_ncols);

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
            while pending_ops.contains_key(&(pi, pj)) && pj < max_ncols {
                // println!(
                //     "\tadding trailing pending ops: {:?}",
                //     pending_ops.get(&(pi, pj)).unwrap()
                // );
                new_row.push(Py::new(py, pending_ops.remove(&(pi, pj)).unwrap())?);
                pj += 1;
            }

            // println!(
            //     ">>> row {}\n\tnew_row: {:?}\n\tpending_ops: {:?}",
            //     pi, new_row, pending_ops
            // );

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

    pub fn n_rows(&self) -> usize {
        self.rows.len()
    }

    pub fn shape(&self, py: Python) -> (usize, usize) {
        if self.rows.len() == 0 {
            (0, 0)
        } else {
            (self.rows.len(), self.rows[0].borrow(py).cells.len())
        }
    }

    pub fn get_cell(&self, py: Python, ri: usize, ci: usize) -> PyResult<Py<Cell>> {
        if ri >= self.rows.len() {
            return Err(PyKeyError::new_err(format!(
                "Key {} is out of rows' range [0, {})",
                ri,
                self.rows.len()
            )));
        }
        let row = self.rows[ri].borrow(py);
        if ci >= row.cells.len() {
            return Err(PyKeyError::new_err(format!(
                "Key {} is out of cells' range [0, {})",
                ci,
                row.cells.len()
            )));
        }

        Ok(row.cells[ci].clone_ref(py))
    }

    pub fn get_row(&self, py: Python, ri: usize) -> PyResult<Py<Row>> {
        if ri >= self.rows.len() {
            return Err(PyKeyError::new_err(format!(
                "Key {} is out of rows' range [0, {})",
                ri,
                self.rows.len()
            )));
        }
        Ok(self.rows[ri].clone_ref(py))
    }

    pub fn iter_cells(slf: Py<Table>, py: Python) -> super::cell_iter::CellTIter {
        super::cell_iter::CellTIter {
            table: slf.clone_ref(py),
            row_index: 0,
            cell_index: 0,
        }
    }

    pub fn enumerate_cells(slf: Py<Table>, py: Python) -> super::cell_iter::CellTEnumerator {
        super::cell_iter::CellTEnumerator {
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

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let out = postcard::to_allocvec(self)?;
        Ok(out)
    }

    #[staticmethod]
    pub fn from_bytes(bytes: &PyBytes) -> Result<Table> {
        let table = postcard::from_bytes(bytes.as_bytes())?;
        Ok(table)
    }

    pub fn to_json(&self) -> Result<String> {
        let out = serde_json::to_string(self)?;
        Ok(out)
    }

    #[staticmethod]
    pub fn from_json(dat: &str) -> Result<Table> {
        let out = serde_json::from_str(dat)?;
        Ok(out)
    }

    pub fn to_base64(&self) -> Result<String> {
        let out = base64::encode(self.to_bytes()?);
        Ok(out)
    }

    #[staticmethod]
    pub fn from_base64(b64s: &PyString) -> Result<Table> {
        let bytes = base64::decode(b64s.to_str()?)?;
        let table = postcard::from_bytes(&bytes)?;
        Ok(table)
    }

    pub fn to_dict(&self, py: Python) -> PyResult<Py<PyDict>> {
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

    pub fn to_list(&self, py: Python) -> PyResult<Vec<Vec<String>>> {
        Ok(self.rows.iter().map(|r| r.borrow(py).to_list(py)).collect())
    }

    fn __getstate__(&self, py: Python) -> PyResult<PyObject> {
        Ok(PyBytes::new(py, &self.to_bytes()?).into())
    }

    fn __setstate__(&mut self, py: Python, state: PyObject) -> PyResult<()> {
        let b = state.as_ref(py).downcast::<PyBytes>()?;
        let slf = Table::from_bytes(b)?;

        self.id = slf.id;
        self.url = slf.url;
        self.caption = slf.caption;
        self.attrs = slf.attrs;
        self.context = slf.context;
        self.rows = slf.rows;

        Ok(())
    }
}

impl fmt::Debug for Table {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Python::with_gil(|py| {
            f.debug_struct("Table")
                .field("id", &self.id)
                .field("url", &self.url)
                .field("caption", &self.caption)
                .field("attrs", &self.attrs)
                .field(
                    "context",
                    &self
                        .context
                        .iter()
                        .map(|x| x.borrow(py))
                        .collect::<Vec<_>>(),
                )
                .field(
                    "rows",
                    &self.rows.iter().map(|x| x.borrow(py)).collect::<Vec<_>>(),
                )
                .finish()
        })
    }
}
