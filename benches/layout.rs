#[macro_use]
extern crate criterion;

use criterion::{BenchmarkId, Criterion};
use fontdue::layout::{Layout, LayoutSettings, TextStyle};
use glyph_brush_layout::{ab_glyph::*, *};

const MESSAGE: &str = "Lorem ipsum is a pseudo-Latin text used in web design, typography, layout, and printing in place of English to emphasise design elements over content. Lorem ipsum is a pseudo-Latin text used in web design, typography, layout, and printing in place of English to emphasise design elements over content.\n:D";
// const MESSAGE: &str = "This is a short ASCII string.";

fn fontdue_layout_benchmark(c: &mut Criterion) {
    // Loading
    let font = include_bytes!("../resources/Roboto-Regular.ttf") as &[u8];
    let roboto_regular = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
    let mut layout = Layout::new();
    let mut output = Vec::new();
    let settings = LayoutSettings {
        max_width: Some(200.0),
        ..LayoutSettings::default()
    };
    let fonts = &[roboto_regular];
    let styles = &[&TextStyle::new(MESSAGE, 20.0, 0)];

    let mut group = c.benchmark_group(format!("Fontdue Layout '{}'", MESSAGE));
    group.measurement_time(core::time::Duration::from_secs(5));
    group.sample_size(250);
    group.bench_function(BenchmarkId::from_parameter(MESSAGE), |b| {
        b.iter(|| {
            layout.layout_horizontal(fonts, styles, &settings, &mut output);
            output.len()
        });
    });
    group.finish();
}

fn glyph_brush_layout_benchmark(c: &mut Criterion) {
    // Loading
    let font = include_bytes!("../resources/Roboto-Regular.ttf") as &[u8];
    let font = FontRef::try_from_slice(font).unwrap();
    let fonts = &[font];
    let settings = &SectionGeometry {
        screen_position: (0.0, 0.0),
        bounds: (200.0, f32::INFINITY),
    };
    let styles = &[SectionText {
        text: MESSAGE,
        scale: PxScale::from(20.0),
        font_id: FontId(0),
    }];
    let layout = glyph_brush_layout::Layout::default();

    let mut group = c.benchmark_group(format!("Glyphbrush Layout '{}'", MESSAGE));
    group.measurement_time(core::time::Duration::from_secs(5));
    group.sample_size(250);
    group.bench_function(BenchmarkId::from_parameter(MESSAGE), |b| {
        b.iter(|| {
            let output = layout.calculate_glyphs(fonts, settings, styles);
            output.len()
        });
    });
    group.finish();
}

criterion_group!(benches, fontdue_layout_benchmark, glyph_brush_layout_benchmark);
criterion_main!(benches);
