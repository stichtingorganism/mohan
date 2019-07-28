//! Algorithm used to hash tree

use std::hash::Hasher;
use crate::merkle::Element;
use crate::hash::H256;
use crate::ser::FixedLength;

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


impl Element for H256 {
    fn byte_len() -> usize {
        H256::LEN
    }

    fn from_slice(bytes: &[u8]) -> Self {
        if bytes.len() != Self::LEN {
            panic!("invalid length {}, expected 32", bytes.len());
        }

        H256::from_vec(bytes)
    }

    fn copy_to_slice(&self, bytes: &mut [u8]) {
        bytes.copy_from_slice(self.as_bytes());
    }
}


