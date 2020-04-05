use crate::parse::*;
use crate::FontResult;
use hashbrown::HashMap;

// Apple: https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6kern.html
// Microsoft: https://docs.microsoft.com/en-us/typography/opentype/spec/kern

#[derive(Debug)]
pub struct TableKern {
    pub horizontal_mappings: Option<HashMap<u32, i16>>,
    pub vertical_mappings: Option<HashMap<u32, i16>>,
}

#[derive(Copy, Clone, Debug)]
pub struct Header {
    pub version_major: u16,
    pub version_minor: u16,
    pub number_sub_tables: u32,
}

#[derive(Copy, Clone, Debug)]
pub struct SubTableHeader {
    pub version: u16,
    pub length: usize,
    pub coverage: Coverage,
    pub tuple_index: u16,
}

#[derive(Copy, Clone, Debug)]
pub struct Coverage(u16);

impl Coverage {
    pub fn is_horizontal(&self) -> bool {
        self.0 & 0x0001 == 0x0001
    }

    pub fn is_minimum(&self) -> bool {
        self.0 & 0x0002 == 0x0002
    }

    pub fn is_cross_stream(&self) -> bool {
        self.0 & 0x0004 == 0x0004
    }

    pub fn is_override(&self) -> bool {
        self.0 & 0x0008 == 0x0008
    }

    pub fn format(&self) -> u16 {
        self.0 >> 8
    }
}

impl TableKern {
    pub fn new(kern: &[u8]) -> FontResult<TableKern> {
        let mut stream = Stream::new(kern);
        let version_major = stream.read_u16();

        let header;
        match version_major {
            0x0000 => header = Self::read_header(&mut stream),
            0x0001 => header = Self::read_aat_header(&mut stream),
            _ => return Err("Font.kern: Unsupported kern table version."),
        }

        for _ in 0..header.number_sub_tables {
            let sub_table_start = stream.offset();
            let sub_header = if version_major == 0x0000 {
                Self::read_subtable(&mut stream)
            } else {
                Self::read_aat_subtable(&mut stream)
            };
            match sub_header.coverage.format() {
                // Ordered List of Kerning Pairs
                0 => {
                    let mappings = Self::read_format0(&mut stream);
                    let (h, v) = if sub_header.coverage.is_horizontal() {
                        (Some(mappings), None)
                    } else {
                        (None, Some(mappings))
                    };
                    return Ok(TableKern {
                        horizontal_mappings: h,
                        vertical_mappings: v,
                    });
                }
                // State Table for Contextual Kerning
                // 1 => { /* TODO: State Table for Contextual Kerning */ }
                // Simple n x m Array of Kerning Values
                // 2 => { /* TODO: Simple n x m Array of Kerning Values */ }
                // Simple n x m Array of Kerning Indices
                // 3 => { /* TODO: Simple n x m Array of Kerning Indices */ }
                _ => {
                    stream.seek(sub_table_start + sub_header.length);
                }
            }
        }

        Err("Font.kern: No supported sub-table format available.")
    }

    fn read_format0(stream: &mut Stream) -> HashMap<u32, i16> {
        let mut mappings = HashMap::new();
        let pairs = stream.read_u16();
        stream.skip(6); // searchRange: u16, entrySelector: u16, rangeShift: u16
        for _ in 0..pairs {
            let left = stream.read_u16();
            let right = stream.read_u16();
            let id = u32::from(left) << 16 | u32::from(right);
            let offset = stream.read_i16();
            mappings.insert(id, offset);
        }
        mappings
    }

    fn read_header(stream: &mut Stream) -> Header {
        let version_major = 0x0000;
        let version_minor = 0x0000;
        let number_sub_tables = stream.read_u16() as u32;
        Header {
            version_major,
            version_minor,
            number_sub_tables,
        }
    }

    fn read_aat_header(stream: &mut Stream) -> Header {
        let version_major = 0x0001;
        let version_minor = stream.read_u16();
        let number_sub_tables = stream.read_u32();
        Header {
            version_major,
            version_minor,
            number_sub_tables,
        }
    }

    fn read_subtable(stream: &mut Stream) -> SubTableHeader {
        let version = stream.read_u16();
        let length = stream.read_u16() as usize;
        let coverage = Coverage(stream.read_u16());
        SubTableHeader {
            version,
            length,
            coverage,
            tuple_index: 0,
        }
    }

    fn read_aat_subtable(stream: &mut Stream) -> SubTableHeader {
        let length = stream.read_u32() as usize;
        let coverage = Coverage(stream.read_u16());
        let tuple_index = stream.read_u16();
        SubTableHeader {
            version: 0x0001,
            length,
            coverage,
            tuple_index,
        }
    }
}
