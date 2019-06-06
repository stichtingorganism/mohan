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

#[macro_use] extern crate uint;
#[macro_use] extern crate fixed_hash;
#[cfg(test)] extern crate serde_json;

/// Fixed Integers & Hash
pub mod types;
/// Repub bech32
pub use bech32;
/// Repub Merlin
pub use merlin;
/// Serilization
pub mod ser;
/// Parking Lot backed Sync Primitives
pub mod sync;
/// To & From Hex
pub mod hex;
/// Various Sized Blake2b hash functions
pub mod hash;
/// Repub Blak2b
pub use blake2b_simd as blake2b;
/// Authenticated Symmetric Encryption Capsule
pub mod secretbox;
/// 64bit Time handling
pub use tai64;
/// Repub Zeroize 
#[macro_use(Zeroize)] pub use zeroize;
/// Repub Bytes
pub use bytes;