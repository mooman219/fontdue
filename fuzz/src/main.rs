#[macro_use]
extern crate afl;
extern crate fontdue;


fn main() {
    fuzz!(|data: &[u8]| {
        let data_font_part = &data[0..data.len()-4];
        let data_char_part = &data[data.len()-4-1..data.len()];

        if let Ok(s) = std::str::from_utf8(&data_char_part) {
            if let Some(chr) = s.chars().last() {
                let font = fontdue::Font::from_bytes(data_font_part, fontdue::FontSettings::default());
                if !font.is_ok() {
                    return;
                }
                let font = font.unwrap();
                let _ = font.rasterize(chr, 60.0);
            }
        }
    });
}
