
//
// Ristretto Helper Abstraction
//

use curve25519_dalek::ristretto::{CompressedRistretto, RistrettoPoint};
use std::fmt::Debug;


/// Compressed Ristretto point length
pub const RISTRETTO_POINT_LENGTH: usize = 32;

/// A `RistrettoBoth` contains both an uncompressed `RistrettoPoint`
/// as well as the corresponding `CompressedRistretto`.  It provides
/// a convenient middle ground for protocols that both hash compressed
/// points to derive scalars for use with uncompressed points.
#[derive(Copy, Clone, Default, Eq)]  // PartialEq optimnized below
pub struct RistrettoBoth {
    compressed: CompressedRistretto,
    point: RistrettoPoint,
}

impl ::zeroize::Zeroize for RistrettoBoth {
    fn zeroize(&mut self) {
        crate::zeroize_hack(&mut self.compressed);
        crate::zeroize_hack(&mut self.point);
    }
}

impl Debug for RistrettoBoth {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(f, "RistrettoPoint( {:?} )", self.compressed)
    }
}

impl RistrettoBoth {

    const DESCRIPTION : &'static str = "A ristretto point represented as a 32-byte compressed point";

    /// Access the compressed Ristretto form
    pub fn as_compressed(&self) -> &CompressedRistretto { &self.compressed }

    /// Extract the compressed Ristretto form
    pub fn into_compressed(self) -> CompressedRistretto { self.compressed }

    /// Access the point form
    pub fn as_point(&self) -> &RistrettoPoint { &self.point }

    /// Extract the point form
    pub fn into_point(self) -> RistrettoPoint { self.point }

    /// Decompress into the `RistrettoBoth` format that also retains the
    /// compressed form.
    pub fn from_compressed(compressed: CompressedRistretto) -> Option<RistrettoBoth> {
        match compressed.decompress() {
            None => None,
            Some(point) => {
                Some(RistrettoBoth {
                    point,
                    compressed,
                })
            }
        }
    }

    /// Compress into the `RistrettoBoth` format that also retains the
    /// uncompressed form.
    pub fn from_point(point: RistrettoPoint) -> RistrettoBoth {
        RistrettoBoth {
            compressed: point.compress(),
            point,
        }
    }

    /// Convert this point to a byte array.
    #[inline]
    pub fn to_bytes(&self) -> [u8; RISTRETTO_POINT_LENGTH] {
        self.compressed.to_bytes()
    }

     /// Convert this point to a byte array.
    #[inline]
    pub fn as_bytes<'a>(&'a self) -> &'a [u8; RISTRETTO_POINT_LENGTH] {
        self.compressed.as_bytes()
    }

    /// Construct a `RistrettoBoth` from a slice of bytes.
    ///
    /// # Warning
    ///
    /// The caller is responsible for ensuring that the bytes passed into this
    /// method actually represent a `curve25519_dalek::ristretto::CompressedRistretto`
    /// and that said compressed point is actually a point on the curve.
    ///
    /// # Example
    ///
    /// ```
    /// # extern crate mohan;
    /// #
    /// 
    /// use mohan::tools::RistrettoBoth;
    ///
    /// # fn doctest() -> Option<RistrettoBoth> {
    /// let public_key_bytes: [u8; 32] = [
    ///    215,  90, 152,   1, 130, 177,  10, 183, 213,  75, 254, 211, 201, 100,   7,  58,
    ///     14, 225, 114, 243, 218, 166,  35,  37, 175,   2,  26, 104, 247,   7,   81, 26];
    ///
    /// let public_key = RistrettoBoth::from_bytes(&public_key_bytes)?;
    /// #
    /// # Some(public_key)
    /// # }
    /// #
    /// # fn main() {
    /// #     doctest();
    /// # }
    /// ```
    ///
    /// # Returns
    ///
    /// A `Result` whose okay value is an `RistrettoBoth` or whose error value
    /// is an `SignatureError` describing the error that occurred.
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Option<RistrettoBoth> {
        if bytes.len() != RISTRETTO_POINT_LENGTH { return None; }
        let mut compressed = CompressedRistretto([0u8; RISTRETTO_POINT_LENGTH]);
        compressed.0.copy_from_slice(&bytes[..32]);
        RistrettoBoth::from_compressed(compressed)
    }
    
}


/// We hide fields largely so that only compairing the compressed forms works.
impl PartialEq<Self> for RistrettoBoth {
    fn eq(&self, other: &Self) -> bool {
        let r = self.compressed.eq(&other.compressed);
        debug_assert_eq!(r, self.point.eq(&other.point));
        r
    }

    // fn ne(&self, other: &Rhs) -> bool {
    //   self.compressed.0.ne(&other.compressed.0)
    // }
}


impl PartialOrd<RistrettoBoth> for RistrettoBoth {
    fn partial_cmp(&self, other: &RistrettoBoth) -> Option<::core::cmp::Ordering> {
        self.compressed.0.partial_cmp(&other.compressed.0)
    }

    // fn lt(&self, other: &RistrettoBoth) -> bool {
    //    self.compressed.0.lt(&other.compressed.0)
    // }
    // fn le(&self, other: &RistrettoBoth) -> bool {
    //    self.compressed.0.le(&other.compressed.0)
    // }
    // fn gt(&self, other: &RistrettoBoth) -> bool {
    //    self.compressed.0.gt(&other.compressed.0)
    // }
    // fn ge(&self, other: &RistrettoBoth) -> bool {
    //    self.compressed.0.ge(&other.compressed.0)
    // }
}

impl Ord for RistrettoBoth {
    fn cmp(&self, other: &Self) -> ::core::cmp::Ordering {
        self.compressed.0.cmp(&other.compressed.0)
    }

    // fn max(self, other: Self) -> Self {
    //    self.compressed.0.max(other.compressed.0)
    // }
    // fn min(self, other: Self) -> Self {
    //    self.compressed.0.min(other.compressed.0)
    // }
}

impl ::core::hash::Hash for RistrettoBoth {
    fn hash<H: ::core::hash::Hasher>(&self, state: &mut H) {
        self.compressed.0.hash(state);
    }
}