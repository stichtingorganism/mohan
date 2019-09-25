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

/// Repub bytes
pub use bytes;
/// bech32
pub mod bech32;
/// Serilization
pub mod ser;
/// Parking Lot backed Sync Primitives
pub mod sync;
/// To & From Hex
pub mod hex;
/// 64bit Time handling
pub mod tai64;
/// Repub Byteorder
pub use byteorder;
/// Keccak-f sponge function
pub mod sponge;
/// Variable Encoding Integer
pub mod varint;
/// Golomb for block filters
pub mod golomb;
/// Export Curve
pub use curve25519_dalek as dalek;
/// Various Hash functions & types
pub mod hash;
/// Export blake2b
pub use blake2b_simd as blake2;
/// That extra sauce
pub mod tools;


#[inline(always)]
/// A hack function to use zeroize
fn zeroize_hack<Z: Default>(z: &mut Z) {
    use core::{ptr, sync::atomic};
    unsafe { ptr::write_volatile(z, Z::default()); }
    atomic::compiler_fence(atomic::Ordering::SeqCst);
}