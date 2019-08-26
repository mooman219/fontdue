use fontdue::{Point, Raster};
use std::fs::File;
use std::io::Write;

pub fn main() {
    let mut raster = Raster::new(100, 100);
    let a = Point::new(1.0, 1.0);
    let b = Point::new(99.0, 1.0);
    let c = Point::new(50.0, 99.0);
    raster.draw_curve(&a, &c, &b);

    let bitmap = raster.get_bitmap();
    let mut o = File::create("shape.pgm").unwrap();
    let _ = o.write(format!("P5\n{} {}\n255\n", 100, 100).as_bytes());
    let _ = o.write(&bitmap);
}
