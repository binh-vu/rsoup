use pyo3::prelude::*;

#[pyclass(module = "rsoup.core")]
pub struct RangeIter {
    pub start: usize,
    pub end: usize,
}

#[pymethods]
impl RangeIter {
    fn __iter__(slf: PyRef<Self>) -> PyRef<Self> {
        slf
    }

    fn __next__(&mut self) -> Option<usize> {
        if self.start >= self.end {
            return None;
        }
        self.start += 1;
        Some(self.start - 1)
    }
}
