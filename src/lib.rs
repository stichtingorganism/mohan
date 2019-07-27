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
extern crate schnorr as schnorr_raw;


/// Fixed Integers & Hash
pub mod types;
/// Repub bech32
pub mod bech32;
/// Serilization
pub mod ser;
/// Serde Support
pub mod mserde;
/// Parking Lot backed Sync Primitives
pub mod sync;
/// To & From Hex
pub mod hex;
/// Various Sized Blake2b hash functions
pub mod hash;
/// 64bit Time handling
pub mod tai64;
/// Repub Bytes
pub use bytes;
/// Repub Byteorder
pub use byteorder;


pub mod schnorr {
    /// Repub Schnorr
    pub use schnorr_raw::*;

    //TODO: Implement Encoding 
}