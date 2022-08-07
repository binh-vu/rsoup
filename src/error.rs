use pyo3::PyErr;
use thiserror::Error;

/// Represent possible errors returned by this library.
#[derive(Error, Debug)]
pub enum TableExtractorError {
    #[error("InvalidHTMLStructureError: {0}")]
    InvalidHTMLStructureError(&'static str),

    #[error("OverlapSpanError: {0}")]
    OverlapSpanError(String),

    #[error("InvalidCellSpanError: {0}")]
    InvalidCellSpanError(String),

    /// Represents all other cases of `std::io::Error`.
    #[error(transparent)]
    IOError(#[from] std::io::Error),

    /// PyO3 error
    #[error(transparent)]
    PyErr(#[from] pyo3::PyErr),
}

pub fn into_pyerr<E: Into<TableExtractorError>>(err: E) -> PyErr {
    let hderr = err.into();
    if let TableExtractorError::PyErr(e) = hderr {
        e
    } else {
        let anyerror: anyhow::Error = hderr.into();
        anyerror.into()
    }
}
