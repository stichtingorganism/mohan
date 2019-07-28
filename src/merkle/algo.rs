//! Algorithm used to hash tree

use std::hash::Hasher;
use crate::merkle::Element;
use crate::types::H256;

/// MT leaf hash prefix
const LEAF: u8 = 0x00;

/// MT interior node hash prefix
const INTERIOR: u8 = 0x01;


/// A hashable type.
///
/// Types implementing `Hashable` are able to be [`hash`]ed with an instance of
/// [`Hasher`].
///
pub trait Hashable<H: Hasher> {
    /// Feeds this value into the given [`Hasher`].
    ///
    /// [`Hasher`]: trait.Hasher.html
    fn hash(&self, state: &mut H);

    /// Feeds a slice of this type into the given [`Hasher`].
    ///
    /// [`Hasher`]: trait.Hasher.html
    fn hash_slice(data: &[Self], state: &mut H)
    where
        Self: Sized,
    {
        for piece in data {
            piece.hash(state);
        }
    }
}


/// A trait for hashing an arbitrary stream of bytes for calculating merkle tree
/// nodes.
///
/// T is a hash item must be of known size at compile time, globally ordered, with
/// default value as a neutral element of the hash space. Neutral element is
/// interpreted as 0 or nil and required for evaluation of merkle tree.
///
/// [`Algorithm`] breaks the [`Hasher`] contract at `finish()`, but that is intended.
/// This trait extends [`Hasher`] with `hash -> T` and `reset` state methods,
/// plus implements default behavior of evaluation of MT interior nodes.
pub trait Algorithm<T>: Hasher + Default
where
    T: Clone + AsRef<[u8]>,
{
    /// Returns the hash value for the data stream written so far.
    fn hash(&mut self) -> T;

    /// Reset Hasher state.
    #[inline]
    fn reset(&mut self) {
        *self = Self::default();
    }

    /// Returns hash value for MT leaf (prefix 0x00).
    #[inline]
    fn leaf(&mut self, leaf: T) -> T {
        self.write(&[LEAF]);
        self.write(leaf.as_ref());
        self.hash()
    }

    /// Returns hash value for MT interior node (prefix 0x01).
    #[inline]
    fn node(&mut self, left: T, right: T, _height: usize) -> T {
        self.write(&[INTERIOR]);
        self.write(left.as_ref());
        self.write(right.as_ref());
        self.hash()
    }
}


// /// Hasher used to build tree @ 256bits
// pub struct B {
//     state: blake2b_simd::State
// }

// impl B {

//     pub fn new() -> Self {
//         Self {
//             state: blake2b_simd::Params::new().hash_length(32).to_state()
//         }
//     }

//     /// Reset Hasher state.
//     #[inline]
//     pub fn reset(&mut self) {
//         *self = Self {
//             state: blake2b_simd::Params::new().hash_length(32).to_state()
//         };
//     }

//     /// Feed data into Hash State
//     #[inline]
//     pub fn update(&mut self, input: &[u8]) {
//         self.state.update(input);
//     }

//     /// Returns the hash value for the data stream and consumes state.
//     #[inline]
//     pub fn finalize(&mut self) -> H256 {
//         H256::from_slice(self.state.finalize().as_bytes())
//     }

//     /// Returns hash value for MT leaf (prefix 0x00).
//     #[inline]
//     pub fn leaf(&mut self, leaf: H256) -> H256 {
//         self.update(&[LEAF]);
//         self.update(leaf.as_bytes());
//         self.finalize()
//     }

//     /// Returns hash value for MT interior node (prefix 0x01).
//     #[inline]
//     pub fn node(&mut self, left: H256, right: H256, _height: usize) -> H256 {
//         self.update(&[INTERIOR]);
//         self.update(left.as_bytes());
//         self.update(right.as_bytes());
//         self.finalize()
//     }


// }

impl Element for H256 {
    fn byte_len() -> usize {
        H256::len_bytes()
    }

    fn from_slice(bytes: &[u8]) -> Self {
        if bytes.len() != Self::len_bytes() {
            panic!("invalid length {}, expected 32", bytes.len());
        }

        H256::from_slice(bytes)
    }

    fn copy_to_slice(&self, bytes: &mut [u8]) {
        bytes.copy_from_slice(self.as_bytes());
    }
}


