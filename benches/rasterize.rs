#[macro_use]
extern crate criterion;

use ab_glyph;
use criterion::{BenchmarkId, Criterion};
use fontdue;
use rusttype;

// Scratch pad for glyphs: ⅞ g
const CHARACTER: char = 'g';
const SIZES: [f32; 4] = [20.0, 40.0, 60.0, 80.0];

fn rusttype_benchmark(c: &mut Criterion) {
    let font = include_bytes!("../resources/Roboto-Regular.ttf") as &[u8];
    let font = rusttype::Font::try_from_bytes(font).unwrap();

    let mut group = c.benchmark_group(format!("rusttype rasterize '{}'", CHARACTER));
    group.measurement_time(core::time::Duration::from_secs(4));
    group.sample_size(500);
    for size in SIZES.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let glyph = font
                    .glyph(CHARACTER)
                    .scaled(rusttype::Scale::uniform(size))
                    .positioned(rusttype::point(0.0, 0.0));
                let (height, width) = if let Some(rect) = glyph.pixel_bounding_box() {
                    (rect.height(), rect.width())
                } else {
                    (0, 0)
                };
                let mut bitmap = vec![0u8; (width * height) as usize];
                glyph.draw(|x, y, v| {
                    bitmap[(x as usize) + (y as usize) * width as usize] = (v * 255.0) as u8;
                });
                assert!(bitmap.len() > 0);
                bitmap
            });
        });
    }
    group.finish();
}

fn ab_glyph_benchmark(c: &mut Criterion) {
    // Loading
    use ab_glyph::{point, Font, FontRef, Glyph};
    let font = FontRef::try_from_slice(include_bytes!("../resources/Roboto-Regular.ttf")).unwrap();

    let mut group = c.benchmark_group(format!("ab_glyph rasterize '{}'", CHARACTER));
    group.measurement_time(core::time::Duration::from_secs(4));
    group.sample_size(500);
    for size in SIZES.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let glyph: Glyph = font.glyph_id(CHARACTER).with_scale_and_position(size, point(0.0, 0.0));
                if let Some(outlined) = font.outline_glyph(glyph) {
                    let bounds = outlined.px_bounds();
                    let width = bounds.width() as usize;
                    let height = bounds.height() as usize;
                    let length = width * height;
                    let mut bitmap = vec![0u8; length];
                    outlined.draw(|x, y, c| {
                        bitmap[(x as usize) + (y as usize) * width as usize] = (c * 255.0) as u8;
                    });
                    assert!(bitmap.len() > 0);
                    bitmap
                } else {
                    panic!("Missing outline");
                }
            });
        });
    }
    group.finish();
}

fn fontdue_benchmark(c: &mut Criterion) {
    let font = include_bytes!("../resources/Roboto-Regular.ttf") as &[u8];

    let mut group = c.benchmark_group(format!("fontdue rasterize '{}'", CHARACTER));
    group.measurement_time(core::time::Duration::from_secs(4));
    group.sample_size(500);
    for size in SIZES.iter() {
        let settings = fontdue::FontSettings {
            scale: *size,
            ..fontdue::FontSettings::default()
        };
        let font = fontdue::Font::from_bytes(font, settings).unwrap();
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let (_, bitmap) = font.rasterize(CHARACTER, size);
                assert!(bitmap.len() > 0);
                bitmap
            });
        });
    }
    group.finish();
}

criterion_group!(benches, rusttype_benchmark, ab_glyph_benchmark, fontdue_benchmark);
criterion_main!(benches);
