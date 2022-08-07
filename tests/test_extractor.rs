use anyhow::Result;
use hashbrown::{HashMap, HashSet};
use scraper::{Html, Selector};
use std::{fs, path::Path};
use table_extractor::extractors::context_v1::ContextExtractor;
use table_extractor::{
    misc::SimpleTree,
    text::{get_rich_text, get_text, RichText, RichTextElement},
};

fn get_doc() -> Result<Html> {
    let html_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/resources/wikipedia/List_of_highest_mountains_on_Earth.html");
    let html = fs::read_to_string(html_file)?;
    Ok(Html::parse_document(&html))
}

#[test]
fn test_context_extractor() -> Result<()> {
    let extractor = ContextExtractor::default();

    let doc = get_doc()?;
    let selector = Selector::parse("table.wikitable").unwrap();

    let elements = doc.select(&selector).collect::<Vec<_>>();
    assert_eq!(elements.len(), 1);

    let context = extractor.extractor_context(*elements[0])?;
    println!("{:#?}", context[0].content_before);

    Ok(())
}
