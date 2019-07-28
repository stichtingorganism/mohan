//! Merkle Tree implementation.
//!
//! Merkle tree (MT) implemented as a full binary tree allocated as a vec
//! of statically sized hashes to give hashes more locality. MT specialized
//! to the extent of hashing algorithm and hash item. [`Hashable`] trait is
//! compatible to the `std::hash::Hasher` and supports custom hash algorithms.
//! Implementation does not depend on any external crypto libraries, and tries
//! to be as performant as possible.
//!
//! This tree implementation uses encoding scheme as in _Certificate Transparency_
//! by default. Encoding scheme for leafs and nodes can be overridden though.
//! [RFC 6962](https://tools.ietf.org/html/rfc6962):
//!
//! ```text
//! MTH({d(0)}) = ALG(0x00 || d(0)).
//! For n > 1, let k be the largest power of two smaller than n (i.e.,
//! k < n <= 2k).  The Merkle tree Hash of an n-element list D[n] is then
//! defined recursively as
//! MTH(D[n]) = ALG(0x01 || MTH(D[0:k]) || MTH(D[k:n])),
//! ```
//!
//! # Interface
//!
//! ```text
//! - build_tree (items) -> tree
//! - get_root -> hash
//! - gen_proof -> proof
//! - validate_proof (proof, leaf, root) -> bool
//! ```
//! # Quick start
//!
//! ```
//! 
//! extern crate mohan;
//! extern crate blake2b_simd; 
//!
//! mod example {
//!     use std::fmt;
//!     use std::hash::Hasher;
//!     use std::iter::FromIterator;
//!     use mohan::types::H256;
//!     use mohan::merkle::{Algorithm, Hashable};
//!
//!     pub struct ExampleAlgorithm(blake2b_simd::State);
//!
//!     impl ExampleAlgorithm {
//!         pub fn new() -> ExampleAlgorithm {
//!             ExampleAlgorithm(blake2b_simd::Params::new().hash_length(32).to_state())
//!         }
//!     }
//!
//!     impl Default for ExampleAlgorithm {
//!         fn default() -> ExampleAlgorithm {
//!             ExampleAlgorithm::new()
//!         }
//!     }
//!
//!     impl Hasher for ExampleAlgorithm {
//!         #[inline]
//!         fn write(&mut self, msg: &[u8]) {
//!             self.0.update(msg);
//!         }
//!
//!         #[inline]
//!         fn finish(&self) -> u64 {
//!             unimplemented!()
//!         }
//!     }
//!
//!     impl Algorithm<H256> for ExampleAlgorithm {
//!         #[inline]
//!         fn hash(&mut self) -> H256 {
//!             H256::from_slice(self.0.finalize().as_bytes())
//!         }
//!
//!         #[inline]
//!         fn reset(&mut self) {
//!             *self =
//!                  ExampleAlgorithm(blake2b_simd::Params::new().hash_length(32).to_state())
//!             
//!         }
//!     }
//! }
//!
//! fn main() {
//!     use example::ExampleAlgorithm;
//!     use mohan::merkle::{MerkleTree,VecStore};
//!     use mohan::types::H256;
//!     use std::iter::FromIterator;
//!
//!     let mut h1 = H256::zero();
//!     let mut h2 = H256::from(2);
//!     let mut h3 = H256::from(3);
//!
//!     let t: MerkleTree<H256, ExampleAlgorithm, VecStore<_>> = MerkleTree::from_iter(vec![h1, h2, h3]);
//!     println!("{:?}", t.root());
//! }
//! ```

/// Merkle tree inclusion proof
mod proof;
pub use proof::Proof;

/// Merkle tree abstractions, implementation and algorithms.
mod merkle;
pub use merkle::MerkleTree;

/// Merkle tree storage abstractions, and Vector backed implementation.
mod store;
pub use store::{Element, Store, VecStore};

/// Hash algo
mod algo;
pub use algo::{Algorithm, Hashable};

/// Common implementations for [`Hashable`].
#[cfg(test)]
mod hash_impl;

/// Tests data.
#[cfg(test)]
mod test_item;

/// Tests SIP.
#[cfg(test)]
mod test_sip;

/// Tests for Merkle Hasher Customization
#[cfg(test)]
mod test_cmh;