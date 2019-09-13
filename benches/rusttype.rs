#[macro_use]
extern crate criterion;

use criterion::black_box;
use criterion::Criterion;
use rusttype::*;

fn rasterize(font: &Font, character: char, size: f32) -> Vec<u8> {
    let glyph = font.glyph(character).scaled(Scale::uniform(size)).positioned(rusttype::point(0.0, 0.0));
    let rect = glyph.pixel_bounding_box().unwrap();
    let mut buffer = vec![0u8; (rect.width() * rect.height()) as usize];
    glyph.draw(|x, y, v| {
        buffer[(x as usize) + (y as usize) * rect.width() as usize] = (v * 255.0) as u8;
    });
    buffer
}

fn criterion_benchmark(c: &mut Criterion) {
    // Loading
    let font = include_bytes!("../resources/Roboto-Regular.ttf") as &[u8];
    let font = Font::from_bytes(font).unwrap();

    c.bench_function("RustType: Rasterize 'g' at 12", |b| b.iter(|| rasterize(black_box(&font), 'g', 12.0)));
    c.bench_function("RustType: Rasterize 'g' at 24", |b| b.iter(|| rasterize(black_box(&font), 'g', 24.0)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
