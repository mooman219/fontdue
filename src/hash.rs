// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Fx Hash
//!
//! This hashing algorithm was extracted from the Rustc compiler.  This is the same hashing
//! algorithm used for some internal operations in Firefox.  The strength of this algorithm
//! is in hashing 8 bytes at a time on 64-bit platforms, where the FNV algorithm works on one
//! byte at a time.
//!
//! ## Disclaimer
//!
//! It is **not a cryptographically secure** hash, so it is strongly recommended that you do
//! not use this hash for cryptographic purproses.  Furthermore, this hashing algorithm was
//! not designed to prevent any attacks for determining collisions which could be used to
//! potentially cause quadratic behavior in `HashMap`s.  So it is not recommended to expose
//! this hash in places where collissions or DDOS attacks may be a concern.

use core::convert::TryInto;
use core::ops::BitXor;

const ROTATE: u32 = 5;
const SEED64: u64 = 0x517cc1b727220a95;
const SEED32: u32 = (SEED64 & 0xFFFF_FFFF) as u32;

#[cfg(target_pointer_width = "32")]
const SEED: usize = SEED32 as usize;
#[cfg(target_pointer_width = "64")]
const SEED: usize = SEED64 as usize;

trait HashWord {
    fn hash_word(&mut self, word: Self);
}

impl HashWord for usize {
    #[inline]
    fn hash_word(&mut self, word: Self) {
        *self = self.rotate_left(ROTATE).bitxor(word).wrapping_mul(SEED);
    }
}

impl HashWord for u32 {
    #[inline]
    fn hash_word(&mut self, word: Self) {
        *self = self.rotate_left(ROTATE).bitxor(word).wrapping_mul(SEED32);
    }
}

impl HashWord for u64 {
    #[inline]
    fn hash_word(&mut self, word: Self) {
        *self = self.rotate_left(ROTATE).bitxor(word).wrapping_mul(SEED64);
    }
}

#[cfg(target_endian = "little")]
fn read_u32(buf: &[u8]) -> u32 {
    u32::from_le_bytes(buf[..4].try_into().unwrap())
}

#[cfg(target_endian = "little")]
fn read_u64(buf: &[u8]) -> u64 {
    u64::from_le_bytes(buf[..8].try_into().unwrap())
}

#[cfg(target_endian = "big")]
fn read_u32(buf: &[u8]) -> u32 {
    u32::from_be_bytes(buf[..4].try_into().unwrap())
}

#[cfg(target_endian = "big")]
fn read_u64(buf: &[u8]) -> u64 {
    u64::from_be_bytes(buf[..8].try_into().unwrap())
}

#[inline]
#[cfg(target_pointer_width = "32")]
fn write(initial_state: usize, mut bytes: &[u8]) -> usize {
    let mut hash = initial_state as u32;
    while bytes.len() >= 4 {
        let n = read_u32(bytes);
        hash.hash_word(n);
        bytes = bytes.split_at(4).1;
    }

    for byte in bytes {
        hash.hash_word(*byte as u32);
    }
    hash as usize
}

#[inline]
#[cfg(target_pointer_width = "64")]
fn write(initial_state: usize, mut bytes: &[u8]) -> usize {
    let mut hash = initial_state as u64;
    while bytes.len() >= 8 {
        let n = read_u64(bytes);
        hash.hash_word(n);
        bytes = bytes.split_at(8).1;
    }

    if bytes.len() >= 4 {
        let n = read_u32(bytes);
        hash.hash_word(n as u64);
        bytes = bytes.split_at(4).1;
    }

    for byte in bytes {
        hash.hash_word(*byte as u64);
    }
    hash as usize
}

pub fn hash(bytes: &[u8]) -> usize {
    write(0usize, bytes)
}
