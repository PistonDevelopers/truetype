
use super::Fixed;
use Error;
use Result;
use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};

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
    pub fn from_data(data: &[u8]) -> Result<HHEA> {
        let mut cursor = Cursor::new(data);
        let version = Fixed(try!(cursor.read_i32::<BigEndian>()));
        if version != Fixed(0x00010000) {
            return Err(Error::HHEAVersionNotSupported);
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
            Err(::Error::HHEAVersionNotSupported) => (),
            _ => panic!("should handle version mismatch"),
        }
    }
}

