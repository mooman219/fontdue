use crate::parse::*;
use crate::FontResult;

// Apple: https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6maxp.html
// Microsoft: https://docs.microsoft.com/en-us/typography/opentype/spec/maxp

pub struct TableMaxp {
    pub version: u32,
    pub num_glyphs: u16,
}

impl TableMaxp {
    pub fn new(maxp: &[u8]) -> FontResult<TableMaxp> {
        let version = read_u32(&maxp[0..]);
        let num_glyphs = read_u16(&maxp[4..]);
        Ok(TableMaxp {
            version,
            num_glyphs,
        })
    }
}
