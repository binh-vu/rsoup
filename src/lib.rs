#[macro_use]
extern crate lazy_static;

use pyo3::prelude::*;

pub mod context;
pub mod error;
pub mod extractors;
pub mod helper;
pub mod table;
pub mod text;

pub fn rs_square(x: i32) -> i32 {
    x * x
}

/// square of a number
#[pyfunction]
fn square(x: f64) -> f64 {
    x * x
}

#[pymodule]
fn table_extractor(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(square, m)?)?;
    Ok(())
}
