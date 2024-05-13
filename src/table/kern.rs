use crate::table::parse::*;
use crate::HashMap;

// Apple: https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6kern.html
// Microsoft: https://docs.microsoft.com/en-us/typography/opentype/spec/kern

#[derive(Debug)]
pub struct TableKern {
    pub horizontal_mappings: HashMap<u32, i16>,
}

#[derive(Copy, Clone, Debug)]
struct Header {
    pub version_major: u16,
    pub version_minor: u16,
    pub number_sub_tables: u32,
}

#[derive(Copy, Clone, Debug)]
struct SubTableHeader {
    pub version: u16,
    pub length: usize,
    pub coverage: Coverage,
    pub format: u8,
    pub tuple_index: u16,
}

#[derive(Copy, Clone, Debug)]
struct Coverage {
    is_horizontal: bool,
}

impl Coverage {
    pub const fn aat(cov: u8) -> Coverage {
        Coverage {
            is_horizontal: cov & 0x80 != 0x80,
        }
    }

    pub const fn ot(cov: u8) -> Coverage {
        Coverage {
            is_horizontal: cov & 0x01 == 0x01,
        }
    }
}

impl TableKern {
    pub fn new(kern: &[u8]) -> Option<TableKern> {
        let mut stream = Stream::new(kern);
        let version_major = stream.read_u16()?;

        let header;
        match version_major {
            0x0000 => header = Self::read_ot_header(&mut stream)?,
            0x0001 => header = Self::read_aat_header(&mut stream)?,
            _ => return None, // Font.kern: Unsupported kern table version.
        }

        for _ in 0..header.number_sub_tables {
            let sub_table_start = stream.offset();
            let sub_header = if version_major == 0x0000 {
                Self::read_ot_subtable(&mut stream)?
            } else {
                Self::read_aat_subtable(&mut stream)?
            };
            match sub_header.format {
                // Ordered List of Kerning Pairs
                0 => {
                    if sub_header.coverage.is_horizontal {
                        let mappings = Self::read_format0(&mut stream)?;
                        return Some(TableKern {
                            horizontal_mappings: mappings,
                        });
                    }
                }
                // State Table for Contextual Kerning
                // 1 => { /* TODO: State Table for Contextual Kerning */ }
                // Simple n x m Array of Kerning Values
                // 2 => { /* TODO: Simple n x m Array of Kerning Values */ }
                // Simple n x m Array of Kerning Indices
                3 => {
                    if sub_header.coverage.is_horizontal {
                        let mappings = Self::read_format3(&mut stream)?;
                        return Some(TableKern {
                            horizontal_mappings: mappings,
                        });
                    }
                }
                _ => {
                    stream.seek(sub_table_start + sub_header.length);
                }
            }
        }

        None // Font.kern: No supported sub-table format available.
    }

    fn read_format0(stream: &mut Stream) -> Option<HashMap<u32, i16>> {
        let pairs = stream.read_u16()?;
        stream.skip(6); // searchRange: u16, entrySelector: u16, rangeShift: u16
        let mut mappings = HashMap::new();
        for _ in 0..pairs {
            let left = stream.read_u16()?;
            let right = stream.read_u16()?;
            let id = u32::from(left) << 16 | u32::from(right);
            let value = stream.read_i16()?;
            mappings.insert(id, value);
        }
        Some(mappings)
    }

    fn read_format3(stream: &mut Stream) -> Option<HashMap<u32, i16>> {
        let glyph_count = stream.read_u16()?;
        let kerning_values_count = stream.read_u8()?;
        let left_hand_classes_count = stream.read_u8()?;
        let right_hand_classes_count = stream.read_u8()?;
        stream.skip(1); // flags - reserved
        let indices_count = u16::from(left_hand_classes_count) * u16::from(right_hand_classes_count);

        let kerning_values = stream.read_i16_slice(usize::from(kerning_values_count))?;
        let left_hand_classes = stream.read_u8_slice(usize::from(glyph_count))?;
        let right_hand_classes = stream.read_u8_slice(usize::from(glyph_count))?;
        let indices = stream.read_u8_slice(usize::from(indices_count))?;

        let mut mappings = HashMap::new();
        for left in 0..glyph_count {
            for right in 0..glyph_count {
                if let Some((id, value)) = {
                    let left_class = left_hand_classes.get(usize::from(left))?;
                    let right_class = right_hand_classes.get(usize::from(right))?;

                    if left_class > left_hand_classes_count || right_class > right_hand_classes_count {
                        continue;
                    }

                    let index =
                        u16::from(left_class) * u16::from(right_hand_classes_count) + u16::from(right_class);
                    let index = indices.get(usize::from(index))?;
                    let id = u32::from(left) << 16 | u32::from(right);
                    let value = kerning_values.get(usize::from(index))?;
                    Some((id, value))
                } {
                    mappings.insert(id, value);
                };
            }
        }

        Some(mappings)
    }

    fn read_ot_header(stream: &mut Stream) -> Option<Header> {
        let version_major = 0x0000;
        let version_minor = 0x0000;
        let number_sub_tables = stream.read_u16()? as u32;
        Some(Header {
            version_major,
            version_minor,
            number_sub_tables,
        })
    }

    fn read_aat_header(stream: &mut Stream) -> Option<Header> {
        let version_major = 0x0001;
        let version_minor = stream.read_u16()?;
        let number_sub_tables = stream.read_u32()?;
        Some(Header {
            version_major,
            version_minor,
            number_sub_tables,
        })
    }

    fn read_ot_subtable(stream: &mut Stream) -> Option<SubTableHeader> {
        let version = stream.read_u16()?;
        let length = stream.read_u16()? as usize;
        let format = stream.read_u8()?;
        let coverage = Coverage::ot(stream.read_u8()?);
        Some(SubTableHeader {
            version,
            length,
            coverage,
            format,
            tuple_index: 0,
        })
    }

    fn read_aat_subtable(stream: &mut Stream) -> Option<SubTableHeader> {
        let length = stream.read_u32()? as usize;
        let coverage = Coverage::aat(stream.read_u8()?);
        let format = stream.read_u8()?;
        let tuple_index = stream.read_u16()?;
        Some(SubTableHeader {
            version: 0x0001,
            length,
            coverage,
            format,
            tuple_index,
        })
    }
}
