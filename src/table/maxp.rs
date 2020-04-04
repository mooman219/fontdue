use crate::parse::*;
use crate::FontResult;

// Apple: https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6maxp.html
// Microsoft: https://docs.microsoft.com/en-us/typography/opentype/spec/maxp

#[derive(Debug, PartialEq)]
pub struct TableMaxp {
    pub num_glyphs: u16,
}

impl TableMaxp {
    pub fn new(maxp: &[u8]) -> FontResult<TableMaxp> {
        let mut stream = Stream::new(maxp);
        stream.skip(4); // version: u32
        let num_glyphs = stream.read_u16();
        Ok(TableMaxp {
            num_glyphs,
        })
    }
}
