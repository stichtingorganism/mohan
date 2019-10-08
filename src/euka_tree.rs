//! Euka Merkle Hash Tree Varient

use std::hash::Hasher;
use crate::{
    merkle::{
        MerkleTree,
        VecStore,
        Algorithm,
        Hashable
    },
    hash::H256,
    blake2::{
        State,
        Params
    }
};

/// Convenient Wrapper 
pub type MukaTree = MerkleTree<H256, BlakeBackend, VecStore<H256>>;

/// Hasher used to build tree @ 256bits
pub struct BlakeBackend {
    state: State
}

impl BlakeBackend {
    pub fn new() -> Self {
        let mut params = Params::new();
        params.personal(b"MukaTree");
        params.hash_length(32);

        Self {
            state: params.to_state()
        }
    }
}

impl Default for BlakeBackend {
    fn default() -> BlakeBackend {
        BlakeBackend::new()
    }
}

impl Hasher for BlakeBackend {

    /// Feed data into Hash State
    #[inline]
    fn write(&mut self, msg: &[u8]) {
        self.state.update(msg);
    }

    /// Returns the hash value for the data stream and consumes state.
    #[inline]
    fn finish(&self) -> u64 {
        unimplemented!()
    }
}

impl Algorithm<H256> for BlakeBackend {
    #[inline]
    fn hash(&mut self) -> H256 {
        let mut result = [0u8; 32];
        let output = self.state.finalize();
        result.clone_from_slice(&output.as_bytes());
        H256::from(result)
    }

    /// Reset Hasher state.
    #[inline]
    fn reset(&mut self) {
        *self = BlakeBackend::default()
    }
    
}