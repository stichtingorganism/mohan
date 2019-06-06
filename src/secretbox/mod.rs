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

//! Secret-key authenticated encryption
//chacha20-poly1305-aead
//https://tools.ietf.org/html/rfc7539


//Expose Internal
pub mod errors;

use zeroize::Zeroize;
use core::fmt::{Debug};
use rand::CryptoRng;
use rand::Rng;
use serde::{Serialize, Deserialize};
use serde::{Serializer, Deserializer};
use serde::de::Error as SerdeError;
use serde::de::Visitor;
use chacha20_poly1305_aead as chacha;
use self::errors::SecretBoxError;
use self::errors::InternalError;


/// The length of a Symmetric Key used for encryption in bytes. 256-bits.
pub const SECRETBOX_KEY_LEN: usize = 32;

/// The length of a Nonce used for unique encryption in bytes. 96-bits.
pub const SECRETBOX_NONCE_LEN: usize = 12;


/// A symmetric key for crypto box
#[derive(Zeroize)]
#[zeroize(drop)]
pub struct SymmetricKey(pub (crate) [u8; SECRETBOX_KEY_LEN]);

impl Debug for SymmetricKey {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "SymmetricKey: {:?}", &self.0[..])
    }
}


impl SymmetricKey {

    /// Convert this Symmetric key to a byte array.
    #[inline]
    pub fn to_bytes(&self) -> [u8; SECRETBOX_KEY_LEN] {
        self.0
    }

    /// View this Symmetric key as a byte array.
    #[inline]
    pub fn as_bytes<'a>(&'a self) -> &'a [u8; SECRETBOX_KEY_LEN] {
        &self.0
    }

    /// Construct a `SymmetricKey` from a slice of bytes.
    ///
    /// # Returns
    ///
    /// A `Result` whose okay value is an encryption `SymmetricKey` or whose error value
    /// is an `SecretBoxError` wrapping the internal error that occurred.
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Result<SymmetricKey, SecretBoxError> {
        if bytes.len() != SECRETBOX_KEY_LEN {
            return Err(SecretBoxError(InternalError::BytesLengthError{
                name: "SymmetricKey", length: SECRETBOX_KEY_LEN 
            }));
        }

        let mut bits: [u8; 32] = [0u8; 32];
        bits.copy_from_slice(&bytes[..32]);

        Ok(SymmetricKey(bits))
    }

    /// Generate a `SymmetricKey` from a `csprng`.
    ///
    /// # Input
    ///
    /// A CSPRNG with a `fill_bytes()` method, e.g. `rand::ChaChaRng`
    pub fn generate<T>(csprng: &mut T) -> SymmetricKey
        where T: CryptoRng + Rng,
    {
        let mut sk: SymmetricKey = SymmetricKey([0u8; 32]);

        csprng.fill_bytes(&mut sk.0);

        sk
    }


}


impl Serialize for SymmetricKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_bytes(self.as_bytes())
    }
}


impl<'d> Deserialize<'d> for SymmetricKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'d> {
        struct SymmetricKeyVisitor;

        impl<'d> Visitor<'d> for SymmetricKeyVisitor {
            type Value = SymmetricKey;

            fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                formatter.write_str("An Symmetric key as 32 bytes.")
            }

            fn visit_bytes<E>(self, bytes: &[u8]) -> Result<SymmetricKey, E> where E: SerdeError {
                SymmetricKey::from_bytes(bytes).or(Err(SerdeError::invalid_length(bytes.len(), &self)))
            }
        }
        deserializer.deserialize_bytes(SymmetricKeyVisitor)
    }
}


/// A nonce key for crypto box
#[derive(Zeroize, Eq, PartialEq)]
#[zeroize(drop)]
pub struct NonceKey(pub (crate) [u8; SECRETBOX_NONCE_LEN]);

impl Debug for NonceKey {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "NonceKey: {:?}", &self.0[..])
    }
}



impl NonceKey {

    /// Convert this secret key to a byte array.
    #[inline]
    pub fn to_bytes(&self) -> [u8; SECRETBOX_NONCE_LEN] {
        self.0
    }

    /// View this secret key as a byte array.
    #[inline]
    pub fn as_bytes<'a>(&'a self) -> &'a [u8; SECRETBOX_NONCE_LEN] {
        &self.0
    }

    /// Construct a `NonceKey` from a slice of bytes.
    ///
    /// # Returns
    ///
    /// A `Result` whose okay value is an `NonceKey` or whose error value
    /// is an `SecretBoxError` wrapping the internal error that occurred.
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Result<NonceKey, SecretBoxError> {
        if bytes.len() != SECRETBOX_NONCE_LEN {
            return Err(SecretBoxError(InternalError::BytesLengthError{
                name: "NonceKey", length: SECRETBOX_NONCE_LEN }));
        }

        let mut bits: [u8; 12] = [0u8; 12];
        bits.copy_from_slice(&bytes[..12]);

        Ok(NonceKey(bits))
    }

    /// Generate a `NonceKey` from a `csprng`.
    ///
    /// # Input
    ///
    /// A CSPRNG with a `fill_bytes()` method, e.g. `rand::ChaChaRng`
    pub fn generate<T>(csprng: &mut T) -> NonceKey
        where T: CryptoRng + Rng,
    {
        let mut nonce: NonceKey = NonceKey([0u8; 12]);

        csprng.fill_bytes(&mut nonce.0);

        nonce
    }

}


impl Serialize for NonceKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_bytes(self.as_bytes())
    }
}


impl<'d> Deserialize<'d> for NonceKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'d> {
        struct NonceKeyVisitor;

        impl<'d> Visitor<'d> for NonceKeyVisitor {
            type Value = NonceKey;

            fn expecting(&self, formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                formatter.write_str("An Nonce key as 12 bytes.")
            }

            fn visit_bytes<E>(self, bytes: &[u8]) -> Result<NonceKey, E> where E: SerdeError {
                NonceKey::from_bytes(bytes).or(Err(SerdeError::invalid_length(bytes.len(), &self)))
            }
        }
        deserializer.deserialize_bytes(NonceKeyVisitor)
    }
}

/// AEAD_CHACHA20_POLY1305 is an authenticated encryption with additional
///   data algorithm.  The inputs to AEAD_CHACHA20_POLY1305 are:
///
///  o  A 256-bit key
///
///  o  A 96-bit nonce -- different for each invocation with the same key
///
///  o  An arbitrary length plaintext
///
///  o  Arbitrary length additional authenticated data (AAD)
///
/// A Box encapsulates the cipher text and associated nonce value, it is ok to be public 

#[derive(Eq, PartialEq, Serialize, Deserialize)]
pub struct SecretBox {
    /// Unique nonce of the data
    pub nonce: NonceKey,
    ///Authentication Tag, 16bytes 128-bit tag from Poly1305
    pub tag: [u8; 16],
    /// Ciphertext of data
    pub cipher: Vec<u8>
}


impl Debug for SecretBox {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "Box( nonce: {:?}, cipher: {:?} )", &self.nonce, &self.cipher)
    }
}


impl SecretBox {

    //takes a plaintext message and returns an box object that holds cipher text and nonce
    pub fn lock(key: &SymmetricKey, nonce: NonceKey, message: &[u8], aad: &[u8]) -> Result<SecretBox, SecretBoxError> { 
        //allocation vector to hold cipher text based on given message len.
        let mut ciphertext = Vec::with_capacity(message.len());

        //mesh it up
        let tag = match chacha::encrypt(&key.to_bytes(), &nonce.to_bytes(), &aad, message, &mut ciphertext) {
            Ok(t) => t,
            Err(_) => return Err(SecretBoxError(InternalError::EncryptingError)) 
        };

        //Return Box 
        Ok(SecretBox { nonce: nonce, tag: tag, cipher: ciphertext })
    }
    

    pub fn unlock(&self, key: &SymmetricKey, aad: &[u8]) -> Result<Vec<u8>, SecretBoxError> {
        //TODO::check the length of cipher text is non zero
        //allocation vector to hold cipher text based on given message len.
        let mut plaintext = Vec::with_capacity(self.cipher.len());

        match chacha::decrypt(&key.to_bytes(), &self.nonce.to_bytes(), &aad, &self.cipher, &self.tag, &mut plaintext) {
            Ok(_) =>  return Ok(plaintext),
            Err(_) => return Err(SecretBoxError(InternalError::DecryptingError)) 
        }
        
    }
}



#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_seal_open() {

        let key = SymmetricKey([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
                17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31]);

        let nonce = NonceKey([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

        let aad = [1, 2, 3, 4];

        let plaintext = b"hello, world";

        // let ciphertext = [0xfc, 0x5a, 0x17, 0x82, 0xab, 0xcf, 0xbc, 0x5d,
        //             0x18, 0x29, 0xbf, 0x97];

        // let tag = [0xdb, 0xb7, 0x0d, 0xda, 0xbd, 0xfa, 0x8c, 0xa5,
        //         0x60, 0xa2, 0x30, 0x3d, 0xe6, 0x07, 0x92, 0x10];

        
        //enc
        let b = SecretBox::lock(&key, nonce, plaintext, &aad).unwrap();

        //test computed cipher
        assert_eq!(b.cipher, [0xfc, 0x5a, 0x17, 0x82, 0xab, 0xcf, 0xbc, 0x5d, 0x18, 0x29, 0xbf, 0x97]);
         //test computed tag
        assert_eq!(b.tag, [0xdb, 0xb7, 0x0d, 0xda, 0xbd, 0xfa, 0x8c, 0xa5, 0x60, 0xa2, 0x30, 0x3d, 0xe6, 0x07, 0x92, 0x10]);

        //dec
        let recovred = b.unlock(&key, &aad).unwrap();

        assert_eq!(recovred, b"hello, world");

    
    }

    #[test]
    fn test_seal_open_tamper(){
        // for i in 0..256usize {
            let key = SymmetricKey([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
                17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31]);

            let nonce = NonceKey([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

            let aad = [1, 2, 3, 4];

            let plaintext = b"hello, world";

            let mut boxy = SecretBox::lock(&key, nonce, plaintext, &aad).unwrap();

            for i in 0..boxy.cipher.len() {
                //modify some bytes 
                boxy.cipher[i] ^= 0x20;
                
                let recovred = boxy.unlock(&key, &aad);

                assert!(Err(SecretBoxError(InternalError::DecryptingError)) == recovred);
                boxy.cipher[i] ^= 0x20;
            }
        // }
    }


}