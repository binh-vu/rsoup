use crate::error::TableExtractorError;
use crate::misc::convert_attrs;
use crate::table::{Row, Table};
use crate::{
    table::Cell,
    text::{get_text, get_text_with_trace},
};
use anyhow::{bail, Result};
use ego_tree::NodeRef;
use hashbrown::HashSet;
use scraper::{ElementRef, Node, Selector};
use url::Url;

use super::context_v1::ContextExtractor;
use super::Document;

pub struct TableExtractor {
    ignored_tags: HashSet<String>,
    discard_tags: HashSet<String>,
    only_keep_inline_tags: bool,
    context_extractor: ContextExtractor,
}

impl TableExtractor {
    pub fn default(context_extractor: ContextExtractor) -> Self {
        let discard_tags = HashSet::from_iter(
            ["script", "style", "noscript"]
                .into_iter()
                .map(str::to_owned),
        );
        let ignored_tags = HashSet::from_iter(["div"].into_iter().map(str::to_owned));

        TableExtractor {
            ignored_tags,
            discard_tags,
            only_keep_inline_tags: false,
            context_extractor,
        }
    }

    /// Extract tables from HTML.
    pub fn extract_tables<'t>(
        &self,
        doc: &'t mut Document,
        auto_span: bool,
        auto_pad: bool,
        extract_context: bool,
    ) -> Result<Vec<Table>> {
        let tree = &doc.html;

        let selector = Selector::parse("table").unwrap();
        let mut tables = vec![];
        let mut table_els = vec![];

        for el in tree.select(&selector) {
            if el.select(&selector).next().is_some() {
                continue;
            }
            tables.push(self.extract_non_nested_table(el)?);
            table_els.push(el);
        }

        if auto_span {
            let mut new_tables = Vec::with_capacity(tables.len());
            let mut new_table_els = Vec::with_capacity(tables.len());

            for (i, tbl) in tables.iter().enumerate() {
                match tbl.span() {
                    Ok(new_tbl) => {
                        new_tables.push(new_tbl);
                        new_table_els.push(table_els[i]);
                    }
                    Err(TableExtractorError::OverlapSpanError(_)) => {}
                    Err(TableExtractorError::InvalidCellSpanError(_)) => {}
                    Err(x) => bail!(x),
                }
            }
            tables = new_tables;
            table_els = new_table_els;
        }

        if auto_pad {
            tables = tables
                .into_iter()
                .map(|tbl| tbl.pad().unwrap_or(tbl))
                .collect::<Vec<_>>()
        }

        if extract_context {
            for i in 0..tables.len() {
                tables[i].context = self.context_extractor.extractor_context(*table_els[i])?;
            }
        }

        let mut url = Url::parse(doc.url)?;
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
            query.extend_from_slice(i.to_string().as_bytes());
            url.set_query(Some(std::str::from_utf8(&query)?));
            tbl.id = url.as_str().to_owned();
            query.truncate(query_len);
            tbl.url = doc.url.to_owned();
        }

        Ok(tables)
    }

    /// Extract content of a single table
    ///
    /// # Arguments
    ///
    /// * `table_el` - The table element
    pub fn extract_non_nested_table(&self, table_el: ElementRef) -> Result<Table> {
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
                            cells.push(self.extract_cell(cell_ref)?);
                        }
                    }

                    rows.push(Row {
                        cells,
                        attrs: convert_attrs(&row_el.attrs),
                    });
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
    fn extract_cell(&self, cell: NodeRef<Node>) -> Result<Cell> {
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
                .map_err(|_| TableExtractorError::InvalidCellSpanError(raw_colspan.to_owned()))?
        };
        let rowspan = if raw_rowspan == "" {
            1
        } else {
            raw_rowspan
                .parse::<u16>()
                .map_err(|_| TableExtractorError::InvalidCellSpanError(raw_rowspan.to_owned()))?
        };

        Ok(Cell {
            is_header,
            html: ElementRef::wrap(cell).unwrap().html(),
            rowspan,
            colspan,
            value: get_text_with_trace(
                &cell,
                &self.ignored_tags,
                self.only_keep_inline_tags,
                &self.discard_tags,
            ),
            attrs: convert_attrs(&el.attrs),
        })
    }
}
