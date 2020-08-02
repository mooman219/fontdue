#[macro_use]
extern crate criterion;

use criterion::{BenchmarkId, Criterion};
use fontdue::*;

// Scratch pad for glyphs: ⅞ g
const CHARACTER: char = 'g';
const SIZES: [f32; 4] = [20.0, 40.0, 60.0, 80.0];

#[inline]
fn rasterize(font: &mut Font, character: char, size: f32) -> (Metrics, Vec<u8>) {
    font.rasterize(character, size)
}

fn fontdue_benchmark(c: &mut Criterion) {
    // Loading
    let font = include_bytes!("../resources/Roboto-Regular.ttf") as &[u8];
    let mut font = Font::from_bytes(font, FontSettings::default()).unwrap();

    let mut group = c.benchmark_group(format!("Fontdue Rasterize '{}'", CHARACTER));
    group.measurement_time(core::time::Duration::from_secs(10));
    group.sample_size(1000);
    for size in SIZES.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| rasterize(&mut font, CHARACTER, size));
        });
    }
    group.finish();
}

criterion_group!(benches, fontdue_benchmark);
criterion_main!(benches);
