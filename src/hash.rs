// Copyright 2018 Stichting Organism
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
use crate::types::{H256, H384, H512};
use blake2b_simd::Params;

/// Blake256 hashes data without key at 32 bytes blake2b
pub fn blake256(data: &[u8]) -> H256 {
    H256::from_slice(Params::new().hash_length(32).hash(data).as_bytes())
}

/// Blake384 hashes data without key at 48 bytes blake2b
pub fn blake384(data: &[u8]) -> H384 {
    H384::from_slice(Params::new().hash_length(48).hash(data).as_bytes())
}

/// Blake512 hashes data without key at 64 bytes blake2b
pub fn blake512(data: &[u8]) -> H512 {
    H512::from_slice(Params::new().hash_length(64).hash(data).as_bytes())
}


