use crate::parse::*;
use crate::simd::abs;
use crate::table::loca::*;
use crate::FontResult;
use alloc::vec::*;

// Apple: https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6glyf.html
// Microsoft: https://docs.microsoft.com/en-us/typography/opentype/spec/glyf

#[allow(non_snake_case)]
pub mod SimpleFlags {
    pub const NONE: u8 = 0x00;

    pub const ON_CURVE: u8 = 0x01;
    pub const X_SHORT: u8 = 0x02;
    pub const Y_SHORT: u8 = 0x04;
    pub const REPEAT: u8 = 0x08;
    pub const X_DUAL: u8 = 0x10;
    pub const Y_DUAL: u8 = 0x20;
    pub const OVERLAP_SIMPLE: u8 = 0x40;

    pub const X_SHORT_AND_DUAL: u8 = X_SHORT | X_DUAL;
    pub const Y_SHORT_AND_DUAL: u8 = Y_SHORT | Y_DUAL;
}

#[allow(non_snake_case)]
pub mod CompoundFlags {
    pub const NONE: u16 = 0x0000;
    pub const ARGS_ARE_WORDS: u16 = 0x0001;
    pub const ARGS_ARE_XY_VALUES: u16 = 0x0002;
    pub const ROUND_XY_TO_GRID: u16 = 0x0004;
    pub const WE_HAVE_A_SCALE: u16 = 0x0008;
    pub const MORE_COMPONENTS: u16 = 0x0020;
    pub const WE_HAVE_AN_X_AND_Y_SCALE: u16 = 0x0040;
    pub const WE_HAVE_A_TWO_BY_TWO: u16 = 0x0080;
    pub const WE_HAVE_INSTRUCTIONS: u16 = 0x0100;
    pub const USE_MY_METRICS: u16 = 0x0200;
    pub const OVERLAP_COMPOUND: u16 = 0x0400;
    pub const SCALED_COMPONENT_OFFSET: u16 = 0x0800;
    pub const UNSCALED_COMPONENT_OFFSET: u16 = 0x1000;

    pub const ARGS_ARE_WORDS_AND_XY_VALUES: u16 = ARGS_ARE_WORDS | ARGS_ARE_XY_VALUES;
}

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct RawPoint {
    /// Absolute X coordinate.
    pub x: f32,
    /// Absolute Y coordinate.
    pub y: f32,
    /// Flags associated with the point. The last bit (0x80) is used to mark if this is the start
    /// of a new contour. The bit is otherwise reserved in spec.
    pub flags: u8,
    /// Marks if this is the last point in the contour.
    pub end_point: bool,
    /// Marks if this is the first point in the contour.
    pub start_point: bool,
}

impl RawPoint {
    #[inline]
    pub fn on_curve(&self) -> bool {
        flag_u8(self.flags, SimpleFlags::ON_CURVE)
    }

    pub fn transform(&mut self, a: f32, b: f32, c: f32, d: f32, cx: f32, cy: f32) {
        let px = self.x;
        let py = self.y;
        self.x = a * px + c * py + cx;
        self.y = b * px + d * py + cy;
    }
}

#[derive(Debug, Clone)]
pub struct Glyph {
    /// The number of contours in the glyph.
    pub num_contours: i16,
    /// The lowest x point. This is derived from the points in case the font is malicious.
    pub xmin: f32,
    /// The highest x point. This is derived from the points in case the font is malicious.
    pub xmax: f32,
    /// The lowest y point. This is derived from the points in case the font is malicious.
    pub ymin: f32,
    /// The highest y point. This is derived from the points in case the font is malicious.
    pub ymax: f32,
    /// The index of the metrics location for this glyph.
    pub metrics: usize,
    /// The first point is always marked as the start point, and the last point is always marked as
    /// the end point.
    pub points: Vec<RawPoint>,
}

impl Glyph {
    /// Reposition vertically flips the glyph and moves it into the +x, +y quadrant of the axis.
    pub fn reposition(&mut self) {
        for point in &mut self.points {
            point.x -= self.xmin as f32;
            point.y -= self.ymax as f32;
            point.y = abs(point.y);
        }
    }
}

impl Default for Glyph {
    fn default() -> Glyph {
        Glyph {
            num_contours: 0,
            xmin: 0.0,
            xmax: 0.0,
            ymin: 0.0,
            ymax: 0.0,
            metrics: 0,
            points: Vec::with_capacity(0),
        }
    }
}

pub struct TableGlyf {
    /// Indexed by glyph id.
    pub glyphs: Vec<Glyph>,
}

// Truetype is a marvelous format.

fn parse_glyph(glyf: &[u8], locations: &[GlyphLocation], index: usize) -> FontResult<Glyph> {
    let loc = &locations[index];
    let mut glyph = Glyph::default();
    glyph.metrics = index;

    // Empty glyph, exit early.
    if loc.length == 0 {
        return Ok(glyph);
    }
    let mut stream = Stream::new(glyf);
    stream.seek(loc.offset);
    glyph.num_contours = stream.read_i16();
    // The boundary box is read here, but can be adjusted if a point goes outside of the box when
    // the glyph is being parsed.
    let xmin = stream.read_i16();
    let ymin = stream.read_i16();
    let xmax = stream.read_i16();
    let ymax = stream.read_i16();

    // Workaround for fonts in http://www.princexml.com/fonts/
    if xmin == 32767 && xmax == -32767 && ymin == 32767 && ymax == -32767 {
        glyph.xmin = 0.0;
        glyph.ymin = 0.0;
        glyph.xmax = 0.0;
        glyph.ymax = 0.0;
    } else {
        glyph.xmin = xmin as f32;
        glyph.ymin = ymin as f32;
        glyph.xmax = xmax as f32;
        glyph.ymax = ymax as f32;
    }

    // Reject bad bounding boxes.
    if glyph.xmin > glyph.xmax || glyph.ymin > glyph.ymax {
        return Err("Font.glyf: Bad glyph bounding box values (xmin > xmax || ymin > ymax)");
    }

    // No contours, exit early.
    if glyph.num_contours == 0 {
        // This is an empty glyph and shouldn’t have any glyph data, but if it does we will simply
        // ignore it.
    } else if glyph.num_contours > 0 {
        // Simple glyphs.

        // Read the contours. A contour is basically just a continous line. The letter 'O'
        // for example has 2 contours. The end_points_of_contours array basically has the
        // number of points to read for each contour.
        let mut end_points_of_contours = Vec::with_capacity(glyph.num_contours as usize);
        for _ in 0..glyph.num_contours as usize {
            let end_points_of_contour = stream.read_u16();
            end_points_of_contours.push(end_points_of_contour);
        }
        // Since end_points_of_contours indexes by the point index, the last contour has
        // the number of points in the glyph minus 1.
        let num_points = end_points_of_contours[glyph.num_contours as usize - 1] as usize + 1;

        // Skip instructions, we don't need a manual to go where we're going.
        // (Really, what is this for in 2019?)
        let instruction_length = stream.read_u16();
        stream.skip(instruction_length as usize);

        // Read flags. Flags can repeat, but they're not stored as literal repeats, they
        // instead have a byte after them for their repeat count and you just need to
        // re-use the prior flag that many times.
        glyph.points = Vec::with_capacity(num_points);
        while glyph.points.len() < num_points {
            let flags = stream.read_u8();
            let point = RawPoint {
                x: 0.0,
                y: 0.0,
                flags,
                // These are set later
                start_point: false,
                end_point: false,
            };
            glyph.points.push(point);
            if flag_u8(flags, SimpleFlags::REPEAT) {
                let count = stream.read_u8();
                for _ in 0..count {
                    glyph.points.push(point);
                }
            }
        }

        // We're now at the x-coordinates. These are not absolute positions, but deltas
        // from the last position. This will parse them into absolute positions.
        let mut last_x = 0i16;
        for point in &mut glyph.points {
            match point.flags & (SimpleFlags::X_SHORT | SimpleFlags::X_DUAL) {
                SimpleFlags::X_SHORT_AND_DUAL => {
                    last_x += stream.read_u8() as i16;
                }
                SimpleFlags::X_SHORT => {
                    last_x -= stream.read_u8() as i16;
                }
                SimpleFlags::X_DUAL => {
                    // Reuse last_x.
                }
                _ => {
                    last_x += stream.read_i16();
                }
            }
            point.x = last_x as f32;
        }

        // Finally, the y-coordinates. Parsed the same as x basically; just use the
        // y-coord version of the flags instead.
        let mut last_y = 0i16;
        for point in &mut glyph.points {
            match point.flags & SimpleFlags::Y_SHORT_AND_DUAL {
                SimpleFlags::Y_SHORT_AND_DUAL => {
                    last_y += stream.read_u8() as i16;
                }
                SimpleFlags::Y_SHORT => {
                    last_y -= stream.read_u8() as i16;
                }
                SimpleFlags::Y_DUAL => {
                    // Reuse last_y.
                }
                _ => {
                    last_y += stream.read_i16();
                }
            }
            point.y = last_y as f32;
        }

        // Mark the start and end points in each contour. This makes figuring out contours easier.
        for end_point in end_points_of_contours {
            let end_point = end_point as usize;
            glyph.points[end_point].end_point = true;
            glyph.points[(end_point + 1) % num_points].start_point = true;
        }
    } else {
        // Compound glyphs.
        let mut flags = CompoundFlags::MORE_COMPONENTS;
        while flag_u16(flags, CompoundFlags::MORE_COMPONENTS) {
            flags = stream.read_u16();
            let compound_glyph_index = stream.read_u16();
            if flag_u16(flags, CompoundFlags::USE_MY_METRICS) {
                glyph.metrics = compound_glyph_index as usize;
            }

            let cx;
            let cy;
            match flags & CompoundFlags::ARGS_ARE_WORDS_AND_XY_VALUES {
                CompoundFlags::ARGS_ARE_WORDS_AND_XY_VALUES => {
                    cx = stream.read_i16() as i32;
                    cy = stream.read_i16() as i32;
                }
                CompoundFlags::ARGS_ARE_WORDS => {
                    return Err("Font.glyf: Component matched point numbers are unsupported");
                    // TODO: Matched point numbers. Rusttype doesn't support them either.
                    // cx = read_u16(&glyf[offset..]) as i32;
                    // cy = read_u16(&glyf[offset + 2..]) as i32;
                    // offset += 4;
                }
                CompoundFlags::ARGS_ARE_XY_VALUES => {
                    cx = stream.read_i8() as i32;
                    cy = stream.read_i8() as i32;
                }
                _ => {
                    return Err("Font.glyf: Component matched point numbers are unsupported");
                    // TODO: Matched point numbers. Rusttype doesn't support them either.
                    // cx = read_u8(&glyf[offset..]) as i32;
                    // cy = read_u8(&glyf[offset + 1..]) as i32;
                    // offset += 2;
                }
            }

            let mut a = 1.0;
            let mut b = 0.0;
            let mut c = 0.0;
            let mut d = 1.0;
            if flag_u16(flags, CompoundFlags::WE_HAVE_A_SCALE) {
                a = stream.read_f2dot14();
                d = a;
            } else if flag_u16(flags, CompoundFlags::WE_HAVE_AN_X_AND_Y_SCALE) {
                a = stream.read_f2dot14();
                d = stream.read_f2dot14();
            } else if flag_u16(flags, CompoundFlags::WE_HAVE_A_TWO_BY_TWO) {
                a = stream.read_f2dot14();
                b = stream.read_f2dot14();
                c = stream.read_f2dot14();
                d = stream.read_f2dot14();
            } else {
                // Do nothing, use the values we have for a, b, c, d.
            }

            // This is the only valid configuration to use scale offsets.
            if flag_u16(flags, CompoundFlags::SCALED_COMPONENT_OFFSET)
                && !flag_u16(flags, CompoundFlags::UNSCALED_COMPONENT_OFFSET)
            {
                return Err("Font.glyf: Scaled component offset is unsupported");
                // TODO: Scaled offset. Rusttype doesn't handle this, neither does Chrome.
            }

            let mut compound_glyph_points =
                parse_glyph(glyf, locations, compound_glyph_index as usize)?.points;
            for point in &mut compound_glyph_points {
                point.transform(a, b, c, d, cx as f32, cy as f32);
            }
            glyph.points.append(&mut compound_glyph_points);
        }
    }

    // In case any of the points go outside of the bounding box, that's fixed here. This can happen
    // with a well meaning font putting an off curve point outside of the bounding box, or a
    // malicious font trying to crash the rasterizer.
    for point in &glyph.points {
        let x = point.x;
        if x < glyph.xmin {
            glyph.xmin = x;
        } else if x > glyph.xmax {
            glyph.xmax = x;
        }
        let y = point.y;
        if y < glyph.ymin {
            glyph.ymin = y;
        } else if y > glyph.ymax {
            glyph.ymax = y;
        }
    }

    Ok(glyph)
}

impl TableGlyf {
    pub fn new(glyf: &[u8], locations: &[GlyphLocation]) -> FontResult<TableGlyf> {
        let mut glyphs = Vec::with_capacity(locations.len());
        for i in 0..locations.len() {
            let glyph = parse_glyph(glyf, locations, i)?;
            glyphs.push(glyph);
        }
        Ok(TableGlyf {
            glyphs,
        })
    }
}
