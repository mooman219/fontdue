#[macro_use]
extern crate criterion;

use criterion::black_box;
use criterion::Criterion;
use fontdue::*;

fn rasterize_g(font: &Font) -> (Metrics, Vec<u8>) {
    font.rasterize('g', 12.0)
}

fn criterion_benchmark(c: &mut Criterion) {
    // Loading and rasterization
    let font = include_bytes!("resources/Roboto-Regular.ttf") as &[u8];
    let font = Font::from_bytes(font).unwrap();

    c.bench_function("Rasterize g", |b| b.iter(|| rasterize_g(black_box(&font))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
