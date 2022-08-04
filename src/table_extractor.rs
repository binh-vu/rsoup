use scraper::{Html, Selector};

/// Extract tables from HTML.
pub fn extract_tables(
    page_url: &str,
    html: &str,
    auto_span: bool,
    auto_pad: bool,
    extract_context: bool,
) {
    let doc = Html::parse_document(html);
    // let tables = vec![];
    for el in doc.select(Selector::parse("table").unwrap()) {
        extract_table(el);
    }
}

// /// Extract content of a single table
pub fn extract_table(table_el: i32) {}
