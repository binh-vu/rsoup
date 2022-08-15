use crate::get_doc;
use anyhow::Result;
use pyo3::Python;
use rsoup::{
    extractors::{context_v1::ContextExtractor, table::TableExtractor},
    models::table::Table,
};

fn get_tables(filename: &str) -> Result<Vec<Table>> {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let extractor = TableExtractor::new(ContextExtractor::default(), None, None, None, true);
    let doc = get_doc(filename)?;

    Ok(extractor.extract_tables(py, &doc, false, false, false)?)
}

#[test]
fn test_span() -> Result<()> {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let tables = get_tables("table_span.html")?;
    let t0 = &tables[0];

    let t0prime = t0.span(py)?;

    assert_eq!(
        t0prime.to_list(py)?,
        vec![
            vec![
                "Mountain name(s)",
                "Height (rounded)",
                "Height (rounded)",
                "Range",
                "Ascents before 2004",
                "Ascents before 2004",
                "Ascents before 2004",
                "Country",
            ],
            vec![
                "Mountain name(s)",
                "Height (rounded)",
                "Height (rounded)",
                "Range",
                "1st",
                "successful",
                "successful",
                "Country",
            ],
            vec![
                "Mountain name(s)",
                "m",
                "ft",
                "Range",
                "1st",
                "y",
                "n",
                "Country",
            ],
            vec![
                "Mount Everest\nSagarmatha\nChomolungma",
                "8,848",
                "29,029",
                "Mahalangur Himalaya",
                "1953",
                "145",
                "121",
                "Nepal\nChina",
            ],
        ]
    );

    Ok(())
}
