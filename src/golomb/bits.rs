//! Bit twingler
use std::io;
use std::cmp;

/// Bitwise stream reader
pub struct BitStreamReader<'a> {
    buffer: [u8; 1],
    offset: u8,
    reader: &'a mut dyn io::Read,
}

impl<'a> BitStreamReader<'a> {
    /// Create a new BitStreamReader that reads bitwise from a given reader
    pub fn new(reader: &'a mut dyn io::Read) -> BitStreamReader {
        BitStreamReader {
            buffer: [0u8],
            reader: reader,
            offset: 8,
        }
    }

    /// Read nbit bits
    pub fn read(&mut self, mut nbits: u8) -> Result<u64, io::Error> {
        if nbits > 64 {
            return Err(io::Error::new(io::ErrorKind::Other, "can not read more than 64 bits at once"));
        }
        let mut data = 0u64;
        while nbits > 0 {
            if self.offset == 8 {
                self.reader.read_exact(&mut self.buffer)?;
                self.offset = 0;
            }
            let bits = cmp::min(8 - self.offset, nbits);
            data <<= bits;
            data |= ((self.buffer[0] << self.offset) >> (8 - bits)) as u64;
            self.offset += bits;
            nbits -= bits;
        }
        Ok(data)
    }
}

/// Bitwise stream writer
pub struct BitStreamWriter<'a> {
    buffer: [u8; 1],
    offset: u8,
    writer: &'a mut dyn io::Write,
}

impl<'a> BitStreamWriter<'a> {
    /// Create a new BitStreamWriter that writes bitwise to a given writer
    pub fn new(writer: &'a mut dyn io::Write) -> BitStreamWriter {
        BitStreamWriter {
            buffer: [0u8],
            writer: writer,
            offset: 0,
        }
    }

    /// Write nbits bits from data
    pub fn write(&mut self, data: u64, mut nbits: u8) -> Result<usize, io::Error> {
        if nbits > 64 {
            return Err(io::Error::new(io::ErrorKind::Other, "can not write more than 64 bits at once"));
        }
        let mut wrote = 0;
        
        while nbits > 0 {
            let bits = cmp::min(8 - self.offset, nbits);
            self.buffer[0] |= ((data << (64 - nbits)) >> (64 - 8 + self.offset)) as u8;
            self.offset += bits;
            nbits -= bits;
            if self.offset == 8 {
                wrote += self.flush()?;
            }
        }
        Ok(wrote)
    }

    /// flush bits not yet written
    pub fn flush(&mut self) -> Result<usize, io::Error> {
        if self.offset > 0 {
            self.writer.write_all(&self.buffer)?;
            self.buffer[0] = 0u8;
            self.offset = 0;
            Ok(1)
        } else {
            Ok(0)
        }
    }
}


#[test]
fn test_bit_stream() {
    use std::io::Cursor;

    let mut out = Cursor::new(Vec::new());
    {
        let mut writer = BitStreamWriter::new(&mut out);
        writer.write(0, 1).unwrap(); // 0
        writer.write(2, 2).unwrap(); // 10
        writer.write(6, 3).unwrap(); // 110
        writer.write(11, 4).unwrap(); // 1011
        writer.write(1, 5).unwrap(); // 00001
        writer.write(32, 6).unwrap(); // 100000
        writer.write(7, 7).unwrap(); // 0000111
        writer.flush().unwrap();
    }

    let bytes = out.into_inner();
    assert_eq!("01011010110000110000000001110000", format!("{:08b}{:08b}{:08b}{:08b}", bytes[0], bytes[1], bytes[2], bytes[3]));
    
    {
        let mut input = Cursor::new(bytes);
        let mut reader = BitStreamReader::new(&mut input);
        assert_eq!(reader.read(1).unwrap(), 0);
        assert_eq!(reader.read(2).unwrap(), 2);
        assert_eq!(reader.read(3).unwrap(), 6);
        assert_eq!(reader.read(4).unwrap(), 11);
        assert_eq!(reader.read(5).unwrap(), 1);
        assert_eq!(reader.read(6).unwrap(), 32);
        assert_eq!(reader.read(7).unwrap(), 7);
        // 4 bits remained
        assert!(reader.read(5).is_err());
    }
}