use crate::parse::*;
use crate::FontResult;
use alloc::vec::*;

// Apple: https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6hmtx.html
// Microsoft: https://docs.microsoft.com/en-us/typography/opentype/spec/hmtx

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct HMetric {
    pub advance_width: u16,
    pub left_side_bearing: i16,
}

pub struct TableHmtx {
    /// Indexed by glyph id.
    pub hmetrics: Vec<HMetric>,
}

impl TableHmtx {
    pub fn new(hmtx: &[u8], num_glyphs: u16, num_long_hmetrics: u16) -> FontResult<TableHmtx> {
        let mut hmetrics = Vec::with_capacity(num_glyphs as usize);
        let mut advance_width = 0;
        for i in 0..num_long_hmetrics as usize {
            advance_width = read_u16(&hmtx[(i * 4)..]);
            let left_side_bearing = read_i16(&hmtx[2 + (i * 4)..]);
            hmetrics.push(HMetric {
                advance_width,
                left_side_bearing,
            });
        }
        let left_side_bearing_offset = num_long_hmetrics as usize * 4;
        for i in 0..(num_glyphs - num_long_hmetrics) as usize {
            let left_side_bearing = read_i16(&hmtx[(i * 2) + left_side_bearing_offset..]);
            hmetrics.push(HMetric {
                advance_width,
                left_side_bearing,
            });
        }
        Ok(TableHmtx {
            hmetrics,
        })
    }
}
