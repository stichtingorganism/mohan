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
mod sip;
pub use sip::SipHasher24 as SipHasher;

mod types;
pub use types::{
    H256,
    HashWriter,
    Hashed,
    DefaultHashable
};

 use crate::blake2::{
        State,
        Params
    };

/// Blake2b Hash Function
pub fn blake256(data: &[u8]) -> H256 {
    let mut params = Params::new();
    params.hash_length(32);
    let mut engine = params.to_state();
    engine.update(data);
    let mut result = [0u8; 32];
    let output = engine.finalize();
    result.clone_from_slice(&output.as_bytes());
    H256::from(result)
}


/// Hasher used to build tree @ 256bits
pub struct BlakeHasher {
    state: State
}

impl Default for BlakeHasher {
    fn default() -> BlakeHasher {
        BlakeHasher::new()
    }
}

impl BlakeHasher {
    pub fn new() -> Self {
       
        let mut params = Params::new();
        params.hash_length(32);

        Self {
            state: params.to_state()
        }
    }

    pub fn new_personal(personal: &[u8]) -> Self {
        let mut params = Params::new();
        params.personal(personal);
        params.hash_length(32);

        Self {
            state: params.to_state()
        }
    }

    /// Feed data into Hash State
    #[inline]
    pub fn write(&mut self, msg: &[u8]) {
        self.state.update(msg);
    }

    /// Returns the hash value for the data stream and consumes state.
    #[inline]
    fn finalize(&self) -> H256 {
        let mut result = [0u8; 32];
        let output = self.state.finalize();
        result.clone_from_slice(&output.as_bytes());
        H256::from(result)
    }

}
