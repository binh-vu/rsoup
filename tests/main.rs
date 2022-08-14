#[cfg(test)]
mod test_text_extractor;

#[cfg(test)]
mod test_context_extractor;

// pub fn get_doc(filename: &str) -> Result<Html> {
//     let html_file = Path::new(env!("CARGO_MANIFEST_DIR"))
//         .join("tests/resources")
//         .join(filename);
//     let html = fs::read_to_string(html_file)?;
//     Ok(Html::parse_document(&html))
// }
