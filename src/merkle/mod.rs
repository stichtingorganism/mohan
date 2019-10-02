//
// Forked and reworked as needed, various previous license.
//
// Copyright 2019 by the Filecoin contributors.
// Ivan Prisyazhnyy <john.koepi@gmail.com>
// BSD 3-Clause License
// Copyright (c) 2016, Spin Research
// All rights reserved.
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
// * Redistributions of source code must retain the above copyright notice, this
//   list of conditions and the following disclaimer.
// * Redistributions in binary form must reproduce the above copyright notice,
//   this list of conditions and the following disclaimer in the documentation
//   and/or other materials provided with the distribution.
// * Neither the name of the copyright holder nor the names of its
//   contributors may be used to endorse or promote products derived from
//   this software without specific prior written permission.
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
// AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

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
//! extern crate bacteria;
//!
//! mod example {
//!     use std::fmt;
//!     use std::hash::Hasher;
//!     use std::iter::FromIterator;
//!     use mohan::hash::H256;
//!     use mohan::merkle::{Algorithm, Hashable};
//!     use bacteria::Strobe128;
//!        
//!     //This example is not the best way to use strobe
//!     pub struct ExampleAlgorithm(Strobe128);
//!
//!     impl ExampleAlgorithm {
//!         pub fn new() -> ExampleAlgorithm {
//!             ExampleAlgorithm(Strobe128::new(b"Example Algorithm Strobe"))
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
//!             self.0.ad(msg, false);
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
//!             let mut result = [0u8; 32];
//!             self.0.prf(&mut result, false);
//!             H256::from(result)
//!         }
//!
//!         #[inline]
//!         fn reset(&mut self) {
//!             *self =
//!                  ExampleAlgorithm::new()
//!             
//!         }
//!     }
//! }
//!
//! fn main() {
//!     use example::ExampleAlgorithm;
//!     use mohan::merkle::{MerkleTree,VecStore};
//!     use mohan::hash::H256;
//!     use std::iter::FromIterator;
//!
//!     let mut h1 = H256::zero();
//!     let mut h2 = H256::from_vec(&vec![1u8, 1u8]);
//!     let mut h3 = H256::from_vec(&vec![2u8, 2u8]);
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