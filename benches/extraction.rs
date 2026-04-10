#![allow(clippy::panic)]

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use decruft::{DecruftOptions, parse};

fn load_fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|_| {
        let path2 = format!(
            "{}/tests/fixtures/defuddle/{name}",
            env!("CARGO_MANIFEST_DIR")
        );
        std::fs::read_to_string(&path2).unwrap_or_else(|_| panic!("fixture not found: {name}"))
    })
}

fn opts(url: &str) -> DecruftOptions {
    let mut o = DecruftOptions::default();
    o.url = Some(url.into());
    o
}

fn bench_small_page(c: &mut Criterion) {
    let html = load_fixture("complex_blog.html");
    let opts = opts("https://example.com/blog");
    c.bench_function("small_page (12KB blog)", |b| {
        b.iter(|| parse(black_box(&html), black_box(&opts)));
    });
}

fn bench_medium_page(c: &mut Criterion) {
    let html = load_fixture("general--stephango.com-buy-wisely.html");
    let opts = opts("https://stephango.com/buy-wisely");
    c.bench_function("medium_page (317KB stephango)", |b| {
        b.iter(|| parse(black_box(&html), black_box(&opts)));
    });
}

fn bench_large_page(c: &mut Criterion) {
    let html = load_fixture("wikipedia_bengaluru.html");
    let opts = opts("https://en.wikipedia.org/wiki/Bengaluru");
    c.bench_function("large_page (1.1MB wikipedia)", |b| {
        b.iter(|| parse(black_box(&html), black_box(&opts)));
    });
}

fn bench_github_fixture(c: &mut Criterion) {
    let html = load_fixture("general--github.com-issue-56.html");
    let opts = opts("https://github.com/kepano/defuddle/issues/56");
    c.bench_function("github_issue (267KB)", |b| {
        b.iter(|| parse(black_box(&html), black_box(&opts)));
    });
}

fn bench_markdown_output(c: &mut Criterion) {
    let html = load_fixture("complex_blog.html");
    let mut opts = opts("https://example.com/blog");
    opts.markdown = true;
    c.bench_function("markdown_output (12KB blog)", |b| {
        b.iter(|| parse(black_box(&html), black_box(&opts)));
    });
}

fn bench_no_scoring(c: &mut Criterion) {
    let html = load_fixture("wikipedia_bengaluru.html");
    let mut opts = opts("https://en.wikipedia.org/wiki/Bengaluru");
    opts.remove_low_scoring = false;
    c.bench_function("large_page_no_scoring (1.1MB)", |b| {
        b.iter(|| parse(black_box(&html), black_box(&opts)));
    });
}

criterion_group!(
    benches,
    bench_small_page,
    bench_medium_page,
    bench_large_page,
    bench_github_fixture,
    bench_markdown_output,
    bench_no_scoring,
);
criterion_main!(benches);
