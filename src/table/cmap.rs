use crate::parse::*;
use crate::FontResult;
use core::num::NonZeroU32;
use hashbrown::HashMap;

// Apple: https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6cmap.html
// Microsoft: https://docs.microsoft.com/en-us/typography/opentype/spec/cmap

/// Formats 4 and 12 are the most popular. Some fonts can include others for compatibility.
fn is_ideal_format(f: u16) -> bool {
    f == 4 || f == 12
}

/// Check for if we support reading the format.
fn is_supported_format(f: u16) -> bool {
    f == 0 || f == 4 || f == 6 || f == 10 || f == 12 || f == 13
}

#[derive(Debug)]
pub struct TableCmap {
    pub map: HashMap<u32, NonZeroU32>,
}

/// Wraps the unsafe creation of NonZeroU32::new_unchecked. For us, a zero value actually
/// represents Option::None, so this is desired.
fn insert(map: &mut HashMap<u32, NonZeroU32>, k: u32, v: u32) {
    map.insert(k, unsafe { NonZeroU32::new_unchecked(v) });
}

impl TableCmap {
    pub fn new(cmap: &[u8]) -> FontResult<TableCmap> {
        let mut stream = Stream::new(cmap);
        stream.skip(2); // version: u16
        let number_sub_tables = stream.read_u16();
        let mut mapping_offset = 0;
        for i in 0..number_sub_tables as usize {
            // The cmap index is 4 bytes. The encoding subtable is 8 bytes in size.
            stream.seek(i * 8 + 4);
            let platform_id = stream.read_u16();
            let specific_id = stream.read_u16();
            let mapping_offset_temp = stream.read_u32() as usize;
            // All mappings should have the format as the first field.
            stream.seek(mapping_offset_temp);
            let format = stream.read_u16();
            if !is_supported_format(format) {
                continue;
            }
            // We're only supporting unicode for obvious reasons.
            match platform_id {
                // Unicode
                0 => {
                    mapping_offset = mapping_offset_temp;
                    if is_ideal_format(format) {
                        break;
                    }
                }
                // Microsoft
                3 => {
                    match specific_id {
                        //  1 UnicodeBmp
                        // 10 UnicodeFull
                        1 | 10 => {
                            mapping_offset = mapping_offset_temp;
                            if is_ideal_format(format) {
                                break;
                            }
                        }
                        // 0 Symbol
                        // 2 ShiftJis
                        // 3 PRC
                        // 4 BigFive
                        // 5 Johab
                        // _ Unknown
                        0 | 2 | 3 | 4 | 5 | _ => {}
                    }
                }
                // 1 Mac
                // 2 Iso - Deprecated
                // 4 Custom
                // _ Unknown
                1 | 2 | 4 | _ => {}
            }
        }
        if mapping_offset == 0 {
            return Err("Font.cmap: Unable to find usable cmap table");
        }
        stream.seek(mapping_offset);
        let format = stream.read_u16();
        let mut mappings = HashMap::new();
        match format {
            // Byte encoding table
            0 => {
                let length = stream.read_u16() as u32;
                stream.skip(2); // language: u16
                for unicode_codepoint in 0..(length - 6) {
                    let pair = stream.read_u8() as u32;
                    insert(&mut mappings, unicode_codepoint, pair);
                }
            }
            // High byte mapping through table
            // 2 => { /* TODO: high-byte mapping for japanese/chinese/korean */ }
            // Segment mapping to delta values
            4 => {
                stream.skip(4); // length: u16, language: u16
                let seg_count = stream.read_u16() as usize >> 1;
                stream.skip(6); // searchRange: u16, entrySelector: u16, rangeShift: u16
                let end_code_array = stream.read_array_u16(seg_count);
                stream.skip(2); // reservedPad: u16
                let start_code_array = stream.read_array_u16(seg_count);
                let id_delta_array = stream.read_array_u16(seg_count);
                let id_range_offset_array = stream.read_array_u16(seg_count);
                for i in 0..(seg_count - 1) {
                    let end_code = end_code_array[i];
                    let start_code = start_code_array[i];
                    let id_delta = id_delta_array[i];
                    let id_range_offset = id_range_offset_array[i];
                    for c in start_code..=end_code {
                        let glyph_index = if id_range_offset != 0 {
                            // To quote chromium "this might seem odd, but it's true. The offset
                            // is relative to the location of the offset value itself."

                            // Offset of the start of the id_range_offset_array.
                            let mut glyph_index_offset = 16 + seg_count * 6;
                            // Offset into where we are in the id_range_offset_array.
                            glyph_index_offset += i * 2;
                            // Add the value of the idRangeOffset, which will move us into the
                            // glyphIndex array.
                            glyph_index_offset += id_range_offset as usize;
                            // Then add the character index of the current segment, multiplied by
                            // 2 for u16.
                            glyph_index_offset += (c - start_code) as usize * 2;
                            stream.seek(mapping_offset + glyph_index_offset);
                            let glyph_index = stream.read_u16();
                            if glyph_index != 0 {
                                glyph_index.wrapping_add(id_delta)
                            } else {
                                glyph_index
                            }
                        } else {
                            c.wrapping_add(id_delta)
                        };
                        insert(&mut mappings, c as u32, glyph_index as u32);
                    }
                }
            }
            // Trimmed table mapping
            6 => {
                stream.skip(4); // length: u16, language: u16
                let first = stream.read_u16() as u32;
                let count = stream.read_u16() as u32;
                for unicode_codepoint in first..(first + count) {
                    let pair = stream.read_u16() as u32;
                    insert(&mut mappings, unicode_codepoint, pair);
                }
            }
            // Mixed coverage
            // 8 => { /* TODO: 8 - Mixed 16-bit and 32-bit coverage */ }
            // Trimmed array
            10 => {
                stream.skip(10); // reserved: u16, length: u32, language: u32
                let start_char_code = stream.read_u32();
                let num_chars = stream.read_u32();
                for unicode_codepoint in start_char_code..(start_char_code + num_chars) {
                    let pair = stream.read_u16() as u32;
                    insert(&mut mappings, unicode_codepoint, pair);
                }
            }
            // Segmented coverage
            12 => {
                stream.skip(10); // reserved: u16, length: u32, language: u32
                let num_groups = stream.read_u32() as usize;
                for _ in 0..num_groups {
                    let start_char_code = stream.read_u32();
                    let end_char_code = stream.read_u32();
                    let mut start_glyph_id = stream.read_u32();
                    for char_code in start_char_code..=end_char_code {
                        insert(&mut mappings, char_code, start_glyph_id);
                        start_glyph_id += 1;
                    }
                }
            }
            // Many-to-one range mappings
            13 => {
                stream.skip(10); // reserved: u16, length: u32, language: u32
                let num_groups = stream.read_u32() as usize;
                for _ in 0..num_groups {
                    let start_char_code = stream.read_u32();
                    let end_char_code = stream.read_u32();
                    let glyph_id = stream.read_u32();
                    for char_code in start_char_code..=end_char_code {
                        insert(&mut mappings, char_code, glyph_id);
                    }
                }
            }
            // Unicode variation sequences
            // 14 => { /* TODO: 14 - Unicode Variation Sequences */ }
            _ => {
                return Err("Font.cmap: Index map format unsupported");
            }
        }
        Ok(TableCmap {
            map: mappings,
        })
    }
}
