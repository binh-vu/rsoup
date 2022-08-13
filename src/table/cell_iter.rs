use pyo3::prelude::*;

use super::{Cell, Table};

#[pyclass(module = "rsoup.rsoup", unsendable)]
pub struct CellIterator {
    table: PyCell<Table>,
}

// #[pymethods]
// impl CellIterator {
//     fn next(slf: PyRef<Self>) -> PyResult<Option<&Cell>> {
//         unimplemented!()
//     }
// }
