use std::env;
use std::fs::File;
use std::io::prelude::*;
extern crate fontdue;


fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("supply file name");
    }

    let mut file = File::open(&args[1])?;
    let mut buffer = Vec::<u8>::new();
    file.read_to_end(&mut buffer)?;
    let data_font_part = &buffer[0..buffer.len()-4];
    let data_char_part = &buffer[buffer.len()-4-1..buffer.len()];

    if let Ok(s) = std::str::from_utf8(&data_char_part) {
            if let Some(chr) = s.chars().last() {
                println!("file={0}, char={1}", &args[1], chr);
                let font = fontdue::Font::from_bytes(data_font_part, fontdue::FontSettings::default());
                println!("parsed font");
                if !font.is_ok() {
                    return Ok(());
                }
                let font = font.unwrap();
                println!("rasterize...");
                let _ = font.rasterize(chr, 60.0);
            }
    }

    return Ok(());
}
