// Rust Bitcoin Library
// Written in 2014 by
//     Andrew Poelstra <apoelstra@wpsoftware.net>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

//! Big unsigned integer types
//!
//! Implementation of a various large-but-fixed sized unsigned integer types.
//! The functions here are designed to be fast.
//!
//!
//!Edits to the original Implementation -> We don't use the u128 type at all so I see no need for a
//!macro. I'd rather have the code easier to understand for a newcomer, then have a macro that is
//!only used for one thing.

// https://github.com/UrkelLabs/extended-primitives/blob/master/src/uint256.rs

use crate::hex::{
    from_hex,
    to_hex
};
use std::fmt;

// When Std::iter::Step is finished being implemented add it to this type. That would allow us to
// use for loops much more easily. Right now it's on nightly only -> https://github.com/rust-lang/rust/issues/42168

/// A trait which allows numbers to act as fixed-size bit arrays
pub trait BitArray {
    /// Is bit set?
    fn bit(&self, idx: usize) -> bool;

    /// Returns an array which is just the bits from start to end
    fn bit_slice(&self, start: usize, end: usize) -> Self;

    /// Bitwise and with `n` ones
    fn mask(&self, n: usize) -> Self;

    /// Trailing zeros
    fn trailing_zeros(&self) -> usize;

    /// Create all-zeros value
    fn zero() -> Self;

    /// Create value representing one
    fn one() -> Self;
}

// use consensus::encode;
// use util::BitArray;
//TODO what is repr actually used for?
//https://doc.rust-lang.org/nomicon/other-reprs.html -> I'm not sure we are going to be transfering
//this through FFI, so this could probably be removed, but I'll leave this for review.
#[repr(C)]
pub struct U256(pub [u64; 4]);

//thing = U256
//ty = u64
//expr = 4

//DO we need these lifetimes here? TODO
impl<'a> From<&'a [u64]> for U256 {
    fn from(data: &'a [u64]) -> U256 {
        assert_eq!(data.len(), 4);
        let mut ret = [0; 4];
        ret.copy_from_slice(&data[..]);
        U256(ret)
    }
}

impl From<u32> for U256 {
    fn from(value: u32) -> U256 {
        let mut ret = [0; 4];
        ret[0] = value as u64;
        U256(ret)
    }
}

impl From<[u8; 32]> for U256 {
    fn from(value: [u8; 32]) -> U256 {
        let mut ret = [0; 4];
        //TODO this might have to be reversed
        for i in 0..4 {
            let start = 0 + i * 8;
            let end = 8 + i * 8;
            let mut bytes = [0; 8];
            bytes.copy_from_slice(&value[start..end]);
            ret[i] = u64::from_be_bytes(bytes);
        }
        U256(ret)
    }
}


// impl FromHex for U256 {
//     type Error = FromHexError;

//     fn from_hex<T: AsRef<[u8]>>(hex: T) -> std::result::Result<Self, Self::Error> {
//         let bytes = from_hex(hex)?;
//         if bytes.len() != 32 {
//             Err(FromHexError::InvalidStringLength)
//         } else {
//             let mut ret = [0; 32];
//             ret.copy_from_slice(&bytes);
//             Ok(U256::from(ret))
//         }
//     }
// }

impl ::std::ops::Index<usize> for U256 {
    type Output = u64;

    #[inline]
    fn index(&self, index: usize) -> &u64 {
        let &U256(ref dat) = self;
        &dat[index]
    }
}

impl ::std::ops::Index<::std::ops::Range<usize>> for U256 {
    type Output = [u64];

    #[inline]
    fn index(&self, index: ::std::ops::Range<usize>) -> &[u64] {
        &self.0[index]
    }
}

impl ::std::ops::Index<::std::ops::RangeTo<usize>> for U256 {
    type Output = [u64];

    #[inline]
    fn index(&self, index: ::std::ops::RangeTo<usize>) -> &[u64] {
        &self.0[index]
    }
}

impl ::std::ops::Index<::std::ops::RangeFrom<usize>> for U256 {
    type Output = [u64];

    #[inline]
    fn index(&self, index: ::std::ops::RangeFrom<usize>) -> &[u64] {
        &self.0[index]
    }
}

impl ::std::ops::Index<::std::ops::RangeFull> for U256 {
    type Output = [u64];

    #[inline]
    fn index(&self, _: ::std::ops::RangeFull) -> &[u64] {
        &self.0[..]
    }
}

impl PartialEq for U256 {
    #[inline]
    fn eq(&self, other: &U256) -> bool {
        &self[..] == &other[..]
    }
}

impl Eq for U256 {}

impl PartialOrd for U256 {
    #[inline]
    fn partial_cmp(&self, other: &U256) -> Option<::std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for U256 {
    #[inline]
    fn cmp(&self, other: &U256) -> ::std::cmp::Ordering {
        // manually implement comparison to get little-endian ordering
        // (we need this for our numeric types; non-numeric ones shouldn't
        // be ordered anyway except to put them in BTrees or whatever, and
        // they don't care how we order as long as we're consistent).
        for i in 0..4 {
            if self[4 - 1 - i] < other[4 - 1 - i] {
                return ::std::cmp::Ordering::Less;
            }
            if self[4 - 1 - i] > other[4 - 1 - i] {
                return ::std::cmp::Ordering::Greater;
            }
        }
        ::std::cmp::Ordering::Equal
    }
}

#[cfg_attr(feature = "clippy", allow(expl_impl_clone_on_copy))] // we don't define the `struct`, we have to explicitly impl
impl Clone for U256 {
    #[inline]
    fn clone(&self) -> U256 {
        U256::from(&self[..])
    }
}

impl Copy for U256 {}

impl U256 {
    #[inline]
    /// Converts the object to a raw pointer
    pub fn as_ptr(&self) -> *const u64 {
        let &U256(ref dat) = self;
        dat.as_ptr()
    }

    #[inline]
    /// Converts the object to a mutable raw pointer
    pub fn as_mut_ptr(&mut self) -> *mut u64 {
        let &mut U256(ref mut dat) = self;
        dat.as_mut_ptr()
    }

    #[inline]
    /// Returns the length of the object as an array
    pub fn len(&self) -> usize {
        4
    }

    #[inline]
    /// Returns whether the object, as an array, is empty. Always false.
    pub fn is_empty(&self) -> bool {
        false
    }

    #[inline]
    pub fn to_hex(&self) -> String {
        to_hex(&self.to_le_bytes())
    }

    #[inline]
    /// Returns the underlying bytes.
    pub fn as_bytes(&self) -> &[u64; 4] {
        &self.0
    }

    #[inline]
    //Remove old Bytes functions TODO move them to returning u8
    //XXX Giant todo, do not forget these, or we will have unstandarized function returns.
    /// Returns the underlying bytes.
    pub fn to_bytes(&self) -> [u64; 4] {
        self.0.clone()
    }

    #[inline]
    //Returns little endian bytes
    pub fn to_le_bytes(&self) -> [u8; 32] {
        let mut bytes = [0; 32];
        for i in 0..4 {
            //Ugly rewrite this code to be more efficient...
            let le_bytes = self.0[i].to_le_bytes();

            bytes[8 * i] = le_bytes[0];
            bytes[1 + 8 * i] = le_bytes[1];
            bytes[2 + 8 * i] = le_bytes[2];
            bytes[3 + 8 * i] = le_bytes[3];

            //Second half
            bytes[4 + 8 * i] = le_bytes[4];
            bytes[5 + 8 * i] = le_bytes[5];
            bytes[6 + 8 * i] = le_bytes[6];
            bytes[7 + 8 * i] = le_bytes[7];
        }

        bytes
    }

    /// The maximum value which can be inhabited by this type.
    #[inline]
    pub fn max_value() -> Self {
        let mut result = [0; 4];
        for i in 0..4 {
            result[i] = u64::max_value();
        }
        U256(result)
    }

    #[inline]
    /// Returns the underlying bytes.
    pub fn into_bytes(self) -> [u64; 4] {
        self.0
    }

    /// Conversion to u32
    #[inline]
    pub fn low_u32(&self) -> u32 {
        let &U256(ref arr) = self;
        arr[0] as u32
    }

    /// Conversion to u64
    #[inline]
    pub fn low_u64(&self) -> u64 {
        let &U256(ref arr) = self;
        arr[0] as u64
    }

    /// Return the least number of bits needed to represent the number
    #[inline]
    pub fn bits(&self) -> usize {
        let &U256(ref arr) = self;
        for i in 1..4 {
            if arr[4 - i] > 0 {
                return (0x40 * (4 - i + 1)) - arr[4 - i].leading_zeros() as usize;
            }
        }
        0x40 - arr[0].leading_zeros() as usize
    }

    /// Multiplication by u32
    pub fn mul_u32(self, other: u32) -> U256 {
        let U256(ref arr) = self;
        let mut carry = [0u64; 4];
        let mut ret = [0u64; 4];
        for i in 0..4 {
            let not_last_word = i < 4 - 1;
            let upper = other as u64 * (arr[i] >> 32);
            let lower = other as u64 * (arr[i] & 0xFFFFFFFF);
            if not_last_word {
                carry[i + 1] += upper >> 32;
            }
            let (sum, overflow) = lower.overflowing_add(upper << 32);
            ret[i] = sum;
            if overflow && not_last_word {
                carry[i + 1] += 1;
            }
        }
        U256(ret) + U256(carry)
    }

    /// Create an object from a given unsigned 64-bit integer
    pub fn from_u64(init: u64) -> Option<U256> {
        let mut ret = [0; 4];
        ret[0] = init;
        Some(U256(ret))
    }

    /// Create an object from a given signed 64-bit integer
    pub fn from_i64(init: i64) -> Option<U256> {
        assert!(init >= 0);
        U256::from_u64(init as u64)
    }

    /// Converts from big endian representation bytes in memory.
    // TODO write a test for this please.
    pub fn from_big_endian(slice: &[u8]) -> Self {
        assert!(4 * 8 >= slice.len());
        assert!(slice.len() % 8 == 0);
        //TODO this may need to be reworked for various size arrays, test this.
        let mut ret = [0; 4];
        let length = slice.len() / 8;
        //TODO this might have to be reversed
        for i in 0..length {
            let start = 0 + i * 8;
            let end = 8 + i * 8;
            let mut bytes = [0; 8];
            bytes.copy_from_slice(&slice[start..end]);
            ret[i] = u64::from_be_bytes(bytes);
        }

        U256(ret)
    }

    //TODO this might or might not work. Needs a lot of testing here.
    pub fn from_bytes(slice: &[u8]) -> Self {
        assert!(4 * 8 >= slice.len());
        assert!(slice.len() % 8 == 0);
        //TODO this may need to be reworked for various size arrays, test this.
        let mut ret = [0; 4];
        let length = slice.len() / 8;
        //TODO this might have to be reversed
        for i in 0..length {
            let start = 0 + i * 8;
            let end = 8 + i * 8;
            let mut bytes = [0; 8];
            bytes.copy_from_slice(&slice[start..end]);
            ret[i] = u64::from_le_bytes(bytes);
        }

        U256(ret)
    }

    #[inline]
    pub fn increment(&mut self) {
        let &mut U256(ref mut arr) = self;
        arr[0] += 1;
        if arr[0] == 0 {
            arr[1] += 1;
            if arr[1] == 0 {
                arr[2] += 1;
                if arr[2] == 0 {
                    arr[3] += 1;
                }
            }
        }
    }
}

impl ::std::ops::Add<U256> for U256 {
    type Output = U256;

    fn add(self, other: U256) -> U256 {
        let U256(ref me) = self;
        let U256(ref you) = other;
        let mut ret = [0u64; 4];
        let mut carry = [0u64; 4];
        let mut b_carry = false;
        for i in 0..4 {
            ret[i] = me[i].wrapping_add(you[i]);
            if i < 4 - 1 && ret[i] < me[i] {
                carry[i + 1] = 1;
                b_carry = true;
            }
        }
        if b_carry {
            U256(ret) + U256(carry)
        } else {
            U256(ret)
        }
    }
}

impl ::std::ops::Sub<U256> for U256 {
    type Output = U256;

    #[inline]
    fn sub(self, other: U256) -> U256 {
        //TODO should this be U256::one()?
        self + !other + BitArray::one()
    }
}

impl ::std::ops::Mul<U256> for U256 {
    type Output = U256;

    fn mul(self, other: U256) -> U256 {
        let mut me = U256::zero();
        // TODO: be more efficient about this
        for i in 0..(2 * 4) {
            let to_mul = (other >> (32 * i)).low_u32();
            me = me + (self.mul_u32(to_mul) << (32 * i));
        }
        me
    }
}

impl ::std::ops::Div<U256> for U256 {
    type Output = U256;

    fn div(self, other: U256) -> U256 {
        let mut sub_copy = self;
        let mut shift_copy = other;
        let mut ret = [0u64; 4];

        let my_bits = self.bits();
        let your_bits = other.bits();

        // Check for division by 0
        assert!(your_bits != 0);

        // Early return in case we are dividing by a larger number than us
        if my_bits < your_bits {
            return U256(ret);
        }

        // Bitwise long division
        let mut shift = my_bits - your_bits;
        shift_copy = shift_copy << shift;
        loop {
            if sub_copy >= shift_copy {
                ret[shift / 64] |= 1 << (shift % 64);
                sub_copy = sub_copy - shift_copy;
            }
            shift_copy = shift_copy >> 1;
            if shift == 0 {
                break;
            }
            shift -= 1;
        }

        U256(ret)
    }
}

/// Little-endian large integer type
// impl_array_newtype!($name, u64, $n_words);

impl BitArray for U256 {
    #[inline]
    fn bit(&self, index: usize) -> bool {
        let &U256(ref arr) = self;
        arr[index / 64] & (1 << (index % 64)) != 0
    }

    #[inline]
    fn bit_slice(&self, start: usize, end: usize) -> U256 {
        (*self >> start).mask(end - start)
    }

    #[inline]
    fn mask(&self, n: usize) -> U256 {
        let &U256(ref arr) = self;
        let mut ret = [0; 4];
        for i in 0..4 {
            if n >= 0x40 * (i + 1) {
                ret[i] = arr[i];
            } else {
                ret[i] = arr[i] & ((1 << (n - 0x40 * i)) - 1);
                break;
            }
        }
        U256(ret)
    }

    #[inline]
    fn trailing_zeros(&self) -> usize {
        let &U256(ref arr) = self;
        for i in 0..(4 - 1) {
            if arr[i] > 0 {
                return (0x40 * i) + arr[i].trailing_zeros() as usize;
            }
        }
        (0x40 * (4 - 1)) + arr[4 - 1].trailing_zeros() as usize
    }

    fn zero() -> U256 {
        U256([0; 4])
    }

    fn one() -> U256 {
        U256({
            let mut ret = [0; 4];
            ret[0] = 1;
            ret
        })
    }
}

impl ::std::default::Default for U256 {
    fn default() -> U256 {
        BitArray::zero()
    }
}

impl ::std::ops::BitAnd<U256> for U256 {
    type Output = U256;

    #[inline]
    fn bitand(self, other: U256) -> U256 {
        let U256(ref arr1) = self;
        let U256(ref arr2) = other;
        let mut ret = [0u64; 4];
        for i in 0..4 {
            ret[i] = arr1[i] & arr2[i];
        }
        U256(ret)
    }
}

impl ::std::ops::BitXor<U256> for U256 {
    type Output = U256;

    #[inline]
    fn bitxor(self, other: U256) -> U256 {
        let U256(ref arr1) = self;
        let U256(ref arr2) = other;
        let mut ret = [0u64; 4];
        for i in 0..4 {
            ret[i] = arr1[i] ^ arr2[i];
        }
        U256(ret)
    }
}

impl ::std::ops::BitOr<U256> for U256 {
    type Output = U256;

    #[inline]
    fn bitor(self, other: U256) -> U256 {
        let U256(ref arr1) = self;
        let U256(ref arr2) = other;
        let mut ret = [0u64; 4];
        for i in 0..4 {
            ret[i] = arr1[i] | arr2[i];
        }
        U256(ret)
    }
}

impl ::std::ops::Not for U256 {
    type Output = U256;

    #[inline]
    fn not(self) -> U256 {
        let U256(ref arr) = self;
        let mut ret = [0u64; 4];
        for i in 0..4 {
            ret[i] = !arr[i];
        }
        U256(ret)
    }
}

impl ::std::ops::Shl<usize> for U256 {
    type Output = U256;

    fn shl(self, shift: usize) -> U256 {
        let U256(ref original) = self;
        let mut ret = [0u64; 4];
        let word_shift = shift / 64;
        let bit_shift = shift % 64;
        for i in 0..4 {
            // Shift
            if bit_shift < 64 && i + word_shift < 4 {
                ret[i + word_shift] += original[i] << bit_shift;
            }
            // Carry
            if bit_shift > 0 && i + word_shift + 1 < 4 {
                ret[i + word_shift + 1] += original[i] >> (64 - bit_shift);
            }
        }
        U256(ret)
    }
}

impl ::std::ops::Shr<usize> for U256 {
    type Output = U256;

    fn shr(self, shift: usize) -> U256 {
        let U256(ref original) = self;
        let mut ret = [0u64; 4];
        let word_shift = shift / 64;
        let bit_shift = shift % 64;
        for i in word_shift..4 {
            // Shift
            ret[i - word_shift] += original[i] >> bit_shift;
            // Carry
            if bit_shift > 0 && i < 4 - 1 {
                ret[i - word_shift] += original[i + 1] << (64 - bit_shift);
            }
        }
        U256(ret)
    }
}

//TODO convert these to hex?
impl fmt::Debug for U256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let &U256(ref data) = self;
        write!(f, "0x")?;
        for ch in data.iter().rev() {
            write!(f, "{:016x}", ch)?;
        }
        Ok(())
    }
}

impl fmt::Display for U256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        <fmt::Debug>::fmt(self, f)
    }
}

#[cfg(feature = "serialization")]
impl serde::Serialize for U256 {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        if s.is_human_readable() {
            s.serialize_str(&self.to_hex())
        } else {
            s.serialize_bytes(&self.to_le_bytes())
        }
    }
}

#[cfg(feature = "serialization")]
impl<'de> serde::Deserialize<'de> for U256 {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<U256, D::Error> {
        if d.is_human_readable() {
            struct HexVisitor;

            impl<'de> serde::de::Visitor<'de> for HexVisitor {
                type Value = U256;

                fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                    formatter.write_str("an ASCII hex string")
                }

                fn visit_bytes<E>(self, v: &[u8]) -> std::result::Result<Self::Value, E>
                where
                    E: ::serde::de::Error,
                {
                    if let Ok(hex) = ::std::str::from_utf8(v) {
                        U256::from_hex(hex).map_err(E::custom)
                    } else {
                        return Err(E::invalid_value(serde::de::Unexpected::Bytes(v), &self));
                    }
                }

                fn visit_str<E>(self, v: &str) -> std::result::Result<Self::Value, E>
                where
                    E: ::serde::de::Error,
                {
                    U256::from_hex(v).map_err(E::custom)
                }
            }

            d.deserialize_str(HexVisitor)
        } else {
            struct BytesVisitor;

            impl<'de> ::serde::de::Visitor<'de> for BytesVisitor {
                type Value = U256;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("a bytestring")
                }

                fn visit_bytes<E>(self, v: &[u8]) -> std::result::Result<Self::Value, E>
                where
                    E: ::serde::de::Error,
                {
                    if v.len() != 32 {
                        Err(E::invalid_length(v.len(), &stringify!(32)))
                    } else {
                        let mut ret = [0; 32];
                        ret.copy_from_slice(v);
                        Ok(U256::from(ret))
                    }
                }
            }

            d.deserialize_bytes(BytesVisitor)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn U256_bits_test() {
        assert_eq!(U256::from_u64(255).unwrap().bits(), 8);
        assert_eq!(U256::from_u64(256).unwrap().bits(), 9);
        assert_eq!(U256::from_u64(300).unwrap().bits(), 9);
        assert_eq!(U256::from_u64(60000).unwrap().bits(), 16);
        assert_eq!(U256::from_u64(70000).unwrap().bits(), 17);

        // Try to read the following lines out loud quickly
        let mut shl = U256::from_u64(70000).unwrap();
        shl = shl << 100;
        assert_eq!(shl.bits(), 117);
        shl = shl << 100;
        assert_eq!(shl.bits(), 217);
        shl = shl << 100;
        assert_eq!(shl.bits(), 0);

        // Bit set check
        assert!(!U256::from_u64(10).unwrap().bit(0));
        assert!(U256::from_u64(10).unwrap().bit(1));
        assert!(!U256::from_u64(10).unwrap().bit(2));
        assert!(U256::from_u64(10).unwrap().bit(3));
        assert!(!U256::from_u64(10).unwrap().bit(4));
    }

    #[test]
    pub fn U256_display_test() {
        assert_eq!(
            format!("{}", U256::from_u64(0xDEADBEEF).unwrap()),
            "0x00000000000000000000000000000000000000000000000000000000deadbeef"
        );
        assert_eq!(
            format!("{}", U256::from_u64(u64::max_value()).unwrap()),
            "0x000000000000000000000000000000000000000000000000ffffffffffffffff"
        );

        let max_val = U256([
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
        ]);
        assert_eq!(
            format!("{}", max_val),
            "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
        );
    }

    #[test]
    pub fn U256_comp_test() {
        let small = U256([10u64, 0, 0, 0]);
        let big = U256([0x8C8C3EE70C644118u64, 0x0209E7378231E632, 0, 0]);
        let bigger = U256([0x9C8C3EE70C644118u64, 0x0209E7378231E632, 0, 0]);
        let biggest = U256([0x5C8C3EE70C644118u64, 0x0209E7378231E632, 0, 1]);

        assert!(small < big);
        assert!(big < bigger);
        assert!(bigger < biggest);
        assert!(bigger <= biggest);
        assert!(biggest <= biggest);
        assert!(bigger >= big);
        assert!(bigger >= small);
        assert!(small <= small);
    }

    #[test]
    pub fn U256_arithmetic_test() {
        let init = U256::from_u64(0xDEADBEEFDEADBEEF).unwrap();
        let copy = init;

        let add = init + copy;
        assert_eq!(add, U256([0xBD5B7DDFBD5B7DDEu64, 1, 0, 0]));
        // Bitshifts
        let shl = add << 88;
        assert_eq!(shl, U256([0u64, 0xDFBD5B7DDE000000, 0x1BD5B7D, 0]));
        let shr = shl >> 40;
        assert_eq!(
            shr,
            U256([0x7DDE000000000000u64, 0x0001BD5B7DDFBD5B, 0, 0])
        );
        // Increment
        let mut incr = shr;
        incr.increment();
        assert_eq!(
            incr,
            U256([0x7DDE000000000001u64, 0x0001BD5B7DDFBD5B, 0, 0])
        );
        // Subtraction
        let sub = incr - init;
        assert_eq!(
            sub,
            U256([0x9F30411021524112u64, 0x0001BD5B7DDFBD5A, 0, 0])
        );
        // Multiplication
        let mult = sub.mul_u32(300);
        assert_eq!(
            mult,
            U256([0x8C8C3EE70C644118u64, 0x0209E7378231E632, 0, 0])
        );
        // Division
        assert_eq!(
            U256::from_u64(105).unwrap() / U256::from_u64(5).unwrap(),
            U256::from_u64(21).unwrap()
        );
        let div = mult / U256::from_u64(300).unwrap();
        assert_eq!(
            div,
            U256([0x9F30411021524112u64, 0x0001BD5B7DDFBD5A, 0, 0])
        );
        // TODO: bit inversion
    }

    #[test]
    pub fn mul_u32_test() {
        let u64_val = U256::from_u64(0xDEADBEEFDEADBEEF).unwrap();

        let u96_res = u64_val.mul_u32(0xFFFFFFFF);
        let u128_res = u96_res.mul_u32(0xFFFFFFFF);
        let u160_res = u128_res.mul_u32(0xFFFFFFFF);
        let u192_res = u160_res.mul_u32(0xFFFFFFFF);
        let u224_res = u192_res.mul_u32(0xFFFFFFFF);
        let u256_res = u224_res.mul_u32(0xFFFFFFFF);

        assert_eq!(u96_res, U256([0xffffffff21524111u64, 0xDEADBEEE, 0, 0]));
        assert_eq!(
            u128_res,
            U256([0x21524111DEADBEEFu64, 0xDEADBEEE21524110, 0, 0])
        );
        assert_eq!(
            u160_res,
            U256([0xBD5B7DDD21524111u64, 0x42A4822200000001, 0xDEADBEED, 0])
        );
        assert_eq!(
            u192_res,
            U256([
                0x63F6C333DEADBEEFu64,
                0xBD5B7DDFBD5B7DDB,
                0xDEADBEEC63F6C334,
                0
            ])
        );
        assert_eq!(
            u224_res,
            U256([
                0x7AB6FBBB21524111u64,
                0xFFFFFFFBA69B4558,
                0x854904485964BAAA,
                0xDEADBEEB
            ])
        );
        assert_eq!(
            u256_res,
            U256([
                0xA69B4555DEADBEEFu64,
                0xA69B455CD41BB662,
                0xD41BB662A69B4550,
                0xDEADBEEAA69B455C
            ])
        );
    }

    #[test]
    pub fn multiplication_test() {
        let u64_val = U256::from_u64(0xDEADBEEFDEADBEEF).unwrap();

        let u128_res = u64_val * u64_val;

        assert_eq!(
            u128_res,
            U256([0x048D1354216DA321u64, 0xC1B1CD13A4D13D46, 0, 0])
        );

        let u256_res = u128_res * u128_res;

        assert_eq!(
            u256_res,
            U256([
                0xF4E166AAD40D0A41u64,
                0xF5CF7F3618C2C886u64,
                0x4AFCFF6F0375C608u64,
                0x928D92B4D7F5DF33u64
            ])
        );
    }

    #[test]
    pub fn U256_bitslice_test() {
        let init = U256::from_u64(0xDEADBEEFDEADBEEF).unwrap();
        let add = init + (init << 64);
        assert_eq!(add.bit_slice(64, 128), init);
        assert_eq!(add.mask(64), init);
    }

    #[test]
    pub fn U256_extreme_bitshift_test() {
        // Shifting a u64 by 64 bits gives an undefined value, so make sure that
        // we're doing the Right Thing here
        let init = U256::from_u64(0xDEADBEEFDEADBEEF).unwrap();

        assert_eq!(init << 64, U256([0, 0xDEADBEEFDEADBEEF, 0, 0]));
        let add = (init << 64) + init;
        assert_eq!(add, U256([0xDEADBEEFDEADBEEF, 0xDEADBEEFDEADBEEF, 0, 0]));
        assert_eq!(
            add >> 0,
            U256([0xDEADBEEFDEADBEEF, 0xDEADBEEFDEADBEEF, 0, 0])
        );
        assert_eq!(
            add << 0,
            U256([0xDEADBEEFDEADBEEF, 0xDEADBEEFDEADBEEF, 0, 0])
        );
        assert_eq!(add >> 64, U256([0xDEADBEEFDEADBEEF, 0, 0, 0]));
        assert_eq!(
            add << 64,
            U256([0, 0xDEADBEEFDEADBEEF, 0xDEADBEEFDEADBEEF, 0])
        );
    }
}
