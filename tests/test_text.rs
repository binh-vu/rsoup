use anyhow::Result;
use hashbrown::{HashMap, HashSet};
use scraper::{Html, Selector};
use std::{fs, path::Path};
use table_extractor::{
    misc::SimpleTree,
    text::{get_rich_text, get_text, RichText, RichTextElement},
};

pub fn get_doc(filename: &str) -> Result<Html> {
    let html_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/resources")
        .join(filename);
    let html = fs::read_to_string(html_file)?;
    Ok(Html::parse_document(&html))
}

#[test]
fn test_get_text() -> Result<()> {
    let doc = get_doc("parser.html")?;
    let selector = Selector::parse(r".test\:get-text").expect("selector is invalid");
    let els = doc.select(&selector).collect::<Vec<_>>();

    assert_eq!(els.len(), 4);
    assert_eq!(get_text(&els[0]), "What are youdoing ?");
    assert_eq!(get_text(&els[1]), "Date: Today\nTime: now\nHello world !\nWhat are youdoing ?\n...\nI'm sleeping\nThis is where the conversationend. or not?");
    assert_eq!(
        get_text(&els[3]),
        "abc def\nContent of section 1\nSection 1.1\nContent of section 1.1\nhello World ."
    );
    Ok(())
}

#[test]
fn test_get_rich_text() -> Result<()> {
    let ignored_tags = HashSet::new();
    let discard_tags = HashSet::new();
    let keep_tags = HashSet::from_iter(
        vec!["h1", "h2", "h3", "h4", "h5", "h6"]
            .into_iter()
            .map(str::to_owned),
    );

    // let doc = Html::parse_fragment("<p>What are you<b>doing </b>?</p>");
    // let mut element = SimpleTree::new(RichTextElement {
    //     tag: "p".to_owned(),
    //     start: 0,
    //     end: 19,
    //     attrs: HashMap::new(),
    // });
    // element.add_node(RichTextElement {
    //     tag: "b".to_owned(),
    //     start: 12,
    //     end: 17,
    //     attrs: HashMap::new(),
    // });
    // element.add_child(0, 1);
    // assert_eq!(
    //     get_rich_text(
    //         &doc.tree
    //             .root()
    //             .first_child()
    //             .unwrap()
    //             .first_child()
    //             .unwrap(),
    //         &ignored_tags,
    //         false,
    //         &discard_tags,
    //         &keep_tags
    //     ),
    //     RichText {
    //         text: "What are youdoing ?".to_owned(),
    //         element
    //     }
    // );

    // let docs = [
    //     "<p>What are you<b>doing </b>?</p>",
    //     "<i></i>",
    //     "  <i>   </i>",
    //     "<a>  Link    to<b> something</b><i></i></a>",
    //     "<a>  Link    to<b> something</b><i></i> <span><b></b></span></a>",
    //     "<span>hello</span> <a>World</a> .",
    // ];
    // let parsed_texts = [
    //     "What are you<b>doing</b> ?",
    //     "<i></i>",
    //     "<i></i>",
    //     "<a>Link to <b>something</b><i></i></a>",
    //     "<a>Link to <b>something</b><i></i><span><b></b></span></a>",
    //     "<span>hello</span> <a>World</a> .",
    // ];

    // for (i, doc) in docs.iter().enumerate() {
    //     let tree = Html::parse_fragment(doc).tree;
    //     let node = tree.root().first_child().unwrap();

    //     // println!("{:#?}", node);
    //     assert_eq!(
    //         get_rich_text(&node, &ignored_tags, true, &discard_tags, &keep_tags)
    //             .to_html(false, false),
    //         parsed_texts[i]
    //     );
    // }

    let doc = get_doc("parser.html")?;
    let selector = Selector::parse(r".test\:get-text").expect("selector is invalid");
    let els = doc.select(&selector).collect::<Vec<_>>();

    let text = get_rich_text(&els[3], &ignored_tags, true, &discard_tags, &keep_tags);
    assert_eq!(
        text.text,
        "abc def\nContent of section 1\nSection 1.1\nContent of section 1.1\nhello World ."
    );

    Ok(())
}
