#[macro_use]
extern crate lazy_static;

use pyo3::prelude::*;

pub mod context;
pub mod error;
pub mod extractors;
pub mod misc;
pub mod table;
pub mod text;

#[pymodule]
fn table_extractor(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<self::table::Table>()?;
    m.add_class::<self::table::Row>()?;
    m.add_class::<self::table::Cell>()?;
    m.add_class::<self::extractors::table::TableExtractor>()?;
    m.add_class::<self::extractors::context_v1::ContextExtractor>()?;
    m.add_class::<self::text::rich_text::RichText>()?;
    m.add_class::<self::text::rich_text::RichTextElement>()?;
    m.add_class::<self::extractors::Document>()?;
    Ok(())
}
