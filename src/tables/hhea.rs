
use super::Fixed;
use Error;
use Result;
use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};

/// A horizontal header.
///
/// This table contains information needed to layout fonts whose characters
/// are written horizontally, that is, either left to right or right to left.
///
/// The table provides such properties as: `ascent`, `descent` and `line_gap`,
/// these are expressed in unscaled coordinates, so you must multiply by
/// the scale factor for a given size. You can advance the vertical position by
/// `ascent - descent + line_gap`.
#[derive(Debug, Default)]
pub struct HHEA {
    version: Fixed,
    ascent: i16,
    descent: i16,
    line_gap: i16,
    advance_width_max: u16,
    min_left_side_bearing: i16,
    min_right_side_bearing: i16,
    x_max_extent: i16,
    caret_slope_rise: i16,
    caret_slope_run: i16,
    caret_offset: i16,
    reserved1: i16,
    reserved2: i16,
    reserved3: i16,
    reserved4: i16,
    metric_data_format: i16,
    num_of_long_hor_metrics: u16,
}

impl HHEA {
    /// Returns `hhea` font table.
    ///
    /// Attempts to read `data` starting from zero position, so you should
    /// provide a correct offset.
    ///
    /// # Errors
    /// Returns error if there is not enough data to read or version of
    /// the `hhea` font table is not supported.
    pub fn from_data(data: &[u8]) -> Result<HHEA> {
        let mut cursor = Cursor::new(data);
        let version = Fixed(try!(cursor.read_i32::<BigEndian>()));
        if version != Fixed(0x00010000) {
            return Err(Error::HHEAVersionIsNotSupported);
        }

        let mut hhea = HHEA::default();
        hhea.version = version;
        hhea.ascent = try!(cursor.read_i16::<BigEndian>());
        hhea.descent = try!(cursor.read_i16::<BigEndian>());
        hhea.line_gap = try!(cursor.read_i16::<BigEndian>());
        hhea.advance_width_max = try!(cursor.read_u16::<BigEndian>());
        hhea.min_left_side_bearing = try!(cursor.read_i16::<BigEndian>());
        hhea.min_right_side_bearing = try!(cursor.read_i16::<BigEndian>());
        hhea.x_max_extent = try!(cursor.read_i16::<BigEndian>());
        hhea.caret_slope_rise = try!(cursor.read_i16::<BigEndian>());
        hhea.caret_slope_run = try!(cursor.read_i16::<BigEndian>());
        hhea.caret_offset = try!(cursor.read_i16::<BigEndian>());
        hhea.reserved1 = try!(cursor.read_i16::<BigEndian>());
        hhea.reserved2 = try!(cursor.read_i16::<BigEndian>());
        hhea.reserved3 = try!(cursor.read_i16::<BigEndian>());
        hhea.reserved4 = try!(cursor.read_i16::<BigEndian>());
        hhea.metric_data_format = try!(cursor.read_i16::<BigEndian>());
        hhea.num_of_long_hor_metrics = try!(cursor.read_u16::<BigEndian>());

        Ok(hhea)
    }

    #[cfg(test)]
    fn bytes(&self) -> Vec<u8> {
        use byteorder::WriteBytesExt;

        let mut data = vec![];
        data.write_i32::<BigEndian>(self.version.0).unwrap();
        data.write_i16::<BigEndian>(self.ascent).unwrap();
        data.write_i16::<BigEndian>(self.descent).unwrap();
        data.write_i16::<BigEndian>(self.line_gap).unwrap();
        data.write_u16::<BigEndian>(self.advance_width_max).unwrap();
        data.write_i16::<BigEndian>(self.min_left_side_bearing).unwrap();
        data.write_i16::<BigEndian>(self.min_right_side_bearing).unwrap();
        data.write_i16::<BigEndian>(self.x_max_extent).unwrap();
        data.write_i16::<BigEndian>(self.caret_slope_rise).unwrap();
        data.write_i16::<BigEndian>(self.caret_slope_run).unwrap();
        data.write_i16::<BigEndian>(self.caret_offset).unwrap();
        data.write_i16::<BigEndian>(self.reserved1).unwrap();
        data.write_i16::<BigEndian>(self.reserved2).unwrap();
        data.write_i16::<BigEndian>(self.reserved3).unwrap();
        data.write_i16::<BigEndian>(self.reserved4).unwrap();
        data.write_i16::<BigEndian>(self.metric_data_format).unwrap();
        data.write_u16::<BigEndian>(self.num_of_long_hor_metrics).unwrap();
        data
    }

    /// Distance from baseline of highest ascender.
    pub fn ascent(&self) -> i32 {
        self.ascent as i32
    }

    /// Distance from baseline of lowest descender (i.e. it is typically negative).
    pub fn descent(&self) -> i32 {
        self.descent as i32
    }

    /// The spacing between one row's descent and the next row's ascent.
    #[allow(dead_code)]
    pub fn line_gap(&self) -> i32 {
        self.line_gap as i32
    }

    /// The number of advance widths in metrics table.
    pub fn num_of_long_hor_metrics(&self) -> u32 {
        self.num_of_long_hor_metrics as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OFFSET: usize = 340;
    const SIZE: usize = 16 * 2 + 4;

    #[test]
    fn runner() {
        let data = super::super::read_file("tests/Tuffy_Bold.ttf");
        test_read_write(&data);
        test_version_mismatch(&data);
        test_read_not_enough_data(&data);
    }

    fn test_read_write(data: &[u8]) {
        let data = &data[OFFSET..OFFSET + SIZE];
        let hhea = HHEA::from_data(data).unwrap();
        assert_eq!(hhea.bytes(), data);
    }

    fn test_version_mismatch(data: &[u8]) {
        let mut data = data.to_owned();
        data[1] = 2;
        match HHEA::from_data(&data) {
            Err(::Error::HHEAVersionIsNotSupported) => (),
            _ => panic!("should return error on version mismatch"),
        }
    }

    fn test_read_not_enough_data(data: &[u8]) {
        match HHEA::from_data(&data[..SIZE - 1]) {
            Err(_) => (),
            _ => panic!("should return error if not enough data"),
        }
    }
}

