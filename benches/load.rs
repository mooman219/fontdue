#[macro_use]
extern crate criterion;

use criterion::{measurement::WallTime, BenchmarkGroup, BenchmarkId, Criterion};

type SetupFunction = fn(&mut BenchmarkGroup<WallTime>, &str, &[u8]);

// Scratch pad for glyphs: ⅞ g
const CHARACTER: char = 'L';
const SIZE: f32 = 10.0;
const FONTS: [(&str, &[u8]); 1] = [("roboto", include_bytes!("../resources/fonts/Roboto-Regular.ttf"))];
const FUNCTIONS: [SetupFunction; 4] = [setup_rusttype, setup_ab_glyph, setup_fontdue, setup_freetype];

fn setup(c: &mut Criterion) {
    let mut group = c.benchmark_group("load");
    group.measurement_time(core::time::Duration::from_secs(4));
    for function in FUNCTIONS.iter() {
        for (label, font) in FONTS.iter() {
            function(&mut group, label, font);
        }
    }
    group.finish();
}

fn setup_rusttype(group: &mut BenchmarkGroup<WallTime>, font_label: &str, font: &[u8]) {
    use rusttype::{Font, Scale};

    let parameter = format!("rusttype {}", font_label);
    group.bench_function(BenchmarkId::from_parameter(parameter), |b| {
        b.iter(|| {
            let font = Font::try_from_bytes(font).unwrap();
            let glyph =
                font.glyph(CHARACTER).scaled(Scale::uniform(SIZE)).positioned(rusttype::point(0.0, 0.0));
            let (height, width) = if let Some(rect) = glyph.pixel_bounding_box() {
                (rect.height(), rect.width())
            } else {
                (0, 0)
            };
            let mut bitmap = vec![0u8; (width * height) as usize];
            glyph.draw(|x, y, v| {
                bitmap[(x as usize) + (y as usize) * width as usize] = (v * 255.0) as u8;
            });
            bitmap
        });
    });
}

fn setup_ab_glyph(group: &mut BenchmarkGroup<WallTime>, font_label: &str, font: &[u8]) {
    use ab_glyph::{point, Font, FontRef, Glyph};

    let parameter = format!("ab_glyph {}", font_label);
    group.bench_function(BenchmarkId::from_parameter(parameter), |b| {
        b.iter(|| {
            let font = FontRef::try_from_slice(font).unwrap();
            let glyph: Glyph = font.glyph_id(CHARACTER).with_scale_and_position(SIZE, point(0.0, 0.0));
            if let Some(outlined) = font.outline_glyph(glyph) {
                let bounds = outlined.px_bounds();
                let width = bounds.width() as usize;
                let height = bounds.height() as usize;
                let length = width * height;
                let mut bitmap = vec![0u8; length];
                outlined.draw(|x, y, c| {
                    bitmap[(x as usize) + (y as usize) * width as usize] = (c * 255.0) as u8;
                });
                bitmap
            } else {
                panic!("Unsupported glyph.");
            }
        });
    });
}

fn setup_fontdue(group: &mut BenchmarkGroup<WallTime>, font_label: &str, font: &[u8]) {
    use fontdue::{Font, FontSettings};

    let parameter = format!("fontdue {}", font_label);
    group.bench_function(BenchmarkId::from_parameter(parameter), |b| {
        b.iter(|| {
            let settings = FontSettings {
                scale: SIZE,
                ..FontSettings::default()
            };
            let font = Font::from_bytes(font, settings).unwrap();
            let (_, bitmap) = font.rasterize(CHARACTER, SIZE);
            bitmap
        })
    });
}

#[cfg(feature = "freetype_benchmark")]
fn setup_freetype(group: &mut BenchmarkGroup<WallTime>, font_label: &str, font: &[u8]) {
    use freetype::Library;

    let parameter = format!("freetype {}", font_label);
    group.bench_function(BenchmarkId::from_parameter(parameter), |b| {
        b.iter(|| {
            let lib = Library::init().unwrap();
            let font = font.to_vec();
            let face = lib.new_memory_face(font, 0).unwrap();
            face.set_char_size(0, (SIZE * 0.75 * 64.0) as isize, 96, 96).unwrap();
            face.load_char(CHARACTER as usize, freetype::face::LoadFlag::RENDER).unwrap();
            let glyph = face.glyph();
            glyph.bitmap()
        })
    });
}

#[cfg(not(feature = "freetype_benchmark"))]
fn setup_freetype(_: &mut BenchmarkGroup<WallTime>, _: &str, _: &[u8]) {}

criterion_group!(benches, setup);
criterion_main!(benches);
