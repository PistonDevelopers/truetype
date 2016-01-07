
use super::Fixed;
use Error;
use Result;
use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};

/// A font header.
///
/// The 'head' table contains global information about the font.
#[derive(Debug, Default)]
pub struct HEAD {
    version: Fixed,
    font_revision: Fixed,
    check_sum_adjustment: u32,
    magic_number: u32,
    flags: u16,
    units_per_em: u16,
    created: i64,
    modified: i64,
    x_min: i16,
    y_min: i16,
    x_max: i16,
    y_max: i16,
    mac_style: u16,
    lowest_rec_ppem: u16,
    font_direction_hint: i16,
    index_to_loc_format: i16,
    glyph_data_format: i16,
}

impl HEAD {
    /// Returns `head` font table.
    ///
    /// Attempts to read `data` starting from `offset` position.
    ///
    /// # Errors
    /// Returns error if there is not enough data to read or version of
    /// the `head` font table is not supported.
    pub fn from_data(data: &[u8], offset: usize) -> Result<HEAD> {
        if offset >= data.len() {
            return Err(Error::Malformed);
        }

        let mut cursor = Cursor::new(&data[offset..]);
        let version = Fixed(try!(cursor.read_i32::<BigEndian>()));
        if version != Fixed(0x00010000) {
            return Err(Error::HEADVersionIsNotSupported);
        }

        let mut head = HEAD::default();
        head.version = version;
        head.font_revision = Fixed(try!(cursor.read_i32::<BigEndian>()));
        head.check_sum_adjustment = try!(cursor.read_u32::<BigEndian>());
        head.magic_number = try!(cursor.read_u32::<BigEndian>());
        head.flags = try!(cursor.read_u16::<BigEndian>());
        head.units_per_em = try!(cursor.read_u16::<BigEndian>());
        head.created = try!(cursor.read_i64::<BigEndian>());
        head.modified = try!(cursor.read_i64::<BigEndian>());
        head.x_min = try!(cursor.read_i16::<BigEndian>());
        head.y_min = try!(cursor.read_i16::<BigEndian>());
        head.x_max = try!(cursor.read_i16::<BigEndian>());
        head.y_max = try!(cursor.read_i16::<BigEndian>());
        head.mac_style = try!(cursor.read_u16::<BigEndian>());
        head.lowest_rec_ppem = try!(cursor.read_u16::<BigEndian>());
        head.font_direction_hint = try!(cursor.read_i16::<BigEndian>());
        head.index_to_loc_format = try!(cursor.read_i16::<BigEndian>());
        head.glyph_data_format = try!(cursor.read_i16::<BigEndian>());

        Ok(head)
    }

    #[cfg(test)]
    fn bytes(&self) -> Vec<u8> {
        use byteorder::WriteBytesExt;

        let mut data = vec![];
        data.write_i32::<BigEndian>(self.version.0).unwrap();
        data.write_i32::<BigEndian>(self.font_revision.0).unwrap();
        data.write_u32::<BigEndian>(self.check_sum_adjustment).unwrap();
        data.write_u32::<BigEndian>(self.magic_number).unwrap();
        data.write_u16::<BigEndian>(self.flags).unwrap();
        data.write_u16::<BigEndian>(self.units_per_em).unwrap();
        data.write_i64::<BigEndian>(self.created).unwrap();
        data.write_i64::<BigEndian>(self.modified).unwrap();
        data.write_i16::<BigEndian>(self.x_min).unwrap();
        data.write_i16::<BigEndian>(self.y_min).unwrap();
        data.write_i16::<BigEndian>(self.x_max).unwrap();
        data.write_i16::<BigEndian>(self.y_max).unwrap();
        data.write_u16::<BigEndian>(self.mac_style).unwrap();
        data.write_u16::<BigEndian>(self.lowest_rec_ppem).unwrap();
        data.write_i16::<BigEndian>(self.font_direction_hint).unwrap();
        data.write_i16::<BigEndian>(self.index_to_loc_format).unwrap();
        data.write_i16::<BigEndian>(self.glyph_data_format).unwrap();
        data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OFFSET: usize = 284;
    const SIZE: usize = 4 * 4 + 2 * 2 + 8 * 2 + 2 * 9;

    #[test]
    fn runner() {
        let data = super::super::read_file("tests/Tuffy_Bold.ttf");
        test_read_write(&data);
        test_version_mismatch(&data);
        test_read_malformed(&data);
    }

    fn test_read_write(data: &[u8]) {
        let head = HEAD::from_data(data, OFFSET).unwrap();
        assert_eq!(head.bytes(), &data[OFFSET..OFFSET + SIZE]);
    }

    fn test_version_mismatch(data: &[u8]) {
        let mut head = HEAD::from_data(data, OFFSET).unwrap();
        head.version = super::super::Fixed(0);
        match HEAD::from_data(&head.bytes(), 0) {
            Err(::Error::HEADVersionIsNotSupported) => (),
            _ => panic!("should return error on version mismatch"),
        }
    }

    fn test_read_malformed(data: &[u8]) {
        match HEAD::from_data(data, data.len()) {
            Err(::Error::Malformed) => (),
            _ => panic!("should return error on malformed data"),
        }
    }
}
