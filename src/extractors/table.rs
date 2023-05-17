use crate::error::{InvalidCellSpanPyError, OverlapSpanPyError, RSoupError};
use crate::extractors::context_v1::ContextExtractor;
use crate::extractors::text::{get_rich_text, get_text};
use crate::extractors::Document;
use crate::misc::convert_attrs;
use crate::misc::url_converter::URLConverter;
use crate::models::table::{Cell, Row, Table};
use anyhow::{bail, Result};
use ego_tree::NodeRef;
use hashbrown::HashSet;
use pyo3::prelude::*;
use scraper::{ElementRef, Node, Selector};
use url::Url;

#[pyclass(module = "rsoup.core")]
pub struct TableExtractor {
    ignored_tags: HashSet<String>,
    discard_tags: HashSet<String>,
    keep_tags: HashSet<String>,
    only_keep_inline_tags: bool,
    context_extractor: ContextExtractor,
}

#[pymethods]
impl TableExtractor {
    #[new]
    #[args(
        "*",
        ignored_tags = "None",
        discard_tags = "None",
        keep_tags = "None",
        only_keep_inline_tags = "true"
    )]
    pub fn new(
        context_extractor: ContextExtractor,
        ignored_tags: Option<Vec<&str>>,
        discard_tags: Option<Vec<&str>>,
        keep_tags: Option<Vec<&str>>,
        only_keep_inline_tags: bool,
    ) -> Self {
        let discard_tags_ = HashSet::from_iter(
            discard_tags
                .unwrap_or(["script", "style", "noscript", "table"].to_vec())
                .into_iter()
                .map(str::to_owned),
        );
        let ignored_tags_ = HashSet::from_iter(
            ignored_tags
                .unwrap_or(["div"].to_vec())
                .into_iter()
                .map(str::to_owned),
        );
        let keep_tags_ = HashSet::from_iter(
            keep_tags
                .unwrap_or(["ol", "ul", "li"].to_vec())
                .into_iter()
                .map(str::to_owned),
        );

        TableExtractor {
            ignored_tags: ignored_tags_,
            discard_tags: discard_tags_,
            keep_tags: keep_tags_,
            only_keep_inline_tags,
            context_extractor,
        }
    }

    #[args(auto_span = "true", auto_pad = "true", extract_context = "true")]
    fn extract(
        &self,
        py: Python,
        url: String,
        doc: String,
        auto_span: bool,
        auto_pad: bool,
        extract_context: bool,
    ) -> PyResult<Vec<Table>> {
        Ok(self.extract_tables(
            py,
            &Document::new(url, doc),
            auto_span,
            auto_pad,
            extract_context,
        )?)
    }
}

impl TableExtractor {
    /// Extract tables from HTML.
    pub fn extract_tables<'t>(
        &self,
        py: Python,
        doc: &'t Document,
        auto_span: bool,
        auto_pad: bool,
        extract_context: bool,
    ) -> Result<Vec<Table>> {
        let tree = &doc.html;

        let selector = Selector::parse("table").unwrap();
        let mut tables = vec![];
        let mut table_els = vec![];
        let mut table_nos = vec![];

        for el in tree.select(&selector) {
            if el.select(&selector).next().is_some() {
                continue;
            }
            let table = self.extract_non_nested_table(py, el)?;
            // skip if no rows or columns
            if table.rows.len() == 0 || table.rows.iter().all(|r| r.borrow(py).cells.len() == 0) {
                continue;
            }
            tables.push(table);
            table_els.push(el);
            table_nos.push(table_nos.len());
        }

        if auto_span {
            let mut new_tables = Vec::with_capacity(tables.len());
            let mut new_table_els = Vec::with_capacity(tables.len());
            let mut new_table_nos = Vec::with_capacity(tables.len());

            for (i, tbl) in tables.iter().enumerate() {
                match tbl.span(py) {
                    Ok(new_tbl) => {
                        new_tables.push(new_tbl);
                        new_table_els.push(table_els[i]);
                        new_table_nos.push(i);
                    }
                    Err(err) => {
                        if !err.is_instance_of::<OverlapSpanPyError>(py)
                            && !err.is_instance_of::<InvalidCellSpanPyError>(py)
                        {
                            bail!(err);
                        }
                    }
                }
            }
            tables = new_tables;
            table_els = new_table_els;
            table_nos = new_table_nos;
        }

        if auto_pad {
            tables = tables
                .into_iter()
                .map(|tbl| Ok(tbl.pad(py)?.unwrap_or(tbl)))
                .collect::<PyResult<Vec<_>>>()?
        }

        if extract_context {
            for i in 0..tables.len() {
                tables[i].context = self
                    .context_extractor
                    .extract_context(py, *table_els[i])?
                    .into_iter()
                    .map(|x| Py::new(py, x))
                    .collect::<PyResult<Vec<_>>>()?;
            }
        }

        // update table id
        let mut url = Url::parse(&doc.url)?;
        let mut query = match url.query() {
            None => "table_no=".as_bytes().to_vec(),
            Some(q) => {
                let mut v = q.as_bytes().to_vec();
                v.extend_from_slice("&table_no=".as_bytes());
                v
            }
        };
        let query_len = query.len();

        for (i, tbl) in tables.iter_mut().enumerate() {
            query.extend_from_slice(table_nos[i].to_string().as_bytes());
            url.set_query(Some(std::str::from_utf8(&query)?));
            tbl.id = url.as_str().to_owned();
            query.truncate(query_len);
            tbl.url = doc.url.to_owned();
        }

        // convert relative urls to absolute urls
        let url_converter = URLConverter::new(doc.url.to_owned())?;
        for table in &mut tables {
            for row in &mut table.rows {
                for cell in &mut (row.borrow_mut(py)).cells {
                    url_converter
                        .normalize_rich_text(&mut *cell.borrow_mut(py).value.borrow_mut(py));
                }
            }

            for content in &mut table.context {
                for line in &mut content.borrow_mut(py).content_before {
                    url_converter.normalize_rich_text(&mut *line.borrow_mut(py));
                }
                for line in &mut content.borrow_mut(py).content_after {
                    url_converter.normalize_rich_text(&mut *line.borrow_mut(py));
                }
            }
        }

        Ok(tables)
    }

    /// Extract content of a single table
    ///
    /// # Arguments
    ///
    /// * `table_el` - The table element
    pub fn extract_non_nested_table(&self, py: Python, table_el: ElementRef) -> Result<Table> {
        let mut caption: String = "".to_owned();
        let mut rows = vec![];

        for child_ref in table_el.children() {
            let child = child_ref.value();
            if !child.is_element() {
                continue;
            }

            let cel = child.as_element().unwrap();
            if cel.name() == "caption" {
                caption = get_text(&child_ref);
                continue;
            }

            if cel.name() != "thead" && cel.name() != "tbody" {
                debug_assert!(cel.name() == "style");
                continue;
            }

            for row_ref in child_ref.children() {
                if let Node::Element(row_el) = row_ref.value() {
                    if row_el.name() != "tr" {
                        debug_assert!(row_el.name() == "style");
                        continue;
                    }

                    let mut cells = vec![];
                    for cell_ref in row_ref.children() {
                        if let Node::Element(cell_el) = cell_ref.value() {
                            if cell_el.name() != "td" && cell_el.name() != "th" {
                                debug_assert!(cell_el.name() == "style");
                                continue;
                            }
                            cells.push(Py::new(py, self.extract_cell(py, cell_ref)?)?);
                        }
                    }

                    rows.push(Py::new(
                        py,
                        Row {
                            cells,
                            attrs: convert_attrs(&row_el.attrs),
                        },
                    )?);
                }
            }
        }

        Ok(Table {
            id: String::new(),
            url: String::new(),
            caption,
            attrs: convert_attrs(&table_el.value().attrs),
            context: Vec::new(),
            rows,
        })
    }

    /// Extract cell from td/th tag. This function does not expect a nested table in the cell
    ///
    /// # Arguments
    ///
    /// * `cell` - td/th tag
    fn extract_cell(&self, py: Python, cell: NodeRef<Node>) -> Result<Cell> {
        let el = cell.value().as_element().expect("Expected element");
        debug_assert!(el.name() == "td" || el.name() == "th");

        let is_header = el.name() == "th";
        let raw_colspan = el.attr("colspan").unwrap_or("1").trim();
        let raw_rowspan = el.attr("rowspan").unwrap_or("1").trim();

        let colspan = if raw_colspan == "" {
            1
        } else {
            // convert
            raw_colspan
                .parse::<u16>()
                .map_err(|_| RSoupError::InvalidCellSpanError(raw_colspan.to_owned()))?
        };
        let rowspan = if raw_rowspan == "" {
            1
        } else {
            raw_rowspan
                .parse::<u16>()
                .map_err(|_| RSoupError::InvalidCellSpanError(raw_rowspan.to_owned()))?
        };

        Ok(Cell {
            is_header,
            rowspan,
            colspan,
            value: Py::new(
                py,
                get_rich_text(
                    &cell,
                    &self.ignored_tags,
                    self.only_keep_inline_tags,
                    &self.discard_tags,
                    &self.keep_tags,
                ),
            )?,
            attrs: convert_attrs(&el.attrs),
        })
    }
}
