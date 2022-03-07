#[macro_use]
extern crate criterion;

use criterion::{BenchmarkId, Criterion};
use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
use glyph_brush_layout::{ab_glyph::*, *};

const MESSAGES: [&str; 3] = ["Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore ", "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Tempor orci eu lobortis elementum nibh tellus. Mi tempus imperdiet nulla malesuada pellentesque elit eget gravida cum. Non nisi est sit amet facilisis magna etiam tempor. In fermentum et sollicitudin ac. Nunc consequat interdum varius sit amet mattis. Est velit egestas dui id ornare arcu odio ut. Venenatis lectus magna fringilla urna porttitor rhoncus dolor purus non. Lobor", "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Feugiat nibh sed pulvinar proin gravida hendrerit. Duis ut diam quam nulla porttitor massa id neque. Lacus viverra vitae congue eu consequat ac felis. Etiam non quam lacus suspendisse faucibus. Eget mauris pharetra et ultrices neque ornare. Libero id faucibus nisl tincidunt eget nullam non. Justo laoreet sit amet cursus sit amet. Velit laoreet id donec ultrices tincidunt arcu non sodales neque.

Aliquet nibh praesent tristique magna sit. Purus viverra accumsan in nisl nisi scelerisque. Tortor vitae purus faucibus ornare suspendisse sed nisi. Dolor sit amet consectetur adipiscing elit pellentesque habitant. Egestas purus viverra accumsan in nisl. Amet venenatis urna cursus eget nunc scelerisque. Dictumst quisque sagittis purus sit amet volutpat. Vel risus commodo viverra maecenas. Imperdiet nulla malesuada pellentesque elit eget gravida cum sociis natoque. Nibh ips"];

fn fontdue_layout_benchmark(c: &mut Criterion) {
    // Loading
    let font = include_bytes!("../resources/fonts/Roboto-Regular.ttf") as &[u8];
    let roboto_regular = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
    let mut layout = Layout::new(CoordinateSystem::PositiveYUp);
    layout.reset(&LayoutSettings {
        max_width: Some(200.0),
        ..LayoutSettings::default()
    });
    let fonts = &[roboto_regular];

    let mut group = c.benchmark_group("layout/fontdue");
    group.measurement_time(core::time::Duration::from_secs(4));
    group.sample_size(250);
    for message in MESSAGES.iter() {
        let style = &TextStyle::new(message, 20.0, 0);
        group.bench_with_input(BenchmarkId::from_parameter(message.len()), &message, |b, _| {
            b.iter(|| {
                layout.clear();
                layout.append(fonts, style);
                layout.glyphs().len()
            });
        });
    }
    group.finish();
}

fn glyph_brush_layout_benchmark(c: &mut Criterion) {
    // Loading
    let font = include_bytes!("../resources/fonts/Roboto-Regular.ttf") as &[u8];
    let font = FontRef::try_from_slice(font).unwrap();
    let fonts = &[font];
    let settings = &SectionGeometry {
        screen_position: (0.0, 0.0),
        bounds: (200.0, f32::INFINITY),
    };
    let layout = glyph_brush_layout::Layout::default();

    let mut group = c.benchmark_group("layout/glyph_brush_layout");
    group.measurement_time(core::time::Duration::from_secs(4));
    group.sample_size(250);
    for message in MESSAGES.iter() {
        let styles = &[SectionText {
            text: message,
            scale: PxScale::from(20.0),
            font_id: FontId(0),
        }];
        group.bench_with_input(BenchmarkId::from_parameter(message.len()), &message, |b, _| {
            b.iter(|| {
                let output = layout.calculate_glyphs(fonts, settings, styles);
                output.len()
            });
        });
    }
    group.finish();
}

criterion_group!(benches, fontdue_layout_benchmark, glyph_brush_layout_benchmark);
criterion_main!(benches);
