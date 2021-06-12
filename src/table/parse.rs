use core::convert::TryInto;

pub struct StreamSliceU8<'a>(&'a [u8]);
pub struct StreamSliceU16<'a>(&'a [u8]);
pub struct StreamSliceU32<'a>(&'a [u8]);
impl<'a> StreamSliceU8<'a> {
    #[inline]
    pub fn get(&self, index: usize) -> Option<u8> {
        let offset = index;
        self.0
            .get(offset..offset) // Option<&[u8]>
            .map(|slice| u8::from_be_bytes(slice.try_into().unwrap()))
    }
}
impl<'a> StreamSliceU16<'a> {
    #[inline]
    pub fn get(&self, index: usize) -> Option<u16> {
        const SIZE: usize = 2;
        let offset = index * SIZE;
        self.0
            .get(offset..offset + SIZE) // Option<&[u8]>
            .map(|slice| u16::from_be_bytes(slice.try_into().unwrap()))
    }
}
impl<'a> StreamSliceU32<'a> {
    #[inline]
    pub fn get(&self, index: usize) -> Option<u32> {
        const SIZE: usize = 4;
        let offset = index * SIZE;
        self.0
            .get(offset..offset + SIZE) // Option<&[u8]>
            .map(|slice| u32::from_be_bytes(slice.try_into().unwrap()))
    }
}

pub struct StreamSliceI8<'a>(StreamSliceU8<'a>);
pub struct StreamSliceI16<'a>(StreamSliceU16<'a>);
pub struct StreamSliceI32<'a>(StreamSliceU32<'a>);
impl<'a> StreamSliceI8<'a> {
    #[inline]
    pub fn get(&self, index: usize) -> Option<i8> {
        Some(unsafe { core::mem::transmute(self.0.get(index)?) })
    }
}
impl<'a> StreamSliceI16<'a> {
    #[inline]
    pub fn get(&self, index: usize) -> Option<i16> {
        Some(unsafe { core::mem::transmute(self.0.get(index)?) })
    }
}
impl<'a> StreamSliceI32<'a> {
    #[inline]
    pub fn get(&self, index: usize) -> Option<i32> {
        Some(unsafe { core::mem::transmute(self.0.get(index)?) })
    }
}

pub struct Stream<'a> {
    bytes: &'a [u8],
    offset: usize,
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

    // UNSIGNED SLICE

    #[inline]
    pub fn read_u8_slice(&mut self, len: usize) -> Option<StreamSliceU8<'a>> {
        let end = self.offset + len;
        let result = self.bytes.get(self.offset..end)?;
        self.offset = end;
        Some(StreamSliceU8(result))
    }

    #[inline]
    pub fn read_u16_slice(&mut self, len: usize) -> Option<StreamSliceU16<'a>> {
        let end = self.offset + len * 2;
        let result = self.bytes.get(self.offset..end)?;
        self.offset = end;
        Some(StreamSliceU16(result))
    }

    #[inline]
    pub fn read_u32_slice(&mut self, len: usize) -> Option<StreamSliceU32<'a>> {
        let end = self.offset + len * 4;
        let result = self.bytes.get(self.offset..end)?;
        self.offset = end;
        Some(StreamSliceU32(result))
    }

    // SIGNED SLICE

    #[inline]
    pub fn read_i8_slice(&mut self, len: usize) -> Option<StreamSliceI8<'a>> {
        Some(StreamSliceI8(self.read_u8_slice(len)?))
    }

    #[inline]
    pub fn read_i16_slice(&mut self, len: usize) -> Option<StreamSliceI16<'a>> {
        Some(StreamSliceI16(self.read_u16_slice(len)?))
    }

    #[inline]
    pub fn read_i32_slice(&mut self, len: usize) -> Option<StreamSliceI32<'a>> {
        Some(StreamSliceI32(self.read_u32_slice(len)?))
    }

    // UNSIGNED

    #[inline]
    pub fn read_u8(&mut self) -> Option<u8> {
        self.bytes
            .get(self.offset..self.offset + 1) // Option<&[u8]>
            .map(|slice| u8::from_be_bytes(slice.try_into().unwrap()))
    }

    #[inline]
    pub fn read_u16(&mut self) -> Option<u16> {
        self.bytes
            .get(self.offset..self.offset + 2) // Option<&[u8]>
            .map(|slice| u16::from_be_bytes(slice.try_into().unwrap()))
    }

    #[inline]
    pub fn read_u32(&mut self) -> Option<u32> {
        self.bytes
            .get(self.offset..self.offset + 4) // Option<&[u8]>
            .map(|slice| u32::from_be_bytes(slice.try_into().unwrap()))
    }

    // SIGNED

    #[inline]
    pub fn read_i8(&mut self) -> Option<i8> {
        Some(unsafe { core::mem::transmute(self.read_u8()?) })
    }

    #[inline]
    pub fn read_i16(&mut self) -> Option<i16> {
        Some(unsafe { core::mem::transmute(self.read_u16()?) })
    }

    #[inline]
    pub fn read_i32(&mut self) -> Option<i32> {
        Some(unsafe { core::mem::transmute(self.read_u32()?) })
    }

    // FONT

    #[inline]
    pub fn read_f2dot14(&mut self) -> Option<f32> {
        let val = self.read_i16()?;
        let result = val as f32 * (1.0 / (1 << 14) as f32);
        Some(result)
    }

    #[inline]
    pub fn read_tag(&mut self) -> Option<[u8; 4]> {
        const SIZE: usize = 4;
        if self.offset + SIZE > self.bytes.len() {
            return None;
        }
        let slice = &self.bytes[self.offset..];
        self.offset += SIZE;
        Some(unsafe { *(slice.as_ptr() as *const [u8; SIZE]) })
    }
}
