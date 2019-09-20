use crate::parse::*;
use crate::FontResult;
use alloc::vec::*;

// Apple: https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6vmtx.html
// Microsoft: https://docs.microsoft.com/en-us/typography/opentype/spec/vmtx

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct VMetric {
    pub advance_height: u16,
    pub top_side_bearing: i16,
}

pub struct TableVmtx {
    /// Indexed by glyph id.
    pub vmetrics: Vec<VMetric>,
}

impl TableVmtx {
    pub fn new(vmtx: &[u8], num_glyphs: u16, num_long_vmetrics: u16) -> FontResult<TableVmtx> {
        let mut vmetrics = Vec::with_capacity(num_glyphs as usize);
        let mut advance_height = 0;
        for i in 0..num_long_vmetrics as usize {
            advance_height = read_u16(&vmtx[(i * 4)..]);
            let top_side_bearing = read_i16(&vmtx[2 + (i * 4)..]);
            vmetrics.push(VMetric {
                advance_height,
                top_side_bearing,
            });
        }
        let top_side_bearing_offset = num_long_vmetrics as usize * 4;
        for i in 0..(num_glyphs - num_long_vmetrics) as usize {
            let top_side_bearing = read_i16(&vmtx[(i * 2) + top_side_bearing_offset..]);
            vmetrics.push(VMetric {
                advance_height,
                top_side_bearing,
            });
        }
        Ok(TableVmtx {
            vmetrics,
        })
    }
}
