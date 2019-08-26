use fontdue::raw::*;
use fontdue::Raster;
use std::fs::File;
use std::io::Write;

pub fn main() {
    let font = include_bytes!("resources/Roboto-Regular.ttf") as &[u8];
    let font = RawFont::new(font).unwrap();
    let glyph = font.glyf.glyphs[52].clone();
    for point in &glyph.points {
        println!("{:?}", point);
    }
    let geometry = fontdue::to_geometry(&glyph.points);
    for element in &geometry {
        println!("{:?}", element);
    }
    let mut raster = Raster::new(glyph.xmax as usize, glyph.ymax as usize);
    for element in &geometry {
        raster.draw(element);
    }
    let bitmap = raster.get_bitmap();
    let mut o = File::create("letter.pgm").unwrap();
    let _ = o.write(format!("P5\n{} {}\n255\n", glyph.xmax, glyph.ymax).as_bytes());
    let _ = o.write(&bitmap);
}
