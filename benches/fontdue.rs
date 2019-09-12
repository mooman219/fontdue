#[macro_use]
extern crate criterion;

use criterion::black_box;
use criterion::Criterion;
use fontdue::*;

fn rasterize(font: &Font, character: char, size: f32) -> (Metrics, Vec<u8>) {
    font.rasterize(character, size)
}

fn rasterize_indexed(font: &Font, index: usize, size: f32) -> (Metrics, Vec<u8>) {
    font.rasterize_indexed(index, size)
}

fn criterion_benchmark(c: &mut Criterion) {
    // Loading
    let font = include_bytes!("resources/Roboto-Regular.ttf") as &[u8];
    let font = Font::from_bytes(font).unwrap();

    c.bench_function("Fontdue: Rasterize 'g' at 12", |b| b.iter(|| rasterize(black_box(&font), 'g', 12.0)));
    c.bench_function("Fontdue: Rasterize 'g' at 24", |b| b.iter(|| rasterize(black_box(&font), 'g', 24.0)));

    // Indexed benchmarks
    let g_index = font.lookup_glyph_index('g');
    c.bench_function("Fontdue: Rasterize 'g' indexed at 12", |b| {
        b.iter(|| rasterize_indexed(black_box(&font), g_index, 12.0))
    });
    c.bench_function("Fontdue: Rasterize 'g' indexed at 24", |b| {
        b.iter(|| rasterize_indexed(black_box(&font), g_index, 24.0))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
