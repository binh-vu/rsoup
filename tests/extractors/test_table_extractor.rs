use crate::get_doc;
use anyhow::Result;
use pyo3::Python;
use rsoup::{
    extractors::{context_v1::ContextExtractor, table::TableExtractor, Document},
    models::table::Table,
};
use scraper::Selector;

fn get_tables(filename: &str, testcase: Option<&str>) -> Result<Vec<Table>> {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let extractor = TableExtractor::new(ContextExtractor::default(), None, None, None, true);
    let mut doc = get_doc(filename)?;

    if testcase.is_some() {
        let selector = Selector::parse(&format!("#{}", testcase.unwrap())).unwrap();
        let el = doc.html.select(&selector).nth(0).unwrap();
        doc = Document::new(doc.url, el.html());
    }

    Ok(extractor.extract_tables(py, &doc, false, false, false)?)
}

#[test]
fn test_extract_empty_table() -> Result<()> {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let tables = get_tables(
        "extractors/table.html",
        Some("infobox-with-nested-opt-empty-tables"),
    )?;
    assert_eq!(tables.len(), 1);
    assert_eq!(
        tables[0].to_list(py)?,
        vec![vec!["← 2012", "October 15, 2016", "2020 →",]]
    );

    let tables = get_tables("wikipedia/2016_Nova_Scotia_municipal_elections.html", None)?;
    assert_eq!(
        tables[1].to_list(py)?[0],
        vec!["Mayoral candidate[1]", "Vote", "%",]
    );

    Ok(())
}
