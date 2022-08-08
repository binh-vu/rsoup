use anyhow::Result;
use hashbrown::{HashMap, HashSet};
use scraper::{Html, Node, Selector};
use std::{fs, path::Path};
use table_extractor::extractors::context_v1::ContextExtractor;
use table_extractor::{
    misc::SimpleTree,
    text::{get_rich_text, get_text, RichText, RichTextElement},
};

fn get_doc(filename: &str) -> Result<Html> {
    let html_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/resources")
        .join(filename);
    let html = fs::read_to_string(html_file)?;
    Ok(Html::parse_document(&html))
}

// #[test]
// fn test_locate_content_before_and_after() -> Result<()> {
//     let extractor = ContextExtractor::default();

//     let doc = get_doc("context/one-level.html")?;
//     let selector = Selector::parse("#marker").unwrap();

//     let elements = doc.select(&selector).collect::<Vec<_>>();
//     assert_eq!(elements.len(), 1);

//     let (tree_before, tree_after) = extractor.locate_content_before_and_after(*elements[0])?;

//     let node_key = |uid| match tree_before.get_node(uid).value() {
//         Node::Element(x) => format!("{}", x.name()),
//         Node::Text(x) => format!("`{}`", &x[..x.len().min(4)].replace("\n", "\\n")),
//         _ => format!("{}", uid),
//     };

//     assert!(tree_before.validate());
//     assert!(tree_after.validate());
//     assert_eq!(tree_before.to_string(&node_key), "body -> {\n    `\\n   `\n    p\n    `\\n   `\n    h1\n    `\\n   `\n    div\n    `\\n   `\n    h2\n    `\\n   `\n    div -> {\n        `\\n   `\n        span\n        ` `\n        b\n        `\\n   `\n    }\n}\n");

//     Ok(())
// }

#[test]
fn test_context_extractor() -> Result<()> {
    let extractor = ContextExtractor::default();

    let doc = get_doc("context/three-level.html")?;
    let selector = Selector::parse("#marker").unwrap();

    let elements = doc.select(&selector).collect::<Vec<_>>();
    assert_eq!(elements.len(), 1);

    let context = extractor.extractor_context(*elements[0])?;
    println!("{:#?}", context);

    Ok(())
}
