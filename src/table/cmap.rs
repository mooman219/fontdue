use crate::parse::*;
use crate::FontResult;
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

pub struct TableCmap {
    pub map: HashMap<u32, u32>,
}

impl TableCmap {
    pub fn new(cmap: &[u8]) -> FontResult<TableCmap> {
        let number_sub_tables = read_u16(&cmap[2..]);
        let mut mapping_offset = 0;
        for i in 0..number_sub_tables as usize {
            // The cmap index is 4 bytes. The encoding subtable is 8 bytes in size.
            let encoding_offset = i * 8 + 4;
            let platform_id = read_u16(&cmap[encoding_offset..]);
            let specific_id = read_u16(&cmap[encoding_offset + 2..]);
            let mapping_offset_temp = read_u32(&cmap[encoding_offset + 4..]) as usize;
            // All mappings should have the format as the first field.
            let format = read_u16(&cmap[mapping_offset_temp..]);
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
        let map = &cmap[mapping_offset..];
        let format = read_u16(&map[..]);
        let mut mappings = HashMap::new();
        match format {
            // Byte encoding table
            0 => {
                let bytes = read_u16(&map[2..]);
                for unicode_codepoint in 0..(bytes as u32 - 6) {
                    let pair = map[6 + unicode_codepoint as usize] as u32;
                    mappings.insert(unicode_codepoint, pair);
                }
            }
            // High byte mapping through table
            // 2 => { /* TODO: high-byte mapping for japanese/chinese/korean */ }
            // Segment mapping to delta values
            4 => {
                let seg_count = read_u16(&map[6..]) as usize >> 1;
                let end_code_array = &map[14..];
                let start_code_array = &map[16 + seg_count * 2..];
                let id_delta_array = &map[16 + seg_count * 4..];
                let id_range_offset_array = &map[16 + seg_count * 6..];
                for i in 0..(seg_count - 1) {
                    let end_code = read_u16(&end_code_array[(i * 2)..]);
                    let start_code = read_u16(&start_code_array[(i * 2)..]);
                    let id_delta = read_u16(&id_delta_array[(i * 2)..]);
                    let id_range_offset = read_u16(&id_range_offset_array[(i * 2)..]);
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
                            // 2 for USHORTs.
                            glyph_index_offset += (c - start_code) as usize * 2;
                            let glyph_index = read_u16(&map[glyph_index_offset..]);
                            if glyph_index != 0 {
                                glyph_index.wrapping_add(id_delta)
                            } else {
                                glyph_index
                            }
                        } else {
                            c.wrapping_add(id_delta)
                        };
                        mappings.insert(c as u32, glyph_index as u32);
                    }
                }
            }
            // Trimmed table mapping
            6 => {
                let first = read_u16(&map[6..]) as u32;
                let count = read_u16(&map[8..]) as u32;
                for unicode_codepoint in first..(first + count) {
                    let pair = read_u16(&map[10 + (unicode_codepoint - first) as usize * 2..]) as u32;
                    mappings.insert(unicode_codepoint, pair);
                }
            }
            // Mixed coverage
            // 8 => { /* TODO: 8 - Mixed 16-bit and 32-bit coverage */ }
            // Trimmed array
            10 => {
                let start_char_code = read_u32(&map[12..]);
                let num_chars = read_u32(&map[16..]);
                for unicode_codepoint in start_char_code..(start_char_code + num_chars) {
                    let pair =
                        read_u16(&map[20 + (unicode_codepoint - start_char_code) as usize * 4..]) as u32;
                    mappings.insert(unicode_codepoint, pair);
                }
            }
            // Segmented coverage | Many-to-one range mappings
            12 | 13 => {
                let groups = &map[16..];
                let num_groups = read_u32(&map[12..]) as usize;
                for group in 0..num_groups {
                    let offset = group * 12;
                    let start_char_code = read_u32(&groups[offset..]);
                    let end_char_code = read_u32(&groups[offset + 4..]);
                    let mut start_glyph_id = read_u32(&groups[offset + 8..]);
                    for char_code in start_char_code..=end_char_code {
                        mappings.insert(char_code, start_glyph_id);
                        start_glyph_id += 1;
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
