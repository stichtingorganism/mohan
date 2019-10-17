// Rust Bitcoin Library
// Written in 2019 by
//   The rust-bitcoin developers
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//
//!
//! # BIP158 Compact Block Filters for Light Clients
//!

mod bits;
use bits::{
    BitStreamReader,
    BitStreamWriter
};

use std::collections::HashSet;
use failure::Fail;
use std::{cmp, io};
use crate::hash::SipHasher;


/// Golomb encoding parameter as in BIP-158, see also https://gist.github.com/sipa/576d5f09c3b86c3b1b75598d799fc845
pub const P_BIP158: u8 = 19;
pub const M_BIP158: u64 = 784931;

/// Errors that may occur when handling Golomb Coded Sets.
#[derive(Debug, Fail)]
pub enum Error {
    /// Returned when attempting to insert an additional element into an
    /// already full Golomb Coded Set.
    #[fail(display = "Limit for the number of elements has been reached")]
    LimitReached,
    /// The Golomb-Rice encoded sequence of bits could not be decoded, returned
    /// when unpacking or calling the `contains` method on a a packed GCS.
    #[fail(display = "Decoding failed due to invalid Golomb-Rice bit sequence")]
    Decode,
    /// todo
    #[fail(display = "IO error: {}", _0)]
    Io(io::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

/// fast reduction of hash to [0, nm) range
fn map_to_range(hash: u64, nm: u64) -> u64 {
    // Use this once we upgrade to rustc >= 1.26
    // ((hash as u128 * nm as u128) >> 64) as u64

    #[inline]
    fn l(n: u64) -> u64 { n & 0xffffffff }
    #[inline]
    fn h(n: u64) -> u64 { n >> 32 }

    let a = h(hash);
    let b = l(hash);
    let c = h(nm);
    let d = l(nm);

    a * c + h(a * d + c * b + h(b * d))
}


/// Golomb Coded Set Filter
struct GCSFilter {
    k0: u64, // sip hash key
    k1: u64, // sip hash key
    p: u8
}


impl GCSFilter {

    /// Create a new filter
    fn new(k0: u64, k1: u64, p: u8) -> GCSFilter {
        if p == 0 {
            panic!("p cannot be 0");
        }

        GCSFilter { k0, k1, p }
    }

    /// Golomb-Rice encode a number n to a bit stream (Parameter 2^k)
    fn golomb_rice_encode(&self, writer: &mut BitStreamWriter, n: u64) -> Result<usize, io::Error> {
        let mut wrote = 0;
        let mut q = n >> self.p;

        while q > 0 {
            let nbits = cmp::min(q, 64);
            wrote += writer.write(!0u64, nbits as u8)?;
            q -= nbits;
        }

        wrote += writer.write(0, 1)?;
        wrote += writer.write(n, self.p)?;
        Ok(wrote)
    }

    /// Golomb-Rice decode a number from a bit stream (Parameter 2^k)
    fn golomb_rice_decode(&self, reader: &mut BitStreamReader) -> Result<u64, io::Error> {
        let mut q = 0u64;
        while reader.read(1)? == 1 {
            q += 1;
        }
        let r = reader.read(self.p)?;
        return Ok((q << self.p) + r);
    }

    /// Hash an arbitrary slice with siphash using parameters of this filter
    fn hash(&self, element: &[u8]) -> u64 {
        SipHasher::hash_to_u64_with_keys(self.k0, self.k1, element)
    }
}

/// Colomb-Rice encoded filter writer
pub struct GCSFilterWriter<'a> {
    filter: GCSFilter,
    writer: &'a mut dyn io::Write,
    elements: HashSet<Vec<u8>>,
    m: u64
}


impl<'a> GCSFilterWriter<'a> {

    /// Create a new GCS writer wrapping a generic writer, with specific seed to siphash
    pub fn new(writer: &'a mut dyn io::Write, k0: u64, k1: u64, m: u64, p: u8) -> GCSFilterWriter<'a> {
        GCSFilterWriter {
            filter: GCSFilter::new(k0, k1, p),
            writer,
            elements: HashSet::new(),
            m
        }
    }

    /// Add some data to the filter
    pub fn add_element(&mut self, element: &[u8]) {
        if !element.is_empty() {
            self.elements.insert(element.to_vec());
        }
    }

    /// write the filter to the wrapped writer
    pub fn finish(&mut self) -> Result<usize, io::Error> {

        let nm = self.elements.len() as u64 * self.m;

        // map hashes to [0, n_elements * M)
        let mut mapped: Vec<_> = self.elements.iter()
            .map(|e| map_to_range(self.filter.hash(e.as_slice()), nm)).collect();
        mapped.sort();

        // write number of elements as u64
        let mut encoder = io::Cursor::new(Vec::new());
        let varint = mapped.len() as u64;
        //TODO handle unwrap with error
        crate::ser::serialize_default(&mut encoder, &varint).unwrap();

        let mut wrote = self.writer.write(encoder.into_inner().as_slice())?;

        // write out deltas of sorted values into a Golonb-Rice coded bit stream
        let mut writer = BitStreamWriter::new(self.writer);
        let mut last = 0;

        for data in mapped {
            wrote += self.filter.golomb_rice_encode(&mut writer, data - last)?;
            last = data;
        }

        wrote += writer.flush()?;
        Ok(wrote)
    }
}


/// Golomb-Rice encoded filter reader
pub struct GCSFilterReader {
    filter: GCSFilter,
    m: u64
}

impl GCSFilterReader {

    /// Create a new filter reader with specific seed to siphash
    pub fn new(k0: u64, k1: u64, m: u64, p: u8) -> GCSFilterReader {
        GCSFilterReader { filter: GCSFilter::new(k0, k1, p), m }
    }

    /// match any query pattern
    pub fn match_any(&self, reader: &mut dyn io::Read, query: &mut dyn Iterator<Item=&[u8]>) -> Result<bool, Error> {

        let mut decoder = reader;
        //decode len in u64
        let n_elements: u64 = crate::ser::deserialize_default(&mut decoder).unwrap_or(0u64);

        let ref mut reader = decoder;
        // map hashes to [0, n_elements << grp]
        let nm = n_elements * self.m;
        let mut mapped = query.map(|e| map_to_range(self.filter.hash(e), nm)).collect::<Vec<_>>();
        // sort
        mapped.sort();
        if mapped.is_empty() {
            return Ok(true);
        }
        if n_elements == 0 {
            return Ok(false);
        }

        // find first match in two sorted arrays in one read pass
        let mut reader = BitStreamReader::new(reader);
        let mut data = self.filter.golomb_rice_decode(&mut reader)?;
        let mut remaining = n_elements - 1;
        for p in mapped {
            loop {
                if data == p {
                    return Ok(true);
                } else if data < p {
                    if remaining > 0 {
                        data += self.filter.golomb_rice_decode(&mut reader)?;
                        remaining -= 1;
                    } else {
                        return Ok(false);
                    }
                } else {
                    break;
                }
            }
        }
        Ok(false)
    }

    /// match all query pattern
    pub fn match_all(&self, reader: &mut dyn io::Read, query: &mut dyn Iterator<Item=&[u8]>) -> Result<bool, Error> {
        //read length
        let mut decoder = reader;
        let n_elements: u64 = crate::ser::deserialize_default(&mut decoder).unwrap_or(0u64);

        let ref mut reader = decoder;
        // map hashes to [0, n_elements << grp]
        let nm = n_elements * self.m;
        let mut mapped = query.map(|e| map_to_range(self.filter.hash(e), nm)).collect::<Vec<_>>();

        // sort
        mapped.sort();
        mapped.dedup();
        if mapped.is_empty() {
            return Ok(true);
        }

        if n_elements == 0 {
            return Ok(false);
        }

        // figure if all mapped are there in one read pass
        let mut reader = BitStreamReader::new(reader);
        let mut data = self.filter.golomb_rice_decode(&mut reader)?;
        let mut remaining = n_elements - 1;
        for p in mapped {
            loop {
                if data == p {
                    break;
                } else if data < p {
                    if remaining > 0 {
                        data += self.filter.golomb_rice_decode(&mut reader)?;
                        remaining -= 1;
                    } else {
                        return Ok(false);
                    }
                } else {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }
}



#[test]
fn test_filter () {
    use std::io::Cursor;
    
    let mut patterns = HashSet::new();

    patterns.insert(hex::decode("000000").unwrap());
    patterns.insert(hex::decode("111111").unwrap());
    patterns.insert(hex::decode("222222").unwrap());
    patterns.insert(hex::decode("333333").unwrap());
    patterns.insert(hex::decode("444444").unwrap());
    patterns.insert(hex::decode("555555").unwrap());
    patterns.insert(hex::decode("666666").unwrap());
    patterns.insert(hex::decode("777777").unwrap());
    patterns.insert(hex::decode("888888").unwrap());
    patterns.insert(hex::decode("999999").unwrap());
    patterns.insert(hex::decode("aaaaaa").unwrap());
    patterns.insert(hex::decode("bbbbbb").unwrap());
    patterns.insert(hex::decode("cccccc").unwrap());
    patterns.insert(hex::decode("dddddd").unwrap());
    patterns.insert(hex::decode("eeeeee").unwrap());
    patterns.insert(hex::decode("ffffff").unwrap());

    let mut out = Cursor::new(Vec::new());
    {
        let mut writer = GCSFilterWriter::new(&mut out, 0, 0, M_BIP158, P_BIP158);
        for p in &patterns {
            writer.add_element(p.as_slice());
        }

        writer.finish().unwrap();
    }

    let bytes = out.into_inner();
    {
            let mut query = Vec::new();
            query.push(hex::decode("abcdef").unwrap());
            query.push(hex::decode("eeeeee").unwrap());

            let reader = GCSFilterReader::new(0, 0, M_BIP158, P_BIP158);
            let mut input = Cursor::new(bytes.clone());
            assert!(reader.match_any(&mut input, &mut query.iter().map(|v| v.as_slice())).unwrap());
    }
    {
            let mut query = Vec::new();
            query.push(hex::decode("abcdef").unwrap());
            query.push(hex::decode("123456").unwrap());

            let reader = GCSFilterReader::new(0, 0, M_BIP158, P_BIP158);
            let mut input = Cursor::new(bytes.clone());
            assert!(!reader.match_any(&mut input, &mut query.iter().map(|v| v.as_slice())).unwrap());
    }
    {
            let reader = GCSFilterReader::new(0, 0, M_BIP158, P_BIP158);
            let mut query = Vec::new();
            for p in &patterns {
                query.push(p.clone());
            }
            let mut input = Cursor::new(bytes.clone());
            assert!(reader.match_all(&mut input, &mut query.iter().map(|v| v.as_slice())).unwrap());
    }

    {
        let reader = GCSFilterReader::new(0, 0, M_BIP158, P_BIP158);
        let mut query = Vec::new();
        for p in &patterns {
            query.push(p.clone());
        }

        query.push(hex::decode("abcdef").unwrap());
        let mut input = Cursor::new(bytes.clone());
        assert!(!reader.match_all(&mut input, &mut query.iter().map(|v| v.as_slice())).unwrap());
    }
}