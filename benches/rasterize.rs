#[macro_use]
extern crate criterion;

use criterion::{BenchmarkId, Criterion};
use fontdue;
use rusttype;

// Scratch pad for glyphs: ⅞ g
const CHARACTER: char = 'g';
const SIZES: [f32; 4] = [20.0, 40.0, 60.0, 80.0];

#[inline]
fn rasterize_rusttype(font: &rusttype::Font, character: char, size: f32) -> Vec<u8> {
    let glyph =
        font.glyph(character).scaled(rusttype::Scale::uniform(size)).positioned(rusttype::point(0.0, 0.0));
    let (height, width) = if let Some(rect) = glyph.pixel_bounding_box() {
        (rect.height(), rect.width())
    } else {
        (0, 0)
    };
    let mut buffer = vec![0u8; (width * height) as usize];
    glyph.draw(|x, y, v| {
        buffer[(x as usize) + (y as usize) * width as usize] = (v * 255.0) as u8;
    });
    buffer
}

#[inline]
fn rasterize_fontdue(font: &mut fontdue::Font, character: char, size: f32) -> (fontdue::Metrics, Vec<u8>) {
    font.rasterize(character, size)
}

fn rusttype_benchmark(c: &mut Criterion) {
    // Loading
    let font = include_bytes!("../resources/Roboto-Regular.ttf") as &[u8];
    let font = rusttype::Font::try_from_bytes(font).unwrap();

    let mut group = c.benchmark_group(format!("rusttype rasterize '{}'", CHARACTER));
    group.measurement_time(core::time::Duration::from_secs(5));
    group.sample_size(500);
    for size in SIZES.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| rasterize_rusttype(&font, CHARACTER, size));
        });
    }
    group.finish();
}

fn fontdue_benchmark(c: &mut Criterion) {
    // Loading
    let font = include_bytes!("../resources/Roboto-Regular.ttf") as &[u8];
    let mut font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();

    let mut group = c.benchmark_group(format!("fontdue rasterize '{}'", CHARACTER));
    group.measurement_time(core::time::Duration::from_secs(5));
    group.sample_size(500);
    for size in SIZES.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| rasterize_fontdue(&mut font, CHARACTER, size));
        });
    }
    group.finish();
}

criterion_group!(benches, rusttype_benchmark, fontdue_benchmark);
criterion_main!(benches);
