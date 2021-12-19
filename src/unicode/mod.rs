mod tables;

use crate::unicode::tables::*;
use alloc::string::String;

const CONT_MASK: u8 = 0b0011_1111;

#[inline(always)]
fn utf8_acc_cont_byte(ch: u32, byte: u8) -> u32 {
    (ch << 6) | (byte & CONT_MASK) as u32
}

pub fn decode_utf16(bytes: &[u8]) -> String {
    let mut output = String::new();
    let mut offset = 0;
    while offset < bytes.len() {
        output.push(read_utf16(bytes, &mut offset));
    }
    output
}

pub fn read_utf16(bytes: &[u8], offset: &mut usize) -> char {
    let a = ((bytes[*offset] as u16) << 8) | bytes[*offset + 1] as u16;
    *offset += 2;
    if a < 0xD800 || 0xDFFF < a {
        unsafe { core::char::from_u32_unchecked(a as u32) }
    } else {
        let b = ((bytes[*offset] as u16) << 8) | bytes[*offset + 1] as u16;
        *offset += 2;
        let c = (((a - 0xD800) as u32) << 10 | (b - 0xDC00) as u32) + 0x1_0000;
        unsafe { core::char::from_u32_unchecked(c as u32) }
    }
}

/// Returns (length, character). Cannot be run at the end of the string.
pub fn read_utf8(bytes: &[u8], byte_offset: &mut usize) -> char {
    let x = bytes[*byte_offset];
    *byte_offset += 1;
    if x < 128 {
        return unsafe { core::char::from_u32_unchecked(x as u32) };
    }
    let init = (x & (0x7F >> 2)) as u32;
    let y = bytes[*byte_offset];
    *byte_offset += 1;
    let mut ch = utf8_acc_cont_byte(init, y);
    if x >= 0xE0 {
        let z = bytes[*byte_offset];
        *byte_offset += 1;
        let y_z = utf8_acc_cont_byte((y & CONT_MASK) as u32, z);
        ch = init << 12 | y_z;
        if x >= 0xF0 {
            let w = bytes[*byte_offset];
            *byte_offset += 1;
            ch = (init & 7) << 18 | utf8_acc_cont_byte(y_z, w);
        }
    }
    unsafe { core::char::from_u32_unchecked(ch) }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// Ordering is based on linebreak priority. Ordering is Hard > Soft > None.
pub struct LinebreakData {
    bits: u8,
}

pub const LINEBREAK_NONE: LinebreakData = LinebreakData::new(0b0000_0000);
pub const LINEBREAK_SOFT: LinebreakData = LinebreakData::new(0b0000_0001);
pub const LINEBREAK_HARD: LinebreakData = LinebreakData::new(0b0000_0010);

impl LinebreakData {
    const NONE: u8 = 0b0000_0000;
    const SOFT: u8 = 0b0000_0001;
    const HARD: u8 = 0b0000_0010;

    const fn new(bits: u8) -> LinebreakData {
        LinebreakData {
            bits,
        }
    }

    pub fn from_mask(wrap_soft_breaks: bool, wrap_hard_breaks: bool, has_width: bool) -> LinebreakData {
        let mut mask = 0;
        if wrap_hard_breaks {
            mask |= LinebreakData::HARD;
        }
        if wrap_soft_breaks && has_width {
            mask |= LinebreakData::SOFT;
        }
        LinebreakData {
            bits: mask,
        }
    }

    pub fn is_hard(&self) -> bool {
        self.bits == LinebreakData::HARD
    }

    pub fn is_soft(&self) -> bool {
        self.bits == LinebreakData::SOFT
    }

    pub fn mask(&self, other: LinebreakData) -> LinebreakData {
        Self::new(self.bits & other.bits)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Linebreaker {
    state: u8,
}

impl Linebreaker {
    pub fn new() -> Linebreaker {
        Linebreaker {
            state: 0,
        }
    }

    pub fn reset(&mut self) {
        self.state = 0;
    }

    // [See license/xi-editor/xi-unicode] Copyright 2016 The xi-editor Authors
    pub fn next(&mut self, codepoint: char) -> LinebreakData {
        let cp = codepoint as usize;
        let lb = if cp < 0x800 {
            LINEBREAK_1_2[cp]
        } else if cp < 0x10000 {
            let child = LINEBREAK_3_ROOT[cp >> 6];
            LINEBREAK_3_CHILD[(child as usize) * 0x40 + (cp & 0x3f)]
        } else {
            let mid = LINEBREAK_4_ROOT[cp >> 12];
            let leaf = LINEBREAK_4_MID[(mid as usize) * 0x40 + ((cp >> 6) & 0x3f)];
            LINEBREAK_4_LEAVES[(leaf as usize) * 0x40 + (cp & 0x3f)]
        };
        let i = (self.state as usize) * N_LINEBREAK_CATEGORIES + (lb as usize);
        let new = LINEBREAK_STATE_MACHINE[i];
        if (new as i8) < 0 {
            self.state = new & 0x3f;
            if new >= 0xc0 {
                LINEBREAK_HARD
            } else {
                LINEBREAK_SOFT
            }
        } else {
            self.state = new;
            LINEBREAK_NONE
        }
    }
}

/// Miscellaneous metadata associated with a character to assist in layout.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct CharacterData {
    bits: u8,
}

impl CharacterData {
    const WHITESPACE: u8 = 0b0000_0001;
    const CONTROL: u8 = 0b0000_0010;
    const MISSING: u8 = 0b0000_0100;

    /// Classifies a character given its index in the font.
    pub fn classify(c: char, index: u16) -> CharacterData {
        let mut class = 0;
        if index == 0 {
            class |= CharacterData::MISSING;
        }
        match c {
            '\t' | '\n' | '\x0C' | '\r' | ' ' => class |= CharacterData::WHITESPACE,
            _ => {}
        }
        match c {
            '\0'..='\x1F' | '\x7F' => class |= CharacterData::CONTROL,
            _ => {}
        }
        CharacterData {
            bits: class,
        }
    }

    /// A heuristic for if the glpyh this was classified from should be rasterized. Missing glyphs,
    /// whitespace, and control characters will return false.
    pub fn rasterize(&self) -> bool {
        self.bits == 0
    }

    /// Marks if the character is an ASCII whitespace character.
    pub fn is_whitespace(&self) -> bool {
        self.bits & CharacterData::WHITESPACE != 0
    }

    /// Marks if the character is an ASCII control character.
    pub fn is_control(&self) -> bool {
        self.bits & CharacterData::CONTROL != 0
    }

    /// Marks if the character is missing from its associated font.
    pub fn is_missing(&self) -> bool {
        self.bits & CharacterData::MISSING != 0
    }
}
