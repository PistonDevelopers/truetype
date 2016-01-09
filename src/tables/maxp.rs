
use types::Fixed;
use Error;
use Result;
use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};

/// A maximum profile.
///
/// The 'maxp' table establishes the memory requirements for a font.
/// TODO: implement parsing of 1.0 version of the table.
#[derive(Debug, Default)]
pub struct MAXP {
    version: Fixed,
    num_glyphs: u16,
}

impl MAXP {
    /// Returns `maxp` font table.
    ///
    /// Attempts to read `data` starting from `offset` position.
    ///
    /// # Errors
    /// Returns error if there is not enough data to read or version of
    /// the `maxp` font table is not supported.
    pub fn from_data(data: &[u8], offset: usize) -> Result<MAXP> {
        if offset >= data.len() {
            return Err(Error::Malformed);
        }

        let mut cursor = Cursor::new(&data[offset..]);
        let version = Fixed(try!(cursor.read_i32::<BigEndian>()));
        match version {
            Fixed(0x00010000) | Fixed(0x00005000) => {
                let mut maxp = MAXP::default();
                maxp.version = version;
                maxp.num_glyphs = try!(cursor.read_u16::<BigEndian>());
                Ok(maxp)
            },
            _ => Err(Error::MAXPVersionIsNotSupported),
        }
    }

    #[cfg(test)]
    fn bytes(&self) -> Vec<u8> {
        use byteorder::WriteBytesExt;

        let mut data = vec![];
        data.write_i32::<BigEndian>(self.version.0).unwrap();
        data.write_u16::<BigEndian>(self.num_glyphs).unwrap();
        data
    }

    /// Returns the number of glyphs in the font.
    pub fn num_glyphs(&self) -> u32 {
        self.num_glyphs as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Error::*;
    use expectest::prelude::*;

    const SIZE: usize = 4 + 2;

    #[test]
    fn smoke() {
        let data = ::utils::read_file("tests/Tuffy_Bold.ttf");
        let offset = ::utils::find_table_offset(&data, 0, b"maxp").unwrap().unwrap();

        let maxp = MAXP::from_data(&data, offset).unwrap();
        assert_eq!(maxp.bytes(), &data[offset..offset + SIZE]);

        let maxp = MAXP::default();
        expect!(MAXP::from_data(&maxp.bytes(), 0)).to(be_err().value(MAXPVersionIsNotSupported));

        expect!(MAXP::from_data(&data, data.len())).to(be_err().value(Malformed));
    }
}
