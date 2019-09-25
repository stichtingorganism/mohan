// Rust Bitcoin Library
// Written in 2014 by
//     Andrew Poelstra <apoelstra@wpsoftware.net>
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

//! BTC Style VarInt

use crate::ser::{
	ProtocolVersion, Readable, Reader, Writeable, Writer, Error, ser_vec, AsFixedBytes
};

/// A variable-length unsigned integer
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct VarInt(pub u64);


impl VarInt {
    /// Gets the length of this VarInt when encoded.
    /// Returns 1 for 0...0xFC, 3 for 0xFD...(2^16-1), 5 for 0x10000...(2^32-1),
    /// and 9 otherwise.
    #[inline]
    pub fn len(&self) -> usize {
        match self.0 {
            0...0xFC             => { 1 }
            0xFD...0xFFFF        => { 3 }
            0x10000...0xFFFFFFFF => { 5 }
            _                    => { 9 }
        }
    }

    #[inline]
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

// //TODO chheck if this cause issues 
// impl std::convert::AsRef<[u8]> for VarInt {
// 	fn as_ref(&self) -> &[u8] {
//         &[1u8]
// 	}
// }

// impl AsFixedBytes for VarInt {
// 	fn len(&self) -> usize {
// 		self.len()
// 	}
// }



impl Readable for VarInt {
	fn read(reader: &mut dyn Reader) -> Result<VarInt, Error> {
		
        let n = reader.read_u8()?;

        match n {
            0xFF => {
                let x = reader.read_u64()?;
                if x < 0x100000000 {
                    Err(Error::InvalidVarInt)
                } else {
                    Ok(VarInt(x))
                }
            }
            0xFE => {
                let x = reader.read_u32()?;
                if x < 0x10000 {
                    Err(Error::InvalidVarInt)
                } else {
                    Ok(VarInt(x as u64))
                }
            }
            0xFD => {
                let x = reader.read_u16()?;

                if x < 0xFD {
                    Err(Error::InvalidVarInt)
                } else {
                    Ok(VarInt(x as u64))
                }
            }
            n => Ok(VarInt(n as u64))
        }
    }
	
}

impl Writeable for VarInt {
	fn write<W: Writer>(&self, writer: &mut W) -> Result<(), Error> {
		match self.0 {
            0...0xFC => {
                writer.write_u8(self.0 as u8)?;
                Ok(())
            },
            0xFD...0xFFFF => {
                writer.write_u8(0xFD)?;
                writer.write_u16(self.0 as u16)?;
                Ok(())
            },
            0x10000...0xFFFFFFFF => {
                writer.write_u8(0xFE)?;
                writer.write_u32(self.0 as u32)?;
                Ok(())
            },
            _ => {
                writer.write_u8(0xFF)?;
                writer.write_u64(self.0 as u64)?;
                Ok(())
            },
        }
    }
}


#[test]
fn serialize_varint_test() {
    assert_eq!(ser_vec(&VarInt(10), ProtocolVersion::local()).unwrap(), vec![10u8]);
    assert_eq!(ser_vec(&VarInt(0xFC), ProtocolVersion::local()).unwrap(), vec![0xFCu8]);
    assert_eq!(ser_vec(&VarInt(0xFD), ProtocolVersion::local()).unwrap(), vec![0xFDu8, 0xFD, 0]);
    assert_eq!(ser_vec(&VarInt(0xFFF), ProtocolVersion::local()).unwrap(), vec![0xFDu8, 0xFF, 0xF]);
    assert_eq!(ser_vec(&VarInt(0xF0F0F0F), ProtocolVersion::local()).unwrap(), vec![0xFEu8, 0xF, 0xF, 0xF, 0xF]);
    assert_eq!(ser_vec(&VarInt(0xF0F0F0F0F0E0), ProtocolVersion::local()).unwrap(), vec![0xFFu8, 0xE0, 0xF0, 0xF0, 0xF0, 0xF0, 0xF0, 0, 0]);
}