use postcard;
use pyo3::PyErr;
use thiserror::Error;

pyo3::create_exception!(rsoup, OverlapSpanPyError, pyo3::exceptions::PyException);
pyo3::create_exception!(rsoup, InvalidCellSpanPyError, pyo3::exceptions::PyException);

/// Represent possible errors returned by this library.
#[derive(Error, Debug)]
pub enum RSoupError {
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

    #[error(transparent)]
    PostcardError(#[from] postcard::Error),
}

pub fn into_pyerr<E: Into<RSoupError>>(err: E) -> PyErr {
    let hderr = err.into();
    if let RSoupError::PyErr(e) = hderr {
        e
    } else {
        let anyerror: anyhow::Error = hderr.into();
        anyerror.into()
    }
}
