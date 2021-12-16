use fontdue::{Font, FontSettings};
use std::{convert::TryInto, fs, io::Cursor, path::Path};
use walkdir::WalkDir;

extern crate png;

use crate::modules::{FONTS, FONT_NAMES};
const SIZES: [f32; 1] = [32.0];

fn clean_local_baselines() {
    fs::remove_dir_all("./resources/baselines/local/characters").ok();
}

fn record_local_baseline(font: &Font, name: &str, character_index: u16, size: f32) -> Option<()> {
    let testcase_name = format!("{}/{}-{}px.png", name, character_index, size);
    let reference_path = format!("./resources/baselines/reference/characters/{}", testcase_name);
    let local_path = format!("./resources/baselines/local/characters/{}", testcase_name);
    let (metrics, new_bitmap) = font.rasterize_indexed(character_index, size);
    if metrics.width == 0 || metrics.height == 0 {
        // No glyph rendered, assert file does not exist on disk
        if Path::new(&reference_path).exists() {
            fs::create_dir_all(Path::new(&local_path).parent()?).ok();
            return fs::write(
                Path::new(&reference_path)
                    .parent()?
                    .join(format!("{}.delete", Path::new(&reference_path).file_stem()?.to_str()?)),
                [],
            )
            .ok();
        }
        return None;
    }
    let mut encoded = vec![];
    let mut encoder = png::Encoder::new(
        Cursor::new(&mut encoded),
        metrics.width.try_into().unwrap(),
        metrics.height.try_into().unwrap(),
    );
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header().unwrap().write_image_data(&new_bitmap).unwrap();
    fs::create_dir_all(Path::new(&reference_path).parent()?).ok();
    if !Path::new(&reference_path).exists() || fs::read(reference_path).unwrap() != encoded {
        // only write out local bitmap when it doesn't match (this saves a lot of disk i/o time)
        fs::create_dir_all(Path::new(&local_path).parent()?).ok();
        if let Err(err) = fs::write(&local_path, encoded) {
            panic!("Unable to save file {}: {}", &local_path, err);
        }
    }
    None
}

fn report_changed_baselines() {
    let reference_path = "./resources/baselines/reference/characters";
    let local_path = "./resources/baselines/local/characters";
    let mut failures: Vec<String> = vec![];
    for entry in WalkDir::new(local_path).into_iter().filter_map(|e| e.ok()) {
        if entry.path().is_dir() {
            continue;
        }
        let ext = entry.path().extension();
        let test_path = format!(
            "{}/{}",
            entry.path().parent().unwrap().file_name().unwrap().to_string_lossy(),
            entry.file_name().to_string_lossy()
        );
        match ext {
            Some(str) => {
                if str == "delete" {
                    failures.push(format!("{} no longer renders a glyph", test_path));
                    continue;
                }
            }
            None => continue,
        }
        if !Path::new(reference_path)
            .join(entry.path().parent().unwrap().file_name().unwrap())
            .join(entry.file_name())
            .exists()
        {
            failures.push(format!("{} has no reference baseline", test_path));
            continue;
        }
    }

    for entry in WalkDir::new(reference_path).into_iter().filter_map(|e| e.ok()) {
        if entry.path().is_dir() {
            continue;
        }
        let test_path = format!(
            "{}/{}",
            entry.path().parent().unwrap().file_name().unwrap().to_string_lossy(),
            entry.file_name().to_string_lossy()
        );
        let local_entry = Path::new(local_path)
            .join(entry.path().parent().unwrap().file_name().unwrap())
            .join(entry.file_name());
        if !local_entry.exists() {
            continue;
        }
        failures.push(format!("{} image data did not match", test_path));
    }

    assert!(
        failures.is_empty(),
        "Baseline failures:
{}

You can view these differences with a difftool and a command like
$DIFFTOOL '.\\resources\\baselines\\reference\\' '.\\resources\\baselines\\local\\'
or you can blindly accept all differences by running
cargo run --example baseline-accept
",
        failures.join("\n")
    );
}

#[test]
fn baseline_all() {
    clean_local_baselines();
    for (index, bytes) in (&FONTS).iter().enumerate() {
        let name = FONT_NAMES[index];
        let font = Font::from_bytes(*bytes, FontSettings::default()).unwrap();
        for g in 0..font.glyph_count() {
            for size in &SIZES {
                record_local_baseline(&font, name, g, *size);
            }
        }
    }
    report_changed_baselines();
}
