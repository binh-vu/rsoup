use anyhow::Result;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use ego_tree::NodeRef;
use rsoup::extractors::context_v1::ContextExtractor;
use rsoup::models::rich_text::RichText;
use scraper::{ElementRef, Html, Node, Selector};
use std::{fs, path::Path};

fn get_doc(filename: &str) -> Html {
    let html_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/resources")
        .join("wikipedia/List_of_highest_mountains_on_Earth.html");
    let html = fs::read_to_string(html_file).unwrap();
    Html::parse_document(&html)
}

fn get_text_v1(elements: &[ElementRef]) -> usize {
    let mut count = 0;
    for el in elements {
        count += rsoup::extractors::text::get_text_v1::get_text(el).len();
    }
    count
}

fn get_text_v2(elements: &[ElementRef]) -> usize {
    let mut count = 0;
    for el in elements {
        count += rsoup::extractors::text::get_text_v2::get_text(el).len();
    }
    count
}

fn criterion_benchmark(c: &mut Criterion) {
    let extractor = ContextExtractor::default();
    let filename = "wikipedia/List_of_highest_mountains_on_Earth.html";

    let html_file = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/resources")
        .join(filename);
    let html = fs::read_to_string(html_file).unwrap();
    let doc = Html::parse_document(&html);

    let query = "td";
    let selector = Selector::parse(query).unwrap();
    let elements = doc.select(&selector).collect::<Vec<_>>();

    let mut group = c.benchmark_group("Get Text");

    group.measurement_time(std::time::Duration::from_secs(20));
    group.bench_function("get_text_v1", |b| b.iter(|| get_text_v1(&elements)));
    group.bench_function("get_text_v2", |b| b.iter(|| get_text_v2(&elements)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
