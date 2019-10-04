#[macro_use]
extern crate criterion;

use criterion::{BenchmarkId, Criterion};
use fontdue::*;

#[inline]
fn rasterize(font: &mut Font, character: char, size: f32) -> (Metrics, Vec<u8>) {
    font.rasterize(character, size)
}

fn fontdue_benchmark(c: &mut Criterion) {
    // Loading
    let font = include_bytes!("../resources/Roboto-Regular.ttf") as &[u8];
    let mut font = Font::from_bytes(font, FontSettings::default()).unwrap();

    let mut group = c.benchmark_group("Fontdue: Rasterize 'g'");
    for size in [12.0, 24.0, 36.0, 48.0].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| rasterize(&mut font, 'g', size));
        });
    }
    group.finish();
}

criterion_group!(benches, fontdue_benchmark);
criterion_main!(benches);
