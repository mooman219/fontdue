use core::convert::TryInto;

pub struct StreamSliceU8<'a>(&'a [u8]);
pub struct StreamSliceU16<'a>(&'a [u8]);
pub struct StreamSliceU32<'a>(&'a [u8]);
impl<'a> StreamSliceU8<'a> {
    #[inline]
    pub fn get(&self, index: usize) -> Option<u8> {
        const SIZE: usize = 1;
        let offset = index * SIZE;
        let slice = self.0.get(offset..offset + SIZE)?;
        Some(slice[0])
    }
}
impl<'a> StreamSliceU16<'a> {
    #[inline]
    pub fn get(&self, index: usize) -> Option<u16> {
        const SIZE: usize = 2;
        let offset = index * SIZE;
        let slice = self.0.get(offset..offset + SIZE)?;
        Some(u16::from_be_bytes(slice.try_into().unwrap()))
    }
}
impl<'a> StreamSliceU32<'a> {
    #[inline]
    pub fn get(&self, index: usize) -> Option<u32> {
        const SIZE: usize = 4;
        let offset = index * SIZE;
        let slice = self.0.get(offset..offset + SIZE)?;
        Some(u32::from_be_bytes(slice.try_into().unwrap()))
    }
}

pub struct StreamSliceI8<'a>(StreamSliceU8<'a>);
pub struct StreamSliceI16<'a>(StreamSliceU16<'a>);
pub struct StreamSliceI32<'a>(StreamSliceU32<'a>);
impl<'a> StreamSliceI8<'a> {
    #[inline]
    pub fn get(&self, index: usize) -> Option<i8> {
        Some(unsafe { core::mem::transmute::<u8, i8>(self.0.get(index)?) })
    }
}
impl<'a> StreamSliceI16<'a> {
    #[inline]
    pub fn get(&self, index: usize) -> Option<i16> {
        Some(unsafe { core::mem::transmute::<u16, i16>(self.0.get(index)?) })
    }
}
impl<'a> StreamSliceI32<'a> {
    #[inline]
    pub fn get(&self, index: usize) -> Option<i32> {
        Some(unsafe { core::mem::transmute::<u32, i32>(self.0.get(index)?) })
    }
}

pub struct Stream<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Stream<'a> {
    pub const fn new(bytes: &'a [u8]) -> Stream<'a> {
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
    pub const fn offset(&self) -> usize {
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
        self.bytes.get(self.offset..end).map(|slice| {
            self.offset = end;
            StreamSliceU8(slice)
        })
    }

    #[inline]
    pub fn read_u16_slice(&mut self, len: usize) -> Option<StreamSliceU16<'a>> {
        let end = self.offset + len * 2;
        self.bytes.get(self.offset..end).map(|slice| {
            self.offset = end;
            StreamSliceU16(slice)
        })
    }

    #[inline]
    pub fn read_u32_slice(&mut self, len: usize) -> Option<StreamSliceU32<'a>> {
        let end = self.offset + len * 4;
        self.bytes.get(self.offset..end).map(|slice| {
            self.offset = end;
            StreamSliceU32(slice)
        })
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
        const SIZE: usize = 1;
        let slice = self.bytes.get(self.offset..self.offset + SIZE)?;
        self.offset += SIZE;
        Some(slice[0])
    }

    #[inline]
    pub fn read_u16(&mut self) -> Option<u16> {
        const SIZE: usize = 2;
        let slice = self.bytes.get(self.offset..self.offset + SIZE)?;
        self.offset += SIZE;
        Some(u16::from_be_bytes(slice.try_into().unwrap()))
    }

    #[inline]
    pub fn read_u32(&mut self) -> Option<u32> {
        const SIZE: usize = 4;
        let slice = self.bytes.get(self.offset..self.offset + SIZE)?;
        self.offset += SIZE;
        Some(u32::from_be_bytes(slice.try_into().unwrap()))
    }

    // SIGNED

    #[inline]
    pub fn read_i8(&mut self) -> Option<i8> {
        Some(unsafe { core::mem::transmute::<u8, i8>(self.read_u8()?) })
    }

    #[inline]
    pub fn read_i16(&mut self) -> Option<i16> {
        Some(unsafe { core::mem::transmute::<u16, i16>(self.read_u16()?) })
    }

    #[inline]
    pub fn read_i32(&mut self) -> Option<i32> {
        Some(unsafe { core::mem::transmute::<u32, i32>(self.read_u32()?) })
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
        let slice = self.bytes.get(self.offset..self.offset + SIZE)?;
        self.offset += SIZE;
        Some(slice.try_into().unwrap())
    }
}
