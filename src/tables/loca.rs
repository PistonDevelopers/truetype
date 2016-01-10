
use Error;
use Result;
use types::LocationFormat;
use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};

/// A location table.
///
/// The 'loca' table stores the offsets to the locations of the glyphs
/// in the font relative to the beginning of the 'glyf' table.
#[derive(Debug)]
pub struct LOCA {
    offsets: Vec<u32>,
    format: LocationFormat,
}

impl Default for LOCA {
    fn default() -> Self {
        LOCA { offsets: Vec::new(), format: LocationFormat::Short }
    }
}

impl LOCA {
    /// Returns `loca` font table.
    ///
    /// Attempts to read `data` starting from `offset` position.
    /// `glyphs` is a number of glyphs in the font.
    /// `lf` is a location format.
    ///
    /// # Errors
    /// Returns error if there is not enough data to read.
    pub fn from_data(data: &[u8], offset: usize, glyphs: u32, lf: LocationFormat) -> Result<LOCA> {
        if offset >= data.len() {
            return Err(Error::Malformed);
        }

        let count = glyphs + 1;
        let mut loca = LOCA {
            offsets: Vec::with_capacity(count as usize),
            format: lf,
        };

        let mut cursor = Cursor::new(&data[offset..]);
        match loca.format {
            LocationFormat::Short => {
                for _ in 0..count {
                    loca.offsets.push(try!(cursor.read_u16::<BigEndian>()) as u32 * 2);
                }
            },
            LocationFormat::Long => {
                for _ in 0..count {
                    loca.offsets.push(try!(cursor.read_u32::<BigEndian>()));
                }
            },
        }

        Ok(loca)
    }

    #[cfg(test)]
    fn bytes(&self) -> Vec<u8> {
        use byteorder::WriteBytesExt;

        let mut data = vec![];
        match self.format {
            LocationFormat::Short => {
                for offset in &self.offsets {
                    data.write_u16::<BigEndian>((offset / 2) as u16).unwrap();
                }
            },
            LocationFormat::Long => {
                for offset in &self.offsets {
                    data.write_u32::<BigEndian>(*offset).unwrap();
                }
            },
        }
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Error::*;
    use tables::{MAXP, HEAD};
    use types::LocationFormat;
    use expectest::prelude::*;

    #[test]
    fn smoke() {
        let data = ::utils::read_file("tests/Tuffy_Bold.ttf");
        let maxp_offset = ::utils::find_table_offset(&data, 0, b"maxp").unwrap().unwrap();
        let glyphs = MAXP::from_data(&data, maxp_offset).unwrap().num_glyphs();
        let head_offset = ::utils::find_table_offset(&data, 0, b"head").unwrap().unwrap();
        let format = HEAD::from_data(&data, head_offset).unwrap().location_format();

        let size = ((glyphs + 1) * format.entry_size()) as usize;
        let loca_offset = ::utils::find_table_offset(&data, 0, b"loca").unwrap().unwrap();
        let loca = LOCA::from_data(&data, loca_offset, glyphs, format).unwrap();
        assert_eq!(loca.bytes(), &data[loca_offset..loca_offset + size]);

        expect!(LOCA::from_data(&data, data.len(), glyphs, format)).to(be_err().value(Malformed));
    }

    #[test]
    fn loca_format_short() {
        let data = &[0, 50, 0, 100, 0, 200];
        let loca = LOCA::from_data(data, 0, 2, LocationFormat::Short).unwrap();
        expect!(loca.bytes()).to(be_equal_to(data));
        expect!(loca.offsets).to(be_equal_to([50 * 2, 100 * 2, 200 * 2]));
    }

    #[test]
    fn loca_format_long() {
        let data = &[0, 0, 0, 50, 0, 0, 0, 100, 0, 0, 0, 200];
        let loca = LOCA::from_data(data, 0, 2, LocationFormat::Long).unwrap();
        expect!(loca.bytes()).to(be_equal_to(data));
        expect!(loca.offsets).to(be_equal_to([50, 100, 200]));
    }
}
