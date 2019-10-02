//! Storage of the Merkle Tree

use std::ops::Index;

/// Element stored in the merkle tree.
pub trait Element: Ord + Clone + AsRef<[u8]> + Sync + Send + Default + std::fmt::Debug {
    /// Returns the length of an element when serialized as a byte slice.
    fn byte_len() -> usize;

    /// Creates the element from its byte form. Panics if the slice is not appropriately sized.
    fn from_slice(bytes: &[u8]) -> Self;

    fn copy_to_slice(&self, bytes: &mut [u8]);
}


/// Backing store of the merkle tree.
pub trait Store<E: Element>:
    std::ops::Deref<Target = [E]> + std::fmt::Debug + Clone + Send + Sync
{
    /// Creates a new store which can store up to `size` elements.
    fn new(size: usize) -> Result<Self, ()>;

    fn new_from_slice(size: usize, data: &[u8]) -> Self;

    fn write_at(&mut self, el: E, i: usize);

    // Used to reduce lock contention and do the `E` to `u8`
    // conversion in `build` *outside* the lock.
    // `buf` is a slice of converted `E`s and `start` is its
    // position in `E` sizes (*not* in `u8`).
    fn copy_from_slice(&mut self, buf: &[u8], start: usize);

    fn read_at(&self, i: usize) -> E;
    fn read_range(&self, r: std::ops::Range<usize>) -> Vec<E>;
    fn read_into(&self, pos: usize, buf: &mut [u8]);

    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn push(&mut self, el: E);

    // Signal to offload the `data` from memory if possible (`DiskMmapStore`
    // case). When the `data` is read/written again it should be automatically
    // reloaded. This function is only a hint with an optional implementation
    // (its mechanism should be transparent to the user who doesn't need to
    // manually reload).
    // Returns `true` if it was able to comply.
    fn try_offload(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct VecStore<E: Element>(Vec<E>);

impl<E: Element> std::ops::Deref for VecStore<E> {
    type Target = [E];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E: Element> Store<E> for VecStore<E> {
    fn new(size: usize) -> Result<Self, ()> {
        Ok(VecStore(Vec::with_capacity(size)))
    }

    fn write_at(&mut self, el: E, i: usize) {
        if self.0.len() <= i {
            self.0.resize(i + 1, E::default());
        }

        self.0[i] = el;
    }

    // NOTE: Performance regression. To conform with the current API we are
    // unnecessarily converting to and from `&[u8]` in the `VecStore` which
    // already stores `E` (in contrast with the `mmap` versions). We are
    // prioritizing performance for the `mmap` case which will be used in
    // production (`VecStore` is mainly for testing and backwards compatibility).
    fn copy_from_slice(&mut self, buf: &[u8], start: usize) {
        assert_eq!(buf.len() % E::byte_len(), 0);
        let num_elem = buf.len() / E::byte_len();

        if self.0.len() < start + num_elem {
            self.0.resize(start + num_elem, E::default());
        }

        self.0.splice(
            start..start + num_elem,
            buf.chunks_exact(E::byte_len()).map(E::from_slice),
        );
    }

    fn new_from_slice(size: usize, data: &[u8]) -> Self {
        let mut v: Vec<_> = data
            .chunks_exact(E::byte_len())
            .map(E::from_slice)
            .collect();
        let additional = size - v.len();
        v.reserve(additional);

        VecStore(v)
    }

    fn read_at(&self, i: usize) -> E {
        self.0[i].clone()
    }

    fn read_into(&self, i: usize, buf: &mut [u8]) {
        self.0[i].copy_to_slice(buf);
    }

    fn read_range(&self, r: std::ops::Range<usize>) -> Vec<E> {
        self.0.index(r).to_vec()
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn push(&mut self, el: E) {
        self.0.push(el);
    }

    fn try_offload(&self) -> bool {
        false
    }
}

