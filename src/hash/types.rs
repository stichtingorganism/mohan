// Copyright 2021 Stichting Organism
// Copyright 2018 The Grin Developers
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
use crate::hex;
use crate::ser::{
    self, AsFixedBytes, FixedLength, ProtocolVersion, Readable, Reader, Writeable, Writer,
};
use byteorder::{BigEndian, ByteOrder};
use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::convert::AsRef;
use std::ops::Add;
use std::{fmt, ops};


fixed_hash::construct_fixed_hash! {
    /// My 256 bit hash type.
    #[derive(Serialize, Deserialize)]
    pub struct H256(32);
}


impl DefaultHashable for H256 {}

impl H256 {
  
    pub fn hash_with<T: Writeable>(&self, other: T) -> H256 {
        let mut hasher = HashWriter::default();
        ser::Writeable::write(self, &mut hasher).unwrap();
        ser::Writeable::write(&other, &mut hasher).unwrap();
        let mut ret = [0; 32];
        hasher.finalize(&mut ret);
        H256(ret)
    }

    /// Builds a Hash from a byte vector. If the vector is too short, it will be
    /// completed by zeroes. If it's too long, it will be truncated.
    pub fn from_vec(v: &[u8]) -> H256 {
        let mut h = [0; H256::LEN];
        let copy_size = min(v.len(), H256::LEN);
        h[..copy_size].copy_from_slice(&v[..copy_size]);
        H256(h)
    }

    /// Converts the hash to a byte vector
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    /// Convert a hash to hex string format.
    pub fn to_hex(&self) -> String {
        hex::to_hex(&self.to_vec())
    }

    /// Convert hex string back to hash.
    pub fn from_hex(hex: &str) -> Result<H256, ser::Error> {
        let bytes = hex::from_hex(hex.to_string())
            .map_err(|_| ser::Error::HexError(format!("failed to decode {}", hex)))?;
        Ok(H256::from_vec(&bytes))
    }

    /// Most significant 64 bits
    pub fn to_u64(&self) -> u64 {
        BigEndian::read_u64(&self.0)
    }

    /// Convert Hash into a Scalar
    pub fn into_scalar(&self) -> crate::dalek::scalar::Scalar {
        crate::dalek::scalar::Scalar::from_bits(self.0)
    }

    ///Flip into u256
    pub fn to_uint(&self) -> crate::U256 {
        crate::U256::from(self.0)
    }
}


impl FixedLength for H256 {
    /// Size of a hash in bytes.
    const LEN: usize = 32;
}


/// A trait for types that have a canonical hash
pub trait Hashed {
    /// Obtain the hash of the object at 256bits
    fn hash(&self) -> H256;
}

impl Readable for H256 {
    fn read(reader: &mut dyn Reader) -> Result<H256, ser::Error> {
        let v = reader.read_fixed_bytes(32)?;
        let mut a = [0; 32];
        a.copy_from_slice(&v[..]);
        Ok(H256(a))
    }
}

impl Writeable for H256 {
    fn write<W: Writer>(&self, writer: &mut W) -> Result<(), ser::Error> {
        writer.write_fixed_bytes(&self.0)
    }
}

impl Add for H256 {
    type Output = H256;
    fn add(self, other: H256) -> H256 {
        self.hash_with(other)
    }
}


/// Serializer that outputs a hash of the serialized object
pub struct HashWriter {
    state: crate::blake2::State,
}

impl HashWriter {
    /// Consume the `HashWriter`, outputting its current hash into a 32-byte
    /// array
    pub fn finalize(&mut self, output: &mut [u8]) {
        output.copy_from_slice(self.state.finalize().as_bytes())
    }

    /// Consume the `HashWriter`, outputting a `Hash` corresponding to its
    /// current state
    pub fn into_hash(&mut self) -> H256 {
        H256::from_hex(self.state.finalize().to_hex().as_str()).unwrap()
    }
}

impl Default for HashWriter {
    fn default() -> HashWriter {
        // Create a Params object with a secret key and a non-default length.
        let mut params = crate::blake2::Params::new();
        params.hash_length(32);

        HashWriter {
            state: params.to_state(),
        }
    }
}

impl ser::Writer for HashWriter {
    fn serialization_mode(&self) -> ser::SerializationMode {
        ser::SerializationMode::Hash
    }

    fn write_fixed_bytes<T: AsFixedBytes>(&mut self, b32: &T) -> Result<(), ser::Error> {
        self.state.update(b32.as_ref());
        Ok(())
    }

    fn protocol_version(&self) -> ProtocolVersion {
        ProtocolVersion::local()
    }
}

/// Implementing this trait enables the default
/// hash implementation
pub trait DefaultHashable: Writeable {}
impl<D: DefaultHashable> Hashed for D {
    fn hash(&self) -> H256 {
        let mut hasher = HashWriter::default();
        Writeable::write(self, &mut hasher).unwrap();
        let mut ret = [0; 32];
        hasher.finalize(&mut ret);
        H256(ret)
    }
}

impl DefaultHashable for Vec<u8> {}
impl DefaultHashable for u64 {}


#[cfg(test)]
mod tests {
    use super::H256;
    use std::str::FromStr;

    #[test]
    fn test_serialize_h256_zero() {
        let rawzero = H256::zero();
        let strzero =
            H256::from_str("0x0000000000000000000000000000000000000000000000000000000000000000")
                .unwrap();

        assert_eq!(rawzero, strzero);
    }

    #[test]
    fn test_serialize_h256_two() {
        let rawzero = H256::from_vec(&vec![2]);
        let strzero =
            H256::from_str("0x0200000000000000000000000000000000000000000000000000000000000000")
                .unwrap();

        assert_eq!(rawzero, strzero);
    }

    #[test]
    fn test_serialize_h256_sixteen() {
        let rawzero = H256::from_vec(&vec![16]);
        let strzero =
            H256::from_hex("0x1000000000000000000000000000000000000000000000000000000000000000")
                .unwrap();

        assert_eq!(rawzero, strzero);
    }
}
