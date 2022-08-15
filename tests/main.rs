use anyhow::Result;
use rsoup::extractors::Document;
use std::{fs, path::Path};

#[cfg(test)]
mod extractors;
#[cfg(test)]
mod models;

pub fn get_doc(filename: &str) -> Result<Document> {
    let html_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/resources")
        .join(filename);
    let url = format!(
        "https://example.org/{}",
        html_file.as_os_str().to_str().unwrap().to_owned()
    );
    let html = fs::read_to_string(html_file)?;

    Ok(Document::new(url, html))
}
