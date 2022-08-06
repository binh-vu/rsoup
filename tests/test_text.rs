use anyhow::Result;
use hashbrown::{HashMap, HashSet};
use scraper::{ElementRef, Html, Selector};
use std::{ffi::OsStr, fs, path::Path};
use table_extractor::text::{get_text, get_text_with_trace, TextHTMLElement, TextTrace};

fn get_doc() -> Result<Html> {
    let html_file = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/resources/parser.html");
    let html = fs::read_to_string(html_file)?;
    Ok(Html::parse_document(&html))
}

#[test]
fn test_get_text() -> Result<()> {
    let doc = get_doc()?;
    let selector = Selector::parse(r".test\:get-text").expect("selector is invalid");
    let els = doc.select(&selector).collect::<Vec<_>>();

    assert_eq!(get_text(&els[0]), "What are youdoing ?");
    assert_eq!(get_text(&els[1]), "Date: Today\nTime: now\nHello world !\nWhat are youdoing ?\n...\nI'm sleeping\nThis is where the conversationend. or not?");
    Ok(())
}

#[test]
fn test_get_text_with_trace() -> Result<()> {
    let ignored_tags = HashSet::new();
    let only_inline_tags = false;

    let doc = Html::parse_fragment("<p>What are you<b>doing </b>?</p>");
    assert_eq!(
        get_text_with_trace(
            &doc.tree.root().first_child().unwrap(),
            &ignored_tags,
            only_inline_tags
        ),
        TextTrace {
            text: "What are youdoing ?".to_owned(),
            trace: vec![TextHTMLElement {
                tag: "p".to_owned(),
                start: 0,
                end: 19,
                attrs: HashMap::new(),
                children: vec![TextHTMLElement {
                    tag: "b".to_owned(),
                    start: 12,
                    end: 17,
                    attrs: HashMap::new(),
                    children: vec![]
                }]
            }]
        }
    );

    let docs = [
        "<p>What are you<b>doing </b>?</p>",
        "<i></i>",
        "  <i>   </i>",
        "<a>  Link    to<b> something</b><i></i></a>",
        "<a>  Link    to<b> something</b><i></i> <span><b></b></span></a>",
    ];
    let parsed_texts = [
        "What are you<b>doing</b> ?",
        "<i></i>",
        "<i></i>",
        "<a>Link to <b>something</b><i></i></a>",
        "<a>Link to <b>something</b><i></i><span><b></b></span></a>",
    ];

    for (i, doc) in docs.iter().enumerate() {
        let tree = Html::parse_fragment(doc).tree;
        let node = tree.root().first_child().unwrap();
        // println!("{:#?}", get_text_with_trace(&node, &ignored_tags, true));
        assert_eq!(
            get_text_with_trace(&node, &ignored_tags, true).to_bare_html(),
            parsed_texts[i]
        );
    }

    Ok(())
}
