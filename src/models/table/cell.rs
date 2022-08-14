use hashbrown::HashMap;
use pyo3::{prelude::*, types::PyDict};
use serde::{Deserialize, Serialize};

use crate::models::rich_text::RichText;

#[pyclass(module = "rsoup.rsoup")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Cell {
    #[pyo3(get, set)]
    pub is_header: bool,
    #[pyo3(get, set)]
    pub rowspan: u16,
    #[pyo3(get, set)]
    pub colspan: u16,
    #[pyo3(get)]
    pub attrs: HashMap<String, String>,
    // include the outer tags of the cell
    #[pyo3(get, set)]
    pub value: Py<RichText>,
}

#[pymethods]
impl Cell {
    fn __str__(&self, py: Python) -> String {
        self.value.borrow(py).to_html(true, false)
    }

    pub(super) fn to_dict(&self, py: Python) -> PyResult<Py<PyDict>> {
        let o = PyDict::new(py);

        o.set_item("is_header", &self.is_header)?;
        o.set_item("rowspan", self.rowspan)?;
        o.set_item("colspan", self.colspan)?;
        o.set_item("attrs", &self.attrs)?;
        o.set_item("value", self.value.borrow(py).to_dict(py)?)?;
        Ok(o.into_py(py))
    }
}
