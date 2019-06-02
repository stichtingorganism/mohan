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

//Parts Taken from
//https://github.com/maidsafe/maidsafe_utilities/blob/master/src/serialisation.rs
//https://github.com/mimblewimble/grin/blob/master/core/src/ser.rs

//!This is opiniated implementation that uses bincode for binary ser/der

// use bincode::{
//     deserialize, deserialize_from, serialize, serialize_into, serialized_size,
//     ErrorKind,
// };

// use bincode::{
//     deserialize, deserialize_from, serialize_into, serialized_size,
//     ErrorKind,
// };

use serde::{de, Deserializer, Serializer};
use std::error;
use std::fmt;

/// Serialisation error.
#[derive(Debug)]
pub enum SerialisationError {
    /// Error during serialisation (encoding).
    Serialise(bincode::ErrorKind),
    /// Bincode error during deserialisation (decoding).
    Deserialise(bincode::ErrorKind),
    /// Not all input bytes were consumed when deserialising (decoding).
    DeserialiseExtraBytes,
}

impl fmt::Display for SerialisationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            SerialisationError::Serialise(ref e) => write!(f, "Serialise error: {}", e),
            SerialisationError::Deserialise(ref e) => write!(f, "Deserialise error: {}", e),
            SerialisationError::DeserialiseExtraBytes => {
                f.write_str("Deserialise error: Not all bytes of slice consumed")
            }
        }
    }
}

impl error::Error for SerialisationError {
    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            SerialisationError::Serialise(ref e) => Some(e),
            SerialisationError::Deserialise(ref e) => Some(e),
            _ => None,
        }
    }

    fn description(&self) -> &str {
        match *self {
            // SerialisationError::Serialise(e) => e,
            // SerialisationError::Deserialise(e) => e,
            SerialisationError::Serialise(_) => "Deserialise error",
            SerialisationError::Deserialise(_) => "Serialise error",
            SerialisationError::DeserialiseExtraBytes => "DeserialiseExtraBytes error",
        }
    }
}

// /// Serialise an `Serialize` type with no limit on the size of the serialised data.
// pub fn serialise<T>(data: &T) -> Result<Vec<u8>, SerialisationError>
// where
//     T: Serialize,
// {
//     serialize(data).map_err(|e| SerialisationError::Serialise(*e))
// }

// /// Deserialise a `Deserialize` type with no limit on the size of the serialised data.
// pub fn deserialise<T>(data: &[u8]) -> Result<T, SerialisationError>
// where
//     T: Serialize + DeserializeOwned,
// {
//     let value = deserialize(data).map_err(|e| SerialisationError::Deserialise(*e))?;
//     if unwrap!(serialized_size(&value)) != data.len() as u64 {
//         return Err(SerialisationError::DeserialiseExtraBytes);
//     }
//     Ok(value)
// }

// /// Serialise an `Serialize` type directly into a `Write` with no limit on the size of the
// /// serialised data.
// pub fn serialise_into<T: Serialize, W: Write>(
//     data: &T,
//     write: &mut W,
// ) -> Result<(), SerialisationError> {
//     serialize_into(write, data).map_err(|e| SerialisationError::Serialise(*e))
// }

// /// Deserialise a `Deserialize` type directly from a `Read` with no limit on the size of the
// /// serialised data.
// pub fn deserialise_from<R: Read, T: DeserializeOwned>(
//     read: &mut R,
// ) -> Result<T, SerialisationError> {
//     deserialize_from(read).map_err(|e| SerialisationError::Deserialise(*e))
// }

// /// Returns the size that an object would be if serialised using [`serialise()`](fn.serialise.html).
// pub fn serialised_size<T: Serialize>(data: &T) -> u64 {
//     unwrap!(serialized_size(data))
// }

// /// Serializes a slice of bytes.
// pub fn serialize_string<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error> where
// 	S: Serializer,
// {
// 	let hex: String = crate::hex::to_hex(bytes.to_vec()).unwrap();
// 	serializer.serialize_str(&format!("0x{}", hex))
// }

/// Serializes a slice of bytes.
pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let hex: String = crate::hex::to_hex(bytes).unwrap();
    serializer.serialize_str(&format!("0x{}", hex))
}

// Taken from https://github.com/paritytech/primitives/blob/master/serialize/src/lib.rs
/// Serialize a slice of bytes as uint.
///
/// The representation will have all leading zeros trimmed.
pub fn serialize_uint<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let non_zero = bytes.iter().take_while(|b| **b == 0).count();
    let bytes = &bytes[non_zero..];
    if bytes.is_empty() {
        return serializer.serialize_str("0x0");
    }

    let hex: String = crate::hex::to_hex(bytes).unwrap();
    let has_leading_zero = !hex.is_empty() && &hex[0..1] == "0";

    serializer.serialize_str(&format!(
        "0x{}",
        if has_leading_zero { &hex[1..] } else { &hex }
    ))
}

/// Expected length of bytes vector.
#[derive(PartialEq, Eq, Debug)]
pub enum ExpectedLen {
    /// Any length in bytes.
    Any,
    /// Exact length in bytes.
    Exact(usize),
    /// A bytes length between (min; max].
    Between(usize, usize),
}

impl fmt::Display for ExpectedLen {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ExpectedLen::Any => write!(fmt, "even length"),
            ExpectedLen::Exact(v) => write!(fmt, "length of {}", v * 2),
            ExpectedLen::Between(min, max) => {
                write!(fmt, "length between ({}; {}]", min * 2, max * 2)
            }
        }
    }
}

/// Deserialize into vector of bytes.
pub fn deserialize_checked<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_check_len(deserializer, ExpectedLen::Any)
}

/// Deserialize into vector of bytes with additional size check.
pub fn deserialize_check_len<'de, D>(deserializer: D, len: ExpectedLen) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Visitor {
        len: ExpectedLen,
    }

    impl<'a> de::Visitor<'a> for Visitor {
        type Value = Vec<u8>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "a 0x-prefixed hex string with {}", self.len)
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
            if !v.starts_with("0x") {
                return Err(E::custom("prefix is missing"));
            }

            let is_len_valid = match self.len {
                // just make sure that we have all nibbles
                ExpectedLen::Any => v.len() % 2 == 0,
                ExpectedLen::Exact(len) => v.len() == 2 * len + 2,
                ExpectedLen::Between(min, max) => v.len() <= 2 * max + 2 && v.len() > 2 * min + 2,
            };

            if !is_len_valid {
                return Err(E::invalid_length(v.len() - 2, &self));
            }

            let bytes = match self.len {
                ExpectedLen::Between(..) if v.len() % 2 != 0 => {
                    crate::hex::from_hex(String::from(&*format!("0{}", &v[2..])))
                }
                _ => crate::hex::from_hex(String::from(&v[2..])),
            };

            fn format_err(e: std::num::ParseIntError) -> String {
                format!("invalid hex value: {:?}", e)
            }

            bytes.map_err(|e| E::custom(format_err(e)))
        }

        fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
            self.visit_str(&v)
        }
    }
    // TODO [ToDr] Use raw bytes if we switch to RLP / binencoding
    // (visit_bytes, visit_bytes_buf)
    deserializer.deserialize_str(Visitor { len })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::{self, Visitor};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::fmt;
    use std::io::Cursor;

    #[test]
    fn serialise_deserialise() {
        let original_data = (
            vec![0u8, 1, 3, 9],
            vec![-1i64, 888, -8765],
            "SomeString".to_string(),
        );

        let serialised_data = bincode::serialize(&original_data)
            .map_err(|e| SerialisationError::Serialise(*e))
            .unwrap();

        let deserialised_data: (Vec<u8>, Vec<i64>, String) = bincode::deserialize(&serialised_data)
            .map_err(|e| SerialisationError::Deserialise(*e))
            .unwrap();
        assert_eq!(original_data, deserialised_data);

        // Try to parse a `String` into a `u64` to check the unused bytes triggers an error.
        let serialised_string = bincode::serialize(&"Another string".to_string())
            .map_err(|e| SerialisationError::Serialise(*e))
            .unwrap();

        bincode::deserialize::<u64>(&serialised_string)
            .map_err(|e| SerialisationError::Deserialise(*e))
            .unwrap();
    }

    #[test]
    fn serialise_into_deserialise_from() {
        let original_data = (
            vec![0u8, 1, 3, 9],
            vec![-1i64, 888, -8765],
            "SomeString".to_string(),
        );
        let mut serialised_data = vec![];
        bincode::serialize_into(&mut serialised_data, &original_data)
            .map_err(|e| SerialisationError::Serialise(*e))
            .unwrap();

        let mut serialised = Cursor::new(serialised_data);
        let deserialised_data: (Vec<u8>, Vec<i64>, String) =
            bincode::deserialize_from(&mut serialised)
                .map_err(|e| SerialisationError::Deserialise(*e))
                .unwrap();

        assert_eq!(original_data, deserialised_data);
    }

    #[test]
    fn sizes() {
        let data = (1u64..8).collect::<Vec<_>>();

        assert_eq!(bincode::serialized_size(&data).unwrap(), 64);
    }

    #[derive(PartialEq, Eq, Debug)]
    struct Wrapper([u8; 1]);

    impl Serialize for Wrapper {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_bytes(&self.0[..])
        }
    }

    impl<'de> Deserialize<'de> for Wrapper {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Wrapper, D::Error> {
            struct WrapperVisitor;
            impl<'de> Visitor<'de> for WrapperVisitor {
                type Value = Wrapper;
                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    write!(formatter, "Wrapper")
                }
                fn visit_bytes<E: de::Error>(self, value: &[u8]) -> Result<Self::Value, E> {
                    if value.len() != 1 {
                        return Err(de::Error::invalid_length(value.len(), &self));
                    }
                    Ok(Wrapper([value[0]]))
                }
            }
            deserializer.deserialize_bytes(WrapperVisitor)
        }
    }

    #[test]
    // The bincode implementation of `serialize_bytes` puts the number of bytes of raw data as the
    // first 8 bytes of the encoded data.  The corresponding `deserialize_bytes` uses these first 8
    // bytes to deduce the size of the buffer into which the raw bytes should then be copied.  If we
    // use bincode's `deserialize_from(.., Infinite)` to try and parse such data, size-checking is
    // disabled when allocating the buffer, and corrupted serialised data could cause an OOM crash.
    fn deserialize_bytes() {
        let wrapper = Wrapper([255]);

        let serialised_wrapper = bincode::serialize(&wrapper)
            .map_err(|e| SerialisationError::Serialise(*e))
            .unwrap();
        // If the following assertion fails, revisit how we're encoding data via `serialize_bytes`
        // to check that the following `tampered` array below is still trying to trigger an OOM
        // error.
        assert_eq!(serialised_wrapper, [1, 0, 0, 0, 0, 0, 0, 0, 255]);
        let deserialised_wrapper: Wrapper = bincode::deserialize(&serialised_wrapper)
            .map_err(|e| SerialisationError::Deserialise(*e))
            .unwrap();

        assert_eq!(wrapper, deserialised_wrapper);

        // Try to trigger an OOM crash.
        //let tampered = [255u8; 9];
        //bincode::deserialize::<Wrapper>(&tampered).map_err(|e| SerialisationError::Deserialise(*e)).unwrap();

        // match  bincode::deserialize::<Wrapper>(&tampered).map_err(|e| SerialisationError::Deserialise(*e)) {
        //     Err(SerialisationError::DeserialiseExtraBytes) => (),
        //     Err(SerialisationError::Deserialise(_)) => (),
        //     Err(SerialisationError::Serialise(_)) => (),
        //     Ok(err) => panic!("{:?}", err),
        // }

        // match bincode::deserialize::<Wrapper>(&[1, 0, 0, 0, 0, 0, 0, 0, 255, 255]).map_err(|e| SerialisationError::Deserialise(*e)) {
        //     Err(SerialisationError::DeserialiseExtraBytes) => (),
        //     Err(SerialisationError::Deserialise(_)) => (),
        //     Err(SerialisationError::Serialise(_)) => (),
        //     Ok(err) => panic!("{:?}", err),
        // }
    }
}
