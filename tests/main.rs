use anyhow::Result;
use scraper::Html;
use std::{fs, path::Path};

#[cfg(test)]
mod test_text;

#[cfg(test)]
mod test_extractor;

// pub fn get_doc(filename: &str) -> Result<Html> {
//     let html_file = Path::new(env!("CARGO_MANIFEST_DIR"))
//         .join("tests/resources")
//         .join(filename);
//     let html = fs::read_to_string(html_file)?;
//     Ok(Html::parse_document(&html))
// }
