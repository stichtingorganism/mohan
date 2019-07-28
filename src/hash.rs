// Copyright 2019 Stichting Organism
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Hash Functions
use crate::types::{H160, H128, H256, H384};
use blake2b_simd::{
    Params
};

/// Repub Blak2b
pub use blake2b_simd as blake2b;

/// Blake160 hashes data without key at 20 bytes blake2b
pub fn blake160(data: &[u8]) -> H160 {
    H160::from_slice(Params::new().hash_length(32).hash(data).as_bytes())
}

/// Blake256 hashes data without key at 32 bytes blake2b
pub fn blake256(data: &[u8]) -> H256 {
    H256::from_slice(Params::new().hash_length(32).hash(data).as_bytes())
}

/// Blake384 hashes data without key at 48 bytes blake2b
pub fn blake384(data: &[u8]) -> H384 {
    H384::from_slice(Params::new().hash_length(48).hash(data).as_bytes())
}


/// Paranoid Hash: blakeblake256 
/// blake256(blake256(blake384(blake384(data)))) 
pub fn blakeblake256(data: &[u8]) -> H256 {
  blake256(blake256(blake384(blake384(data).as_bytes()).as_bytes()).as_bytes())
}

/// Blake2b based Message Authentication Code @ 16 bytes
pub fn hmac_128(info: &[u8], salt: &[u8], key: &[u8]) -> H128 {
  let mut params = Params::new();
  params.personal(info);
  params.salt(salt);
  params.key(key);
  params.hash_length(16);

  // Use those params to hash an input all at once.
  H128::from_slice(params.hash(b"euka").as_bytes())
}


/// Blake2b based Message Authentication Code @ 32 bytes
pub fn hmac_256(info: &[u8], salt: &[u8], key: &[u8]) -> H256 {
  let mut params = Params::new();
  params.personal(info);
  params.salt(salt);
  params.key(key);
  params.hash_length(32);

  // Use those params to hash an input all at once.
  H256::from_slice(params.hash(b"euka").as_bytes())
}

//
// Copyright 2018 The Grin Developers
//

/// A trait for types that have a canonical hash
pub trait Hashed {
  
	/// Obtain the hash of the object at 256bits
	fn hash(&self) -> H256;
}
