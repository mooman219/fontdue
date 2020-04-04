use crate::parse::*;
use crate::FontResult;
use hashbrown::HashMap;

// Apple: https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6.html
// Microsoft: https://docs.microsoft.com/en-us/typography/opentype/spec/otff

#[derive(Debug, PartialEq)]
pub struct TableDirectory {
    pub map: HashMap<[u8; 4], TableOffset>,
}

#[derive(Debug, PartialEq, Hash)]
pub struct TableOffset {
    pub checksum: u32,
    pub offset: usize,
    pub length: u32,
}

impl TableDirectory {
    pub fn new(data: &[u8]) -> FontResult<TableDirectory> {
        if data.len() < 4 {
            return Err("Font: File isn't large enough to be a font.");
        }
        let mut stream = Stream::new(data);
        let version = stream.read_u32();
        if !Self::is_font(version) {
            return Err("Font: Unsupported font type.");
        }
        let table_count = stream.read_u16();
        stream.skip(6); // searchRange: u16, entrySelector: u16, rangeShift: u16
        let mut map = HashMap::new();
        for _ in 0..table_count {
            let identifier = stream.read_tag();
            let checksum = stream.read_u32();
            let offset = stream.read_u32() as usize;
            let length = stream.read_u32();
            map.insert(
                identifier,
                TableOffset {
                    checksum,
                    offset,
                    length,
                },
            );
        }
        Ok(TableDirectory {
            map,
        })
    }

    fn is_font(tag: u32) -> bool {
        tag == 0x7472_7565 // true
        || tag == 0x7479_7031 // typ1
        || tag == 0x4F54_544F // OTTO
        || tag == 0x0001_0000 // The normal one
    }

    fn is_collection(tag: u32) -> bool {
        tag == 0x7474_6366 // ttcf
    }
}
