use alloc::vec::*;

pub struct Stream<'a> {
    pub bytes: &'a [u8],
    pub offset: usize,
}

impl<'a> Stream<'a> {
    pub fn new(bytes: &'a [u8]) -> Stream<'a> {
        Stream {
            bytes,
            offset: 0,
        }
    }

    // UTILITY

    #[inline]
    pub fn reset(&mut self) {
        self.offset = 0;
    }

    #[inline]
    pub fn offset(&self) -> usize {
        self.offset
    }

    #[inline]
    pub fn seek(&mut self, offset: usize) {
        self.offset = offset;
    }

    #[inline]
    pub fn skip(&mut self, offset: usize) {
        self.offset += offset;
    }

    // UNSIGNED

    #[inline]
    pub fn read_u8(&mut self) -> u8 {
        const SIZE: usize = 1;
        let result = self.bytes[self.offset];
        self.offset += SIZE;
        result
    }

    #[inline]
    pub fn read_u16(&mut self) -> u16 {
        const SIZE: usize = 2;
        let slice = &self.bytes[self.offset..];
        assert!(slice.len() >= SIZE);
        let result = u16::from_be_bytes(unsafe { *(slice.as_ptr() as *const [u8; SIZE]) });
        self.offset += SIZE;
        result
    }

    #[inline]
    pub fn read_u32(&mut self) -> u32 {
        const SIZE: usize = 4;
        let slice = &self.bytes[self.offset..];
        assert!(slice.len() >= SIZE);
        let result = u32::from_be_bytes(unsafe { *(slice.as_ptr() as *const [u8; SIZE]) });
        self.offset += SIZE;
        result
    }

    #[inline]
    pub fn read_u64(&mut self) -> u64 {
        const SIZE: usize = 8;
        let slice = &self.bytes[self.offset..];
        assert!(slice.len() >= SIZE);
        let result = u64::from_be_bytes(unsafe { *(slice.as_ptr() as *const [u8; SIZE]) });
        self.offset += SIZE;
        result
    }

    // UNSIGNED BATCH

    #[inline]
    pub fn read_array_u16(&mut self, count: usize) -> Vec<u16> {
        let mut values = Vec::with_capacity(count);
        for _ in 0..count {
            values.push(self.read_u16());
        }
        values
    }

    // SIGNED

    #[inline]
    pub fn read_i8(&mut self) -> i8 {
        const SIZE: usize = 1;
        let result = self.bytes[self.offset] as i8;
        self.offset += SIZE;
        result
    }

    #[inline]
    pub fn read_i16(&mut self) -> i16 {
        const SIZE: usize = 2;
        let slice = &self.bytes[self.offset..];
        assert!(slice.len() >= SIZE);
        let result = i16::from_be_bytes(unsafe { *(slice.as_ptr() as *const [u8; SIZE]) });
        self.offset += SIZE;
        result
    }

    #[inline]
    pub fn read_i32(&mut self) -> i32 {
        const SIZE: usize = 4;
        let slice = &self.bytes[self.offset..];
        assert!(slice.len() >= SIZE);
        let result = i32::from_be_bytes(unsafe { *(slice.as_ptr() as *const [u8; SIZE]) });
        self.offset += SIZE;
        result
    }

    #[inline]
    pub fn read_i64(&mut self) -> i64 {
        const SIZE: usize = 8;
        let slice = &self.bytes[self.offset..];
        assert!(slice.len() >= SIZE);
        let result = i64::from_be_bytes(unsafe { *(slice.as_ptr() as *const [u8; SIZE]) });
        self.offset += SIZE;
        result
    }

    // FONT

    #[inline]
    pub fn read_f2dot14(&mut self) -> f32 {
        let val = self.read_i16();
        let result = val as f32 * (1.0 / (1 << 14) as f32);
        result
    }

    #[inline]
    pub fn read_tag(&mut self) -> [u8; 4] {
        const SIZE: usize = 4;
        let slice = &self.bytes[self.offset..];
        assert!(slice.len() >= SIZE);
        let result = unsafe { *(slice.as_ptr() as *const [u8; SIZE]) };
        self.offset += SIZE;
        result
    }
}

// FLAG UNSIGNED

#[inline]
pub fn flag_u8(value: u8, flags: u8) -> bool {
    value & flags == flags
}

#[inline]
pub fn flag_u16(value: u16, flags: u16) -> bool {
    value & flags == flags
}

#[inline]
pub fn flag_u32(value: u32, flags: u32) -> bool {
    value & flags == flags
}

#[inline]
pub fn flag_u64(value: u64, flags: u64) -> bool {
    value & flags == flags
}

#[inline]
pub fn flag_u128(value: u128, flags: u128) -> bool {
    value & flags == flags
}

// FLAG SIGNED

#[inline]
pub fn flag_i8(value: i8, flags: i8) -> bool {
    value & flags == flags
}

#[inline]
pub fn flag_i16(value: i16, flags: i16) -> bool {
    value & flags == flags
}

#[inline]
pub fn flag_i32(value: i32, flags: i32) -> bool {
    value & flags == flags
}

#[inline]
pub fn flag_i64(value: i64, flags: i64) -> bool {
    value & flags == flags
}

#[inline]
pub fn flag_i128(value: i128, flags: i128) -> bool {
    value & flags == flags
}
