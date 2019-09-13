use fontdue::raw::*;

pub fn main() {
    let font = include_bytes!("../resources/Roboto-Regular.ttf") as &[u8];
    let font = RawFont::new(font).unwrap();
    // Letter lookup code.
    let glyph = font.glyf.glyphs[298].clone();
    println!("glyph.xmin {}", glyph.xmin);
    println!("glyph.xmax {}", glyph.xmax);
    println!("glyph.ymin {}", glyph.ymin);
    println!("glyph.ymax {}", glyph.ymax);
    println!("width {}", glyph.xmax - glyph.xmin);
    println!("height {}", glyph.ymax - glyph.ymin);
    for point in &glyph.points {
        println!("{:?}", point);
    }
}
