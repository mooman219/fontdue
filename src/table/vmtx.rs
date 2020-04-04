use crate::parse::*;
use crate::FontResult;
use alloc::vec::*;

// Apple: https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6vmtx.html
// Microsoft: https://docs.microsoft.com/en-us/typography/opentype/spec/vmtx

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct VMetric {
    pub advance_height: u16,
    pub top_side_bearing: i16,
}

#[derive(Debug, PartialEq)]
pub struct TableVmtx {
    /// Indexed by glyph id.
    pub vmetrics: Vec<VMetric>,
}

impl TableVmtx {
    pub fn new(vmtx: &[u8], num_glyphs: u16, num_long_vmetrics: u16) -> FontResult<TableVmtx> {
        let mut stream = Stream::new(vmtx);
        let mut vmetrics = Vec::with_capacity(num_glyphs as usize);
        let mut advance_height = 0;
        for _ in 0..num_long_vmetrics {
            advance_height = stream.read_u16();
            let top_side_bearing = stream.read_i16();
            vmetrics.push(VMetric {
                advance_height,
                top_side_bearing,
            });
        }
        for _ in 0..(num_glyphs - num_long_vmetrics) {
            let top_side_bearing = stream.read_i16();
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
