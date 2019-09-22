#[macro_use]
extern crate criterion;

use criterion::{BenchmarkId, Criterion};
use rusttype::*;

#[inline]
fn rasterize(font: &Font, character: char, size: f32) -> Vec<u8> {
    let glyph = font.glyph(character).scaled(Scale::uniform(size)).positioned(rusttype::point(0.0, 0.0));
    let rect = glyph.pixel_bounding_box().unwrap();
    let mut buffer = vec![0u8; (rect.width() * rect.height()) as usize];
    glyph.draw(|x, y, v| {
        buffer[(x as usize) + (y as usize) * rect.width() as usize] = (v * 255.0) as u8;
    });
    buffer
}

fn rusttype_benchmark(c: &mut Criterion) {
    // Loading
    let font = include_bytes!("../resources/Roboto-Regular.ttf") as &[u8];
    let font = Font::from_bytes(font).unwrap();

    let mut group = c.benchmark_group("RustType: Rasterize 'g'");
    for size in [12.0, 24.0, 36.0, 48.0].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| rasterize(&font, 'g', size));
        });
    }
    group.finish();
}

criterion_group!(benches, rusttype_benchmark);
criterion_main!(benches);
