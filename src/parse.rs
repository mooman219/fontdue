// READ UNSIGNED

#[inline]
pub fn read_u8(buf: &[u8]) -> u8 {
    buf[0]
}

#[inline]
pub fn read_u16(buf: &[u8]) -> u16 {
    assert!(buf.len() >= 2);
    u16::from_be_bytes(unsafe { *(buf.as_ptr() as *const [u8; 2]) })
}

#[inline]
pub fn read_u32(buf: &[u8]) -> u32 {
    assert!(buf.len() >= 4);
    u32::from_be_bytes(unsafe { *(buf.as_ptr() as *const [u8; 4]) })
}

#[inline]
pub fn read_u64(buf: &[u8]) -> u64 {
    assert!(buf.len() >= 8);
    u64::from_be_bytes(unsafe { *(buf.as_ptr() as *const [u8; 8]) })
}

#[inline]
pub fn read_u128(buf: &[u8]) -> u128 {
    assert!(buf.len() >= 16);
    u128::from_be_bytes(unsafe { *(buf.as_ptr() as *const [u8; 16]) })
}

// READ SIGNED

#[inline]
pub fn read_i8(buf: &[u8]) -> i8 {
    read_u8(buf) as i8
}

#[inline]
pub fn read_i16(buf: &[u8]) -> i16 {
    read_u16(buf) as i16
}

#[inline]
pub fn read_i32(buf: &[u8]) -> i32 {
    read_u32(buf) as i32
}

#[inline]
pub fn read_i64(buf: &[u8]) -> i64 {
    read_u64(buf) as i64
}

#[inline]
pub fn read_i128(buf: &[u8]) -> i128 {
    read_u128(buf) as i128
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

// FONT

#[inline]
pub fn read_f2dot14(buf: &[u8]) -> f32 {
    let val = read_i16(buf);
    val as f32 * (1.0 / (1 << 14) as f32)
}
