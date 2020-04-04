use crate::parse::*;
use crate::FontResult;
use alloc::vec::*;

// Apple: https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6loca.html
// Microsoft: https://docs.microsoft.com/en-us/typography/opentype/spec/loca

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GlyphLocation {
    pub offset: usize,
    pub length: usize,
}

#[derive(Debug, PartialEq)]
pub struct TableLoca {
    /// Indexed by glyph id.
    pub locations: Vec<GlyphLocation>,
}

impl TableLoca {
    pub fn new(loca: &[u8], index_to_loc_format: i16, num_glyphs: u16) -> FontResult<TableLoca> {
        if index_to_loc_format > 1 {
            return Err("Font.loca: Unknown index_to_loc_format");
        }
        let mut stream = Stream::new(loca);
        let mut locations = Vec::with_capacity(num_glyphs as usize);
        if index_to_loc_format == 0 {
            let mut offset = stream.read_u16() as usize * 2;
            for _ in 0..num_glyphs {
                let next_offset = stream.read_u16() as usize * 2;
                locations.push(GlyphLocation {
                    offset,
                    length: next_offset - offset,
                });
                offset = next_offset;
            }
        } else {
            let mut offset = stream.read_u32() as usize;
            for _ in 0..num_glyphs {
                let next_offset = stream.read_u32() as usize;
                locations.push(GlyphLocation {
                    offset,
                    length: next_offset - offset,
                });
                offset = next_offset;
            }
        }
        Ok(TableLoca {
            locations,
        })
    }
}
