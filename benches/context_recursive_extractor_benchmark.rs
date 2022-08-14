use anyhow::Result;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use ego_tree::NodeRef;
use rsoup::extractors::context_v1::ContextExtractor;
use rsoup::models::rich_text::RichText;
use scraper::{Html, Node, Selector};
use std::{fs, path::Path};

fn get_doc(filename: &str) -> Html {
    let html_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/resources")
        .join("wikipedia/List_of_highest_mountains_on_Earth.html");
    let html = fs::read_to_string(html_file).unwrap();
    Html::parse_document(&html)
}

// fn extract_context(extractor: &ContextExtractor, filename: &str, index: usize) -> usize {
fn extract_context(extractor: &ContextExtractor, table_el: NodeRef<Node>) -> usize {
    // let doc = get_doc(filename);
    // let selector = Selector::parse("table").unwrap();
    // let table_el = *doc.select(&selector).nth(index).unwrap();

    let (tree_before, tree_after) = extractor.locate_content_before_and_after(table_el).unwrap();

    let mut context_before: Vec<RichText> = vec![];
    let mut context_after: Vec<RichText> = vec![];

    extractor.flatten_tree(&tree_before, &mut context_before);
    extractor.flatten_tree(&tree_after, &mut context_after);

    context_before.len() + context_after.len()
}

// fn extract_context_recur(extractor: &ContextExtractor, filename: &str, index: usize) -> usize {
fn extract_context_recur(extractor: &ContextExtractor, table_el: NodeRef<Node>) -> usize {
    // let doc = get_doc(filename);
    // let selector = Selector::parse("table").unwrap();
    // let table_el = *doc.select(&selector).nth(index).unwrap();
    let (tree_before, tree_after) = extractor.locate_content_before_and_after(table_el).unwrap();

    let mut context_before: Vec<RichText> = vec![];
    let mut context_after: Vec<RichText> = vec![];

    extractor.flatten_tree_recur(&tree_before, tree_before.get_root_id(), &mut context_before);
    extractor.flatten_tree_recur(&tree_after, tree_after.get_root_id(), &mut context_after);

    context_before.len() + context_after.len()
}

fn criterion_benchmark(c: &mut Criterion) {
    let extractor = ContextExtractor::default();
    let filename = "wikipedia/List_of_highest_mountains_on_Earth.html";

    let html_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/resources")
        .join(filename);
    let html = fs::read_to_string(html_file).unwrap();
    let doc = Html::parse_document(&html);

    let selector = Selector::parse("table").unwrap();
    let elements = doc.select(&selector).collect::<Vec<_>>();

    let mut group = c.benchmark_group("Context-Extractor");

    for i in 0..elements.len() {
        // group.bench_with_input(
        //     BenchmarkId::new("iterative", i),
        //     &(filename, i),
        //     |b, (fname, i)| b.iter(|| extract_context(&extractor, fname, *i)),
        // );
        // group.bench_with_input(
        //     BenchmarkId::new("recursive", i),
        //     &(filename, i),
        //     |b, (fname, i)| b.iter(|| extract_context_recur(&extractor, fname, *i)),
        // );
        group.bench_with_input(BenchmarkId::new("iterative", i), &i, |b, i| {
            b.iter(|| extract_context(&extractor, *elements[*i]))
        });
        group.bench_with_input(BenchmarkId::new("recursive", i), &i, |b, i| {
            b.iter(|| extract_context_recur(&extractor, *elements[*i]))
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
